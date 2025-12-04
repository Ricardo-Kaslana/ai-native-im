mod common;

use ai_im_server::services::message::{
    redis_client::{RedisClient, RedisClientTrait},
};
use redis::Client as RedisClientLib;
use std::time::Duration;

/// Helper to create a test Redis client
async fn setup_redis_client() -> RedisClient {
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    RedisClient::new(&redis_url)
        .await
        .expect("Failed to connect to Redis")
}

/// Helper to flush Redis database
async fn flush_redis() {
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    let client = RedisClientLib::open(redis_url).unwrap();
    let mut conn = client.get_connection().unwrap();
    let _: () = redis::cmd("FLUSHDB").query(&mut conn).unwrap();
}

#[tokio::test]
async fn test_redis_publish_message() {
    flush_redis().await;
    
    let client = setup_redis_client().await;
    let channel = "test_channel";
    let message = b"Hello, Redis!";
    
    // Publish message
    let result = client.publish(channel, message).await;
    assert!(result.is_ok(), "Should publish message successfully");
}

#[tokio::test]
async fn test_redis_publish_multiple_messages() {
    flush_redis().await;
    
    let client = setup_redis_client().await;
    let channel = "test_channel_multi";
    
    // Publish multiple messages
    for i in 0..5 {
        let message = format!("Message {}", i);
        let result = client.publish(channel, message.as_bytes()).await;
        assert!(result.is_ok(), "Should publish message {} successfully", i);
    }
}

#[tokio::test]
async fn test_redis_subscribe_to_channel() {
    flush_redis().await;
    
    let client = setup_redis_client().await;
    let channels = vec!["test_sub_channel".to_string()];
    
    // Subscribe to channel
    let result = client.subscribe(channels.clone()).await;
    assert!(result.is_ok(), "Should subscribe successfully");
    
    // Verify subscription
    let subscribed = client.get_subscribed_channels().await;
    assert!(subscribed.contains(&"test_sub_channel".to_string()));
}

#[tokio::test]
async fn test_redis_subscribe_multiple_channels() {
    flush_redis().await;
    
    let client = setup_redis_client().await;
    let channels = vec![
        "channel_1".to_string(),
        "channel_2".to_string(),
        "channel_3".to_string(),
    ];
    
    // Subscribe to multiple channels
    let result = client.subscribe(channels.clone()).await;
    assert!(result.is_ok(), "Should subscribe to multiple channels");
    
    // Verify all subscriptions
    let subscribed = client.get_subscribed_channels().await;
    assert_eq!(subscribed.len(), 3);
    for channel in &channels {
        assert!(subscribed.contains(channel), "Should be subscribed to {}", channel);
    }
}

#[tokio::test]
async fn test_redis_unsubscribe_from_channel() {
    flush_redis().await;
    
    let client = setup_redis_client().await;
    let channels = vec![
        "unsub_channel_1".to_string(),
        "unsub_channel_2".to_string(),
    ];
    
    // Subscribe first
    client.subscribe(channels.clone()).await.unwrap();
    
    // Verify subscription
    let subscribed = client.get_subscribed_channels().await;
    assert_eq!(subscribed.len(), 2);
    
    // Unsubscribe from one channel
    let result = client.unsubscribe(vec!["unsub_channel_1".to_string()]).await;
    assert!(result.is_ok(), "Should unsubscribe successfully");
    
    // Verify only one channel remains
    let subscribed = client.get_subscribed_channels().await;
    assert_eq!(subscribed.len(), 1);
    assert!(subscribed.contains(&"unsub_channel_2".to_string()));
}

#[tokio::test]
async fn test_redis_pubsub_publish_and_subscribe_tracking() {
    flush_redis().await;
    
    // Create publisher and subscriber
    let publisher = setup_redis_client().await;
    let subscriber = setup_redis_client().await;
    
    let channel = "pubsub_test_channel";
    
    // Subscribe to channel (tracks subscription locally)
    subscriber.subscribe(vec![channel.to_string()]).await.unwrap();
    
    // Verify subscription is tracked
    let subscribed = subscriber.get_subscribed_channels().await;
    assert!(subscribed.contains(&channel.to_string()));
    
    // Publish a message (this works with actual Redis)
    let test_message = b"Test message for pub/sub";
    let result = publisher.publish(channel, test_message).await;
    assert!(result.is_ok(), "Should publish message successfully");
    
    // Note: Actual message delivery requires a separate PubSub connection
    // which is handled by RedisSubscriptionManager in production
}

