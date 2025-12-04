# Requirements Document

## Introduction

本文档定义了 AI 原生即时通讯应用中的 AI Agent 核心系统需求。该系统旨在创建一个"真实群友"体验的 AI Bot，而非传统的工具型聊天机器人。AI Agent 具备长期记忆、个性化人设、智能触发机制和多模态交互能力，能够像真实用户一样自然地参与群聊对话。

## Glossary

- **AI Agent**: 在群聊中具备拟人化特征、记忆能力和主动交互能力的智能机器人实体
- **RAG System**: 检索增强生成系统（Retrieval-Augmented Generation），通过向量数据库检索历史对话来增强 AI 回复的上下文相关性
- **Vector Database**: 向量数据库，用于存储和检索对话的语义向量表示，支持相似度搜索
- **Personality System**: 人设系统，定义 AI Agent 的性格特征、说话风格和行为模式的配置系统
- **Trigger Logic**: 触发逻辑，决定 AI Agent 何时应该参与对话的判断机制
- **System Prompt**: 系统提示词，用于指导大语言模型生成符合特定人设和上下文的回复的指令模板
- **Embedding**: 嵌入向量，将文本转换为高维向量表示以支持语义搜索
- **Message Pipeline**: 消息处理管道，群聊消息从接收到处理的完整流程
- **Typing Indicator**: 正在输入指示器，显示用户或 AI 正在输入消息的状态提示
- **Group Chat**: 群聊，多个用户和可选的 AI Agent 参与的聊天会话
- **Context Window**: 上下文窗口，AI 生成回复时考虑的历史消息范围

## Requirements

### Requirement 1

**User Story:** 作为群主，我希望 AI Agent 能够记住之前的对话内容，以便它能够在后续对话中引用历史信息并保持上下文连贯性。

#### Acceptance Criteria

1. WHEN a user sends a message in a group chat THEN the AI Agent System SHALL store the message embedding in the Vector Database within 500 milliseconds
2. WHEN the AI Agent generates a response THEN the AI Agent System SHALL retrieve the top 10 most semantically similar historical messages from the Vector Database
3. WHEN retrieved historical messages are older than 30 days THEN the AI Agent System SHALL apply a time-decay weight factor between 0.5 and 1.0 to their relevance scores
4. WHEN the AI Agent references historical context THEN the AI Agent System SHALL include at least one retrieved message in the context window provided to the language model
5. WHEN the Vector Database query fails THEN the AI Agent System SHALL fall back to generating responses using only the current conversation context

### Requirement 2

**User Story:** 作为群主，我希望能够高度定制 AI Agent 的性格和说话风格，以便它能够符合群聊的氛围和我的偏好。

#### Acceptance Criteria

1. WHEN a group owner creates an AI Agent THEN the Personality System SHALL allow configuration of at least 5 personality parameters including toxicity level, enthusiasm level, expertise domain, speech patterns, and user affinity
2. WHEN personality parameters are modified THEN the Personality System SHALL update the System Prompt template within 100 milliseconds
3. WHEN the AI Agent generates a response THEN the Personality System SHALL inject the configured personality traits into the System Prompt
4. WHILE the AI Agent is active THEN the Personality System SHALL maintain consistency in speech patterns across all generated responses
5. WHEN a speech pattern includes suffix markers THEN the Personality System SHALL append the configured suffix to at least 70% of generated messages

### Requirement 3

**User Story:** 作为群成员，我希望 AI Agent 不会对每条消息都回复，以便群聊保持自然的对话节奏而不被 AI 打断。

#### Acceptance Criteria

1. WHEN a message is sent in a group chat THEN the Trigger Logic SHALL evaluate whether the AI Agent should respond based on multiple trigger conditions
2. WHEN the AI Agent is explicitly mentioned using @ syntax THEN the Trigger Logic SHALL set trigger probability to 100%
3. WHEN a message matches configured keyword patterns THEN the Trigger Logic SHALL set trigger probability between 60% and 80%
4. WHEN the Vector Database similarity score exceeds 0.8 for retrieved context THEN the Trigger Logic SHALL set trigger probability between 40% and 60%
5. WHEN no specific trigger conditions are met THEN the Trigger Logic SHALL set trigger probability between 1% and 5% for random interjection
6. WHEN the group chat has been silent for more than 2 hours THEN the Trigger Logic SHALL increase random interjection probability to between 10% and 20%

### Requirement 4

**User Story:** 作为群成员，我希望 AI Agent 的回复看起来像真人在打字，以便增强真实感和沉浸式体验。

#### Acceptance Criteria

1. WHEN the AI Agent decides to respond THEN the AI Agent System SHALL calculate typing delay as message_length multiplied by 50 milliseconds
2. WHEN the typing delay is calculated THEN the AI Agent System SHALL send a Typing Indicator to all group members before sending the actual message
3. WHEN the Typing Indicator is active THEN the AI Agent System SHALL maintain the indicator state for the full duration of the calculated delay
4. WHEN the typing delay expires THEN the AI Agent System SHALL send the generated message and clear the Typing Indicator
5. WHEN the calculated typing delay exceeds 10 seconds THEN the AI Agent System SHALL cap the delay at 10 seconds

