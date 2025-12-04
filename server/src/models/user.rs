use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Online status enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum OnlineStatus {
    Online,
    Offline,
    Away,
}

/// Account status enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum AccountStatus {
    Active,
    Suspended,
    Deleted,
}

/// User role enum for access control
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, PartialOrd, Ord)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum UserRole {
    User,
    Moderator,
    Admin,
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::User
    }
}

/// User model (database representation)
#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub online_status: OnlineStatus,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub account_status: AccountStatus,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// User profile (public representation, excludes sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub online_status: OnlineStatus,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            avatar_url: user.avatar_url,
            bio: user.bio,
            online_status: user.online_status,
            last_seen_at: user.last_seen_at,
            role: user.role,
            created_at: user.created_at,
        }
    }
}

/// User admin view (for admin panel, includes account status and email)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAdminView {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub role: UserRole,
    pub account_status: AccountStatus,
    pub online_status: OnlineStatus,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserAdminView {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            avatar_url: user.avatar_url,
            role: user.role,
            account_status: user.account_status,
            online_status: user.online_status,
            last_seen_at: user.last_seen_at,
            created_at: user.created_at,
        }
    }
}

/// User creation data
#[derive(Debug, Clone)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

/// User update data
#[derive(Debug, Clone, Default)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}
