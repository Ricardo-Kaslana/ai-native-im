//! Comprehensive Error Handling Module
//! 
//! Provides retry strategies, graceful degradation, and error isolation for the message system.

use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

/// Message system error categories
#[derive(Debug, Error)]
pub enum MessageSystemError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),
    
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Database-specific errors
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Connection pool exhausted")]
    PoolExhausted,
    
    #[error("Query timeout")]
    QueryTimeout,
    
    #[error("Connection lost")]
    ConnectionLost,
    
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    
    #[error("Query error: {0}")]
    QueryError(String),
}

/// Redis-specific errors
#[derive(Debug, Error)]
pub enum RedisError {
    #[error("Connection failed")]
    ConnectionFailed,
    
    #[error("Publish failed")]
    PublishFailed,
    
    #[error("Subscribe failed")]
    SubscribeFailed,
    
    #[error("Timeout")]
    Timeout,
    
    #[error("Cluster error: {0}")]
    ClusterError(String),
}

/// Connection-specific errors
#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("Authentication failed")]
    AuthFailed,
    
    #[error("Connection closed")]
    Closed,
    
    #[error("Heartbeat timeout")]
    HeartbeatTimeout,
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
}

/// Error classification for retry decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    /// Transient error, should retry
    Transient,
    /// Permanent error, should not retry
    Permanent,
    /// Resource exhaustion, apply backpressure
    ResourceExhaustion,
    /// Unknown, use default behavior
    Unknown,
}

impl MessageSystemError {
    /// Classify the error for retry decisions
    pub fn classify(&self) -> ErrorClass {
        match self {
            MessageSystemError::Database(db_err) => match db_err {
                DatabaseError::PoolExhausted => ErrorClass::ResourceExhaustion,
                DatabaseError::QueryTimeout => ErrorClass::Transient,
                DatabaseError::ConnectionLost => ErrorClass::Transient,
                DatabaseError::ConstraintViolation(_) => ErrorClass::Permanent,
                DatabaseError::QueryError(_) => ErrorClass::Unknown,
            },
            MessageSystemError::Redis(redis_err) => match redis_err {
                RedisError::ConnectionFailed => ErrorClass::Transient,
                RedisError::PublishFailed => ErrorClass::Transient,
                RedisError::SubscribeFailed => ErrorClass::Transient,
                RedisError::Timeout => ErrorClass::Transient,
                RedisError::ClusterError(_) => ErrorClass::Unknown,
            },
            MessageSystemError::Connection(conn_err) => match conn_err {
                ConnectionError::AuthFailed => ErrorClass::Permanent,
                ConnectionError::Closed => ErrorClass::Permanent,
                ConnectionError::HeartbeatTimeout => ErrorClass::Transient,
                ConnectionError::ProtocolError(_) => ErrorClass::Permanent,
            },
            MessageSystemError::Validation(_) => ErrorClass::Permanent,
            MessageSystemError::Timeout(_) => ErrorClass::Transient,
            MessageSystemError::Internal(_) => ErrorClass::Unknown,
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self.classify(), ErrorClass::Transient)
    }
}

/// Retry strategy configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Add jitter to delays
    pub add_jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            add_jitter: true,
        }
    }
}

/// Retry executor with exponential backoff
pub struct RetryExecutor {
    config: RetryConfig,
}

impl RetryExecutor {
    /// Create a new retry executor
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute an async operation with retries
    pub async fn execute<F, Fut, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let mut attempt = 0;
        let mut delay = self.config.initial_delay;

        loop {
            attempt += 1;
            
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt >= self.config.max_attempts {
                        tracing::error!(
                            attempt = attempt,
                            error = %e,
                            "All retry attempts exhausted"
                        );
                        return Err(e);
                    }

                    tracing::warn!(
                        attempt = attempt,
                        max_attempts = self.config.max_attempts,
                        error = %e,
                        delay_ms = delay.as_millis(),
                        "Operation failed, retrying"
                    );

                    // Apply jitter if configured
                    let actual_delay = if self.config.add_jitter {
                        // Simple jitter using system time nanoseconds
                        let nanos = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.subsec_nanos())
                            .unwrap_or(0);
                        let jitter = (nanos % 300) as f64 / 1000.0; // 0-0.3 jitter
                        Duration::from_secs_f64(delay.as_secs_f64() * (1.0 + jitter))
                    } else {
                        delay
                    };

