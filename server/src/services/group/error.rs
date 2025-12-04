use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum GroupError {
    #[error("Group not found: {0}")]
    GroupNotFound(Uuid),

    #[error("Group name must be between 1 and 50 characters")]
    InvalidGroupName,

    #[error("Group description must not exceed 500 characters")]
    InvalidDescription,

    #[error("Invalid avatar URL")]
    InvalidAvatarUrl,

    #[error("Group has reached maximum capacity of {0} members")]
    GroupFull(i32),

    #[error("User {0} is already a member of this group")]
    AlreadyMember(Uuid),

    #[error("User {0} is not a member of this group")]
    NotMember(Uuid),

    #[error("User {0} is banned from this group")]
    UserBanned(Uuid),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Cannot remove group owner")]
    CannotRemoveOwner,

    #[error("Admins cannot remove other admins")]
    CannotRemoveAdmin,

    #[error("Only the group owner can perform this action")]
    OwnerOnly,

    #[error("Group has been dissolved")]
    GroupDissolved,

    #[error("Invalid invite code")]
    InvalidInviteCode,

    #[error("Invite code has expired")]
    InviteCodeExpired,

    #[error("Invite code has reached maximum uses")]
    InviteCodeMaxUsesReached,

    #[error("Search query must be at least 2 characters")]
    SearchQueryTooShort,

    #[error("User {0} is already muted")]
    AlreadyMuted(Uuid),

    #[error("User {0} is not muted")]
    NotMuted(Uuid),

    #[error("User {0} is not banned")]
    NotBanned(Uuid),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl GroupError {
    pub fn status_code(&self) -> u16 {
        match self {
            GroupError::GroupNotFound(_) => 404,
            GroupError::InvalidGroupName
            | GroupError::InvalidDescription
            | GroupError::InvalidAvatarUrl
            | GroupError::SearchQueryTooShort => 400,
            GroupError::GroupFull(_)
            | GroupError::AlreadyMember(_)
            | GroupError::UserBanned(_)
            | GroupError::GroupDissolved
            | GroupError::InvalidInviteCode
            | GroupError::InviteCodeExpired
            | GroupError::InviteCodeMaxUsesReached
            | GroupError::AlreadyMuted(_)
            | GroupError::NotMuted(_)
            | GroupError::NotBanned(_) => 400,
            GroupError::NotMember(_)
            | GroupError::PermissionDenied(_)
            | GroupError::CannotRemoveOwner
            | GroupError::CannotRemoveAdmin
            | GroupError::OwnerOnly => 403,
            GroupError::Database(_) | GroupError::Redis(_) | GroupError::Internal(_) => 500,
        }
    }
}
