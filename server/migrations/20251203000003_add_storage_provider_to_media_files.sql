-- Add storage_provider column to media_files table for multi-cloud support
ALTER TABLE media_files 
ADD COLUMN storage_provider VARCHAR(20) NOT NULL DEFAULT 'aws_s3';

-- Add constraint to validate storage provider values
ALTER TABLE media_files
ADD CONSTRAINT storage_provider_check CHECK (
    storage_provider IN ('aws_s3', 'minio', 'tencent_cos', 'aliyun_oss', 'cloudflare_r2')
);

-- Create index for storage provider queries
CREATE INDEX idx_media_storage_provider ON media_files(storage_provider);

-- Add comment for documentation
COMMENT ON COLUMN media_files.storage_provider IS 'Cloud storage provider used for this file: aws_s3, minio, tencent_cos, aliyun_oss, or cloudflare_r2';
