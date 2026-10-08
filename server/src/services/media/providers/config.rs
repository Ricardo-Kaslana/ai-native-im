use crate::services::media::error::{MediaError, Result};
use crate::services::media::storage_manager::StorageProviderType;

/// Configuration for a single storage provider
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub provider_type: StorageProviderType,
    pub region: Option<String>,
    pub endpoint: Option<String>,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub cdn_domain: Option<String>,
}

impl StorageConfig {
    /// Validate that all required fields are present for the provider type
    pub fn validate(&self) -> Result<()> {
        match self.provider_type {
            StorageProviderType::TencentCOS => {
                if self.region.is_none() {
                    return Err(MediaError::StorageError(
                        "Tencent COS requires region".to_string(),
                    ));
                }
            }
            StorageProviderType::AliyunOSS => {
                if self.region.is_none() {
                    return Err(MediaError::StorageError(
                        "Aliyun OSS requires region".to_string(),
                    ));
                }
            }
        }

        if self.bucket.is_empty() {
            return Err(MediaError::StorageError(
                "Bucket name is required".to_string(),
            ));
        }

        if self.access_key.is_empty() {
            return Err(MediaError::StorageError(
                "Access key is required".to_string(),
            ));
        }

        if self.secret_key.is_empty() {
            return Err(MediaError::StorageError(
                "Secret key is required".to_string(),
            ));
        }

        Ok(())
    }

    /// Load configuration from environment variables with a prefix
    pub fn from_env(prefix: &str) -> Result<Self> {
        let provider_type_str = std::env::var(format!("{}_PROVIDER", prefix))
            .map_err(|_| {
                MediaError::StorageError(format!("{}_PROVIDER not set", prefix))
            })?;

        let provider_type = StorageProviderType::from_str(&provider_type_str)?;

        let config = Self {
            provider_type,
            region: std::env::var(format!("{}_REGION", prefix)).ok(),
            endpoint: std::env::var(format!("{}_ENDPOINT", prefix)).ok(),
            bucket: std::env::var(format!("{}_BUCKET", prefix)).map_err(|_| {
                MediaError::StorageError(format!("{}_BUCKET not set", prefix))
            })?,
            access_key: std::env::var(format!("{}_ACCESS_KEY", prefix)).map_err(|_| {
                MediaError::StorageError(format!("{}_ACCESS_KEY not set", prefix))
            })?,
            secret_key: std::env::var(format!("{}_SECRET_KEY", prefix)).map_err(|_| {
                MediaError::StorageError(format!("{}_SECRET_KEY not set", prefix))
            })?,
            cdn_domain: std::env::var(format!("{}_CDN_DOMAIN", prefix)).ok(),
        };

        config.validate()?;
        Ok(config)
    }
}

/// Configuration for media storage (separate configs for images and videos)
#[derive(Debug, Clone)]
pub struct MediaStorageConfig {
    pub image_storage: StorageConfig,
    pub video_storage: StorageConfig,
}

impl MediaStorageConfig {
    /// Load configuration from environment variables
    /// Uses IMAGE_STORAGE_* and VIDEO_STORAGE_* prefixes
    pub fn from_env() -> Result<Self> {
        let image_storage = StorageConfig::from_env("IMAGE_STORAGE")?;
        let video_storage = StorageConfig::from_env("VIDEO_STORAGE")?;

        Ok(Self {
            image_storage,
            video_storage,
        })
    }

    /// Create a configuration with the same provider for both images and videos
    pub fn single_provider(config: StorageConfig) -> Self {
        Self {
            image_storage: config.clone(),
            video_storage: config,
        }
    }
}
