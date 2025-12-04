use crate::models::{GroupStatus, MemberRole};
use sqlx::PgPool;
use uuid::Uuid;

use super::GroupError;

/// Permission types for group actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    ViewGroup,
    SendMessage,
    InviteMembers,
    RemoveMembers,
    UpdateSettings,
    ManageRoles,
    MuteMembers,
    BanMembers,
    DissolveGroup,
}

/// Permission service for checking user permissions in groups
#[derive(Clone)]
pub struct PermissionService {
    pool: PgPool,
}

impl PermissionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get user's role in a group
    pub async fn get_user_role(
        &self,
        group_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<MemberRole>, GroupError> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT role as "role: MemberRole"
            FROM group_members
            WHERE group_id = $1 AND user_id = $2
            "#,
            group_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Check if user is a member of the group
    pub async fn is_member(&self, group_id: Uuid, user_id: Uuid) -> Result<bool, GroupError> {
        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM group_members
                WHERE group_id = $1 AND user_id = $2
            ) as "exists!"
            "#,
            group_id,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    /// Check if user is banned from the group
    pub async fn is_banned(&self, group_id: Uuid, user_id: Uuid) -> Result<bool, GroupError> {
        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM group_bans
                WHERE group_id = $1 AND user_id = $2
            ) as "exists!"
            "#,
            group_id,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    /// Check if user is muted in the group
    pub async fn is_muted(&self, group_id: Uuid, user_id: Uuid) -> Result<bool, GroupError> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT is_muted
            FROM group_members
            WHERE group_id = $1 AND user_id = $2
            "#,
            group_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.unwrap_or(false))
    }

    /// Check if group is active
    pub async fn is_group_active(&self, group_id: Uuid) -> Result<bool, GroupError> {
        let status = sqlx::query_scalar!(
            r#"
            SELECT status as "status: GroupStatus"
            FROM groups
            WHERE id = $1
            "#,
            group_id
        )
        .fetch_optional(&self.pool)
        .await?;

        match status {
            Some(GroupStatus::Active) => Ok(true),
            Some(GroupStatus::Dissolved) => Ok(false),
            None => Err(GroupError::GroupNotFound(group_id)),
        }
    }


    /// Check if user has a specific permission
    pub async fn check_permission(
        &self,
        group_id: Uuid,
        user_id: Uuid,
        permission: Permission,
    ) -> Result<bool, GroupError> {
        // First check if group is active
        if !self.is_group_active(group_id).await? {
            return Err(GroupError::GroupDissolved);
        }

        // Get user's role
        let role = match self.get_user_role(group_id, user_id).await? {
            Some(role) => role,
            None => return Ok(false), // Not a member
        };

        // Check permission based on role
        let has_permission = match permission {
            Permission::ViewGroup | Permission::SendMessage => {
                // All members can view and send (unless muted for SendMessage)
                if permission == Permission::SendMessage {
                    !self.is_muted(group_id, user_id).await?
                } else {
                    true
                }
            }
            Permission::InviteMembers => {
                // Owner and admins can invite
                matches!(role, MemberRole::Owner | MemberRole::Admin)
            }
            Permission::RemoveMembers | Permission::MuteMembers => {
                // Owner and admins can remove/mute members
                matches!(role, MemberRole::Owner | MemberRole::Admin)
            }
            Permission::UpdateSettings => {
                // Owner and admins can update settings
                matches!(role, MemberRole::Owner | MemberRole::Admin)
            }
            Permission::ManageRoles | Permission::BanMembers | Permission::DissolveGroup => {
                // Only owner can manage roles, ban, or dissolve
                matches!(role, MemberRole::Owner)
            }
        };

        Ok(has_permission)
    }

    /// Check if actor can remove target from group
    pub async fn can_remove_member(
        &self,
        group_id: Uuid,
        actor_id: Uuid,
        target_id: Uuid,
    ) -> Result<(), GroupError> {
        let actor_role = self
            .get_user_role(group_id, actor_id)
            .await?
            .ok_or(GroupError::NotMember(actor_id))?;

        let target_role = self
            .get_user_role(group_id, target_id)
            .await?
            .ok_or(GroupError::NotMember(target_id))?;

        // Owner can remove anyone except themselves
        if actor_role == MemberRole::Owner {
            if actor_id == target_id {
                return Err(GroupError::CannotRemoveOwner);
            }
            return Ok(());
        }

        // Admin can only remove members, not owner or other admins
        if actor_role == MemberRole::Admin {
            match target_role {
                MemberRole::Owner => return Err(GroupError::CannotRemoveOwner),
                MemberRole::Admin => return Err(GroupError::CannotRemoveAdmin),
                MemberRole::Member => return Ok(()),
            }
        }

        // Members cannot remove anyone
        Err(GroupError::PermissionDenied(
            "Only owner or admin can remove members".to_string(),
        ))
    }

    /// Require user to be group owner
    pub async fn require_owner(&self, group_id: Uuid, user_id: Uuid) -> Result<(), GroupError> {
        let role = self
            .get_user_role(group_id, user_id)
            .await?
            .ok_or(GroupError::NotMember(user_id))?;

        if role != MemberRole::Owner {
            return Err(GroupError::OwnerOnly);
        }

        Ok(())
    }

    /// Require user to be owner or admin
    pub async fn require_admin_or_owner(
        &self,
        group_id: Uuid,
        user_id: Uuid,
    ) -> Result<MemberRole, GroupError> {
        let role = self
            .get_user_role(group_id, user_id)
            .await?
            .ok_or(GroupError::NotMember(user_id))?;

        if !matches!(role, MemberRole::Owner | MemberRole::Admin) {
            return Err(GroupError::PermissionDenied(
                "Only owner or admin can perform this action".to_string(),
            ));
        }

        Ok(role)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_enum() {
        // Basic enum test
        assert_ne!(Permission::ViewGroup, Permission::SendMessage);
        assert_eq!(Permission::ViewGroup, Permission::ViewGroup);
    }

    #[test]
    fn test_member_role_ordering() {
        // Test role ordering for permission checks
        assert!(MemberRole::Owner > MemberRole::Admin);
        assert!(MemberRole::Admin > MemberRole::Member);
    }
}
