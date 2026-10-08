// Storage provider implementations

pub mod tencent_cos;
pub mod aliyun_oss;
pub mod config;

pub use tencent_cos::TencentCOSProvider;
pub use aliyun_oss::AliyunOSSProvider;
pub use config::{StorageConfig, MediaStorageConfig};

use crate::services::media::error::Result;
use crate::services::media::storage_manager::{StorageProvider, StorageProviderType};
use std::sync::Arc;

/// Factory function to create a storage provider from configuration
pub async fn create_provider(config: &StorageConfig) -> Result<Arc<dyn StorageProvider>> {
    let provider: Arc<dyn StorageProvider> = match config.provider_type {
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
    };

    Ok(provider)
}
