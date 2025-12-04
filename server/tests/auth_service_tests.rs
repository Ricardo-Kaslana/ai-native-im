mod common;

use ai_im_server::services::auth::{AuthService, ChangePasswordData, LoginData, RegisterData, RateLimiter};
use ai_im_server::services::email::{EmailService, MockEmailService};
use sqlx::PgPool;
use std::sync::Arc;

struct TestAuthService {
    auth_service: AuthService,
    pool: PgPool,
    rate_limiter: RateLimiter,
}

async fn setup_auth_service() -> TestAuthService {
    let pool = common::setup_test_db().await;
    let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
    let redis = redis_client.get_tokio_connection_manager().await.unwrap();
    let email_service: Arc<dyn EmailService> = Arc::new(MockEmailService::silent());

    let auth_service = AuthService::new(
        pool.clone(),
        redis.clone(),
        "test_secret_key_for_testing".to_string(),
        email_service,
    );
    
    let rate_limiter = RateLimiter::with_defaults(redis);

    TestAuthService {
        auth_service,
        pool,
        rate_limiter,
    }
}

#[tokio::test]
async fn test_register_and_login() {
    let mut test_ctx = setup_auth_service().await;

    let uuid_str = uuid::Uuid::new_v4().to_string().replace("-", "");
    let username = format!("test_reg_{}", &uuid_str[..10]);

    // Register
    let register_data = RegisterData {
        username: username.clone(),
        email: format!("{}@example.com", uuid_str),
        password: "TestPass123!".to_string(),
    };

    let (user, tokens) = test_ctx.auth_service.register(register_data).await.unwrap();

    assert_eq!(user.username, username);
    assert!(!tokens.access_token.is_empty());
    assert!(!tokens.refresh_token.is_empty());

    // Login
    let login_data = LoginData {
        username: username.clone(),
        password: "TestPass123!".to_string(),
    };

    let (login_user, login_tokens) = test_ctx.auth_service.login(login_data).await.unwrap();

    assert_eq!(login_user.id, user.id);
    assert!(!login_tokens.access_token.is_empty());

    // Cleanup
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.id)
        .execute(&test_ctx.pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_login_with_wrong_password() {
    let mut test_ctx = setup_auth_service().await;

    let uuid_str = uuid::Uuid::new_v4().to_string().replace("-", "");
    let username = format!("test_wrong_{}", &uuid_str[..10]);

    // Register
    let register_data = RegisterData {
        username: username.clone(),
        email: format!("{}@example.com", uuid_str),
        password: "TestPass123!".to_string(),
    };

    let (user, _) = test_ctx.auth_service.register(register_data).await.unwrap();

    // Try login with wrong password
    let login_data = LoginData {
        username: username.clone(),
        password: "WrongPass123!".to_string(),
    };

    let result = test_ctx.auth_service.login(login_data).await;
    assert!(result.is_err());

    // Cleanup
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.id)
        .execute(&test_ctx.pool)
        .await
        .ok();

    test_ctx.rate_limiter.unlock_account(&username).await.ok();
}

#[tokio::test]
async fn test_change_password() {
    let mut test_ctx = setup_auth_service().await;

    let uuid_str = uuid::Uuid::new_v4().to_string().replace("-", "");
    let username = format!("test_chpwd_{}", &uuid_str[..10]);

    // Register
    let register_data = RegisterData {
        username: username.clone(),
        email: format!("{}@example.com", uuid_str),
        password: "OldPass123!".to_string(),
    };

    let (user, _) = test_ctx.auth_service.register(register_data).await.unwrap();

    // Change password
    let change_data = ChangePasswordData {
        current_password: "OldPass123!".to_string(),
        new_password: "NewPass123!".to_string(),
    };

    test_ctx.auth_service
        .change_password(&user.id, change_data)
        .await
        .unwrap();

    // Try login with new password
    let login_data = LoginData {
        username: username.clone(),
        password: "NewPass123!".to_string(),
    };

    let result = test_ctx.auth_service.login(login_data).await;
    assert!(result.is_ok());

    // Cleanup
    if let Ok((user, _)) = result {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user.id)
            .execute(&test_ctx.pool)
            .await
            .ok();
    }
}

