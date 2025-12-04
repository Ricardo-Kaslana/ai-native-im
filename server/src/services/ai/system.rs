//! AI Agent System
//!
//! Provides a unified interface for initializing and managing the AI Agent system.
//! Supports multiple LLM providers: OpenAI, Anthropic, Google Gemini, SiliconFlow, and custom/local models.

use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};

use super::{
    AgentMessageHandler, AgentResponse, AgentResponseWorker, AgentUserIdResolver, AIAgentService,
    DirectAgentIdResolver, EmbeddingService, LLMClient, LLMClientFactory, LLMProvider,
    MockEmbeddingService, MockLLMClient, MockVectorStore, MockVisionModel, MultimodalProcessor,
    OpenAIEmbedding, OpenAIVision, QdrantVectorStore, RouterMessageSender, SiliconFlowEmbedding,
    SiliconFlowVision, VectorStore, VisionModel,
};
use crate::services::message::message_router::MessageRouter;

/// AI Agent System configuration
#[derive(Debug, Clone)]
pub struct AIAgentSystemConfig {
    // OpenAI configuration
    /// OpenAI API key
    pub openai_api_key: Option<String>,
    /// OpenAI API base URL (optional, for Azure OpenAI or custom endpoints)
    pub openai_api_base: Option<String>,
    /// OpenAI model to use
    pub openai_model: Option<String>,

    // Anthropic configuration
    /// Anthropic API key
    pub anthropic_api_key: Option<String>,
    /// Anthropic model to use
    pub anthropic_model: Option<String>,

    // Google configuration
    /// Google API key
    pub google_api_key: Option<String>,
    /// Google model to use
    pub google_model: Option<String>,

    // SiliconFlow (硅基流动) configuration
    /// SiliconFlow API key
    pub siliconflow_api_key: Option<String>,
    /// SiliconFlow model to use
    pub siliconflow_model: Option<String>,

    // Custom/Local LLM configuration
    /// Custom LLM API base URL (OpenAI-compatible)
    pub custom_llm_api_base: Option<String>,
    /// Custom LLM model name
    pub custom_llm_model: Option<String>,
    /// Custom LLM API key (optional)
    pub custom_llm_api_key: Option<String>,

    // Embedding configuration
    /// Embedding model to use (OpenAI or SiliconFlow)
    pub embedding_model: Option<String>,
    /// SiliconFlow embedding model (if using SiliconFlow for embeddings)
    pub siliconflow_embedding_model: Option<String>,

    // Vision configuration
    /// Vision model to use (OpenAI or SiliconFlow)
    pub vision_model: Option<String>,
    /// SiliconFlow vision model (if using SiliconFlow for vision)
    pub siliconflow_vision_model: Option<String>,

    // Vector store configuration
    /// Qdrant URL (optional, uses mock if not provided)
    pub qdrant_url: Option<String>,

    // General configuration
    /// Maximum recent messages to cache per group
    pub max_recent_messages: usize,
    /// Enable mock mode for testing
    pub mock_mode: bool,
    /// Enable vision/multimodal capabilities
    pub enable_vision: bool,
}

impl Default for AIAgentSystemConfig {
    fn default() -> Self {
        Self {
            // OpenAI
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            openai_api_base: std::env::var("OPENAI_API_BASE").ok(),
            openai_model: std::env::var("OPENAI_MODEL").ok(),

            // Anthropic
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            anthropic_model: std::env::var("ANTHROPIC_MODEL").ok(),

            // Google
            google_api_key: std::env::var("GOOGLE_API_KEY").ok(),
            google_model: std::env::var("GOOGLE_MODEL").ok(),

            // SiliconFlow
            siliconflow_api_key: std::env::var("SILICONFLOW_API_KEY").ok(),
            siliconflow_model: std::env::var("SILICONFLOW_MODEL").ok(),

            // Custom/Local
            custom_llm_api_base: std::env::var("LLM_API_BASE").ok(),
            custom_llm_model: std::env::var("LLM_MODEL").ok(),
            custom_llm_api_key: std::env::var("LLM_API_KEY").ok(),

            // Embedding & Vision
            embedding_model: std::env::var("OPENAI_EMBEDDING_MODEL").ok(),
            siliconflow_embedding_model: std::env::var("SILICONFLOW_EMBEDDING_MODEL").ok(),
            vision_model: std::env::var("OPENAI_VISION_MODEL").ok(),
            siliconflow_vision_model: std::env::var("SILICONFLOW_VISION_MODEL").ok(),

            // Vector store
            qdrant_url: std::env::var("QDRANT_URL").ok(),

            // General
            max_recent_messages: 20,
            mock_mode: false,
            enable_vision: std::env::var("ENABLE_VISION")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
        }
    }
}

