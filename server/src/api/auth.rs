use crate::error::AppError;
use crate::middleware::UserContext;
use crate::response::ApiResponse;
use crate::services::auth::{AuthService, ChangePasswordData, LoginData, RegisterData};
use axum::{
    extract::State,
    response::Json,
    routing::post,
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub pool: sqlx::PgPool,
    pub connection_manager: Option<Arc<crate::api::ConnectionManager>>,
}

/// Registration request
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    #[serde(alias = "email")]
    pub username: String,
    pub password: String,
}

/// Password change request
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// Password reset request
#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
}

/// Password reset confirm request
#[derive(Debug, Deserialize)]
pub struct ResetPasswordConfirmRequest {
    pub token: String,
    pub new_password: String,
}

/// Refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// User info in auth response
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub online_status: crate::models::user::OnlineStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Auth response data
#[derive(Debug, Serialize)]
pub struct AuthResponseData {
    pub user: UserInfo,
    pub access_token: String,
    pub refresh_token: String,
}

/// Token refresh response data
#[derive(Debug, Serialize)]
pub struct TokenRefreshData {
    pub access_token: String,
    pub refresh_token: String,
}

/// Success message response
#[derive(Debug, Serialize)]
pub struct MessageData {
    pub message: String,
}

/// POST /auth/register - Register a new user
async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<AuthResponseData>>, AppError> {
    tracing::info!("📝 Registration request: username={}, email={}", req.username, req.email);
    
    let data = RegisterData {
        username: req.username,
        email: req.email,
        password: req.password,
    };

    let (user, tokens) = state.auth_service.register(data).await?;
    tracing::info!("✅ User registered successfully: {}", user.id);

    let response = AuthResponseData {
        user: UserInfo {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
            avatar_url: user.avatar_url,
            bio: user.bio,
            online_status: user.online_status,
            created_at: user.created_at,
        },
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// POST /auth/login - Login a user
async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<AuthResponseData>>, AppError> {
    let data = LoginData {
        username: req.username,
        password: req.password,
    };

    let mut auth_service = (*state.auth_service).clone();
    let (user, tokens) = auth_service.login(data).await?;

    let response = AuthResponseData {
        user: UserInfo {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
            avatar_url: user.avatar_url,
            bio: user.bio,
            online_status: user.online_status,
            created_at: user.created_at,
        },
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
    };

    Ok(Json(ApiResponse::success(response)))
}


/// POST /auth/logout - Logout a user
async fn logout(
    State(state): State<AppState>,
    Extension(user_context): Extension<UserContext>,
    headers: axum::http::HeaderMap,
) -> Result<Json<ApiResponse<MessageData>>, AppError> {
    // Extract token from Authorization header
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(&header[7..])
            } else {
                None
            }
        });

    state.auth_service.logout(&user_context.user_id, token).await?;

    // Close all WebSocket connections for this user
    if let Some(ref connection_manager) = state.connection_manager {
        connection_manager.close_user_connections(&user_context.user_id).await;
        tracing::info!("Closed all WebSocket connections for user {}", user_context.user_id);
    }

    Ok(Json(ApiResponse::success(MessageData {
        message: "Logged out successfully".to_string(),
    })))
}

/// POST /auth/refresh - Refresh access token
async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<ApiResponse<TokenRefreshData>>, AppError> {
    let tokens = state.auth_service.refresh_token(&req.refresh_token).await?;

    Ok(Json(ApiResponse::success(TokenRefreshData {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
    })))
}

/// POST /auth/password/change - Change password
async fn change_password(
    State(state): State<AppState>,
    Extension(user_context): Extension<UserContext>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<ApiResponse<MessageData>>, AppError> {
    let data = ChangePasswordData {
        current_password: req.current_password,
        new_password: req.new_password,
    };

    state
        .auth_service
        .change_password(&user_context.user_id, data)
        .await?;

    Ok(Json(ApiResponse::success(MessageData {
        message: "Password changed successfully".to_string(),
    })))
}

/// POST /auth/password/reset/request - Request password reset
async fn request_password_reset(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<ApiResponse<MessageData>>, AppError> {
    let _token = state
        .auth_service
        .request_password_reset(&req.email)
        .await?;

    Ok(Json(ApiResponse::success(MessageData {
        message: "Password reset email sent".to_string(),
    })))
}

/// POST /auth/password/reset/confirm - Confirm password reset
async fn confirm_password_reset(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordConfirmRequest>,
) -> Result<Json<ApiResponse<MessageData>>, AppError> {
    state
        .auth_service
        .complete_password_reset(&req.token, &req.new_password)
        .await?;

    Ok(Json(ApiResponse::success(MessageData {
        message: "Password reset successfully".to_string(),
    })))
}

/// Create public auth routes (no authentication required)
pub fn auth_routes(state: AppState) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
        .route("/password/reset/request", post(request_password_reset))
        .route("/password/reset/confirm", post(confirm_password_reset))
        .with_state(state)
}

/// Create protected auth routes (authentication required)
pub fn auth_protected_routes(state: AppState) -> Router {
    Router::new()
        .route("/logout", post(logout))
        .route("/password/change", post(change_password))
        .with_state(state)
}
