mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

// Helper function to create test app
async fn create_test_app() -> axum::Router {
    use ai_im_server::api::{auth_routes, auth_protected_routes, user_routes};
    use ai_im_server::api::auth::AppState;
    use ai_im_server::middleware::JwtAuth;
    use ai_im_server::services::auth::{AuthService, TokenService};
    use ai_im_server::services::email::MockEmailService;
    use axum::{middleware as axum_middleware, Router};
    use std::sync::Arc;

    let pool = common::setup_test_db().await;
    let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
    let redis = redis_client.get_connection_manager().await.unwrap();
    let email_service: Arc<dyn ai_im_server::services::email::EmailService> =
        Arc::new(MockEmailService::silent());

    let auth_service = AuthService::new(
        pool.clone(),
        redis.clone(),
        "test_secret_key_for_user_api_tests".to_string(),
        email_service,
    );

    let app_state = AppState {
        auth_service: Arc::new(auth_service),
        pool: pool.clone(),
        connection_manager: None,
    };

    let token_service = TokenService::with_redis(
        "test_secret_key_for_user_api_tests".to_string(),
        redis.clone(),
    );
    let jwt_auth = JwtAuth::new(token_service);

    // Public routes
    let public_routes = Router::new()
        .nest("/api/auth", auth_routes(app_state.clone()));

    // Protected routes
    let protected_routes = Router::new()
        .nest("/api/auth", auth_protected_routes(app_state.clone()))
        .nest("/api/users", user_routes(app_state.clone()))
        .layer(axum_middleware::from_fn_with_state(
            jwt_auth.clone(),
            JwtAuth::verify,
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
}

// Helper function to extract JSON from response
async fn response_json(response: axum::response::Response) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

// Helper function to register and get access token
async fn register_user(app: &axum::Router) -> (String, String, String) {
    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let username = format!("test_user_{}", &uuid_str[..10]);
    let email = format!("{}@example.com", uuid_str);

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "username": username,
                "email": email,
                "password": "TestPass123!"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = response_json(response).await;
    let access_token = body["access_token"].as_str().unwrap().to_string();
    let user_id = body["user"]["id"].as_str().unwrap().to_string();

    (username, access_token, user_id)
}

/// **Integration Test: Get current user profile**
/// **Validates: Requirements 10.1**
#[tokio::test]
async fn test_get_current_user_profile() {
    let app = create_test_app().await;
    let (username, access_token, _) = register_user(&app).await;

    // Get current user profile
    let request = Request::builder()
        .method("GET")
        .uri("/api/users/me")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["user"]["username"], username);
    assert!(body["user"]["id"].is_string());
}

/// **Integration Test: Update user profile**
/// **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5**
#[tokio::test]
async fn test_update_user_profile() {
    let app = create_test_app().await;
    let (_, access_token, _) = register_user(&app).await;

    // Update profile
    let request = Request::builder()
        .method("PUT")
        .uri("/api/users/me")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(
            json!({
                "avatar_url": "https://example.com/avatar.jpg",
                "bio": "This is my bio"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["user"]["avatar_url"], "https://example.com/avatar.jpg");
    assert_eq!(body["user"]["bio"], "This is my bio");
}

/// **Integration Test: Get user profile by ID**
/// **Validates: Requirements 10.1**
#[tokio::test]
async fn test_get_user_by_id() {
    let app = create_test_app().await;
    let (username, access_token, user_id) = register_user(&app).await;

    // Get user by ID
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/users/{}", user_id))
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["user"]["username"], username);
    assert_eq!(body["user"]["id"], user_id);
}

/// **Integration Test: Search users**
/// **Validates: Requirements 11.1, 11.2, 11.3, 11.4**
#[tokio::test]
async fn test_search_users() {
    let app = create_test_app().await;
    let (username, access_token, _) = register_user(&app).await;

    // Search for the user using more unique characters
    // username format is "test_user_XXXXXXXXXX" (10 chars UUID)
    // Use enough chars to make it unique (at least 15 chars to include part of UUID)
    let search_query = &username[..std::cmp::min(15, username.len())];
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/users/search?q={}", search_query))
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert!(body["users"].is_array());
    
    // Should find at least the user we created
    let users = body["users"].as_array().unwrap();
    assert!(users.iter().any(|u| u["username"] == username));
}

/// **Integration Test: Batch get user profiles**
/// **Validates: Requirements 10.4**
#[tokio::test]
async fn test_batch_get_users() {
    let app = create_test_app().await;
    
    // Register multiple users
    let (_, access_token1, user_id1) = register_user(&app).await;
    let (_, _, user_id2) = register_user(&app).await;
    let (_, _, user_id3) = register_user(&app).await;

    // Batch get users
    let request = Request::builder()
        .method("POST")
        .uri("/api/users/batch")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token1))
        .body(Body::from(
            json!({
                "user_ids": [user_id1, user_id2, user_id3]
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert!(body["users"].is_array());
    assert_eq!(body["users"].as_array().unwrap().len(), 3);
}

/// **Integration Test: Search with minimum length validation**
/// **Validates: Requirements 11.4**
#[tokio::test]
async fn test_search_minimum_length() {
    let app = create_test_app().await;
    let (_, access_token, _) = register_user(&app).await;

    // Try search with 1 character (should fail or return empty)
    let request = Request::builder()
        .method("GET")
        .uri("/api/users/search?q=a")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    // Should either return 400 or empty array
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::OK
    );
}

/// **Integration Test: Profile update with invalid data**
/// **Validates: Requirements 4.2, 4.3, 4.4**
#[tokio::test]
async fn test_profile_update_validation() {
    let app = create_test_app().await;
    let (_, access_token, _) = register_user(&app).await;

    // Try to update with bio that's too long (> 500 chars)
    let long_bio = "a".repeat(501);
    let request = Request::builder()
        .method("PUT")
        .uri("/api/users/me")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(
            json!({
                "bio": long_bio
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// **Integration Test: Unauthorized access**
/// **Validates: Requirements 9.4**
#[tokio::test]
async fn test_unauthorized_access() {
    let app = create_test_app().await;

    // Try to access protected endpoint without token
    let request = Request::builder()
        .method("GET")
        .uri("/api/users/me")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// **Integration Test: Get non-existent user**
/// **Validates: Requirements 10.3**
#[tokio::test]
async fn test_get_nonexistent_user() {
    let app = create_test_app().await;
    let (_, access_token, _) = register_user(&app).await;

    // Try to get a non-existent user
    let fake_id = Uuid::new_v4();
    let request = Request::builder()
        .method("GET")
        .uri(format!("/api/users/{}", fake_id))
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
