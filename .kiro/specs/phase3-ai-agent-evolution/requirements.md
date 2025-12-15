# Requirements Document - Phase 3: AI Agent Evolution

## Introduction

Phase 3 focuses on evolving the AI Agent system from a simple LLM wrapper to a true "Cognitive Architecture" with human-like capabilities. The architecture follows a "Brain, Senses, and Limbs" (大脑、感官与手脚) design pattern, enabling rich multi-modal interactions beyond text. The system emphasizes three core principles: **High Extensibility** (高可扩展性), **High Robustness** (高稳健性), and **High Independence** (高独立性).

## Glossary

- **AgentCore**: The central trait defining AI Agent behavior (the "Brain"), including perception, decision-making, and action execution
- **Perception Layer (感知层)**: Components that process multi-modal inputs (text, image, audio) into structured data the agent can understand
- **Cognition Layer (认知层)**: The decision-making core that determines responses, tool usage, and conversation strategies
- **Action Layer (执行层)**: Components that execute agent decisions (send messages, call tools, generate media)
- **Memory System**: Short-term (conversation context) and long-term (RAG-based vector store) memory
- **AgentEvent**: Multi-modal input events (user messages, system signals, tool outputs, audio transcriptions)
- **AgentAction**: Output actions (reply text, reply audio, reply image, call tool, ignore, summarize)
- **Persona**: Personality configuration including system prompts, response style, expertise areas, and voice settings
- **TTS (Text-to-Speech)**: Service that converts text responses to audio
- **STT (Speech-to-Text)**: Service that transcribes audio messages to text (Whisper)
- **Semantic Search**: Vector-based search that finds conceptually related content
- **Emotion State (情绪状态)**: The agent's current emotional context that influences response style and expression
- **Self-Learning (自我学习)**: The agent's ability to improve responses based on user feedback and interaction patterns
- **Provider Priority (服务商优先级)**: Configuration that prioritizes domestic (Chinese) AI service providers for better latency and compliance

## Requirements

### Requirement 1: Layered Agent Architecture

**User Story:** As a developer, I want a clearly layered agent architecture, so that each component can evolve independently without affecting others.

#### Acceptance Criteria

1. WHEN implementing the agent system THEN the Architecture SHALL separate into three layers: Perception, Cognition, and Action
2. WHEN the Perception layer processes input THEN the Perception Layer SHALL convert raw input (text, image, audio) into structured AgentEvent
3. WHEN the Cognition layer makes decisions THEN the Cognition Layer SHALL depend only on AgentEvent, not raw input formats
4. WHEN the Action layer executes THEN the Action Layer SHALL translate AgentAction into concrete operations (WebSocket messages, API calls)
5. WHEN a new input modality is added THEN the Developer SHALL only modify the Perception layer without touching Cognition or Action

### Requirement 2: AgentCore Trait Abstraction

**User Story:** As a developer, I want a unified AgentCore trait, so that I can implement different AI agent behaviors without changing business logic.

#### Acceptance Criteria

1. WHEN implementing an AI agent THEN the Developer SHALL implement the AgentCore trait with `process()` and `should_intervene()` methods
2. WHEN the agent receives an event THEN the AgentCore SHALL return a list of AgentAction variants (ReplyText, ReplyAudio, ReplyImage, CallTool, Ignore, Summarize)
3. WHEN the business layer calls the agent THEN the Business Layer SHALL only depend on the AgentCore trait, not concrete implementations
4. WHEN a new LLM provider is added THEN the Developer SHALL only need to implement the LLMClient trait without modifying AgentCore
5. WHEN the agent encounters an error THEN the AgentCore SHALL return a graceful fallback action instead of propagating the error

### Requirement 3: Speech-to-Text (STT) Integration

**User Story:** As a user, I want the AI agent to understand my voice messages, so that I don't have to type everything.

#### Acceptance Criteria

1. WHEN a user sends a voice message THEN the STT Service SHALL transcribe it to text using Whisper API or whisper.cpp
2. WHEN transcription completes THEN the System SHALL display the transcribed text alongside the audio message
3. WHEN transcription fails THEN the System SHALL notify the user and allow manual retry
4. WHEN the audio is too long (>5 minutes) THEN the System SHALL split it into chunks for processing
5. WHEN the STT service is unavailable THEN the Agent SHALL inform the user that voice messages cannot be processed temporarily

### Requirement 4: Text-to-Speech (TTS) Integration

**User Story:** As a user, I want to hear the AI agent's responses as audio, so that I can listen while doing other things.

#### Acceptance Criteria

1. WHEN the agent generates a text response THEN the System MAY also generate an audio version using TTS (OpenAI TTS or Edge TTS)
2. WHEN displaying an AI message THEN the Client SHALL show a play button to listen to the audio version
3. WHEN TTS generation fails THEN the System SHALL display text-only response without error
4. WHEN the user enables "voice mode" THEN the Agent SHALL automatically generate audio for all responses
5. WHEN generating audio THEN the TTS Service SHALL use the persona's configured voice settings

