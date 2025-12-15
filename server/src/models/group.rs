use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Group status enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum GroupStatus {
    Active,
    Dissolved,
}

impl Default for GroupStatus {
    fn default() -> Self {
        GroupStatus::Active
    }
}

/// Member role enum
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq, PartialOrd, Ord, Copy)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum MemberRole {
    Member,
    Admin,
    Owner,
}

impl Default for MemberRole {
    fn default() -> Self {
        MemberRole::Member
    }
}

/// Group model (database representation)
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub owner_id: Uuid,
    pub is_public: bool,
    pub member_count: i32,
    pub max_capacity: i32,
    pub status: GroupStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Group member model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct GroupMember {
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub role: MemberRole,
    pub is_muted: bool,
    pub joined_at: DateTime<Utc>,
}

/// Group member with user info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberInfo {
    pub user_id: Uuid,
    pub username: String,
    pub avatar_url: Option<String>,
    pub role: MemberRole,
    pub is_muted: bool,
    pub online: bool,
    pub joined_at: DateTime<Utc>,
}

/// Group ban record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct GroupBan {
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub banned_by: Uuid,
    pub reason: Option<String>,
    pub banned_at: DateTime<Utc>,
}


/// Invite code model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct InviteCode {
    pub code: String,
    pub group_id: Uuid,
    pub creator_id: Uuid,
    pub usage_count: i32,
    pub max_uses: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Group audit log entry
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct GroupAuditLog {
    pub id: i64,
    pub group_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub action_type: String,
    pub target_user_id: Option<Uuid>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Group creation request
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CreateGroup {
    #[serde(default)]
    pub name: String,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub is_public: Option<bool>,
    pub member_ids: Option<Vec<Uuid>>,
}

/// Group settings update request
#[derive(Debug, Clone, Deserialize, Default)]
pub struct UpdateGroupSettings {
    pub name: Option<String>,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub is_public: Option<bool>,
}

/// Group search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSearchResult {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub member_count: i32,
    pub is_public: bool,
}

/// Group details response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupDetails {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub owner_id: Uuid,
    pub owner_username: Option<String>,
    pub is_public: bool,
    pub member_count: i32,
    pub max_capacity: i32,
    pub status: GroupStatus,
    pub created_at: DateTime<Utc>,
    pub requester_role: Option<MemberRole>,
    pub requester_is_muted: Option<bool>,
}

/// Group audit action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupAuditAction {
    GroupCreated,
    GroupUpdated,
    GroupDissolved,
    MemberJoined,
    MemberLeft,
    MemberKicked,
    MemberBanned,
    MemberUnbanned,
    MemberMuted,
    MemberUnmuted,
    RoleChanged,
    InviteCodeCreated,
    InviteCodeRevoked,
}

impl std::fmt::Display for GroupAuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GroupAuditAction::GroupCreated => write!(f, "group_created"),
            GroupAuditAction::GroupUpdated => write!(f, "group_updated"),
            GroupAuditAction::GroupDissolved => write!(f, "group_dissolved"),
            GroupAuditAction::MemberJoined => write!(f, "member_joined"),
            GroupAuditAction::MemberLeft => write!(f, "member_left"),
            GroupAuditAction::MemberKicked => write!(f, "member_kicked"),
            GroupAuditAction::MemberBanned => write!(f, "member_banned"),
            GroupAuditAction::MemberUnbanned => write!(f, "member_unbanned"),
            GroupAuditAction::MemberMuted => write!(f, "member_muted"),
            GroupAuditAction::MemberUnmuted => write!(f, "member_unmuted"),
            GroupAuditAction::RoleChanged => write!(f, "role_changed"),
            GroupAuditAction::InviteCodeCreated => write!(f, "invite_code_created"),
            GroupAuditAction::InviteCodeRevoked => write!(f, "invite_code_revoked"),
        }
    }
}