                    sleep(actual_delay).await;

                    // Calculate next delay with exponential backoff
                    delay = Duration::from_secs_f64(
                        (delay.as_secs_f64() * self.config.backoff_multiplier)
                            .min(self.config.max_delay.as_secs_f64()),
                    );
                }
            }
        }
    }

    /// Execute with custom retry predicate
    pub async fn execute_with_predicate<F, Fut, T, E, P>(
        &self,
        mut operation: F,
        should_retry: P,
    ) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
        P: Fn(&E) -> bool,
    {
        let mut attempt = 0;
        let mut delay = self.config.initial_delay;

        loop {
            attempt += 1;
            
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt >= self.config.max_attempts || !should_retry(&e) {
                        return Err(e);
                    }

                    tracing::warn!(
                        attempt = attempt,
                        error = %e,
                        "Operation failed, retrying"
                    );

                    sleep(delay).await;
                    delay = Duration::from_secs_f64(
                        (delay.as_secs_f64() * self.config.backoff_multiplier)
                            .min(self.config.max_delay.as_secs_f64()),
                    );
                }
            }
        }
    }
}

/// Graceful degradation handler
pub struct DegradationHandler {
    /// Current degradation level
    level: std::sync::atomic::AtomicU8,
}

/// Degradation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DegradationLevel {
    /// Normal operation
    Normal = 0,
    /// Redis unavailable, using database polling
    RedisDown = 1,
    /// High load, applying rate limiting
    HighLoad = 2,
    /// Critical, only essential operations
    Critical = 3,
}

impl From<u8> for DegradationLevel {
    fn from(value: u8) -> Self {
        match value {
            0 => DegradationLevel::Normal,
            1 => DegradationLevel::RedisDown,
            2 => DegradationLevel::HighLoad,
            3 => DegradationLevel::Critical,
            _ => DegradationLevel::Normal,
        }
    }
}

impl DegradationHandler {
    /// Create a new degradation handler
    pub fn new() -> Self {
        Self {
            level: std::sync::atomic::AtomicU8::new(0),
        }
    }

    /// Get current degradation level
    pub fn level(&self) -> DegradationLevel {
        DegradationLevel::from(self.level.load(std::sync::atomic::Ordering::SeqCst))
    }

    /// Set degradation level
    pub fn set_level(&self, level: DegradationLevel) {
        self.level.store(level as u8, std::sync::atomic::Ordering::SeqCst);
        tracing::warn!(level = ?level, "Degradation level changed");
    }

    /// Check if operation should proceed
    pub fn should_proceed(&self, priority: OperationPriority) -> bool {
        let level = self.level();
        match level {
            DegradationLevel::Normal => true,
            DegradationLevel::RedisDown => true,
            DegradationLevel::HighLoad => priority >= OperationPriority::Normal,
            DegradationLevel::Critical => priority >= OperationPriority::High,
        }
    }

    /// Upgrade degradation level
    pub fn upgrade(&self) {
        let current = self.level.load(std::sync::atomic::Ordering::SeqCst);
        if current < 3 {
            self.level.store(current + 1, std::sync::atomic::Ordering::SeqCst);
        }
    }

    /// Downgrade degradation level
    pub fn downgrade(&self) {
        let current = self.level.load(std::sync::atomic::Ordering::SeqCst);
        if current > 0 {
            self.level.store(current - 1, std::sync::atomic::Ordering::SeqCst);
        }
    }
}

impl Default for DegradationHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Operation priority for degradation decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperationPriority {
    /// Low priority, can be dropped
    Low = 0,
    /// Normal priority
    Normal = 1,
    /// High priority, should proceed
    High = 2,
    /// Critical, must proceed
    Critical = 3,
}

/// Error isolation wrapper
pub struct ErrorIsolator {
    /// Error counts per connection
    error_counts: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<uuid::Uuid, u32>>>,
    /// Maximum errors before isolation
    max_errors: u32,
}

impl ErrorIsolator {
    /// Create a new error isolator
    pub fn new(max_errors: u32) -> Self {
        Self {
            error_counts: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            max_errors,
        }
    }

