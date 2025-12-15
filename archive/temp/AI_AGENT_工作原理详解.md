# AI Agent 系统工作原理详解

## 目录
1. [系统概述](#系统概述)
2. [核心架构](#核心架构)
3. [完整工作流程](#完整工作流程)
4. [具体例子：从消息到响应](#具体例子从消息到响应)
5. [关键组件详解](#关键组件详解)
6. [数据流图](#数据流图)

---

## 系统概述

AI Agent 系统是一个三层认知架构的智能对话代理系统，能够在群聊中像真实用户一样参与对话。系统通过感知层、认知层和行动层的协同工作，实现智能化的消息理解、决策和响应。

### 核心特点
- **三层认知架构**：感知 → 认知 → 行动
- **多触发机制**：@提及、关键词、专业领域、沉默打破、随机插话
- **记忆系统**：基于向量的语义记忆，支持上下文理解
- **人设系统**：可配置的性格特征（毒舌度、热情度、专业领域等）
- **打字模拟**：真实的打字延迟效果
- **防重复机制**：冷却时间和去重处理

### Phase 4 新增特性 (2025-12)
- **用户自定义人设**：完整的人设配置系统，包含情绪边界、贴纸偏好、语音设置等
- **思维链 (Hidden CoT)**：隐藏式思考过程，提升回复质量
- **用户画像系统**：基于互动历史的关系级别识别 (6级关系)
- **意图分类系统**：10种意图类型识别，智能调整回复策略
- **多模态理解增强**：支持图片/音频/视频/文件/贴纸的统一上下文处理
- **主动对话系统**：基于用户关系的主动问候、关心消息和日常问候

---

## 核心架构

```
┌─────────────────────────────────────────────────────────────┐
│                    感知层 (Perception Layer)                │
│                                                              │
│  - 消息接收和解析（多模态的消息 图片 文本等）                                             │
│  - 触发条件检测（@提及、关键词、专业领域等）                    │
│  - 情感分析                                                  │
│  - 上下文信息提取                                             │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                    认知层 (Cognition Layer)                  │
│                                                              │
│  - AgentCore 决策核心                                         │
│  - 记忆检索（向量相似度搜索）                                  │
│  - 人设和情感状态管理                                          │
│  - 响应内容生成（LLM调用）                                     │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                    行动层 (Action Layer)                     │
│                                                              │
│  - 打字延迟模拟                                               │
│  - 消息发送                                                  │
│  - 统计信息更新                                               │
│  - 防重复处理                                                 │
└─────────────────────────────────────────────────────────────┘
```

---

## 完整工作流程

### 流程图

```
用户发送消息
    ↓
[1] 消息路由器接收 (AIMessageRouter)
    ↓
[2] 添加到最近消息缓存
    ↓
[3] 创建 AgentMessage
    ↓
[4] 触发 AgentMessageHandler.process_message()
    ↓
[5] 检查是否有活跃的 AI Agent
    ↓
[6] 调用 AgentService.process_message()
    ├─ 触发评估 (TriggerEvaluator)
    │   ├─ 检查是否回复 Agent 消息 (95% 触发)
    │   ├─ 检查 @提及 (100% 触发)
    │   ├─ 检查冷却时间
    │   ├─ 检查关键词匹配 (60-80% 触发)
    │   ├─ 检查专业领域匹配 (40-60% 触发)
    │   ├─ 检查沉默时间 (10-20% 触发)
    │   └─ 随机插话 (3% 触发)
    └─ 记忆检索 (VectorStore)
        └─ 语义相似度搜索
    ↓
[7] 决定是否响应？
    ├─ 否 → 结束
    └─ 是 ↓
[8] 生成响应 (LLM Client)
    ├─ 构建系统提示词（基于人设）
    ├─ 添加历史上下文
    ├─ 添加最近消息
    └─ 调用 LLM API
    ↓
[9] 计算打字延迟
    ↓
[10] 发送到响应通道 (response_tx)
    ↓
[11] AgentResponseWorker 接收
    ├─ 检查是否重复
    ├─ 模拟打字延迟
    └─ 发送消息到群聊
    ↓
[12] 更新统计信息
    ↓
完成
```

---

## 具体例子：从消息到响应

### 场景设置

**群聊信息**：
- 群组ID: `group-123`
- 群成员: Alice, Bob, Charlie
- AI Agent: "小助手"（毒舌度: 7, 热情度: 5, 专业领域: ["编程", "AI"]）

**最近消息历史**：
```
Alice: 今天天气真好
Bob: 是啊，适合出去玩
Charlie: 我在学 Rust，有点难
```

### 触发消息
```
Charlie: @小助手 Rust 的所有权系统怎么理解？
```

---

### 详细执行流程

#### 第1步：消息接收 (AIMessageRouter)

```rust
// src/services/ai/ai_message_router.rs:route_message()

let message = CreateMessage {
    id: Some(msg_001),
    sender_id: charlie_id,
    group_id: Some(group_123),
    content: "@小助手 Rust 的所有权系统怎么理解？",
    msg_type: MsgType::Text,
    is_ai_generated: false,
};

// 路由消息并返回 ACK
let ack = inner_router.route_message(message).await?;
```

#### 第2步：添加到最近消息缓存

```rust
// 更新群组的最近消息列表
self.add_to_recent(
    group_123,
    "Charlie",
    "@小助手 Rust 的所有权系统怎么理解？"
).await;

// 缓存现在包含：
// [
//   "Alice: 今天天气真好",
//   "Bob: 是啊，适合出去玩",
//   "Charlie: 我在学 Rust，有点难",
//   "Charlie: @小助手 Rust 的所有权系统怎么理解？"
// ]
```

#### 第3步：创建 AgentMessage

```rust
let agent_message = AgentMessage {
    message_id: msg_001,
    group_id: group_123,
    sender_id: charlie_id,
    sender_name: "Charlie".to_string(),
    content: "@小助手 Rust 的所有权系统怎么理解？".to_string(),
    timestamp: 1702345678000, // 2024-12-11 10:00:00
};
```

#### 第4步：触发评估 (TriggerEvaluator)

```rust
// src/services/ai/trigger.rs:evaluate()

let context = GroupContext {
    group_id: group_123,
    agent: 小助手,
    last_message_time: Some(2024-12-11 09:58:00),
    is_reply_to_agent: false,
};

// 检查触发条件
// 1. 检查是否回复 Agent → 否
// 2. 检查 @提及 → 是！发现 "@小助手"
let decision = TriggerDecision {
    should_respond: true,
    probability: 1.0,  // 100% 触发
    trigger_type: TriggerType::Mention,
};
```

**触发优先级**：
```
优先级 0: 回复 Agent 消息    → 95% 触发 ✓
优先级 1: @提及             → 100% 触发 ✓✓ (本例触发)
优先级 2: 关键词匹配         → 60-80% 触发
优先级 2.5: 专业领域匹配     → 40-60% 触发
优先级 3: 沉默打破          → 10-20% 触发
优先级 4: 随机插话          → 3% 触发
```

#### 第5步：记忆检索 (VectorStore)

```rust
// src/services/ai/memory/vector_store.rs:retrieve()

// 1. 生成查询嵌入向量
let query = "Rust 的所有权系统怎么理解？";
let query_embedding = embedding_service.embed(query).await?;
// query_embedding: [0.123, -0.456, 0.789, ...] (1024维)

// 2. 语义相似度搜索
let memories = vector_store.retrieve(query, limit: 5).await?;

// 返回的相关记忆：
// [
//   Memory {
//     content: "Charlie 之前问过 Rust 的借用检查器",
//     importance: 0.8,
//     similarity: 0.92,
//   },
//   Memory {
//     content: "群里讨论过所有权和生命周期的关系",
//     importance: 0.7,
//     similarity: 0.85,
//   },
//   ...
// ]
```

#### 第6步：构建 LLM 提示词

```rust
// src/services/ai/agent_service.rs:generate_response()

// 1. 系统提示词（基于人设）
let system_prompt = format!(
    "你是{}，一个群聊中的AI助手。
    
    性格特点：
    - 毒舌程度：经常吐槽和调侃 (7/10)
    - 热情程度：积极热情 (5/10)
    - 专业领域：编程、AI
    
    请根据以上性格特点回复用户消息。保持自然、有趣的对话风格。",
    "小助手"
);

// 2. 历史上下文（从记忆检索）
let context_messages = vec![
    "Charlie 之前问过 Rust 的借用检查器",
    "群里讨论过所有权和生命周期的关系",
];

// 3. 最近消息
let recent_messages = vec![
    "Alice: 今天天气真好",
    "Bob: 是啊，适合出去玩",
    "Charlie: 我在学 Rust，有点难",
    "Charlie: @小助手 Rust 的所有权系统怎么理解？",
];

// 4. 构建完整提示
let full_prompt = format!(
    "相关上下文：\n{}\n\n最近对话：\n{}\n\n当前消息：{}",
    context_messages.join("\n"),
    recent_messages.join("\n"),
    "@小助手 Rust 的所有权系统怎么理解？"
);
```

#### 第7步：调用 LLM 生成响应

```rust
// src/services/ai/llm_client.rs:generate()

let llm_request = LLMRequest {
    messages: vec![
        LLMMessage {
            role: "system",
            content: system_prompt,
        },
        LLMMessage {
            role: "user",
            content: full_prompt,
        },
    ],
    temperature: 0.8,
    max_tokens: 500,
};

// 调用 LLM API (例如 OpenAI)
let response = llm_client.generate(llm_request).await?;

// LLM 返回的响应：
let ai_response = "
哟，又是你啊Charlie！上次借用检查器还没搞明白呢？😏

好吧好吧，既然你诚心诚意地问了，我就大发慈悲地告诉你。

Rust的所有权系统其实就三条规则：
1. 每个值都有一个所有者
2. 同一时间只能有一个所有者
3. 所有者离开作用域，值就被丢弃

简单来说，就是"谁创建谁负责"。你创建了一个变量，你就是它的主人，
别人想用？要么借用（&），要么你转让所有权（move）。

这样设计是为了在编译期就防止内存泄漏和数据竞争，不用垃圾回收器。
虽然一开始会觉得编译器很烦，但习惯了就会爱上这种安全感~

还有问题就继续问吧，反正我闲着也是闲着 😎
";
```

#### 第8步：计算打字延迟

```rust
// src/services/ai/typing_simulator.rs:calculate_delay()

let response_length = ai_response.chars().count(); // 约 200 字符
let base_delay_ms = 50; // 每字符 50ms
let variation = rand::gen_range(0.8..1.2); // 随机变化 ±20%

let typing_delay = Duration::from_millis(
    (response_length as u64 * base_delay_ms as f64 * variation) as u64
);
// 结果：约 10,000ms = 10秒
```

#### 第9步：创建响应对象

```rust
let agent_response = AgentResponse {
    agent_id: 小助手_id,
    agent_name: "小助手".to_string(),
    group_id: group_123,
    content: ai_response,
    reply_to_message_id: msg_001, // 回复 Charlie 的消息
    typing_delay_ms: 10000, // 10秒打字延迟
};
```

#### 第10步：发送到响应通道

```rust
// src/services/ai/agent_message_handler.rs:process_message()

// 发送到响应通道
response_tx.send(agent_response.clone())?;

// 记录日志
info!(
    "Agent {} will respond to message {} (trigger: Mention, prob: 1.00)",
    "小助手", msg_001
);
```

#### 第11步：响应工作器处理 (AgentResponseWorker)

```rust
// src/services/ai/agent_message_handler.rs:AgentResponseWorker::run()

// 1. 从通道接收响应
let response = response_rx.recv().await?;

// 2. 检查是否重复
if processed_messages.contains(&response.reply_to_message_id) {
    // 跳过重复响应
    return;
}
processed_messages.insert(response.reply_to_message_id);

// 3. 模拟打字延迟
tokio::time::sleep(Duration::from_millis(10000)).await;

// 用户在这10秒内会看到 "小助手 正在输入..." 的提示

// 4. 发送消息
let message_id = message_sender.send_message(
    group_123,
    小助手_id,
    "小助手",
    ai_response,
    Some(msg_001), // 回复目标
).await?;

info!("Agent 小助手 sent message {} to group {}", message_id, group_123);
```

#### 第12步：更新统计信息

```rust
// 更新 Agent 统计
sqlx::query!(
    r#"
    UPDATE agent_message_stats
    SET total_messages = total_messages + 1,
        avg_response_time_ms = (avg_response_time_ms * total_messages + $2) / (total_messages + 1)
    WHERE agent_id = $1
    "#,
    小助手_id,
    10000 // 响应时间
).execute(&pool).await?;

// 记录触发日志
sqlx::query!(
    r#"
    INSERT INTO agent_trigger_log (agent_id, group_id, message_id, trigger_type, probability)
    VALUES ($1, $2, $3, $4, $5)
    "#,
    小助手_id,
    group_123,
    msg_001,
    "Mention",
    1.0
).execute(&pool).await?;
```

---

## 关键组件详解

### 1. TriggerEvaluator（触发评估器）

**职责**：决定 AI Agent 是否应该响应消息

**触发类型和概率**：

| 触发类型 | 触发条件 | 概率 | 忽略冷却 |
|---------|---------|------|---------|
| 回复 Agent | 用户回复 Agent 的消息 | 95% | ✓ |
| @提及 | 消息中包含 `@AgentName` | 100% | ✓ |
| 关键词 | 匹配触发关键词 | 60-80% | ✗ |
| 专业领域 | 匹配专业领域词汇 | 40-60% | ✗ |
| 沉默打破 | 群组沉默超过阈值（默认2小时） | 10-20% | ✗ |
| 随机插话 | 随机触发 | 3% | ✗ |

**冷却机制**：
- 默认冷却时间：30秒
- @提及和回复 Agent 忽略冷却
- 其他触发类型必须等待冷却

**示例代码**：
```rust
// 检查 @提及
if message.contains(&format!("@{}", agent.name)) {
    return TriggerDecision {
        should_respond: true,
        probability: 1.0,
        trigger_type: TriggerType::Mention,
    };
}

// 检查关键词（受冷却影响）
if !in_cooldown && check_keywords(message, &agent.trigger_keywords) {
    let base_prob = rand::gen_range(0.6..0.8);
    let boosted_prob = (base_prob * 1.2).min(1.0); // 应用关键词加成
    return TriggerDecision {
        should_respond: should_respond_random(boosted_prob),
        probability: boosted_prob,
        trigger_type: TriggerType::Keyword,
    };
}
```

### 2. VectorStore（向量存储）

**职责**：基于语义相似度的记忆存储和检索

**工作原理**：
1. **存储记忆**：
   - 将文本转换为嵌入向量（1024维或1536维）
   - 存储向量和元数据
   - 维护重要性评分

2. **检索记忆**：
   - 查询文本转换为向量
   - 计算余弦相似度
   - 返回最相关的记忆

**余弦相似度计算**：
```rust
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    dot_product / (norm_a * norm_b)
}
```

**示例**：
```rust
// 存储记忆
let memory = Memory {
    id: Uuid::new_v4(),
    group_id: group_123,
    content: "Charlie 问了关于 Rust 所有权的问题",
    embedding: vec![0.123, -0.456, ...], // 1024维向量
    importance: 0.8,
    created_at: Utc::now(),
    metadata: MemoryMetadata {
        source_message_id: Some(msg_001),
        topic: Some("Rust编程".to_string()),
        entities: vec!["Charlie", "Rust", "所有权"],
        emotion: Some(EmotionState::Curious),
    },
};
vector_store.store(memory).await?;

// 检索记忆
let query = "Rust 所有权系统";
let relevant_memories = vector_store.retrieve(query, limit: 5).await?;
// 返回相似度最高的5条记忆
```

### 3. LLMClient（LLM 客户端）

**职责**：统一的多提供商 LLM 接口

**支持的提供商**：
- **国际**：OpenAI, Anthropic, Google
- **国内**：SiliconFlow (硅基流动), Zhipu (智谱), Moonshot (月之暗面), Tongyi (通义千问), Wenxin (文心一言)
- **自定义**：Ollama, vLLM 等 OpenAI 兼容 API

**特性**：
- 自动重试（指数退避）
- 超时处理
- 流式响应支持
- 速率限制处理

**示例**：
```rust
// 配置 LLM 客户端
let config = LLMConfig::siliconflow(
    api_key.clone(),
    Some("Qwen/Qwen2.5-7B-Instruct".to_string())
);

let client = DefaultLLMClient::new(config);

// 生成响应
let request = LLMRequest {
    messages: vec![
        LLMMessage::system("你是一个友好的AI助手"),
        LLMMessage::user("你好！"),
    ],
    temperature: 0.8,
    max_tokens: 500,
};

let response = client.generate(request).await?;
// response.content: "你好！很高兴见到你..."
```

### 4. PersonalityManager（人设管理器）

**职责**：根据人设配置调整响应风格

**人设属性**：
```rust
pub struct Persona {
    pub name: String,
    pub toxicity_level: i16,      // 毒舌度 0-10
    pub enthusiasm_level: i16,    // 热情度 0-10
    pub expertise_domains: Vec<String>, // 专业领域
    pub speech_suffix: Option<String>,  // 说话后缀（如"喵~"）
    pub trigger_keywords: Vec<String>,  // 触发关键词
}
```

**系统提示词生成**：
```rust
fn generate_system_prompt(persona: &Persona) -> String {
    let toxicity_desc = match persona.toxicity_level {
        0..=2 => "非常友善温和",
        3..=4 => "友善但偶尔会开玩笑",
        5..=6 => "有点毒舌但不过分",
        7..=8 => "经常吐槽和调侃",
        _ => "毒舌王，喜欢犀利点评",
    };
    
    format!(
        "你是{}，性格特点：{}。专业领域：{}",
        persona.name,
        toxicity_desc,
        persona.expertise_domains.join("、")
    )
}
```

### 5. TypingSimulator（打字模拟器）

**职责**：计算真实的打字延迟

**策略类型**：
- **均匀速度**：固定速度打字
- **变化速度**：速度随机波动
- **停顿模式**：模拟思考停顿

**计算公式**：
```rust
fn calculate_delay(content: &str) -> Duration {
    let char_count = content.chars().count();
    let base_delay_per_char = 50; // 毫秒
    let variation = rand::gen_range(0.8..1.2); // ±20% 随机变化
    
    let total_ms = (char_count as f64 * base_delay_per_char as f64 * variation) as u64;
    
    // 添加思考时间（长消息需要更多思考）
    let think_time = if char_count > 100 {
        rand::gen_range(1000..3000) // 1-3秒
    } else {
        0
    };
    
    Duration::from_millis(total_ms + think_time)
}
```

**示例**：
```
短消息（20字符）：50ms × 20 × 1.1 = 1100ms ≈ 1秒
中等消息（100字符）：50ms × 100 × 0.95 + 2000ms = 6750ms ≈ 7秒
长消息（200字符）：50ms × 200 × 1.15 + 2500ms = 14000ms ≈ 14秒
```

### 6. AgentResponseWorker（响应工作器）

**职责**：异步处理 Agent 响应，避免阻塞消息路由

**防重复机制**：
```rust
pub struct AgentResponseWorker {
    response_rx: mpsc::UnboundedReceiver<AgentResponse>,
    message_sender: Arc<dyn AgentMessageSender>,
    processed_messages: HashSet<Uuid>, // 已处理的消息ID
}

impl AgentResponseWorker {
    pub async fn run(mut self) {
        while let Some(response) = self.response_rx.recv().await {
            // 检查是否已处理（防止重复响应）
            if self.processed_messages.contains(&response.reply_to_message_id) {
                continue; // 跳过
            }
            
            // 标记为已处理
            self.processed_messages.insert(response.reply_to_message_id);
            
            // 模拟打字延迟
            tokio::time::sleep(Duration::from_millis(response.typing_delay_ms)).await;
            
            // 发送消息
            self.message_sender.send_message(...).await?;
            
            // 定期清理（保留最近1000条）
            if self.processed_messages.len() > 1000 {
                self.processed_messages.clear();
            }
        }
    }
}
```

---

## 数据流图

### 消息流向

```
┌─────────────┐
│  用户发送消息  │
└──────┬──────┘
       │
       ↓
┌─────────────────────────────┐
│   WebSocket/HTTP 接收        │
│   (api/message_ws.rs)        │
└──────┬──────────────────────┘
       │
       ↓
┌─────────────────────────────┐
│   MessageRouter              │
│   路由到群组/私聊             │
└──────┬──────────────────────┘
       │
       ↓
┌─────────────────────────────┐
│   AIMessageRouter            │
│   - 添加到最近消息缓存        │
│   - 创建 AgentMessage        │
└──────┬──────────────────────┘
       │
       ↓
┌─────────────────────────────┐
│   AgentMessageHandler        │
│   - 获取群组的 AI Agent      │
│   - 调用 process_message()   │
└──────┬──────────────────────┘
       │
       ↓
┌─────────────────────────────┐
│   AIAgentService             │
│   - 触发评估                 │
│   - 记忆检索                 │
│   - 响应生成                 │
└──────┬──────────────────────┘
       │
       ├─────────────────────────┐
       │                         │
       ↓                         ↓
┌──────────────┐      ┌──────────────────┐
│ TriggerEval  │      │   VectorStore    │
│ 评估触发条件  │      │   检索相关记忆    │
└──────┬───────┘      └────────┬─────────┘
       │                       │
       └───────┬───────────────┘
               │
               ↓
       ┌───────────────┐
       │  LLMClient    │
       │  生成响应内容  │
       └───────┬───────┘
               │
               ↓
       ┌──────────────────┐
       │ TypingSimulator  │
       │ 计算打字延迟      │
       └───────┬──────────┘
               │
               ↓
       ┌──────────────────┐
       │  response_tx     │
       │  发送到响应通道   │
       └───────┬──────────┘
               │
               ↓
┌──────────────────────────────┐
│   AgentResponseWorker        │
│   - 检查重复                  │
│   - 模拟打字延迟              │
│   - 发送消息                  │
└──────┬───────────────────────┘
       │
       ↓
┌─────────────────────────────┐
│   MessageSender              │
│   发送到群组                  │
└──────┬──────────────────────┘
       │
       ↓
┌─────────────────────────────┐
│   WebSocket 推送给群成员      │
└─────────────────────────────┘
```

### 数据库交互

```
┌─────────────────┐
│  AI Agent 创建   │
└────────┬────────┘
         │
         ↓
┌──────────────────────────┐
│  INSERT INTO ai_agents   │
│  - id, name, group_id    │
│  - toxicity_level        │
│  - enthusiasm_level      │
│  - expertise_domains     │
│  - trigger_keywords      │
└────────┬─────────────────┘
         │
         ↓
┌──────────────────────────────┐
│  INSERT INTO                 │
│  agent_message_stats         │
│  - agent_id                  │
│  - total_messages = 0        │
│  - avg_response_time_ms = 0  │
└──────────────────────────────┘

┌─────────────────┐
│  消息处理        │
└────────┬────────┘
         │
         ↓
┌──────────────────────────────┐
│  SELECT FROM ai_agents       │
│  WHERE group_id = ? AND      │
│        is_active = TRUE      │
└────────┬─────────────────────┘
         │
         ↓
┌──────────────────────────────┐
│  触发评估 + 响应生成          │
└────────┬─────────────────────┘
         │
         ↓
┌──────────────────────────────┐
│  INSERT INTO                 │
│  agent_trigger_log           │
│  - agent_id, message_id      │
│  - trigger_type              │
│  - probability               │
└────────┬─────────────────────┘
         │
         ↓
┌──────────────────────────────┐
│  UPDATE agent_message_stats  │
│  SET total_messages += 1,    │
│      avg_response_time_ms    │
└──────────────────────────────┘
```

---

## 性能优化

### 1. 异步处理

**问题**：如果同步处理 AI 响应，会阻塞消息路由，影响其他用户

**解决方案**：使用异步通道和后台工作器

```rust
// 消息路由立即返回，不等待 AI 处理
let ack = inner_router.route_message(message).await?;

// AI 处理在后台异步进行
tokio::spawn(async move {
    handler.process_message(agent_message, &recent_messages).await
});

return Ok(ack); // 立即返回 ACK
```

### 2. 缓存策略

**最近消息缓存**：
```rust
// 每个群组维护最近 20 条消息
let recent_messages: HashMap<Uuid, Vec<String>>;

// 添加消息时自动裁剪
if messages.len() > 20 {
    messages.remove(0);
}
```

**记忆限制**：
```rust
// 每个群组最多存储 1000 条记忆
if group_memories.len() >= 1000 {
    // 移除重要性最低的记忆
    remove_least_important_memory();
}
```

### 3. 并发控制

**冷却机制**：
```rust
// 防止 Agent 频繁响应
let last_response_times: HashMap<Uuid, DateTime<Utc>>;

if is_in_cooldown(group_id) {
    return TriggerDecision {
        should_respond: false,
        probability: 0.0,
        trigger_type: TriggerType::Random,
    };
}
```

**防重复处理**：
```rust
// 使用 HashSet 跟踪已处理的消息
let processed_messages: HashSet<Uuid>;

if processed_messages.contains(&reply_to_message_id) {
    return; // 跳过重复响应
}
```

---

## 错误处理

### 1. LLM 调用失败

```rust
// 自动重试机制
let mut retries = 0;
let max_retries = 3;

loop {
    match llm_client.generate(request.clone()).await {
        Ok(response) => return Ok(response),
        Err(e) if retries < max_retries => {
            retries += 1;
            let backoff = Duration::from_secs(2_u64.pow(retries));
            tokio::time::sleep(backoff).await;
        }
        Err(e) => return Err(AIAgentError::LLMError(e)),
    }
}
```

### 2. 数据库错误

```rust
// 优雅降级：即使统计更新失败，也不影响响应发送
if let Err(e) = update_message_stats(agent_id, response_time).await {
    warn!("Failed to update stats: {}", e);
    // 继续执行，不中断流程
}
```

### 3. 通道关闭

```rust
// 检测通道关闭，优雅退出
while let Some(response) = response_rx.recv().await {
    // 处理响应
}

info!("Response channel closed, worker stopping");
// 清理资源并退出
```

---

## 总结

AI Agent 系统通过三层认知架构实现了智能化的群聊参与：

1. **感知层**：接收消息，检测触发条件，提取上下文
2. **认知层**：检索记忆，应用人设，生成响应
3. **行动层**：模拟打字，发送消息，更新统计

关键特性：
- ✅ 多触发机制（@提及、关键词、专业领域等）
- ✅ 语义记忆系统（向量存储 + 相似度搜索）
- ✅ 可配置人设（毒舌度、热情度、专业领域）
- ✅ 真实打字延迟模拟
- ✅ 防重复和冷却机制
- ✅ 异步处理不阻塞主流程
- ✅ 多 LLM 提供商支持

### Phase 4 新增能力 (2025-12)

| 模块 | 能力 | 说明 |
|------|------|------|
| **用户自定义人设** | PersonaConfig | 完整的人设配置，包含情绪边界、贴纸偏好、语音设置 |
| **思维链系统** | CoTProcessor | 隐藏式思考过程 (Hidden CoT)，提升回复质量 |
| **用户画像** | UserProfile | 6级关系识别 (Stranger→CloseFriend)，亲密度追踪 |
| **意图分类** | IntentClassifier | 10种意图类型，智能回复策略调整 |
| **多模态增强** | MultimodalContext | 图片/音频/视频/文件/贴纸统一上下文 |
| **主动对话** | ProactiveMessenger | 基于关系的主动问候/关心消息/日常问候 |

通过这个系统，AI Agent 能够像真实用户一样自然地参与群聊，提供智能化的对话体验。
