//! Deduplication Cache Module
//! 
//! Prevents duplicate message processing using an in-memory cache with TTL.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Deduplication cache trait
pub trait DedupCache: Send + Sync {
    /// Check if message ID exists, and insert if not
    /// Returns true if the message is new (not a duplicate)
    fn check_and_insert(&self, message_id: &Uuid) -> bool;

    /// Check if message ID exists in cache
    fn contains(&self, message_id: &Uuid) -> bool;

    /// Get cache size
    fn size(&self) -> usize;

    /// Clear expired entries
    fn cleanup(&self);
}

/// In-memory deduplication cache with TTL
pub struct InMemoryDedupCache {
    cache: Arc<RwLock<HashMap<Uuid, Instant>>>,
    ttl: Duration,
}

impl InMemoryDedupCache {
    /// Create a new dedup cache with specified TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    /// Create with default TTL (10 minutes)
    pub fn with_default_ttl() -> Self {
        Self::new(Duration::from_secs(600))
    }

    /// Start background cleanup task
    pub fn start_cleanup_task(self: Arc<Self>) {
        let cache = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                cache.cleanup_async().await;
            }
        });
    }

    /// Async version of check_and_insert
    pub async fn check_and_insert_async(&self, message_id: &Uuid) -> bool {
        let now = Instant::now();
        let mut cache = self.cache.write().await;

        // Check if exists and not expired
        if let Some(inserted_at) = cache.get(message_id) {
            if now.duration_since(*inserted_at) < self.ttl {
                return false; // Duplicate
            }
        }

        // Insert new entry
        cache.insert(*message_id, now);
        true // New message
    }

    /// Async version of contains
    pub async fn contains_async(&self, message_id: &Uuid) -> bool {
        let now = Instant::now();
        let cache = self.cache.read().await;

        if let Some(inserted_at) = cache.get(message_id) {
            now.duration_since(*inserted_at) < self.ttl
        } else {
            false
        }
    }

    /// Async version of size
    pub async fn size_async(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// Async version of cleanup
    pub async fn cleanup_async(&self) {
        let now = Instant::now();
        let mut cache = self.cache.write().await;
        cache.retain(|_, inserted_at| now.duration_since(*inserted_at) < self.ttl);
    }
}

impl DedupCache for InMemoryDedupCache {
    fn check_and_insert(&self, message_id: &Uuid) -> bool {
        let cache = self.cache.clone();
        let ttl = self.ttl;
        let message_id = *message_id;

        let rt = tokio::runtime::Handle::current();
        rt.block_on(async move {
            let now = Instant::now();
            let mut cache = cache.write().await;

            if let Some(inserted_at) = cache.get(&message_id) {
                if now.duration_since(*inserted_at) < ttl {
                    return false;
                }
            }

            cache.insert(message_id, now);
            true
        })
    }

    fn contains(&self, message_id: &Uuid) -> bool {
        let cache = self.cache.clone();
        let ttl = self.ttl;
        let message_id = *message_id;

        let rt = tokio::runtime::Handle::current();
        rt.block_on(async move {
            let now = Instant::now();
            let cache = cache.read().await;

            if let Some(inserted_at) = cache.get(&message_id) {
                now.duration_since(*inserted_at) < ttl
            } else {
                false
            }
        })
    }

    fn size(&self) -> usize {
        let cache = self.cache.clone();

        let rt = tokio::runtime::Handle::current();
        rt.block_on(async move {
            let cache = cache.read().await;
            cache.len()
        })
    }

