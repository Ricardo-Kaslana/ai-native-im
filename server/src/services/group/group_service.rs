use crate::models::{
    CreateGroup, Group, GroupAuditAction, GroupDetails, GroupSearchResult, GroupStatus,
    MemberRole, UpdateGroupSettings,
};
use sqlx::PgPool;
use uuid::Uuid;

use super::{GroupError, PermissionService};

/// Default maximum group capacity
pub const DEFAULT_MAX_CAPACITY: i32 = 500;

/// Group service for managing group lifecycle
#[derive(Clone)]
pub struct GroupService {
    pool: PgPool,
    permission_service: PermissionService,
}

impl GroupService {
    pub fn new(pool: PgPool) -> Self {
        let permission_service = PermissionService::new(pool.clone());
        Self {
            pool,
            permission_service,
        }
    }

    /// Validate group name (1-50 characters)
    fn validate_name(name: &str) -> Result<(), GroupError> {
        let len = name.chars().count();
        if len < 1 || len > 50 {
            return Err(GroupError::InvalidGroupName);
        }
        Ok(())
    }

    /// Validate group description (max 500 characters)
    fn validate_description(description: &Option<String>) -> Result<(), GroupError> {
        if let Some(desc) = description {
            if desc.chars().count() > 500 {
                return Err(GroupError::InvalidDescription);
            }
        }
        Ok(())
    }

    /// Create a new group
    pub async fn create_group(
        &self,
        owner_id: Uuid,
        request: CreateGroup,
    ) -> Result<Group, GroupError> {
        // Validate inputs
        Self::validate_name(&request.name)?;
        Self::validate_description(&request.description)?;

        let is_public = request.is_public.unwrap_or(false);

        // Create group in transaction
        let mut tx = self.pool.begin().await?;

        // Insert group
        let group = sqlx::query_as!(
            Group,
            r#"
            INSERT INTO groups (name, description, avatar, owner_id, is_public, member_count, max_capacity, status)
            VALUES ($1, $2, $3, $4, $5, 1, $6, 'active')
            RETURNING id, name, description, avatar, owner_id, is_public, member_count, max_capacity,
                      status as "status: GroupStatus", created_at, updated_at
            "#,
            request.name,
            request.description,
            request.avatar,
            owner_id,
            is_public,
            DEFAULT_MAX_CAPACITY
        )
        .fetch_one(&mut *tx)
        .await?;

        // Add owner as member
        sqlx::query!(
            r#"
            INSERT INTO group_members (group_id, user_id, role)
            VALUES ($1, $2, 'owner')
            "#,
            group.id,
            owner_id
        )
        .execute(&mut *tx)
        .await?;

        // Log audit event
        self.log_audit_event(
            &mut tx,
            group.id,
            Some(owner_id),
            GroupAuditAction::GroupCreated,
            None,
            None,
            Some(&group.name),
            None,
        )
        .await?;

        tx.commit().await?;

        Ok(group)
    }