### Requirement 5

**User Story:** 作为开发者，我希望 AI Agent 系统能够异步处理消息，以便不阻塞主消息处理流程并保持系统高性能。

#### Acceptance Criteria

1. WHEN a message enters the Message Pipeline THEN the AI Agent System SHALL spawn an asynchronous task for AI analysis without blocking message delivery
2. WHEN the AI analysis task is spawned THEN the AI Agent System SHALL complete the task using a separate Tokio task
3. WHEN multiple messages arrive simultaneously THEN the AI Agent System SHALL process each message analysis task concurrently
4. WHEN an AI analysis task fails THEN the AI Agent System SHALL log the error and continue processing other messages without system degradation
5. WHEN the AI analysis task completes THEN the AI Agent System SHALL publish the response through Redis Pub/Sub or Broadcast channel

### Requirement 6

**User Story:** 作为群成员，我希望 AI Agent 能够发送表情包和生成图片，以便它的交互方式更加丰富和生动。

#### Acceptance Criteria

1. WHEN the AI Agent determines an emoji response is appropriate THEN the AI Agent System SHALL retrieve the most contextually relevant emoji from the emoji library
2. WHEN the AI Agent sends an emoji THEN the AI Agent System SHALL set the message type field to EMOJI in the ChatMessage protobuf
3. WHEN the AI Agent is configured to support image generation THEN the AI Agent System SHALL invoke the image generation API when appropriate
4. WHEN an image is generated THEN the AI Agent System SHALL store the image URL in the content field and set the message type to IMAGE
5. WHEN emoji or image retrieval fails THEN the AI Agent System SHALL fall back to sending a text-only response

### Requirement 7

**User Story:** 作为系统管理员，我希望 AI Agent 系统能够与外部 LLM API 可靠地集成，以便生成高质量的回复内容。

#### Acceptance Criteria

1. WHEN the AI Agent needs to generate a response THEN the AI Agent System SHALL send a request to the configured LLM API with the System Prompt and conversation context
2. WHEN the LLM API request is sent THEN the AI Agent System SHALL include a timeout of 30 seconds
3. WHEN the LLM API returns a response THEN the AI Agent System SHALL validate that the response is non-empty and contains valid UTF-8 text
4. WHEN the LLM API request fails or times out THEN the AI Agent System SHALL retry up to 2 additional times with exponential backoff
5. WHEN all retry attempts fail THEN the AI Agent System SHALL log the failure and skip sending a response for that trigger event

### Requirement 8

**User Story:** 作为开发者，我希望 AI Agent 系统使用 Protobuf 进行消息序列化，以便保持与整体系统架构的一致性和高性能。

#### Acceptance Criteria

1. WHEN the AI Agent sends a message THEN the AI Agent System SHALL serialize the message using the ChatMessage protobuf definition
2. WHEN serializing a ChatMessage THEN the AI Agent System SHALL set the is_ai_generated field to true
3. WHEN the ChatMessage is serialized THEN the AI Agent System SHALL include all required fields: id, sender_id, group_id, type, content, and timestamp
4. WHEN deserializing incoming messages for analysis THEN the AI Agent System SHALL use the prost library to parse protobuf binary data
5. WHEN protobuf serialization or deserialization fails THEN the AI Agent System SHALL return a descriptive error and skip processing that message

### Requirement 9

**User Story:** 作为群主，我希望能够查看 AI Agent 的触发统计和行为分析，以便了解它的活跃度和调整配置。

#### Acceptance Criteria

1. WHEN the AI Agent evaluates a trigger condition THEN the AI Agent System SHALL record the trigger type and decision result in the database
2. WHEN the AI Agent sends a message THEN the AI Agent System SHALL increment the message count metric for that group
3. WHEN a group owner requests statistics THEN the AI Agent System SHALL return trigger counts, message counts, and average response times for the past 30 days
4. WHEN calculating statistics THEN the AI Agent System SHALL group data by trigger type including mention, keyword, context similarity, and random
5. WHEN statistics are unavailable THEN the AI Agent System SHALL return an empty result set without raising an error

### Requirement 10

**User Story:** 作为系统架构师，我希望 AI Agent 系统的各个组件职责清晰分离，以便系统易于维护和扩展。

#### Acceptance Criteria

1. WHEN the system is initialized THEN the AI Agent System SHALL separate concerns into distinct modules: trigger evaluation, memory retrieval, personality management, and LLM integration
2. WHEN a module interface is defined THEN the AI Agent System SHALL use Rust traits to enforce contract boundaries between modules
3. WHEN the trigger evaluation module is invoked THEN the AI Agent System SHALL not directly access the Vector Database or LLM API
4. WHEN the memory retrieval module is invoked THEN the AI Agent System SHALL not contain trigger logic or personality configuration
5. WHEN any module is replaced or updated THEN the AI Agent System SHALL continue functioning without modifications to other modules
