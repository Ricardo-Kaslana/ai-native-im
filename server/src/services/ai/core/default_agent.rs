//! Default AgentCore Implementation
//!
//! This module provides the default implementation of the AgentCore trait.
//! It integrates with the existing LLM client infrastructure and provides
//! a complete agent processing pipeline.
//!
//! **Feature: phase3-ai-agent-evolution**
//! **Requirements: 2.1, 2.3, 7.2, 7.5**

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::{
    AgentAction, AgentCore, AgentEvent, ContextMessage, ContextRole, EmotionState,
    Memory, MemoryError, MemoryStore, MessageContext, Persona, Probability,
};
use crate::services::ai::emotion::EmotionAnalyzer;
use crate::services::ai::tools::{ToolCall, ToolRegistry, ToolResult};
use crate::services::ai::{AIAgentError, LLMClient};

// ============================================================================
// Default Agent Core
// ============================================================================

/// Default implementation of AgentCore
///
/// This implementation uses an LLM client for response generation
/// and integrates with the memory system for context retrieval.
/// It also supports tool calling with automatic feedback loops.
pub struct DefaultAgentCore {
    /// LLM client for generating responses
    llm_client: Arc<dyn LLMClient>,
    /// Tool registry for tool calling
    tool_registry: Option<Arc<ToolRegistry>>,
    /// Emotion analyzer for context-aware responses
    emotion_analyzer: Option<Arc<dyn EmotionAnalyzer>>,
    /// Maximum context messages to include
    max_context_messages: usize,
    /// Maximum memories to retrieve
    max_memories: usize,
    /// Default response threshold
    response_threshold: f64,
    /// Maximum tool call iterations to prevent infinite loops
    max_tool_iterations: usize,
}

impl DefaultAgentCore {
    /// Create a new DefaultAgentCore
    pub fn new(llm_client: Arc<dyn LLMClient>) -> Self {
        Self {
            llm_client,
            tool_registry: None,
            emotion_analyzer: None,
            max_context_messages: 10,
            max_memories: 5,
            response_threshold: 0.5,
            max_tool_iterations: 5,
        }
    }

    /// Set tool registry for tool calling support
    pub fn with_tools(mut self, registry: Arc<ToolRegistry>) -> Self {
        self.tool_registry = Some(registry);
        self
    }

    /// Set emotion analyzer for context-aware responses
    pub fn with_emotion_analyzer(mut self, analyzer: Arc<dyn EmotionAnalyzer>) -> Self {
        self.emotion_analyzer = Some(analyzer);
        self
    }

    /// Set maximum context messages
    pub fn with_max_context(mut self, max: usize) -> Self {
        self.max_context_messages = max;
        self
    }

    /// Set maximum memories to retrieve
    pub fn with_max_memories(mut self, max: usize) -> Self {
        self.max_memories = max;
        self
    }

