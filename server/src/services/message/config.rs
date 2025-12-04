//! Message System Configuration Module
//! 
//! Centralized configuration for all message system components.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Complete message system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSystemConfig {
    /// WebSocket configuration
    pub websocket: WebSocketSettings,
    /// Message routing configuration
    pub routing: RoutingSettings,
    /// Persistence configuration
    pub persistence: PersistenceSettings,
    /// Redis configuration
    pub redis: RedisSettings,
    /// Offline sync configuration
    pub offline_sync: OfflineSyncSettings,
    /// Concurrent processing configuration
    pub concurrent: ConcurrentSettings,
    /// Metrics configuration
    pub metrics: MetricsSettings,
}

impl Default for MessageSystemConfig {
    fn default() -> Self {
        Self {
            websocket: WebSocketSettings::default(),
            routing: RoutingSettings::default(),
            persistence: PersistenceSettings::default(),
            redis: RedisSettings::default(),
            offline_sync: OfflineSyncSettings::default(),
            concurrent: ConcurrentSettings::default(),
            metrics: MetricsSettings::default(),
        }
    }
}

/// WebSocket settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketSettings {
    /// Authentication timeout in seconds
    pub auth_timeout_secs: u64,
    /// Heartbeat interval in seconds
    pub heartbeat_interval_secs: u64,
    /// Heartbeat timeout in seconds
    pub heartbeat_timeout_secs: u64,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Connection idle timeout in seconds
    pub idle_timeout_secs: u64,
}

impl Default for WebSocketSettings {
    fn default() -> Self {
        Self {
            auth_timeout_secs: 3,
            heartbeat_interval_secs: 30,
            heartbeat_timeout_secs: 10,
            max_message_size: 1024 * 1024, // 1MB
            idle_timeout_secs: 60,
        }
    }
}

/// Message routing settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingSettings {
    /// Maximum routing latency in milliseconds
    pub max_routing_latency_ms: u64,
    /// Enable Redis pub/sub
    pub enable_redis_pubsub: bool,
    /// Deduplication cache TTL in seconds
    pub dedup_cache_ttl_secs: u64,
    /// Maximum text message length
    pub max_text_length: usize,
}

impl Default for RoutingSettings {
    fn default() -> Self {
        Self {
            max_routing_latency_ms: 50,
            enable_redis_pubsub: true,
            dedup_cache_ttl_secs: 600, // 10 minutes
            max_text_length: 10000,
        }
    }
}

/// Persistence settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceSettings {
    /// Maximum persistence latency in milliseconds
    pub max_persistence_latency_ms: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Retry delays in milliseconds
    pub retry_delays_ms: Vec<u64>,
    /// Connection pool size
    pub pool_size: u32,
}

impl Default for PersistenceSettings {
    fn default() -> Self {
        Self {
            max_persistence_latency_ms: 200,
            max_retries: 3,
            retry_delays_ms: vec![100, 500, 2000],
            pool_size: 10,
        }
    }
}

/// Redis settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisSettings {
    /// Redis URL
    pub url: String,
    /// Enable fallback to database polling
    pub enable_fallback: bool,
    /// Fallback polling interval in milliseconds
    pub fallback_polling_interval_ms: u64,
    /// Health check interval in milliseconds
    pub health_check_interval_ms: u64,
}

impl Default for RedisSettings {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            enable_fallback: true,
            fallback_polling_interval_ms: 1000,
            health_check_interval_ms: 5000,
        }
    }
}

/// Offline sync settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineSyncSettings {
    /// Batch size for offline message retrieval
    pub batch_size: i32,
    /// Delay between batches in milliseconds
    pub batch_delay_ms: u64,
    /// Maximum messages to sync per connection
    pub max_messages: i32,
}

impl Default for OfflineSyncSettings {
    fn default() -> Self {
        Self {
            batch_size: 100,
            batch_delay_ms: 500,
            max_messages: 10000,
        }
    }
}

/// Concurrent processing settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrentSettings {
    /// Maximum concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Queue size per conversation
    pub queue_size_per_conversation: usize,
    /// Processing timeout in milliseconds
    pub processing_timeout_ms: u64,
    /// Preserve conversation ordering
    pub preserve_conversation_order: bool,
}

impl Default for ConcurrentSettings {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 1000,
            queue_size_per_conversation: 100,
            processing_timeout_ms: 5000,
            preserve_conversation_order: true,
        }
    }
}

/// Metrics settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSettings {
    /// Enable metrics collection
    pub enabled: bool,
    /// Latency sample size
    pub latency_sample_size: usize,
    /// Expose Prometheus endpoint
    pub expose_prometheus: bool,
}

impl Default for MetricsSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            latency_sample_size: 1000,
            expose_prometheus: true,
        }
    }
}

