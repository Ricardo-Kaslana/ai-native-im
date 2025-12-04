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
        "test_secret_key_for_integration_tests".to_string(),
        email_service,
    );

    let app_state = AppState {
        auth_service: Arc::new(auth_service),
        pool: pool.clone(),
        connection_manager: None,
    };

    let token_service = TokenService::with_redis(
        "test_secret_key_for_integration_tests".to_string(),
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

/// **Integration Test: Full registration flow**
/// **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5**
#[tokio::test]
async fn test_registration_flow() {
    let app = create_test_app().await;

    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let username = format!("test_reg_{}", &uuid_str[..10]);
    let email = format!("{}@example.com", uuid_str);

    // Register a new user
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
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["user"]["username"], username);
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
}

/// **Integration Test: Login and token refresh flow**
/// **Validates: Requirements 2.1, 2.2, 2.3, 3.1**
#[tokio::test]
async fn test_login_and_refresh_flow() {
    let app = create_test_app().await;

    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let username = format!("test_login_{}", &uuid_str[..10]);
    let email = format!("{}@example.com", uuid_str);
    let password = "TestPass123!";

    // Register first
    let register_request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "username": username,
                "email": email,
                "password": password
            })
            .to_string(),
        ))
        .unwrap();

    let register_response = app.clone().oneshot(register_request).await.unwrap();
    assert_eq!(register_response.status(), StatusCode::OK);

    // Login
    let login_request = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "username": username,
                "password": password
            })
            .to_string(),
        ))
        .unwrap();

    let login_response = app.clone().oneshot(login_request).await.unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);

    let login_body = response_json(login_response).await;
    let refresh_token = login_body["refresh_token"].as_str().unwrap();

    // Refresh token
    let refresh_request = Request::builder()
        .method("POST")
        .uri("/api/auth/refresh")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "refresh_token": refresh_token
            })
            .to_string(),
        ))
        .unwrap();

    let refresh_response = app.clone().oneshot(refresh_request).await.unwrap();
    assert_eq!(refresh_response.status(), StatusCode::OK);

    let refresh_body = response_json(refresh_response).await;
    assert!(refresh_body["access_token"].is_string());
    assert!(refresh_body["refresh_token"].is_string());
}

/// **Integration Test: Password change flow**
/// **Validates: Requirements 5.1, 5.2, 5.3, 5.4**
#[tokio::test]
async fn test_password_change_flow() {
    let app = create_test_app().await;

    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let username = format!("test_pwchange_{}", &uuid_str[..10]);
    let email = format!("{}@example.com", uuid_str);
    let old_password = "OldPass123!";
    let new_password = "NewPass123!";

    // Register
    let register_request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "username": username,
                "email": email,
                "password": old_password
            })
            .to_string(),
        ))
        .unwrap();

    let register_response = app.clone().oneshot(register_request).await.unwrap();
    let register_body = response_json(register_response).await;
    let access_token = register_body["access_token"].as_str().unwrap();

    // Change password
    let change_request = Request::builder()
        .method("POST")
        .uri("/api/auth/password/change")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(
            json!({
                "current_password": old_password,
                "new_password": new_password
            })
            .to_string(),
        ))
        .unwrap();

    let change_response = app.clone().oneshot(change_request).await.unwrap();
    assert_eq!(change_response.status(), StatusCode::OK);

    // Try login with new password
    let login_request = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "username": username,
                "password": new_password
            })
            .to_string(),
        ))
        .unwrap();

    let login_response = app.clone().oneshot(login_request).await.unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);
}

/// **Integration Test: Password reset flow**
/// **Validates: Requirements 6.1, 6.2, 6.3, 6.4**
#[tokio::test]
async fn test_password_reset_flow() {
    let app = create_test_app().await;

    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let username = format!("test_reset_{}", &uuid_str[..10]);
    let email = format!("{}@example.com", uuid_str);
    let password = "OldPass123!";

    // Register
    let register_request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "username": username,
                "email": email,
                "password": password
            })
            .to_string(),
        ))
        .unwrap();

    let _register_response = app.clone().oneshot(register_request).await.unwrap();

    // Request password reset
    let reset_request = Request::builder()
        .method("POST")
        .uri("/api/auth/password/reset/request")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": email
            })
            .to_string(),
        ))
        .unwrap();

    let reset_response = app.clone().oneshot(reset_request).await.unwrap();
    assert_eq!(reset_response.status(), StatusCode::OK);

    // Note: In a real test, we would extract the token from the email
    // For now, we just verify the request was successful
}

/// **Integration Test: Logout flow**
/// **Validates: Requirements 12.1, 12.2**
#[tokio::test]
async fn test_logout_flow() {
    let app = create_test_app().await;

    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let username = format!("test_logout_{}", &uuid_str[..10]);
    let email = format!("{}@example.com", uuid_str);
    let password = "TestPass123!";

    // Register
    let register_request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "username": username,
                "email": email,
                "password": password
            })
            .to_string(),
        ))
        .unwrap();

    let register_response = app.clone().oneshot(register_request).await.unwrap();
    let register_body = response_json(register_response).await;
    let access_token = register_body["access_token"].as_str().unwrap();

    // Logout
    let logout_request = Request::builder()
        .method("POST")
        .uri("/api/auth/logout")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();

    let logout_response = app.clone().oneshot(logout_request).await.unwrap();
    assert_eq!(logout_response.status(), StatusCode::OK);
}

/// **Integration Test: Invalid credentials**
/// **Validates: Requirements 2.5**
#[tokio::test]
async fn test_invalid_credentials() {
    let app = create_test_app().await;

    // Try login with non-existent user
    let login_request = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "username": "nonexistent_user",
                "password": "WrongPass123!"
            })
            .to_string(),
        ))
        .unwrap();

    let login_response = app.clone().oneshot(login_request).await.unwrap();
    assert_eq!(login_response.status(), StatusCode::UNAUTHORIZED);
}

/// **Integration Test: Duplicate username**
/// **Validates: Requirements 1.1**
#[tokio::test]
async fn test_duplicate_username() {
    let app = create_test_app().await;

    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let username = format!("test_dup_{}", &uuid_str[..10]);

    // Register first user
    let register1_request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "username": username,
                "email": format!("user1_{}@example.com", uuid_str),
                "password": "TestPass123!"
            })
            .to_string(),
        ))
        .unwrap();

    let register1_response = app.clone().oneshot(register1_request).await.unwrap();
    assert_eq!(register1_response.status(), StatusCode::OK);

    // Try to register with same username
    let register2_request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "username": username,
                "email": format!("user2_{}@example.com", uuid_str),
                "password": "TestPass123!"
            })
            .to_string(),
        ))
        .unwrap();

    let register2_response = app.clone().oneshot(register2_request).await.unwrap();
    assert_eq!(register2_response.status(), StatusCode::CONFLICT);
}
