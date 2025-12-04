mod common;

use ai_im_server::models::user::CreateUser;
use ai_im_server::services::auth::{AuthService, LoginData, RegisterData, UserService};
use ai_im_server::services::email::MockEmailService;
use std::sync::Arc;
use uuid::Uuid;

/// **Security Test: Brute force protection**
/// Validates that the rate limiter prevents brute force attacks
#[tokio::test]
async fn test_brute_force_protection() {
    let pool = common::setup_test_db().await;
    let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
    let redis = redis_client.get_connection_manager().await.unwrap();
    let email_service: Arc<dyn ai_im_server::services::email::EmailService> =
        Arc::new(MockEmailService::silent());

    let mut auth_service = AuthService::new(
        pool.clone(),
        redis.clone(),
        "test_secret_key_for_security_tests".to_string(),
        email_service,
    );

    // Create a test user
    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let username = format!("bruteforce_{}", &uuid_str[..10]);
    let email = format!("{}@example.com", uuid_str);
    let password = "CorrectPass123!";

    let register_data = RegisterData {
        username: username.clone(),
        email: email.clone(),
        password: password.to_string(),
    };
    auth_service.register(register_data).await.unwrap();

    // Attempt 5 failed logins
    for i in 0..5 {
        let login_data = LoginData {
            username: username.clone(),
            password: "WrongPassword123!".to_string(),
        };
        let result = auth_service.login(login_data).await;
        assert!(result.is_err(), "Login attempt {} should fail", i + 1);
    }

    // 6th attempt should be blocked due to rate limiting
    let login_data = LoginData {
        username: username.clone(),
        password: password.to_string(),
    };
    let result = auth_service.login(login_data).await;
    assert!(
        result.is_err(),
        "6th login attempt should be blocked by rate limiter"
    );

    // Cleanup
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(&username)
        .execute(&pool)
        .await
        .ok();
}

/// **Security Test: Token security - expired tokens rejected**
/// Validates that expired tokens are properly rejected
#[tokio::test]
async fn test_expired_token_rejection() {
    use ai_im_server::services::auth::{TokenService, TokenError};
    use chrono::{Duration, Utc};

    let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
    let redis = redis_client.get_connection_manager().await.unwrap();

    let token_service = TokenService::with_redis("test_secret_key".to_string(), redis);

    let user_id = Uuid::new_v4();
    let username = "test_user";

    // Create a token that's already expired (100 seconds ago to avoid timing issues)
    let now = Utc::now();
    let expired_at = now - Duration::seconds(100);
    let issued_at = now - Duration::seconds(1000);
    
    let claims = ai_im_server::services::auth::Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        token_type: ai_im_server::services::auth::TokenType::Access,
        exp: expired_at.timestamp(),
        iat: issued_at.timestamp(),
        jti: Uuid::new_v4().to_string(),
    };

    // Create the expired token
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret("test_secret_key".as_bytes()),
    )
    .unwrap();

    let result = token_service.verify_token(&token);
    
    // Check that the result is an error (Expired or Invalid)
    match result {
        Err(TokenError::Expired) => {
            // Expected behavior
        }
        Err(_) => {
            // Any error is acceptable for an expired token
        }
        Ok(_) => {
            panic!("Expired token should be rejected");
        }
    }
}

/// **Security Test: Token security - invalid signature rejected**
/// Validates that tokens with invalid signatures are rejected
#[tokio::test]
async fn test_invalid_signature_rejection() {
    use ai_im_server::services::auth::TokenService;

    let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
    let redis = redis_client.get_connection_manager().await.unwrap();

    let token_service = TokenService::with_redis("correct_secret_key".to_string(), redis);

    let user_id = Uuid::new_v4();
    let username = "test_user";

    // Create a token with a different secret key
    let claims = ai_im_server::services::auth::Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        token_type: ai_im_server::services::auth::TokenType::Access,
        exp: (chrono::Utc::now() + chrono::Duration::minutes(15)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
        jti: Uuid::new_v4().to_string(),
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret("wrong_secret_key".as_bytes()),
    )
    .unwrap();

    let result = token_service.verify_token(&token);
    assert!(
        result.is_err(),
        "Token with invalid signature should be rejected"
    );
}

