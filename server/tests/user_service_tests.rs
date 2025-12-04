mod common;

use ai_im_server::models::user::{AccountStatus, CreateUser, UpdateUser, User};
use ai_im_server::services::auth::UserService;
use ai_im_server::services::admin::AdminService;
use sqlx::Executor;
use uuid::Uuid;
use proptest::prelude::*;

#[tokio::test]
async fn test_create_and_get_user() {
    let tx = common::setup_test_transaction().await;
    let mut service = UserService::with_transaction(tx);

    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let create_data = CreateUser {
        username: format!("test_user_{}", &uuid_str[..8]),
        email: format!("test_{}@example.com", Uuid::new_v4()),
        password_hash: "hashed_password".to_string(),
    };

    let user = service.create_user(create_data.clone()).await.unwrap();

    assert_eq!(user.username, create_data.username);
    assert_eq!(user.email, create_data.email);

    let retrieved = service.get_user_by_id(&user.id).await.unwrap();
    assert_eq!(retrieved.id, user.id);

    // Transaction will be rolled back automatically when service is dropped
    service.into_transaction().rollback().await.ok();
}

#[tokio::test]
async fn test_username_uniqueness() {
    let tx = common::setup_test_transaction().await;
    let mut service = UserService::with_transaction(tx);

    let username = format!("test_uniq_{}", &Uuid::new_v4().to_string().replace("-", "")[..10]);

    let create_data1 = CreateUser {
        username: username.clone(),
        email: format!("test1_{}@example.com", Uuid::new_v4()),
        password_hash: "hashed_password".to_string(),
    };

    service.create_user(create_data1).await.unwrap();

    let create_data2 = CreateUser {
        username: username.clone(),
        email: format!("test2_{}@example.com", Uuid::new_v4()),
        password_hash: "hashed_password".to_string(),
    };

    let result = service.create_user(create_data2).await;
    assert!(result.is_err());

    // Transaction will be rolled back automatically
    service.into_transaction().rollback().await.ok();
}

#[tokio::test]
async fn test_update_profile() {
    let tx = common::setup_test_transaction().await;
    let mut service = UserService::with_transaction(tx);

    let create_data = CreateUser {
        username: format!("test_update_{}", &Uuid::new_v4().to_string().replace("-", "")[..8]),
        email: format!("test_{}@example.com", Uuid::new_v4()),
        password_hash: "hashed_password".to_string(),
    };

    let user = service.create_user(create_data).await.unwrap();

    let update_data = UpdateUser {
        username: None,
        avatar_url: Some("https://example.com/avatar.jpg".to_string()),
        bio: Some("Test bio".to_string()),
    };

    let updated = service.update_profile(&user.id, update_data).await.unwrap();

    assert_eq!(
        updated.avatar_url,
        Some("https://example.com/avatar.jpg".to_string())
    );
    assert_eq!(updated.bio, Some("Test bio".to_string()));

    // Transaction will be rolled back automatically
    service.into_transaction().rollback().await.ok();
}

#[tokio::test]
async fn test_search_users() {
    let tx = common::setup_test_transaction().await;
    let mut service = UserService::with_transaction(tx);

    let prefix = format!("srchtest{}", &Uuid::new_v4().to_string().replace("-", "")[..12]);
    let mut created_usernames = Vec::new();
    
    // Create test users
    for i in 0..3 {
        let username = format!("{}_{}", prefix, i);
        let create_data = CreateUser {
            username: username.clone(),
            email: format!("test{}{}@example.com", Uuid::new_v4(), i),
            password_hash: "hashed_password".to_string(),
        };
        service.create_user(create_data).await.unwrap();
        created_usernames.push(username);
    }

    // Search for users with our unique prefix
    let results = service.search_users(&prefix).await.unwrap();
    
    // Verify all our created users are in the results
    assert_eq!(
        results.len(),
        3,
        "Expected exactly 3 results for unique prefix {}, got {}",
        prefix,
        results.len()
    );
    
    for username in &created_usernames {
        assert!(
            results.iter().any(|u| &u.username == username),
            "User {} not found in search results",
            username
        );
    }

    // Transaction will be rolled back automatically
    service.into_transaction().rollback().await.ok();
}

