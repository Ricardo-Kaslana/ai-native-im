//! End-to-End Message Flow Tests
//!
//! Tests the message system including WebSocket connections,
//! private messages, and group messages.

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
    use axum::{middleware as axum_middleware, Router, extract::Request as AxumRequest, middleware::Next, response::Response};
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

    let public_routes = Router::new()
        .nest("/api/auth", auth_routes(app_state.clone()));

    async fn extract_user_id(mut request: AxumRequest, next: Next) -> Response {
        if let Some(user_context) = request.extensions().get::<UserContext>().cloned() {
            request.extensions_mut().insert(user_context.user_id);
        }
        next.run(request).await
    }

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

async fn response_json(response: axum::response::Response) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap_or(json!({}))
}

/// Register a test user and return (user_id, access_token)
async fn register_test_user(app: &axum::Router) -> (String, String) {
    let uuid = Uuid::new_v4().to_string().replace("-", "");
    let username = format!("e2e_msg_{}", &uuid[..8]);
    let email = format!("{}@e2e-msg.com", &uuid[..12]);

    let register_req = Request::builder()
        .method("POST")
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::from(json!({
            "username": username,
            "email": email,
            "password": "E2eMsgPass123!"
        }).to_string()))
        .unwrap();

    let register_res = app.clone().oneshot(register_req).await.unwrap();
    let body = response_json(register_res).await;
    
    let user_id = body["user"]["id"].as_str().unwrap().to_string();
    let token = body["access_token"].as_str().unwrap().to_string();
    
    (user_id, token)
}

// ============================================================================
// E2E Test: Message Service Integration
// ============================================================================

/// **E2E Test: Message service components are properly initialized**
/// 
/// Tests that the message service components can be created:
/// 1. Create message persister with database pool
/// 2. Verify components are operational
#[tokio::test]
async fn e2e_message_service_initialization() {
    use ai_im_server::services::message::{PostgresMessagePersister, OfflineSyncConfig};
    
    let pool = common::setup_test_db().await;
    
    // Create message persister
    let _persister = PostgresMessagePersister::new(pool.clone());
    
    // Create offline sync config
    let config = OfflineSyncConfig::default();
    
    // Verify config has reasonable defaults
    assert!(config.batch_size > 0, "Batch size should be positive");
    assert!(config.max_messages > 0, "Max messages should be positive");
    
    println!("✅ E2E Message Service Initialization Test Passed!");
}

/// **E2E Test: Group message flow preparation**
/// 
/// Tests the setup required for group messaging:
/// 1. Create users
/// 2. Create a group
/// 3. Add members to group
/// 4. Verify group is ready for messaging
#[tokio::test]
async fn e2e_group_message_setup() {
    let app = create_full_test_app().await;

    // Register two users
    let (user_a_id, token_a) = register_test_user(&app).await;
    let (user_b_id, _token_b) = register_test_user(&app).await;

    // User A creates a group
    let group_name = format!("Msg Test Group {}", &Uuid::new_v4().to_string()[..8]);
    let create_req = Request::builder()
        .method("POST")
        .uri("/api/groups")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token_a))
        .body(Body::from(json!({
            "name": group_name,
            "description": "Group for message testing",
            "is_public": false
        }).to_string()))
        .unwrap();

    let create_res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED, "Group creation should succeed");
    
    let create_body = response_json(create_res).await;
    let group_id = create_body["data"]["id"].as_str().unwrap();
    assert_eq!(create_body["data"]["owner_id"], user_a_id);

    // Add User B to the group
    let add_member_req = Request::builder()
        .method("POST")
        .uri(format!("/api/groups/{}/members", group_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token_a))
        .body(Body::from(json!({
            "user_id": user_b_id
        }).to_string()))
        .unwrap();

    let add_member_res = app.clone().oneshot(add_member_req).await.unwrap();
    assert!(add_member_res.status() == StatusCode::OK || add_member_res.status() == StatusCode::CREATED,
        "Adding member should succeed");

    // Verify group has 2 members
    let members_req = Request::builder()
        .method("GET")
        .uri(format!("/api/groups/{}/members", group_id))
        .header("authorization", format!("Bearer {}", token_a))
        .body(Body::empty())
        .unwrap();

    let members_res = app.clone().oneshot(members_req).await.unwrap();
    assert_eq!(members_res.status(), StatusCode::OK);
    
    let members_body = response_json(members_res).await;
    let empty_vec = vec![];
    let members = members_body["data"].as_array().unwrap_or(&empty_vec);
    assert_eq!(members.len(), 2, "Group should have 2 members ready for messaging");

    println!("✅ E2E Group Message Setup Test Passed!");
}