impl AIAgentSystemConfig {
    /// Create a mock configuration for testing
    pub fn mock() -> Self {
        Self {
            openai_api_key: None,
            openai_api_base: None,
            openai_model: None,
            anthropic_api_key: None,
            anthropic_model: None,
            google_api_key: None,
            google_model: None,
            siliconflow_api_key: None,
            siliconflow_model: None,
            custom_llm_api_base: None,
            custom_llm_model: None,
            custom_llm_api_key: None,
            embedding_model: None,
            siliconflow_embedding_model: None,
            vision_model: None,
            siliconflow_vision_model: None,
            qdrant_url: None,
            max_recent_messages: 20,
            mock_mode: true,
            enable_vision: true,
        }
    }

    /// Check if any LLM provider is configured
    pub fn has_llm(&self) -> bool {
        self.mock_mode
            || self.openai_api_key.is_some()
            || self.anthropic_api_key.is_some()
            || self.google_api_key.is_some()
            || self.siliconflow_api_key.is_some()
            || self.custom_llm_api_base.is_some()
    }

    /// Get the configured LLM provider
    pub fn llm_provider(&self) -> Option<LLMProvider> {
        if self.mock_mode {
            return Some(LLMProvider::Custom);
        }
        if self.openai_api_key.is_some() {
            return Some(LLMProvider::OpenAI);
        }
        if self.anthropic_api_key.is_some() {
            return Some(LLMProvider::Anthropic);
        }
        if self.google_api_key.is_some() {
            return Some(LLMProvider::Google);
        }
        if self.siliconflow_api_key.is_some() {
            return Some(LLMProvider::SiliconFlow);
        }
        if self.custom_llm_api_base.is_some() {
            return Some(LLMProvider::Custom);
        }
        None
    }

    /// Check if vector store is available
    pub fn has_vector_store(&self) -> bool {
        self.mock_mode || self.qdrant_url.is_some()
    }

    /// Check if vision features are available (requires OpenAI or SiliconFlow)
    pub fn has_vision(&self) -> bool {
        self.enable_vision
            && (self.mock_mode
                || self.openai_api_key.is_some()
                || self.siliconflow_api_key.is_some())
    }

    /// Check if embedding service is available (requires OpenAI or SiliconFlow)
    pub fn has_embeddings(&self) -> bool {
        self.mock_mode || self.openai_api_key.is_some() || self.siliconflow_api_key.is_some()
    }
}

/// AI Agent System
///
/// Manages all AI Agent components and provides a unified interface.
pub struct AIAgentSystem {
    /// The AI Agent service
    pub agent_service: Arc<AIAgentService>,
    /// The message handler
    pub message_handler: Arc<AgentMessageHandler>,
    /// The multimodal processor
    pub multimodal_processor: Arc<MultimodalProcessor>,
    /// Response channel sender
    response_tx: mpsc::UnboundedSender<AgentResponse>,
    /// Response channel receiver (taken when worker is started)
    response_rx: Option<mpsc::UnboundedReceiver<AgentResponse>>,
    /// Configuration
    config: AIAgentSystemConfig,
}