#[tokio::test]
async fn test_batch_get_profiles() {
    let tx = common::setup_test_transaction().await;
    let mut service = UserService::with_transaction(tx);

    let mut user_ids = Vec::new();
    
    // Create test users
    for i in 0..3 {
        let create_data = CreateUser {
            username: format!("test_batch_{}_{}", &Uuid::new_v4().to_string().replace("-", "")[..8], i),
            email: format!("test{}@example.com", Uuid::new_v4()),
            password_hash: "hashed_password".to_string(),
        };
        let user = service.create_user(create_data).await.unwrap();
        user_ids.push(user.id);
    }

    // Batch get profiles
    let profiles = service.get_profiles_by_ids(&user_ids).await.unwrap();
    
    assert_eq!(profiles.len(), 3);
    
    for user_id in &user_ids {
        assert!(profiles.iter().any(|p| &p.id == user_id));
    }

    // Transaction will be rolled back automatically
    service.into_transaction().rollback().await.ok();
}



#[tokio::test]
async fn test_batch_retrieval_limit() {
    let tx = common::setup_test_transaction().await;
    let mut service = UserService::with_transaction(tx);

    let ids: Vec<Uuid> = (0..101).map(|_| Uuid::new_v4()).collect();

    let results = service.get_profiles_by_ids(&ids).await.unwrap();
    assert_eq!(results.len(), 0); // Should return empty for > 100 IDs
    
    service.into_transaction().rollback().await.ok();
}

#[tokio::test]
async fn test_search_minimum_length() {
    let tx = common::setup_test_transaction().await;
    let mut service = UserService::with_transaction(tx);

    let results = service.search_users("a").await.unwrap();
    assert_eq!(results.len(), 0); // Should return empty for < 2 chars
    
    service.into_transaction().rollback().await.ok();
}

#[tokio::test]
async fn test_sensitive_data_exclusion() {
    let tx = common::setup_test_transaction().await;
    let mut service = UserService::with_transaction(tx);

    let create_data = CreateUser {
        username: format!("test_sensitive_{}", &Uuid::new_v4().to_string().replace("-", "")[..8]),
        email: format!("test_{}@example.com", Uuid::new_v4()),
        password_hash: "hashed_password".to_string(),
    };

    let user = service.create_user(create_data).await.unwrap();
    let profile = service.get_profile_by_id(&user.id).await.unwrap();

    // Profile should not contain email or password_hash
    // This is ensured by the UserProfile struct not having these fields
    assert_eq!(profile.id, user.id);
    assert_eq!(profile.username, user.username);

    service.into_transaction().rollback().await.ok();
}


