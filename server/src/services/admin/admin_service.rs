use crate::models::user::{AccountStatus, User, UserAdminView, UserRole};
use crate::services::auth::{RateLimiter, UserService};
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Admin service errors
#[derive(Debug, Error)]
pub enum AdminError {
    #[error("User not found")]
    UserNotFound,

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Cannot modify user with higher privileges")]
    InsufficientPrivileges,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

/// Admin action types for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminAction {
    SuspendUser,
    ActivateUser,
    DeleteUser,
    ChangeRole,
    UnlockAccount,
    ViewUsers,
    ViewStats,
}

/// User management request
#[derive(Debug, Deserialize)]
pub struct UserManagementRequest {
    pub reason: Option<String>,
}

/// Role change request
#[derive(Debug, Deserialize)]
pub struct RoleChangeRequest {
    pub new_role: UserRole,
    pub reason: Option<String>,
}

/// User search query parameters
#[derive(Debug, Deserialize, Default)]
pub struct UserSearchQuery {
    pub username: Option<String>,
    pub email: Option<String>,
    pub role: Option<UserRole>,
    pub account_status: Option<AccountStatus>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}


/// Admin statistics response
#[derive(Debug, Serialize)]
pub struct AdminStats {
    pub total_users: i64,
    pub active_users: i64,
    pub suspended_users: i64,
    pub deleted_users: i64,
    pub online_users: i64,
    pub users_by_role: HashMap<String, i64>,
}

/// User list response with pagination
#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserAdminView>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

/// Admin service for user management
#[derive(Clone)]
pub struct AdminService {
    pool: PgPool,
    user_service: UserService,
    redis: Option<ConnectionManager>,
}

impl AdminService {
    /// Create a new admin service
    pub fn new(pool: PgPool) -> Self {
        Self {
            user_service: UserService::new(pool.clone()),
            pool,
            redis: None,
        }
    }

    /// Create a new admin service with Redis support (for rate limiter functions)
    pub fn with_redis(pool: PgPool, redis: ConnectionManager) -> Self {
        Self {
            user_service: UserService::new(pool.clone()),
            pool,
            redis: Some(redis),
        }
    }

    /// Check if user has admin or moderator privileges
    /// Returns the user's role if they have sufficient privileges
    pub async fn check_admin_permission(&self, user_id: &Uuid) -> Result<UserRole, AdminError> {
        let role = self
            .user_service
            .get_user_role(user_id)
            .await
            .map_err(|_| AdminError::UserNotFound)?;

        match role {
            UserRole::Admin | UserRole::Moderator => Ok(role),
            UserRole::User => Err(AdminError::PermissionDenied(
                "Admin or Moderator role required".to_string(),
            )),
        }
    }

    /// Check if user has admin-only privileges (not moderator)
    pub async fn check_admin_only_permission(&self, user_id: &Uuid) -> Result<(), AdminError> {
        let role = self
            .user_service
            .get_user_role(user_id)
            .await
            .map_err(|_| AdminError::UserNotFound)?;

        match role {
            UserRole::Admin => Ok(()),
            _ => Err(AdminError::PermissionDenied("Admin role required".to_string())),
        }
    }

    /// Check if admin can modify target user based on privilege hierarchy
    fn can_modify_user(&self, admin_role: &UserRole, target_role: &UserRole) -> bool {
        match (admin_role, target_role) {
            (UserRole::Admin, _) => true,
            (UserRole::Moderator, UserRole::User) => true,
            _ => false,
        }
    }
}


impl AdminService {
    /// Suspend a user account
    pub async fn suspend_user(
        &self,
        admin_id: &Uuid,
        target_user_id: &Uuid,
        reason: Option<String>,
    ) -> Result<(), AdminError> {
        // Check admin permission
        let admin_role = self.check_admin_permission(admin_id).await?;

        // Get target user
        let target_user = self
            .user_service
            .get_user_by_id(target_user_id)
            .await
            .map_err(|_| AdminError::UserNotFound)?;

        // Check privilege hierarchy
        if !self.can_modify_user(&admin_role, &target_user.role) {
            return Err(AdminError::InsufficientPrivileges);
        }

        // Update account status
        sqlx::query("UPDATE users SET account_status = 'suspended', updated_at = NOW() WHERE id = $1")
            .bind(target_user_id)
            .execute(&self.pool)
            .await?;

        // Log admin action
        self.log_admin_action(admin_id, target_user_id, AdminAction::SuspendUser, reason)
            .await?;

        Ok(())
    }

    /// Activate a suspended user account
    pub async fn activate_user(
        &self,
        admin_id: &Uuid,
        target_user_id: &Uuid,
        reason: Option<String>,
    ) -> Result<(), AdminError> {
        // Check admin permission
        let admin_role = self.check_admin_permission(admin_id).await?;

        // Get target user
        let target_user = self
            .user_service
            .get_user_by_id(target_user_id)
            .await
            .map_err(|_| AdminError::UserNotFound)?;

        // Check privilege hierarchy
        if !self.can_modify_user(&admin_role, &target_user.role) {
            return Err(AdminError::InsufficientPrivileges);
        }

        // Update account status
        sqlx::query("UPDATE users SET account_status = 'active', updated_at = NOW() WHERE id = $1")
            .bind(target_user_id)
            .execute(&self.pool)
            .await?;

        // Log admin action
        self.log_admin_action(admin_id, target_user_id, AdminAction::ActivateUser, reason)
            .await?;

        Ok(())
    }

