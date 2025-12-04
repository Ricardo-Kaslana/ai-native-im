//! Multi-Instance Message Routing Module
//! 
//! Provides cross-instance message routing using Redis Pub/Sub for horizontal scaling.

use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use super::connection_manager::ConnectionManager;
use super::models::{Message, CreateMessage};
use super::redis_client::{RedisClient, RedisClientTrait, RedisMessage};

/// Multi-instance router error types
#[derive(Debug, thiserror::Error)]
pub enum MultiInstanceError {
    #[error("Redis error: {0}")]
    RedisError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Channel closed")]
    ChannelClosed,
    
    #[error("Instance not registered")]
    NotRegistered,
}

/// Instance identifier
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct InstanceId(String);

impl InstanceId {
    /// Create a new random instance ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create from string
    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    /// Get the instance ID as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for InstanceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Channel names for Redis Pub/Sub
pub struct ChannelNames;

impl ChannelNames {
    /// Global broadcast channel
    pub const BROADCAST: &'static str = "im:broadcast";
    
    /// User-specific channel prefix
    pub const USER_PREFIX: &'static str = "im:user:";
    
    /// Group-specific channel prefix
    pub const GROUP_PREFIX: &'static str = "im:group:";
    
    /// Instance heartbeat channel
    pub const HEARTBEAT: &'static str = "im:heartbeat";
    
    /// Get user channel name
    pub fn user_channel(user_id: &Uuid) -> String {
        format!("{}{}", Self::USER_PREFIX, user_id)
    }
    
