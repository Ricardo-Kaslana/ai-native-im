use crate::services::media::error::{MediaError, Result};
use crate::services::media::storage_manager::StorageProvider;
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use std::path::Path;
use std::time::Duration;

/// CloudFlare R2 storage provider implementation (S3-compatible)
pub struct CloudFlareR2Provider {
    client: Client,
    bucket: String,
    account_id: String,
    custom_domain: Option<String>,
}

impl CloudFlareR2Provider {
    /// Create a new CloudFlare R2 provider
    pub async fn new(
        account_id: String,
        bucket: String,
        access_key: String,
        secret_key: String,
        custom_domain: Option<String>,
    ) -> Result<Self> {
        // R2 endpoint format: https://<account_id>.r2.cloudflarestorage.com
        let endpoint = format!("https://{}.r2.cloudflarestorage.com", account_id);

        // Create credentials
        let credentials = aws_sdk_s3::config::Credentials::new(
            access_key,
            secret_key,
            None,
            None,
            "media-storage",
        );

        // Load AWS config with R2 endpoint
        let config = aws_config::defaults(BehaviorVersion::latest())
            .credentials_provider(credentials)
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .endpoint_url(&endpoint)
            .force_path_style(false) // R2 uses virtual-hosted-style URLs
            .build();

        let client = Client::from_conf(s3_config);

        Ok(Self {
            client,
            bucket,
            account_id,
            custom_domain,
        })
    }

    /// Get the R2 public URL for a key
    fn get_r2_url(&self, key: &str) -> String {
        if let Some(domain) = &self.custom_domain {
            format!("https://{}/{}", domain, key)
        } else {
            format!(
                "https://{}.{}.r2.cloudflarestorage.com/{}",
                self.bucket, self.account_id, key
            )
        }
    }
}

#[async_trait]
impl StorageProvider for CloudFlareR2Provider {
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

        // Upload to R2
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body)
            .content_type(content_type)
            .cache_control("public, max-age=2592000") // 30 days
            .send()
            .await
            .map_err(|e| MediaError::S3Error(format!("R2 upload failed: {}", e)))?;

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
            .presigned(presigning_config)
            .await
            .map_err(|e| MediaError::S3Error(format!("R2 presign failed: {}", e)))?;

        Ok(presigned_request.uri().to_string())
    }

    fn get_public_url(&self, key: &str) -> String {
        self.get_r2_url(key)
    }

    fn get_cdn_url(&self, key: &str) -> Option<String> {
        // R2 has built-in CDN, so we return the custom domain if available
        // Otherwise, the public URL already uses R2's CDN
        self.custom_domain
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
            .map_err(|e| MediaError::S3Error(format!("R2 delete failed: {}", e)))?;

        Ok(())
    }

    fn provider_type(&self) -> &str {
        "cloudflare_r2"
    }
}
