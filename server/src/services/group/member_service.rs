use crate::models::{GroupAuditAction, GroupMember, GroupMemberInfo, GroupStatus, MemberRole};
use sqlx::PgPool;
use uuid::Uuid;

use super::{GroupError, GroupService, PermissionService};

/// Member service for managing group membership
#[derive(Clone)]
pub struct MemberService {
    pool: PgPool,
    permission_service: PermissionService,
    group_service: GroupService,
}

impl MemberService {
    pub fn new(pool: PgPool) -> Self {
        let permission_service = PermissionService::new(pool.clone());
        let group_service = GroupService::new(pool.clone());
        Self {
            pool,
            permission_service,
            group_service,
        }
    }

    /// Add a member to a group
    pub async fn add_member(
        &self,
        group_id: Uuid,
        user_id: Uuid,
        inviter_id: Uuid,
    ) -> Result<GroupMember, GroupError> {
        // Check inviter has permission
        self.permission_service
            .require_admin_or_owner(group_id, inviter_id)
            .await?;

        self.add_member_internal(group_id, user_id, MemberRole::Member, Some(inviter_id))
            .await
    }

    /// Internal method to add member (used by invite code join too)
    pub(crate) async fn add_member_internal(
        &self,
        group_id: Uuid,
        user_id: Uuid,
        role: MemberRole,
        inviter_id: Option<Uuid>,
    ) -> Result<GroupMember, GroupError> {
        // Check if group is active
        let group = self.group_service.get_group(group_id).await?;
        if group.status == GroupStatus::Dissolved {
            return Err(GroupError::GroupDissolved);
        }

        // Check capacity
        if group.member_count >= group.max_capacity {
            return Err(GroupError::GroupFull(group.max_capacity));
        }

        // Check if user is banned
        if self.permission_service.is_banned(group_id, user_id).await? {
            return Err(GroupError::UserBanned(user_id));
        }

        // Check if already a member
        if self.permission_service.is_member(group_id, user_id).await? {
            return Err(GroupError::AlreadyMember(user_id));
        }

        let mut tx = self.pool.begin().await?;

        // Add member
        let member = sqlx::query_as!(
            GroupMember,
            r#"
            INSERT INTO group_members (group_id, user_id, role)
            VALUES ($1, $2, $3)
            RETURNING group_id, user_id, role as "role: MemberRole", is_muted, joined_at
            "#,
            group_id,
            user_id,
            role as MemberRole
        )
        .fetch_one(&mut *tx)
        .await?;

        // Increment member count
        self.group_service
            .increment_member_count(&mut tx, group_id)
            .await?;

        // Log audit event
        self.group_service
            .log_audit_event(
                &mut tx,
                group_id,
                inviter_id,
                GroupAuditAction::MemberJoined,
                Some(user_id),
                None,
                None,
                None,
            )
            .await?;

        tx.commit().await?;

        Ok(member)
    }

    /// Remove a member from a group (kick)
    pub async fn remove_member(
        &self,
        group_id: Uuid,
        target_id: Uuid,
        actor_id: Uuid,
    ) -> Result<(), GroupError> {
        // Check permission to remove
        self.permission_service
            .can_remove_member(group_id, actor_id, target_id)
            .await?;

        let mut tx = self.pool.begin().await?;

        // Remove member
        let result = sqlx::query!(
            r#"DELETE FROM group_members WHERE group_id = $1 AND user_id = $2"#,
            group_id,
            target_id
        )
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(GroupError::NotMember(target_id));
        }

        // Decrement member count
        self.group_service
            .decrement_member_count(&mut tx, group_id)
            .await?;

        // Log audit event
        self.group_service
            .log_audit_event(
                &mut tx,
                group_id,
                Some(actor_id),
                GroupAuditAction::MemberKicked,
                Some(target_id),
                None,
                None,
                None,
            )
            .await?;

        tx.commit().await?;

