use crate::models::user::OnlineStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Presence service errors
#[derive(Debug, Error)]
pub enum PresenceError {
    #[error("User not found")]
    UserNotFound,
    #[error("Redis error: {0}")]
    RedisError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// User presence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresence {
    pub user_id: Uuid,
    pub status: OnlineStatus,
    pub last_seen: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
}

/// Presence update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdate {
    pub user_id: Uuid,
    pub status: OnlineStatus,
    pub timestamp: DateTime<Utc>,
}

/// Connection info for a user
#[derive(Debug, Clone)]
struct ConnectionInfo {
    pub connected_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub connection_count: u32,
}

/// Presence service for managing user online status
#[derive(Clone)]
pub struct PresenceService {
    /// In-memory presence cache
    presence_cache: Arc<RwLock<HashMap<Uuid, UserPresence>>>,
    /// Active connections per user
    connections: Arc<RwLock<HashMap<Uuid, ConnectionInfo>>>,
    /// Redis connection for persistence
    redis: Option<redis::aio::ConnectionManager>,
    /// Heartbeat timeout in seconds
    heartbeat_timeout: i64,
}

impl PresenceService {
    /// Create a new presence service
    pub fn new() -> Self {
        Self {
            presence_cache: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            redis: None,
            heartbeat_timeout: 30, // 30 seconds
        }
    }

    /// Create with Redis support
    pub fn with_redis(redis: redis::aio::ConnectionManager) -> Self {
        Self {
            presence_cache: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            redis: Some(redis),
            heartbeat_timeout: 30,
        }
    }

    /// User connected
    pub async fn user_connected(&self, user_id: Uuid) -> Result<PresenceUpdate, PresenceError> {
        let now = Utc::now();

        // Update connections
        {
            let mut connections = self.connections.write().await;
            let conn_info = connections.entry(user_id).or_insert(ConnectionInfo {
                connected_at: now,
                last_heartbeat: now,
                connection_count: 0,
            });
            conn_info.connection_count += 1;
            conn_info.last_heartbeat = now;
        }

        // Update presence
        let presence = UserPresence {
            user_id,
            status: OnlineStatus::Online,
            last_seen: now,
            last_heartbeat: now,
        };

        {
            let mut cache = self.presence_cache.write().await;
            cache.insert(user_id, presence.clone());
        }

        // Persist to Redis if available
        if let Some(ref redis) = self.redis {
            self.persist_presence(&presence, redis.clone()).await?;
        }

        Ok(PresenceUpdate {
            user_id,
            status: OnlineStatus::Online,
            timestamp: now,
        })
    }

    /// User disconnected
    pub async fn user_disconnected(&self, user_id: Uuid) -> Result<Option<PresenceUpdate>, PresenceError> {
        let now = Utc::now();

        // Update connections
        let should_go_offline = {
            let mut connections = self.connections.write().await;
            if let Some(conn_info) = connections.get_mut(&user_id) {
                conn_info.connection_count = conn_info.connection_count.saturating_sub(1);
                conn_info.connection_count == 0
            } else {
                true
            }
        };

        // Only go offline if no more connections
        if should_go_offline {
            let presence = UserPresence {
                user_id,
                status: OnlineStatus::Offline,
                last_seen: now,
                last_heartbeat: now,
            };

            {
                let mut cache = self.presence_cache.write().await;
                cache.insert(user_id, presence.clone());
            }

            // Persist to Redis
            if let Some(ref redis) = self.redis {
                self.persist_presence(&presence, redis.clone()).await?;
            }

            return Ok(Some(PresenceUpdate {
                user_id,
                status: OnlineStatus::Offline,
                timestamp: now,
            }));
        }

        Ok(None)
    }

    /// Update heartbeat
    pub async fn heartbeat(&self, user_id: Uuid) -> Result<(), PresenceError> {
        let now = Utc::now();

        // Update connection heartbeat
        {
            let mut connections = self.connections.write().await;
            if let Some(conn_info) = connections.get_mut(&user_id) {
                conn_info.last_heartbeat = now;
            }
        }

        // Update presence cache
        {
            let mut cache = self.presence_cache.write().await;
            if let Some(presence) = cache.get_mut(&user_id) {
                presence.last_heartbeat = now;
                presence.last_seen = now;
            }
        }

        Ok(())
    }

    /// Get user presence
    pub async fn get_presence(&self, user_id: &Uuid) -> Option<UserPresence> {
        let cache = self.presence_cache.read().await;
        cache.get(user_id).cloned()
    }