    /// Soft delete a user account (Admin only)
    pub async fn delete_user(
        &self,
        admin_id: &Uuid,
        target_user_id: &Uuid,
        reason: Option<String>,
    ) -> Result<(), AdminError> {
        // Only Admin can delete users
        self.check_admin_only_permission(admin_id).await?;

        // Get target user to verify existence
        let target_user = self
            .user_service
            .get_user_by_id(target_user_id)
            .await
            .map_err(|_| AdminError::UserNotFound)?;

        // Admin cannot delete other admins
        if target_user.role == UserRole::Admin && admin_id != target_user_id {
            return Err(AdminError::InsufficientPrivileges);
        }

        // Soft delete
        sqlx::query("UPDATE users SET account_status = 'deleted', deleted_at = NOW(), updated_at = NOW() WHERE id = $1")
            .bind(target_user_id)
            .execute(&self.pool)
            .await?;

        // Log admin action
        self.log_admin_action(admin_id, target_user_id, AdminAction::DeleteUser, reason)
            .await?;

        Ok(())
    }

    /// Change user role (Admin only)
    pub async fn change_user_role(
        &self,
        admin_id: &Uuid,
        target_user_id: &Uuid,
        new_role: UserRole,
        reason: Option<String>,
    ) -> Result<(), AdminError> {
        // Only Admin can change roles
        self.check_admin_only_permission(admin_id).await?;

        // Verify target user exists
        self.user_service
            .get_user_by_id(target_user_id)
            .await
            .map_err(|_| AdminError::UserNotFound)?;

        // Update role
        sqlx::query("UPDATE users SET role = $1, updated_at = NOW() WHERE id = $2")
            .bind(&new_role)
            .bind(target_user_id)
            .execute(&self.pool)
            .await?;

        // Log admin action
        self.log_admin_action(admin_id, target_user_id, AdminAction::ChangeRole, reason)
            .await?;

        Ok(())
    }

    /// Unlock a user account (clear rate limit lockout)
    /// Admin or Moderator can unlock accounts
    pub async fn unlock_user_account(
        &self,
        admin_id: &Uuid,
        username: &str,
        reason: Option<String>,
    ) -> Result<(), AdminError> {
        // Check admin permission
        self.check_admin_permission(admin_id).await?;

        // Get user by username to verify existence and get user_id
        let user = self
            .user_service
            .get_user_by_username(username)
            .await
            .map_err(|_| AdminError::UserNotFound)?;

        // Clear rate limit lockout if Redis is available
        if let Some(redis) = self.redis.clone() {
            let mut rate_limiter = RateLimiter::with_defaults(redis);
            rate_limiter
                .unlock_account(username)
                .await
                .map_err(|e| AdminError::InvalidOperation(format!("Failed to unlock account: {}", e)))?;
        }

        // Log admin action
        self.log_admin_action(admin_id, &user.id, AdminAction::UnlockAccount, reason)
            .await?;

        Ok(())
    }
}


