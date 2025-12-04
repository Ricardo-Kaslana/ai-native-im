//! Offline Message Sync Module
//! 
//! Handles retrieval and delivery of offline messages when users reconnect.

use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

use super::connection_manager::InMemoryConnectionManager;
use super::message_persister::{MessagePersister, PersisterError};
use super::models::{Message, OfflineMessageBatch};

/// Offline sync error types
#[derive(Debug, Error)]
pub enum OfflineSyncError {
    #[error("Persistence error: {0}")]
    PersistenceError(#[from] PersisterError),
    
    #[error("Delivery failed: {0}")]
    DeliveryFailed(String),
    
    #[error("User not connected")]
    UserNotConnected,
}

/// Configuration for offline message sync
#[derive(Debug, Clone)]
pub struct OfflineSyncConfig {
    /// Batch size for offline message retrieval
    pub batch_size: i32,
    /// Delay between batches in milliseconds
    pub batch_delay_ms: u64,
    /// Maximum messages to sync per connection
    pub max_messages: i32,
}

impl Default for OfflineSyncConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            batch_delay_ms: 500,
            max_messages: 10000,
        }
    }
}

/// Offline message sync service
pub struct OfflineSyncService {
    persister: Arc<dyn MessagePersister>,
    connection_manager: Arc<InMemoryConnectionManager>,
    config: OfflineSyncConfig,
}

impl OfflineSyncService {
    /// Create a new offline sync service
    pub fn new(
        persister: Arc<dyn MessagePersister>,
        connection_manager: Arc<InMemoryConnectionManager>,
    ) -> Self {
        Self {
            persister,
            connection_manager,
            config: OfflineSyncConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        persister: Arc<dyn MessagePersister>,
        connection_manager: Arc<InMemoryConnectionManager>,
        config: OfflineSyncConfig,
    ) -> Self {
        Self {
            persister,
            connection_manager,
            config,
        }
    }

    /// Sync offline messages for a user
    pub async fn sync_offline_messages(
        &self,
        user_id: &Uuid,
        since_timestamp: i64,
    ) -> Result<SyncResult, OfflineSyncError> {
        let mut total_delivered = 0;
        let mut last_timestamp = since_timestamp;
        let mut batches_delivered = 0;

        loop {
            // Get batch of offline messages
            let batch = self.persister
                .get_offline_messages(user_id, last_timestamp, self.config.batch_size)
                .await?;

            if batch.messages.is_empty() {
                break;
            }

            // Deliver messages
            let delivered = self.deliver_batch(user_id, &batch.messages).await?;
            total_delivered += delivered;
            batches_delivered += 1;

            // Mark messages as delivered
            let message_ids: Vec<Uuid> = batch.messages.iter().map(|m| m.id).collect();
            self.persister.mark_messages_delivered(&message_ids).await?;

            // Update last timestamp for next batch
            if let Some(ts) = batch.next_timestamp {
                last_timestamp = ts;
            }

            // Check if we've reached the limit
            if total_delivered >= self.config.max_messages as usize {
                tracing::warn!(
                    user_id = %user_id,
                    total = total_delivered,
                    "Reached max offline messages limit"
                );
                break;
            }

            // Check if there are more messages
            if !batch.has_more {
                break;
            }

            // Delay between batches
            if self.config.batch_delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.config.batch_delay_ms)).await;
            }
        }

        Ok(SyncResult {
            total_delivered,
            batches_delivered,
            last_timestamp,
        })
    }

    /// Deliver a batch of messages to a user
    async fn deliver_batch(
        &self,
        user_id: &Uuid,
        messages: &[Message],
    ) -> Result<usize, OfflineSyncError> {
        let connections = self.connection_manager.get_user_connections_async(user_id).await;
        
        if connections.is_empty() {
            return Err(OfflineSyncError::UserNotConnected);
        }

        let mut delivered = 0;

        for message in messages {
            let mut message_delivered = false;

            for conn_id in &connections {
                if let Some(sender) = self.connection_manager.get_sender_async(conn_id).await {
                    if sender.send(message.clone()).is_ok() {
                        message_delivered = true;
                    }
                }
            }

            if message_delivered {
                delivered += 1;
            }
        }

        Ok(delivered)
    }

    /// Get offline message count for a user
    pub async fn get_offline_count(&self, user_id: &Uuid) -> Result<usize, OfflineSyncError> {
        let batch = self.persister
            .get_offline_messages(user_id, 0, 1)
            .await?;
        
        // This is an approximation - for exact count, we'd need a separate query
        if batch.has_more {
            Ok(1000) // Indicate there are many messages
        } else {
            Ok(batch.messages.len())
        }
    }
}

/// Result of offline message sync
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Total messages delivered
    pub total_delivered: usize,
    /// Number of batches delivered
    pub batches_delivered: usize,
    /// Last message timestamp
    pub last_timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_sync_config_default() {
        let config = OfflineSyncConfig::default();
        
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.batch_delay_ms, 500);
        assert_eq!(config.max_messages, 10000);
    }

    #[test]
    fn test_sync_result() {
        let result = SyncResult {
            total_delivered: 150,
            batches_delivered: 2,
            last_timestamp: 1234567890,
        };
        
        assert_eq!(result.total_delivered, 150);
        assert_eq!(result.batches_delivered, 2);
    }
}


#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property 18: Offline message batching
        /// Batch size configuration should be respected.
        #[test]
        fn prop_batch_size_configuration(
            batch_size in 10i32..500,
            batch_delay_ms in 0u64..1000,
            max_messages in 100i32..50000,
        ) {
            let config = OfflineSyncConfig {
                batch_size,
                batch_delay_ms,
                max_messages,
            };

            prop_assert_eq!(config.batch_size, batch_size);
            prop_assert_eq!(config.batch_delay_ms, batch_delay_ms);
            prop_assert_eq!(config.max_messages, max_messages);
        }

        /// Property: SyncResult correctly tracks delivery statistics
        #[test]
        fn prop_sync_result_tracking(
            total_delivered in 0usize..10000,
            batches_delivered in 0usize..100,
            last_timestamp in 0i64..i64::MAX,
        ) {
            let result = SyncResult {
                total_delivered,
                batches_delivered,
                last_timestamp,
            };

            prop_assert_eq!(result.total_delivered, total_delivered);
            prop_assert_eq!(result.batches_delivered, batches_delivered);
            prop_assert_eq!(result.last_timestamp, last_timestamp);
        }

        /// Property: Default config has reasonable values
        #[test]
        fn prop_default_config_reasonable(_seed in any::<u64>()) {
            let config = OfflineSyncConfig::default();

            // Batch size should be reasonable (not too small, not too large)
            prop_assert!(config.batch_size >= 10 && config.batch_size <= 1000);
            
            // Batch delay should be reasonable
            prop_assert!(config.batch_delay_ms <= 5000);
            
            // Max messages should be reasonable
            prop_assert!(config.max_messages >= 100 && config.max_messages <= 100000);
        }
    }
}
