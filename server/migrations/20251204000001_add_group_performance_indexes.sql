-- Add performance indexes for group chat system

-- Index for member list queries (ordered by role and join date)
CREATE INDEX IF NOT EXISTS idx_group_members_list 
ON group_members(group_id, role, joined_at);

-- Index for member lookup by user
CREATE INDEX IF NOT EXISTS idx_group_members_user_lookup 
ON group_members(user_id, group_id);

-- Index for ban checks
CREATE INDEX IF NOT EXISTS idx_group_bans_lookup 
ON group_bans(group_id, user_id) 
WHERE unbanned_at IS NULL;

-- Index for active groups search
CREATE INDEX IF NOT EXISTS idx_groups_search 
ON groups(status, is_public) 
WHERE status = 'active' AND is_public = TRUE;

-- GIN index for full-text search on group name and description
CREATE INDEX IF NOT EXISTS idx_groups_fulltext_search 
ON groups USING gin(to_tsvector('english', name || ' ' || COALESCE(description, '')))
WHERE status = 'active' AND is_public = TRUE;

-- Index for user's groups query
CREATE INDEX IF NOT EXISTS idx_user_groups 
ON group_members(user_id, group_id);

-- Index for audit log queries
CREATE INDEX IF NOT EXISTS idx_group_audit_log_lookup 
ON group_audit_log(group_id, created_at DESC);

-- Index for invite code lookup
CREATE INDEX IF NOT EXISTS idx_group_invite_codes_lookup 
ON group_invite_codes(code) 
WHERE revoked_at IS NULL;

-- Analyze tables to update statistics
ANALYZE groups;
ANALYZE group_members;
ANALYZE group_bans;
ANALYZE group_invite_codes;
ANALYZE group_audit_log;
