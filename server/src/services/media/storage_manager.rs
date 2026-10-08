use super::error::{MediaError, Result};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

/// Unified storage provider trait for all cloud storage services
#[async_trait]
pub trait StorageProvider: Send + Sync {
    /// Upload a file to storage
    /// Returns the public URL of the uploaded file
    async fn upload_file(
        &self,
        local_path: &str,
        key: &str,
        content_type: &str,
    ) -> Result<String>;

    /// Generate a presigned URL for direct upload
    /// Returns a temporary URL that allows uploading without credentials
    async fn generate_presigned_url(
        &self,
        key: &str,
        expires_in: Duration,
        content_type: &str,
    ) -> Result<String>;

    /// Get the public URL for accessing a file
    fn get_public_url(&self, key: &str) -> String;

    /// Get the CDN URL if available, otherwise returns None
    fn get_cdn_url(&self, key: &str) -> Option<String>;

    /// Delete a file from storage
    async fn delete_file(&self, key: &str) -> Result<()>;

    /// Get the provider type name
    fn provider_type(&self) -> &str;
}

/// Storage provider type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageProviderType {
    TencentCOS,
    AliyunOSS,
}

impl StorageProviderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TencentCOS => "tencent_cos",
            Self::AliyunOSS => "aliyun_oss",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "tencent_cos" | "cos" => Ok(Self::TencentCOS),
            "aliyun_oss" | "oss" => Ok(Self::AliyunOSS),
            _ => Err(MediaError::StorageError(format!(
                "Unsupported storage provider: {}",
                s
            ))),
        }
    }
}

/// Storage Manager - manages multiple storage providers
pub struct StorageManager {
    image_provider: Arc<dyn StorageProvider>,
    video_provider: Arc<dyn StorageProvider>,
}

impl StorageManager {
    /// Create a new StorageManager with separate providers for images and videos
    pub fn new(
        image_provider: Arc<dyn StorageProvider>,
        video_provider: Arc<dyn StorageProvider>,
    ) -> Self {
        Self {
            image_provider,
            video_provider,
        }
    }

    /// Create a StorageManager with the same provider for both images and videos
    pub fn new_single_provider(provider: Arc<dyn StorageProvider>) -> Self {
        Self {
            image_provider: provider.clone(),
            video_provider: provider,
        }
    }

    /// Get the provider for images
    pub fn image_provider(&self) -> &Arc<dyn StorageProvider> {
        &self.image_provider
    }

    /// Get the provider for videos
    pub fn video_provider(&self) -> &Arc<dyn StorageProvider> {
        &self.video_provider
    }

    /// Upload an image file
    pub async fn upload_image(
        &self,
        local_path: &str,
        key: &str,
        content_type: &str,
    ) -> Result<String> {
        self.image_provider
            .upload_file(local_path, key, content_type)
            .await
    }

    /// Upload a video file
    pub async fn upload_video(
        &self,
        local_path: &str,
        key: &str,
        content_type: &str,
    ) -> Result<String> {
        self.video_provider
            .upload_file(local_path, key, content_type)
            .await
    }

    /// Generate presigned URL for image upload
    pub async fn generate_image_upload_url(
        &self,
        key: &str,
        expires_in: Duration,
        content_type: &str,
    ) -> Result<String> {
        self.image_provider
            .generate_presigned_url(key, expires_in, content_type)
            .await
    }

    /// Generate presigned URL for video upload
    pub async fn generate_video_upload_url(
        &self,
        key: &str,
        expires_in: Duration,
        content_type: &str,
    ) -> Result<String> {
        self.video_provider
            .generate_presigned_url(key, expires_in, content_type)
            .await
    }

    /// Get public URL for an image
    pub fn get_image_url(&self, key: &str) -> String {
        self.image_provider
            .get_cdn_url(key)
            .unwrap_or_else(|| self.image_provider.get_public_url(key))
    }

    /// Get public URL for a video
    pub fn get_video_url(&self, key: &str) -> String {
        self.video_provider
            .get_cdn_url(key)
            .unwrap_or_else(|| self.video_provider.get_public_url(key))
    }

    /// Delete a file using the appropriate provider
    pub async fn delete_file(&self, key: &str, is_image: bool) -> Result<()> {
        if is_image {
            self.image_provider.delete_file(key).await
        } else {
            self.video_provider.delete_file(key).await
        }
    }
}
