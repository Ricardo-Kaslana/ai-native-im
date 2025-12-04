-- Create media audit log table for tracking media operations
CREATE TABLE IF NOT EXISTS media_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(50) NOT NULL,
    media_id UUID,
    user_id UUID,
    file_size BIGINT,
    duration_ms BIGINT,
    details TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Add indexes for common queries
CREATE INDEX IF NOT EXISTS idx_media_audit_media_id ON media_audit_log(media_id);
CREATE INDEX IF NOT EXISTS idx_media_audit_user_id ON media_audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_media_audit_event_type ON media_audit_log(event_type);
CREATE INDEX IF NOT EXISTS idx_media_audit_created_at ON media_audit_log(created_at);