impl AIAgentSystem {
    /// Initialize the AI Agent system
    pub async fn new(pool: sqlx::PgPool, config: AIAgentSystemConfig) -> Result<Self, String> {
        info!("🤖 Initializing AI Agent System...");

        // Create base service
        let mut agent_service = AIAgentService::new(pool.clone());

        // Configure LLM client based on available providers
        if config.mock_mode {
            info!("  Using mock LLM client");
            let llm_client: Arc<dyn LLMClient> =
                Arc::new(MockLLMClient::new("Mock AI response".to_string()));
            agent_service = agent_service.with_llm_client(llm_client);
        } else if let Some(llm_client) = Self::create_llm_client(&config) {
            agent_service = agent_service.with_llm_client(llm_client);
        } else {
            warn!("  No LLM client configured - AI responses will not work");
            warn!("  Set one of: OPENAI_API_KEY, ANTHROPIC_API_KEY, GOOGLE_API_KEY, SILICONFLOW_API_KEY, or LLM_API_BASE");
        }

        // Configure embedding service
        // Priority: OpenAI > SiliconFlow > Mock
        if config.mock_mode {
            info!("  Using mock embedding service");
            let embedding_service: Arc<dyn EmbeddingService> =
                Arc::new(MockEmbeddingService::default());
            agent_service = agent_service.with_embedding_service(embedding_service);
        } else if let Some(ref api_key) = config.openai_api_key {
            info!("  Configuring OpenAI embedding service");
            let embedding_service: Arc<dyn EmbeddingService> = Arc::new(OpenAIEmbedding::new(
                api_key.clone(),
                config.openai_api_base.clone(),
                config.embedding_model.clone(),
            ));
            agent_service = agent_service.with_embedding_service(embedding_service);
        } else if let Some(ref api_key) = config.siliconflow_api_key {
            info!("  Configuring SiliconFlow embedding service");
            let model = config.siliconflow_embedding_model.clone();
            if let Some(ref m) = model {
                info!("    Model: {}", m);
            }
            let embedding_service: Arc<dyn EmbeddingService> =
                Arc::new(SiliconFlowEmbedding::new(api_key.clone(), model));
            agent_service = agent_service.with_embedding_service(embedding_service);
        }

        // Configure vector store
        if config.mock_mode {
            info!("  Using mock vector store");
            let vector_store: Arc<dyn VectorStore> = Arc::new(MockVectorStore::new());
            agent_service = agent_service.with_vector_store(vector_store);
        } else if let Some(ref qdrant_url) = config.qdrant_url {
            info!("  Connecting to Qdrant at {}", qdrant_url);
            match QdrantVectorStore::new(qdrant_url).await {
                Ok(store) => {
                    info!("  ✅ Qdrant connected");
                    let vector_store: Arc<dyn VectorStore> = Arc::new(store);
                    agent_service = agent_service.with_vector_store(vector_store);
                }
                Err(e) => {
                    warn!("  ⚠️ Failed to connect to Qdrant: {}", e);
                    warn!("  Using mock vector store as fallback");
                    let vector_store: Arc<dyn VectorStore> = Arc::new(MockVectorStore::new());
                    agent_service = agent_service.with_vector_store(vector_store);
                }
            }
        } else {
            info!("  Using mock vector store (no Qdrant URL configured)");
            let vector_store: Arc<dyn VectorStore> = Arc::new(MockVectorStore::new());
            agent_service = agent_service.with_vector_store(vector_store);
        }

        let agent_service = Arc::new(agent_service);

        // Create response channel
        let (response_tx, response_rx) = mpsc::unbounded_channel();

        // Create message handler
        let message_handler = Arc::new(
            AgentMessageHandler::new((*agent_service).clone())
                .with_response_channel(response_tx.clone()),
        );

        // Create multimodal processor
        // Priority: OpenAI > SiliconFlow > Mock
        let multimodal_processor = if config.has_vision() {
            if config.mock_mode {
                info!("  Using mock vision model");
                let vision: Box<dyn VisionModel> = Box::new(MockVisionModel::default_response());
                Arc::new(MultimodalProcessor::new().with_vision_model(vision))
            } else if let Some(ref api_key) = config.openai_api_key {
                info!("  Configuring OpenAI Vision model");
                let vision: Box<dyn VisionModel> = Box::new(OpenAIVision::new(
                    api_key.clone(),
                    config.openai_api_base.clone(),
                    config.vision_model.clone(),
                ));
                Arc::new(MultimodalProcessor::new().with_vision_model(vision))
            } else if let Some(ref api_key) = config.siliconflow_api_key {
                info!("  Configuring SiliconFlow Vision model");
                let model = config.siliconflow_vision_model.clone();
                if let Some(ref m) = model {
                    info!("    Model: {}", m);
                }
                let vision: Box<dyn VisionModel> =
                    Box::new(SiliconFlowVision::new(api_key.clone(), model));
                Arc::new(MultimodalProcessor::new().with_vision_model(vision))
            } else {
                info!("  Vision disabled (no API key)");
                Arc::new(MultimodalProcessor::new())
            }
        } else {
            info!("  Vision disabled");
            Arc::new(MultimodalProcessor::new())
        };

        info!("✅ AI Agent System initialized");

        Ok(Self {
            agent_service,
            message_handler,
            multimodal_processor,
            response_tx,
            response_rx: Some(response_rx),
            config,
        })
    }