    fn cleanup(&self) {
        let cache = self.cache.clone();
        let ttl = self.ttl;

        let rt = tokio::runtime::Handle::current();
        rt.block_on(async move {
            let now = Instant::now();
            let mut cache = cache.write().await;
            cache.retain(|_, inserted_at| now.duration_since(*inserted_at) < ttl);
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_and_insert_new_message() {
        let cache = InMemoryDedupCache::with_default_ttl();
        let message_id = Uuid::new_v4();

        // First insert should return true (new message)
        assert!(cache.check_and_insert_async(&message_id).await);
        
        // Second insert should return false (duplicate)
        assert!(!cache.check_and_insert_async(&message_id).await);
    }

    #[tokio::test]
    async fn test_contains() {
        let cache = InMemoryDedupCache::with_default_ttl();
        let message_id = Uuid::new_v4();

        assert!(!cache.contains_async(&message_id).await);
        
        cache.check_and_insert_async(&message_id).await;
        
        assert!(cache.contains_async(&message_id).await);
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let cache = InMemoryDedupCache::new(Duration::from_millis(100));
        let message_id = Uuid::new_v4();

        cache.check_and_insert_async(&message_id).await;
        assert!(cache.contains_async(&message_id).await);

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be able to insert again after expiration
        assert!(cache.check_and_insert_async(&message_id).await);
    }

    #[tokio::test]
    async fn test_cleanup() {
        let cache = InMemoryDedupCache::new(Duration::from_millis(100));
        
        // Insert some messages
        for _ in 0..10 {
            cache.check_and_insert_async(&Uuid::new_v4()).await;
        }
        
        assert_eq!(cache.size_async().await, 10);

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Cleanup should remove expired entries
        cache.cleanup_async().await;
        
        assert_eq!(cache.size_async().await, 0);
    }

    #[tokio::test]
    async fn test_multiple_messages() {
        let cache = InMemoryDedupCache::with_default_ttl();
        let ids: Vec<Uuid> = (0..100).map(|_| Uuid::new_v4()).collect();

        // Insert all messages
        for id in &ids {
            assert!(cache.check_and_insert_async(id).await);
        }

        // All should be duplicates now
        for id in &ids {
            assert!(!cache.check_and_insert_async(id).await);
        }

        assert_eq!(cache.size_async().await, 100);
    }
}


#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

    proptest! {
        /// Property 37: Deduplication check and cache usage
        /// For any message ID, the first check_and_insert returns true,
        /// and subsequent calls return false until TTL expires.
        #[test]
        fn prop_dedup_check_and_insert(
            message_id_bytes in prop::array::uniform16(any::<u8>()),
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let cache = InMemoryDedupCache::with_default_ttl();
                let message_id = Uuid::from_bytes(message_id_bytes);

                // First insert should succeed
                let first_result = cache.check_and_insert_async(&message_id).await;
                prop_assert!(first_result, "First insert should return true");

                // Second insert should fail (duplicate)
                let second_result = cache.check_and_insert_async(&message_id).await;
                prop_assert!(!second_result, "Second insert should return false");

                // Should be in cache
                prop_assert!(cache.contains_async(&message_id).await);

                Ok(())
            })?;
        }

        /// Property 38: Duplicate message handling
        /// Multiple inserts of the same message ID should all return false after the first.
        #[test]
        fn prop_duplicate_handling(
            message_id_bytes in prop::array::uniform16(any::<u8>()),
            duplicate_count in 2usize..10,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let cache = InMemoryDedupCache::with_default_ttl();
                let message_id = Uuid::from_bytes(message_id_bytes);

                // First insert
                let first = cache.check_and_insert_async(&message_id).await;
                prop_assert!(first);

                // All subsequent inserts should be duplicates
                for i in 0..duplicate_count {
                    let result = cache.check_and_insert_async(&message_id).await;
                    prop_assert!(!result, "Insert {} should be duplicate", i + 1);
                }

                // Cache size should be 1
                prop_assert_eq!(cache.size_async().await, 1);

                Ok(())
            })?;
        }

        /// Property: Unique messages should all be accepted
        #[test]
        fn prop_unique_messages_accepted(
            message_count in 1usize..50,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let cache = InMemoryDedupCache::with_default_ttl();
                let mut inserted = HashSet::new();

                for _ in 0..message_count {
                    let message_id = Uuid::new_v4();
                    let result = cache.check_and_insert_async(&message_id).await;
                    prop_assert!(result, "Unique message should be accepted");
                    inserted.insert(message_id);
                }

                // All should be in cache
                for id in &inserted {
                    prop_assert!(cache.contains_async(id).await);
                }

                prop_assert_eq!(cache.size_async().await, message_count);

                Ok(())
            })?;
        }

        /// Property: Cache cleanup removes expired entries
        #[test]
        fn prop_cleanup_removes_expired(
            message_count in 1usize..20,
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Use very short TTL for testing
                let cache = InMemoryDedupCache::new(Duration::from_millis(10));

                // Insert messages
                for _ in 0..message_count {
                    cache.check_and_insert_async(&Uuid::new_v4()).await;
                }

                prop_assert_eq!(cache.size_async().await, message_count);

                // Wait for TTL to expire
                tokio::time::sleep(Duration::from_millis(50)).await;

                // Cleanup
                cache.cleanup_async().await;

                // All should be removed
                prop_assert_eq!(cache.size_async().await, 0);

                Ok(())
            })?;
        }

        /// Property: Messages can be re-inserted after TTL expiration
        #[test]
        fn prop_reinsert_after_expiration(
            message_id_bytes in prop::array::uniform16(any::<u8>()),
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let cache = InMemoryDedupCache::new(Duration::from_millis(10));
                let message_id = Uuid::from_bytes(message_id_bytes);

                // First insert
                let first = cache.check_and_insert_async(&message_id).await;
                prop_assert!(first);

                // Wait for expiration
                tokio::time::sleep(Duration::from_millis(50)).await;

                // Should be able to insert again
                let second = cache.check_and_insert_async(&message_id).await;
                prop_assert!(second, "Should be able to re-insert after TTL expiration");

                Ok(())
            })?;
        }
    }
}