#[tokio::test]
async fn test_redis_multiple_subscribers_tracking() {
    flush_redis().await;
    
    let publisher = setup_redis_client().await;
    
    // Create two subscribers
    let subscriber1 = setup_redis_client().await;
    let subscriber2 = setup_redis_client().await;
    
    let channel = "multi_sub_channel";
    
    // Both subscribe to the same channel
    subscriber1.subscribe(vec![channel.to_string()]).await.unwrap();
    subscriber2.subscribe(vec![channel.to_string()]).await.unwrap();
    
    // Verify both are tracking the subscription
    let sub1_channels = subscriber1.get_subscribed_channels().await;
    let sub2_channels = subscriber2.get_subscribed_channels().await;
    
    assert!(sub1_channels.contains(&channel.to_string()));
    assert!(sub2_channels.contains(&channel.to_string()));
    
    // Publish one message
    let test_message = b"Broadcast message";
    let result = publisher.publish(channel, test_message).await;
    assert!(result.is_ok(), "Should publish successfully");
}

#[tokio::test]
async fn test_redis_channel_isolation_tracking() {
    flush_redis().await;
    
    let publisher = setup_redis_client().await;
    let subscriber1 = setup_redis_client().await;
    let subscriber2 = setup_redis_client().await;
    
    // Subscribe to different channels
    subscriber1.subscribe(vec!["channel_a".to_string()]).await.unwrap();
    subscriber2.subscribe(vec!["channel_b".to_string()]).await.unwrap();
    
    // Verify each subscriber tracks only their channel
    let sub1_channels = subscriber1.get_subscribed_channels().await;
    let sub2_channels = subscriber2.get_subscribed_channels().await;
    
    assert!(sub1_channels.contains(&"channel_a".to_string()));
    assert!(!sub1_channels.contains(&"channel_b".to_string()));
    
    assert!(sub2_channels.contains(&"channel_b".to_string()));
    assert!(!sub2_channels.contains(&"channel_a".to_string()));
    
    // Publish to both channels
    publisher.publish("channel_a", b"Message for A").await.unwrap();
    publisher.publish("channel_b", b"Message for B").await.unwrap();
}

#[tokio::test]
async fn test_redis_reconnection_handling() {
    flush_redis().await;
    
    let client = setup_redis_client().await;
    
    // Publish before any potential disconnection
    let result1 = client.publish("test_channel", b"Message 1").await;
    assert!(result1.is_ok());
    
    // Simulate some delay
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Should still work (connection manager handles reconnection)
    let result2 = client.publish("test_channel", b"Message 2").await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_redis_large_message_publish() {
    flush_redis().await;
    
    let client = setup_redis_client().await;
    
    // Create a large message (1MB)
    let large_message = vec![b'X'; 1024 * 1024];
    
    let result = client.publish("large_message_channel", &large_message).await;
    assert!(result.is_ok(), "Should handle large messages");
}

#[tokio::test]
async fn test_redis_binary_data_publish() {
    flush_redis().await;
    
    let client = setup_redis_client().await;
    
    // Binary data with null bytes
    let binary_data = vec![0u8, 1, 2, 255, 254, 0, 128];
    
    let result = client.publish("binary_channel", &binary_data).await;
    assert!(result.is_ok(), "Should handle binary data");
}

#[tokio::test]
async fn test_redis_empty_message_publish() {
    flush_redis().await;
    
    let client = setup_redis_client().await;
    
    let result = client.publish("empty_channel", b"").await;
    assert!(result.is_ok(), "Should handle empty messages");
}

#[tokio::test]
async fn test_redis_special_characters_in_channel() {
    flush_redis().await;
    
    let client = setup_redis_client().await;
    
    // Channel names with special characters
    let channels = vec![
        "channel:with:colons",
        "channel-with-dashes",
        "channel_with_underscores",
        "channel.with.dots",
    ];
    
    for channel in channels {
        let result = client.publish(channel, b"test").await;
        assert!(result.is_ok(), "Should handle channel name: {}", channel);
    }
}