        Ok(())
    }


    /// Leave a group voluntarily
    pub async fn leave_group(&self, group_id: Uuid, user_id: Uuid) -> Result<(), GroupError> {
        let role = self
            .permission_service
            .get_user_role(group_id, user_id)
            .await?
            .ok_or(GroupError::NotMember(user_id))?;

        let mut tx = self.pool.begin().await?;

        // If owner is leaving, transfer ownership or dissolve
        if role == MemberRole::Owner {
            // Find oldest admin to transfer ownership
            let new_owner = sqlx::query_scalar!(
                r#"
                SELECT user_id
                FROM group_members
                WHERE group_id = $1 AND role = 'admin'
                ORDER BY joined_at ASC
                LIMIT 1
                "#,
                group_id
            )
            .fetch_optional(&mut *tx)
            .await?;

            if let Some(new_owner_id) = new_owner {
                // Transfer ownership
                sqlx::query!(
                    r#"UPDATE group_members SET role = 'owner' WHERE group_id = $1 AND user_id = $2"#,
                    group_id,
                    new_owner_id
                )
                .execute(&mut *tx)
                .await?;

                sqlx::query!(
                    r#"UPDATE groups SET owner_id = $2 WHERE id = $1"#,
                    group_id,
                    new_owner_id
                )
                .execute(&mut *tx)
                .await?;

                // Log ownership transfer
                self.group_service
                    .log_audit_event(
                        &mut tx,
                        group_id,
                        Some(user_id),
                        GroupAuditAction::RoleChanged,
                        Some(new_owner_id),
                        Some("admin"),
                        Some("owner"),
                        Some("Ownership transferred due to owner leaving"),
                    )
                    .await?;
            } else {
                // No admins, check for any other members
                let has_members = sqlx::query_scalar!(
                    r#"
                    SELECT EXISTS(
                        SELECT 1 FROM group_members
                        WHERE group_id = $1 AND user_id != $2
                    ) as "exists!"
                    "#,
                    group_id,
                    user_id
                )
                .fetch_one(&mut *tx)
                .await?;

                if has_members {
                    // Find oldest member to become owner
                    let new_owner_id = sqlx::query_scalar!(
                        r#"
                        SELECT user_id
                        FROM group_members
                        WHERE group_id = $1 AND user_id != $2
                        ORDER BY joined_at ASC
                        LIMIT 1
                        "#,
                        group_id,
                        user_id
                    )
                    .fetch_one(&mut *tx)
                    .await?;

                    sqlx::query!(
                        r#"UPDATE group_members SET role = 'owner' WHERE group_id = $1 AND user_id = $2"#,
                        group_id,
                        new_owner_id
                    )
                    .execute(&mut *tx)
                    .await?;

                    sqlx::query!(
                        r#"UPDATE groups SET owner_id = $2 WHERE id = $1"#,
                        group_id,
                        new_owner_id
                    )
                    .execute(&mut *tx)
                    .await?;
                } else {
                    // No other members, dissolve group
                    sqlx::query!(
                        r#"UPDATE groups SET status = 'dissolved' WHERE id = $1"#,
                        group_id
                    )
                    .execute(&mut *tx)
                    .await?;

                    self.group_service
                        .log_audit_event(
                            &mut tx,
                            group_id,
                            Some(user_id),
                            GroupAuditAction::GroupDissolved,
                            None,
                            None,
                            None,
                            Some("Group dissolved due to owner leaving with no other members"),
                        )
                        .await?;
                }
            }
        }

        // Remove the member
        sqlx::query!(
            r#"DELETE FROM group_members WHERE group_id = $1 AND user_id = $2"#,
            group_id,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        // Decrement member count
        self.group_service
            .decrement_member_count(&mut tx, group_id)
            .await?;

        // Log leave event
        self.group_service
            .log_audit_event(
                &mut tx,
                group_id,
                Some(user_id),
                GroupAuditAction::MemberLeft,
                Some(user_id),
                None,
                None,
                None,
            )
            .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Update member role (promote/demote)
    pub async fn update_member_role(
        &self,
        group_id: Uuid,
        target_id: Uuid,
        new_role: MemberRole,
        actor_id: Uuid,
    ) -> Result<GroupMember, GroupError> {
        // Only owner can change roles
        self.permission_service
            .require_owner(group_id, actor_id)
            .await?;

        // Cannot change owner's role
        let current_role = self
            .permission_service
            .get_user_role(group_id, target_id)
            .await?
            .ok_or(GroupError::NotMember(target_id))?;

        if current_role == MemberRole::Owner {
            return Err(GroupError::CannotRemoveOwner);
        }

        // Cannot promote to owner
        if new_role == MemberRole::Owner {
            return Err(GroupError::PermissionDenied(
                "Cannot promote to owner. Use ownership transfer instead.".to_string(),
            ));
        }

        let mut tx = self.pool.begin().await?;

        let member = sqlx::query_as!(
            GroupMember,
            r#"
            UPDATE group_members
            SET role = $3
            WHERE group_id = $1 AND user_id = $2
            RETURNING group_id, user_id, role as "role: MemberRole", is_muted, joined_at
            "#,
            group_id,
            target_id,
            new_role as MemberRole
        )
        .fetch_one(&mut *tx)
        .await?;

        // Log audit event
        self.group_service
            .log_audit_event(
                &mut tx,
                group_id,
                Some(actor_id),
                GroupAuditAction::RoleChanged,
                Some(target_id),
                Some(&format!("{:?}", current_role)),
                Some(&format!("{:?}", new_role)),
                None,
            )
            .await?;

        tx.commit().await?;

        Ok(member)
    }


    /// Mute a member
    pub async fn mute_member(
        &self,
        group_id: Uuid,
        target_id: Uuid,
        actor_id: Uuid,
    ) -> Result<(), GroupError> {
        // Check permission
        self.permission_service
            .require_admin_or_owner(group_id, actor_id)
            .await?;

        // Cannot mute owner
        let target_role = self
            .permission_service
            .get_user_role(group_id, target_id)
            .await?
            .ok_or(GroupError::NotMember(target_id))?;

        if target_role == MemberRole::Owner {
            return Err(GroupError::PermissionDenied("Cannot mute group owner".to_string()));
        }

        // Check if already muted
        if self.permission_service.is_muted(group_id, target_id).await? {
            return Err(GroupError::AlreadyMuted(target_id));
        }

        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"UPDATE group_members SET is_muted = TRUE WHERE group_id = $1 AND user_id = $2"#,
            group_id,
            target_id
        )
        .execute(&mut *tx)
        .await?;

        self.group_service
            .log_audit_event(
                &mut tx,
                group_id,
                Some(actor_id),
                GroupAuditAction::MemberMuted,
                Some(target_id),
                None,
                None,
                None,
            )
            .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Unmute a member
    pub async fn unmute_member(
        &self,
        group_id: Uuid,
        target_id: Uuid,
        actor_id: Uuid,
    ) -> Result<(), GroupError> {
        // Check permission
        self.permission_service
            .require_admin_or_owner(group_id, actor_id)
            .await?;

        // Check if muted
        if !self.permission_service.is_muted(group_id, target_id).await? {
            return Err(GroupError::NotMuted(target_id));
        }

        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"UPDATE group_members SET is_muted = FALSE WHERE group_id = $1 AND user_id = $2"#,
            group_id,
            target_id
        )
        .execute(&mut *tx)
        .await?;

        self.group_service
            .log_audit_event(
                &mut tx,
                group_id,
                Some(actor_id),
                GroupAuditAction::MemberUnmuted,
                Some(target_id),
                None,
                None,
                None,
            )
            .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Ban a user from the group
    pub async fn ban_member(
        &self,
        group_id: Uuid,
        target_id: Uuid,
        actor_id: Uuid,
        reason: Option<String>,
    ) -> Result<(), GroupError> {
        // Only owner can ban
        self.permission_service
            .require_owner(group_id, actor_id)
            .await?;

        // Cannot ban owner
        if actor_id == target_id {
            return Err(GroupError::PermissionDenied("Cannot ban yourself".to_string()));
        }

        let mut tx = self.pool.begin().await?;

        // Remove from group if member
        let was_member = sqlx::query!(
            r#"DELETE FROM group_members WHERE group_id = $1 AND user_id = $2"#,
            group_id,
            target_id
        )
        .execute(&mut *tx)
        .await?
        .rows_affected()
            > 0;

        if was_member {
            self.group_service
                .decrement_member_count(&mut tx, group_id)
                .await?;
        }

        // Add to ban list
        sqlx::query!(
            r#"
            INSERT INTO group_bans (group_id, user_id, banned_by, reason)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (group_id, user_id) DO UPDATE SET
                banned_by = $3,
                reason = $4,
                banned_at = NOW()
            "#,
            group_id,
            target_id,
            actor_id,
            reason.as_deref()
        )
        .execute(&mut *tx)
        .await?;

        self.group_service
            .log_audit_event(
                &mut tx,
                group_id,
                Some(actor_id),
                GroupAuditAction::MemberBanned,
                Some(target_id),
                None,
                None,
                reason.as_deref(),
            )
            .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Unban a user from the group
    pub async fn unban_member(
        &self,
        group_id: Uuid,
        target_id: Uuid,
        actor_id: Uuid,
    ) -> Result<(), GroupError> {
        // Only owner can unban
        self.permission_service
            .require_owner(group_id, actor_id)
            .await?;

        // Check if banned
        if !self.permission_service.is_banned(group_id, target_id).await? {
            return Err(GroupError::NotBanned(target_id));
        }

        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"DELETE FROM group_bans WHERE group_id = $1 AND user_id = $2"#,
            group_id,
            target_id
        )
        .execute(&mut *tx)
        .await?;

        self.group_service
            .log_audit_event(
                &mut tx,
                group_id,
                Some(actor_id),
                GroupAuditAction::MemberUnbanned,
                Some(target_id),
                None,
                None,
                None,
            )
            .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Get group members with pagination
    pub async fn get_group_members(
        &self,
        group_id: Uuid,
        requester_id: Uuid,
        page: usize,
        limit: usize,
    ) -> Result<Vec<GroupMemberInfo>, GroupError> {
        // Check if requester is a member
        if !self
            .permission_service
            .is_member(group_id, requester_id)
            .await?
        {
            return Err(GroupError::NotMember(requester_id));
        }

        let limit = limit.min(100) as i64;
        let offset = (page * limit as usize) as i64;

        let members = sqlx::query_as!(
            GroupMemberInfo,
            r#"
            SELECT 
                gm.user_id,
                u.username,
                u.avatar_url,
                gm.role as "role: MemberRole",
                gm.is_muted,
                (u.online_status = 'online') as "online!",
                gm.joined_at
            FROM group_members gm
            INNER JOIN users u ON gm.user_id = u.id
            WHERE gm.group_id = $1
            ORDER BY 
                CASE gm.role 
                    WHEN 'owner' THEN 1 
                    WHEN 'admin' THEN 2 
                    ELSE 3 
                END,
                gm.joined_at ASC
            LIMIT $2 OFFSET $3
            "#,
            group_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(members)
    }

    /// Get permission service reference
    pub fn permission_service(&self) -> &PermissionService {
        &self.permission_service
    }
}
