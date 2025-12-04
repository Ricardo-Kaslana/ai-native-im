mod common;

use ai_im_server::models::user::CreateUser;
use ai_im_server::services::auth::UserService;
use redis::Client as RedisClient;
use uuid::Uuid;

#[tokio::test]
async fn test_cache_hit_scenario() {
    let pool = common::setup_test_db().await;
    
    // Set up Redis
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let redis_client = RedisClient::open(redis_url).unwrap();
    let redis_conn = redis_client.get_connection_manager().await.unwrap();
    
    let service = UserService::with_redis(pool.clone(), redis_conn.clone());

    // Create a test user
    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let create_data = CreateUser {
        username: format!("cache_test_{}", &uuid_str[..8]),
        email: format!("cache_{}@example.com", Uuid::new_v4()),
        password_hash: "hashed_password".to_string(),
    };

    let user = service.create_user(create_data).await.unwrap();

    // First call - should hit database and cache the result
    let profile1 = service.get_profile_by_id(&user.id).await.unwrap();
    assert_eq!(profile1.id, user.id);
    assert_eq!(profile1.username, user.username);

    // Second call - should hit cache
    let profile2 = service.get_profile_by_id(&user.id).await.unwrap();
    assert_eq!(profile2.id, user.id);
    assert_eq!(profile2.username, user.username);

    // Profiles should be identical
    assert_eq!(profile1.username, profile2.username);

    // Clean up
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.id)
        .execute(&pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_cache_invalidation_on_update() {
    let pool = common::setup_test_db().await;
    
    // Set up Redis
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let redis_client = RedisClient::open(redis_url).unwrap();
    let redis_conn = redis_client.get_connection_manager().await.unwrap();
    
    let service = UserService::with_redis(pool.clone(), redis_conn.clone());

    // Create a test user
    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let create_data = CreateUser {
        username: format!("cache_inv_{}", &uuid_str[..8]),
        email: format!("cache_inv_{}@example.com", Uuid::new_v4()),
        password_hash: "hashed_password".to_string(),
    };

    let user = service.create_user(create_data).await.unwrap();

    // Get profile to cache it
    let profile1 = service.get_profile_by_id(&user.id).await.unwrap();
    assert_eq!(profile1.bio, None);

    // Update profile
    let update_data = ai_im_server::models::user::UpdateUser {
        username: None,
        avatar_url: None,
        bio: Some("Updated bio".to_string()),
    };
    service.update_profile(&user.id, update_data).await.unwrap();

    // Get profile again - should reflect the update (cache should be invalidated)
    let profile2 = service.get_profile_by_id(&user.id).await.unwrap();
    assert_eq!(profile2.bio, Some("Updated bio".to_string()));

    // Clean up
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.id)
        .execute(&pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_cache_miss_for_nonexistent_user() {
    let pool = common::setup_test_db().await;
    
    // Set up Redis
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let redis_client = RedisClient::open(redis_url).unwrap();
    let redis_conn = redis_client.get_connection_manager().await.unwrap();
    
    let service = UserService::with_redis(pool.clone(), redis_conn);

    // Try to get a non-existent user
    let non_existent_id = Uuid::new_v4();
    let result = service.get_profile_by_id(&non_existent_id).await;

    // Should return NotFound error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_graceful_degradation_without_redis() {
    let pool = common::setup_test_db().await;
    
    // Create service without Redis
    let service = UserService::new(pool.clone());

    // Create a test user
    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let create_data = CreateUser {
        username: format!("no_redis_{}", &uuid_str[..8]),
        email: format!("no_redis_{}@example.com", Uuid::new_v4()),
        password_hash: "hashed_password".to_string(),
    };

    let user = service.create_user(create_data).await.unwrap();

    // Should still work without Redis
    let profile = service.get_profile_by_id(&user.id).await.unwrap();
    assert_eq!(profile.id, user.id);
    assert_eq!(profile.username, user.username);

    // Clean up
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.id)
        .execute(&pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_cache_invalidation_on_status_change() {
    let pool = common::setup_test_db().await;
    
    // Set up Redis
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let redis_client = RedisClient::open(redis_url).unwrap();
    let redis_conn = redis_client.get_connection_manager().await.unwrap();
    
    let service = UserService::with_redis(pool.clone(), redis_conn);

    // Create a test user
    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let create_data = CreateUser {
        username: format!("status_test_{}", &uuid_str[..8]),
        email: format!("status_{}@example.com", Uuid::new_v4()),
        password_hash: "hashed_password".to_string(),
    };

    let user = service.create_user(create_data).await.unwrap();

    // Get profile to cache it
    let profile1 = service.get_profile_by_id(&user.id).await.unwrap();
    assert_eq!(profile1.online_status, ai_im_server::models::user::OnlineStatus::Offline);

    // Update online status
    service
        .update_online_status(&user.id, ai_im_server::models::user::OnlineStatus::Online)
        .await
        .unwrap();

    // Get profile again - should reflect the status change
    let profile2 = service.get_profile_by_id(&user.id).await.unwrap();
    assert_eq!(profile2.online_status, ai_im_server::models::user::OnlineStatus::Online);

    // Clean up
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.id)
        .execute(&pool)
        .await
        .ok();
}
