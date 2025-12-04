-- Create groups table
CREATE TABLE IF NOT EXISTS groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(50) NOT NULL,
    description VARCHAR(500),
    avatar TEXT,
    owner_id UUID NOT NULL REFERENCES users(id),
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    member_count INTEGER NOT NULL DEFAULT 1,
    max_capacity INTEGER NOT NULL DEFAULT 500,
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'dissolved')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT groups_name_length CHECK (char_length(name) >= 1 AND char_length(name) <= 50),
    CONSTRAINT groups_description_length CHECK (description IS NULL OR char_length(description) <= 500)
);

-- Create indexes for groups
CREATE INDEX idx_groups_owner ON groups(owner_id);
CREATE INDEX idx_groups_status ON groups(status) WHERE status = 'active';
CREATE INDEX idx_groups_public ON groups(is_public) WHERE is_public = TRUE AND status = 'active';
CREATE INDEX idx_groups_name_search ON groups USING gin(to_tsvector('simple', name)) WHERE status = 'active';

-- Create trigger for groups updated_at
CREATE TRIGGER update_groups_updated_at
    BEFORE UPDATE ON groups
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Create group_members table
CREATE TABLE IF NOT EXISTS group_members (
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL DEFAULT 'member' CHECK (role IN ('owner', 'admin', 'member')),
    is_muted BOOLEAN NOT NULL DEFAULT FALSE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (group_id, user_id)
);

-- Create indexes for group_members
CREATE INDEX idx_group_members_user ON group_members(user_id);
CREATE INDEX idx_group_members_role ON group_members(group_id, role);

-- Create group_bans table
CREATE TABLE IF NOT EXISTS group_bans (
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    banned_by UUID NOT NULL REFERENCES users(id),
    reason TEXT,
    banned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (group_id, user_id)
);

-- Create index for group_bans
CREATE INDEX idx_group_bans_user ON group_bans(user_id);

-- Create group_invite_codes table
CREATE TABLE IF NOT EXISTS group_invite_codes (
    code VARCHAR(8) PRIMARY KEY,
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    creator_id UUID NOT NULL REFERENCES users(id),
    usage_count INTEGER NOT NULL DEFAULT 0,
    max_uses INTEGER,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for group_invite_codes
CREATE INDEX idx_invite_codes_group ON group_invite_codes(group_id);
CREATE INDEX idx_invite_codes_expiry ON group_invite_codes(expires_at) WHERE expires_at IS NOT NULL;

-- Create group_audit_log table
CREATE TABLE IF NOT EXISTS group_audit_log (
    id BIGSERIAL PRIMARY KEY,
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    actor_id UUID REFERENCES users(id),
    action_type VARCHAR(50) NOT NULL,
    target_user_id UUID REFERENCES users(id),
    old_value TEXT,
    new_value TEXT,
    reason TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for group_audit_log
CREATE INDEX idx_group_audit_group ON group_audit_log(group_id, created_at DESC);
CREATE INDEX idx_group_audit_actor ON group_audit_log(actor_id, created_at DESC);
CREATE INDEX idx_group_audit_action ON group_audit_log(action_type, created_at DESC);

-- Add comments
COMMENT ON TABLE groups IS 'Chat groups';
COMMENT ON TABLE group_members IS 'Group membership with roles';
COMMENT ON TABLE group_bans IS 'Banned users per group';
COMMENT ON TABLE group_invite_codes IS 'Invite codes for joining groups';
COMMENT ON TABLE group_audit_log IS 'Audit log for group management actions';
