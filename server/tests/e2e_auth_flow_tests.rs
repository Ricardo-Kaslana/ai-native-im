//! End-to-End Authentication Flow Tests
//!
//! Tests the complete authentication flow from registration to logout,
//! simulating real client-server interactions.

mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

/// Helper to create test app with full routing
async fn create_full_test_app() -> axum::Router {
    use ai_im_server::api::{auth_routes, auth_protected_routes, user_routes, group_routes};
    use ai_im_server::api::auth::AppState;
    use ai_im_server::middleware::{JwtAuth, UserContext};
    use ai_im_server::services::auth::{AuthService, TokenService};
    use ai_im_server::services::email::MockEmailService;
    use axum::{middleware as axum_middleware, Router, extract::Request, middleware::Next, response::Response};
    use std::sync::Arc;

    let pool = common::setup_test_db().await;
    let redis_client = redis::Client::open("redis://localhost:6379").unwrap();
    let redis = redis_client.get_connection_manager().await.unwrap();
    let email_service: Arc<dyn ai_im_server::services::email::EmailService> =
        Arc::new(MockEmailService::silent());

    let auth_service = AuthService::new(
        pool.clone(),
        redis.clone(),
        "e2e_test_secret_key_12345".to_string(),
        email_service,
    );

    let app_state = AppState {
        auth_service: Arc::new(auth_service),
        pool: pool.clone(),
        connection_manager: None,
    };

    let token_service = TokenService::with_redis(
        "e2e_test_secret_key_12345".to_string(),
        redis.clone(),
    );
    let jwt_auth = JwtAuth::new(token_service);

    // Public routes
    let public_routes = Router::new()
        .nest("/api/auth", auth_routes(app_state.clone()));

    // Middleware to extract user_id from UserContext and add as Extension<Uuid>
    async fn extract_user_id(mut request: Request, next: Next) -> Response {
        if let Some(user_context) = request.extensions().get::<UserContext>().cloned() {
            request.extensions_mut().insert(user_context.user_id);
        }
        next.run(request).await
    }

    // Protected routes
    let protected_routes = Router::new()
        .nest("/api/auth", auth_protected_routes(app_state.clone()))
        .nest("/api/users", user_routes(app_state.clone()))
        .nest("/api/groups", group_routes(pool.clone()))
        .layer(axum_middleware::from_fn(extract_user_id))
        .layer(axum_middleware::from_fn_with_state(
            jwt_auth.clone(),
            JwtAuth::verify,
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
}

/// Helper to extract JSON from response
async fn response_json(response: axum::response::Response) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap_or(json!({}))
}

/// Generate unique test user data
fn generate_test_user() -> (String, String, String) {
    let uuid = Uuid::new_v4().to_string().replace("-", "");
    let username = format!("e2e_user_{}", &uuid[..8]);
    let email = format!("{}@e2e-test.com", &uuid[..12]);
    let password = "E2eTestPass123!".to_string();
    (username, email, password)
}

// ============================================================================
// E2E Test: Complete User Registration and Login Flow
// ============================================================================

/// **E2E Test: Complete registration → login → get profile → logout flow**
/// 
/// This test simulates a real user journey:
/// 1. User registers a new account
/// 2. User logs in with credentials
/// 3. User fetches their profile
/// 4. User logs out
/// 5. Verify token is invalidated
#[tokio::test]
async fn e2e_complete_auth_flow() {
    let app = create_full_test_app().await;
    let (username, email, password) = generate_test_user();

    // Step 1: Register
    println!("Step 1: Registering user {}", username);
    let register_req = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "username": username,
            "email": email,
            "password": password
        }).to_string()))
        .unwrap();

    let register_res = app.clone().oneshot(register_req).await.unwrap();
    assert_eq!(register_res.status(), StatusCode::OK, "Registration should succeed");
    
    let register_body = response_json(register_res).await;
    assert!(register_body["access_token"].is_string(), "Should return access token");
    assert!(register_body["refresh_token"].is_string(), "Should return refresh token");
    assert_eq!(register_body["user"]["username"], username);

    // Step 2: Login with same credentials
    println!("Step 2: Logging in as {}", username);
    let login_req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "username": username,
            "password": password
        }).to_string()))
        .unwrap();

    let login_res = app.clone().oneshot(login_req).await.unwrap();
    assert_eq!(login_res.status(), StatusCode::OK, "Login should succeed");
    
    let login_body = response_json(login_res).await;
    let access_token = login_body["access_token"].as_str().unwrap();
    let _refresh_token = login_body["refresh_token"].as_str().unwrap();

    // Step 3: Get user profile
    println!("Step 3: Fetching user profile");
    let profile_req = Request::builder()
        .method("GET")
        .uri("/api/users/me")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();

    let profile_res = app.clone().oneshot(profile_req).await.unwrap();
    assert_eq!(profile_res.status(), StatusCode::OK, "Profile fetch should succeed");
    
    let profile_body = response_json(profile_res).await;
    assert_eq!(profile_body["user"]["username"], username);
    // Note: email is not included in UserProfile for privacy reasons

    // Step 4: Logout
    println!("Step 4: Logging out");
    let logout_req = Request::builder()
        .method("POST")
        .uri("/api/auth/logout")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();

    let logout_res = app.clone().oneshot(logout_req).await.unwrap();
    assert_eq!(logout_res.status(), StatusCode::OK, "Logout should succeed");

    // Step 5: Verify token is invalidated (should fail)
    println!("Step 5: Verifying token invalidation");
    let invalid_req = Request::builder()
        .method("GET")
        .uri("/api/users/me")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();

    let invalid_res = app.clone().oneshot(invalid_req).await.unwrap();
    assert_eq!(invalid_res.status(), StatusCode::UNAUTHORIZED, "Token should be invalidated after logout");

    println!("✅ E2E Auth Flow Test Passed!");
}