    /// Get group by ID
    pub async fn get_group(&self, group_id: Uuid) -> Result<Group, GroupError> {
        let group = sqlx::query_as!(
            Group,
            r#"
            SELECT id, name, description, avatar, owner_id, is_public, member_count, max_capacity,
                   status as "status: GroupStatus", created_at, updated_at
            FROM groups
            WHERE id = $1
            "#,
            group_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(GroupError::GroupNotFound(group_id))?;

        Ok(group)
    }


    /// Get group details with requester's role
    pub async fn get_group_details(
        &self,
        group_id: Uuid,
        requester_id: Option<Uuid>,
    ) -> Result<GroupDetails, GroupError> {
        let group = self.get_group(group_id).await?;

        // Get owner username
        let owner_username = sqlx::query_scalar!(
            r#"SELECT username FROM users WHERE id = $1"#,
            group.owner_id
        )
        .fetch_optional(&self.pool)
        .await?;

        // Get requester's role and mute status if they are a member
        let (requester_role, requester_is_muted) = if let Some(user_id) = requester_id {
            let member = sqlx::query!(
                r#"
                SELECT role as "role: MemberRole", is_muted
                FROM group_members
                WHERE group_id = $1 AND user_id = $2
                "#,
                group_id,
                user_id
            )
            .fetch_optional(&self.pool)
            .await?;

            match member {
                Some(m) => (Some(m.role), Some(m.is_muted)),
                None => (None, None),
            }
        } else {
            (None, None)
        };

        // If group is private and requester is not a member, return limited info
        if !group.is_public && requester_role.is_none() {
            return Ok(GroupDetails {
                id: group.id,
                name: group.name,
                description: None,
                avatar: group.avatar,
                owner_id: group.owner_id,
                owner_username: None,
                is_public: group.is_public,
                member_count: group.member_count,
                max_capacity: group.max_capacity,
                status: group.status,
                created_at: group.created_at,
                requester_role: None,
                requester_is_muted: None,
            });
        }

        Ok(GroupDetails {
            id: group.id,
            name: group.name,
            description: group.description,
            avatar: group.avatar,
            owner_id: group.owner_id,
            owner_username,
            is_public: group.is_public,
            member_count: group.member_count,
            max_capacity: group.max_capacity,
            status: group.status,
            created_at: group.created_at,
            requester_role,
            requester_is_muted,
        })
    }

    /// Update group settings
    pub async fn update_group_settings(
        &self,
        group_id: Uuid,
        actor_id: Uuid,
        updates: UpdateGroupSettings,
    ) -> Result<Group, GroupError> {
        // Check permission
        self.permission_service
            .require_admin_or_owner(group_id, actor_id)
            .await?;

        // Validate inputs
        if let Some(ref name) = updates.name {
            Self::validate_name(name)?;
        }
        Self::validate_description(&updates.description)?;

        let mut tx = self.pool.begin().await?;

        // Get current group for audit log
        let current = self.get_group(group_id).await?;
        if current.status == GroupStatus::Dissolved {
            return Err(GroupError::GroupDissolved);
        }

        // Build update query dynamically
        let group = sqlx::query_as!(
            Group,
            r#"
            UPDATE groups
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                avatar = COALESCE($4, avatar),
                is_public = COALESCE($5, is_public),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, description, avatar, owner_id, is_public, member_count, max_capacity,
                      status as "status: GroupStatus", created_at, updated_at
            "#,
            group_id,
            updates.name,
            updates.description,
            updates.avatar,
            updates.is_public
        )
        .fetch_one(&mut *tx)
        .await?;

        // Log audit event
        self.log_audit_event(
            &mut tx,
            group_id,
            Some(actor_id),
            GroupAuditAction::GroupUpdated,
            None,
            Some(&serde_json::to_string(&current).unwrap_or_default()),
            Some(&serde_json::to_string(&group).unwrap_or_default()),
            None,
        )
        .await?;

        tx.commit().await?;

        Ok(group)
    }

    /// Dissolve a group (only owner can do this)
    pub async fn dissolve_group(&self, group_id: Uuid, actor_id: Uuid) -> Result<(), GroupError> {
        // Only owner can dissolve
        self.permission_service
            .require_owner(group_id, actor_id)
            .await?;

        let mut tx = self.pool.begin().await?;

        // Update group status
        let result = sqlx::query!(
            r#"
            UPDATE groups
            SET status = 'dissolved', updated_at = NOW()
            WHERE id = $1 AND status = 'active'
            "#,
            group_id
        )
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(GroupError::GroupNotFound(group_id));
        }

        // Log audit event
        self.log_audit_event(
            &mut tx,
            group_id,
            Some(actor_id),
            GroupAuditAction::GroupDissolved,
            None,
            None,
            None,
            None,
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }


    /// Search public groups
    pub async fn search_groups(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<GroupSearchResult>, GroupError> {
        if query.chars().count() < 2 {
            return Err(GroupError::SearchQueryTooShort);
        }

        let limit = limit.min(20) as i64; // Cap at 20 results

        // Use full-text search for better performance with GIN index
        // Convert query to tsquery format (words separated by &)
        let tsquery = query
            .split_whitespace()
            .map(|word| word.to_lowercase())
            .collect::<Vec<_>>()
            .join(" & ");

        // Try full-text search first
        let groups = match sqlx::query_as!(
            GroupSearchResult,
            r#"
            SELECT id, name, description, avatar, member_count, is_public
            FROM groups
            WHERE status = 'active'
              AND is_public = TRUE
              AND to_tsvector('english', name || ' ' || COALESCE(description, '')) @@ to_tsquery('english', $1)
            ORDER BY 
                ts_rank(to_tsvector('english', name || ' ' || COALESCE(description, '')), to_tsquery('english', $1)) DESC,
                member_count DESC
            LIMIT $2
            "#,
            tsquery,
            limit
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(groups) => groups,
            Err(_) => {
                // Fallback to LIKE search if full-text search fails (e.g., invalid query)
                let search_pattern = format!("%{}%", query.to_lowercase());
                sqlx::query_as!(
                    GroupSearchResult,
                    r#"
                    SELECT id, name, description, avatar, member_count, is_public
                    FROM groups
                    WHERE status = 'active'
                      AND is_public = TRUE
                      AND (LOWER(name) LIKE $1 OR LOWER(COALESCE(description, '')) LIKE $1)
                    ORDER BY member_count DESC
                    LIMIT $2
                    "#,
                    search_pattern,
                    limit
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(groups)
    }

    /// Get groups for a user
    pub async fn get_user_groups(&self, user_id: Uuid) -> Result<Vec<Group>, GroupError> {
        let groups = sqlx::query_as!(
            Group,
            r#"
            SELECT g.id, g.name, g.description, g.avatar, g.owner_id, g.is_public,
                   g.member_count, g.max_capacity, g.status as "status: GroupStatus",
                   g.created_at, g.updated_at
            FROM groups g
            INNER JOIN group_members gm ON g.id = gm.group_id
            WHERE gm.user_id = $1 AND g.status = 'active'
            ORDER BY g.updated_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(groups)
    }

    /// Increment member count
    pub(crate) async fn increment_member_count(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        group_id: Uuid,
    ) -> Result<(), GroupError> {
        sqlx::query!(
            r#"UPDATE groups SET member_count = member_count + 1 WHERE id = $1"#,
            group_id
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// Decrement member count
    pub(crate) async fn decrement_member_count(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        group_id: Uuid,
    ) -> Result<(), GroupError> {
        sqlx::query!(
            r#"UPDATE groups SET member_count = member_count - 1 WHERE id = $1"#,
            group_id
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// Log audit event
    pub(crate) async fn log_audit_event(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        group_id: Uuid,
        actor_id: Option<Uuid>,
        action: GroupAuditAction,
        target_user_id: Option<Uuid>,
        old_value: Option<&str>,
        new_value: Option<&str>,
        reason: Option<&str>,
    ) -> Result<(), GroupError> {
        sqlx::query!(
            r#"
            INSERT INTO group_audit_log (group_id, actor_id, action_type, target_user_id, old_value, new_value, reason)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            group_id,
            actor_id,
            action.to_string(),
            target_user_id,
            old_value,
            new_value,
            reason
        )
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    /// Get permission service reference
    pub fn permission_service(&self) -> &PermissionService {
        &self.permission_service
    }

    /// Get pool reference
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name() {
        // Valid names
        assert!(GroupService::validate_name("Test Group").is_ok());
        assert!(GroupService::validate_name("A").is_ok());
        assert!(GroupService::validate_name(&"A".repeat(50)).is_ok());

        // Invalid names
        assert!(GroupService::validate_name("").is_err());
        assert!(GroupService::validate_name(&"A".repeat(51)).is_err());
    }

    #[test]
    fn test_validate_description() {
        // Valid descriptions
        assert!(GroupService::validate_description(&None).is_ok());
        assert!(GroupService::validate_description(&Some("Test".to_string())).is_ok());
        assert!(GroupService::validate_description(&Some("A".repeat(500))).is_ok());

        // Invalid descriptions
        assert!(GroupService::validate_description(&Some("A".repeat(501))).is_err());
    }
}
