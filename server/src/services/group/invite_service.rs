use crate::models::{Group, GroupAuditAction, GroupStatus, InviteCode, MemberRole};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use super::{GroupError, GroupService, MemberService, PermissionService};

/// Invite service for managing group invite codes
#[derive(Clone)]
pub struct InviteService {
    pool: PgPool,
    permission_service: PermissionService,
    member_service: MemberService,
    group_service: GroupService,
}

impl InviteService {
    pub fn new(pool: PgPool) -> Self {
        let permission_service = PermissionService::new(pool.clone());
        let member_service = MemberService::new(pool.clone());
        let group_service = GroupService::new(pool.clone());
        Self {
            pool,
            permission_service,
            member_service,
            group_service,
        }
    }

    /// Generate a unique 8-character alphanumeric code
    fn generate_code() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();
        (0..8)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Generate an invite code for a group
    pub async fn generate_invite_code(
        &self,
        group_id: Uuid,
        creator_id: Uuid,
        expires_in_hours: Option<i64>,
        max_uses: Option<i32>,
    ) -> Result<InviteCode, GroupError> {
        // Check permission
        self.permission_service
            .require_admin_or_owner(group_id, creator_id)
            .await?;

        // Check group is active
        let group = self.group_service.get_group(group_id).await?;
        if group.status == GroupStatus::Dissolved {
            return Err(GroupError::GroupDissolved);
        }

        let expires_at = expires_in_hours.map(|hours| Utc::now() + chrono::Duration::hours(hours));

        // Try to generate a unique code (retry up to 5 times)
        let mut attempts = 0;
        let code = loop {
            let code = Self::generate_code();
            let exists = sqlx::query_scalar!(
                r#"SELECT EXISTS(SELECT 1 FROM group_invite_codes WHERE code = $1) as "exists!""#,
                code
            )
            .fetch_one(&self.pool)
            .await?;

            if !exists {
                break code;
            }

            attempts += 1;
            if attempts >= 5 {
                return Err(GroupError::Internal("Failed to generate unique code".to_string()));
            }
        };

        let mut tx = self.pool.begin().await?;

        let invite_code = sqlx::query_as!(
            InviteCode,
            r#"
            INSERT INTO group_invite_codes (code, group_id, creator_id, max_uses, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING code, group_id, creator_id, usage_count, max_uses, expires_at, created_at
            "#,
            code,
            group_id,
            creator_id,
            max_uses,
            expires_at
        )
        .fetch_one(&mut *tx)
        .await?;

        // Log audit event
        self.group_service
            .log_audit_event(
                &mut tx,
                group_id,
                Some(creator_id),
                GroupAuditAction::InviteCodeCreated,
                None,
                None,
                Some(&code),
                None,
            )
            .await?;

        tx.commit().await?;

        Ok(invite_code)
    }

    /// Join a group via invite code
    pub async fn join_via_invite_code(
        &self,
        code: &str,
        user_id: Uuid,
    ) -> Result<Group, GroupError> {
        // Find and validate the invite code
        let invite_code = sqlx::query_as!(
            InviteCode,
            r#"
            SELECT code, group_id, creator_id, usage_count, max_uses, expires_at, created_at
            FROM group_invite_codes
            WHERE code = $1
            "#,
            code.to_uppercase()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(GroupError::InvalidInviteCode)?;

        // Check if expired
        if let Some(expires_at) = invite_code.expires_at {
            if expires_at < Utc::now() {
                return Err(GroupError::InviteCodeExpired);
            }
        }

        // Check if max uses reached
        if let Some(max_uses) = invite_code.max_uses {
            if invite_code.usage_count >= max_uses {
                return Err(GroupError::InviteCodeMaxUsesReached);
            }
        }

        // Add member (this will check for banned, already member, capacity, etc.)
        self.member_service
            .add_member_internal(invite_code.group_id, user_id, MemberRole::Member, None)
            .await?;

        // Increment usage count
        sqlx::query!(
            r#"UPDATE group_invite_codes SET usage_count = usage_count + 1 WHERE code = $1"#,
            code.to_uppercase()
        )
        .execute(&self.pool)
        .await?;

        // Return the group
        self.group_service.get_group(invite_code.group_id).await
    }

    /// Revoke an invite code
    pub async fn revoke_invite_code(&self, code: &str, actor_id: Uuid) -> Result<(), GroupError> {
        // Find the invite code
        let invite_code = sqlx::query_as!(
            InviteCode,
            r#"
            SELECT code, group_id, creator_id, usage_count, max_uses, expires_at, created_at
            FROM group_invite_codes
            WHERE code = $1
            "#,
            code.to_uppercase()
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(GroupError::InvalidInviteCode)?;

        // Check permission
        self.permission_service
            .require_admin_or_owner(invite_code.group_id, actor_id)
            .await?;

        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"DELETE FROM group_invite_codes WHERE code = $1"#,
            code.to_uppercase()
        )
        .execute(&mut *tx)
        .await?;

        // Log audit event
        self.group_service
            .log_audit_event(
                &mut tx,
                invite_code.group_id,
                Some(actor_id),
                GroupAuditAction::InviteCodeRevoked,
                None,
                Some(code),
                None,
                None,
            )
            .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Get all invite codes for a group
    pub async fn get_group_invite_codes(
        &self,
        group_id: Uuid,
        actor_id: Uuid,
    ) -> Result<Vec<InviteCode>, GroupError> {
        // Check permission
        self.permission_service
            .require_admin_or_owner(group_id, actor_id)
            .await?;

        let codes = sqlx::query_as!(
            InviteCode,
            r#"
            SELECT code, group_id, creator_id, usage_count, max_uses, expires_at, created_at
            FROM group_invite_codes
            WHERE group_id = $1
            ORDER BY created_at DESC
            "#,
            group_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(codes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_code() {
        let code = InviteService::generate_code();
        assert_eq!(code.len(), 8);
        assert!(code.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_generate_code_uniqueness() {
        // Generate multiple codes and check they're different
        let codes: Vec<String> = (0..10).map(|_| InviteService::generate_code()).collect();
        let unique: std::collections::HashSet<_> = codes.iter().collect();
        // With 36^8 possibilities, collisions are extremely unlikely
        assert!(unique.len() >= 9); // Allow for one unlikely collision
    }
}