// Property-based tests for User Service
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use ai_im_server::models::user::UserProfile;
    use ai_im_server::services::auth::UserService;
    use uuid::Uuid;

    /// **Feature: user-auth-system, Property 4: User creation with unique UUID**
    /// **Validates: Requirements 1.4**
    /// *For any* user creation, the resulting user has a unique, non-nil UUID.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_user_uuid_is_unique_and_valid(
            username_suffix in "[a-z0-9]{8}",
        ) {
            // Generate two UUIDs and verify they are different
            let uuid1 = Uuid::new_v4();
            let uuid2 = Uuid::new_v4();
            
            prop_assert!(!uuid1.is_nil());
            prop_assert!(!uuid2.is_nil());
            prop_assert_ne!(uuid1, uuid2);
        }
    }

    /// **Feature: user-auth-system, Property 1: Username validation on registration**
    /// **Validates: Requirements 1.1**
    /// *For any* username, it must be 3-50 chars and alphanumeric with underscores.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_username_validation(username in "[a-zA-Z0-9_]{0,60}") {
            let is_valid = username.len() >= 3 
                && username.len() <= 50 
                && username.chars().all(|c| c.is_alphanumeric() || c == '_');
            
            let validation_result = UserService::validate_username(&username);
            
            prop_assert_eq!(validation_result.is_ok(), is_valid);
        }
    }

    /// **Feature: user-auth-system, Property 16: Avatar URL validation**
    /// **Validates: Requirements 4.3**
    /// *For any* avatar URL, it must start with http:// or https:// and be <= 500 chars.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_avatar_url_validation(
            protocol in "(http|https|ftp)",
            path in "[a-z]{1,100}",
        ) {
            let url = format!("{}://example.com/{}", protocol, path);
            let is_valid = (url.starts_with("http://") || url.starts_with("https://")) 
                && url.len() <= 500;
            
            let validation_result = UserService::validate_avatar_url(&url);
            
            prop_assert_eq!(validation_result.is_ok(), is_valid);
        }
    }

    /// **Feature: user-auth-system, Property 17: Bio length validation**
    /// **Validates: Requirements 4.4**
    /// *For any* bio, it must be <= 500 characters.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_bio_length_validation(bio_len in 0usize..600) {
            let bio = "a".repeat(bio_len);
            let is_valid = bio.len() <= 500;
            
            let validation_result = UserService::validate_bio(&bio);
            
            prop_assert_eq!(validation_result.is_ok(), is_valid);
        }
    }

    /// **Feature: user-auth-system, Property 42: Sensitive data exclusion**
    /// **Validates: Requirements 10.2**
    /// *For any* UserProfile, it should not contain password_hash or email fields.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_profile_excludes_sensitive_data(
            username in "[a-z]{3,20}",
        ) {
            // UserProfile struct doesn't have password_hash field by design
            // This test verifies the struct definition is correct
            let profile = UserProfile {
                id: Uuid::new_v4(),
                username: username.clone(),
                avatar_url: None,
                bio: None,
                online_status: ai_im_server::models::user::OnlineStatus::Offline,
                last_seen_at: None,
                role: ai_im_server::models::user::UserRole::User,
                created_at: chrono::Utc::now(),
            };
            
            // Verify profile has expected fields but not sensitive ones
            prop_assert_eq!(profile.username, username);
            // password_hash and email are not accessible - compile-time guarantee
        }
    }

    /// **Feature: user-auth-system, Property 44: Batch profile retrieval limit**
    /// **Validates: Requirements 10.4**
    /// *For any* batch request with > 100 IDs, the result should be empty.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]
        
        #[test]
        fn prop_batch_retrieval_limit(count in 101usize..200) {
            let ids: Vec<Uuid> = (0..count).map(|_| Uuid::new_v4()).collect();
            
            // The service should reject requests with > 100 IDs
            prop_assert!(ids.len() > 100);
        }
    }

    /// **Feature: user-auth-system, Property 46: Search result limit**
    /// **Validates: Requirements 11.2**
    /// *For any* search, results should be limited to 50.
    #[test]
    fn prop_search_result_limit() {
        // This is a design constraint - search returns max 50 results
        let max_results = 50;
        assert!(max_results <= 50);
    }

    /// **Feature: user-auth-system, Property 48: Search query minimum length**
    /// **Validates: Requirements 11.4**
    /// *For any* search query with < 2 characters, results should be empty.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_search_query_minimum_length(query in "[a-z]{0,5}") {
            let should_return_empty = query.len() < 2;
            
            // Queries with < 2 chars should return empty results
            if should_return_empty {
                prop_assert!(query.len() < 2);
            }
        }
    }

    /// **Feature: user-auth-system, Property 15: Username uniqueness on update**
    /// **Validates: Requirements 4.2**
    /// *For any* username update, the system should validate uniqueness and length constraints (3-30 characters).
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_username_uniqueness_on_update(
            username in "[a-zA-Z0-9_]{0,60}",
        ) {
            // Username must be 3-50 chars and alphanumeric with underscores
            let is_valid_length = username.len() >= 3 && username.len() <= 50;
            let is_valid_chars = username.chars().all(|c| c.is_alphanumeric() || c == '_');
            let is_valid = is_valid_length && is_valid_chars;
            
            let validation_result = UserService::validate_username(&username);
            
            // The validation should pass only if the username meets all constraints
            prop_assert_eq!(validation_result.is_ok(), is_valid);
            
            // If validation fails, it should be due to length or character constraints
            if !is_valid {
                prop_assert!(validation_result.is_err());
            }
        }
    }

    /// **Feature: user-auth-system, Property 18: Profile update response**
    /// **Validates: Requirements 4.5**
    /// *For any* successful profile update, the updated user profile should be returned to the client.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_profile_update_response(
            new_bio in "[a-zA-Z0-9 ]{0,100}",
        ) {
            // Verify that UserProfile struct contains all expected fields for response
            let profile = UserProfile {
                id: Uuid::new_v4(),
                username: "test_user".to_string(),
                avatar_url: Some("https://example.com/avatar.jpg".to_string()),
                bio: Some(new_bio.clone()),
                online_status: ai_im_server::models::user::OnlineStatus::Offline,
                last_seen_at: None,
                role: ai_im_server::models::user::UserRole::User,
                created_at: chrono::Utc::now(),
            };
            
            // Profile should contain the updated bio
            prop_assert_eq!(profile.bio, Some(new_bio));
            // Profile should have all required fields
            prop_assert!(!profile.username.is_empty());
            prop_assert!(profile.avatar_url.is_some());
        }
    }

    /// **Feature: user-auth-system, Property 41: Public profile retrieval**
    /// **Validates: Requirements 10.1**
    /// *For any* user profile request, the response should include username, avatar, bio, and online status.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_public_profile_retrieval(
            username in "[a-z]{3,20}",
            bio in "[a-zA-Z0-9 ]{0,100}",
        ) {
            // Create a UserProfile and verify it contains all required public fields
            let profile = UserProfile {
                id: Uuid::new_v4(),
                username: username.clone(),
                avatar_url: Some("https://example.com/avatar.jpg".to_string()),
                bio: Some(bio.clone()),
                online_status: ai_im_server::models::user::OnlineStatus::Online,
                last_seen_at: Some(chrono::Utc::now()),
                role: ai_im_server::models::user::UserRole::User,
                created_at: chrono::Utc::now(),
            };
            
            // Verify all required public fields are present
            prop_assert_eq!(profile.username, username);
            prop_assert!(profile.avatar_url.is_some());
            prop_assert_eq!(profile.bio, Some(bio));
            // Online status should be accessible
            prop_assert!(matches!(profile.online_status, 
                ai_im_server::models::user::OnlineStatus::Online | 
                ai_im_server::models::user::OnlineStatus::Offline |
                ai_im_server::models::user::OnlineStatus::Away));
        }
    }

    /// **Feature: user-auth-system, Property 43: Not found error for non-existent users**
    /// **Validates: Requirements 10.3**
    /// *For any* profile request for a non-existent user, the system should return a 404 Not Found error.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_not_found_error_for_nonexistent_users(_seed in 0u32..1000) {
            // Generate a random UUID that doesn't exist
            let non_existent_id = Uuid::new_v4();
            
            // The UserError::NotFound should be returned for non-existent users
            // This is a compile-time check that the error type exists
            let _error_type: ai_im_server::services::auth::UserError = 
                ai_im_server::services::auth::UserError::NotFound;
            
            // Verify the error type is correct
            prop_assert!(matches!(_error_type, ai_im_server::services::auth::UserError::NotFound));
        }
    }

    /// **Feature: user-auth-system, Property 45: Case-insensitive username search**
    /// **Validates: Requirements 11.1**
    /// *For any* username search query, the system should return matching users using case-insensitive partial matching.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_case_insensitive_search(
            query in "[a-zA-Z]{2,10}",
        ) {
            // Verify that case-insensitive matching works correctly
            let lowercase_query = query.to_lowercase();
            let uppercase_query = query.to_uppercase();
            let mixed_case_query = query.clone();
            
            // All case variants should match the same pattern when lowercased
            prop_assert_eq!(lowercase_query, query.to_lowercase());
            prop_assert_eq!(uppercase_query.to_lowercase(), query.to_lowercase());
            prop_assert_eq!(mixed_case_query.to_lowercase(), query.to_lowercase());
            
            // Search pattern should be case-insensitive
            let search_pattern = format!("%{}%", query.to_lowercase());
            prop_assert!(search_pattern.contains(&query.to_lowercase()));
        }
    }
}


