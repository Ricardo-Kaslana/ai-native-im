mod common;

use ai_im_server::services::message::{
    connection_manager::{ConnectionManager, InMemoryConnectionManager},
    dedup_cache::{DedupCache, InMemoryDedupCache},
    message_persister::{MessagePersister, PostgresMessagePersister},
    message_router::{MessageRouter, MessageRouterImpl},
    message_validator::MessageValidator,
    models::{CreateMessage, Message, MsgType},
    redis_client::{RedisClient, RedisClientTrait},
};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
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
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(user_id)
    .bind(format!("test_user_{}", user_id))
    .bind(format!("test_{}@example.com", user_id))
    .bind("$2b$12$dummy_hash")
    .execute(pool)
    .await?;
    Ok(())
}

/// Setup message router with all dependencies
async fn setup_message_router(pool: PgPool) -> (Arc<MessageRouterImpl>, Arc<InMemoryConnectionManager>) {
    let connection_manager = Arc::new(InMemoryConnectionManager::new());
    let persister: Arc<dyn MessagePersister> = Arc::new(PostgresMessagePersister::new(pool));
    let dedup_cache = Arc::new(InMemoryDedupCache::new(Duration::from_secs(300)));

    let router = Arc::new(MessageRouterImpl::new(
        connection_manager.clone(),
        dedup_cache,
        persister,
    ));
    
    (router, connection_manager)
}