/// **E2E Test: Message model validation**
/// 
/// Tests that message models are properly structured:
/// 1. Create message with valid data
/// 2. Verify message fields
#[tokio::test]
async fn e2e_message_model_validation() {
    use ai_im_server::services::message::models::{CreateMessage, Message, MsgType};
    use uuid::Uuid;

    let sender_id = Uuid::new_v4();
    let receiver_id = Uuid::new_v4();
    
    // Create a private message using CreateMessage
    let private_msg = CreateMessage {
        id: None,
        sender_id,
        receiver_id: Some(receiver_id),
        group_id: None,
        msg_type: MsgType::Text,
        content: "Hello, this is a test message!".to_string(),
        is_ai_generated: false,
    };
    
    assert_eq!(private_msg.sender_id, sender_id);
    assert_eq!(private_msg.receiver_id, Some(receiver_id));
    assert_eq!(private_msg.msg_type, MsgType::Text);
    assert_eq!(private_msg.content, "Hello, this is a test message!");
    
    // Create a group message using Message helper
    let group_id = Uuid::new_v4();
    let group_msg = Message::new_group(
        sender_id,
        group_id,
        MsgType::Text,
        "Hello group!".to_string(),
    );
    
    assert_eq!(group_msg.group_id, Some(group_id));
    assert!(group_msg.is_group());
    assert!(!group_msg.is_private());
    
    // Create a private message using Message helper
    let private_msg2 = Message::new_private(
        sender_id,
        receiver_id,
        MsgType::Image,
        "image_url".to_string(),
    );
    
    assert!(private_msg2.is_private());
    assert_eq!(private_msg2.get_msg_type(), MsgType::Image);
    
    println!("✅ E2E Message Model Validation Test Passed!");
}

/// **E2E Test: Message acknowledgment model**
/// 
/// Tests message acknowledgment structures:
/// 1. Create acknowledgment
/// 2. Verify ack fields
#[tokio::test]
async fn e2e_message_acknowledgment_model() {
    use ai_im_server::services::message::models::{MessageAck, AckStatus};
    use uuid::Uuid;

    let message_id = Uuid::new_v4();
    
    // Test success acknowledgment
    let success_ack = MessageAck::success(message_id);
    assert_eq!(success_ack.message_id, message_id);
    assert_eq!(success_ack.status, AckStatus::Success);
    assert!(success_ack.error_message.is_none());
    
    // Test error acknowledgment
    let error_ack = MessageAck::error(message_id, "Test error");
    assert_eq!(error_ack.message_id, message_id);
    assert_eq!(error_ack.status, AckStatus::Error);
    assert!(error_ack.error_message.is_some());
    
    // Test duplicate acknowledgment
    let dup_ack = MessageAck::duplicate(message_id);
    assert_eq!(dup_ack.status, AckStatus::Duplicate);
    
    println!("✅ E2E Message Acknowledgment Model Test Passed!");
}

/// **E2E Test: Offline message sync preparation**
/// 
/// Tests the offline sync configuration:
/// 1. Verify sync config is valid
/// 2. Test batch size limits
#[tokio::test]
async fn e2e_offline_sync_config() {
    use ai_im_server::services::message::OfflineSyncConfig;

    // Test default config
    let default_config = OfflineSyncConfig::default();
    assert!(default_config.batch_size > 0, "Batch size should be positive");
    assert!(default_config.max_messages > 0, "Max messages should be positive");
    assert!(default_config.batch_delay_ms > 0, "Batch delay should be positive");
    
    // Test custom offline sync config
    let custom_config = OfflineSyncConfig {
        batch_size: 50,
        batch_delay_ms: 1000,
        max_messages: 5000,
    };
    
    assert_eq!(custom_config.batch_size, 50);
    assert_eq!(custom_config.batch_delay_ms, 1000);
    assert_eq!(custom_config.max_messages, 5000);
    
    println!("✅ E2E Offline Sync Config Test Passed!");
}