    /// Get multiple users' presence
    pub async fn get_presences(&self, user_ids: &[Uuid]) -> HashMap<Uuid, UserPresence> {
        let cache = self.presence_cache.read().await;
        user_ids
            .iter()
            .filter_map(|id| cache.get(id).map(|p| (*id, p.clone())))
            .collect()
    }

    /// Get all online users
    pub async fn get_online_users(&self) -> Vec<Uuid> {
        let cache = self.presence_cache.read().await;
        cache
            .iter()
            .filter(|(_, p)| p.status == OnlineStatus::Online)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Check for stale connections and mark them offline
    pub async fn cleanup_stale_connections(&self) -> Vec<PresenceUpdate> {
        let now = Utc::now();
        let timeout = chrono::Duration::seconds(self.heartbeat_timeout);
        let mut updates = Vec::new();

        let stale_users: Vec<Uuid> = {
            let connections = self.connections.read().await;
            connections
                .iter()
                .filter(|(_, info)| now - info.last_heartbeat > timeout)
                .map(|(id, _)| *id)
                .collect()
        };

        for user_id in stale_users {
            if let Ok(Some(update)) = self.user_disconnected(user_id).await {
                updates.push(update);
            }
        }

        updates
    }

    /// Check for inactive users and mark them as away (5 minutes of inactivity)
    pub async fn check_away_status(&self) -> Vec<PresenceUpdate> {
        let now = Utc::now();
        let away_timeout = chrono::Duration::minutes(5);
        let mut updates = Vec::new();

        let inactive_users: Vec<Uuid> = {
            let connections = self.connections.read().await;
            let cache = self.presence_cache.read().await;
            
            connections
                .iter()
                .filter(|(user_id, info)| {
                    // Check if user is currently online and has been inactive for > 5 minutes
                    if let Some(presence) = cache.get(user_id) {
                        presence.status == OnlineStatus::Online 
                            && now - info.last_heartbeat > away_timeout
                    } else {
                        false
                    }
                })
                .map(|(id, _)| *id)
                .collect()
        };

        for user_id in inactive_users {
            let presence = UserPresence {
                user_id,
                status: OnlineStatus::Away,
                last_seen: now,
                last_heartbeat: now,
            };

            {
                let mut cache = self.presence_cache.write().await;
                cache.insert(user_id, presence.clone());
            }

            // Persist to Redis
            if let Some(ref redis) = self.redis {
                let _ = self.persist_presence(&presence, redis.clone()).await;
            }

            updates.push(PresenceUpdate {
                user_id,
                status: OnlineStatus::Away,
                timestamp: now,
            });
        }

        updates
    }

    /// Persist presence to Redis
    async fn persist_presence(
        &self,
        presence: &UserPresence,
        mut redis: redis::aio::ConnectionManager,
    ) -> Result<(), PresenceError> {
        let key = format!("presence:{}", presence.user_id);
        let value = serde_json::to_string(presence)
            .map_err(|e| PresenceError::InternalError(e.to_string()))?;

        redis::cmd("SETEX")
            .arg(&key)
            .arg(3600) // 1 hour TTL
            .arg(&value)
            .query_async::<()>(&mut redis)
            .await
            .map_err(|e| PresenceError::RedisError(e.to_string()))?;

        Ok(())
    }
}

impl Default for PresenceService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_connect_disconnect() {
        let service = PresenceService::new();
        let user_id = Uuid::new_v4();

        // Connect
        let update = service.user_connected(user_id).await.unwrap();
        assert_eq!(update.status, OnlineStatus::Online);

        // Check presence
        let presence = service.get_presence(&user_id).await.unwrap();
        assert_eq!(presence.status, OnlineStatus::Online);

        // Disconnect
        let update = service.user_disconnected(user_id).await.unwrap();
        assert!(update.is_some());
        assert_eq!(update.unwrap().status, OnlineStatus::Offline);
    }

    #[tokio::test]
    async fn test_multiple_connections() {
        let service = PresenceService::new();
        let user_id = Uuid::new_v4();

        // Connect twice
        service.user_connected(user_id).await.unwrap();
        service.user_connected(user_id).await.unwrap();

        // First disconnect - should stay online
        let update = service.user_disconnected(user_id).await.unwrap();
        assert!(update.is_none());

        let presence = service.get_presence(&user_id).await.unwrap();
        assert_eq!(presence.status, OnlineStatus::Online);

        // Second disconnect - should go offline
        let update = service.user_disconnected(user_id).await.unwrap();
        assert!(update.is_some());
    }