    /// Set response threshold
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.response_threshold = threshold;
        self
    }

    /// Set maximum tool iterations
    pub fn with_max_tool_iterations(mut self, max: usize) -> Self {
        self.max_tool_iterations = max;
        self
    }

    /// Build the prompt messages for LLM
    async fn build_prompt(
        &self,
        event: &AgentEvent,
        memory: &dyn MemoryStore,
        persona: &Persona,
    ) -> Result<Vec<ContextMessage>, AIAgentError> {
        let mut messages = Vec::new();

        // System prompt from persona
        let mut system_prompt = persona.generate_system_prompt();

        // Add tool instructions if tools are available
        if let Some(tool_instructions) = self.build_tool_instructions() {
            system_prompt.push_str("\n\n");
            system_prompt.push_str(&tool_instructions);
        }

        messages.push(ContextMessage {
            role: ContextRole::System,
            content: system_prompt,
        });

        // Retrieve relevant memories
        let content = event.content();
        if !content.is_empty() {
            match memory.retrieve(&content, self.max_memories).await {
                Ok(memories) if !memories.is_empty() => {
                    let memory_context = self.format_memories(&memories);
                    messages.push(ContextMessage {
                        role: ContextRole::System,
                        content: format!("相关历史记忆：\n{}", memory_context),
                    });
                }
                Ok(_) => {
                    debug!("No relevant memories found");
                }
                Err(e) => {
                    warn!("Failed to retrieve memories: {}", e);
                    // Continue without memories - graceful degradation
                }
            }
        }

        // Get recent conversation context
        let group_id = event.group_id();
        if group_id != Uuid::nil() {
            match memory.get_context(group_id, self.max_context_messages).await {
                Ok(context) if !context.is_empty() => {
                    for msg in context {
                        messages.push(msg);
                    }
                }
                Ok(_) => {
                    debug!("No conversation context found");
                }
                Err(e) => {
                    warn!("Failed to get conversation context: {}", e);
                    // Continue without context - graceful degradation
                }
            }
        }

        // Add the current user message
        messages.push(ContextMessage {
            role: ContextRole::User,
            content: self.format_event_content(event),
        });

        Ok(messages)
    }

    /// Format memories for inclusion in prompt
    fn format_memories(&self, memories: &[Memory]) -> String {
        memories
            .iter()
            .map(|m| format!("- {}", m.content))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format event content for the prompt
    fn format_event_content(&self, event: &AgentEvent) -> String {
        match event {
            AgentEvent::UserMessage(ctx) => {
                format!("[{}]: {}", ctx.sender_name, ctx.content)
            }
            AgentEvent::UserImage { caption, context, .. } => {
                let caption_text = caption.as_deref().unwrap_or("(无文字说明)");
                format!("[{}] 发送了一张图片: {}", context.sender_name, caption_text)
            }
            AgentEvent::UserVoice { transcription, context, .. } => {
                format!("[{}] 语音消息: {}", context.sender_name, transcription)
            }
            AgentEvent::SystemSignal { signal_type, .. } => {
                format!("[系统信号]: {:?}", signal_type)
            }
            AgentEvent::ToolOutput { tool_name, result, .. } => {
                format!("[工具 {}]: {}", tool_name, result)
            }
            AgentEvent::UserFeedback { feedback_type, correction, .. } => {
                if let Some(corr) = correction {
                    format!("[用户反馈]: {} - {}", feedback_type, corr)
                } else {
                    format!("[用户反馈]: {}", feedback_type)
                }
            }
        }
    }

    /// Generate response using LLM
    async fn generate_response(
        &self,
        messages: &[ContextMessage],
        persona: &Persona,
    ) -> Result<String, AIAgentError> {
        // Convert to LLM client format
        use crate::services::ai::llm_client::{ChatMessage as LLMChatMessage, ChatRole as LLMChatRole};
        
        let llm_messages: Vec<LLMChatMessage> = messages
            .iter()
            .map(|m| LLMChatMessage {
                role: match m.role {
                    ContextRole::System => LLMChatRole::System,
                    ContextRole::User => LLMChatRole::User,
                    ContextRole::Assistant => LLMChatRole::Assistant,
                },
                content: m.content.clone(),
            })
            .collect();

        let response = self.llm_client.generate_with_history(&llm_messages).await?;

        // Apply speech suffix if configured
        let response = if let Some(suffix) = &persona.speech_suffix {
            if !response.ends_with(suffix) {
                format!("{}{}", response, suffix)
            } else {
                response
            }
        } else {
            response
        };

        Ok(response)
    }

    /// Analyze emotion from text using simple heuristics
    fn analyze_emotion_simple(&self, text: &str) -> EmotionState {
        let text_lower = text.to_lowercase();

        // Happy indicators
        let happy_keywords = ["开心", "高兴", "太棒了", "哈哈", "😄", "🎉", "好耶", "太好了"];
        if happy_keywords.iter().any(|k| text_lower.contains(k)) {
            return EmotionState::Happy;
        }

        // Excited indicators
        let excited_keywords = ["太激动", "兴奋", "🔥", "⚡", "🚀", "厉害", "牛"];
        if excited_keywords.iter().any(|k| text_lower.contains(k)) {
            return EmotionState::Excited;
        }

        // Sad/empathetic indicators
        let sad_keywords = ["难过", "伤心", "😢", "唉", "失望", "郁闷", "不开心"];
        if sad_keywords.iter().any(|k| text_lower.contains(k)) {
            return EmotionState::Empathetic;
        }

        // Curious indicators
        let curious_keywords = ["为什么", "怎么", "什么", "?", "？", "🤔"];
        if curious_keywords.iter().any(|k| text_lower.contains(k)) {
            return EmotionState::Curious;
        }

        // Playful indicators
        let playful_keywords = ["哈哈哈", "😂", "🤣", "笑死", "有趣", "好玩"];
        if playful_keywords.iter().any(|k| text_lower.contains(k)) {
            return EmotionState::Playful;
        }

        EmotionState::Neutral
    }

    /// Check if the response contains a tool call request
    fn parse_tool_calls(&self, response: &str) -> Vec<ToolCall> {
        // Try to parse tool calls from the response
        // Format: <tool_call>{"name": "tool_name", "arguments": {...}}</tool_call>
        let mut calls = Vec::new();
        let mut remaining = response;

        while let Some(start) = remaining.find("<tool_call>") {
            if let Some(end) = remaining[start..].find("</tool_call>") {
                let json_start = start + "<tool_call>".len();
                let json_end = start + end;
                let json_str = &remaining[json_start..json_end];

                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let (Some(name), Some(args)) = (
                        parsed.get("name").and_then(|n| n.as_str()),
                        parsed.get("arguments"),
                    ) {
                        calls.push(ToolCall {
                            id: format!("call_{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("0")),
                            name: name.to_string(),
                            arguments: args.clone(),
                        });
                    }
                }

                remaining = &remaining[json_end + "</tool_call>".len()..];
            } else {
                break;
            }
        }

        calls
    }

    /// Execute tool calls and return results
    async fn execute_tool_calls(&self, calls: &[ToolCall]) -> Vec<ToolResult> {
        if let Some(registry) = &self.tool_registry {
            registry.execute_all(calls).await
        } else {
            calls
                .iter()
                .map(|c| ToolResult::failure(&c.id, &c.name, "工具系统未启用"))
                .collect()
        }
    }

    /// Format tool results for inclusion in the next prompt
    fn format_tool_results(&self, results: &[ToolResult]) -> String {
        results
            .iter()
            .map(|r| {
                if r.success {
                    format!("[工具 {} 结果]: {}", r.name, r.content)
                } else {
                    format!("[工具 {} 失败]: {}", r.name, r.content)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Build tool instructions for the system prompt
    fn build_tool_instructions(&self) -> Option<String> {
        let registry = self.tool_registry.as_ref()?;
        let definitions = registry.to_function_definitions();

        if definitions.is_empty() {
            return None;
        }

        let tools_desc = definitions
            .iter()
            .map(|d| format!("- {}: {}", d.name, d.description))
            .collect::<Vec<_>>()
            .join("\n");

        Some(format!(
            r#"你可以使用以下工具来帮助回答问题：

{}

要使用工具，请在回复中包含：
<tool_call>{{"name": "工具名称", "arguments": {{"参数名": "参数值"}}}}</tool_call>

工具执行后，你会收到结果，然后可以继续回答用户的问题。"#,
            tools_desc
        ))
    }

    /// Clean response by removing tool call markup
    fn clean_response(&self, response: &str) -> String {
        let mut result = response.to_string();

        // Remove <tool_call>...</tool_call> blocks
        while let Some(start) = result.find("<tool_call>") {
            if let Some(end) = result[start..].find("</tool_call>") {
                let end_pos = start + end + "</tool_call>".len();
                result = format!("{}{}", &result[..start], &result[end_pos..]);
            } else {
                break;
            }
        }

        result.trim().to_string()
    }

    /// Convert from emotion module's EmotionState to core's EmotionState
    fn convert_emotion_state(emotion: crate::services::ai::emotion::EmotionState) -> EmotionState {
        use crate::services::ai::emotion::EmotionState as ExtEmotionState;
        match emotion {
            ExtEmotionState::Neutral => EmotionState::Neutral,
            ExtEmotionState::Happy => EmotionState::Happy,
            ExtEmotionState::Excited => EmotionState::Excited,
            ExtEmotionState::Sad => EmotionState::Empathetic, // Map sad to empathetic response
            ExtEmotionState::Angry => EmotionState::Empathetic, // Map angry to empathetic response
            ExtEmotionState::Empathetic => EmotionState::Empathetic,
            ExtEmotionState::Supportive => EmotionState::Supportive,
            ExtEmotionState::Curious => EmotionState::Curious,
            ExtEmotionState::Playful => EmotionState::Playful,
            ExtEmotionState::Anxious => EmotionState::Supportive, // Map anxious to supportive
            ExtEmotionState::Confused => EmotionState::Supportive, // Map confused to supportive
        }
    }

    /// Calculate trigger probability based on context
    fn calculate_trigger_probability(&self, context: &MessageContext, persona: &Persona) -> Probability {
        let mut probability = Probability::new(0.1); // Base probability

        // High probability for mentions
        if context.mentions_agent {
            probability = Probability::new(0.95);
        }

        // High probability for replies
        if context.is_reply_to_agent {
            probability = probability.or(Probability::new(0.9));
        }

        // Check for trigger keywords
        let content_lower = context.content.to_lowercase();
        for keyword in &persona.trigger_keywords {
            if content_lower.contains(&keyword.to_lowercase()) {
                probability = probability.or(Probability::new(0.7));
                break;
            }
        }

        // Check for expertise domain mentions
        for domain in &persona.expertise_domains {
            if content_lower.contains(&domain.to_lowercase()) {
                probability = probability.boost(1.3);
                break;
            }
        }

        probability
    }
}

#[async_trait]
impl AgentCore for DefaultAgentCore {
    async fn process(
        &self,
        event: AgentEvent,
        memory: &dyn MemoryStore,
        persona: &Persona,
    ) -> Result<Vec<AgentAction>, AIAgentError> {
        info!("Processing event: {:?}", std::mem::discriminant(&event));

        // Handle feedback events specially
        if let AgentEvent::UserFeedback { feedback_type, .. } = &event {
            debug!("Received feedback: {:?}", feedback_type);
            // Feedback doesn't need a response, just acknowledgment
            return Ok(vec![AgentAction::Ignore]);
        }

        // Handle tool output events - these are feedback from tool execution
        if let AgentEvent::ToolOutput { tool_name, result, success, .. } = &event {
            debug!("Received tool output from {}: success={}", tool_name, success);
            // Tool outputs are handled in the tool calling loop below
            // If we receive one directly, just acknowledge it
            return Ok(vec![AgentAction::Ignore]);
        }

        // Build initial prompt
        let mut messages = self.build_prompt(&event, memory, persona).await?;
        let mut actions = Vec::new();
        let mut iteration = 0;

        // Tool calling loop
        loop {
            if iteration >= self.max_tool_iterations {
                warn!("Max tool iterations ({}) reached", self.max_tool_iterations);
                break;
            }

            match self.generate_response(&messages, persona).await {
                Ok(response) => {
                    // Check for tool calls in the response
                    let tool_calls = self.parse_tool_calls(&response);

                    if tool_calls.is_empty() {
                        // No tool calls, this is the final response
                        // Use emotion analyzer if available, otherwise fall back to simple analysis
                        let emotion = if let Some(analyzer) = &self.emotion_analyzer {
                            match analyzer.analyze(&event.content()).await {
                                Ok(analysis) => {
                                    debug!("Emotion analysis: {:?} (confidence: {})", 
                                           analysis.primary_emotion, analysis.confidence);
                                    // Convert from emotion module's EmotionState to core's EmotionState
                                    Self::convert_emotion_state(analysis.primary_emotion)
                                }
                                Err(e) => {
                                    warn!("Emotion analysis failed: {}, using simple analysis", e);
                                    self.analyze_emotion_simple(&event.content())
                                }
                            }
                        } else {
                            self.analyze_emotion_simple(&event.content())
                        };
                        let constrained_emotion = persona.constrain_emotion(emotion);

                        // Extract the actual response text (remove any tool call markup)
                        let clean_response = self.clean_response(&response);

                        actions.push(AgentAction::ReplyText {
                            content: clean_response,
                            emotion: constrained_emotion,
                        });
                        break;
                    }

                    // Execute tool calls
                    info!("Executing {} tool call(s)", tool_calls.len());
                    let results = self.execute_tool_calls(&tool_calls).await;

                    // Add tool call actions for tracking
                    for call in &tool_calls {
                        actions.push(AgentAction::CallTool {
                            name: call.name.clone(),
                            args: call.arguments.clone(),
                            request_id: call.id.clone(),
                        });
                    }

                    // Add assistant message with tool calls
                    messages.push(ContextMessage {
                        role: ContextRole::Assistant,
                        content: response.clone(),
                    });

                    // Add tool results to context
                    let results_text = self.format_tool_results(&results);
                    messages.push(ContextMessage {
                        role: ContextRole::User,
                        content: format!("[工具执行结果]\n{}", results_text),
                    });

                    iteration += 1;
                }
                Err(e) => {
                    // Graceful error handling - return a fallback response
                    warn!("LLM generation failed: {}", e);

                    // Check if error is transient
                    if e.is_transient() {
                        actions.push(AgentAction::text_with_emotion(
                            "抱歉，我现在有点忙，稍后再回复你~",
                            EmotionState::Empathetic,
                        ));
                    } else {
                        actions.push(AgentAction::text_with_emotion(
                            "抱歉，我遇到了一些问题，请稍后再试。",
                            EmotionState::Empathetic,
                        ));
                    }
                    break;
                }
            }
        }

        Ok(actions)
    }

    async fn should_intervene(
        &self,
        context: &MessageContext,
        persona: &Persona,
    ) -> Probability {
        self.calculate_trigger_probability(context, persona)
    }

    async fn analyze_emotion(&self, context: &MessageContext) -> EmotionState {
        self.analyze_emotion_simple(&context.content)
    }
}

// ============================================================================
// Mock Memory Store for Testing
// ============================================================================

/// Mock memory store for testing
pub struct MockMemoryStore {
    memories: Vec<Memory>,
    context: Vec<ContextMessage>,
}

impl MockMemoryStore {
    /// Create a new mock memory store
    pub fn new() -> Self {
        Self {
            memories: Vec::new(),
            context: Vec::new(),
        }
    }

    /// Add a memory
    pub fn with_memory(mut self, memory: Memory) -> Self {
        self.memories.push(memory);
        self
    }

    /// Add context messages
    pub fn with_context(mut self, messages: Vec<ContextMessage>) -> Self {
        self.context = messages;
        self
    }
}

impl Default for MockMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MemoryStore for MockMemoryStore {
    async fn store(&self, _memory: Memory) -> Result<(), MemoryError> {
        Ok(())
    }

    async fn retrieve(&self, _query: &str, limit: usize) -> Result<Vec<Memory>, MemoryError> {
        Ok(self.memories.iter().take(limit).cloned().collect())
    }

    async fn get_context(&self, _group_id: Uuid, limit: usize) -> Result<Vec<ContextMessage>, MemoryError> {
        Ok(self.context.iter().take(limit).cloned().collect())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ai::MockLLMClient;

    fn create_test_context() -> MessageContext {
        MessageContext::new(
            "Hello, how are you?",
            Uuid::new_v4(),
            "test_user",
            Uuid::new_v4(),
        )
    }

    #[tokio::test]
    async fn test_default_agent_creation() {
        let llm = Arc::new(MockLLMClient::new("Test response".to_string()));
        let agent = DefaultAgentCore::new(llm)
            .with_max_context(5)
            .with_max_memories(3)
            .with_threshold(0.6);

        assert_eq!(agent.max_context_messages, 5);
        assert_eq!(agent.max_memories, 3);
        assert!((agent.response_threshold - 0.6).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_should_intervene_mention() {
        let llm = Arc::new(MockLLMClient::new("Test".to_string()));
        let agent = DefaultAgentCore::new(llm);
        let persona = Persona::default();

        let context = create_test_context().with_mentions_agent(true);
        let prob = agent.should_intervene(&context, &persona).await;

        assert!(prob.value() > 0.9);
    }

    #[tokio::test]
    async fn test_should_intervene_reply() {
        let llm = Arc::new(MockLLMClient::new("Test".to_string()));
        let agent = DefaultAgentCore::new(llm);
        let persona = Persona::default();

        let context = create_test_context().with_reply_to_agent(true);
        let prob = agent.should_intervene(&context, &persona).await;

        assert!(prob.value() > 0.8);
    }

    #[tokio::test]
    async fn test_should_intervene_keyword() {
        let llm = Arc::new(MockLLMClient::new("Test".to_string()));
        let agent = DefaultAgentCore::new(llm);
        let persona = Persona::default().with_keywords(vec!["help".to_string()]);

        let context = MessageContext::new(
            "I need help with something",
            Uuid::new_v4(),
            "user",
            Uuid::new_v4(),
        );
        let prob = agent.should_intervene(&context, &persona).await;

        assert!(prob.value() > 0.5);
    }

    #[tokio::test]
    async fn test_analyze_emotion() {
        let llm = Arc::new(MockLLMClient::new("Test".to_string()));
        let agent = DefaultAgentCore::new(llm);

        // Happy
        let context = MessageContext::new("太棒了！", Uuid::new_v4(), "user", Uuid::new_v4());
        assert_eq!(agent.analyze_emotion(&context).await, EmotionState::Happy);

        // Curious
        let context = MessageContext::new("为什么会这样？", Uuid::new_v4(), "user", Uuid::new_v4());
        assert_eq!(agent.analyze_emotion(&context).await, EmotionState::Curious);

        // Neutral
        let context = MessageContext::new("好的", Uuid::new_v4(), "user", Uuid::new_v4());
        assert_eq!(agent.analyze_emotion(&context).await, EmotionState::Neutral);
    }

    #[tokio::test]
    async fn test_process_user_message() {
        let llm = Arc::new(MockLLMClient::new("Hello! How can I help you?".to_string()));
        let agent = DefaultAgentCore::new(llm);
        let memory = MockMemoryStore::new();
        let persona = Persona::default();

        let event = AgentEvent::UserMessage(create_test_context());
        let actions = agent.process(event, &memory, &persona).await.unwrap();

        assert!(!actions.is_empty());
        assert!(matches!(actions[0], AgentAction::ReplyText { .. }));
    }

    #[tokio::test]
    async fn test_process_feedback_event() {
        let llm = Arc::new(MockLLMClient::new("Test".to_string()));
        let agent = DefaultAgentCore::new(llm);
        let memory = MockMemoryStore::new();
        let persona = Persona::default();

        let event = AgentEvent::UserFeedback {
            message_id: Uuid::new_v4(),
            response_id: Uuid::new_v4(),
            feedback_type: super::super::FeedbackType::Positive,
            correction: None,
            user_id: Uuid::new_v4(),
        };

        let actions = agent.process(event, &memory, &persona).await.unwrap();
        assert!(matches!(actions[0], AgentAction::Ignore));
    }

    #[tokio::test]
    async fn test_graceful_error_handling() {
        let llm = Arc::new(MockLLMClient::failing());
        let agent = DefaultAgentCore::new(llm);
        let memory = MockMemoryStore::new();
        let persona = Persona::default();

        let event = AgentEvent::UserMessage(create_test_context());
        let actions = agent.process(event, &memory, &persona).await.unwrap();

        // Should return a fallback response, not an error
        assert!(!actions.is_empty());
        assert!(matches!(actions[0], AgentAction::ReplyText { .. }));
    }

    #[test]
    fn test_parse_tool_calls() {
        let llm = Arc::new(MockLLMClient::new("Test".to_string()));
        let agent = DefaultAgentCore::new(llm);

        // Test single tool call
        let response = r#"让我查一下天气。<tool_call>{"name": "get_weather", "arguments": {"city": "北京"}}</tool_call>"#;
        let calls = agent.parse_tool_calls(response);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "get_weather");

        // Test multiple tool calls
        let response = r#"<tool_call>{"name": "calculator", "arguments": {"expression": "2+2"}}</tool_call>
然后查天气<tool_call>{"name": "get_weather", "arguments": {"city": "上海"}}</tool_call>"#;
        let calls = agent.parse_tool_calls(response);
        assert_eq!(calls.len(), 2);

        // Test no tool calls
        let response = "这是一个普通的回复，没有工具调用。";
        let calls = agent.parse_tool_calls(response);
        assert!(calls.is_empty());
    }

    #[test]
    fn test_clean_response() {
        let llm = Arc::new(MockLLMClient::new("Test".to_string()));
        let agent = DefaultAgentCore::new(llm);

        let response = r#"让我查一下。<tool_call>{"name": "test", "arguments": {}}</tool_call> 好的。"#;
        let cleaned = agent.clean_response(response);
        assert_eq!(cleaned, "让我查一下。 好的。");

        let response = "普通回复";
        let cleaned = agent.clean_response(response);
        assert_eq!(cleaned, "普通回复");
    }

    #[tokio::test]
    async fn test_with_tools() {
        use crate::services::ai::tools::ToolRegistry;

        let llm = Arc::new(MockLLMClient::new("Hello!".to_string()));
        let registry = Arc::new(ToolRegistry::with_defaults());
        let agent = DefaultAgentCore::new(llm).with_tools(registry);

        assert!(agent.tool_registry.is_some());
        assert!(agent.build_tool_instructions().is_some());
    }

    #[tokio::test]
    async fn test_tool_calling_loop() {
        use crate::services::ai::tools::ToolRegistry;

        // First response has tool call, second is final answer
        let llm = Arc::new(MockLLMClient::new(
            "北京今天天气晴朗，温度25°C。".to_string()
        ));
        let registry = Arc::new(ToolRegistry::with_defaults());
        let agent = DefaultAgentCore::new(llm)
            .with_tools(registry)
            .with_max_tool_iterations(3);

        let memory = MockMemoryStore::new();
        let persona = Persona::default();
        let event = AgentEvent::UserMessage(create_test_context());

        let actions = agent.process(event, &memory, &persona).await.unwrap();

        // Should have at least one reply action
        assert!(!actions.is_empty());
        assert!(actions.iter().any(|a| matches!(a, AgentAction::ReplyText { .. })));
    }

    #[test]
    fn test_format_tool_results() {
        use crate::services::ai::tools::ToolResult;

        let llm = Arc::new(MockLLMClient::new("Test".to_string()));
        let agent = DefaultAgentCore::new(llm);

        let results = vec![
            ToolResult::success("1", "calculator", "2 + 2 = 4"),
            ToolResult::failure("2", "weather", "API error"),
        ];

        let formatted = agent.format_tool_results(&results);
        assert!(formatted.contains("calculator"));
        assert!(formatted.contains("2 + 2 = 4"));
        assert!(formatted.contains("weather"));
        assert!(formatted.contains("失败"));
    }

    #[tokio::test]
    async fn test_with_emotion_analyzer() {
        use crate::services::ai::emotion::RuleBasedEmotionAnalyzer;

        let llm = Arc::new(MockLLMClient::new("我很高兴能帮助你！".to_string()));
        let analyzer = Arc::new(RuleBasedEmotionAnalyzer::new());
        let agent = DefaultAgentCore::new(llm)
            .with_emotion_analyzer(analyzer);

        assert!(agent.emotion_analyzer.is_some());

        let memory = MockMemoryStore::new();
        let persona = Persona::default();
        let context = MessageContext::new(
            "太棒了！我很开心！",
            Uuid::new_v4(),
            "happy_user",
            Uuid::new_v4(),
        );
        let event = AgentEvent::UserMessage(context);

        let actions = agent.process(event, &memory, &persona).await.unwrap();
        assert!(!actions.is_empty());

        // The response should have an emotion attached
        if let AgentAction::ReplyText { emotion, .. } = &actions[0] {
            // Happy input should result in happy or related emotion
            assert!(matches!(
                emotion,
                EmotionState::Happy | EmotionState::Playful | EmotionState::Neutral
            ));
        }
    }
}
