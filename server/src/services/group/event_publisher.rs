use crate::services::group::GroupError;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
use uuid::Uuid;

/// Group event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GroupEvent {
    /// Group lifecycle events
    GroupCreated {
        group_id: Uuid,
        group_name: String,
        owner_id: Uuid,
        timestamp: i64,
    },
    GroupUpdated {
        group_id: Uuid,
        updated_fields: Vec<String>,
        timestamp: i64,
    },
    GroupDissolved {
        group_id: Uuid,
        timestamp: i64,
    },

    /// Member events
    MemberJoined {
        group_id: Uuid,
        user_id: Uuid,
        username: String,
        timestamp: i64,
    },
    MemberLeft {
        group_id: Uuid,
        user_id: Uuid,
        username: String,
        timestamp: i64,
    },
    MemberKicked {
        group_id: Uuid,
        user_id: Uuid,
        username: String,
        kicked_by: Uuid,
        timestamp: i64,
    },
    MemberBanned {
        group_id: Uuid,
        user_id: Uuid,
        username: String,
        banned_by: Uuid,
        reason: Option<String>,
        timestamp: i64,
    },
    MemberUnbanned {
        group_id: Uuid,
        user_id: Uuid,
        unbanned_by: Uuid,
        timestamp: i64,
    },

    /// Role change events
    RoleChanged {
        group_id: Uuid,
        user_id: Uuid,
        username: String,
        old_role: String,
        new_role: String,
        changed_by: Uuid,
        timestamp: i64,
    },

    /// Mute events
    MemberMuted {
        group_id: Uuid,
        user_id: Uuid,
        muted_by: Uuid,
        timestamp: i64,
    },
    MemberUnmuted {
        group_id: Uuid,
        user_id: Uuid,
        unmuted_by: Uuid,
        timestamp: i64,
    },

    /// AI Agent events
    AgentAdded {
        group_id: Uuid,
        agent_id: Uuid,
        agent_name: String,
        timestamp: i64,
    },
    AgentRemoved {
        group_id: Uuid,
        agent_id: Uuid,
        timestamp: i64,
    },
    AgentConfigUpdated {
        group_id: Uuid,
        agent_id: Uuid,
        updated_fields: Vec<String>,
        timestamp: i64,
    },
}

impl GroupEvent {
    /// Get the group ID from the event
    pub fn group_id(&self) -> Uuid {
        match self {
            GroupEvent::GroupCreated { group_id, .. }
            | GroupEvent::GroupUpdated { group_id, .. }
            | GroupEvent::GroupDissolved { group_id, .. }
            | GroupEvent::MemberJoined { group_id, .. }
            | GroupEvent::MemberLeft { group_id, .. }
            | GroupEvent::MemberKicked { group_id, .. }
            | GroupEvent::MemberBanned { group_id, .. }
            | GroupEvent::MemberUnbanned { group_id, .. }
            | GroupEvent::RoleChanged { group_id, .. }
            | GroupEvent::MemberMuted { group_id, .. }
            | GroupEvent::MemberUnmuted { group_id, .. }
            | GroupEvent::AgentAdded { group_id, .. }
            | GroupEvent::AgentRemoved { group_id, .. }
            | GroupEvent::AgentConfigUpdated { group_id, .. } => *group_id,
        }
    }

    /// Get the event type as a string
    pub fn event_type(&self) -> &'static str {
        match self {
            GroupEvent::GroupCreated { .. } => "group_created",
            GroupEvent::GroupUpdated { .. } => "group_updated",
            GroupEvent::GroupDissolved { .. } => "group_dissolved",
            GroupEvent::MemberJoined { .. } => "member_joined",
            GroupEvent::MemberLeft { .. } => "member_left",
            GroupEvent::MemberKicked { .. } => "member_kicked",
            GroupEvent::MemberBanned { .. } => "member_banned",
            GroupEvent::MemberUnbanned { .. } => "member_unbanned",
            GroupEvent::RoleChanged { .. } => "role_changed",
            GroupEvent::MemberMuted { .. } => "member_muted",
            GroupEvent::MemberUnmuted { .. } => "member_unmuted",
            GroupEvent::AgentAdded { .. } => "agent_added",
            GroupEvent::AgentRemoved { .. } => "agent_removed",
            GroupEvent::AgentConfigUpdated { .. } => "agent_config_updated",
        }
    }
}

