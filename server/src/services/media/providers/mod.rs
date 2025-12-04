// Storage provider implementations

pub mod aws_s3;
pub mod minio;
pub mod tencent_cos;
pub mod aliyun_oss;
pub mod cloudflare_r2;
pub mod config;

pub use aws_s3::AwsS3Provider;
pub use minio::MinIOProvider;
pub use tencent_cos::TencentCOSProvider;
pub use aliyun_oss::AliyunOSSProvider;
pub use cloudflare_r2::CloudFlareR2Provider;
pub use config::{StorageConfig, MediaStorageConfig};

use crate::services::media::error::Result;
use crate::services::media::storage_manager::{StorageProvider, StorageProviderType};
use std::sync::Arc;

/// Factory function to create a storage provider from configuration
pub async fn create_provider(config: &StorageConfig) -> Result<Arc<dyn StorageProvider>> {
    let provider: Arc<dyn StorageProvider> = match config.provider_type {
        StorageProviderType::AwsS3 => {
            let region = config.region.clone().unwrap_or_else(|| "us-east-1".to_string());
            Arc::new(
                AwsS3Provider::new(
                    region,
                    config.bucket.clone(),
                    config.access_key.clone(),
                    config.secret_key.clone(),
                    config.cdn_domain.clone(),
                )
                .await?,
            )
        }
        StorageProviderType::MinIO => {
            let endpoint = config.endpoint.clone().unwrap_or_else(|| "http://localhost:9000".to_string());
            Arc::new(
                MinIOProvider::new(
                    endpoint,
                    config.bucket.clone(),
                    config.access_key.clone(),
                    config.secret_key.clone(),
                    config.cdn_domain.clone(),
                )
                .await?,
            )
        }
        StorageProviderType::TencentCOS => {
            let region = config.region.clone().unwrap_or_else(|| "ap-guangzhou".to_string());
            Arc::new(
                TencentCOSProvider::new(
                    region,
                    config.bucket.clone(),
                    config.access_key.clone(),
                    config.secret_key.clone(),
                    config.cdn_domain.clone(),
                )
                .await?,
            )
        }
        StorageProviderType::AliyunOSS => {
            let region = config.region.clone().unwrap_or_else(|| "oss-cn-hangzhou".to_string());
            Arc::new(
                AliyunOSSProvider::new(
                    region,
                    config.bucket.clone(),
                    config.access_key.clone(),
                    config.secret_key.clone(),
                    config.cdn_domain.clone(),
                )
                .await?,
            )
        }
        StorageProviderType::CloudFlareR2 => {
            let account_id = config.endpoint.clone().unwrap_or_default();
            Arc::new(
                CloudFlareR2Provider::new(
                    account_id,
                    config.bucket.clone(),
                    config.access_key.clone(),
                    config.secret_key.clone(),
                    config.cdn_domain.clone(),
                )
                .await?,
            )
        }
    };

    Ok(provider)
}
