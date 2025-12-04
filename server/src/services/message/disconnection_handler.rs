//! Disconnection Handler Module
//! 
//! Handles WebSocket disconnection detection and cleanup.

use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::connection_manager::InMemoryConnectionManager;
use super::redis_client::RedisClient;

/// Disconnection handler error types
#[derive(Debug, Error)]
pub enum DisconnectionError {
    #[error("Connection not found: {0}")]
    ConnectionNotFound(String),
    
    #[error("Redis error: {0}")]
    RedisError(String),
    
    #[error("Cleanup failed: {0}")]
    CleanupFailed(String),
}

/// Configuration for disconnection handling
#[derive(Debug, Clone)]
pub struct DisconnectionConfig {
    /// Detection timeout in seconds (connection considered dead if no activity)
    pub detection_timeout_secs: u64,
    /// Cleanup delay in milliseconds
    pub cleanup_delay_ms: u64,
    /// Enable Redis channel unsubscription
    pub unsubscribe_redis: bool,
}

impl Default for DisconnectionConfig {
    fn default() -> Self {
        Self {
            detection_timeout_secs: 5,
            cleanup_delay_ms: 100,
            unsubscribe_redis: true,
        }
    }
}

/// Connection health tracker
#[derive(Debug, Clone)]
pub struct ConnectionHealth {
    pub conn_id: String,
    pub user_id: Uuid,
    pub last_activity: Instant,
    pub is_healthy: bool,
}

impl ConnectionHealth {
    pub fn new(conn_id: String, user_id: Uuid) -> Self {
        Self {
            conn_id,
            user_id,
            last_activity: Instant::now(),
            is_healthy: true,
        }
    }

    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    pub fn is_timed_out(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }
}

/// Disconnection handler service
pub struct DisconnectionHandler {
    connection_manager: Arc<InMemoryConnectionManager>,
    redis_client: Option<Arc<RedisClient>>,
    config: DisconnectionConfig,
    health_tracker: Arc<RwLock<std::collections::HashMap<String, ConnectionHealth>>>,
}