    /// Get group channel name
    pub fn group_channel(group_id: &Uuid) -> String {
        format!("{}{}", Self::GROUP_PREFIX, group_id)
    }
}

/// Cross-instance message envelope
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MessageEnvelope {
    /// Source instance ID
    pub source_instance: String,
    /// Target user ID (for private messages)
    pub target_user_id: Option<Uuid>,
    /// Target group ID (for group messages)
    pub target_group_id: Option<Uuid>,
    /// The actual message
    pub message: EnvelopeMessage,
    /// Timestamp
    pub timestamp: i64,
}

/// Simplified message for envelope
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnvelopeMessage {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub msg_type: String,
    pub content: String,
    pub created_at: i64,
}

impl From<&Message> for EnvelopeMessage {
    fn from(msg: &Message) -> Self {
        Self {
            id: msg.id,
            sender_id: msg.sender_id,
            receiver_id: msg.receiver_id,
            group_id: msg.group_id,
            msg_type: format!("{:?}", msg.msg_type),
            content: msg.content.clone(),
            created_at: msg.created_at.timestamp_millis(),
        }
    }
}

/// Multi-instance router configuration
#[derive(Debug, Clone)]
pub struct MultiInstanceConfig {
    /// This instance's ID
    pub instance_id: InstanceId,
    /// Heartbeat interval in seconds
    pub heartbeat_interval_secs: u64,
    /// Instance timeout in seconds (consider dead if no heartbeat)
    pub instance_timeout_secs: u64,
}

impl Default for MultiInstanceConfig {
    fn default() -> Self {
        Self {
            instance_id: InstanceId::new(),
            heartbeat_interval_secs: 10,
            instance_timeout_secs: 30,
        }
    }
}

/// Multi-instance message router
pub struct MultiInstanceRouter<C: ConnectionManager, R: RedisClientTrait> {
    config: MultiInstanceConfig,
    connection_manager: Arc<C>,
    redis_client: Arc<R>,
    subscribed_channels: Arc<RwLock<HashSet<String>>>,
    message_sender: mpsc::Sender<MessageEnvelope>,
    is_running: Arc<std::sync::atomic::AtomicBool>,
}

impl<C: ConnectionManager + Send + Sync + 'static, R: RedisClientTrait + Send + Sync + 'static>
    MultiInstanceRouter<C, R>
{
    /// Create a new multi-instance router
    pub fn new(
        connection_manager: Arc<C>,
        redis_client: Arc<R>,
        message_sender: mpsc::Sender<MessageEnvelope>,
    ) -> Self {
        Self {
            config: MultiInstanceConfig::default(),
            connection_manager,
            redis_client,
            subscribed_channels: Arc::new(RwLock::new(HashSet::new())),
            message_sender,
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        config: MultiInstanceConfig,
        connection_manager: Arc<C>,
        redis_client: Arc<R>,
        message_sender: mpsc::Sender<MessageEnvelope>,
    ) -> Self {
        Self {
            config,
            connection_manager,
            redis_client,
            subscribed_channels: Arc::new(RwLock::new(HashSet::new())),
            message_sender,
            is_running: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        }
    }

    /// Get instance ID
    pub fn instance_id(&self) -> &InstanceId {
        &self.config.instance_id
    }

    /// Publish message to user channel
    pub async fn publish_to_user(
        &self,
        user_id: &Uuid,
        message: &Message,
    ) -> Result<(), MultiInstanceError> {
        let channel = ChannelNames::user_channel(user_id);
        let envelope = MessageEnvelope {
            source_instance: self.config.instance_id.as_str().to_string(),
            target_user_id: Some(*user_id),
            target_group_id: None,
            message: EnvelopeMessage::from(message),
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        self.publish_envelope(&channel, &envelope).await
    }

    /// Publish message to group channel
    pub async fn publish_to_group(
        &self,
        group_id: &Uuid,
        message: &Message,
    ) -> Result<(), MultiInstanceError> {
        let channel = ChannelNames::group_channel(group_id);
        let envelope = MessageEnvelope {
            source_instance: self.config.instance_id.as_str().to_string(),
            target_user_id: None,
            target_group_id: Some(*group_id),
            message: EnvelopeMessage::from(message),
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        self.publish_envelope(&channel, &envelope).await
    }

    /// Publish envelope to channel
    async fn publish_envelope(
        &self,
        channel: &str,
        envelope: &MessageEnvelope,
    ) -> Result<(), MultiInstanceError> {
        let payload = serde_json::to_vec(envelope)
            .map_err(|e| MultiInstanceError::SerializationError(e.to_string()))?;

        self.redis_client
            .publish(channel, &payload)
            .await
            .map_err(|e| MultiInstanceError::RedisError(e.to_string()))?;

        Ok(())
    }

    /// Subscribe to user channel
    pub async fn subscribe_user(&self, user_id: &Uuid) -> Result<(), MultiInstanceError> {
        let channel = ChannelNames::user_channel(user_id);
        self.subscribe_channel(&channel).await
    }

    /// Unsubscribe from user channel
    pub async fn unsubscribe_user(&self, user_id: &Uuid) -> Result<(), MultiInstanceError> {
        let channel = ChannelNames::user_channel(user_id);
        self.unsubscribe_channel(&channel).await
    }

    /// Subscribe to group channel
    pub async fn subscribe_group(&self, group_id: &Uuid) -> Result<(), MultiInstanceError> {
        let channel = ChannelNames::group_channel(group_id);
        self.subscribe_channel(&channel).await
    }

    /// Unsubscribe from group channel
    pub async fn unsubscribe_group(&self, group_id: &Uuid) -> Result<(), MultiInstanceError> {
        let channel = ChannelNames::group_channel(group_id);
        self.unsubscribe_channel(&channel).await
    }

    /// Subscribe to a channel
    async fn subscribe_channel(&self, channel: &str) -> Result<(), MultiInstanceError> {
        let mut channels = self.subscribed_channels.write().await;
        
        if !channels.contains(channel) {
            self.redis_client
                .subscribe(vec![channel.to_string()])
                .await
                .map_err(|e| MultiInstanceError::RedisError(e.to_string()))?;
            channels.insert(channel.to_string());
        }

        Ok(())
    }

    /// Unsubscribe from a channel
    async fn unsubscribe_channel(&self, channel: &str) -> Result<(), MultiInstanceError> {
        let mut channels = self.subscribed_channels.write().await;
        
        if channels.contains(channel) {
            self.redis_client
                .unsubscribe(vec![channel.to_string()])
                .await
                .map_err(|e| MultiInstanceError::RedisError(e.to_string()))?;
            channels.remove(channel);
        }

        Ok(())
    }

    /// Get subscribed channels count
    pub async fn subscribed_count(&self) -> usize {
        let channels = self.subscribed_channels.read().await;
        channels.len()
    }

    /// Check if running
    pub fn is_running(&self) -> bool {
        self.is_running.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Shutdown the router
    pub fn shutdown(&self) {
        self.is_running.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Message receiver that processes incoming Redis messages
pub struct MessageReceiver {
    instance_id: InstanceId,
    receiver: mpsc::Receiver<RedisMessage>,
    output: mpsc::Sender<MessageEnvelope>,
}

impl MessageReceiver {
    /// Create a new message receiver
    pub fn new(
        instance_id: InstanceId,
        receiver: mpsc::Receiver<RedisMessage>,
        output: mpsc::Sender<MessageEnvelope>,
    ) -> Self {
        Self {
            instance_id,
            receiver,
            output,
        }
    }

    /// Start processing messages
    pub async fn run(mut self) {
        while let Some(redis_msg) = self.receiver.recv().await {
            // Parse envelope from bytes
            let envelope: Result<MessageEnvelope, _> = serde_json::from_slice(&redis_msg.payload);
            
            match envelope {
                Ok(env) => {
                    // Skip messages from this instance
                    if env.source_instance == self.instance_id.as_str() {
                        continue;
                    }

                    // Forward to output channel
                    if self.output.send(env).await.is_err() {
                        tracing::error!("Failed to forward message envelope");
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        channel = %redis_msg.channel,
                        "Failed to parse message envelope"
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_id_creation() {
        let id1 = InstanceId::new();
        let id2 = InstanceId::new();
        
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_channel_names() {
        let user_id = Uuid::new_v4();
        let group_id = Uuid::new_v4();
        
        let user_channel = ChannelNames::user_channel(&user_id);
        let group_channel = ChannelNames::group_channel(&group_id);
        
        assert!(user_channel.starts_with("im:user:"));
        assert!(group_channel.starts_with("im:group:"));
    }

    #[test]
    fn test_message_envelope_serialization() {
        let envelope = MessageEnvelope {
            source_instance: "test-instance".to_string(),
            target_user_id: Some(Uuid::new_v4()),
            target_group_id: None,
            message: EnvelopeMessage {
                id: Uuid::new_v4(),
                sender_id: Uuid::new_v4(),
                receiver_id: Some(Uuid::new_v4()),
                group_id: None,
                msg_type: "Text".to_string(),
                content: "Hello".to_string(),
                created_at: chrono::Utc::now().timestamp_millis(),
            },
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        let json = serde_json::to_string(&envelope).unwrap();
        let parsed: MessageEnvelope = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.source_instance, envelope.source_instance);
    }

    #[test]
    fn test_multi_instance_config_default() {
        let config = MultiInstanceConfig::default();
        
        assert_eq!(config.heartbeat_interval_secs, 10);
        assert_eq!(config.instance_timeout_secs, 30);
    }
}
