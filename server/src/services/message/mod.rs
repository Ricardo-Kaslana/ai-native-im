//! Message System Module
//! 
//! This module provides the core messaging infrastructure for the IM application,
//! including WebSocket connection management, message routing, persistence, and delivery.

pub mod acknowledgment;
pub mod concurrent_processor;
pub mod config;
pub mod connection_manager;
pub mod dedup_cache;
pub mod disconnection_handler;
pub mod error_handling;
pub mod message_persister;
pub mod message_router;
pub mod message_validator;
pub mod metrics;
pub mod models;
pub mod multi_instance;
pub mod offline_sync;
pub mod redis_client;
pub mod redis_fallback;
pub mod websocket_handler;

pub use acknowledgment::{AcknowledgmentService, ClientPacket, ServerPacket};
pub use concurrent_processor::{ConcurrentConfig, ConcurrentProcessor, ConversationKey, ProcessorError};
pub use config::{
    ConfigError, ConcurrentSettings, MessageSystemConfig, MetricsSettings, OfflineSyncSettings,
    PersistenceSettings, RedisSettings, RoutingSettings, WebSocketSettings,
};
pub use error_handling::{
    ConnectionError, DatabaseError, DegradationHandler, DegradationLevel, ErrorClass,
    ErrorIsolator, MessageSystemError, OperationPriority, RedisError, RetryConfig, RetryExecutor,
};
pub use connection_manager::{ConnectionManager, InMemoryConnectionManager};
pub use dedup_cache::{DedupCache, InMemoryDedupCache};
pub use disconnection_handler::{
    ConnectionHealth, DisconnectionConfig, DisconnectionError, DisconnectionHandler,
};
pub use message_persister::{MessagePersister, PersisterError, PostgresMessagePersister};
pub use message_router::{MessageRouter, MessageRouterImpl, RouterError};
pub use message_validator::{MessageValidator, ValidationError};
pub use models::*;
pub use offline_sync::{OfflineSyncConfig, OfflineSyncError, OfflineSyncService, SyncResult};
pub use redis_client::{RedisClient, RedisClientError, RedisClientTrait, RedisMessage};
pub use metrics::{LatencyTimer, MessageMetrics, MetricsSnapshot};
pub use redis_fallback::{FallbackConfig, FallbackError, RedisFallbackService, RedisHealth};
pub use multi_instance::{
    ChannelNames, EnvelopeMessage, InstanceId, MessageEnvelope, MessageReceiver,
    MultiInstanceConfig, MultiInstanceError, MultiInstanceRouter,
};
pub use websocket_handler::{ConnectionState, WebSocketConfig, WebSocketError, WebSocketHandler};