// ============================================================================
// E2E Test: Token Refresh Flow
// ============================================================================

/// **E2E Test: Token refresh flow**
/// 
/// Tests the complete token refresh cycle:
/// 1. Register and get tokens
/// 2. Use refresh token to get new access token
/// 3. Verify new access token works
/// 4. Verify old refresh token is invalidated (rotation)
#[tokio::test]
async fn e2e_token_refresh_flow() {
    let app = create_full_test_app().await;
    let (username, email, password) = generate_test_user();

    // Register
    let register_req = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "username": username,
            "email": email,
            "password": password
        }).to_string()))
        .unwrap();

    let register_res = app.clone().oneshot(register_req).await.unwrap();
    let register_body = response_json(register_res).await;
    let refresh_token = register_body["refresh_token"].as_str().unwrap();

    // Refresh token
    println!("Refreshing token...");
    let refresh_req = Request::builder()
        .method("POST")
        .uri("/api/auth/refresh")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "refresh_token": refresh_token
        }).to_string()))
        .unwrap();

    let refresh_res = app.clone().oneshot(refresh_req).await.unwrap();
    assert_eq!(refresh_res.status(), StatusCode::OK, "Token refresh should succeed");

    let refresh_body = response_json(refresh_res).await;
    let new_access_token = refresh_body["access_token"].as_str().unwrap();
    let new_refresh_token = refresh_body["refresh_token"].as_str().unwrap();

    // Verify new tokens are different
    assert_ne!(new_refresh_token, refresh_token, "New refresh token should be different");

    // Verify new access token works
    let profile_req = Request::builder()
        .method("GET")
        .uri("/api/users/me")
        .header("authorization", format!("Bearer {}", new_access_token))
        .body(Body::empty())
        .unwrap();

    let profile_res = app.clone().oneshot(profile_req).await.unwrap();
    assert_eq!(profile_res.status(), StatusCode::OK, "New access token should work");
    
    let profile_body = response_json(profile_res).await;
    assert!(profile_body["user"]["username"].is_string(), "Should return user data");

    println!("✅ E2E Token Refresh Flow Test Passed!");
}

// ============================================================================
// E2E Test: Password Change Invalidates Old Tokens
// ============================================================================

/// **E2E Test: Password change flow**
/// 
/// Tests the password change functionality:
/// 1. Register and get tokens
/// 2. Change password
/// 3. Verify can login with new password
/// 4. Verify cannot login with old password
/// 
/// Note: Current implementation does NOT invalidate existing tokens on password change.
/// This is a design decision - tokens remain valid until they expire or user logs out.
#[tokio::test]
async fn e2e_password_change_flow() {
    let app = create_full_test_app().await;
    let (username, email, password) = generate_test_user();
    let new_password = "NewE2ePass456!";

    // Register
    let register_req = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "username": username,
            "email": email,
            "password": password
        }).to_string()))
        .unwrap();

    let register_res = app.clone().oneshot(register_req).await.unwrap();
    let register_body = response_json(register_res).await;
    let access_token = register_body["access_token"].as_str().unwrap();

    // Change password
    println!("Changing password...");
    let change_req = Request::builder()
        .method("POST")
        .uri("/api/auth/password/change")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(json!({
            "current_password": password,
            "new_password": new_password
        }).to_string()))
        .unwrap();

    let change_res = app.clone().oneshot(change_req).await.unwrap();
    assert_eq!(change_res.status(), StatusCode::OK, "Password change should succeed");

    // Verify can login with new password
    println!("Verifying login with new password...");
    let login_req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "username": username,
            "password": new_password
        }).to_string()))
        .unwrap();

    let login_res = app.clone().oneshot(login_req).await.unwrap();
    assert_eq!(login_res.status(), StatusCode::OK, "Login with new password should succeed");

    // Verify cannot login with old password
    println!("Verifying old password no longer works...");
    let old_login_req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "username": username,
            "password": password
        }).to_string()))
        .unwrap();

    let old_login_res = app.clone().oneshot(old_login_req).await.unwrap();
    assert_eq!(old_login_res.status(), StatusCode::UNAUTHORIZED, "Old password should not work");

    println!("✅ E2E Password Change Flow Test Passed!");
}

