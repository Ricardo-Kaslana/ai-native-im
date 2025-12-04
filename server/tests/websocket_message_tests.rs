mod common;

use ai_im_server::services::message::{
    models::{Message, MsgType},
    websocket_handler::WebSocketConfig,
};
use serde_json::json;
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

#[tokio::test]
async fn test_websocket_message_json_structure() {
    let pool = setup_test_pool().await;
    let tx = pool.begin().await.unwrap();

    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();

    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();

    // Test message structure
    let message = json!({
        "type": "message",
        "id": Uuid::new_v4().to_string(),
        "receiver_id": receiver_id.to_string(),
        "content": "Test message",
        "msg_type": 0
    });

    assert!(message.get("type").is_some());
    assert!(message.get("content").is_some());
    assert_eq!(message.get("type").unwrap(), "message");

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_websocket_heartbeat_message() {
    // Test heartbeat message structure
    let heartbeat = json!({
        "type": "heartbeat"
    });

    assert_eq!(heartbeat.get("type").unwrap(), "heartbeat");
}

#[tokio::test]
async fn test_websocket_sync_request() {
    // Test sync request structure
    let sync_request = json!({
        "type": "sync",
        "since_timestamp": 0
    });

    assert_eq!(sync_request.get("type").unwrap(), "sync");
    assert!(sync_request.get("since_timestamp").is_some());
}

#[tokio::test]
async fn test_message_json_serialization() {
    let pool = setup_test_pool().await;
    let tx = pool.begin().await.unwrap();

    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();

    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();

    // Create a message
    let message = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Text,
        "Test message".to_string(),
    );

    // Serialize to JSON
    let json_str = serde_json::to_string(&message).unwrap();
    assert!(json_str.contains(&sender_id.to_string()));
    assert!(json_str.contains(&receiver_id.to_string()));
    assert!(json_str.contains("Test message"));

    // Deserialize back
    let deserialized: Message = serde_json::from_str(&json_str).unwrap();
    assert_eq!(deserialized.sender_id, sender_id);
    assert_eq!(deserialized.receiver_id, Some(receiver_id));
    assert_eq!(deserialized.content, "Test message");

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_websocket_config_defaults() {
    let config = WebSocketConfig::default();

    assert_eq!(config.auth_timeout_secs, 3);
    assert_eq!(config.heartbeat_interval_secs, 30);
    assert_eq!(config.heartbeat_timeout_secs, 10);
    assert_eq!(config.max_message_size, 1024 * 1024);
}

#[tokio::test]
async fn test_websocket_config_custom() {
    let config = WebSocketConfig {
        auth_timeout_secs: 5,
        heartbeat_interval_secs: 60,
        heartbeat_timeout_secs: 15,
        max_message_size: 2 * 1024 * 1024,
    };

    assert_eq!(config.auth_timeout_secs, 5);
    assert_eq!(config.heartbeat_interval_secs, 60);
    assert_eq!(config.heartbeat_timeout_secs, 15);
    assert_eq!(config.max_message_size, 2 * 1024 * 1024);
}

#[tokio::test]
async fn test_message_types_serialization() {
    let pool = setup_test_pool().await;
    let tx = pool.begin().await.unwrap();

    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();

    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();

    // Test different message types
    let text_msg = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Text,
        "Text".to_string(),
    );
    assert_eq!(text_msg.get_msg_type(), MsgType::Text);

    let image_msg = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Image,
        "https://example.com/image.jpg".to_string(),
    );
    assert_eq!(image_msg.get_msg_type(), MsgType::Image);

    let video_msg = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Video,
        "https://example.com/video.mp4".to_string(),
    );
    assert_eq!(video_msg.get_msg_type(), MsgType::Video);

    let emoji_msg = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Emoji,
        "😀".to_string(),
    );
    assert_eq!(emoji_msg.get_msg_type(), MsgType::Emoji);

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_group_message_creation() {
    let pool = setup_test_pool().await;
    let tx = pool.begin().await.unwrap();

    let sender_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();

    create_test_user(&pool, sender_id).await.unwrap();

    let message = Message::new_group(
        sender_id,
        group_id,
        MsgType::Text,
        "Group message".to_string(),
    );

    assert!(message.is_group());
    assert!(!message.is_private());
    assert_eq!(message.group_id, Some(group_id));
    assert_eq!(message.receiver_id, None);

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_private_message_creation() {
    let pool = setup_test_pool().await;
    let tx = pool.begin().await.unwrap();

    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();

    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();

    let message = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Text,
        "Private message".to_string(),
    );

    assert!(message.is_private());
    assert!(!message.is_group());
    assert_eq!(message.receiver_id, Some(receiver_id));
    assert_eq!(message.group_id, None);

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_message_timestamp_generation() {
    let pool = setup_test_pool().await;
    let tx = pool.begin().await.unwrap();

    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();

    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();

    let message1 = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Text,
        "Message 1".to_string(),
    );

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let message2 = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Text,
        "Message 2".to_string(),
    );

    // Verify timestamps are different and in order
    assert!(message1.timestamp < message2.timestamp);

    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_message_id_uniqueness() {
    let pool = setup_test_pool().await;
    let tx = pool.begin().await.unwrap();

    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();

    create_test_user(&pool, sender_id).await.unwrap();
    create_test_user(&pool, receiver_id).await.unwrap();

    let message1 = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Text,
        "Message 1".to_string(),
    );

    let message2 = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Text,
        "Message 2".to_string(),
    );

    // Verify IDs are unique
    assert_ne!(message1.id, message2.id);

    tx.rollback().await.unwrap();
}