    /// Create LLM client based on configuration
    /// Priority: OpenAI > Anthropic > Google > SiliconFlow > Custom
    fn create_llm_client(config: &AIAgentSystemConfig) -> Option<Arc<dyn LLMClient>> {
        // Try OpenAI first
        if let Some(ref api_key) = config.openai_api_key {
            info!("  Configuring OpenAI LLM client");
            if let Some(ref model) = config.openai_model {
                info!("    Model: {}", model);
            }
            return Some(LLMClientFactory::openai_compatible(
                api_key.clone(),
                config
                    .openai_api_base
                    .clone()
                    .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
                config
                    .openai_model
                    .clone()
                    .unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
            ));
        }

        // Try Anthropic
        if let Some(ref api_key) = config.anthropic_api_key {
            info!("  Configuring Anthropic Claude LLM client");
            if let Some(ref model) = config.anthropic_model {
                info!("    Model: {}", model);
            }
            return Some(LLMClientFactory::anthropic(
                api_key.clone(),
                config.anthropic_model.clone(),
            ));
        }

        // Try Google
        if let Some(ref api_key) = config.google_api_key {
            info!("  Configuring Google Gemini LLM client");
            if let Some(ref model) = config.google_model {
                info!("    Model: {}", model);
            }
            return Some(LLMClientFactory::google(
                api_key.clone(),
                config.google_model.clone(),
            ));
        }

        // Try SiliconFlow (硅基流动)
        if let Some(ref api_key) = config.siliconflow_api_key {
            info!("  Configuring SiliconFlow LLM client");
            if let Some(ref model) = config.siliconflow_model {
                info!("    Model: {}", model);
            }
            return Some(LLMClientFactory::siliconflow(
                api_key.clone(),
                config.siliconflow_model.clone(),
            ));
        }

        // Try custom/local LLM
        if let Some(ref api_base) = config.custom_llm_api_base {
            let model = config
                .custom_llm_model
                .clone()
                .unwrap_or_else(|| "default".to_string());
            info!("  Configuring custom LLM client at {}", api_base);
            info!("    Model: {}", model);
            return Some(LLMClientFactory::openai_compatible(
                config.custom_llm_api_key.clone().unwrap_or_default(),
                api_base.clone(),
                model,
            ));
        }

        None
    }