impl MessageSystemConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // WebSocket settings
        if let Ok(val) = std::env::var("WS_AUTH_TIMEOUT_SECS") {
            if let Ok(v) = val.parse() {
                config.websocket.auth_timeout_secs = v;
            }
        }
        if let Ok(val) = std::env::var("WS_HEARTBEAT_INTERVAL_SECS") {
            if let Ok(v) = val.parse() {
                config.websocket.heartbeat_interval_secs = v;
            }
        }
        
        // Redis settings
        if let Ok(val) = std::env::var("REDIS_URL") {
            config.redis.url = val;
        }
        
        // Persistence settings
        if let Ok(val) = std::env::var("DB_POOL_SIZE") {
            if let Ok(v) = val.parse() {
                config.persistence.pool_size = v;
            }
        }
        
        // Concurrent settings
        if let Ok(val) = std::env::var("MAX_CONCURRENT_TASKS") {
            if let Ok(v) = val.parse() {
                config.concurrent.max_concurrent_tasks = v;
            }
        }
        
        config
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // WebSocket validation
        if self.websocket.auth_timeout_secs == 0 {
            return Err(ConfigError::InvalidValue("auth_timeout_secs must be > 0".into()));
        }
        if self.websocket.heartbeat_timeout_secs >= self.websocket.idle_timeout_secs {
            return Err(ConfigError::InvalidValue(
                "heartbeat_timeout_secs must be < idle_timeout_secs".into(),
            ));
        }
        
        // Routing validation
        if self.routing.max_text_length == 0 {
            return Err(ConfigError::InvalidValue("max_text_length must be > 0".into()));
        }
        
        // Persistence validation
        if self.persistence.max_retries == 0 {
            return Err(ConfigError::InvalidValue("max_retries must be > 0".into()));
        }
        
        // Concurrent validation
        if self.concurrent.max_concurrent_tasks == 0 {
            return Err(ConfigError::InvalidValue("max_concurrent_tasks must be > 0".into()));
        }
        
        Ok(())
    }
}

/// Configuration error
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
    
    #[error("Missing required configuration: {0}")]
    MissingRequired(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MessageSystemConfig::default();
        
        assert_eq!(config.websocket.auth_timeout_secs, 3);
        assert_eq!(config.routing.max_text_length, 10000);
        assert_eq!(config.persistence.max_retries, 3);
        assert!(config.redis.enable_fallback);
    }

    #[test]
    fn test_config_validation_success() {
        let config = MessageSystemConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_failure() {
        let mut config = MessageSystemConfig::default();
        config.websocket.auth_timeout_secs = 0;
        
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_websocket_settings_default() {
        let settings = WebSocketSettings::default();
        
        assert_eq!(settings.auth_timeout_secs, 3);
        assert_eq!(settings.heartbeat_interval_secs, 30);
        assert_eq!(settings.max_message_size, 1024 * 1024);
    }

    #[test]
    fn test_redis_settings_default() {
        let settings = RedisSettings::default();
        
        assert_eq!(settings.url, "redis://localhost:6379");
        assert!(settings.enable_fallback);
        assert_eq!(settings.fallback_polling_interval_ms, 1000);
    }

    #[test]
    fn test_concurrent_settings_default() {
        let settings = ConcurrentSettings::default();
        
        assert_eq!(settings.max_concurrent_tasks, 1000);
        assert!(settings.preserve_conversation_order);
    }
}


#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: WebSocket settings validation
        #[test]
        fn prop_websocket_settings_valid(
            auth_timeout in 1u64..60,
            heartbeat_interval in 10u64..120,
            heartbeat_timeout in 5u64..30,
            max_message_size in 1024usize..10485760,
            idle_timeout in 30u64..300,
        ) {
            // Ensure heartbeat_timeout < idle_timeout
            let idle_timeout = idle_timeout.max(heartbeat_timeout + 1);
            
            let settings = WebSocketSettings {
                auth_timeout_secs: auth_timeout,
                heartbeat_interval_secs: heartbeat_interval,
                heartbeat_timeout_secs: heartbeat_timeout,
                max_message_size,
                idle_timeout_secs: idle_timeout,
            };

            prop_assert!(settings.auth_timeout_secs > 0);
            prop_assert!(settings.heartbeat_timeout_secs < settings.idle_timeout_secs);
        }

        /// Property: Routing settings validation
        #[test]
        fn prop_routing_settings_valid(
            max_latency in 10u64..1000,
            dedup_ttl in 60u64..3600,
            max_text_length in 100usize..100000,
        ) {
            let settings = RoutingSettings {
                max_routing_latency_ms: max_latency,
                enable_redis_pubsub: true,
                dedup_cache_ttl_secs: dedup_ttl,
                max_text_length,
            };

            prop_assert!(settings.max_text_length > 0);
            prop_assert!(settings.dedup_cache_ttl_secs > 0);
        }

        /// Property: Persistence settings validation
        #[test]
        fn prop_persistence_settings_valid(
            max_latency in 50u64..5000,
            max_retries in 1u32..10,
            pool_size in 1u32..100,
        ) {
            let settings = PersistenceSettings {
                max_persistence_latency_ms: max_latency,
                max_retries,
                retry_delays_ms: vec![100, 500, 2000],
                pool_size,
            };

            prop_assert!(settings.max_retries > 0);
            prop_assert!(settings.pool_size > 0);
        }

        /// Property: Config validation catches invalid values
        #[test]
        fn prop_config_validation_catches_invalid(_seed in any::<u64>()) {
            let mut config = MessageSystemConfig::default();
            
            // Valid config should pass
            prop_assert!(config.validate().is_ok());
            
            // Invalid auth_timeout should fail
            config.websocket.auth_timeout_secs = 0;
            prop_assert!(config.validate().is_err());
            
            // Reset and test another invalid case
            config = MessageSystemConfig::default();
            config.routing.max_text_length = 0;
            prop_assert!(config.validate().is_err());
        }

        /// Property: Default config is always valid
        #[test]
        fn prop_default_config_valid(_seed in any::<u64>()) {
            let config = MessageSystemConfig::default();
            prop_assert!(config.validate().is_ok());
        }

        /// Property: Redis settings have reasonable defaults
        #[test]
        fn prop_redis_settings_defaults(_seed in any::<u64>()) {
            let settings = RedisSettings::default();
            
            prop_assert!(!settings.url.is_empty());
            prop_assert!(settings.fallback_polling_interval_ms > 0);
            prop_assert!(settings.health_check_interval_ms > 0);
        }
    }
}
