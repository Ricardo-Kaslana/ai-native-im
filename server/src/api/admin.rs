use crate::middleware::UserContext;
use crate::models::user::UserRole;
use crate::services::admin::{AdminService, AuditLogQuery, UserSearchQuery};
use crate::api::auth::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Extension, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Generic admin API response
#[derive(Serialize)]
pub struct AdminResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}

/// Pagination query parameters
#[derive(Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_per_page")]
    pub per_page: i32,
}

fn default_page() -> i32 { 1 }
fn default_per_page() -> i32 { 20 }

/// User management request body
#[derive(Deserialize)]
pub struct UserManagementBody {
    pub reason: Option<String>,
}

/// Role change request body
#[derive(Deserialize)]
pub struct RoleChangeBody {
    pub new_role: UserRole,
    pub reason: Option<String>,
}

/// GET /admin/stats - Get admin statistics
async fn get_stats(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
) -> Result<Json<AdminResponse<serde_json::Value>>, StatusCode> {
    let admin_service = AdminService::new(state.pool.clone());

    match admin_service.get_stats(&user_ctx.user_id).await {
        Ok(stats) => Ok(Json(AdminResponse {
            success: true,
            data: Some(serde_json::to_value(stats).unwrap()),
            message: "Statistics retrieved successfully".to_string(),
        })),
        Err(e) => {
            tracing::error!("Failed to get stats: {:?}", e);
            Err(StatusCode::FORBIDDEN)
        }
    }
}


/// GET /admin/users - List users with pagination
async fn list_users(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<AdminResponse<serde_json::Value>>, StatusCode> {
    let admin_service = AdminService::new(state.pool.clone());

    match admin_service.list_users(&user_ctx.user_id, query.page, query.per_page).await {
        Ok(users) => Ok(Json(AdminResponse {
            success: true,
            data: Some(serde_json::to_value(users).unwrap()),
            message: "Users retrieved successfully".to_string(),
        })),
        Err(e) => {
            tracing::error!("Failed to list users: {:?}", e);
            Err(StatusCode::FORBIDDEN)
        }
    }
}

/// GET /admin/users/search - Search users with filters
async fn search_users(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Query(query): Query<UserSearchQuery>,
) -> Result<Json<AdminResponse<serde_json::Value>>, StatusCode> {
    let admin_service = AdminService::new(state.pool.clone());

    match admin_service.search_users(&user_ctx.user_id, query).await {
        Ok(users) => Ok(Json(AdminResponse {
            success: true,
            data: Some(serde_json::to_value(users).unwrap()),
            message: "Search completed successfully".to_string(),
        })),
        Err(e) => {
            tracing::error!("Failed to search users: {:?}", e);
            Err(StatusCode::FORBIDDEN)
        }
    }
}

/// POST /admin/users/:id/suspend - Suspend a user
async fn suspend_user(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<UserManagementBody>,
) -> Result<Json<AdminResponse<()>>, StatusCode> {
    let admin_service = AdminService::new(state.pool.clone());

    match admin_service.suspend_user(&user_ctx.user_id, &user_id, body.reason).await {
        Ok(_) => Ok(Json(AdminResponse {
            success: true,
            data: None,
            message: "User suspended successfully".to_string(),
        })),
        Err(e) => {
            tracing::error!("Failed to suspend user: {:?}", e);
            Err(StatusCode::FORBIDDEN)
        }
    }
}

/// POST /admin/users/:id/activate - Activate a user
async fn activate_user(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<UserManagementBody>,
) -> Result<Json<AdminResponse<()>>, StatusCode> {
    let admin_service = AdminService::new(state.pool.clone());

    match admin_service.activate_user(&user_ctx.user_id, &user_id, body.reason).await {
        Ok(_) => Ok(Json(AdminResponse {
            success: true,
            data: None,
            message: "User activated successfully".to_string(),
        })),
        Err(e) => {
            tracing::error!("Failed to activate user: {:?}", e);
            Err(StatusCode::FORBIDDEN)
        }
    }
}

/// DELETE /admin/users/:id - Soft delete a user
async fn delete_user(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<UserManagementBody>,
) -> Result<Json<AdminResponse<()>>, StatusCode> {
    let admin_service = AdminService::new(state.pool.clone());

    match admin_service.delete_user(&user_ctx.user_id, &user_id, body.reason).await {
        Ok(_) => Ok(Json(AdminResponse {
            success: true,
            data: None,
            message: "User deleted successfully".to_string(),
        })),
        Err(e) => {
            tracing::error!("Failed to delete user: {:?}", e);
            Err(StatusCode::FORBIDDEN)
        }
    }
}

/// PUT /admin/users/:id/role - Change user role
async fn change_user_role(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<RoleChangeBody>,
) -> Result<Json<AdminResponse<()>>, StatusCode> {
    let admin_service = AdminService::new(state.pool.clone());

    match admin_service.change_user_role(&user_ctx.user_id, &user_id, body.new_role, body.reason).await {
        Ok(_) => Ok(Json(AdminResponse {
            success: true,
            data: None,
            message: "User role changed successfully".to_string(),
        })),
        Err(e) => {
            tracing::error!("Failed to change user role: {:?}", e);
            Err(StatusCode::FORBIDDEN)
        }
    }
}

/// GET /admin/audit-logs - Get audit logs
async fn get_audit_logs(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Query(query): Query<AuditLogQuery>,
) -> Result<Json<AdminResponse<serde_json::Value>>, StatusCode> {
    let admin_service = AdminService::new(state.pool.clone());

    match admin_service.get_audit_logs(&user_ctx.user_id, query).await {
        Ok(logs) => Ok(Json(AdminResponse {
            success: true,
            data: Some(serde_json::to_value(logs).unwrap()),
            message: "Audit logs retrieved successfully".to_string(),
        })),
        Err(e) => {
            tracing::error!("Failed to get audit logs: {:?}", e);
            Err(StatusCode::FORBIDDEN)
        }
    }
}

/// Account unlock request body
#[derive(Deserialize)]
pub struct AccountUnlockBody {
    pub username: String,
    pub reason: Option<String>,
}

/// POST /admin/accounts/unlock - Unlock a rate-limited account
async fn unlock_account(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<AccountUnlockBody>,
) -> Result<Json<AdminResponse<()>>, StatusCode> {
    let admin_service = state.admin_service();

    match admin_service
        .unlock_user_account(&user_ctx.user_id, &body.username, body.reason)
        .await
    {
        Ok(_) => Ok(Json(AdminResponse {
            success: true,
            data: None,
            message: format!("Account '{}' unlocked successfully", body.username),
        })),
        Err(e) => {
            tracing::error!("Failed to unlock account: {:?}", e);
            Err(StatusCode::FORBIDDEN)
        }
    }
}

/// Create admin routes
pub fn admin_routes(state: AppState) -> Router {
    Router::new()
        .route("/stats", get(get_stats))
        .route("/users", get(list_users))
        .route("/users/search", get(search_users))
        .route("/users/:id/suspend", post(suspend_user))
        .route("/users/:id/activate", post(activate_user))
        .route("/users/:id", delete(delete_user))
        .route("/users/:id/role", put(change_user_role))
        .route("/audit-logs", get(get_audit_logs))
        .route("/accounts/unlock", post(unlock_account))
        .with_state(state)
}
