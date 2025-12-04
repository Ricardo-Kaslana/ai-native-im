-- Add username column to security_audit_log table
ALTER TABLE security_audit_log 
ADD COLUMN IF NOT EXISTS username VARCHAR(50);

-- Add index for username
CREATE INDEX IF NOT EXISTS idx_security_audit_log_username ON security_audit_log(username);

-- Add comment
COMMENT ON COLUMN security_audit_log.username IS 'Username associated with the event';
