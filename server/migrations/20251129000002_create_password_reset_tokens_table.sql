-- Create password_reset_tokens table
CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);
CREATE INDEX idx_password_reset_tokens_token ON password_reset_tokens(token) WHERE used_at IS NULL;
CREATE INDEX idx_password_reset_tokens_expires_at ON password_reset_tokens(expires_at) WHERE used_at IS NULL;

-- Add comments for documentation
COMMENT ON TABLE password_reset_tokens IS 'Password reset tokens for secure password recovery';
COMMENT ON COLUMN password_reset_tokens.id IS 'Unique token identifier';
COMMENT ON COLUMN password_reset_tokens.user_id IS 'User who requested password reset';
COMMENT ON COLUMN password_reset_tokens.token IS 'Secure random token (URL-safe)';
COMMENT ON COLUMN password_reset_tokens.expires_at IS 'Token expiration timestamp (1 hour from creation)';
COMMENT ON COLUMN password_reset_tokens.used_at IS 'Timestamp when token was used (NULL if unused)';
COMMENT ON COLUMN password_reset_tokens.created_at IS 'Token creation timestamp';
