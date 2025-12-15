# AI Agent 工作流程详解

本文档详细说明 AI-Native-IM 项目中 AI Agent 的完整工作流程，从创建、配置、触发到响应的全过程。

## 目录

- [1. AI Agent 生命周期](#1-ai-agent-生命周期)
- [2. 创建 AI Agent](#2-创建-ai-agent)
- [3. 消息处理流程](#3-消息处理流程)
- [4. 智能触发机制](#4-智能触发机制)
- [5. 主动对话流程](#5-主动对话流程)
- [6. 记忆系统工作流程](#6-记忆系统工作流程)
- [7. 用户画像更新流程](#7-用户画像更新流程)
- [8. 多模态处理流程](#8-多模态处理流程)

---

## 1. AI Agent 生命周期

### 1.1 状态图

```
┌─────────────┐
│   未创建    │
└──────┬──────┘
       │ POST /api/groups/{id}/ai-agent
       ▼
┌─────────────┐
│   已创建    │◄──────────┐
└──────┬──────┘           │
       │                  │ PUT /api/groups/{id}/ai-agent
       │ 等待触发          │ (更新人设/配置)
       ▼                  │
┌─────────────┐           │
│   活跃中    │───────────┘
└──────┬──────┘
       │ DELETE /api/groups/{id}/ai-agent
       ▼
┌─────────────┐
│   已删除    │
└─────────────┘
```

### 1.2 核心组件

| 组件 | 职责 | 位置 |
|------|------|------|
| AIAgentService | AI Agent 核心服务 | `server/src/services/ai/mod.rs` |
| AgentMessageHandler | 消息处理协调器 | `server/src/services/ai/agent_message_handler.rs` |
| TriggerDetector | 触发检测器 | `server/src/services/ai/triggers.rs` |
| ProactiveMessenger | 主动消息生成器 | `server/src/services/ai/proactive/mod.rs` |
| UserProfileManager | 用户画像管理器 | `server/src/services/ai/user_profile/mod.rs` |
| IntentClassifier | 意图分类器 | `server/src/services/ai/intent/mod.rs` |

---

## 2. 创建 AI Agent

### 2.1 API 请求

**端点**: `POST /api/groups/{group_id}/ai-agent`

**请求体**:
```json
{
  "agent_name": "小智",
  "personality": {
    "traits": ["幽默", "专业", "热心"],
    "toxicity_level": 3,
    "enthusiasm_level": 8,
    "custom_prompt": "你是一个技术极客，热爱编程..."
  }
}
```

### 2.2 创建流程

```
1. 验证请求参数
   ├─ 检查 agent_name (1-100 字符)
   ├─ 检查 toxicity_level (0-10)
   └─ 检查 enthusiasm_level (0-10)
   
2. 检查 AI Agent 是否已存在
   └─ 如果存在 → 返回 409 Conflict
   
3. 生成系统提示词
   ├─ 基础人设模板
   ├─ 毒舌度调整
   ├─ 热情度调整
   └─ 自定义人设融合
   
4. 创建数据库记录
   ├─ 插入 ai_agents 表
   └─ 设置初始配置
   
5. 返回创建结果
   └─ 包含 agent_id 和完整配置
```

### 2.3 人设系统处理

**代码位置**: `server/src/services/ai/personality.rs`

```rust
// 人设生成流程
PersonalityConfig {
    traits,           // 性格特质
    toxicity_level,   // 毒舌度 (0-10)
    enthusiasm_level, // 热情度 (0-10)
    custom_prompt,    // 自定义提示词
}
↓
generate_system_prompt()
↓
"你是 {agent_name}，性格特点：{traits}
 毒舌程度：{toxicity_description}
 热情程度：{enthusiasm_description}
 {custom_prompt}"
```

---

## 3. 消息处理流程

### 3.1 完整流程图

```
用户发送消息
    ↓
WebSocket 接收
    ↓
消息入库 (messages 表)
    ↓
触发检测 (TriggerDetector)
    ├─ @提及检测
    ├─ 关键词匹配
    ├─ 情绪波动检测
    ├─ 问题检测
    └─ 持续对话检测
    ↓
是否触发? ──No──→ 结束
    │ Yes
    ↓
加载上下文
    ├─ 最近消息 (100条)
    ├─ RAG 记忆检索
    ├─ 用户画像
    └─ 意图分类
    ↓
生成响应
    ├─ 构建消息历史
    ├─ 思维链推理 (CoT)
    ├─ LLM API 调用
    └─ 流式响应处理
    ↓
打字模拟延迟
    ↓
发送消息
    ├─ 保存到数据库
    ├─ WebSocket 广播
    └─ 更新记忆系统
    ↓
后台任务
    ├─ 更新用户画像
    ├─ 生成记忆摘要
    └─ 情绪分析
```

### 3.2 触发检测详解

**代码位置**: `server/src/services/ai/triggers.rs`

#### 3.2.1 触发类型优先级

| 优先级 | 触发类型 | 触发条件 | 响应概率 |
|--------|----------|----------|----------|
| 1 (最高) | DirectMention | 消息中 @AI Agent | 100% |
| 2 | QuestionDetected | 包含问号或疑问词 | 70% |
| 3 | KeywordMatch | 匹配关键词列表 | 60% |
| 4 | EmotionalResponse | 检测到强烈情绪 | 50% |
| 5 | ContinuousConversation | 持续对话中 | 30% |

#### 3.2.2 触发配置

```rust
pub struct TriggerConfig {
    pub mention_response_rate: f32,      // @提及响应率: 1.0
    pub keyword_response_rate: f32,      // 关键词响应率: 0.6
    pub question_response_rate: f32,     // 问题响应率: 0.7
    pub emotion_response_rate: f32,      // 情绪响应率: 0.5
    pub conversation_response_rate: f32, // 对话响应率: 0.3
    pub keywords: Vec<String>,           // 关键词列表
}
```

### 3.3 上下文构建

**代码位置**: `server/src/services/ai/agent_message_handler.rs`

```rust
// 上下文加载步骤
1. 获取最近消息 (默认 100 条)
   ↓
2. RAG 语义检索相关记忆
   ├─ 将当前消息转换为向量
   ├─ 在 Qdrant 中搜索相似记忆
   └─ 返回 Top-K 记忆 (默认 5 条)
   ↓
3. 加载用户画像
   ├─ 用户基本信息
   ├─ 性格特质
   ├─ 兴趣爱好
   ├─ 交互历史
   └─ 关系等级
   ↓
4. 意图分类
   ├─ Greeting (问候)
   ├─ Question (提问)
   ├─ SmallTalk (闲聊)
   ├─ TaskRequest (任务请求)
   └─ Emotional (情绪表达)
   ↓
5. 构建最终上下文
```

### 3.4 思维链推理 (CoT)

**代码位置**: `server/src/services/ai/cot.rs`

```
隐藏思维链流程:
1. 分析用户意图
   "用户在询问技术问题，需要专业解答"
   ↓
2. 制定回复策略
   "结合用户画像，采用简洁专业的语气"
   ↓
3. 生成最终回复
   "根据您的项目，建议使用..."
```

**配置项**:
```rust
pub struct CoTConfig {
    pub enabled: bool,          // 是否启用
    pub max_tokens: usize,      // 最大思考 tokens
    pub show_reasoning: bool,   // 是否显示推理过程
}
```

---

## 4. 智能触发机制

### 4.1 触发决策树

```
新消息到达
    ↓
是否 @AI Agent? ──Yes──→ 触发 (100%)
    │ No
    ↓
是否包含问号? ──Yes──→ 随机触发 (70%)
    │ No
    ↓
是否匹配关键词? ──Yes──→ 随机触发 (60%)
    │ No
    ↓
是否强烈情绪? ──Yes──→ 随机触发 (50%)
    │ No
    ↓
是否持续对话? ──Yes──→ 随机触发 (30%)
    │ No
    ↓
不触发
```

### 4.2 关键词匹配策略

**配置示例**:
```json
{
  "keywords": [
    "AI",
    "人工智能",
    "机器学习",
    "帮我",
    "怎么办"
  ]
}
```

**匹配逻辑**:
```rust
fn matches_keyword(&self, message: &str) -> bool {
    let message_lower = message.to_lowercase();
    self.keywords.iter().any(|kw| {
        message_lower.contains(&kw.to_lowercase())
    })
}
```

### 4.3 情绪检测

**支持的情绪类型**:
- 😊 Happy (开心)
- 😢 Sad (悲伤)
- 😠 Angry (愤怒)
- 😰 Anxious (焦虑)
- 😴 Tired (疲惫)

**检测方法**:
```rust
// 通过 LLM 分析情绪强度
EmotionalState {
    emotion_type: Emotion,
    intensity: f32,  // 0.0 - 1.0
}
```

---

## 5. 主动对话流程

### 5.1 主动触发类型

**代码位置**: `server/src/services/ai/proactive/mod.rs`

| 触发类型 | 触发条件 | 示例场景 |
|----------|----------|----------|
| SilenceBreaker | 群组 N 小时无消息 | "好久没见大家了，最近怎么样？" |
| Scheduled | 定时任务 | "早上好！新的一天开始了" |
| ActivityBased | 特定活动后 | "看到大家讨论得很热烈" |
| SpecialEvent | 特殊事件 | "今天是周五，周末有什么计划？" |
| RelationshipBased | 基于用户关系 | "好久不见，最近忙吗？" |
| DailyGreeting | 日常问候 | "下午好！" |
| CareMessage | 关心消息 | "最近压力大吗？注意休息哦" |

### 5.2 沉默打破机制

**配置**:
```rust
pub struct SilenceBreakerConfig {
    pub silence_threshold_hours: u32,  // 触发阈值 (小时)
    pub max_per_day: u32,              // 每日最大次数
    pub preferred_hours: Vec<u8>,      // 偏好时段
}
```

**流程**:
```
检查群组最后消息时间
    ↓
超过阈值? ──No──→ 结束
    │ Yes
    ↓
今日次数未超限? ──No──→ 结束
    │ Yes
    ↓
当前在偏好时段? ──No──→ 结束
    │ Yes
    ↓
生成破冰消息
    ↓
发送消息
```

### 5.3 基于关系的主动消息

**关系等级**:
```rust
pub enum RelationshipLevel {
    Stranger,        // 陌生人 (0-20)
    Acquaintance,    // 熟人 (21-40)
    Friend,          // 朋友 (41-60)
    GoodFriend,      // 好友 (61-80)
    CloseFriend,     // 密友 (81-100)
    BestFriend,      // 挚友 (100+)
}
```

**触发原因**:
```rust
pub enum RelationshipTriggerReason {
    MissedCloseFriend,              // 想念密友
    MoodChange,                     // 情绪变化
    Milestone { days_known: u32 },  // 里程碑
    ReconnectAttempt,               // 重新联系
}
```

**消息生成策略**:
```rust
// 根据关系等级调整语气
match relationship_level {
    Stranger => "您好，有什么可以帮助的吗？",
    Friend => "嗨，最近怎么样？",
    CloseFriend => "好久不见！想你了~",
    BestFriend => "兄弟/姐妹，最近还好吗？咱们聊聊！",
}
```

---

## 6. 记忆系统工作流程

### 6.1 RAG 记忆架构

```
消息发送
    ↓
生成嵌入向量 (Embedding)
    ↓
存储到 Qdrant
    ├─ Collection: group_{group_id}
    ├─ Vector: [768 维度]
    └─ Payload: { message_id, content, timestamp, user_id }
    ↓
需要检索时
    ↓
查询向量相似度
    ↓
返回 Top-K 相关记忆
```

### 6.2 记忆类型

**代码位置**: `server/src/services/ai/memory/types.rs`

| 类型 | 重要性 | 保留时间 | 示例 |
|------|--------|----------|------|
| Important | 高 | 永久 | 用户偏好、重要事件 |
| Contextual | 中 | 30 天 | 近期对话、临时信息 |
| Trivial | 低 | 7 天 | 日常闲聊 |

### 6.3 记忆检索流程

```rust
// 检索步骤
1. 生成查询向量
   embedding_service.generate_embedding(query)
   ↓
2. 向量搜索
   qdrant_client.search(
       collection: "group_{id}",
       vector: query_vector,
       limit: 5,
   )
   ↓
3. 过滤和排序
   ├─ 按相似度排序
   ├─ 过滤过期记忆
   └─ 去重
   ↓
4. 返回结果
   Vec<Memory>
```

### 6.4 记忆摘要生成

**触发条件**:
- 消息数量达到阈值 (默认 100 条)
- 定时任务 (每日凌晨)

**流程**:
```
获取最近消息
    ↓
LLM 生成摘要
    ↓
存储摘要
    ├─ conversation_summaries 表
    └─ 作为高重要性记忆
```

---

## 7. 用户画像更新流程

### 7.1 画像数据结构

**代码位置**: `server/src/services/ai/user_profile/mod.rs`

```rust
pub struct UserProfile {
    pub user_id: Uuid,
    pub personality_traits: Vec<String>,  // 性格特质
    pub interests: Vec<String>,           // 兴趣爱好
    pub communication_style: String,      // 沟通风格
    pub activity_pattern: ActivityPattern, // 活跃模式
    pub emotional_tendency: String,       // 情绪倾向
    pub interaction_history: InteractionStats, // 交互统计
    pub relationship_level: RelationshipLevel, // 关系等级
}
```

### 7.2 更新触发时机

| 触发时机 | 更新内容 | 频率限制 |
|----------|----------|----------|
| 每次对话后 | 交互统计 | 实时 |
| 检测到新兴趣 | 兴趣列表 | 每小时 |
| 情绪分析完成 | 情绪倾向 | 每次 |
| 定时分析 | 性格特质 | 每日 |
| 关系变化 | 关系等级 | 每次 |

### 7.3 画像更新流程

```
消息处理完成
    ↓
提取用户信息
    ├─ 兴趣关键词
    ├─ 情绪状态
    ├─ 交互模式
    └─ 话题偏好
    ↓
LLM 分析
    ├─ 分析性格特质
    ├─ 识别新兴趣
    └─ 评估关系变化
    ↓
更新数据库
    ├─ user_profiles 表
    └─ user_profile_updates 表 (历史)
    ↓
计算关系分数
    ├─ 互动频率 (40%)
    ├─ 对话质量 (30%)
    ├─ 情感亲密度 (20%)
    └─ 时间跨度 (10%)
```

### 7.4 关系等级计算

```rust
fn calculate_relationship_score(stats: &InteractionStats) -> i16 {
    let frequency_score = (stats.message_count as f32 / 100.0).min(1.0) * 40.0;
    let quality_score = stats.avg_response_length / 50.0 * 30.0;
    let intimacy_score = stats.emotional_depth * 20.0;
    let duration_score = (stats.days_known as f32 / 365.0).min(1.0) * 10.0;
    
    (frequency_score + quality_score + intimacy_score + duration_score) as i16
}
```

---

## 8. 多模态处理流程

### 8.1 支持的媒体类型

**代码位置**: `server/src/services/ai/multimodal.rs`

| 媒体类型 | 处理方式 | API 提供商 |
|----------|----------|------------|
| 图片 (JPG/PNG/WebP) | Vision API | OpenAI GPT-4V |
| 音频 (MP3/WAV) | STT → 文本 | OpenAI Whisper |
| 视频 | 提取关键帧 → Vision API | 暂不支持 |

### 8.2 图片处理流程

```
用户发送图片消息
    ↓
媒体文件上传
    ├─ 存储到 S3/本地
    ├─ 生成缩略图
    └─ 病毒扫描
    ↓
消息入库
    ├─ messages 表
    └─ media_attachments 表
    ↓
触发 AI Agent
    ↓
Vision API 分析
    ├─ 提取图片 URL
    ├─ 调用 GPT-4V
    └─ 获取图片描述
    ↓
融合到上下文
    ↓
生成文本响应
```

### 8.3 Vision API 调用

**代码示例**:
```rust
let analysis = vision_service.analyze_image(
    image_url,
    Some("描述这张图片，关注细节")
).await?;

// 返回: "这是一张代码截图，显示了 Rust 程序..."
```

### 8.4 语音处理流程

```
用户发送语音消息
    ↓
音频文件上传
    ↓
STT 转录
    ├─ OpenAI Whisper API
    └─ 返回文本内容
    ↓
作为文本消息处理
    ↓
生成文本回复
    ↓
(可选) TTS 合成语音
    ↓
返回文本 + 语音
```

---

## 9. 性能优化

### 9.1 并发处理

- **WebSocket 连接池**: 复用连接，减少握手开销
- **数据库连接池**: PgPool (最大 20 连接)
- **Redis 连接池**: 异步连接管理
- **LLM API 并发限制**: 信号量控制 (最大 10 并发)

### 9.2 缓存策略

| 缓存项 | 存储位置 | TTL | 缓存键 |
|--------|----------|-----|--------|
| AI Agent 配置 | Redis | 1 小时 | `agent:{group_id}` |
| 用户画像 | Redis | 30 分钟 | `profile:{user_id}` |
| 最近消息 | Redis | 10 分钟 | `messages:{group_id}` |
| 触发配置 | Redis | 1 小时 | `trigger:{group_id}` |

### 9.3 异步任务

使用 Tokio spawn 处理非关键任务:
```rust
// 异步更新用户画像
tokio::spawn(async move {
    profile_manager.update_profile(user_id, updates).await
});

// 异步生成摘要
tokio::spawn(async move {
    memory_service.generate_summary(group_id).await
});
```

---

## 10. 监控与调试

### 10.1 日志级别

| 级别 | 用途 | 示例 |
|------|------|------|
| ERROR | 严重错误 | LLM API 失败 |
| WARN | 警告信息 | 触发频率过高 |
| INFO | 关键流程 | AI Agent 创建 |
| DEBUG | 详细调试 | 触发决策过程 |
| TRACE | 超详细 | LLM 请求/响应 |

### 10.2 关键指标

- **触发准确率**: 触发消息 / 总消息
- **响应时间**: P50, P95, P99
- **LLM API 成功率**: 成功请求 / 总请求
- **用户满意度**: 基于反馈统计

### 10.3 调试工具

**环境变量**:
```bash
# 启用详细日志
RUST_LOG=ai_native_im=debug

# 显示思维链推理
AI_AGENT_SHOW_COT=true

# 禁用随机触发（测试用）
AI_AGENT_DETERMINISTIC=true
```

---

## 11. 最佳实践

### 11.1 人设配置建议

✅ **推荐做法**:
- 人设描述具体、有特色
- 毒舌度控制在 3-5 之间
- 热情度根据群组氛围调整
- 定期更新自定义提示词

❌ **避免**:
- 人设过于矛盾 (如 "非常毒舌" + "极度热情")
- 毒舌度过高 (>8) 可能冒犯用户
- 人设描述过长 (>500 字)

### 11.2 触发策略建议

✅ **推荐做法**:
- 问题检测响应率保持 60-80%
- 关键词列表精心筛选 (5-15 个)
- 沉默打破阈值设置 4-8 小时
- 每日主动消息不超过 3 次

❌ **避免**:
- 所有触发类型响应率都是 100%
- 关键词过多导致过度触发
- 沉默打破阈值过短 (< 2 小时)

### 11.3 性能优化建议

✅ **推荐做法**:
- 启用 Redis 缓存
- 限制上下文消息数量 (50-100 条)
- 使用流式响应提升体验
- 异步处理非关键任务

❌ **避免**:
- 每次都加载全部历史消息
- 同步等待所有后台任务
- 未设置 LLM API 超时

---

## 12. 故障排查

### 12.1 常见问题

#### Q1: AI Agent 不响应消息

**检查清单**:
1. ✅ AI Agent 是否已创建？
2. ✅ 触发配置是否合理？
3. ✅ LLM API 是否可用？
4. ✅ Redis/数据库连接是否正常？

**调试步骤**:
```bash
# 1. 检查 AI Agent 状态
curl http://localhost:3000/api/groups/{id}/ai-agent

# 2. 查看日志
tail -f logs/ai-native-im.log | grep "TriggerDetector"

# 3. 测试 @提及 (100% 触发)
发送消息: "@小智 你好"
```

#### Q2: 响应速度慢

**可能原因**:
- LLM API 延迟高
- 上下文消息过多
- 未启用缓存
- 数据库查询慢

**优化措施**:
```rust
// 1. 减少上下文消息数量
limit_messages: 50  // 从 100 → 50

// 2. 启用流式响应
streaming: true

// 3. 增加缓存 TTL
ttl: 3600  // 1 小时
```

#### Q3: 主动消息过于频繁

**调整配置**:
```json
{
  "silence_breaker": {
    "silence_threshold_hours": 8,  // 增加阈值
    "max_per_day": 2,              // 减少次数
    "preferred_hours": [9, 14, 20] // 限制时段
  }
}
```

---

## 13. 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| 1.0 | 2025-12-11 | 初始版本，涵盖 Phase 1-6 完整工作流程 |

---

## 14. 相关文档

- [AI Agent 开发指南](./AI_AGENT_GUIDE.md)
- [API 文档](./API.md)
- [错误码参考](./ERROR_CODES.md)
- [数据库设计](./DATABASE.md)

