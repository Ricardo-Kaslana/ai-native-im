mod common;

use ai_im_server::services::message::{
    message_persister::{MessagePersister, PostgresMessagePersister},
    models::{Message, MsgType},
};
use sqlx::PgPool;
use uuid::Uuid;

/// Helper to create a test pool
async fn setup_test_pool() -> PgPool {
    common::setup_test_db().await
}

/// Helper to create a test user in the database
async fn create_test_user(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, role) 
         VALUES ($1, $2, $3, $4, 'user') 
         ON CONFLICT (id) DO NOTHING"
    )
    .bind(user_id)
    .bind(format!("test_user_{}", user_id))
    .bind(format!("test_{}@example.com", user_id))
    .bind("$2b$12$dummy_hash")
    .execute(pool)
    .await?;
    Ok(())
}

#[tokio::test]
async fn test_message_persistence_timing() {
    let pool = setup_test_pool().await;
    let mut tx = pool.begin().await.unwrap();
    
    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();
    
    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();
    
    let persister = PostgresMessagePersister::new(pool.clone());
    
    let message = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Text,
        "Test message".to_string(),
    );
    
    let start = std::time::Instant::now();
    let result = persister.persist_message(&message).await;
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Message persistence should succeed");
    assert!(duration.as_millis() < 200, "Persistence should complete within 200ms");
    
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_message_persistence_completeness() {
    let pool = setup_test_pool().await;
    let mut tx = pool.begin().await.unwrap();
    
    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();
    
    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();
    
    let persister = PostgresMessagePersister::new(pool.clone());
    
    let content = "Complete test message with all fields";
    let message = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Text,
        content.to_string(),
    );
    
    let message_id = message.id;
    let timestamp = message.timestamp;
    
    persister.persist_message(&message).await.unwrap();
    
    // Verify all fields are persisted correctly
    let retrieved: (Uuid, Uuid, Option<Uuid>, String, i64) = sqlx::query_as(
        "SELECT id, sender_id, receiver_id, content, timestamp FROM messages WHERE id = $1"
    )
    .bind(message_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(retrieved.0, message_id);
    assert_eq!(retrieved.1, sender_id);
    assert_eq!(retrieved.2, Some(receiver_id));
    assert_eq!(retrieved.3, content);
    assert_eq!(retrieved.4, timestamp);
    
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_offline_message_retrieval() {
    let pool = setup_test_pool().await;
    let mut tx = pool.begin().await.unwrap();
    
    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();
    
    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();
    
    let persister = PostgresMessagePersister::new(pool.clone());
    
    // Insert 3 offline messages with different timestamps
    for i in 0..3 {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let message = Message::new_private(
            sender_id,
            receiver_id,
            MsgType::Text,
            format!("Offline message {}", i),
        );
        persister.persist_message(&message).await.unwrap();
    }
    
    // Retrieve offline messages
    let batch = persister.get_offline_messages(&receiver_id, 0, 10).await.unwrap();
    
    assert_eq!(batch.messages.len(), 3, "Should retrieve all 3 offline messages");
    
    // Verify chronological ordering (oldest first)
    for i in 0..batch.messages.len() - 1 {
        assert!(
            batch.messages[i].timestamp <= batch.messages[i + 1].timestamp,
            "Messages should be in chronological order"
        );
    }
    
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_delivery_status_update() {
    let pool = setup_test_pool().await;
    let mut tx = pool.begin().await.unwrap();
    
    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();
    
    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();
    
    let persister = PostgresMessagePersister::new(pool.clone());
    
    let message = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Text,
        "Test message".to_string(),
    );
    let message_id = message.id;
    
    persister.persist_message(&message).await.unwrap();
    
    // Verify initial status is not delivered
    let (delivered,): (bool,) = sqlx::query_as(
        "SELECT delivered FROM messages WHERE id = $1"
    )
    .bind(message_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!delivered, "Message should initially be undelivered");
    
    // Update delivery status
    persister.mark_messages_delivered(&[message_id]).await.unwrap();
    
    // Verify status is now delivered
    let (delivered,): (bool,) = sqlx::query_as(
        "SELECT delivered FROM messages WHERE id = $1"
    )
    .bind(message_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(delivered, "Message should be marked as delivered");
    
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_group_message_persistence() {
    let pool = setup_test_pool().await;
    let mut tx = pool.begin().await.unwrap();
    
    let sender_id = Uuid::new_v4();
    create_test_user(&pool, sender_id).await.unwrap();
    
    let persister = PostgresMessagePersister::new(pool.clone());
    
    let group_id = Uuid::new_v4();
    let message = Message::new_group(
        sender_id,
        group_id,
        MsgType::Text,
        "Group message".to_string(),
    );
    let message_id = message.id;
    
    let result = persister.persist_message(&message).await;
    assert!(result.is_ok(), "Group message persistence should succeed");
    
    // Verify group_id is stored correctly
    let (stored_group_id,): (Option<Uuid>,) = sqlx::query_as(
        "SELECT group_id FROM messages WHERE id = $1"
    )
    .bind(message_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert_eq!(stored_group_id, Some(group_id));
    
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_different_message_types() {
    let pool = setup_test_pool().await;
    let mut tx = pool.begin().await.unwrap();
    
    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();
    
    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();
    
    let persister = PostgresMessagePersister::new(pool.clone());
    
    // Test TEXT message
    let text_msg = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Text,
        "Text message".to_string(),
    );
    assert!(persister.persist_message(&text_msg).await.is_ok());
    
    // Test IMAGE message
    let image_msg = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Image,
        "https://example.com/image.jpg".to_string(),
    );
    assert!(persister.persist_message(&image_msg).await.is_ok());
    
    // Test VIDEO message
    let video_msg = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Video,
        "https://example.com/video.mp4".to_string(),
    );
    assert!(persister.persist_message(&video_msg).await.is_ok());
    
    // Test EMOJI message
    let emoji_msg = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Emoji,
        "😀".to_string(),
    );
    assert!(persister.persist_message(&emoji_msg).await.is_ok());
    
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_message_deduplication_via_database() {
    let pool = setup_test_pool().await;
    let mut tx = pool.begin().await.unwrap();
    
    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();
    
    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();
    
    let persister = PostgresMessagePersister::new(pool.clone());
    
    let message = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Text,
        "Duplicate test".to_string(),
    );
    let message_id = message.id;
    
    // First insert should succeed
    let result1 = persister.persist_message(&message).await;
    assert!(result1.is_ok());
    
    // Verify message was inserted
    let count1: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM messages WHERE id = $1"
    )
    .bind(message_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count1.0, 1, "Message should be inserted once");
    
    // Second insert with same ID should be idempotent (ON CONFLICT DO NOTHING)
    let result2 = persister.persist_message(&message).await;
    assert!(result2.is_ok(), "Duplicate message should be handled gracefully");
    
    // Verify still only one message exists
    let count2: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM messages WHERE id = $1"
    )
    .bind(message_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count2.0, 1, "Should still have only one message (idempotent)");
    
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_message_exists_check() {
    let pool = setup_test_pool().await;
    let mut tx = pool.begin().await.unwrap();
    
    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();
    
    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();
    
    let persister = PostgresMessagePersister::new(pool.clone());
    
    let message = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Text,
        "Test message".to_string(),
    );
    let message_id = message.id;
    
    // Check before insertion
    let exists_before = persister.message_exists(&message_id).await.unwrap();
    assert!(!exists_before, "Message should not exist before insertion");
    
    // Insert message
    persister.persist_message(&message).await.unwrap();
    
    // Check after insertion
    let exists_after = persister.message_exists(&message_id).await.unwrap();
    assert!(exists_after, "Message should exist after insertion");
    
    tx.rollback().await.unwrap();
}
