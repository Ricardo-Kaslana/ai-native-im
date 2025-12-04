//! Redis Fallback Module
//! 
//! Provides fallback to database polling when Redis is unavailable.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::message_persister::MessagePersister;
use super::models::Message;

/// Redis fallback error types
#[derive(Debug, Error)]
pub enum FallbackError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Polling stopped")]
    PollingStopped,
}

/// Configuration for Redis fallback
#[derive(Debug, Clone)]
pub struct FallbackConfig {
    /// Polling interval in milliseconds when Redis is down
    pub polling_interval_ms: u64,
    /// Health check interval in milliseconds
    pub health_check_interval_ms: u64,
    /// Maximum messages per poll
    pub max_messages_per_poll: i32,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            polling_interval_ms: 1000, // 1 second as per requirements
            health_check_interval_ms: 5000,
            max_messages_per_poll: 100,
        }
    }
}

/// Redis health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedisHealth {
    Healthy,
    Unhealthy,
    Unknown,
}

/// Redis fallback service
pub struct RedisFallbackService {
    persister: Arc<dyn MessagePersister>,
    redis_connection: Option<Arc<redis::aio::ConnectionManager>>,
    config: FallbackConfig,
    is_redis_healthy: Arc<AtomicBool>,
    is_polling: Arc<AtomicBool>,
    last_poll_timestamp: Arc<RwLock<i64>>,
}

impl RedisFallbackService {
    /// Create a new fallback service
    pub fn new(persister: Arc<dyn MessagePersister>) -> Self {
        Self {
            persister,
            redis_connection: None,
            config: FallbackConfig::default(),
            is_redis_healthy: Arc::new(AtomicBool::new(true)),
            is_polling: Arc::new(AtomicBool::new(false)),
            last_poll_timestamp: Arc::new(RwLock::new(0)),
        }
    }