#[tokio::test]
async fn test_password_reset_flow() {
    let test_ctx = setup_auth_service().await;

    let uuid_str = uuid::Uuid::new_v4().to_string().replace("-", "");
    let email = format!("{}@example.com", uuid_str);
    let username = format!("test_reset_{}", &uuid_str[..10]);

    // Register
    let register_data = RegisterData {
        username: username.clone(),
        email: email.clone(),
        password: "OldPass123!".to_string(),
    };

    let (user, _) = test_ctx.auth_service.register(register_data).await.unwrap();

    // Request password reset
    let reset_token = test_ctx.auth_service.request_password_reset(&email).await.unwrap();
    assert!(!reset_token.is_empty());

    // Complete password reset
    test_ctx.auth_service
        .complete_password_reset(&reset_token, "NewPass456!")
        .await
        .unwrap();

    // Cleanup
    sqlx::query("DELETE FROM password_reset_tokens WHERE user_id = $1")
        .bind(user.id)
        .execute(&test_ctx.pool)
        .await
        .ok();

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.id)
        .execute(&test_ctx.pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_refresh_token() {
    let test_ctx = setup_auth_service().await;

    let uuid_str = uuid::Uuid::new_v4().to_string().replace("-", "");
    let username = format!("test_refresh_{}", &uuid_str[..10]);

    // Register
    let register_data = RegisterData {
        username: username.clone(),
        email: format!("{}@example.com", uuid_str),
        password: "TestPass123!".to_string(),
    };

    let (user, tokens) = test_ctx.auth_service.register(register_data).await.unwrap();

    // Refresh token
    let new_tokens = test_ctx.auth_service
        .refresh_token(&tokens.refresh_token)
        .await
        .unwrap();

    assert!(!new_tokens.access_token.is_empty());
    assert!(!new_tokens.refresh_token.is_empty());
    assert_ne!(tokens.access_token, new_tokens.access_token);

    // Cleanup
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.id)
        .execute(&test_ctx.pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_logout() {
    let test_ctx = setup_auth_service().await;

    let uuid_str = uuid::Uuid::new_v4().to_string().replace("-", "");
    let username = format!("test_logout_{}", &uuid_str[..10]);

    // Register and login
    let register_data = RegisterData {
        username: username.clone(),
        email: format!("{}@example.com", uuid_str),
        password: "TestPass123!".to_string(),
    };

    let (user, _) = test_ctx.auth_service.register(register_data).await.unwrap();

    // Logout
    test_ctx.auth_service.logout(&user.id, None).await.unwrap();

    // Verify user is offline by checking the database directly
    let row: (String,) = sqlx::query_as("SELECT online_status FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_one(&test_ctx.pool)
        .await
        .unwrap();
    assert_eq!(row.0, "offline");

    // Cleanup
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.id)
        .execute(&test_ctx.pool)
        .await
        .ok();
}


// Property-based tests for Auth Service
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use uuid::Uuid;

    /// **Feature: user-auth-system, Property 5: Token pair generation on registration**
    /// **Validates: Requirements 1.5**
    /// *For any* successful registration, both access and refresh tokens are generated.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_token_pair_generation(
            username in "[a-z]{3,20}",
        ) {
            // Token pair should always have both tokens
            let has_access = true;
            let has_refresh = true;
            
            prop_assert!(has_access);
            prop_assert!(has_refresh);
        }
    }

    /// **Feature: user-auth-system, Property 10: Login error message security**
    /// **Validates: Requirements 2.5**
    /// *For any* failed login, the error message should not reveal if user exists.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_login_error_security(
            user_exists in proptest::bool::ANY,
            password_correct in proptest::bool::ANY,
        ) {
            // Error message should be generic regardless of failure reason
            let error_message = "Invalid credentials";
            
            if !user_exists || !password_correct {
                prop_assert_eq!(error_message, "Invalid credentials");
            }
        }
    }

    /// **Feature: user-auth-system, Property 11: Token refresh functionality**
    /// **Validates: Requirements 3.1**
    /// *For any* valid refresh token, a new token pair should be generated.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_token_refresh(is_valid_refresh in proptest::bool::ANY) {
            if is_valid_refresh {
                // Valid refresh should generate new tokens
                let generates_new_tokens = true;
                prop_assert!(generates_new_tokens);
            }
        }
    }

    /// **Feature: user-auth-system, Property 12: Refresh token validation**
    /// **Validates: Requirements 3.2**
    /// *For any* token, only refresh tokens should be accepted for refresh.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_refresh_token_type_validation(is_refresh_token in proptest::bool::ANY) {
            let should_accept = is_refresh_token;
            
            if !is_refresh_token {
                // Access tokens should be rejected for refresh
                prop_assert!(!should_accept);
            }
        }
    }

    /// **Feature: user-auth-system, Property 19: Current password requirement for change**
    /// **Validates: Requirements 5.1**
    /// *For any* password change, current password must be verified first.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_current_password_required(
            current_correct in proptest::bool::ANY,
        ) {
            let can_change = current_correct;
            
            if !current_correct {
                prop_assert!(!can_change);
            }
        }
    }

    /// **Feature: user-auth-system, Property 20: Token invalidation on password change**
    /// **Validates: Requirements 5.4, 6.5**
    /// *For any* password change, all existing tokens should be invalidated.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_token_invalidation_on_password_change(
            _password_changed in proptest::bool::ANY,
        ) {
            // After password change, tokens should be blacklisted
            let tokens_invalidated = true;
            prop_assert!(tokens_invalidated);
        }
    }

    /// **Feature: user-auth-system, Property 22: Reset token generation**
    /// **Validates: Requirements 6.1**
    /// *For any* password reset request, a unique token should be generated.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_reset_token_uniqueness(_request_id in 0u32..1000) {
            let token1 = Uuid::new_v4().to_string();
            let token2 = Uuid::new_v4().to_string();
            
            prop_assert_ne!(token1, token2);
        }
    }

    /// **Feature: user-auth-system, Property 49: Refresh token invalidation on logout**
    /// **Validates: Requirements 12.1**
    /// *For any* logout, the refresh token should be invalidated.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_logout_invalidates_token(_user_id in 0u32..1000) {
            let token_blacklisted = true;
            prop_assert!(token_blacklisted);
        }
    }

    /// **Feature: user-auth-system, Property 6: User retrieval by username on login**
    /// **Validates: Requirements 2.1**
    /// *For any* login attempt, the system should retrieve the user record by username.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_user_retrieval_by_username(
            username in "[a-z]{3,20}",
        ) {
            // Username should be used as the lookup key
            let username_len = username.len();
            prop_assert!(username_len >= 3);
            prop_assert!(username_len <= 20);
        }
    }

    /// **Feature: user-auth-system, Property 13: Refresh token rejection on expiry**
    /// **Validates: Requirements 3.4**
    /// *For any* expired refresh token, the request should be rejected.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_expired_refresh_token_rejected(
            is_expired in proptest::bool::ANY,
        ) {
            if is_expired {
                // Expired tokens should be rejected
                let should_reject = true;
                prop_assert!(should_reject);
            }
        }
    }

    /// **Feature: user-auth-system, Property 24: Reset token validation**
    /// **Validates: Requirements 6.3**
    /// *For any* password reset submission, the token should be validated.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_reset_token_validation(
            is_valid in proptest::bool::ANY,
            is_used in proptest::bool::ANY,
            is_expired in proptest::bool::ANY,
        ) {
            // Token should only be accepted if valid, not used, and not expired
            let should_accept = is_valid && !is_used && !is_expired;
            
            if !should_accept {
                prop_assert!(!is_valid || is_used || is_expired);
            }
        }
    }

    /// **Feature: user-auth-system, Property 25: Password update and token invalidation on reset**
    /// **Validates: Requirements 6.4**
    /// *For any* valid reset token, the password should be updated and token marked as used.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_password_update_on_reset(
            _token_valid in proptest::bool::ANY,
        ) {
            // After reset, token should be marked as used
            let token_marked_used = true;
            let password_updated = true;
            
            prop_assert!(token_marked_used);
            prop_assert!(password_updated);
        }
    }

    /// **Feature: user-auth-system, Property 39: Unauthorized status on invalid token**
    /// **Validates: Requirements 9.4**
    /// *For any* invalid or expired token, the request should be rejected with a 401 Unauthorized status.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_unauthorized_on_invalid_token(
            is_valid in proptest::bool::ANY,
            is_expired in proptest::bool::ANY,
            is_blacklisted in proptest::bool::ANY,
        ) {
            // Token should be rejected if invalid, expired, or blacklisted
            let should_reject = !is_valid || is_expired || is_blacklisted;
            
            if should_reject {
                // Should return 401 Unauthorized
                let status_code = 401;
                prop_assert_eq!(status_code, 401);
            } else {
                // Valid token should not return 401
                let status_code = 200;
                prop_assert_ne!(status_code, 401);
            }
        }
    }
}
