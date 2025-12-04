-- Add lifecycle management fields to media_files table
ALTER TABLE media_files
ADD COLUMN IF NOT EXISTS last_accessed_at TIMESTAMP,
ADD COLUMN IF NOT EXISTS lifecycle_status VARCHAR(30) DEFAULT 'active',
ADD COLUMN IF NOT EXISTS marked_for_deletion_at TIMESTAMP;

-- Add constraint for lifecycle_status
ALTER TABLE media_files
ADD CONSTRAINT check_lifecycle_status 
CHECK (lifecycle_status IN ('active', 'marked_for_deletion', 'cold_storage', 'deleted'));

-- Add index for lifecycle queries
CREATE INDEX IF NOT EXISTS idx_media_lifecycle_status ON media_files(lifecycle_status);
CREATE INDEX IF NOT EXISTS idx_media_last_accessed ON media_files(last_accessed_at);
CREATE INDEX IF NOT EXISTS idx_media_marked_deletion ON media_files(marked_for_deletion_at);
