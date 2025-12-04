use std::env;

/// Media processing system configuration
#[derive(Debug, Clone)]
pub struct MediaConfig {
    /// S3 bucket name
    pub s3_bucket: String,
    
    /// S3 region
    pub s3_region: String,
    
    /// CDN domain (e.g., CloudFront)
    pub cdn_domain: Option<String>,
    
    /// Maximum image size in bytes (default: 10MB)
    pub max_image_size: u64,
    
    /// Maximum video size in bytes (default: 100MB)
    pub max_video_size: u64,
    
    /// Image upload URL expiry in seconds (default: 15 minutes)
    pub image_upload_expiry: u64,
    
    /// Video upload URL expiry in seconds (default: 30 minutes)
    pub video_upload_expiry: u64,
    
    /// Temporary directory for processing
    pub temp_dir: String,
}

impl Default for MediaConfig {
    fn default() -> Self {
        Self {
            s3_bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "media-bucket".to_string()),
            s3_region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            cdn_domain: env::var("CDN_DOMAIN").ok(),
            max_image_size: 10 * 1024 * 1024,  // 10MB
            max_video_size: 100 * 1024 * 1024, // 100MB
            image_upload_expiry: 15 * 60,      // 15 minutes
            video_upload_expiry: 30 * 60,      // 30 minutes
            temp_dir: env::var("TEMP_DIR").unwrap_or_else(|_| "/tmp/media".to_string()),
        }
    }
}

impl MediaConfig {
    pub fn from_env() -> Self {
        Self::default()
    }
}
