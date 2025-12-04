use crate::api::auth::{AppState, ErrorResponse};
use crate::middleware::UserContext;
use crate::models::user::{UpdateUser, UserProfile};
use crate::services::auth::UserError;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post, put},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Update profile request
#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

/// Batch get users request
#[derive(Debug, Deserialize)]
pub struct BatchGetUsersRequest {
    pub user_ids: Vec<Uuid>,
}

/// Search users query
#[derive(Debug, Deserialize)]
pub struct SearchUsersQuery {
    pub q: String,
}

/// User response
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub user: UserProfile,
}

/// Users response
#[derive(Debug, Serialize)]
pub struct UsersResponse {
    pub users: Vec<UserProfile>,
}

/// Convert UserError to HTTP response
impl IntoResponse for UserError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            UserError::NotFound => (StatusCode::NOT_FOUND, "User not found"),
            UserError::UsernameExists => (StatusCode::CONFLICT, "Username already exists"),
            UserError::EmailExists => (StatusCode::CONFLICT, "Email already exists"),
            UserError::InvalidUsername => (StatusCode::BAD_REQUEST, "Invalid username"),
            UserError::InvalidAvatarUrl => (StatusCode::BAD_REQUEST, "Invalid avatar URL"),
            UserError::BioTooLong => (StatusCode::BAD_REQUEST, "Bio too long"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(ErrorResponse {
            error: message.to_string(),
        });

        (status, body).into_response()
    }
}

/// GET /users/me - Get current user profile
async fn get_current_user(
    State(state): State<AppState>,
    Extension(user_context): Extension<UserContext>,
) -> Result<Json<UserResponse>, UserError> {
    let user = state
        .auth_service
        .user_service
        .get_user_by_id(&user_context.user_id)
        .await?;

    Ok(Json(UserResponse { user: user.into() }))
}

/// PUT /users/me - Update current user profile
async fn update_current_user(
    State(state): State<AppState>,
    Extension(user_context): Extension<UserContext>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserResponse>, UserError> {
    let update_data = UpdateUser {
        username: req.username,
        avatar_url: req.avatar_url,
        bio: req.bio,
    };

    let user = state
        .auth_service
        .user_service
        .update_profile(&user_context.user_id, update_data)
        .await?;

    Ok(Json(UserResponse { user }))
}

/// GET /users/:id - Get user profile by ID
async fn get_user_by_id(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserResponse>, UserError> {
    let user = state
        .auth_service
        .user_service
        .get_user_by_id(&user_id)
        .await?;

    Ok(Json(UserResponse { user: user.into() }))
}

/// POST /users/batch - Batch get user profiles
async fn batch_get_users(
    State(state): State<AppState>,
    Json(req): Json<BatchGetUsersRequest>,
) -> Result<Json<UsersResponse>, UserError> {
    // Fetch users one by one (can be optimized later with a batch query)
    let mut profiles = Vec::new();
    for user_id in req.user_ids {
        if let Ok(user) = state
            .auth_service
            .user_service
            .get_user_by_id(&user_id)
            .await
        {
            profiles.push(user.into());
        }
    }

    Ok(Json(UsersResponse { users: profiles }))
}

/// GET /users/search - Search users by username
async fn search_users(
    State(state): State<AppState>,
    Query(query): Query<SearchUsersQuery>,
) -> Result<Json<UsersResponse>, UserError> {
    let users = state
        .auth_service
        .user_service
        .search_users(&query.q)
        .await?;

    let profiles: Vec<UserProfile> = users.into_iter().map(|u| u.into()).collect();

    Ok(Json(UsersResponse { users: profiles }))
}

/// Create user routes
pub fn user_routes(state: AppState) -> Router {
    Router::new()
        .route("/me", get(get_current_user))
        .route("/me", put(update_current_user))
        .route("/:id", get(get_user_by_id))
        .route("/batch", post(batch_get_users))
        .route("/search", get(search_users))
        .with_state(state)
}