impl AdminService {
    /// Log admin action to audit log
    pub async fn log_admin_action(
        &self,
        admin_id: &Uuid,
        target_id: &Uuid,
        action: AdminAction,
        reason: Option<String>,
    ) -> Result<(), AdminError> {
        let admin = self
            .user_service
            .get_user_by_id(admin_id)
            .await
            .map_err(|_| AdminError::UserNotFound)?;

        let details = serde_json::json!({
            "action": action,
            "target_user_id": target_id,
            "reason": reason
        });

        sqlx::query(
            r#"
            INSERT INTO security_audit_log (user_id, username, event_type, user_agent, details)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(admin_id)
        .bind(&admin.username)
        .bind("admin_action")
        .bind(None::<String>)
        .bind(&details)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get admin statistics
    pub async fn get_stats(&self, admin_id: &Uuid) -> Result<AdminStats, AdminError> {
        // Check admin permission
        self.check_admin_permission(admin_id).await?;

        // Get counts
        let stats: (i64, i64, i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT 
                COUNT(*) as total,
                COUNT(CASE WHEN account_status = 'active' THEN 1 END) as active,
                COUNT(CASE WHEN account_status = 'suspended' THEN 1 END) as suspended,
                COUNT(CASE WHEN account_status = 'deleted' THEN 1 END) as deleted,
                COUNT(CASE WHEN online_status = 'online' THEN 1 END) as online
            FROM users
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        // Get users by role
        let role_counts: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT role, COUNT(*) as count
            FROM users
            WHERE account_status != 'deleted'
            GROUP BY role
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let users_by_role: HashMap<String, i64> = role_counts.into_iter().collect();

        Ok(AdminStats {
            total_users: stats.0,
            active_users: stats.1,
            suspended_users: stats.2,
            deleted_users: stats.3,
            online_users: stats.4,
            users_by_role,
        })
    }

    /// List users with pagination
    pub async fn list_users(
        &self,
        admin_id: &Uuid,
        page: i32,
        per_page: i32,
    ) -> Result<UserListResponse, AdminError> {
        // Check admin permission
        self.check_admin_permission(admin_id).await?;

        let page = page.max(1);
        let per_page = per_page.clamp(1, 100);
        let offset = (page - 1) * per_page;

        // Get total count
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE account_status != 'deleted'")
            .fetch_one(&self.pool)
            .await?;

        // Get users
        let users: Vec<User> = sqlx::query_as(
            r#"
            SELECT * FROM users
            WHERE account_status != 'deleted'
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(UserListResponse {
            users: users.into_iter().map(UserAdminView::from).collect(),
            total: total.0,
            page,
            per_page,
        })
    }

    /// Search users with filters
    pub async fn search_users(
        &self,
        admin_id: &Uuid,
        query: UserSearchQuery,
    ) -> Result<UserListResponse, AdminError> {
        // Check admin permission
        self.check_admin_permission(admin_id).await?;

        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * per_page;

        // Build dynamic query
        let mut conditions = vec!["account_status != 'deleted'".to_string()];
        let mut bind_idx = 1;

        if query.username.is_some() {
            conditions.push(format!("LOWER(username) LIKE ${}", bind_idx));
            bind_idx += 1;
        }
        if query.email.is_some() {
            conditions.push(format!("LOWER(email) LIKE ${}", bind_idx));
            bind_idx += 1;
        }
        if query.role.is_some() {
            conditions.push(format!("role = ${}", bind_idx));
            bind_idx += 1;
        }
        if query.account_status.is_some() {
            conditions.push(format!("account_status = ${}", bind_idx));
            bind_idx += 1;
        }

        let where_clause = conditions.join(" AND ");
        let count_sql = format!("SELECT COUNT(*) FROM users WHERE {}", where_clause);
        let select_sql = format!(
            "SELECT * FROM users WHERE {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
            where_clause, bind_idx, bind_idx + 1
        );

        // Build count query
        let mut count_query = sqlx::query_as::<_, (i64,)>(&count_sql);
        if let Some(ref username) = query.username {
            count_query = count_query.bind(format!("%{}%", username.to_lowercase()));
        }
        if let Some(ref email) = query.email {
            count_query = count_query.bind(format!("%{}%", email.to_lowercase()));
        }
        if let Some(ref role) = query.role {
            count_query = count_query.bind(role);
        }
        if let Some(ref status) = query.account_status {
            count_query = count_query.bind(status);
        }

        let total = count_query.fetch_one(&self.pool).await?.0;

        // Build select query
        let mut select_query = sqlx::query_as::<_, User>(&select_sql);
        if let Some(ref username) = query.username {
            select_query = select_query.bind(format!("%{}%", username.to_lowercase()));
        }
        if let Some(ref email) = query.email {
            select_query = select_query.bind(format!("%{}%", email.to_lowercase()));
        }
        if let Some(ref role) = query.role {
            select_query = select_query.bind(role);
        }
        if let Some(ref status) = query.account_status {
            select_query = select_query.bind(status);
        }
        select_query = select_query.bind(per_page).bind(offset);

        let users = select_query.fetch_all(&self.pool).await?;

        Ok(UserListResponse {
            users: users.into_iter().map(UserAdminView::from).collect(),
            total,
            page,
            per_page,
        })
    }
}


/// Audit log entry for API response
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub event_type: String,
    pub details: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Audit log query parameters
#[derive(Debug, Deserialize, Default)]
pub struct AuditLogQuery {
    pub admin_id: Option<Uuid>,
    pub action_type: Option<String>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

/// Audit log list response
#[derive(Debug, Serialize)]
pub struct AuditLogResponse {
    pub logs: Vec<AuditLogEntry>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

impl AdminService {
    /// Get audit logs with filtering and pagination
    pub async fn get_audit_logs(
        &self,
        admin_id: &Uuid,
        query: AuditLogQuery,
    ) -> Result<AuditLogResponse, AdminError> {
        // Only Admin can view audit logs
        self.check_admin_only_permission(admin_id).await?;

        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * per_page;

        // Build query with filters
        let mut conditions = vec!["1=1".to_string()];
        
        if query.admin_id.is_some() {
            conditions.push("user_id = $1".to_string());
        }
        if query.action_type.is_some() {
            conditions.push(format!("event_type = ${}", if query.admin_id.is_some() { 2 } else { 1 }));
        }

        let where_clause = conditions.join(" AND ");
        
        // Simple query without complex filtering for now
        let logs: Vec<AuditLogEntry> = sqlx::query_as(
            r#"
            SELECT id, user_id, username, event_type, details, created_at
            FROM security_audit_log
            WHERE event_type = 'admin_action'
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM security_audit_log WHERE event_type = 'admin_action'"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(AuditLogResponse {
            logs,
            total: total.0,
            page,
            per_page,
        })
    }
}
