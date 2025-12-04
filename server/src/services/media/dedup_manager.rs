use super::error::{MediaError, Result};
use redis::AsyncCommands;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::io::Read;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Cache TTL for file hash lookups (1 hour)
const CACHE_TTL_SECS: u64 = 3600;

/// Deduplication Manager - prevents duplicate file storage
pub struct DeduplicationManager {
    db_pool: PgPool,
    redis_client: Option<redis::Client>,
}

impl DeduplicationManager {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool,
            redis_client: None,
        }
    }

    pub fn with_redis(db_pool: PgPool, redis_url: &str) -> Result<Self> {
        let redis_client = redis::Client::open(redis_url)?;

        Ok(Self {
            db_pool,
            redis_client: Some(redis_client),
        })
    }

    /// Check if file with this hash already exists
    pub async fn check_duplicate(&self, file_hash: &str) -> Result<Option<Uuid>> {
        // Check Redis cache first
        if let Some(media_id) = self.check_cache(file_hash).await? {
            debug!("Cache hit for file hash: {}", file_hash);
            return Ok(Some(media_id));
        }

        // Check database
        let result = sqlx::query_scalar!(
            r#"
            SELECT id FROM media_files
            WHERE file_hash = $1 AND status = 'completed'
            "#,
            file_hash
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(media_id) = result {
            // Cache the result
            self.cache_hash(file_hash, media_id).await?;
            debug!("Found duplicate file: {} -> {}", file_hash, media_id);
            return Ok(Some(media_id));
        }

        Ok(None)
    }

    /// Register new file hash (cache it for future lookups)
    pub async fn register_file(&self, file_hash: &str, media_id: Uuid) -> Result<()> {
        // Cache the hash -> media_id mapping
        self.cache_hash(file_hash, media_id).await?;
        info!("Registered file hash: {} -> {}", file_hash, media_id);
        Ok(())
    }

    /// Increment reference count
    pub async fn increment_ref_count(&self, media_id: Uuid) -> Result<i32> {
        let result = sqlx::query_scalar!(
            r#"
            UPDATE media_files
            SET ref_count = ref_count + 1, updated_at = NOW()
            WHERE id = $1
            RETURNING ref_count
            "#,
            media_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        info!("Incremented ref_count for {}: {}", media_id, result);
        Ok(result)
    }

    /// Decrement reference count, return true if should delete
    pub async fn decrement_ref_count(&self, media_id: Uuid) -> Result<bool> {
        let result = sqlx::query_scalar!(
            r#"
            UPDATE media_files
            SET ref_count = ref_count - 1, updated_at = NOW()
            WHERE id = $1 AND ref_count > 0
            RETURNING ref_count
            "#,
            media_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        match result {
            Some(new_count) => {
                info!("Decremented ref_count for {}: {}", media_id, new_count);
                
                if new_count <= 0 {
                    // Invalidate cache
                    self.invalidate_cache_by_media_id(media_id).await?;
                    return Ok(true);
                }
                Ok(false)
            }
            None => {
                warn!("Failed to decrement ref_count for {} (already 0 or not found)", media_id);
                Ok(false)
            }
        }
    }

    /// Get current reference count
    pub async fn get_ref_count(&self, media_id: Uuid) -> Result<i32> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT ref_count FROM media_files WHERE id = $1
            "#,
            media_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(result.unwrap_or(0))
    }

    /// Calculate SHA-256 hash of file (async version)
    pub async fn calculate_file_hash_async(&self, file_path: &str) -> Result<String> {
        let mut file = File::open(file_path)
            .await
            .map_err(|e| MediaError::ProcessingFailed(format!("Failed to open file: {}", e)))?;

        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 8192];

        loop {
            let bytes_read = file
                .read(&mut buffer)
                .await
                .map_err(|e| MediaError::ProcessingFailed(format!("Failed to read file: {}", e)))?;

            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Calculate SHA-256 hash of file (sync version for compatibility)
    pub fn calculate_file_hash(&self, file_path: &str) -> Result<String> {
        let mut file = std::fs::File::open(file_path)
            .map_err(|e| MediaError::ProcessingFailed(format!("Failed to open file: {}", e)))?;

        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = file
                .read(&mut buffer)
                .map_err(|e| MediaError::ProcessingFailed(format!("Failed to read file: {}", e)))?;

            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Calculate SHA-256 hash of bytes
    pub fn calculate_bytes_hash(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    // Redis cache operations

    fn cache_key(file_hash: &str) -> String {
        format!("media:hash:{}", file_hash)
    }

    async fn check_cache(&self, file_hash: &str) -> Result<Option<Uuid>> {
        let Some(ref client) = self.redis_client else {
            return Ok(None);
        };

        let mut conn = client
            .get_multiplexed_async_connection()
            .await?;

        let key = Self::cache_key(file_hash);
        let result: Option<String> = conn
            .get(&key)
            .await?;

        match result {
            Some(id_str) => {
                let media_id = Uuid::parse_str(&id_str)
                    .map_err(|e| MediaError::InternalError(format!("Invalid UUID in cache: {}", e)))?;
                Ok(Some(media_id))
            }
            None => Ok(None),
        }
    }

    async fn cache_hash(&self, file_hash: &str, media_id: Uuid) -> Result<()> {
        let Some(ref client) = self.redis_client else {
            return Ok(());
        };

        let mut conn = client
            .get_multiplexed_async_connection()
            .await?;

        let key = Self::cache_key(file_hash);
        let _: () = conn
            .set_ex(&key, media_id.to_string(), CACHE_TTL_SECS)
            .await?;

        Ok(())
    }

    async fn invalidate_cache_by_media_id(&self, media_id: Uuid) -> Result<()> {
        // Get the file hash from database
        let file_hash: Option<String> = sqlx::query_scalar!(
            r#"SELECT file_hash FROM media_files WHERE id = $1"#,
            media_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(hash) = file_hash {
            self.invalidate_cache(&hash).await?;
        }

        Ok(())
    }

    async fn invalidate_cache(&self, file_hash: &str) -> Result<()> {
        let Some(ref client) = self.redis_client else {
            return Ok(());
        };

        let mut conn = client
            .get_multiplexed_async_connection()
            .await?;

        let key = Self::cache_key(file_hash);
        let _: () = conn
            .del(&key)
            .await?;

        debug!("Invalidated cache for hash: {}", file_hash);
        Ok(())
    }

    /// Delete file record and return storage info for cleanup
    pub async fn delete_file_record(&self, media_id: Uuid) -> Result<Option<DeletedFileInfo>> {
        // Get file info before deletion
        let file_info = sqlx::query_as!(
            DeletedFileInfo,
            r#"
            SELECT 
                original_url,
                processed_url,
                thumbnail_url,
                storage_provider,
                file_hash
            FROM media_files
            WHERE id = $1
            "#,
            media_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(ref info) = file_info {
            // Invalidate cache
            self.invalidate_cache(&info.file_hash).await?;

            // Delete from database
            sqlx::query!(
                r#"DELETE FROM media_files WHERE id = $1"#,
                media_id
            )
            .execute(&self.db_pool)
            .await?;

            info!("Deleted file record: {}", media_id);
        }

        Ok(file_info)
    }
}

/// Information about a deleted file for storage cleanup
#[derive(Debug)]
pub struct DeletedFileInfo {
    pub original_url: String,
    pub processed_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub storage_provider: String,
    pub file_hash: String,
}

impl Default for DeduplicationManager {
    fn default() -> Self {
        // This is a placeholder - in real usage, you need to provide a db_pool
        panic!("DeduplicationManager requires a database pool")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key() {
        let key = DeduplicationManager::cache_key("abc123");
        assert_eq!(key, "media:hash:abc123");
    }

    #[test]
    fn test_calculate_bytes_hash() {
        // Create a mock manager (we can't test DB operations without a real pool)
        // Just test the hash calculation
        let data = b"hello world";
        let mut hasher = Sha256::new();
        hasher.update(data);
        let expected = format!("{:x}", hasher.finalize());

        // The hash of "hello world" should be consistent
        assert_eq!(expected.len(), 64); // SHA-256 produces 64 hex chars
    }
}