    /// Record an error for a connection
    pub async fn record_error(&self, connection_id: uuid::Uuid) -> bool {
        let mut counts = self.error_counts.write().await;
        let count = counts.entry(connection_id).or_insert(0);
        *count += 1;
        
        if *count >= self.max_errors {
            tracing::warn!(
                connection_id = %connection_id,
                error_count = *count,
                "Connection isolated due to excessive errors"
            );
            true
        } else {
            false
        }
    }

    /// Clear errors for a connection
    pub async fn clear_errors(&self, connection_id: uuid::Uuid) {
        let mut counts = self.error_counts.write().await;
        counts.remove(&connection_id);
    }

    /// Check if connection is isolated
    pub async fn is_isolated(&self, connection_id: uuid::Uuid) -> bool {
        let counts = self.error_counts.read().await;
        counts.get(&connection_id).map_or(false, |&c| c >= self.max_errors)
    }

    /// Get error count for connection
    pub async fn error_count(&self, connection_id: uuid::Uuid) -> u32 {
        let counts = self.error_counts.read().await;
        counts.get(&connection_id).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_classification() {
        let db_pool_err = MessageSystemError::Database(DatabaseError::PoolExhausted);
        assert_eq!(db_pool_err.classify(), ErrorClass::ResourceExhaustion);
        
        let db_timeout_err = MessageSystemError::Database(DatabaseError::QueryTimeout);
        assert_eq!(db_timeout_err.classify(), ErrorClass::Transient);
        assert!(db_timeout_err.is_retryable());
        
        let validation_err = MessageSystemError::Validation("invalid".to_string());
        assert_eq!(validation_err.classify(), ErrorClass::Permanent);
        assert!(!validation_err.is_retryable());
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert!(config.add_jitter);
    }

    #[tokio::test]
    async fn test_retry_executor_success() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            add_jitter: false,
        };
        let executor = RetryExecutor::new(config);
        
        let result: Result<i32, &str> = executor.execute(|| async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_executor_failure() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            backoff_multiplier: 2.0,
            add_jitter: false,
        };
        let executor = RetryExecutor::new(config);
        
        let result: Result<i32, &str> = executor.execute(|| async { Err("always fails") }).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_degradation_handler() {
        let handler = DegradationHandler::new();
        
        assert_eq!(handler.level(), DegradationLevel::Normal);
        
        handler.set_level(DegradationLevel::HighLoad);
        assert_eq!(handler.level(), DegradationLevel::HighLoad);
        
        handler.upgrade();
        assert_eq!(handler.level(), DegradationLevel::Critical);
        
        handler.downgrade();
        assert_eq!(handler.level(), DegradationLevel::HighLoad);
    }

    #[test]
    fn test_degradation_should_proceed() {
        let handler = DegradationHandler::new();
        
        // Normal level - all proceed
        assert!(handler.should_proceed(OperationPriority::Low));
        
        // High load - only normal and above
        handler.set_level(DegradationLevel::HighLoad);
        assert!(!handler.should_proceed(OperationPriority::Low));
        assert!(handler.should_proceed(OperationPriority::Normal));
        
        // Critical - only high and above
        handler.set_level(DegradationLevel::Critical);
        assert!(!handler.should_proceed(OperationPriority::Normal));
        assert!(handler.should_proceed(OperationPriority::High));
    }

    #[tokio::test]
    async fn test_error_isolator() {
        let isolator = ErrorIsolator::new(3);
        let conn_id = uuid::Uuid::new_v4();
        
        // Record errors
        assert!(!isolator.record_error(conn_id).await);
        assert!(!isolator.record_error(conn_id).await);
        assert!(isolator.record_error(conn_id).await); // Third error triggers isolation
        
        assert!(isolator.is_isolated(conn_id).await);
        assert_eq!(isolator.error_count(conn_id).await, 3);
        
        // Clear errors
        isolator.clear_errors(conn_id).await;
        assert!(!isolator.is_isolated(conn_id).await);
    }
}


