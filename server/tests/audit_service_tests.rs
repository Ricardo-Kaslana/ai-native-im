mod common;

use ai_im_server::models::user::CreateUser;
use ai_im_server::services::auth::{AuditService, UserService};
use proptest::prelude::*;
use uuid::Uuid;

// ============================================================================
// Property-Based Tests for Security Audit Logging (Task 12)
// ============================================================================

// **Feature: user-auth-system, Property 59: Successful login audit logging**
// *For any* successful login, an audit log entry should be created with user ID, timestamp, and IP address
// **Validates: Requirements 14.1**
mod property_59_successful_login_logging {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn successful_login_creates_audit_log(
            username_suffix in "[a-z0-9]{8}",
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let pool = common::setup_test_db().await;
                let audit_service = AuditService::new(pool.clone());
                let user_service = UserService::new(pool.clone());
                
                // Generate unique identifier
                let unique_id = Uuid::new_v4().to_string().replace("-", "");
                
                // Create user
                let user = user_service.create_user(CreateUser {
                    username: format!("user_{}_{}", username_suffix, &unique_id[..8]),
                    email: format!("user_{}_{}@test.com", username_suffix, &unique_id[..8]),
                    password_hash: "hash".to_string(),
                }).await.unwrap();
                
                // Log successful login
                audit_service.log_login_success(
                    user.id,
                    &user.username,
                    None,
                    None,
                ).await.unwrap();
                
                // Verify audit log entry exists
                let log_entry: (uuid::Uuid, uuid::Uuid, String, String) = sqlx::query_as(
                    "SELECT id, user_id, username, event_type FROM security_audit_log WHERE user_id = $1 AND event_type = 'login_success' ORDER BY created_at DESC LIMIT 1"
                )
                .bind(&user.id)
                .fetch_one(&pool)
                .await
                .unwrap();
                
                prop_assert_eq!(log_entry.1, user.id, "user_id should match");
                prop_assert_eq!(log_entry.2, user.username, "username should match");
                prop_assert_eq!(log_entry.3, "login_success", "event_type should be login_success");
                
                // Cleanup
                sqlx::query("DELETE FROM security_audit_log WHERE user_id = $1")
                    .bind(&user.id)
                    .execute(&pool)
                    .await
                    .ok();
                sqlx::query("DELETE FROM users WHERE id = $1")
                    .bind(&user.id)
                    .execute(&pool)
                    .await
                    .ok();
                
                Ok(())
            })?;
        }
    }
}

// **Feature: user-auth-system, Property 60: Failed login audit logging**
// *For any* failed login attempt, an audit log entry should be created with username, timestamp, IP address, and failure reason
// **Validates: Requirements 14.2**
mod property_60_failed_login_logging {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn failed_login_creates_audit_log(
            username_suffix in "[a-z0-9]{8}",
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let pool = common::setup_test_db().await;
                let audit_service = AuditService::new(pool.clone());
                
                // Generate unique identifier
                let unique_id = Uuid::new_v4().to_string().replace("-", "");
                let username = format!("user_{}_{}", username_suffix, &unique_id[..8]);
                
                // Log failed login
                audit_service.log_login_failed(
                    &username,
                    "invalid_credentials",
                    None,
                    None,
                ).await.unwrap();
                
                // Verify audit log entry exists
                let log_entry: (uuid::Uuid, String, String) = sqlx::query_as(
                    "SELECT id, username, event_type FROM security_audit_log WHERE username = $1 AND event_type = 'login_failed' ORDER BY created_at DESC LIMIT 1"
                )
                .bind(&username)
                .fetch_one(&pool)
                .await
                .unwrap();
                
                prop_assert_eq!(log_entry.1, username.clone(), "username should match");
                prop_assert_eq!(log_entry.2, "login_failed", "event_type should be login_failed");
                
                // Cleanup
                sqlx::query("DELETE FROM security_audit_log WHERE username = $1")
                    .bind(&username)
                    .execute(&pool)
                    .await
                    .ok();
                
                Ok(())
            })?;
        }
    }
}

// **Feature: user-auth-system, Property 61: Password change audit logging**
// *For any* password change, an audit log entry should be created with user ID and timestamp
// **Validates: Requirements 14.3**
mod property_61_password_change_logging {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn password_change_creates_audit_log(
            username_suffix in "[a-z0-9]{8}",
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let pool = common::setup_test_db().await;
                let audit_service = AuditService::new(pool.clone());
                let user_service = UserService::new(pool.clone());
                
                // Generate unique identifier
                let unique_id = Uuid::new_v4().to_string().replace("-", "");
                
                // Create user
                let user = user_service.create_user(CreateUser {
                    username: format!("user_{}_{}", username_suffix, &unique_id[..8]),
                    email: format!("user_{}_{}@test.com", username_suffix, &unique_id[..8]),
                    password_hash: "hash".to_string(),
                }).await.unwrap();
                
                // Log password change
                audit_service.log_password_changed(
                    user.id,
                    &user.username,
                    None,
                    None,
                ).await.unwrap();
                
                // Verify audit log entry exists
                let log_entry: (uuid::Uuid, uuid::Uuid, String, String) = sqlx::query_as(
                    "SELECT id, user_id, username, event_type FROM security_audit_log WHERE user_id = $1 AND event_type = 'password_changed' ORDER BY created_at DESC LIMIT 1"
                )
                .bind(&user.id)
                .fetch_one(&pool)
                .await
                .unwrap();
                
                prop_assert_eq!(log_entry.1, user.id, "user_id should match");
                prop_assert_eq!(log_entry.2, user.username, "username should match");
                prop_assert_eq!(log_entry.3, "password_changed", "event_type should be password_changed");
                
                // Cleanup
                sqlx::query("DELETE FROM security_audit_log WHERE user_id = $1")
                    .bind(&user.id)
                    .execute(&pool)
                    .await
                    .ok();
                sqlx::query("DELETE FROM users WHERE id = $1")
                    .bind(&user.id)
                    .execute(&pool)
                    .await
                    .ok();
                
                Ok(())
            })?;
        }
    }
}

