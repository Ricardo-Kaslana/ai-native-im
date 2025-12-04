use crate::middleware::UserContext;
use crate::services::auth::{AuthError, AuthService, ChangePasswordData, LoginData, RegisterData};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
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

/// Auth response
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: serde_json::Value,
    pub access_token: String,
    pub refresh_token: String,
}

/// Success response
#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub message: String,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Convert AuthError to HTTP response
impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AuthError::AccountLocked => (StatusCode::FORBIDDEN, "Account is locked"),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
            AuthError::UsernameExists => (StatusCode::CONFLICT, "Username already exists"),
            AuthError::EmailExists => (StatusCode::CONFLICT, "Email already exists"),
            AuthError::InvalidPassword => (StatusCode::BAD_REQUEST, "Invalid password format"),
            AuthError::IncorrectCurrentPassword => {
                (StatusCode::BAD_REQUEST, "Current password is incorrect")
            }
            AuthError::UserError(user_err) => {
                // Handle nested UserError
                match user_err {
                    crate::services::auth::UserError::UsernameExists => {
                        (StatusCode::CONFLICT, "Username already exists")
                    }
                    crate::services::auth::UserError::EmailExists => {
                        (StatusCode::CONFLICT, "Email already exists")
                    }
                    crate::services::auth::UserError::NotFound => {
                        (StatusCode::NOT_FOUND, "User not found")
                    }
                    crate::services::auth::UserError::InvalidUsername => {
                        (StatusCode::BAD_REQUEST, "Invalid username")
                    }
                    crate::services::auth::UserError::InvalidAvatarUrl => {
                        (StatusCode::BAD_REQUEST, "Invalid avatar URL")
                    }
                    crate::services::auth::UserError::BioTooLong => {
                        (StatusCode::BAD_REQUEST, "Bio too long")
                    }
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
                }
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(ErrorResponse {
            error: message.to_string(),
        });

        (status, body).into_response()
    }
}

/// POST /auth/register - Register a new user
async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AuthError> {
    let data = RegisterData {
        username: req.username,
        email: req.email,
        password: req.password,
    };

    let (user, tokens) = state.auth_service.register(data).await?;

    let response = AuthResponse {
        user: serde_json::json!({
            "id": user.id,
            "username": user.username,
            "email": user.email,
            "avatar_url": user.avatar_url,
            "bio": user.bio,
            "online_status": user.online_status,
            "created_at": user.created_at,
        }),
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
    };

    Ok(Json(response))
}

/// POST /auth/login - Login a user
async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AuthError> {
    let data = LoginData {
        username: req.username,
        password: req.password,
    };

    // Clone the auth service to make it mutable
    let mut auth_service = (*state.auth_service).clone();
    let (user, tokens) = auth_service.login(data).await?;

    let response = AuthResponse {
        user: serde_json::json!({
            "id": user.id,
            "username": user.username,
            "email": user.email,
            "avatar_url": user.avatar_url,
            "bio": user.bio,
            "online_status": user.online_status,
            "created_at": user.created_at,
        }),
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
    };

    Ok(Json(response))
}

/// POST /auth/logout - Logout a user
async fn logout(
    State(state): State<AppState>,
    Extension(user_context): Extension<UserContext>,
    headers: axum::http::HeaderMap,
) -> Result<Json<SuccessResponse>, AuthError> {
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

    Ok(Json(SuccessResponse {
        message: "Logged out successfully".to_string(),
    }))
}

/// POST /auth/refresh - Refresh access token
async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<Json<AuthResponse>, AuthError> {
    let tokens = state.auth_service.refresh_token(&req.refresh_token).await?;

    // Parse user ID from refresh token to get username
    // For now, just return minimal user info
    let response = AuthResponse {
        user: serde_json::json!({}),
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
    };

    Ok(Json(response))
}

/// POST /auth/password/change - Change password
async fn change_password(
    State(state): State<AppState>,
    Extension(user_context): Extension<UserContext>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<SuccessResponse>, AuthError> {
    let data = ChangePasswordData {
        current_password: req.current_password,
        new_password: req.new_password,
    };

    state
        .auth_service
        .change_password(&user_context.user_id, data)
        .await?;

    Ok(Json(SuccessResponse {
        message: "Password changed successfully".to_string(),
    }))
}

/// POST /auth/password/reset/request - Request password reset
async fn request_password_reset(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<SuccessResponse>, AuthError> {
    let _token = state
        .auth_service
        .request_password_reset(&req.email)
        .await?;

    // In production, send email with token
    // For now, just return success

    Ok(Json(SuccessResponse {
        message: "Password reset email sent".to_string(),
    }))
}

/// POST /auth/password/reset/confirm - Confirm password reset
async fn confirm_password_reset(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordConfirmRequest>,
) -> Result<Json<SuccessResponse>, AuthError> {
    state
        .auth_service
        .complete_password_reset(&req.token, &req.new_password)
        .await?;

    Ok(Json(SuccessResponse {
        message: "Password reset successfully".to_string(),
    }))
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