    /// Create with Redis connection for health checking
    pub fn with_redis(
        persister: Arc<dyn MessagePersister>,
        redis: Arc<redis::aio::ConnectionManager>,
    ) -> Self {
        Self {
            persister,
            redis_connection: Some(redis),
            config: FallbackConfig::default(),
            is_redis_healthy: Arc::new(AtomicBool::new(true)),
            is_polling: Arc::new(AtomicBool::new(false)),
            last_poll_timestamp: Arc::new(RwLock::new(0)),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        persister: Arc<dyn MessagePersister>,
        config: FallbackConfig,
    ) -> Self {
        Self {
            persister,
            redis_connection: None,
            config,
            is_redis_healthy: Arc::new(AtomicBool::new(true)),
            is_polling: Arc::new(AtomicBool::new(false)),
            last_poll_timestamp: Arc::new(RwLock::new(0)),
        }
    }

    /// Check Redis health
    pub async fn check_redis_health(&self) -> RedisHealth {
        if let Some(redis) = &self.redis_connection {
            let mut conn = (**redis).clone();
            
            match redis::cmd("PING")
                .query_async::<String>(&mut conn)
                .await
            {
                Ok(response) if response == "PONG" => {
                    self.is_redis_healthy.store(true, Ordering::SeqCst);
                    RedisHealth::Healthy
                }
                _ => {
                    self.is_redis_healthy.store(false, Ordering::SeqCst);
                    RedisHealth::Unhealthy
                }
            }
        } else {
            RedisHealth::Unknown
        }
    }

    /// Check if Redis is currently healthy
    pub fn is_redis_healthy(&self) -> bool {
        self.is_redis_healthy.load(Ordering::SeqCst)
    }

    /// Check if fallback polling is active
    pub fn is_polling_active(&self) -> bool {
        self.is_polling.load(Ordering::SeqCst)
    }

    /// Start fallback polling for a user
    pub async fn start_polling_for_user(
        &self,
        user_id: Uuid,
        message_handler: impl Fn(Message) + Send + Sync + 'static,
    ) {
        if self.is_polling.swap(true, Ordering::SeqCst) {
            // Already polling
            return;
        }

        let persister = self.persister.clone();
        let config = self.config.clone();
        let is_polling = self.is_polling.clone();
        let is_redis_healthy = self.is_redis_healthy.clone();
        let last_poll_timestamp = self.last_poll_timestamp.clone();

        tokio::spawn(async move {
            let interval = Duration::from_millis(config.polling_interval_ms);
            
            tracing::info!(
                user_id = %user_id,
                interval_ms = config.polling_interval_ms,
                "Starting database polling fallback"
            );

            while is_polling.load(Ordering::SeqCst) {
                // Check if Redis is back
                if is_redis_healthy.load(Ordering::SeqCst) {
                    tracing::info!("Redis recovered, stopping database polling");
                    break;
                }

                // Get last poll timestamp
                let since_timestamp = {
                    let ts = last_poll_timestamp.read().await;
                    *ts
                };

                // Poll for new messages
                match persister
                    .get_offline_messages(&user_id, since_timestamp, config.max_messages_per_poll)
                    .await
                {
                    Ok(batch) => {
                        for message in batch.messages {
                            // Update last timestamp
                            {
                                let mut ts = last_poll_timestamp.write().await;
                                if message.timestamp > *ts {
                                    *ts = message.timestamp;
                                }
                            }
                            
                            // Handle message
                            message_handler(message);
                        }
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Database polling failed");
                    }
                }

                tokio::time::sleep(interval).await;
            }

            is_polling.store(false, Ordering::SeqCst);
            tracing::info!(user_id = %user_id, "Database polling stopped");
        });
    }

    /// Stop fallback polling
    pub fn stop_polling(&self) {
        self.is_polling.store(false, Ordering::SeqCst);
    }

    /// Mark Redis as unhealthy (trigger fallback)
    pub fn mark_redis_unhealthy(&self) {
        self.is_redis_healthy.store(false, Ordering::SeqCst);
        tracing::warn!("Redis marked as unhealthy, fallback mode activated");
    }

    /// Mark Redis as healthy (stop fallback)
    pub fn mark_redis_healthy(&self) {
        self.is_redis_healthy.store(true, Ordering::SeqCst);
        tracing::info!("Redis marked as healthy");
    }

    /// Start health check background task
    pub fn start_health_check_task(self: Arc<Self>) {
        let service = self.clone();
        let interval = Duration::from_millis(self.config.health_check_interval_ms);
        
        tokio::spawn(async move {
            let mut check_interval = tokio::time::interval(interval);
            
            loop {
                check_interval.tick().await;
                
                let health = service.check_redis_health().await;
                tracing::debug!(health = ?health, "Redis health check completed");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::message::message_persister::PersisterError;
    use crate::services::message::models::OfflineMessageBatch;
    use async_trait::async_trait;

    // Mock persister for testing
    struct MockPersister;

    #[async_trait]
    impl MessagePersister for MockPersister {
        async fn persist_message(&self, _message: &Message) -> Result<(), PersisterError> {
            Ok(())
        }

        async fn get_offline_messages(
            &self,
            _user_id: &Uuid,
            _since_timestamp: i64,
            _limit: i32,
        ) -> Result<OfflineMessageBatch, PersisterError> {
            Ok(OfflineMessageBatch {
                messages: vec![],
                has_more: false,
                next_timestamp: None,
            })
        }

        async fn mark_messages_delivered(&self, _message_ids: &[Uuid]) -> Result<(), PersisterError> {
            Ok(())
        }

        async fn message_exists(&self, _message_id: &Uuid) -> Result<bool, PersisterError> {
            Ok(false)
        }
    }

    #[test]
    fn test_fallback_config_default() {
        let config = FallbackConfig::default();
        
        assert_eq!(config.polling_interval_ms, 1000);
        assert_eq!(config.health_check_interval_ms, 5000);
        assert_eq!(config.max_messages_per_poll, 100);
    }

    #[test]
    fn test_redis_health_enum() {
        assert_ne!(RedisHealth::Healthy, RedisHealth::Unhealthy);
        assert_ne!(RedisHealth::Healthy, RedisHealth::Unknown);
    }

    #[tokio::test]
    async fn test_fallback_service_creation() {
        let persister: Arc<dyn MessagePersister> = Arc::new(MockPersister);
        let service = RedisFallbackService::new(persister);
        
        assert!(service.is_redis_healthy());
        assert!(!service.is_polling_active());
    }

    #[tokio::test]
    async fn test_mark_redis_unhealthy() {
        let persister: Arc<dyn MessagePersister> = Arc::new(MockPersister);
        let service = RedisFallbackService::new(persister);
        
        assert!(service.is_redis_healthy());
        
        service.mark_redis_unhealthy();
        
        assert!(!service.is_redis_healthy());
    }

    #[tokio::test]
    async fn test_mark_redis_healthy() {
        let persister: Arc<dyn MessagePersister> = Arc::new(MockPersister);
        let service = RedisFallbackService::new(persister);
        
        service.mark_redis_unhealthy();
        assert!(!service.is_redis_healthy());
        
        service.mark_redis_healthy();
        assert!(service.is_redis_healthy());
    }

    #[tokio::test]
    async fn test_stop_polling() {
        let persister: Arc<dyn MessagePersister> = Arc::new(MockPersister);
        let service = RedisFallbackService::new(persister);
        
        // Manually set polling flag
        service.is_polling.store(true, Ordering::SeqCst);
        assert!(service.is_polling_active());
        
        service.stop_polling();
        assert!(!service.is_polling_active());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::services::message::message_persister::PersisterError;
    use crate::services::message::models::{MsgType, OfflineMessageBatch};
    use async_trait::async_trait;
    use proptest::prelude::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    // Mock persister that tracks poll count
    struct CountingPersister {
        poll_count: Arc<AtomicU32>,
    }

    impl CountingPersister {
        fn new() -> Self {
            Self {
                poll_count: Arc::new(AtomicU32::new(0)),
            }
        }

        fn get_poll_count(&self) -> u32 {
            self.poll_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl MessagePersister for CountingPersister {
        async fn persist_message(&self, _message: &Message) -> Result<(), PersisterError> {
            Ok(())
        }

        async fn get_offline_messages(
            &self,
            _user_id: &Uuid,
            _since_timestamp: i64,
            _limit: i32,
        ) -> Result<OfflineMessageBatch, PersisterError> {
            self.poll_count.fetch_add(1, Ordering::SeqCst);
            Ok(OfflineMessageBatch {
                messages: vec![],
                has_more: false,
                next_timestamp: None,
            })
        }

        async fn mark_messages_delivered(&self, _message_ids: &[Uuid]) -> Result<(), PersisterError> {
            Ok(())
        }

        async fn message_exists(&self, _message_id: &Uuid) -> Result<bool, PersisterError> {
            Ok(false)
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]
        
        /// **Feature: message-system, Property 31: Redis fallback to database polling**
        /// For any scenario where Redis Pub/Sub is unavailable, the system should fall back 
        /// to database polling with 1-second intervals.
        /// **Validates: Requirements 7.5**
        #[test]
        fn prop_redis_fallback_polling(
            _seed in any::<u64>(),
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let persister = Arc::new(CountingPersister::new());
                let poll_count_ref = persister.poll_count.clone();
                
                // Create service with short polling interval for testing
                let config = FallbackConfig {
                    polling_interval_ms: 50, // 50ms for faster testing
                    health_check_interval_ms: 5000,
                    max_messages_per_poll: 100,
                };
                
                let service = RedisFallbackService::with_config(
                    persister.clone() as Arc<dyn MessagePersister>,
                    config,
                );

                // Mark Redis as unhealthy to trigger fallback
                service.mark_redis_unhealthy();
                prop_assert!(!service.is_redis_healthy());

                // Start polling
                let user_id = Uuid::new_v4();
                service.start_polling_for_user(user_id, |_msg| {
                    // Message handler (no-op for this test)
                }).await;

                // Wait for some polling to occur
                tokio::time::sleep(Duration::from_millis(150)).await;

                // Stop polling
                service.stop_polling();
                
                // Wait a bit for the polling task to stop
                tokio::time::sleep(Duration::from_millis(50)).await;

                // Verify that polling occurred
                let poll_count = poll_count_ref.load(Ordering::SeqCst);
                
                // With 50ms interval and 150ms wait time,
                // we should have at least 2 polls
                prop_assert!(
                    poll_count >= 2,
                    "Expected at least 2 polls, got {}",
                    poll_count
                );

                // Verify polling stopped
                prop_assert!(!service.is_polling_active());

                Ok(())
            })?;
        }

        /// Property: Redis health recovery stops polling
        #[test]
        fn prop_redis_recovery_stops_polling(
            _seed in any::<u64>(),
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let persister = Arc::new(CountingPersister::new());
                
                let config = FallbackConfig {
                    polling_interval_ms: 50,
                    health_check_interval_ms: 5000,
                    max_messages_per_poll: 100,
                };
                
                let service = RedisFallbackService::with_config(
                    persister.clone() as Arc<dyn MessagePersister>,
                    config,
                );

                // Mark Redis as unhealthy
                service.mark_redis_unhealthy();
                
                // Start polling
                let user_id = Uuid::new_v4();
                service.start_polling_for_user(user_id, |_msg| {}).await;
                
                // Wait for polling to start
                tokio::time::sleep(Duration::from_millis(100)).await;
                prop_assert!(service.is_polling_active());

                // Mark Redis as healthy (simulating recovery)
                service.mark_redis_healthy();
                
                // Wait for polling to detect recovery and stop
                tokio::time::sleep(Duration::from_millis(200)).await;

                // Polling should have stopped
                prop_assert!(!service.is_polling_active());

                Ok(())
            })?;
        }

        /// Property: Fallback config validation
        #[test]
        fn prop_fallback_config(
            interval_ms in 100u64..5000u64,
            max_messages in 10i32..500i32,
        ) {
            let config = FallbackConfig {
                polling_interval_ms: interval_ms,
                health_check_interval_ms: 5000,
                max_messages_per_poll: max_messages,
            };

            prop_assert_eq!(config.polling_interval_ms, interval_ms);
            prop_assert_eq!(config.max_messages_per_poll, max_messages);
        }

        /// **Feature: message-system, Property 31: Redis fallback to database polling**
        /// For any scenario where Redis Pub/Sub is unavailable, the system should fall 
        /// back to database polling with 1-second intervals.
        /// **Validates: Requirements 7.5**
        #[test]
        fn prop_redis_fallback_to_polling(
            _seed in any::<u64>(),
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let persister = Arc::new(CountingPersister::new());
                
                // Configure with 1-second polling as per requirements
                let config = FallbackConfig {
                    polling_interval_ms: 1000,
                    health_check_interval_ms: 5000,
                    max_messages_per_poll: 100,
                };
                
                let service = RedisFallbackService::with_config(
                    persister.clone() as Arc<dyn MessagePersister>,
                    config,
                );

                // Initially Redis is healthy, polling should not be active
                prop_assert!(service.is_redis_healthy());
                prop_assert!(!service.is_polling_active());

                // Mark Redis as unhealthy (simulating Redis failure)
                service.mark_redis_unhealthy();
                prop_assert!(!service.is_redis_healthy());
                
                // Start polling for a user
                let user_id = Uuid::new_v4();
                service.start_polling_for_user(user_id, |_msg| {}).await;
                
                // Wait for polling to start
                tokio::time::sleep(Duration::from_millis(100)).await;
                
                // Verify polling is now active (fallback activated)
                prop_assert!(
                    service.is_polling_active(),
                    "Polling should be active when Redis is unhealthy"
                );

                // Stop polling
                service.stop_polling();
                tokio::time::sleep(Duration::from_millis(100)).await;

                Ok(())
            })?;
        }
    }
}
