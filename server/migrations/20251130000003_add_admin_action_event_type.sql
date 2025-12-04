-- Add admin_action event type to security_audit_log
ALTER TABLE security_audit_log DROP CONSTRAINT IF EXISTS security_audit_log_event_type_check;

ALTER TABLE security_audit_log ADD CONSTRAINT security_audit_log_event_type_check 
CHECK (event_type IN (
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
    'logout',
    'admin_action'
));

-- Add comment
COMMENT ON COLUMN security_audit_log.event_type IS 'Type of security event including admin actions';
