use crate::services::media::error::{MediaError, Result};
use crate::services::media::storage_manager::StorageProvider;
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use std::path::Path;
use std::time::Duration;

/// AWS S3 storage provider implementation
pub struct AwsS3Provider {
    client: Client,
    bucket: String,
    region: String,
    cdn_domain: Option<String>,
}

impl AwsS3Provider {
    /// Create a new AWS S3 provider
    pub async fn new(
        region: String,
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

        // Load AWS config
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_config::Region::new(region.clone()))
            .credentials_provider(credentials)
            .load()
            .await;

        let client = Client::new(&config);

        Ok(Self {
            client,
            bucket,
            region,
            cdn_domain,
        })
    }

    /// Get the S3 URL for a key
    fn get_s3_url(&self, key: &str) -> String {
        format!(
            "https://{}.s3.{}.amazonaws.com/{}",
            self.bucket, self.region, key
        )
    }
}

#[async_trait]
impl StorageProvider for AwsS3Provider {
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

        // Upload to S3
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
            .map_err(|e| MediaError::S3Error(e.to_string()))?;

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
            .map_err(|e| MediaError::S3Error(e.to_string()))?;

        Ok(presigned_request.uri().to_string())
    }

    fn get_public_url(&self, key: &str) -> String {
        self.get_s3_url(key)
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
            .map_err(|e| MediaError::S3Error(e.to_string()))?;

        Ok(())
    }

    fn provider_type(&self) -> &str {
        "aws_s3"
    }
}
