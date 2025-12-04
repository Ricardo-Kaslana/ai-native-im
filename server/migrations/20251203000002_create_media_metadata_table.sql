-- Create media_metadata table for storing detailed media metadata
CREATE TABLE media_metadata (
    media_id UUID PRIMARY KEY REFERENCES media_files(id) ON DELETE CASCADE,
    format VARCHAR(20),
    codec VARCHAR(50),
    bitrate INTEGER,
    framerate REAL,
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Create index for JSONB queries
CREATE INDEX idx_media_metadata_jsonb ON media_metadata USING GIN (metadata);

-- Add comments for documentation
COMMENT ON TABLE media_metadata IS 'Stores detailed metadata for media files';
COMMENT ON COLUMN media_metadata.format IS 'Media format (e.g., mp4, webp, jpeg)';
COMMENT ON COLUMN media_metadata.codec IS 'Video/audio codec (e.g., h264, aac)';
COMMENT ON COLUMN media_metadata.bitrate IS 'Bitrate in bits per second';
COMMENT ON COLUMN media_metadata.framerate IS 'Video framerate (fps)';
COMMENT ON COLUMN media_metadata.metadata IS 'Additional metadata in JSON format';