/// **Security Test: SQL injection prevention**
/// Validates that SQL injection attempts are safely handled
#[tokio::test]
async fn test_sql_injection_prevention() {
    let tx = common::setup_test_transaction().await;
    let mut service = UserService::with_transaction(tx);

    // Attempt SQL injection in username
    let sql_injection_username = "admin' OR '1'='1";
    let create_data = CreateUser {
        username: sql_injection_username.to_string(),
        email: format!("test_{}@example.com", Uuid::new_v4()),
        password_hash: "hashed_password".to_string(),
    };

    // Should fail validation, not execute SQL injection
    let result = service.create_user(create_data).await;
    assert!(
        result.is_err(),
        "SQL injection attempt should be rejected by validation"
    );

    // Attempt SQL injection in search
    let sql_injection_search = "'; DROP TABLE users; --";
    let result = service.search_users(sql_injection_search).await;
    // Should return empty results or error, not execute SQL injection
    assert!(
        result.is_ok(),
        "SQL injection in search should be safely handled"
    );
    if let Ok(results) = result {
        // Should not find any users with this malicious query
        assert!(results.is_empty(), "SQL injection should not return results");
    }
}

/// **Security Test: XSS prevention in user data**
/// Validates that XSS attempts in user data are safely stored
#[tokio::test]
async fn test_xss_prevention() {
    let tx = common::setup_test_transaction().await;
    let mut service = UserService::with_transaction(tx);

    // Create user with XSS attempt in bio
    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let username = format!("xss_test_{}", &uuid_str[..8]);
    let xss_bio = "<script>alert('XSS')</script>";

    let create_data = CreateUser {
        username: username.clone(),
        email: format!("test_{}@example.com", Uuid::new_v4()),
        password_hash: "hashed_password".to_string(),
    };

    let user = service.create_user(create_data).await.unwrap();

    // Update profile with XSS attempt
    let update_data = ai_im_server::models::user::UpdateUser {
        username: None,
        avatar_url: Some("https://example.com/avatar.jpg".to_string()),
        bio: Some(xss_bio.to_string()),
    };

    let updated = service.update_profile(&user.id, update_data).await.unwrap();

    // Bio should be stored as-is (XSS prevention happens on frontend/output)
    // The database should store the raw data
    assert_eq!(updated.bio, Some(xss_bio.to_string()));

    // Note: XSS prevention is typically handled by:
    // 1. Content-Security-Policy headers
    // 2. Frontend sanitization when rendering
    // 3. Proper HTML escaping in templates
    // The backend stores data as-is for data integrity
}

