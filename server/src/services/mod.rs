pub mod admin;
pub mod ai;
pub mod auth;
pub mod email;
pub mod group;
pub mod media;
pub mod message;
pub mod presence;

pub use admin::{
    AdminError, AdminService, AdminStats, AuditLogQuery, AuditLogResponse,
    UserListResponse, UserSearchQuery,
};
pub use ai::{
    AgentMessage, AgentMessageHandler, AgentMessageSender, AgentResponse, AgentResponseWorker,
    AgentUserIdResolver, AIAgentError, AIAgentService, AIAgentSystem, AIAgentSystemConfig,
    AIMessageRouter, AIMessageRouterBuilder, AnthropicClient, ChatMessage, ChatRole,
    DatabaseAgentIdResolver, DefaultPersonalityManager, DefaultTriggerEvaluator,
    DefaultTypingSimulator, DirectAgentIdResolver, EmbeddingService, FastTypingSimulator,
    GoogleGeminiClient, LLMClient, LLMClientFactory, LLMConfig, LLMProvider, MessageContent,
    MockEmbeddingService, MockLLMClient, MockVectorStore, MockVisionModel, MoonshotClient,
    MultimodalProcessor, OpenAIClient, OpenAIEmbedding, OpenAIVision, PersonalityManager,
    QdrantVectorStore, RouterMessageSender, SiliconFlowClient, SiliconFlowEmbedding,
    SiliconFlowVision, StreamChunk, TongyiClient, TriggerEvaluator, TypingSimulator, VectorStore,
    VisionModel, WebSocketMessageSender, WenxinClient, ZhipuClient, EMBEDDING_DIMENSION,
    MESSAGES_COLLECTION,
};
pub use email::{EmailError, EmailService, EmailTemplate, MockEmailService};
pub use group::{
    GroupError, GroupService, InviteService, MemberService, Permission, PermissionService,
    DEFAULT_MAX_CAPACITY,
};
pub use message::{
    ConnectionManager, ConnectionState, CreateMessage, DedupCache, InMemoryConnectionManager,
    InMemoryDedupCache, Message, MessageAck, MessagePersister, MessageRouter, MessageRouterImpl,
    MessageValidator, MsgType, OfflineMessageBatch, PersisterError, PostgresMessagePersister,
    RedisClient, RedisClientError, RedisClientTrait, RedisMessage, RouterError, ValidationError,
    WebSocketConfig, WebSocketError, WebSocketHandler,
};
pub use media::{
    DeduplicationManager, ImageProcessor, MediaConfig, MediaError, MediaFile, MediaInfoResponse,
    MediaMetadata, MediaStatus, MediaType, ProcessedImage, ProcessedVideo, StorageManager,
    UploadHandler, UploadToken, VideoProcessor,
};
pub use presence::{PresenceError, PresenceService, PresenceUpdate, UserPresence};