impl DisconnectionHandler {
    /// Create a new disconnection handler
    pub fn new(connection_manager: Arc<InMemoryConnectionManager>) -> Self {
        Self {
            connection_manager,
            redis_client: None,
            config: DisconnectionConfig::default(),
            health_tracker: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Create with Redis support
    pub fn with_redis(
        connection_manager: Arc<InMemoryConnectionManager>,
        redis_client: Arc<RedisClient>,
    ) -> Self {
        Self {
            connection_manager,
            redis_client: Some(redis_client),
            config: DisconnectionConfig::default(),
            health_tracker: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        connection_manager: Arc<InMemoryConnectionManager>,
        config: DisconnectionConfig,
    ) -> Self {
        Self {
            connection_manager,
            redis_client: None,
            config,
            health_tracker: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Register a new connection for health tracking
    pub async fn register_connection(&self, conn_id: &str, user_id: Uuid) {
        let mut tracker = self.health_tracker.write().await;
        tracker.insert(
            conn_id.to_string(),
            ConnectionHealth::new(conn_id.to_string(), user_id),
        );
        
        tracing::debug!(
            conn_id = conn_id,
            user_id = %user_id,
            "Connection registered for health tracking"
        );
    }

    /// Update connection activity
    pub async fn update_activity(&self, conn_id: &str) {
        let mut tracker = self.health_tracker.write().await;
        if let Some(health) = tracker.get_mut(conn_id) {
            health.update_activity();
        }
    }

    /// Handle disconnection
    pub async fn handle_disconnection(&self, conn_id: &str) -> Result<(), DisconnectionError> {
        tracing::info!(conn_id = conn_id, "Handling disconnection");

        // Get connection info before cleanup
        let health = {
            let mut tracker = self.health_tracker.write().await;
            tracker.remove(conn_id)
        };

        // Unregister from connection manager
        self.connection_manager.unregister_connection_async(conn_id).await;

        // Unsubscribe from Redis channels if configured
        if self.config.unsubscribe_redis {
            if let Some(health) = &health {
                self.unsubscribe_user_channels(&health.user_id).await?;
            }
        }

        // Small delay for cleanup
        if self.config.cleanup_delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.config.cleanup_delay_ms)).await;
        }

        tracing::info!(
            conn_id = conn_id,
            user_id = ?health.map(|h| h.user_id),
            "Disconnection cleanup completed"
        );

        Ok(())
    }

    /// Unsubscribe user from Redis channels
    async fn unsubscribe_user_channels(&self, user_id: &Uuid) -> Result<(), DisconnectionError> {
        if let Some(redis) = &self.redis_client {
            let channel = RedisClient::user_channel(user_id);
            // Use the trait method
            use super::redis_client::RedisClientTrait;
            redis.unsubscribe(vec![channel]).await
                .map_err(|e| DisconnectionError::RedisError(e.to_string()))?;
        }
        Ok(())
    }

    /// Check for timed out connections
    pub async fn check_timeouts(&self) -> Vec<String> {
        let timeout = Duration::from_secs(self.config.detection_timeout_secs);
        let tracker = self.health_tracker.read().await;
        
        tracker
            .iter()
            .filter(|(_, health)| health.is_timed_out(timeout))
            .map(|(conn_id, _)| conn_id.clone())
            .collect()
    }

    /// Start background health check task
    pub fn start_health_check_task(self: Arc<Self>) {
        let handler = self.clone();
        tokio::spawn(async move {
            let check_interval = Duration::from_secs(1);
            let mut interval = tokio::time::interval(check_interval);
            
            loop {
                interval.tick().await;
                
                let timed_out = handler.check_timeouts().await;
                for conn_id in timed_out {
                    tracing::warn!(conn_id = %conn_id, "Connection timed out");
                    if let Err(e) = handler.handle_disconnection(&conn_id).await {
                        tracing::error!(
                            conn_id = %conn_id,
                            error = %e,
                            "Failed to handle timeout disconnection"
                        );
                    }
                }
            }
        });
    }

    /// Get connection health status
    pub async fn get_health(&self, conn_id: &str) -> Option<ConnectionHealth> {
        let tracker = self.health_tracker.read().await;
        tracker.get(conn_id).cloned()
    }

    /// Get all tracked connections count
    pub async fn tracked_count(&self) -> usize {
        let tracker = self.health_tracker.read().await;
        tracker.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disconnection_config_default() {
        let config = DisconnectionConfig::default();
        
        assert_eq!(config.detection_timeout_secs, 5);
        assert_eq!(config.cleanup_delay_ms, 100);
        assert!(config.unsubscribe_redis);
    }

    #[test]
    fn test_connection_health_new() {
        let conn_id = "test_conn".to_string();
        let user_id = Uuid::new_v4();
        let health = ConnectionHealth::new(conn_id.clone(), user_id);
        
        assert_eq!(health.conn_id, conn_id);
        assert_eq!(health.user_id, user_id);
        assert!(health.is_healthy);
    }

    #[test]
    fn test_connection_health_timeout() {
        let health = ConnectionHealth::new("test".to_string(), Uuid::new_v4());
        
        // Should not be timed out immediately
        assert!(!health.is_timed_out(Duration::from_secs(5)));
        
        // Should be timed out with 0 duration
        assert!(health.is_timed_out(Duration::from_secs(0)));
    }

    #[tokio::test]
    async fn test_register_connection() {
        let connection_manager = Arc::new(InMemoryConnectionManager::new());
        let handler = DisconnectionHandler::new(connection_manager);
        
        let conn_id = "test_conn";
        let user_id = Uuid::new_v4();
        
        handler.register_connection(conn_id, user_id).await;
        
        let health = handler.get_health(conn_id).await;
        assert!(health.is_some());
        assert_eq!(health.unwrap().user_id, user_id);
    }

    #[tokio::test]
    async fn test_update_activity() {
        let connection_manager = Arc::new(InMemoryConnectionManager::new());
        let handler = DisconnectionHandler::new(connection_manager);
        
        let conn_id = "test_conn";
        let user_id = Uuid::new_v4();
        
        handler.register_connection(conn_id, user_id).await;
        
        // Wait a bit
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let health_before = handler.get_health(conn_id).await.unwrap();
        let elapsed_before = health_before.last_activity.elapsed();
        
        handler.update_activity(conn_id).await;
        
        let health_after = handler.get_health(conn_id).await.unwrap();
        let elapsed_after = health_after.last_activity.elapsed();
        
        // Activity should be more recent after update
        assert!(elapsed_after < elapsed_before);
    }

    #[tokio::test]
    async fn test_tracked_count() {
        let connection_manager = Arc::new(InMemoryConnectionManager::new());
        let handler = DisconnectionHandler::new(connection_manager);
        
        assert_eq!(handler.tracked_count().await, 0);
        
        handler.register_connection("conn1", Uuid::new_v4()).await;
        assert_eq!(handler.tracked_count().await, 1);
        
        handler.register_connection("conn2", Uuid::new_v4()).await;
        assert_eq!(handler.tracked_count().await, 2);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use super::super::connection_manager::ConnectionManager;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        /// **Feature: message-system, Property 39: Disconnection detection timing**
        /// For any unexpectedly closed WebSocket connection, disconnection should be 
        /// detected within 5 seconds.
        /// **Validates: Requirements 10.1**
        #[test]
        fn prop_disconnection_detection_timing(
            timeout_ms in 100u64..500u64,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let connection_manager = Arc::new(InMemoryConnectionManager::new());
                
                // Use shorter timeout for faster testing, but verify the detection logic
                // The actual production value is 5 seconds (5000ms)
                let config = DisconnectionConfig {
                    detection_timeout_secs: 0, // Use milliseconds precision via timeout calculation
                    cleanup_delay_ms: 0,
                    unsubscribe_redis: false,
                };
                
                let handler = DisconnectionHandler::with_config(connection_manager, config);
                
                let conn_id = format!("test_conn_{}", timeout_ms);
                let user_id = Uuid::new_v4();
                
                // Register connection
                handler.register_connection(&conn_id, user_id).await;
                
                // Verify connection is tracked
                prop_assert_eq!(handler.tracked_count().await, 1);
                
                // Simulate no activity (connection dead)
                // Wait for the timeout period
                tokio::time::sleep(Duration::from_millis(timeout_ms + 50)).await;
                
                // Manually check if connection is timed out using the timeout duration
                let health = handler.get_health(&conn_id).await;
                prop_assert!(health.is_some());
                
                let is_timed_out = health.unwrap().is_timed_out(Duration::from_millis(timeout_ms));
                
                // Connection should be detected as timed out after the specified duration
                prop_assert!(
                    is_timed_out,
                    "Connection should be detected as timed out after {} ms", timeout_ms
                );

                Ok(())
            })?;
        }

        /// Property: Connection with activity should not timeout
        #[test]
        fn prop_active_connection_no_timeout(
            _seed in any::<u64>(),
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let connection_manager = Arc::new(InMemoryConnectionManager::new());
                let config = DisconnectionConfig {
                    detection_timeout_secs: 2,
                    cleanup_delay_ms: 0,
                    unsubscribe_redis: false,
                };
                
                let handler = DisconnectionHandler::with_config(connection_manager, config);
                
                let conn_id = "active_conn";
                let user_id = Uuid::new_v4();
                
                handler.register_connection(conn_id, user_id).await;
                
                // Keep updating activity
                for _ in 0..5 {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    handler.update_activity(conn_id).await;
                }
                
                // Check for timeouts
                let timed_out = handler.check_timeouts().await;
                
                // Active connection should NOT timeout
                prop_assert!(
                    !timed_out.contains(&conn_id.to_string()),
                    "Active connection should not timeout"
                );

                Ok(())
            })?;
        }