// **Feature: user-auth-system, Property 62: Account lockout audit logging**
// *For any* account lockout due to failed attempts, an audit log entry should be created with user ID and lockout duration
// **Validates: Requirements 14.4**
mod property_62_lockout_logging {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn account_lockout_creates_audit_log(
            username_suffix in "[a-z0-9]{8}",
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let pool = common::setup_test_db().await;
                let audit_service = AuditService::new(pool.clone());
                
                // Generate unique identifier
                let unique_id = Uuid::new_v4().to_string().replace("-", "");
                let username = format!("user_{}_{}", username_suffix, &unique_id[..8]);
                
                // Log account lockout
                audit_service.log_account_locked(
                    &username,
                    None,
                    None,
                ).await.unwrap();
                
                // Verify audit log entry exists
                let log_entry: (uuid::Uuid, String, String) = sqlx::query_as(
                    "SELECT id, username, event_type FROM security_audit_log WHERE username = $1 AND event_type = 'account_locked' ORDER BY created_at DESC LIMIT 1"
                )
                .bind(&username)
                .fetch_one(&pool)
                .await
                .unwrap();
                
                prop_assert_eq!(log_entry.1, username.clone(), "username should match");
                prop_assert_eq!(log_entry.2, "account_locked", "event_type should be account_locked");
                
                // Cleanup
                sqlx::query("DELETE FROM security_audit_log WHERE username = $1")
                    .bind(&username)
                    .execute(&pool)
                    .await
                    .ok();
                
                Ok(())
            })?;
        }
    }
}

// **Feature: user-auth-system, Property 63: Audit log storage**
// *For any* security event, the audit log should be stored in a separate audit log table
// **Validates: Requirements 14.5**
mod property_63_audit_log_storage {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn audit_logs_stored_in_separate_table(
            username_suffix in "[a-z0-9]{8}",
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let pool = common::setup_test_db().await;
                let audit_service = AuditService::new(pool.clone());
                let user_service = UserService::new(pool.clone());
                
                // Generate unique identifier
                let unique_id = Uuid::new_v4().to_string().replace("-", "");
                
                // Create user
                let user = user_service.create_user(CreateUser {
                    username: format!("user_{}_{}", username_suffix, &unique_id[..8]),
                    email: format!("user_{}_{}@test.com", username_suffix, &unique_id[..8]),
                    password_hash: "hash".to_string(),
                }).await.unwrap();
                
                // Log multiple events
                audit_service.log_login_success(user.id, &user.username, None, None).await.unwrap();
                audit_service.log_password_changed(user.id, &user.username, None, None).await.unwrap();
                audit_service.log_logout(user.id, &user.username, None, None).await.unwrap();
                
                // Verify all logs are in security_audit_log table
                let log_count: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM security_audit_log WHERE user_id = $1"
                )
                .bind(&user.id)
                .fetch_one(&pool)
                .await
                .unwrap();
                
                prop_assert!(log_count.0 >= 3, "At least 3 audit log entries should exist");
                
                // Verify table structure has required columns
                let table_info: Vec<(String,)> = sqlx::query_as(
                    "SELECT column_name FROM information_schema.columns WHERE table_name = 'security_audit_log'"
                )
                .fetch_all(&pool)
                .await
                .unwrap();
                
                let column_names: Vec<String> = table_info.into_iter().map(|(name,)| name).collect();
                prop_assert!(column_names.contains(&"user_id".to_string()), "Table should have user_id column");
                prop_assert!(column_names.contains(&"username".to_string()), "Table should have username column");
                prop_assert!(column_names.contains(&"event_type".to_string()), "Table should have event_type column");
                prop_assert!(column_names.contains(&"created_at".to_string()), "Table should have created_at column");
                
                // Cleanup
                sqlx::query("DELETE FROM security_audit_log WHERE user_id = $1")
                    .bind(&user.id)
                    .execute(&pool)
                    .await
                    .ok();
                sqlx::query("DELETE FROM users WHERE id = $1")
                    .bind(&user.id)
                    .execute(&pool)
                    .await
                    .ok();
                
                Ok(())
            })?;
        }
    }
}