### Requirement 5: Intelligent Message Summary

**User Story:** As a user who missed many messages, I want to get a quick summary, so that I can catch up without reading everything.

#### Acceptance Criteria

1. WHEN a user requests a summary THEN the SummarizeService SHALL read the last N messages (configurable, default 100)
2. WHEN generating a summary THEN the Service SHALL produce a concise overview including main topics, key decisions, and notable events
3. WHEN the summary is generated THEN the System SHALL cache it for subsequent requests within a time window
4. WHEN the group has no recent messages THEN the Service SHALL return "No recent activity to summarize"
5. WHEN summarization fails THEN the Service SHALL return a partial summary or error message gracefully

### Requirement 6: Semantic Search

**User Story:** As a user, I want to search messages by meaning, so that I can find content even when I don't remember exact words.

#### Acceptance Criteria

1. WHEN a message is stored THEN the System SHALL asynchronously generate and store its embedding in the vector database
2. WHEN a user performs a semantic search THEN the Search Service SHALL query the vector database for similar content
3. WHEN displaying search results THEN the System SHALL show messages ranked by semantic similarity
4. WHEN the vector database is unavailable THEN the System SHALL fall back to traditional text search
5. WHEN embedding generation fails for a message THEN the System SHALL log the error and continue without blocking message storage

### Requirement 7: Tool Calling System

**User Story:** As a user, I want the AI agent to call external tools like weather API, so that it can provide real-time information.

#### Acceptance Criteria

1. WHEN the agent decides to call a tool THEN the Agent SHALL return AgentAction::CallTool with tool name and arguments
2. WHEN a tool call completes THEN the System SHALL feed the result back to the agent as AgentEvent::ToolOutput
3. WHEN a tool call fails THEN the Agent SHALL handle the error gracefully and inform the user
4. WHEN defining a new tool THEN the Developer SHALL implement the Tool trait with `name()`, `description()`, and `execute()` methods
5. WHEN the agent generates a response THEN the Agent SHALL be able to chain multiple tool calls before final reply

### Requirement 8: Persona System

**User Story:** As a group admin, I want to customize the AI agent's personality, so that it fits the group's culture and purpose.

#### Acceptance Criteria

1. WHEN creating an AI agent THEN the Admin SHALL be able to set personality parameters (toxicity, enthusiasm, expertise, voice)
2. WHEN the agent generates a response THEN the Agent SHALL incorporate the persona's system prompt and style guidelines
3. WHEN the persona is updated THEN the Agent SHALL immediately reflect the new personality in subsequent responses
4. WHEN defining expertise areas THEN the Agent SHALL prioritize responding to messages matching those areas
5. WHEN the persona includes voice settings THEN the TTS Service SHALL use those settings for audio generation

### Requirement 9: Intelligent Response Decision

**User Story:** As a user, I want the AI agent to decide intelligently whether to respond, so that it doesn't spam the chat with unnecessary messages.

#### Acceptance Criteria

1. WHEN a message is received THEN the TriggerEvaluator SHALL calculate a response probability based on context
2. WHEN the message mentions the agent by name THEN the TriggerEvaluator SHALL return high probability (>0.9)
3. WHEN the message contains configured keywords THEN the TriggerEvaluator SHALL increase response probability
4. WHEN the agent has recently responded THEN the TriggerEvaluator SHALL decrease probability to avoid over-responding
5. WHEN the message is a direct reply to the agent THEN the TriggerEvaluator SHALL return high probability

### Requirement 10: Memory System

**User Story:** As a user, I want the AI agent to remember past conversations, so that it can provide contextually relevant responses.

#### Acceptance Criteria

1. WHEN processing a message THEN the Agent SHALL retrieve relevant memories from the vector store
2. WHEN generating a response THEN the Agent SHALL incorporate retrieved memories into the context
3. WHEN a conversation ends THEN the Agent SHALL store important information in long-term memory
4. WHEN memory storage fails THEN the Agent SHALL continue functioning with short-term context only
5. WHEN memory retrieval is slow THEN the Agent SHALL proceed with available context and append memories asynchronously

### Requirement 11: Proactive Messaging

**User Story:** As a group member, I want the AI agent to occasionally initiate conversations, so that the group stays active and engaging.

#### Acceptance Criteria

1. WHEN the group has been silent for a configurable duration THEN the Agent MAY send a proactive message
2. WHEN sending a proactive message THEN the Agent SHALL consider the group's activity patterns and time of day
3. WHEN proactive messaging is disabled THEN the Agent SHALL never initiate conversations
4. WHEN the agent sends a proactive message THEN the System SHALL record it to prevent excessive messaging