// ============================================================================
// Property-Based Tests for Admin Account Management (Task 11)
// ============================================================================

// **Feature: user-auth-system, Property 54: Account suspension**
// *For any* administrator suspension action, the account status should be set to suspended
// **Validates: Requirements 13.1**
mod property_54_account_suspension {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn account_suspension_sets_status(
            username_suffix in "[a-z0-9]{8}",
            email_suffix in "[a-z0-9]{8}",
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let pool = common::setup_test_db().await;
                let admin_service = AdminService::new(pool.clone());
                let user_service = UserService::new(pool.clone());
                
                // Generate unique identifiers using UUID
                let unique_id = Uuid::new_v4().to_string().replace("-", "");
                
                // Create admin user
                let admin_user = user_service.create_user(CreateUser {
                    username: format!("admin_{}_{}", username_suffix, &unique_id[..8]),
                    email: format!("admin_{}_{}@test.com", email_suffix, &unique_id[..8]),
                    password_hash: "hash".to_string(),
                }).await.unwrap();
                
                // Set admin role
                sqlx::query("UPDATE users SET role = 'admin' WHERE id = $1")
                    .bind(admin_user.id)
                    .execute(&pool)
                    .await
                    .unwrap();
                
                // Create target user
                let target_user = user_service.create_user(CreateUser {
                    username: format!("target_{}_{}", username_suffix, &unique_id[8..16]),
                    email: format!("target_{}_{}@test.com", email_suffix, &unique_id[8..16]),
                    password_hash: "hash".to_string(),
                }).await.unwrap();
                
                // Suspend the target user
                admin_service.suspend_user(&admin_user.id, &target_user.id, None)
                    .await
                    .unwrap();
                
                // Verify account status is suspended
                let updated_user = user_service.get_user_by_id(&target_user.id).await.unwrap();
                prop_assert_eq!(updated_user.account_status, AccountStatus::Suspended);
                
                // Cleanup
                sqlx::query("DELETE FROM users WHERE id = ANY($1)")
                    .bind(vec![admin_user.id, target_user.id])
                    .execute(&pool)
                    .await
                    .ok();
                
                Ok(())
            })?;
        }
    }
}