    /// Start the response worker
    ///
    /// This spawns a background task that processes AI responses with typing delays.
    pub fn start_response_worker<R: MessageRouter + 'static>(
        &mut self,
        router: Arc<R>,
    ) -> Option<tokio::task::JoinHandle<()>> {
        let response_rx = self.response_rx.take()?;

        let resolver: Arc<dyn AgentUserIdResolver> = Arc::new(DirectAgentIdResolver);
        let message_sender = Arc::new(RouterMessageSender::new(router, resolver));

        let worker = AgentResponseWorker::new(response_rx, message_sender);

        let handle = tokio::spawn(async move {
            worker.run().await;
        });

        info!("🚀 AI Agent response worker started");

        Some(handle)
    }

    /// Get a clone of the response sender
    pub fn response_sender(&self) -> mpsc::UnboundedSender<AgentResponse> {
        self.response_tx.clone()
    }

    /// Get the configuration
    pub fn config(&self) -> &AIAgentSystemConfig {
        &self.config
    }

    /// Check if the system is fully configured
    pub fn is_fully_configured(&self) -> bool {
        self.config.has_llm() && self.config.has_vector_store()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = AIAgentSystemConfig::default();
        assert_eq!(config.max_recent_messages, 20);
        assert!(!config.mock_mode);
    }

    #[test]
    fn test_config_mock() {
        let config = AIAgentSystemConfig::mock();
        assert!(config.mock_mode);
        assert!(config.has_llm());
        assert!(config.has_vector_store());
        assert!(config.has_vision());
    }

    #[test]
    fn test_config_has_llm_openai() {
        let mut config = AIAgentSystemConfig::mock();
        config.mock_mode = false;
        config.openai_api_key = None;
        config.anthropic_api_key = None;
        config.google_api_key = None;
        config.custom_llm_api_base = None;
        assert!(!config.has_llm());

        config.openai_api_key = Some("test-key".to_string());
        assert!(config.has_llm());
        assert_eq!(config.llm_provider(), Some(LLMProvider::OpenAI));
    }

    #[test]
    fn test_config_has_llm_anthropic() {
        let mut config = AIAgentSystemConfig::mock();
        config.mock_mode = false;
        config.openai_api_key = None;
        config.anthropic_api_key = Some("test-key".to_string());
        config.google_api_key = None;
        config.custom_llm_api_base = None;

        assert!(config.has_llm());
        assert_eq!(config.llm_provider(), Some(LLMProvider::Anthropic));
    }

    #[test]
    fn test_config_has_llm_google() {
        let mut config = AIAgentSystemConfig::mock();
        config.mock_mode = false;
        config.openai_api_key = None;
        config.anthropic_api_key = None;
        config.google_api_key = Some("test-key".to_string());
        config.custom_llm_api_base = None;

        assert!(config.has_llm());
        assert_eq!(config.llm_provider(), Some(LLMProvider::Google));
    }

    #[test]
    fn test_config_has_llm_siliconflow() {
        let mut config = AIAgentSystemConfig::mock();
        config.mock_mode = false;
        config.openai_api_key = None;
        config.anthropic_api_key = None;
        config.google_api_key = None;
        config.siliconflow_api_key = Some("test-key".to_string());
        config.custom_llm_api_base = None;

        assert!(config.has_llm());
        assert_eq!(config.llm_provider(), Some(LLMProvider::SiliconFlow));
    }

    #[test]
    fn test_config_has_llm_custom() {
        let mut config = AIAgentSystemConfig::mock();
        config.mock_mode = false;
        config.openai_api_key = None;
        config.anthropic_api_key = None;
        config.google_api_key = None;
        config.siliconflow_api_key = None;
        config.custom_llm_api_base = Some("http://localhost:11434/v1".to_string());

        assert!(config.has_llm());
        assert_eq!(config.llm_provider(), Some(LLMProvider::Custom));
    }

    #[test]
    fn test_config_llm_priority() {
        let mut config = AIAgentSystemConfig::mock();
        config.mock_mode = false;
        config.openai_api_key = Some("openai-key".to_string());
        config.anthropic_api_key = Some("anthropic-key".to_string());
        config.google_api_key = Some("google-key".to_string());

        // OpenAI should have highest priority
        assert_eq!(config.llm_provider(), Some(LLMProvider::OpenAI));
    }

    #[test]
    fn test_config_has_vision() {
        let mut config = AIAgentSystemConfig::mock();
        config.mock_mode = false;
        config.openai_api_key = None;
        config.enable_vision = true;
        assert!(!config.has_vision()); // No API key

        config.openai_api_key = Some("test-key".to_string());
        assert!(config.has_vision());

        config.enable_vision = false;
        assert!(!config.has_vision()); // Vision disabled

        config.mock_mode = true;
        config.enable_vision = true;
        config.openai_api_key = None;
        assert!(config.has_vision()); // Mock mode
    }

    #[test]
    fn test_config_has_embeddings() {
        let mut config = AIAgentSystemConfig::mock();
        config.mock_mode = false;
        config.openai_api_key = None;
        assert!(!config.has_embeddings());

        config.openai_api_key = Some("test-key".to_string());
        assert!(config.has_embeddings());

        config.openai_api_key = None;
        config.mock_mode = true;
        assert!(config.has_embeddings());
    }

    #[test]
    fn test_create_llm_client_openai() {
        let mut config = AIAgentSystemConfig::mock();
        config.mock_mode = false;
        config.openai_api_key = Some("test-key".to_string());
        config.openai_model = Some("gpt-4".to_string());

        let client = AIAgentSystem::create_llm_client(&config);
        assert!(client.is_some());
        let client = client.unwrap();
        assert_eq!(client.provider(), LLMProvider::OpenAI);
        assert_eq!(client.model(), "gpt-4");
    }

    #[test]
    fn test_create_llm_client_anthropic() {
        let mut config = AIAgentSystemConfig::mock();
        config.mock_mode = false;
        config.anthropic_api_key = Some("test-key".to_string());
        config.anthropic_model = Some("claude-3-opus".to_string());

        let client = AIAgentSystem::create_llm_client(&config);
        assert!(client.is_some());
        let client = client.unwrap();
        assert_eq!(client.provider(), LLMProvider::Anthropic);
        assert_eq!(client.model(), "claude-3-opus");
    }

    #[test]
    fn test_create_llm_client_google() {
        let mut config = AIAgentSystemConfig::mock();
        config.mock_mode = false;
        config.google_api_key = Some("test-key".to_string());
        config.google_model = Some("gemini-pro".to_string());

        let client = AIAgentSystem::create_llm_client(&config);
        assert!(client.is_some());
        let client = client.unwrap();
        assert_eq!(client.provider(), LLMProvider::Google);
        assert_eq!(client.model(), "gemini-pro");
    }

    #[test]
    fn test_create_llm_client_siliconflow() {
        let mut config = AIAgentSystemConfig::mock();
        config.mock_mode = false;
        config.siliconflow_api_key = Some("test-key".to_string());
        config.siliconflow_model = Some("Qwen/Qwen2.5-7B-Instruct".to_string());

        let client = AIAgentSystem::create_llm_client(&config);
        assert!(client.is_some());
        let client = client.unwrap();
        assert_eq!(client.provider(), LLMProvider::SiliconFlow);
        assert_eq!(client.model(), "Qwen/Qwen2.5-7B-Instruct");
    }

    #[test]
    fn test_create_llm_client_custom() {
        let mut config = AIAgentSystemConfig::mock();
        config.mock_mode = false;
        config.custom_llm_api_base = Some("http://localhost:11434/v1".to_string());
        config.custom_llm_model = Some("llama2".to_string());

        let client = AIAgentSystem::create_llm_client(&config);
        assert!(client.is_some());
        let client = client.unwrap();
        // Custom uses OpenAI-compatible API, so provider is OpenAI
        assert_eq!(client.model(), "llama2");
    }

    #[test]
    fn test_create_llm_client_none() {
        let mut config = AIAgentSystemConfig::mock();
        config.mock_mode = false;
        config.openai_api_key = None;
        config.anthropic_api_key = None;
        config.google_api_key = None;
        config.siliconflow_api_key = None;
        config.custom_llm_api_base = None;

        let client = AIAgentSystem::create_llm_client(&config);
        assert!(client.is_none());
    }
}
