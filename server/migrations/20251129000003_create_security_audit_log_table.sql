-- Create security_audit_log table
CREATE TABLE IF NOT EXISTS security_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    event_type VARCHAR(50) NOT NULL CHECK (event_type IN (
        'login_success',
        'login_failed',
        'password_changed',
        'password_reset_requested',
        'password_reset_completed',
        'account_locked',
        'account_unlocked',
        'account_suspended',
        'account_reactivated',
        'account_deleted',
        'token_refreshed',
        'logout'
    )),
    ip_address INET,
    user_agent TEXT,
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance and querying
CREATE INDEX idx_security_audit_log_user_id ON security_audit_log(user_id);
CREATE INDEX idx_security_audit_log_event_type ON security_audit_log(event_type);
CREATE INDEX idx_security_audit_log_created_at ON security_audit_log(created_at DESC);
CREATE INDEX idx_security_audit_log_ip_address ON security_audit_log(ip_address);

-- Create composite index for common queries
CREATE INDEX idx_security_audit_log_user_event ON security_audit_log(user_id, event_type, created_at DESC);

-- Add comments for documentation
COMMENT ON TABLE security_audit_log IS 'Security audit log for tracking authentication and account events';
COMMENT ON COLUMN security_audit_log.id IS 'Unique log entry identifier';
COMMENT ON COLUMN security_audit_log.user_id IS 'User associated with the event (NULL if user not found)';
COMMENT ON COLUMN security_audit_log.event_type IS 'Type of security event';
COMMENT ON COLUMN security_audit_log.ip_address IS 'IP address of the request';
COMMENT ON COLUMN security_audit_log.user_agent IS 'User agent string from the request';
COMMENT ON COLUMN security_audit_log.details IS 'Additional event details in JSON format';
COMMENT ON COLUMN security_audit_log.created_at IS 'Event timestamp';