/// **Security Test: GDPR compliance - data anonymization**
/// Validates that user data is properly anonymized on deletion
#[tokio::test]
async fn test_gdpr_data_anonymization() {
    let pool = common::setup_test_db().await;
    let service = UserService::new(pool.clone());

    // Create a user with personal data
    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let username = format!("gdpr_test_{}", &uuid_str[..8]);
    let email = format!("personal_{}@example.com", uuid_str);

    let create_data = CreateUser {
        username: username.clone(),
        email: email.clone(),
        password_hash: "hashed_password".to_string(),
    };

    let user = service.create_user(create_data).await.unwrap();
    let user_id = user.id;

    // Update with personal information
    let update_data = ai_im_server::models::user::UpdateUser {
        username: None,
        avatar_url: Some("https://example.com/my_photo.jpg".to_string()),
        bio: Some("My personal information and story".to_string()),
    };

    service.update_profile(&user_id, update_data).await.unwrap();

    // Delete the account (GDPR right to be forgotten)
    service.delete_account(&user_id).await.unwrap();

    // Verify data is anonymized
    let deleted_user = sqlx::query_as::<_, ai_im_server::models::user::User>(
        "SELECT * FROM users WHERE id = $1",
    )
    .bind(&user_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    // Username and email should be anonymized
    assert!(
        deleted_user.username.starts_with("deleted_"),
        "Username should be anonymized"
    );
    assert!(
        deleted_user.email.ends_with("@deleted.local"),
        "Email should be anonymized"
    );

    // Personal data should be removed
    assert_eq!(deleted_user.avatar_url, None, "Avatar should be removed");
    assert_eq!(deleted_user.bio, None, "Bio should be removed");

    // Account status should be deleted
    assert_eq!(
        deleted_user.account_status,
        ai_im_server::models::user::AccountStatus::Deleted,
        "Account status should be deleted"
    );

    // Cleanup
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(&user_id)
        .execute(&pool)
        .await
        .ok();
}

/// **Security Test: Password strength requirements**
/// Validates that weak passwords are rejected
#[tokio::test]
async fn test_password_strength_requirements() {
    use ai_im_server::services::auth::PasswordService;

    let password_service = PasswordService::new();

    // Test various weak passwords
    let weak_passwords = vec![
        "short",           // Too short
        "nouppercase123!", // No uppercase
        "NOLOWERCASE123!", // No lowercase
        "NoDigits!",       // No digits
        "NoSpecial123",    // No special characters
        "12345678",        // Only digits
        "abcdefgh",        // Only lowercase
    ];

    for password in weak_passwords {
        let result = password_service.validate_strength(password);
        assert!(
            result.is_err(),
            "Weak password '{}' should be rejected",
            password
        );
    }

    // Test strong password
    let strong_password = "StrongPass123!";
    let result = password_service.validate_strength(strong_password);
    assert!(
        result.is_ok(),
        "Strong password should be accepted: {:?}",
        result
    );
}

/// **Security Test: Token blacklist prevents reuse**
/// Validates that blacklisted tokens cannot be reused
#[tokio::test]
async fn test_token_blacklist_prevents_reuse() {
    use ai_im_server::services::auth::TokenService;

    let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
    let redis = redis_client.get_connection_manager().await.unwrap();

    let token_service = TokenService::with_redis("test_secret_key".to_string(), redis);

    let user_id = Uuid::new_v4();
    let username = "test_user";

    // Generate a valid token
    let token_pair = token_service
        .generate_token_pair(&user_id, username)
        .unwrap();
    let access_token = token_pair.access_token;

    // Verify it works initially (using verify_token_with_blacklist for full check)
    let result = token_service.verify_token_with_blacklist(&access_token).await;
    assert!(result.is_ok(), "Token should be valid initially");

    // Blacklist the token
    token_service
        .blacklist_token(&access_token)
        .await
        .unwrap();

    // Try to use the blacklisted token (must use verify_token_with_blacklist to check blacklist)
    let result = token_service.verify_token_with_blacklist(&access_token).await;
    assert!(
        result.is_err(),
        "Blacklisted token should be rejected: {:?}",
        result
    );
}

/// **Security Test: Rate limiter resets on successful login**
/// Validates that failed attempt counter resets after successful login
#[tokio::test]
async fn test_rate_limiter_reset_on_success() {
    let pool = common::setup_test_db().await;
    let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
    let redis = redis_client.get_connection_manager().await.unwrap();
    let email_service: Arc<dyn ai_im_server::services::email::EmailService> =
        Arc::new(MockEmailService::silent());

    let mut auth_service = AuthService::new(
        pool.clone(),
        redis.clone(),
        "test_secret_key_for_security_tests".to_string(),
        email_service,
    );

    // Create a test user
    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let username = format!("reset_test_{}", &uuid_str[..10]);
    let email = format!("{}@example.com", uuid_str);
    let password = "CorrectPass123!";

    let register_data = RegisterData {
        username: username.clone(),
        email: email.clone(),
        password: password.to_string(),
    };
    auth_service.register(register_data).await.unwrap();

    // Attempt 3 failed logins
    for _ in 0..3 {
        let login_data = LoginData {
            username: username.clone(),
            password: "WrongPassword123!".to_string(),
        };
        let _ = auth_service.login(login_data).await;
    }

    // Successful login should reset the counter
    let login_data = LoginData {
        username: username.clone(),
        password: password.to_string(),
    };
    let result = auth_service.login(login_data).await;
    assert!(result.is_ok(), "Successful login should work");

    // Now we should be able to fail 5 more times before lockout
    for i in 0..5 {
        let login_data = LoginData {
            username: username.clone(),
            password: "WrongPassword123!".to_string(),
        };
        let result = auth_service.login(login_data).await;
        assert!(result.is_err(), "Failed login {} should fail", i + 1);
    }

    // 6th attempt should be blocked
    let login_data = LoginData {
        username: username.clone(),
        password: password.to_string(),
    };
    let result = auth_service.login(login_data).await;
    assert!(
        result.is_err(),
        "Should be locked out after 5 new failures"
    );

    // Cleanup
    sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(&username)
        .execute(&pool)
        .await
        .ok();
}