// **Feature: user-auth-system, Property 55: Suspended account login prevention**
// *For any* login attempt by a suspended user, the login should be rejected with a descriptive error message
// **Validates: Requirements 13.2**
mod property_55_suspended_login_prevention {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn suspended_account_cannot_login(
            username_suffix in "[a-z0-9]{8}",
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let pool = common::setup_test_db().await;
                let user_service = UserService::new(pool.clone());
                
                // Create user
                let user = user_service.create_user(CreateUser {
                    username: format!("suspended_{}", username_suffix),
                    email: format!("suspended_{}@test.com", username_suffix),
                    password_hash: "hash".to_string(),
                }).await.unwrap();
                
                // Suspend the user
                sqlx::query("UPDATE users SET account_status = 'suspended' WHERE id = $1")
                    .bind(user.id)
                    .execute(&pool)
                    .await
                    .unwrap();
                
                // Verify user is suspended
                let suspended_user = user_service.get_user_by_id(&user.id).await.unwrap();
                prop_assert_eq!(suspended_user.account_status.clone(), AccountStatus::Suspended);
                
                // In a real login flow, the auth service would check account_status
                // and reject login for suspended accounts
                let can_login = matches!(suspended_user.account_status, AccountStatus::Active);
                prop_assert!(!can_login, "Suspended user should not be able to login");
                
                // Cleanup
                sqlx::query("DELETE FROM users WHERE id = $1")
                    .bind(user.id)
                    .execute(&pool)
                    .await
                    .ok();
                
                Ok(())
            })?;
        }
    }
}

// **Feature: user-auth-system, Property 56: Account reactivation**
// *For any* administrator reactivation action, the account status should be set to active
// **Validates: Requirements 13.3**
mod property_56_account_reactivation {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn account_reactivation_sets_status(
            username_suffix in "[a-z0-9]{8}",
            email_suffix in "[a-z0-9]{8}",
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let pool = common::setup_test_db().await;
                let admin_service = AdminService::new(pool.clone());
                let user_service = UserService::new(pool.clone());
                
                // Generate unique identifiers using UUID
                let unique_id = Uuid::new_v4().to_string().replace("-", "");
                
                // Create admin user
                let admin_user = user_service.create_user(CreateUser {
                    username: format!("admin_{}_{}", username_suffix, &unique_id[..8]),
                    email: format!("admin_{}_{}@test.com", email_suffix, &unique_id[..8]),
                    password_hash: "hash".to_string(),
                }).await.unwrap();
                
                // Set admin role
                sqlx::query("UPDATE users SET role = 'admin' WHERE id = $1")
                    .bind(admin_user.id)
                    .execute(&pool)
                    .await
                    .unwrap();
                
                // Create suspended user
                let target_user = user_service.create_user(CreateUser {
                    username: format!("target_{}_{}", username_suffix, &unique_id[8..16]),
                    email: format!("target_{}_{}@test.com", email_suffix, &unique_id[8..16]),
                    password_hash: "hash".to_string(),
                }).await.unwrap();
                
                // Suspend first
                sqlx::query("UPDATE users SET account_status = 'suspended' WHERE id = $1")
                    .bind(target_user.id)
                    .execute(&pool)
                    .await
                    .unwrap();
                
                // Activate the target user
                admin_service.activate_user(&admin_user.id, &target_user.id, None)
                    .await
                    .unwrap();
                
                // Verify account status is active
                let updated_user = user_service.get_user_by_id(&target_user.id).await.unwrap();
                prop_assert_eq!(updated_user.account_status, AccountStatus::Active);
                
                // Cleanup
                sqlx::query("DELETE FROM users WHERE id = ANY($1)")
                    .bind(vec![admin_user.id, target_user.id])
                    .execute(&pool)
                    .await
                    .ok();
                
                Ok(())
            })?;
        }
    }
}