/// Event publisher trait for group events
#[async_trait::async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a group event
    async fn publish(&self, event: GroupEvent) -> Result<(), GroupError>;

    /// Publish multiple events in batch
    async fn publish_batch(&self, events: Vec<GroupEvent>) -> Result<(), GroupError>;
}

/// Redis-based event publisher
pub struct RedisEventPublisher {
    redis: Arc<RwLock<MultiplexedConnection>>,
}

impl RedisEventPublisher {
    pub fn new(redis: Arc<RwLock<MultiplexedConnection>>) -> Self {
        Self { redis }
    }

    /// Get the Redis channel name for a group
    fn get_channel_name(group_id: Uuid) -> String {
        format!("group:{}:events", group_id)
    }
}

#[async_trait::async_trait]
impl EventPublisher for RedisEventPublisher {
    async fn publish(&self, event: GroupEvent) -> Result<(), GroupError> {
        let group_id = event.group_id();
        let channel = Self::get_channel_name(group_id);
        let event_type = event.event_type();

        // Serialize event to JSON
        let payload = serde_json::to_string(&event).map_err(|e| {
            error!("Failed to serialize group event: {}", e);
            GroupError::Internal(format!("Failed to serialize event: {}", e))
        })?;

        // Publish to Redis
        let mut conn = self.redis.write().await;
        conn.publish::<_, _, ()>(&channel, &payload)
            .await
            .map_err(|e| {
                error!("Failed to publish group event to Redis: {}", e);
                GroupError::Internal(format!("Failed to publish event: {}", e))
            })?;

        info!(
            "Published {} event to group {} on channel {}",
            event_type, group_id, channel
        );

        Ok(())
    }

    async fn publish_batch(&self, events: Vec<GroupEvent>) -> Result<(), GroupError> {
        if events.is_empty() {
            return Ok(());
        }

        let event_count = events.len();
        let mut conn = self.redis.write().await;

        for event in events {
            let group_id = event.group_id();
            let channel = Self::get_channel_name(group_id);

            let payload = serde_json::to_string(&event).map_err(|e| {
                error!("Failed to serialize group event: {}", e);
                GroupError::Internal(format!("Failed to serialize event: {}", e))
            })?;

            conn.publish::<_, _, ()>(&channel, &payload)
                .await
                .map_err(|e| {
                    error!("Failed to publish group event to Redis: {}", e);
                    GroupError::Internal(format!("Failed to publish event: {}", e))
                })?;
        }

        info!("Published {} events in batch", event_count);

        Ok(())
    }
}

/// Mock event publisher for testing
#[cfg(test)]
pub struct MockEventPublisher {
    events: Arc<RwLock<Vec<GroupEvent>>>,
}

#[cfg(test)]
impl MockEventPublisher {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get_events(&self) -> Vec<GroupEvent> {
        self.events.read().await.clone()
    }

    pub async fn clear(&self) {
        self.events.write().await.clear();
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl EventPublisher for MockEventPublisher {
    async fn publish(&self, event: GroupEvent) -> Result<(), GroupError> {
        self.events.write().await.push(event);
        Ok(())
    }

    async fn publish_batch(&self, events: Vec<GroupEvent>) -> Result<(), GroupError> {
        self.events.write().await.extend(events);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_event_publisher() {
        let publisher = MockEventPublisher::new();

        // Test single event
        let event = GroupEvent::GroupCreated {
            group_id: Uuid::new_v4(),
            group_name: "Test Group".to_string(),
            owner_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        publisher.publish(event.clone()).await.unwrap();

        let events = publisher.get_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type(), "group_created");

        // Test batch
        publisher.clear().await;

        let events_batch = vec![
            GroupEvent::MemberJoined {
                group_id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                username: "user1".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            },
            GroupEvent::MemberJoined {
                group_id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                username: "user2".to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            },
        ];

        publisher.publish_batch(events_batch).await.unwrap();

        let events = publisher.get_events().await;
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_group_event_serialization() {
        let event = GroupEvent::GroupCreated {
            group_id: Uuid::new_v4(),
            group_name: "Test Group".to_string(),
            owner_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: GroupEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.event_type(), deserialized.event_type());
        assert_eq!(event.group_id(), deserialized.group_id());
    }
}
