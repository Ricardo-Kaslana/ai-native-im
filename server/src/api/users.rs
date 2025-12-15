use crate::api::auth::AppState;
use crate::error::AppError;
use crate::middleware::UserContext;
use crate::models::user::{UpdateUser, UserProfile};
use crate::response::ApiResponse;
use axum::{
    extract::{Path, Query, State},
    response::Json,
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

/// User response data
#[derive(Debug, Serialize)]
pub struct UserData {
    pub user: UserProfile,
}

/// Users list response data
#[derive(Debug, Serialize)]
pub struct UsersData {
    pub users: Vec<UserProfile>,
}

/// GET /users/me - Get current user profile
async fn get_current_user(
    State(state): State<AppState>,
    Extension(user_context): Extension<UserContext>,
) -> Result<Json<ApiResponse<UserData>>, AppError> {
    let user = state
        .auth_service
        .user_service
        .get_user_by_id(&user_context.user_id)
        .await
        .map_err(|e| AppError::from(crate::services::auth::AuthError::UserError(e)))?;

    Ok(Json(ApiResponse::success(UserData { user: user.into() })))
}

/// PUT /users/me - Update current user profile
async fn update_current_user(
    State(state): State<AppState>,
    Extension(user_context): Extension<UserContext>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<ApiResponse<UserData>>, AppError> {
    let update_data = UpdateUser {
        username: req.username,
        avatar_url: req.avatar_url,
        bio: req.bio,
    };

    let user = state
        .auth_service
        .user_service
        .update_profile(&user_context.user_id, update_data)
        .await
        .map_err(|e| AppError::from(crate::services::auth::AuthError::UserError(e)))?;

    Ok(Json(ApiResponse::success(UserData { user })))
}

/// GET /users/:id - Get user profile by ID
async fn get_user_by_id(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<UserData>>, AppError> {
    let user = state
        .auth_service
        .user_service
        .get_user_by_id(&user_id)
        .await
        .map_err(|e| AppError::from(crate::services::auth::AuthError::UserError(e)))?;

    Ok(Json(ApiResponse::success(UserData { user: user.into() })))
}

/// POST /users/batch - Batch get user profiles
async fn batch_get_users(
    State(state): State<AppState>,
    Json(req): Json<BatchGetUsersRequest>,
) -> Result<Json<ApiResponse<UsersData>>, AppError> {
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

    Ok(Json(ApiResponse::success(UsersData { users: profiles })))
}

/// GET /users/search - Search users by username
async fn search_users(
    State(state): State<AppState>,
    Query(query): Query<SearchUsersQuery>,
) -> Result<Json<ApiResponse<UsersData>>, AppError> {
    let users = state
        .auth_service
        .user_service
        .search_users(&query.q)
        .await
        .map_err(|e| AppError::from(crate::services::auth::AuthError::UserError(e)))?;

    let profiles: Vec<UserProfile> = users.into_iter().map(|u| u.into()).collect();

    Ok(Json(ApiResponse::success(UsersData { users: profiles })))
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