#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Property: Retry config respects bounds
        #[test]
        fn prop_retry_config_bounds(
            max_attempts in 1u32..10,
            initial_delay_ms in 10u64..1000,
            max_delay_ms in 1000u64..60000,
            backoff_multiplier in 1.0f64..5.0,
        ) {
            let config = RetryConfig {
                max_attempts,
                initial_delay: Duration::from_millis(initial_delay_ms),
                max_delay: Duration::from_millis(max_delay_ms),
                backoff_multiplier,
                add_jitter: true,
            };

            prop_assert_eq!(config.max_attempts, max_attempts);
            prop_assert!(config.initial_delay.as_millis() as u64 == initial_delay_ms);
            prop_assert!(config.max_delay.as_millis() as u64 == max_delay_ms);
        }

        /// Property: Degradation levels are ordered
        #[test]
        fn prop_degradation_level_ordering(_seed in any::<u64>()) {
            let handler = DegradationHandler::new();
            
            // Start at Normal
            prop_assert_eq!(handler.level(), DegradationLevel::Normal);
            
            // Upgrade should increase level
            handler.upgrade();
            prop_assert_eq!(handler.level(), DegradationLevel::RedisDown);
            
            handler.upgrade();
            prop_assert_eq!(handler.level(), DegradationLevel::HighLoad);
            
            handler.upgrade();
            prop_assert_eq!(handler.level(), DegradationLevel::Critical);
            
            // Can't go higher than Critical
            handler.upgrade();
            prop_assert_eq!(handler.level(), DegradationLevel::Critical);
            
            // Downgrade should decrease level
            handler.downgrade();
            prop_assert_eq!(handler.level(), DegradationLevel::HighLoad);
        }

        /// Property: Error isolation tracks errors correctly
        #[test]
        fn prop_error_isolation_tracking(
            max_errors in 1u32..10,
            error_count in 0u32..20,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let isolator = ErrorIsolator::new(max_errors);
                let conn_id = uuid::Uuid::new_v4();

                for i in 0..error_count {
                    let should_isolate = isolator.record_error(conn_id).await;
                    
                    if i + 1 >= max_errors {
                        prop_assert!(should_isolate || isolator.is_isolated(conn_id).await);
                    }
                }

                let final_count = isolator.error_count(conn_id).await;
                prop_assert_eq!(final_count, error_count);

                Ok(())
            })?;
        }

        /// Property: Error classification is consistent
        #[test]
        fn prop_error_classification_consistent(_seed in any::<u64>()) {
            // Transient errors should be retryable
            let timeout_err = MessageSystemError::Timeout("test".to_string());
            prop_assert_eq!(timeout_err.classify(), ErrorClass::Transient);
            prop_assert!(timeout_err.is_retryable());

            // Permanent errors should not be retryable
            let validation_err = MessageSystemError::Validation("test".to_string());
            prop_assert_eq!(validation_err.classify(), ErrorClass::Permanent);
            prop_assert!(!validation_err.is_retryable());

            // Resource exhaustion errors
            let pool_err = MessageSystemError::Database(DatabaseError::PoolExhausted);
            prop_assert_eq!(pool_err.classify(), ErrorClass::ResourceExhaustion);
        }

        /// Property: Operation priority ordering
        #[test]
        fn prop_operation_priority_ordering(_seed in any::<u64>()) {
            prop_assert!(OperationPriority::Low < OperationPriority::Normal);
            prop_assert!(OperationPriority::Normal < OperationPriority::High);
            prop_assert!(OperationPriority::High < OperationPriority::Critical);
        }

        /// Property: Degradation should_proceed respects priority
        #[test]
        fn prop_degradation_should_proceed(_seed in any::<u64>()) {
            let handler = DegradationHandler::new();

            // Normal level - all proceed
            prop_assert!(handler.should_proceed(OperationPriority::Low));
            prop_assert!(handler.should_proceed(OperationPriority::Critical));

            // High load - only normal and above
            handler.set_level(DegradationLevel::HighLoad);
            prop_assert!(!handler.should_proceed(OperationPriority::Low));
            prop_assert!(handler.should_proceed(OperationPriority::Normal));
            prop_assert!(handler.should_proceed(OperationPriority::High));

            // Critical - only high and above
            handler.set_level(DegradationLevel::Critical);
            prop_assert!(!handler.should_proceed(OperationPriority::Low));
            prop_assert!(!handler.should_proceed(OperationPriority::Normal));
            prop_assert!(handler.should_proceed(OperationPriority::High));
            prop_assert!(handler.should_proceed(OperationPriority::Critical));
        }
    }
}