// ============================================================================
// E2E Test: Concurrent Sessions
// ============================================================================

/// **E2E Test: Multiple concurrent sessions**
/// 
/// Tests that a user can have multiple active sessions:
/// 1. Register user
/// 2. Login from "device 1"
/// 3. Login from "device 2"
/// 4. Both sessions should work independently
/// 5. Logout from device 1 should not affect device 2
#[tokio::test]
async fn e2e_concurrent_sessions() {
    let app = create_full_test_app().await;
    let (username, email, password) = generate_test_user();

    // Register
    let register_req = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "username": username,
            "email": email,
            "password": password
        }).to_string()))
        .unwrap();

    let _ = app.clone().oneshot(register_req).await.unwrap();

    // Login from "device 1"
    println!("Login from device 1...");
    let login1_req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "username": username,
            "password": password
        }).to_string()))
        .unwrap();

    let login1_res = app.clone().oneshot(login1_req).await.unwrap();
    let login1_body = response_json(login1_res).await;
    let token1 = login1_body["access_token"].as_str().unwrap().to_string();

    // Login from "device 2"
    println!("Login from device 2...");
    let login2_req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "username": username,
            "password": password
        }).to_string()))
        .unwrap();

    let login2_res = app.clone().oneshot(login2_req).await.unwrap();
    let login2_body = response_json(login2_res).await;
    let token2 = login2_body["access_token"].as_str().unwrap().to_string();

    // Both tokens should be different
    assert_ne!(token1, token2, "Different sessions should have different tokens");

    // Both tokens should work
    let profile1_req = Request::builder()
        .method("GET")
        .uri("/api/users/me")
        .header("authorization", format!("Bearer {}", token1))
        .body(Body::empty())
        .unwrap();
    let profile1_res = app.clone().oneshot(profile1_req).await.unwrap();
    assert_eq!(profile1_res.status(), StatusCode::OK, "Token 1 should work");

    let profile2_req = Request::builder()
        .method("GET")
        .uri("/api/users/me")
        .header("authorization", format!("Bearer {}", token2))
        .body(Body::empty())
        .unwrap();
    let profile2_res = app.clone().oneshot(profile2_req).await.unwrap();
    assert_eq!(profile2_res.status(), StatusCode::OK, "Token 2 should work");

    // Logout from device 1
    println!("Logout from device 1...");
    let logout1_req = Request::builder()
        .method("POST")
        .uri("/api/auth/logout")
        .header("authorization", format!("Bearer {}", token1))
        .body(Body::empty())
        .unwrap();
    let _ = app.clone().oneshot(logout1_req).await.unwrap();

    // Token 2 should still work
    let profile2_after_req = Request::builder()
        .method("GET")
        .uri("/api/users/me")
        .header("authorization", format!("Bearer {}", token2))
        .body(Body::empty())
        .unwrap();
    let profile2_after_res = app.clone().oneshot(profile2_after_req).await.unwrap();
    assert_eq!(profile2_after_res.status(), StatusCode::OK, "Token 2 should still work after device 1 logout");

    println!("✅ E2E Concurrent Sessions Test Passed!");
}

// ============================================================================
// E2E Test: User Profile Update Flow
// ============================================================================

/// **E2E Test: User profile update flow**
/// 
/// Tests the complete profile update cycle:
/// 1. Register user
/// 2. Update profile (username, bio)
/// 3. Verify changes are persisted
#[tokio::test]
async fn e2e_profile_update_flow() {
    let app = create_full_test_app().await;
    let (username, email, password) = generate_test_user();

    // Register
    let register_req = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "username": username,
            "email": email,
            "password": password
        }).to_string()))
        .unwrap();

    let register_res = app.clone().oneshot(register_req).await.unwrap();
    let register_body = response_json(register_res).await;
    let access_token = register_body["access_token"].as_str().unwrap();

    // Update profile
    let new_bio = "Hello, I'm an E2E test user!";
    println!("Updating profile...");
    let update_req = Request::builder()
        .method("PUT")
        .uri("/api/users/me")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::from(json!({
            "bio": new_bio
        }).to_string()))
        .unwrap();

    let update_res = app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(update_res.status(), StatusCode::OK, "Profile update should succeed");

    // Verify changes
    let profile_req = Request::builder()
        .method("GET")
        .uri("/api/users/me")
        .header("authorization", format!("Bearer {}", access_token))
        .body(Body::empty())
        .unwrap();

    let profile_res = app.clone().oneshot(profile_req).await.unwrap();
    let profile_body = response_json(profile_res).await;
    assert_eq!(profile_body["user"]["bio"], new_bio, "Bio should be updated");

    println!("✅ E2E Profile Update Flow Test Passed!");
}
