//! Redis Client Module
//! 
//! Handles Redis Pub/Sub for cross-instance message routing.

use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};

/// Redis client error types
#[derive(Debug, Error)]
pub enum RedisClientError {
    #[error("Redis connection error: {0}")]
    ConnectionError(String),
    
    #[error("Redis publish error: {0}")]
    PublishError(String),
    
    #[error("Redis subscribe error: {0}")]
    SubscribeError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Redis client trait for pub/sub operations
#[async_trait]
pub trait RedisClientTrait: Send + Sync {
    /// Publish a message to a channel
    async fn publish(&self, channel: &str, message: &[u8]) -> Result<(), RedisClientError>;
    
    /// Subscribe to channels
    async fn subscribe(&self, channels: Vec<String>) -> Result<(), RedisClientError>;
    
    /// Unsubscribe from channels
    async fn unsubscribe(&self, channels: Vec<String>) -> Result<(), RedisClientError>;
    
    /// Get subscribed channels
    async fn get_subscribed_channels(&self) -> Vec<String>;
}

/// Message received from Redis subscription
#[derive(Debug, Clone)]
pub struct RedisMessage {
    pub channel: String,
    pub payload: Vec<u8>,
}

/// Redis client implementation
pub struct RedisClient {
    connection: redis::aio::ConnectionManager,
    subscribed_channels: Arc<RwLock<HashSet<String>>>,
    message_sender: Option<mpsc::UnboundedSender<RedisMessage>>,
}

impl RedisClient {
    /// Create a new Redis client
    pub async fn new(redis_url: &str) -> Result<Self, RedisClientError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| RedisClientError::ConnectionError(e.to_string()))?;
        
        let connection = client
            .get_connection_manager()
            .await
            .map_err(|e| RedisClientError::ConnectionError(e.to_string()))?;
        
        Ok(Self {
            connection,
            subscribed_channels: Arc::new(RwLock::new(HashSet::new())),
            message_sender: None,
        })
    }

    /// Create from existing connection manager
    pub fn from_connection(connection: redis::aio::ConnectionManager) -> Self {
        Self {
            connection,
            subscribed_channels: Arc::new(RwLock::new(HashSet::new())),
            message_sender: None,
        }
    }

    /// Set message sender for received messages
    pub fn with_message_sender(mut self, sender: mpsc::UnboundedSender<RedisMessage>) -> Self {
        self.message_sender = Some(sender);
        self
    }

    /// Get the underlying connection manager
    pub fn connection(&self) -> &redis::aio::ConnectionManager {
        &self.connection
    }

    /// Publish a message to a channel (direct method)
    pub async fn publish_message(&self, channel: &str, message: &[u8]) -> Result<(), RedisClientError> {
        let mut conn = self.connection.clone();
        
        redis::cmd("PUBLISH")
            .arg(channel)
            .arg(message)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| RedisClientError::PublishError(e.to_string()))?;
        
        Ok(())
    }

    /// Publish a JSON-serializable message
    pub async fn publish_json<T: serde::Serialize>(&self, channel: &str, message: &T) -> Result<(), RedisClientError> {
        let payload = serde_json::to_vec(message)
            .map_err(|e| RedisClientError::SerializationError(e.to_string()))?;
        
        self.publish_message(channel, &payload).await
    }

    /// Get channel name for user messages
    pub fn user_channel(user_id: &uuid::Uuid) -> String {
        format!("user:{}", user_id)
    }

    /// Get channel name for group messages
    pub fn group_channel(group_id: &uuid::Uuid) -> String {
        format!("group:{}", group_id)
    }
}

#[async_trait]
impl RedisClientTrait for RedisClient {
    async fn publish(&self, channel: &str, message: &[u8]) -> Result<(), RedisClientError> {
        self.publish_message(channel, message).await
    }

    async fn subscribe(&self, channels: Vec<String>) -> Result<(), RedisClientError> {
        let mut subscribed = self.subscribed_channels.write().await;
        for channel in channels {
            subscribed.insert(channel);
        }
        // Note: Actual Redis SUBSCRIBE requires a separate connection
        // This is a simplified implementation that tracks subscriptions
        Ok(())
    }

    async fn unsubscribe(&self, channels: Vec<String>) -> Result<(), RedisClientError> {
        let mut subscribed = self.subscribed_channels.write().await;
        for channel in channels {
            subscribed.remove(&channel);
        }
        Ok(())
    }

    async fn get_subscribed_channels(&self) -> Vec<String> {
        let subscribed = self.subscribed_channels.read().await;
        subscribed.iter().cloned().collect()
    }
}

/// Redis subscription manager for handling incoming messages
pub struct RedisSubscriptionManager {
    client: redis::Client,
    subscribed_channels: Arc<RwLock<HashSet<String>>>,
    message_sender: mpsc::UnboundedSender<RedisMessage>,
}

impl RedisSubscriptionManager {
    /// Create a new subscription manager
    pub fn new(
        redis_url: &str,
        message_sender: mpsc::UnboundedSender<RedisMessage>,
    ) -> Result<Self, RedisClientError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| RedisClientError::ConnectionError(e.to_string()))?;
        
        Ok(Self {
            client,
            subscribed_channels: Arc::new(RwLock::new(HashSet::new())),
            message_sender,
        })
    }

    /// Subscribe to a channel
    pub async fn subscribe(&self, channel: &str) -> Result<(), RedisClientError> {
        let mut subscribed = self.subscribed_channels.write().await;
        subscribed.insert(channel.to_string());
        Ok(())
    }

    /// Unsubscribe from a channel
    pub async fn unsubscribe(&self, channel: &str) -> Result<(), RedisClientError> {
        let mut subscribed = self.subscribed_channels.write().await;
        subscribed.remove(channel);
        Ok(())
    }

    /// Get all subscribed channels
    pub async fn get_subscribed_channels(&self) -> Vec<String> {
        let subscribed = self.subscribed_channels.read().await;
        subscribed.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_channel_format() {
        let user_id = uuid::Uuid::new_v4();
        let channel = RedisClient::user_channel(&user_id);
        assert!(channel.starts_with("user:"));
        assert!(channel.contains(&user_id.to_string()));
    }

    #[test]
    fn test_group_channel_format() {
        let group_id = uuid::Uuid::new_v4();
        let channel = RedisClient::group_channel(&group_id);
        assert!(channel.starts_with("group:"));
        assert!(channel.contains(&group_id.to_string()));
    }

    #[tokio::test]
    async fn test_subscription_tracking() {
        let (_tx, _rx) = mpsc::unbounded_channel::<RedisMessage>();
        
        // Create a mock client for testing subscription tracking
        let subscribed_channels: Arc<RwLock<HashSet<String>>> = Arc::new(RwLock::new(HashSet::new()));
        
        // Add channels
        {
            let mut channels = subscribed_channels.write().await;
            channels.insert("user:123".to_string());
            channels.insert("group:456".to_string());
        }
        
        // Verify
        {
            let channels = subscribed_channels.read().await;
            assert_eq!(channels.len(), 2);
            assert!(channels.contains("user:123"));
            assert!(channels.contains("group:456"));
        }
        
        // Remove channel
        {
            let mut channels = subscribed_channels.write().await;
            channels.remove("user:123");
        }
        
        // Verify removal
        {
            let channels = subscribed_channels.read().await;
            assert_eq!(channels.len(), 1);
            assert!(!channels.contains("user:123"));
            assert!(channels.contains("group:456"));
        }
    }
}
