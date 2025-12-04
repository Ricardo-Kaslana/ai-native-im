use crate::services::media::error::{MediaError, Result};
use crate::services::media::storage_manager::StorageProvider;
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use std::path::Path;
use std::time::Duration;

/// MinIO storage provider implementation (S3-compatible)
pub struct MinIOProvider {
    client: Client,
    bucket: String,
    endpoint: String,
    cdn_domain: Option<String>,
}

impl MinIOProvider {
    /// Create a new MinIO provider
    pub async fn new(
        endpoint: String,
        bucket: String,
        access_key: String,
        secret_key: String,
        cdn_domain: Option<String>,
    ) -> Result<Self> {
        // Create credentials
        let credentials = aws_sdk_s3::config::Credentials::new(
            access_key,
            secret_key,
            None,
            None,
            "media-storage",
        );

        // Load AWS config with custom endpoint
        let config = aws_config::defaults(BehaviorVersion::latest())
            .credentials_provider(credentials)
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .endpoint_url(&endpoint)
            .force_path_style(true) // MinIO requires path-style URLs
            .build();

        let client = Client::from_conf(s3_config);

        Ok(Self {
            client,
            bucket,
            endpoint,
            cdn_domain,
        })
    }

    /// Get the MinIO URL for a key
    fn get_minio_url(&self, key: &str) -> String {
        format!("{}/{}/{}", self.endpoint, self.bucket, key)
    }
}

#[async_trait]
impl StorageProvider for MinIOProvider {
    async fn upload_file(
        &self,
        local_path: &str,
        key: &str,
        content_type: &str,
    ) -> Result<String> {
        // Read file
        let body = ByteStream::from_path(Path::new(local_path))
            .await
            .map_err(|e| MediaError::StorageError(format!("Failed to read file: {}", e)))?;

        // Upload to MinIO
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body)
            .content_type(content_type)
            .cache_control("public, max-age=2592000") // 30 days
            .acl(aws_sdk_s3::types::ObjectCannedAcl::PublicRead)
            .send()
            .await
            .map_err(|e| MediaError::S3Error(format!("MinIO upload failed: {}", e)))?;

        Ok(self.get_public_url(key))
    }

    async fn generate_presigned_url(
        &self,
        key: &str,
        expires_in: Duration,
        content_type: &str,
    ) -> Result<String> {
        let presigning_config = PresigningConfig::expires_in(expires_in)
            .map_err(|e| MediaError::StorageError(format!("Invalid expiration: {}", e)))?;

        let presigned_request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .acl(aws_sdk_s3::types::ObjectCannedAcl::PublicRead)
            .presigned(presigning_config)
            .await
            .map_err(|e| MediaError::S3Error(format!("MinIO presign failed: {}", e)))?;

        Ok(presigned_request.uri().to_string())
    }

    fn get_public_url(&self, key: &str) -> String {
        self.get_minio_url(key)
    }

    fn get_cdn_url(&self, key: &str) -> Option<String> {
        self.cdn_domain
            .as_ref()
            .map(|domain| format!("https://{}/{}", domain, key))
    }

    async fn delete_file(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| MediaError::S3Error(format!("MinIO delete failed: {}", e)))?;

        Ok(())
    }

    fn provider_type(&self) -> &str {
        "minio"
    }
}
