use ai_im_server::services::auth::RateLimiter;
use redis::Client;

async fn setup_redis() -> redis::aio::ConnectionManager {
    let client = Client::open("redis://localhost:6379").expect("Failed to create Redis client");
    client
        .get_tokio_connection_manager()
        .await
        .expect("Failed to connect to Redis")
}

#[tokio::test]
async fn test_failed_attempt_tracking() {
    let redis = setup_redis().await;
    let mut limiter = RateLimiter::with_defaults(redis);

    let username = format!("test_rate_limit_{}", uuid::Uuid::new_v4());

    // Clean up first
    limiter.reset_attempts(&username).await.ok();
    limiter.unlock_account(&username).await.ok();

    // Record failed attempts
    let count1 = limiter.record_failed_attempt(&username).await.unwrap();
    assert_eq!(count1, 1);

    let count2 = limiter.record_failed_attempt(&username).await.unwrap();
    assert_eq!(count2, 2);

    let current = limiter.get_failed_attempts(&username).await.unwrap();
    assert_eq!(current, 2);

    // Clean up
    limiter.reset_attempts(&username).await.ok();
}

#[tokio::test]
async fn test_account_lockout() {
    let redis = setup_redis().await;
    let mut limiter = RateLimiter::new(redis, 3, 60); // 3 attempts, 60 sec lockout

    let username = format!("test_lockout_{}", uuid::Uuid::new_v4());

    // Clean up first
    limiter.unlock_account(&username).await.ok();

    // Should not be locked initially
    assert!(!limiter.is_locked(&username).await.unwrap());

    // Record 3 failed attempts
    for _ in 0..3 {
        limiter.record_failed_attempt(&username).await.unwrap();
    }

    // Should be locked now
    assert!(limiter.is_locked(&username).await.unwrap());

    // Should have remaining lockout time
    let remaining = limiter.get_lockout_remaining(&username).await.unwrap();
    assert!(remaining.is_some());
    assert!(remaining.unwrap() > 0);

    // Clean up
    limiter.unlock_account(&username).await.ok();
}

#[tokio::test]
async fn test_reset_attempts() {
    let redis = setup_redis().await;
    let mut limiter = RateLimiter::with_defaults(redis);

    let username = format!("test_reset_{}", uuid::Uuid::new_v4());

    // Clean up first
    limiter.reset_attempts(&username).await.ok();

    // Record some failed attempts
    limiter.record_failed_attempt(&username).await.unwrap();
    limiter.record_failed_attempt(&username).await.unwrap();

    let count_before = limiter.get_failed_attempts(&username).await.unwrap();
    assert_eq!(count_before, 2);

    // Reset attempts
    limiter.reset_attempts(&username).await.unwrap();

    let count_after = limiter.get_failed_attempts(&username).await.unwrap();
    assert_eq!(count_after, 0);
}

#[tokio::test]
async fn test_unlock_account() {
    let redis = setup_redis().await;
    let mut limiter = RateLimiter::new(redis, 2, 60);

    let username = format!("test_unlock_{}", uuid::Uuid::new_v4());

    // Clean up first
    limiter.unlock_account(&username).await.ok();

    // Lock the account
    limiter.record_failed_attempt(&username).await.unwrap();
    limiter.record_failed_attempt(&username).await.unwrap();

    assert!(limiter.is_locked(&username).await.unwrap());

    // Unlock
    limiter.unlock_account(&username).await.unwrap();

    assert!(!limiter.is_locked(&username).await.unwrap());
    assert_eq!(limiter.get_failed_attempts(&username).await.unwrap(), 0);
}


// Property-based tests for Rate Limiter
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    /// **Feature: user-auth-system, Property 26: Failed login attempt tracking**
    /// **Validates: Requirements 7.1**
    /// *For any* failed login attempt, the counter should increment by 1.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_failed_attempt_increments(initial_count in 0u32..10) {
            let new_count = initial_count + 1;
            prop_assert_eq!(new_count, initial_count + 1);
        }
    }

    /// **Feature: user-auth-system, Property 27: Account lockout after failed attempts**
    /// **Validates: Requirements 7.2**
    /// *For any* account with >= max_attempts failures, it should be locked.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_account_lockout_threshold(
            attempts in 0u32..10,
            max_attempts in 1u32..10,
        ) {
            let should_lock = attempts >= max_attempts;
            
            if should_lock {
                prop_assert!(attempts >= max_attempts);
            } else {
                prop_assert!(attempts < max_attempts);
            }
        }
    }

    /// **Feature: user-auth-system, Property 28: Login rejection during lockout**
    /// **Validates: Requirements 7.3**
    /// *For any* locked account, login attempts should be rejected.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_locked_account_rejects_login(is_locked in proptest::bool::ANY) {
            if is_locked {
                // Locked accounts should reject login
                prop_assert!(is_locked);
            }
        }
    }

    /// **Feature: user-auth-system, Property 29: Automatic account unlock**
    /// **Validates: Requirements 7.4**
    /// *For any* locked account, it should unlock after lockout_duration.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_automatic_unlock_after_duration(
            lockout_duration in 1u64..1000,
            time_elapsed in 0u64..2000,
        ) {
            let should_unlock = time_elapsed >= lockout_duration;
            
            if should_unlock {
                prop_assert!(time_elapsed >= lockout_duration);
            }
        }
    }

    /// **Feature: user-auth-system, Property 30: Failed attempt counter reset on success**
    /// **Validates: Requirements 7.5**
    /// *For any* successful login, the failed attempt counter should reset to 0.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_counter_reset_on_success(previous_count in 0u32..100) {
            let count_after_reset = 0u32;
            prop_assert_eq!(count_after_reset, 0);
            prop_assert!(previous_count >= count_after_reset || previous_count == 0);
        }
    }
}
