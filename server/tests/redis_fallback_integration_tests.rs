use ai_im_server::services::message::{
    message_persister::{MessagePersister, PostgresMessagePersister},
    models::{Message, MsgType},
    redis_fallback::{FallbackConfig, RedisFallbackService, RedisHealth},
};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

mod common;

/// **Feature: message-system, Integration Test for Redis Fallback**
/// **Validates: Requirements 7.5**
/// 
/// This test verifies that when Redis becomes unavailable, the system
/// falls back to database polling and messages are still delivered.
/// 
/// Note: This test requires a running PostgreSQL database.
/// Set DATABASE_URL environment variable to run this test.
#[tokio::test]
#[ignore] // Requires database connection
async fn test_redis_fallback_integration() {
    // Set up database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/ai_im_test".to_string());
    
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Create test user
    let user_id = Uuid::new_v4();
    
    // Insert test user into database
    sqlx::query!(
        r#"
        INSERT INTO users (id, username, email, password_hash, role)
        VALUES ($1, $2, $3, $4, 'user')
        ON CONFLICT (id) DO NOTHING
        "#,
        user_id,
        format!("test_user_{}", user_id),
        format!("test_{}@example.com", user_id),
        "dummy_hash"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert test user");

    // Create persister
    let persister: Arc<dyn MessagePersister> = Arc::new(PostgresMessagePersister::new(pool.clone()));

    // Create fallback service with short polling interval for testing
    let config = FallbackConfig {
        polling_interval_ms: 100,
        health_check_interval_ms: 1000,
        max_messages_per_poll: 10,
    };
    
    let service = RedisFallbackService::with_config(persister.clone(), config);

    // Initially, Redis is considered healthy (no actual Redis connection in this test)
    assert!(service.is_redis_healthy());

    // Simulate Redis becoming unavailable
    service.mark_redis_unhealthy();
    assert!(!service.is_redis_healthy());

    // Create a test message using the helper method
    let test_message = Message::new_private(
        Uuid::new_v4(),
        user_id,
        MsgType::Text,
        "Test message during Redis fallback".to_string(),
    );

    // Persist the message
    persister.persist_message(&test_message).await.expect("Failed to persist message");

    // Track received messages
    let received_messages = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let received_clone = received_messages.clone();

    // Start polling for the user
    service.start_polling_for_user(user_id, move |msg| {
        let received = received_clone.clone();
        tokio::spawn(async move {
            let mut msgs = received.lock().await;
            msgs.push(msg);
        });
    }).await;

    // Wait for polling to retrieve the message
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Stop polling
    service.stop_polling();

    // Wait for polling to stop
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify that the message was received via fallback polling
    let msgs = received_messages.lock().await;
    assert!(
        msgs.iter().any(|m| m.id == test_message.id),
        "Message should be received via database polling fallback"
    );

    // Verify polling stopped
    assert!(!service.is_polling_active());

    // Simulate Redis recovery
    service.mark_redis_healthy();
    assert!(service.is_redis_healthy());

    // Clean up test data
    sqlx::query!("DELETE FROM messages WHERE id = $1", test_message.id)
        .execute(&pool)
        .await
        .ok();
    
    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .ok();
}

/// Test that fallback service correctly detects Redis health status
#[tokio::test]
#[ignore] // Requires database connection
async fn test_redis_health_check() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/ai_im_test".to_string());
    
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let persister: Arc<dyn MessagePersister> = Arc::new(PostgresMessagePersister::new(pool));
    let service = RedisFallbackService::new(persister);

    // Without Redis connection, health check should return Unknown
    let health = service.check_redis_health().await;
    assert_eq!(health, RedisHealth::Unknown);

    // Manual health status changes
    service.mark_redis_unhealthy();
    assert!(!service.is_redis_healthy());

    service.mark_redis_healthy();
    assert!(service.is_redis_healthy());
}

/// Test that polling can be started and stopped correctly
#[tokio::test]
#[ignore] // Requires database connection
async fn test_polling_lifecycle() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/ai_im_test".to_string());
    
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let user_id = Uuid::new_v4();
    
    // Insert test user
    sqlx::query!(
        r#"
        INSERT INTO users (id, username, email, password_hash, role)
        VALUES ($1, $2, $3, $4, 'user')
        ON CONFLICT (id) DO NOTHING
        "#,
        user_id,
        format!("test_user_{}", user_id),
        format!("test_{}@example.com", user_id),
        "dummy_hash"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert test user");

    let persister: Arc<dyn MessagePersister> = Arc::new(PostgresMessagePersister::new(pool.clone()));
    
    let config = FallbackConfig {
        polling_interval_ms: 50,
        health_check_interval_ms: 1000,
        max_messages_per_poll: 10,
    };
    
    let service = RedisFallbackService::with_config(persister, config);

    // Mark Redis as unhealthy to enable polling
    service.mark_redis_unhealthy();

    // Start polling
    service.start_polling_for_user(user_id, |_msg| {
        // No-op handler
    }).await;

    // Wait for polling to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(service.is_polling_active());

    // Stop polling
    service.stop_polling();
    
    // Wait for polling to stop
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(!service.is_polling_active());

    // Clean up
    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .ok();
}