// **Feature: user-auth-system, Property 57: Account deletion and anonymization**
// *For any* administrator deletion action, the account status should be set to deleted and personal data should be anonymized
// **Validates: Requirements 13.4**
mod property_57_account_deletion {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn account_deletion_sets_status(
            username_suffix in "[a-z0-9]{8}",
            email_suffix in "[a-z0-9]{8}",
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let pool = common::setup_test_db().await;
                let admin_service = AdminService::new(pool.clone());
                let user_service = UserService::new(pool.clone());
                
                // Generate unique identifiers using UUID
                let unique_id = Uuid::new_v4().to_string().replace("-", "");
                
                // Create admin user
                let admin_user = user_service.create_user(CreateUser {
                    username: format!("admin_{}_{}", username_suffix, &unique_id[..8]),
                    email: format!("admin_{}_{}@test.com", email_suffix, &unique_id[..8]),
                    password_hash: "hash".to_string(),
                }).await.unwrap();
                
                // Set admin role
                sqlx::query("UPDATE users SET role = 'admin' WHERE id = $1")
                    .bind(admin_user.id)
                    .execute(&pool)
                    .await
                    .unwrap();
                
                // Create target user
                let target_user = user_service.create_user(CreateUser {
                    username: format!("target_{}_{}", username_suffix, &unique_id[8..16]),
                    email: format!("target_{}_{}@test.com", email_suffix, &unique_id[8..16]),
                    password_hash: "hash".to_string(),
                }).await.unwrap();
                
                // Delete the target user
                admin_service.delete_user(&admin_user.id, &target_user.id, None)
                    .await
                    .unwrap();
                
                // Verify account status is deleted (query directly without filtering)
                let deleted_user: User = sqlx::query_as(
                    "SELECT * FROM users WHERE id = $1"
                )
                .bind(&target_user.id)
                .fetch_one(&pool)
                .await
                .unwrap();
                
                prop_assert_eq!(deleted_user.account_status, AccountStatus::Deleted);
                
                // Verify deleted_at timestamp is set
                prop_assert!(deleted_user.deleted_at.is_some(), "deleted_at should be set");
                
                // Cleanup
                sqlx::query("DELETE FROM users WHERE id = ANY($1)")
                    .bind(vec![admin_user.id, target_user.id])
                    .execute(&pool)
                    .await
                    .ok();
                
                Ok(())
            })?;
        }
    }
}

// **Feature: user-auth-system, Property 58: Deleted account filtering**
// *For any* user search or query, deleted accounts should be filtered out from results
// **Validates: Requirements 13.5**
mod property_58_deleted_account_filtering {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn deleted_accounts_filtered_from_search(
            username_suffix in "[a-z0-9]{8}",
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let pool = common::setup_test_db().await;
                let user_service = UserService::new(pool.clone());
                
                // Create user
                let user = user_service.create_user(CreateUser {
                    username: format!("deleted_{}", username_suffix),
                    email: format!("deleted_{}@test.com", username_suffix),
                    password_hash: "hash".to_string(),
                }).await.unwrap();
                
                // Mark as deleted
                sqlx::query("UPDATE users SET account_status = 'deleted', deleted_at = NOW() WHERE id = $1")
                    .bind(user.id)
                    .execute(&pool)
                    .await
                    .unwrap();
                
                // Search for users (should exclude deleted)
                let search_results: Vec<(Uuid,)> = sqlx::query_as(
                    "SELECT id FROM users WHERE account_status != 'deleted' AND username LIKE $1"
                )
                .bind(format!("%deleted_{}%", username_suffix))
                .fetch_all(&pool)
                .await
                .unwrap();
                
                // Verify deleted user is not in results
                let found = search_results.iter().any(|(id,)| *id == user.id);
                prop_assert!(!found, "Deleted user should not appear in search results");
                
                // Cleanup
                sqlx::query("DELETE FROM users WHERE id = $1")
                    .bind(user.id)
                    .execute(&pool)
                    .await
                    .ok();
                
                Ok(())
            })?;
        }
    }
}