### Requirement 12: Rate Limiting and Cost Control

**User Story:** As a system administrator, I want to limit AI API usage, so that costs remain manageable.

#### Acceptance Criteria

1. WHEN a user sends a message to the agent THEN the System SHALL check the user's remaining quota
2. WHEN the quota is exceeded THEN the System SHALL return a friendly message explaining the limit
3. WHEN tracking usage THEN the System SHALL record token counts per user and per group
4. WHEN configuring limits THEN the Admin SHALL be able to set daily/monthly token limits per user
5. WHEN the STT/TTS services are used THEN the System SHALL track audio processing costs separately

### Requirement 13: Response Streaming

**User Story:** As a user, I want to see the AI's response appear gradually, so that I know it's working and can read as it types.

#### Acceptance Criteria

1. WHEN the agent generates a long response THEN the System SHALL stream the response to the client
2. WHEN streaming THEN the Client SHALL display text as it arrives with a typing animation
3. WHEN streaming is interrupted THEN the System SHALL gracefully handle the partial response
4. WHEN the user sends a new message during streaming THEN the System SHALL queue it for processing after completion

### Requirement 14: Multi-Modal Input Processing

**User Story:** As a user, I want the AI agent to understand images and other media, so that I can have richer conversations.

#### Acceptance Criteria

1. WHEN a user sends an image with text THEN the Agent SHALL process both the image content and text together
2. WHEN processing an image THEN the Agent SHALL use the VisionModel to generate a description
3. WHEN the image processing fails THEN the Agent SHALL gracefully degrade to text-only response
4. WHEN multiple images are sent THEN the Agent SHALL process them in order and combine insights

### Requirement 15: Sticker and Emoji Support

**User Story:** As a user, I want the AI agent to send stickers and emojis, so that conversations feel more natural and fun.

#### Acceptance Criteria

1. WHEN the agent decides to send a sticker THEN the Agent SHALL return AgentAction::ReplySticker with sticker ID
2. WHEN generating text responses THEN the Agent MAY include appropriate emojis based on sentiment
3. WHEN a sticker is requested but not found THEN the Agent SHALL fall back to an emoji or text description
4. WHEN the persona has sticker preferences THEN the Agent SHALL prioritize those stickers

### Requirement 16: Multi-Provider Support with Domestic Priority

**User Story:** As a system administrator, I want to configure multiple AI service providers with domestic (Chinese) providers as priority, so that I can ensure low latency and compliance.

#### Acceptance Criteria

1. WHEN configuring LLM providers THEN the System SHALL support domestic providers including SiliconFlow (硅基流动), Zhipu AI (智谱), Baidu Wenxin (文心一言), Alibaba Tongyi (通义千问), and Moonshot (月之暗面)
2. WHEN selecting a provider THEN the System SHALL prioritize domestic providers by default for users in China
3. WHEN a primary provider fails THEN the System SHALL automatically fallback to the next provider in the priority list
4. WHEN configuring embedding services THEN the System SHALL support domestic embedding APIs (SiliconFlow, Zhipu)
5. WHEN configuring TTS/STT services THEN the System SHALL support domestic alternatives (Azure China, Alibaba TTS)
6. WHEN adding a new provider THEN the Developer SHALL only need to implement the LLMClient trait without modifying core logic

### Requirement 17: Emotion-Driven Expression

**User Story:** As a user, I want the AI agent to express emotions naturally, so that conversations feel more human-like and engaging.

#### Acceptance Criteria

1. WHEN processing a message THEN the Agent SHALL analyze the emotional context of the conversation
2. WHEN the conversation is happy/excited THEN the Agent SHALL use more enthusiastic language and emojis
3. WHEN the conversation is sad/serious THEN the Agent SHALL use more empathetic and supportive language
4. WHEN the user expresses frustration THEN the Agent SHALL acknowledge the emotion before providing help
5. WHEN the emotion state changes THEN the Agent SHALL smoothly transition its expression style
6. WHEN the persona defines emotion preferences THEN the Agent SHALL respect those boundaries (e.g., always calm, never angry)

### Requirement 18: Self-Learning and Improvement

**User Story:** As a user, I want the AI agent to learn from our interactions, so that it becomes more helpful over time.

#### Acceptance Criteria

1. WHEN a user provides positive feedback (thumbs up, "thanks") THEN the System SHALL record the successful interaction pattern
2. WHEN a user provides negative feedback (thumbs down, "wrong") THEN the System SHALL record the failed pattern for improvement
3. WHEN generating responses THEN the Agent SHALL consider past successful patterns for similar contexts
4. WHEN the agent makes repeated mistakes THEN the System SHALL flag the pattern for review
5. WHEN learning data is stored THEN the System SHALL anonymize and aggregate patterns to protect privacy
6. WHEN the learning system is disabled THEN the Agent SHALL function normally without personalization

