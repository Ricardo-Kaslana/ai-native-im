-- Create media_files table for storing media file information
CREATE TABLE media_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    uploader_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    file_hash VARCHAR(64) UNIQUE NOT NULL,
    media_type VARCHAR(20) NOT NULL,
    mime_type VARCHAR(50) NOT NULL,
    original_url TEXT NOT NULL,
    processed_url TEXT,
    thumbnail_url TEXT,
    width INTEGER,
    height INTEGER,
    duration REAL,
    file_size BIGINT NOT NULL,
    ref_count INTEGER DEFAULT 1 NOT NULL,
    status VARCHAR(20) DEFAULT 'processing' NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    
    CONSTRAINT media_type_check CHECK (media_type IN ('image', 'video')),
    CONSTRAINT status_check CHECK (status IN ('processing', 'completed', 'failed')),
    CONSTRAINT ref_count_positive CHECK (ref_count >= 0),
    CONSTRAINT file_size_positive CHECK (file_size > 0)
);

-- Create indexes for performance
CREATE INDEX idx_media_hash ON media_files(file_hash);
CREATE INDEX idx_media_uploader ON media_files(uploader_id);
CREATE INDEX idx_media_status ON media_files(status);
CREATE INDEX idx_media_created_at ON media_files(created_at);
CREATE INDEX idx_media_type ON media_files(media_type);

-- Create updated_at trigger
CREATE OR REPLACE FUNCTION update_media_files_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER media_files_updated_at
    BEFORE UPDATE ON media_files
    FOR EACH ROW
    EXECUTE FUNCTION update_media_files_updated_at();

-- Add comments for documentation
COMMENT ON TABLE media_files IS 'Stores information about uploaded media files (images and videos)';
COMMENT ON COLUMN media_files.file_hash IS 'SHA-256 hash of file content for deduplication';
COMMENT ON COLUMN media_files.ref_count IS 'Number of references to this file (for deduplication)';
COMMENT ON COLUMN media_files.status IS 'Processing status: processing, completed, or failed';