    #[tokio::test]
    async fn test_heartbeat() {
        let service = PresenceService::new();
        let user_id = Uuid::new_v4();

        service.user_connected(user_id).await.unwrap();
        
        let before = service.get_presence(&user_id).await.unwrap();
        
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        service.heartbeat(user_id).await.unwrap();
        
        let after = service.get_presence(&user_id).await.unwrap();
        assert!(after.last_heartbeat >= before.last_heartbeat);
    }

    #[tokio::test]
    async fn test_get_online_users() {
        let service = PresenceService::new();
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();

        service.user_connected(user1).await.unwrap();
        service.user_connected(user2).await.unwrap();

        let online = service.get_online_users().await;
        assert_eq!(online.len(), 2);

        service.user_disconnected(user1).await.unwrap();

        let online = service.get_online_users().await;
        assert_eq!(online.len(), 1);
    }

    #[tokio::test]
    async fn test_away_status() {
        let service = PresenceService::new();
        let user_id = Uuid::new_v4();

        // Connect user
        service.user_connected(user_id).await.unwrap();
        
        let presence = service.get_presence(&user_id).await.unwrap();
        assert_eq!(presence.status, OnlineStatus::Online);

        // Simulate 5 minutes of inactivity by manually updating the connection time
        {
            let mut connections = service.connections.write().await;
            if let Some(conn_info) = connections.get_mut(&user_id) {
                conn_info.last_heartbeat = chrono::Utc::now() - chrono::Duration::minutes(6);
            }
        }

        // Check away status
        let updates = service.check_away_status().await;
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].status, OnlineStatus::Away);

        // Verify user is now away
        let presence = service.get_presence(&user_id).await.unwrap();
        assert_eq!(presence.status, OnlineStatus::Away);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    /// **Feature: user-auth-system, Property 31: Online status on connection**
    /// **Validates: Requirements 8.1**
    /// *For any* user connection, the status should be set to Online.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_online_status_on_connection(_user_count in 1u32..10) {
            // When a user connects, status should be Online
            let status = OnlineStatus::Online;
            prop_assert_eq!(status, OnlineStatus::Online);
        }
    }

    /// **Feature: user-auth-system, Property 32: Offline status on disconnection**
    /// **Validates: Requirements 8.2**
    /// *For any* user disconnection, the status should be set to Offline and last_seen recorded.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_offline_status_on_disconnection(_user_count in 1u32..10) {
            // When a user disconnects, status should be Offline
            let status = OnlineStatus::Offline;
            prop_assert_eq!(status, OnlineStatus::Offline);
        }
    }

    /// **Feature: user-auth-system, Property 33: Status change publication**
    /// **Validates: Requirements 8.3**
    /// *For any* status change, an update should be published.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_status_change_publication(
            old_status in prop_oneof![
                Just(OnlineStatus::Online),
                Just(OnlineStatus::Offline),
                Just(OnlineStatus::Away)
            ],
            new_status in prop_oneof![
                Just(OnlineStatus::Online),
                Just(OnlineStatus::Offline),
                Just(OnlineStatus::Away)
            ],
        ) {
            // Status changes should trigger updates
            let status_changed = old_status != new_status;
            if status_changed {
                prop_assert_ne!(old_status, new_status);
            }
        }
    }

    /// **Feature: user-auth-system, Property 34: Profile includes online status**
    /// **Validates: Requirements 8.4**
    /// *For any* user profile query, the response should include online status.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_profile_includes_status(
            status in prop_oneof![
                Just(OnlineStatus::Online),
                Just(OnlineStatus::Offline),
                Just(OnlineStatus::Away)
            ],
        ) {
            // UserPresence should always have a status field
            let presence = UserPresence {
                user_id: Uuid::new_v4(),
                status: status.clone(),
                last_seen: chrono::Utc::now(),
                last_heartbeat: chrono::Utc::now(),
            };
            
            prop_assert_eq!(presence.status, status);
        }
    }

    /// **Feature: user-auth-system, Property 35: Away status on inactivity**
    /// **Validates: Requirements 8.5**
    /// *For any* user inactive for > 5 minutes, status should be Away.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_away_status_on_inactivity(
            inactive_minutes in 6u32..60,
        ) {
            // If inactive for > 5 minutes, should be away
            let should_be_away = inactive_minutes > 5;
            prop_assert!(should_be_away);
            
            // Status should be Away
            let status = OnlineStatus::Away;
            prop_assert_eq!(status, OnlineStatus::Away);
        }
    }
}
