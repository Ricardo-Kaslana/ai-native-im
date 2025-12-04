-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    avatar_url VARCHAR(500),
    bio TEXT,
    online_status VARCHAR(20) NOT NULL DEFAULT 'offline' CHECK (online_status IN ('online', 'offline', 'away')),
    last_seen_at TIMESTAMPTZ,
    account_status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (account_status IN ('active', 'suspended', 'deleted')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Create indexes for performance
CREATE INDEX idx_users_username ON users(username) WHERE account_status != 'deleted';
CREATE INDEX idx_users_email ON users(email) WHERE account_status != 'deleted';
CREATE INDEX idx_users_online_status ON users(online_status) WHERE account_status = 'active';
CREATE INDEX idx_users_created_at ON users(created_at);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for users table
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Add comments for documentation
COMMENT ON TABLE users IS 'User accounts and profiles';
COMMENT ON COLUMN users.id IS 'Unique user identifier (UUID v4)';
COMMENT ON COLUMN users.username IS 'Unique username (3-50 characters, alphanumeric and underscore)';
COMMENT ON COLUMN users.email IS 'User email address';
COMMENT ON COLUMN users.password_hash IS 'bcrypt hashed password (cost factor 12)';
COMMENT ON COLUMN users.avatar_url IS 'URL to user avatar image';
COMMENT ON COLUMN users.bio IS 'User biography (max 500 characters)';
COMMENT ON COLUMN users.online_status IS 'Current online status: online, offline, away';
COMMENT ON COLUMN users.last_seen_at IS 'Last activity timestamp';
COMMENT ON COLUMN users.account_status IS 'Account status: active, suspended, deleted';
COMMENT ON COLUMN users.created_at IS 'Account creation timestamp';
COMMENT ON COLUMN users.updated_at IS 'Last update timestamp';
COMMENT ON COLUMN users.deleted_at IS 'Soft delete timestamp';