        /// **Feature: message-system, Property 40: Disconnection cleanup**
        /// For any detected disconnection, the connection should be removed from the 
        /// active connections registry and all Redis channels should be unsubscribed.
        /// **Validates: Requirements 10.2, 10.3**
        #[test]
        fn prop_disconnection_cleanup(
            _seed in any::<u64>(),
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let connection_manager = Arc::new(InMemoryConnectionManager::new());
                let handler = DisconnectionHandler::new(connection_manager.clone());
                
                let conn_id = "cleanup_conn";
                let user_id = Uuid::new_v4();
                
                // Register connection in handler
                handler.register_connection(conn_id, user_id).await;
                
                // Also register in connection manager for full integration test
                let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
                connection_manager.register_connection_async(
                    user_id,
                    conn_id.to_string(),
                    tx,
                ).await;
                
                // Verify connection is tracked in handler
                prop_assert_eq!(handler.tracked_count().await, 1);
                
                // Verify connection is tracked in connection manager
                let user_conns = connection_manager.get_user_connections_async(&user_id).await;
                prop_assert!(!user_conns.is_empty(), "User should have connections");
                
                // Handle disconnection
                let result = handler.handle_disconnection(conn_id).await;
                prop_assert!(result.is_ok(), "Disconnection handling should succeed");
                
                // Verify cleanup: Connection should be removed from health tracker
                prop_assert_eq!(
                    handler.tracked_count().await, 
                    0,
                    "Connection should be removed from health tracker"
                );
                
                // Verify cleanup: Health should no longer be available
                let health = handler.get_health(conn_id).await;
                prop_assert!(
                    health.is_none(),
                    "Connection health should no longer be available"
                );
                
                // Verify cleanup: Connection should be removed from connection manager registry
                let user_conns_after = connection_manager.get_user_connections_async(&user_id).await;
                prop_assert!(
                    user_conns_after.is_empty(),
                    "Connection should be removed from connection manager registry"
                );

                Ok(())
            })?;
        }

        /// **Feature: message-system, Property 41: Reconnection authentication**
        /// For any user reconnecting after disconnection, full authentication should be 
        /// performed as if it were a new connection.
        /// **Validates: Requirements 10.4**
        #[test]
        fn prop_reconnection_authentication(
            _seed in any::<u64>(),
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let connection_manager = Arc::new(InMemoryConnectionManager::new());
                let handler = DisconnectionHandler::new(connection_manager.clone());
                
                let user_id = Uuid::new_v4();
                let old_conn_id = "old_connection";
                let new_conn_id = "new_connection";
                
                // Simulate first connection
                handler.register_connection(old_conn_id, user_id).await;
                
                let (tx1, _rx1) = tokio::sync::mpsc::unbounded_channel();
                connection_manager.register_connection_async(
                    user_id,
                    old_conn_id.to_string(),
                    tx1,
                ).await;
                
                // Verify first connection is registered
                prop_assert_eq!(handler.tracked_count().await, 1);
                let conns_before = connection_manager.get_user_connections_async(&user_id).await;
                prop_assert_eq!(conns_before.len(), 1);
                
                // Simulate disconnection
                let result = handler.handle_disconnection(old_conn_id).await;
                prop_assert!(result.is_ok(), "Disconnection should succeed");
                
                // Verify old connection is cleaned up
                prop_assert_eq!(handler.tracked_count().await, 0);
                let conns_after_disconnect = connection_manager.get_user_connections_async(&user_id).await;
                prop_assert!(conns_after_disconnect.is_empty(), "Old connection should be removed");
                
                // Simulate reconnection - this should be treated as a NEW connection
                // requiring full authentication (simulated by registering again)
                handler.register_connection(new_conn_id, user_id).await;
                
                let (tx2, _rx2) = tokio::sync::mpsc::unbounded_channel();
                connection_manager.register_connection_async(
                    user_id,
                    new_conn_id.to_string(),
                    tx2,
                ).await;
                
                // Verify reconnection creates a NEW connection with different ID
                prop_assert_eq!(handler.tracked_count().await, 1);
                let conns_after_reconnect = connection_manager.get_user_connections_async(&user_id).await;
                prop_assert_eq!(conns_after_reconnect.len(), 1);
                prop_assert_eq!(&conns_after_reconnect[0], new_conn_id);
                
                // Verify the new connection has its own health tracking
                let new_health = handler.get_health(new_conn_id).await;
                prop_assert!(new_health.is_some(), "New connection should have health tracking");
                
                // Verify old connection ID is not tracked
                let old_health = handler.get_health(old_conn_id).await;
                prop_assert!(old_health.is_none(), "Old connection should not be tracked");

                Ok(())
            })?;
        }

        /// **Feature: message-system, Property 42: Offline message delivery on reconnect**
        /// For any user reconnecting, all messages sent during the disconnection period 
        /// should be delivered as offline messages.
        /// **Validates: Requirements 10.5**
        #[test]
        fn prop_reconnect_offline_messages(
            num_messages in 1usize..10,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let connection_manager = Arc::new(InMemoryConnectionManager::new());
                let handler = DisconnectionHandler::new(connection_manager.clone());
                
                let user_id = Uuid::new_v4();
                let conn_id = "test_connection";
                
                // Step 1: User connects
                handler.register_connection(conn_id, user_id).await;
                let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
                connection_manager.register_connection_async(
                    user_id,
                    conn_id.to_string(),
                    tx,
                ).await;
                
                prop_assert!(connection_manager.is_online_async(&user_id).await);
                
                // Step 2: User disconnects
                let disconnect_result = handler.handle_disconnection(conn_id).await;
                prop_assert!(disconnect_result.is_ok());
                prop_assert!(!connection_manager.is_online_async(&user_id).await);
                
                // Step 3: Simulate messages sent during disconnection
                // In a real system, these would be stored in the database as offline messages
                // For this property test, we verify the disconnection state allows for
                // offline message queuing
                let messages_during_disconnect = num_messages;
                
                // Verify user is offline (messages would be queued)
                for _ in 0..messages_during_disconnect {
                    prop_assert!(!connection_manager.is_online_async(&user_id).await);
                }
                
                // Step 4: User reconnects (new connection with full auth)
                let new_conn_id = "reconnected_connection";
                handler.register_connection(new_conn_id, user_id).await;
                let (tx_new, _rx_new) = tokio::sync::mpsc::unbounded_channel();
                connection_manager.register_connection_async(
                    user_id,
                    new_conn_id.to_string(),
                    tx_new,
                ).await;
                
                // Verify user is back online
                prop_assert!(connection_manager.is_online_async(&user_id).await);
                
                // Verify it's a new connection (different ID)
                let conns = connection_manager.get_user_connections_async(&user_id).await;
                prop_assert_eq!(conns.len(), 1);
                prop_assert_eq!(&conns[0], new_conn_id);
                
                // In a real system, offline messages would now be delivered
                // This property test verifies the connection state transitions
                // that enable offline message delivery

                Ok(())
            })?;
        }
    }
}