#[tokio::test]
async fn test_route_private_message_to_online_user() {
    let pool = setup_test_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();

    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();

    let (router, connection_manager) = setup_message_router(pool.clone()).await;

    // Register receiver as online
    let (tx_msg, mut rx_msg) = mpsc::unbounded_channel();
    connection_manager
        .register_connection_async(receiver_id, "conn_1".to_string(), tx_msg)
        .await;

    // Create and route message
    let message = CreateMessage {
        id: Some(Uuid::new_v4()),
        sender_id,
        receiver_id: Some(receiver_id),
        group_id: None,
        msg_type: MsgType::Text,
        content: "Hello, online user!".to_string(),
        is_ai_generated: false,
    };

    let ack = router.route_message(message.clone()).await.unwrap();
    assert_eq!(ack.status, ai_im_server::services::message::AckStatus::Success);

    // Verify message was delivered to online user
    let delivered = tokio::time::timeout(Duration::from_secs(1), rx_msg.recv()).await;
    assert!(
        delivered.is_ok(),
        "Online user should receive message immediately"
    );

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_route_private_message_to_offline_user() {
    let pool = setup_test_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();

    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();

    let (router, _connection_manager) = setup_message_router(pool.clone()).await;

    // Receiver is NOT online (no connection registered)

    // Create and route message
    let message = CreateMessage {
        id: Some(Uuid::new_v4()),
        sender_id,
        receiver_id: Some(receiver_id),
        group_id: None,
        msg_type: MsgType::Text,
        content: "Hello, offline user!".to_string(),
        is_ai_generated: false,
    };

    let message_id = message.id.unwrap();
    let ack = router.route_message(message).await.unwrap();
    assert_eq!(ack.status, ai_im_server::services::message::AckStatus::Success);

    // Verify message was persisted for offline delivery
    let persister = PostgresMessagePersister::new(pool.clone());
    let offline_batch = persister
        .get_offline_messages(&receiver_id, 0, 10)
        .await
        .unwrap();

    assert_eq!(offline_batch.messages.len(), 1);
    assert_eq!(offline_batch.messages[0].id, message_id);
    assert_eq!(offline_batch.messages[0].content, "Hello, offline user!");

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_route_group_message() {
    let pool = setup_test_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let sender_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();

    create_test_user(&pool, sender_id).await.unwrap();

    let (router, _connection_manager) = setup_message_router(pool.clone()).await;

    // Create and route group message
    let message = CreateMessage {
        id: Some(Uuid::new_v4()),
        sender_id,
        receiver_id: None,
        group_id: Some(group_id),
        msg_type: MsgType::Text,
        content: "Hello, group!".to_string(),
        is_ai_generated: false,
    };

    let message_id = message.id.unwrap();
    let ack = router.route_message(message).await.unwrap();
    assert_eq!(ack.status, ai_im_server::services::message::AckStatus::Success);

    // Verify message was persisted
    let persister = PostgresMessagePersister::new(pool.clone());
    let exists = persister.message_exists(&message_id).await.unwrap();
    assert!(exists, "Group message should be persisted");

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_message_deduplication_in_routing() {
    let pool = setup_test_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();

    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();

    let (router, _connection_manager) = setup_message_router(pool.clone()).await;

    let message_id = Uuid::new_v4();

    // Create message
    let message = CreateMessage {
        id: Some(message_id),
        sender_id,
        receiver_id: Some(receiver_id),
        group_id: None,
        msg_type: MsgType::Text,
        content: "Duplicate test".to_string(),
        is_ai_generated: false,
    };

    // Route message first time
    let ack1 = router.route_message(message.clone()).await.unwrap();
    assert_eq!(ack1.status, ai_im_server::services::message::AckStatus::Success);

    // Route same message again (should be deduplicated)
    let ack2 = router.route_message(message).await.unwrap();
    assert_eq!(
        ack2.status,
        ai_im_server::services::message::AckStatus::Duplicate
    );

    // Verify only one message in database
    let persister = PostgresMessagePersister::new(pool.clone());
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages WHERE id = $1")
        .bind(message_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(count.0, 1, "Should have only one message (deduplicated)");

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_invalid_message_validation() {
    let pool = setup_test_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let sender_id = Uuid::new_v4();
    create_test_user(&pool, sender_id).await.unwrap();

    let (router, _connection_manager) = setup_message_router(pool.clone()).await;

    // Message with no receiver and no group (invalid)
    let invalid_message = CreateMessage {
        id: Some(Uuid::new_v4()),
        sender_id,
        receiver_id: None,
        group_id: None,
        msg_type: MsgType::Text,
        content: "Invalid message".to_string(),
        is_ai_generated: false,
    };

    let result = router.route_message(invalid_message).await;
    assert!(result.is_err(), "Should reject invalid message");

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_empty_content_validation() {
    let pool = setup_test_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();

    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();

    let (router, _connection_manager) = setup_message_router(pool.clone()).await;

    // Message with empty content
    let empty_message = CreateMessage {
        id: Some(Uuid::new_v4()),
        sender_id,
        receiver_id: Some(receiver_id),
        group_id: None,
        msg_type: MsgType::Text,
        content: "".to_string(),
        is_ai_generated: false,
    };

    let result = router.route_message(empty_message).await;
    assert!(result.is_err(), "Should reject empty content");

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_multiple_messages_ordering() {
    let pool = setup_test_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();

    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();

    let (router, _connection_manager) = setup_message_router(pool.clone()).await;

    // Send multiple messages
    for i in 0..5 {
        let message = CreateMessage {
            id: Some(Uuid::new_v4()),
            sender_id,
            receiver_id: Some(receiver_id),
            group_id: None,
            msg_type: MsgType::Text,
            content: format!("Message {}", i),
            is_ai_generated: false,
        };

        let ack = router.route_message(message).await.unwrap();
        assert_eq!(ack.status, ai_im_server::services::message::AckStatus::Success);

        // Small delay to ensure timestamp ordering
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // Verify all messages are persisted in order
    let persister = PostgresMessagePersister::new(pool.clone());
    let offline_batch = persister
        .get_offline_messages(&receiver_id, 0, 10)
        .await
        .unwrap();

    assert_eq!(offline_batch.messages.len(), 5);

    // Verify chronological order
    for i in 0..offline_batch.messages.len() - 1 {
        assert!(
            offline_batch.messages[i].timestamp <= offline_batch.messages[i + 1].timestamp,
            "Messages should be in chronological order"
        );
    }

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_different_message_types_routing() {
    let pool = setup_test_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();

    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();

    let (router, _connection_manager) = setup_message_router(pool.clone()).await;

    // Test different message types
    let message_types = vec![
        (MsgType::Text, "Hello"),
        (MsgType::Image, "https://example.com/image.jpg"),
        (MsgType::Video, "https://example.com/video.mp4"),
        (MsgType::Emoji, "😀"),
    ];

    for (msg_type, content) in message_types {
        let message = CreateMessage {
            id: Some(Uuid::new_v4()),
            sender_id,
            receiver_id: Some(receiver_id),
            group_id: None,
            msg_type,
            content: content.to_string(),
            is_ai_generated: false,
        };

        let ack = router.route_message(message).await.unwrap();
        assert_eq!(
            ack.status,
            ai_im_server::services::message::AckStatus::Success,
            "Should route {:?} message",
            msg_type
        );
    }

    // Verify all messages are persisted
    let persister = PostgresMessagePersister::new(pool.clone());
    let offline_batch = persister
        .get_offline_messages(&receiver_id, 0, 10)
        .await
        .unwrap();

    assert_eq!(offline_batch.messages.len(), 4);

    tx.rollback().await.unwrap();
}
