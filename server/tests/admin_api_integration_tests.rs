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
    use ai_im_server::api::{admin_routes, auth_protected_routes, auth_routes, user_routes};
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
        "test_secret_key_for_admin_api_tests".to_string(),
        email_service,
    );

    let app_state = AppState {
        auth_service: Arc::new(auth_service),
        pool: pool.clone(),
        connection_manager: None,
    };

    let token_service = TokenService::with_redis(
        "test_secret_key_for_admin_api_tests".to_string(),
        redis.clone(),
    );
    let jwt_auth = JwtAuth::new(token_service);

    // Public routes
    let public_routes = Router::new().nest("/api/auth", auth_routes(app_state.clone()));

    // Protected routes
    let protected_routes = Router::new()
        .nest("/api/auth", auth_protected_routes(app_state.clone()))
        .nest("/api/users", user_routes(app_state.clone()))
        .nest("/api/admin", admin_routes(app_state.clone()))
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

// Helper function to register an admin user
async fn register_admin(app: &axum::Router) -> (String, String) {
    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let username = format!("admin_{}", &uuid_str[..10]);
    let email = format!("{}@example.com", uuid_str);

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "username": username,
                "email": email,
                "password": "AdminPass123!"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = response_json(response).await;
    let access_token = body["access_token"].as_str().unwrap().to_string();
    let user_id = body["user"]["id"].as_str().unwrap().to_string();

    // Manually set user as admin in database
    let pool = common::setup_test_db().await;
    sqlx::query("UPDATE users SET role = 'admin' WHERE id = $1")
        .bind(Uuid::parse_str(&user_id).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    (access_token, user_id)
}

// Helper function to register a regular user
async fn register_user(app: &axum::Router) -> (String, String) {
    let uuid_str = Uuid::new_v4().to_string().replace("-", "");
    let username = format!("user_{}", &uuid_str[..10]);
    let email = format!("{}@example.com", uuid_str);

    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "username": username,
                "email": email,
                "password": "UserPass123!"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let body = response_json(response).await;
    let access_token = body["access_token"].as_str().unwrap().to_string();
    let user_id = body["user"]["id"].as_str().unwrap().to_string();

    (access_token, user_id)
}

/// **Integration Test: Suspend user account**
/// **Validates: Requirements 13.1**
#[tokio::test]
async fn test_suspend_user() {
    let app = create_test_app().await;
    let (admin_token, _) = register_admin(&app).await;
    let (_, user_id) = register_user(&app).await;

    // Suspend the user
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/admin/users/{}/suspend", user_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", admin_token))
        .body(Body::from(
            json!({
                "reason": "Test suspension"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["message"], "User suspended successfully");
}

/// **Integration Test: Activate user account**
/// **Validates: Requirements 13.3**
#[tokio::test]
async fn test_activate_user() {
    let app = create_test_app().await;
    let (admin_token, _) = register_admin(&app).await;
    let (_, user_id) = register_user(&app).await;

    // First suspend the user
    let suspend_request = Request::builder()
        .method("POST")
        .uri(format!("/api/admin/users/{}/suspend", user_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", admin_token))
        .body(Body::from(json!({"reason": "Test"}).to_string()))
        .unwrap();

    app.clone().oneshot(suspend_request).await.unwrap();

    // Then activate the user
    let activate_request = Request::builder()
        .method("POST")
        .uri(format!("/api/admin/users/{}/activate", user_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", admin_token))
        .body(Body::from(
            json!({
                "reason": "Test activation"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(activate_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["message"], "User activated successfully");
}

/// **Integration Test: Delete user account**
/// **Validates: Requirements 13.4**
#[tokio::test]
async fn test_delete_user() {
    let app = create_test_app().await;
    let (admin_token, _) = register_admin(&app).await;
    let (_, user_id) = register_user(&app).await;

    // Delete the user
    let request = Request::builder()
        .method("DELETE")
        .uri(format!("/api/admin/users/{}", user_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", admin_token))
        .body(Body::from(
            json!({
                "reason": "Test deletion"
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["message"], "User deleted successfully");
}

/// **Integration Test: Non-admin cannot access admin endpoints**
/// **Validates: Requirements 13.1**
#[tokio::test]
async fn test_non_admin_authorization() {
    let app = create_test_app().await;
    let (user_token, _) = register_user(&app).await;
    let (_, target_user_id) = register_user(&app).await;

    // Try to suspend a user as non-admin
    let request = Request::builder()
        .method("POST")
        .uri(format!("/api/admin/users/{}/suspend", target_user_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", user_token))
        .body(Body::from(json!({"reason": "Test"}).to_string()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    // Should be forbidden
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

/// **Integration Test: Admin can list users**
#[tokio::test]
async fn test_list_users() {
    let app = create_test_app().await;
    let (admin_token, _) = register_admin(&app).await;

    // Create a few users
    register_user(&app).await;
    register_user(&app).await;

    // List users
    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/users?page=1&per_page=10")
        .header("authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["users"].is_array());
}

/// **Integration Test: Admin can get statistics**
#[tokio::test]
async fn test_get_stats() {
    let app = create_test_app().await;
    let (admin_token, _) = register_admin(&app).await;

    // Get stats
    let request = Request::builder()
        .method("GET")
        .uri("/api/admin/stats")
        .header("authorization", format!("Bearer {}", admin_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_json(response).await;
    assert_eq!(body["success"], true);
    assert!(body["data"]["total_users"].is_number());
}

/// **Integration Test: Admin cannot delete another admin**
#[tokio::test]
async fn test_admin_cannot_delete_admin() {
    let app = create_test_app().await;
    let (admin_token1, _) = register_admin(&app).await;
    let (_, admin_id2) = register_admin(&app).await;

    // Try to delete another admin
    let request = Request::builder()
        .method("DELETE")
        .uri(format!("/api/admin/users/{}", admin_id2))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", admin_token1))
        .body(Body::from(json!({"reason": "Test"}).to_string()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    // Should be forbidden (admins cannot delete other admins)
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
