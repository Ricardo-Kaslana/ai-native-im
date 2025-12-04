use redis::{aio::ConnectionManager, AsyncCommands};
use thiserror::Error;

/// Rate limiter errors
#[derive(Debug, Error)]
pub enum RateLimiterError {
    #[error("Account is locked")]
    AccountLocked,
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
}

/// Rate limiter for preventing brute force attacks
#[derive(Clone)]
pub struct RateLimiter {
    redis: ConnectionManager,
    max_attempts: u32,
    lockout_duration_seconds: u64,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(redis: ConnectionManager, max_attempts: u32, lockout_duration_seconds: u64) -> Self {
        Self {
            redis,
            max_attempts,
            lockout_duration_seconds,
        }
    }

    /// Create with default settings (5 attempts, 15 min lockout)
    pub fn with_defaults(redis: ConnectionManager) -> Self {
        Self::new(redis, 5, 900)
    }

    /// Get the Redis key for failed attempts
    fn failed_attempts_key(username: &str) -> String {
        format!("auth:failed_attempts:{}", username)
    }

    /// Get the Redis key for lockout
    fn lockout_key(username: &str) -> String {
        format!("auth:lockout:{}", username)
    }

    /// Check if account is locked
    pub async fn is_locked(&mut self, username: &str) -> Result<bool, RateLimiterError> {
        let key = Self::lockout_key(username);
        let locked: bool = self.redis.exists(&key).await?;
        Ok(locked)
    }

    /// Get remaining lockout time in seconds
    pub async fn get_lockout_remaining(
        &mut self,
        username: &str,
    ) -> Result<Option<u64>, RateLimiterError> {
        let key = Self::lockout_key(username);
        let ttl: i64 = self.redis.ttl(&key).await?;

        if ttl > 0 {
            Ok(Some(ttl as u64))
        } else {
            Ok(None)
        }
    }

    /// Record a failed login attempt
    pub async fn record_failed_attempt(&mut self, username: &str) -> Result<u32, RateLimiterError> {
        let key = Self::failed_attempts_key(username);

        // Increment counter
        let count: u32 = self.redis.incr(&key, 1).await?;

        // Set expiry on first attempt (15 minutes)
        if count == 1 {
            let _: () = self.redis.expire(&key, 900).await?;
        }

        // Lock account if max attempts reached
        if count >= self.max_attempts {
            self.lock_account(username).await?;
        }

        Ok(count)
    }

    /// Lock an account
    async fn lock_account(&mut self, username: &str) -> Result<(), RateLimiterError> {
        let key = Self::lockout_key(username);
        let _: () = self
            .redis
            .set_ex(&key, "locked", self.lockout_duration_seconds)
            .await?;
        Ok(())
    }

    /// Reset failed attempts counter (called on successful login)
    pub async fn reset_attempts(&mut self, username: &str) -> Result<(), RateLimiterError> {
        let key = Self::failed_attempts_key(username);
        let _: () = self.redis.del(&key).await?;
        Ok(())
    }

    /// Get current failed attempts count
    pub async fn get_failed_attempts(&mut self, username: &str) -> Result<u32, RateLimiterError> {
        let key = Self::failed_attempts_key(username);
        let count: Option<u32> = self.redis.get(&key).await?;
        Ok(count.unwrap_or(0))
    }

    /// Manually unlock an account (admin function)
    pub async fn unlock_account(&mut self, username: &str) -> Result<(), RateLimiterError> {
        let lockout_key = Self::lockout_key(username);
        let attempts_key = Self::failed_attempts_key(username);

        let _: () = self.redis.del(&lockout_key).await?;
        let _: () = self.redis.del(&attempts_key).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running Redis instance
    // They are integration tests and should be run with --ignored flag

    #[test]
    fn test_key_generation() {
        assert_eq!(
            RateLimiter::failed_attempts_key("testuser"),
            "auth:failed_attempts:testuser"
        );
        assert_eq!(
            RateLimiter::lockout_key("testuser"),
            "auth:lockout:testuser"
        );
    }
}
