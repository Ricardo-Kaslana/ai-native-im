# 📝 更新日志

所有重要的项目变更都会记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [未发布]

### 计划中 - Phase 5
- 生产环境部署优化
- 性能监控和告警
- A/B 测试框架
- 更多工具集成

---

## [1.4.0] - 2025-12-13

### 🎉 客户端 Phase 3 核心聊天功能完成

#### 数据层实现
- **Conversation 实体和模型**: 会话数据结构定义
- **Message 实体和模型**: 消息数据结构定义
- **ChatRepository**: 聊天数据仓库接口和实现
- **ChatRemoteDataSource**: 远程数据源

#### 业务层实现
- **GetConversationsUseCase**: 获取会话列表
- **GetMessagesUseCase**: 获取消息列表
- **SendMessageUseCase**: 发送消息
- **ConversationsProvider**: 会话列表状态管理
- **MessagesProvider**: 消息列表状态管理

#### UI层实现
- **ConversationTile**: 会话列表项组件，带微交互动效
- **ChatListPage**: 会话列表页面，带骨架屏加载
- **MessageBubble**: 消息气泡组件，带出现动画
- **MessageInput**: 消息输入框，带焦点动画
- **ChatPage**: 聊天页面

#### 3轮优化完成

**第1轮: 体验优化**
- 骨架屏加载效果 (8个渐变闪烁的占位符)
- 列表项滑入动画 (交错延迟淡入+上移)
- 下拉刷新震动反馈 (HapticFeedback)
- 空状态动画图标 (弹性缩放)

**第2轮: 美工优化**
- 消息出现动画 (滑入+淡入+缩放)
- 输入框焦点边框高亮
- 发送按钮弹性动画 (AnimatedSwitcher)
- 页面过渡动画工具类 (PageTransitions)

**第3轮: 代码优化**
- 会话项按压缩放动效 (0.98倍缩放)
- 通用状态组件库 (EmptyStateWidget/LoadingStateWidget/ErrorStateWidget)
- `flutter analyze` 零警告

#### 新增文件
- `page_transitions.dart` - 页面过渡动画工具
- `state_widgets.dart` - 通用状态组件库
- `conversation.dart` - 会话实体
- `message.dart` - 消息实体
- `conversation_model.dart` - 会话JSON模型
- `message_model.dart` - 消息JSON模型
- `chat_repository.dart` - 聊天仓库接口
- `chat_repository_impl.dart` - 聊天仓库实现
- `chat_remote_datasource.dart` - 远程数据源
- `conversations_provider.dart` - 会话状态管理
- `messages_provider.dart` - 消息状态管理
- `conversation_tile.dart` - 会话列表项组件
- `chat_list_page.dart` - 会话列表页面
- `message_bubble.dart` - 消息气泡组件
- `message_input.dart` - 消息输入框
- `chat_page.dart` - 聊天页面

#### 修改的文件
- `injection_config.dart` - 添加Chat模块依赖注入
- `app_router.dart` - 添加聊天页面路由
- `contacts_page.dart` - 使用新的空状态组件

#### 开发原则更新
- **无Mock原则**: 所有业务逻辑必须完整实现，真实API对接
- **阶段性优化原则**: 每完成一个阶段进行3轮优化（体验/美工/代码）

---

## [1.3.0] - 2025-12-12

### 🎉 客户端三轮优化完成

#### 第一轮: 功能修复
- **认证流程修复**: 登录/注册全流程正常工作
- **错误状态清除**: `AuthState.copyWith` 添加 `clearError` 参数
- **响应解析修复**: 正确提取服务器返回的 `data` 字段
- **字段名映射**: `online_status` 映射到 `status`
- **缺失字段处理**: `expiresIn` 设为可选，默认3600
- **Enum值匹配**: 服务器返回 "Online" → 客户端期望 "online"

#### 第二轮: UI优化 (QQ简洁风格)
- **主题色系优化**:
  - 主色: QQ蓝 `#12B7F5`
  - 成功色: 微信绿 `#07C160`
  - 警告色: `#FA9D3B`
  - 错误色: `#FA5151`
- **登录/注册页面重构**:
  - 渐变Logo图标 + 阴影效果
  - 简洁输入框样式（无边框填充背景）
  - 入场动画（淡入 + 上滑）
- **底部导航简化**:
  - 从4Tab简化为3Tab（消息/联系人/我）
  - 自定义导航栏样式
  - 选中状态动画切换

#### 第三轮: 架构稳健性
- **统一UI组件库**:
  - `AppButton`: 统一按钮组件，支持primary/secondary/text样式
  - `AppTextField`: 统一输入框，支持前缀/后缀图标
  - `AppToast`: Toast提示组件，支持success/error/warning/info
- **完善错误处理**:
  - 创建 `ResultHandler` 统一处理 Either 结果
  - 错误自动显示Toast提示
- **安全加固**:
  - 添加并发Token刷新锁
  - 防止多个请求同时刷新Token
- **代码规范**:
  - `flutter analyze` 零警告
  - 废弃API替换: `withOpacity` → `withValues(alpha:)`
  - 废弃API替换: `activeColor` → `activeThumbColor`
  - const构造函数优化
  - super parameters 使用

#### 修改的文件
- `auth_provider.dart` - clearError参数
- `user_model.dart` - online_status字段映射
- `auth_response_model.dart` - expiresIn可选
- `auth_remote_datasource.dart` - 提取data字段
- `app_theme.dart` - QQ风格色系
- `login_page.dart` / `register_page.dart` - 简洁UI
- `login_form.dart` / `register_form.dart` - 简洁表单
- `main_layout.dart` - 三Tab底部导航
- `exceptions.dart` - super parameters
- `theme_toggle.dart` - 修复废弃API
- `auth_repository_impl.dart` - const Left优化
- 新增 `widgets.dart` - 统一UI组件库导出
- 新增 `app_button.dart` - 统一按钮组件
- 新增 `app_text_field.dart` - 统一输入框
- 新增 `app_toast.dart` - Toast提示组件
- 新增 `result_handler.dart` - Either结果处理器

---

## [1.2.0] - 2025-12-11

### 🎉 Phase 4 AI Agent 智能增强完成

#### 用户自定义人设系统
- **PersonaConfig 完整配置**: 毒舌度/热情度/专业领域等基础配置
- **EmotionBoundaries**: 情绪边界设置，控制 Agent 情绪表达范围
- **StickerPreferences**: 贴纸偏好配置，支持情绪到贴纸的映射
- **VoiceSettings**: TTS 语音设置，包含语音ID/语速/音调
- **ResponseStyle/ResponseLength**: 回复风格和长度控制
- **PersonaPresets**: 预设人设模板 (friendly_assistant, sarcastic_bot 等)

#### 思维链系统 (Hidden CoT)
- **CoTProcessor**: 思维链处理器，提取和隐藏思考过程
- **CoTConfig**: 思维链配置，支持多种思考风格
- **ThinkingStyle**: 5种思考风格 (Analytical, Creative, Empathetic, Concise, Comprehensive)
- **CoTMarkers**: 可自定义的思考标记符
- **CoTResult**: 思考结果，包含思考内容和最终回复

#### 用户画像系统 (UserProfile)
- **UserProfile**: 用户画像模型，包含亲密度/尊重度/信任度
- **RelationshipLevel**: 6级关系识别 (Stranger→Distant→Neutral→Acquaintance→Friend→CloseFriend)
- **UserProfileService**: 用户画像服务，支持创建/更新/互动记录
- **InteractionEvent**: 互动事件记录
- **数据库迁移**: `agent_user_profiles` 和 `user_interaction_events` 表

#### 意图分类系统 (IntentClassifier)
- **Intent 枚举**: 10种意图类型 (Question, HelpRequest, Chitchat, Emotional, Command, Feedback, Greeting, Sharing, Humor, Unclear)
- **IntentClassifier**: 基于规则的意图分类器
- **IntentResult**: 分类结果，包含主意图和置信度
- **ResponseStrategy**: 回复策略生成 (should_be_informative, emotional_support, can_be_playful 等)
- **ResponsePriority**: 回复优先级 (Low, Normal, High, Urgent)

#### 多模态理解增强
- **MessageContent 扩展**: 新增 Audio/Video/File/Sticker 媒体类型
- **MultimodalContext**: 统一多模态上下文结构
- **MediaDescription**: 媒体描述结构体
- **process_content 增强**: 支持所有新媒体类型的处理
- **优雅降级**: 媒体处理失败时的回退机制

#### 主动对话系统
- **ProactiveTrigger 扩展**: 新增 RelationshipBased, DailyGreeting, CareMessage 触发类型
- **RelationshipTriggerReason**: 关系触发原因 (MissedCloseFriend, MoodChange, Milestone, ReconnectAttempt)
- **GreetingType**: 日常问候类型 (Morning, Afternoon, Evening, Weekend, Holiday)
- **CareReason**: 关心消息原因 (StressDetected, BusyPeriod, GeneralCheckIn, WeatherCare, HealthReminder)
- **RelationshipContext**: 关系上下文，用于个性化消息生成
- **generate_personalized_message**: 基于用户画像的个性化消息生成
- **evaluate_relationship_trigger**: 评估是否应发送关系型消息

#### API 端点新增
- `GET /api/ai/agents/presets`: 获取预设人设列表
- `PUT /api/ai/agents/:id/persona`: 更新 Agent 人设配置
- `POST /api/ai/agents/:id/persona/test`: 测试人设配置
- `GET /api/ai/agents/:id/profiles/:user_id`: 获取用户画像
- `PUT /api/ai/agents/:id/profiles/:user_id`: 更新用户画像

#### 数据库迁移
- `20251211000001_add_persona_customization.sql`: 人设自定义字段
- `20251211000002_create_user_profiles.sql`: 用户画像表

#### 测试
- **393+ 个测试通过**
- 新增模块测试: UserProfile, IntentClassifier, CoTProcessor, Proactive Enhanced

#### P.S.
phase 4 的所有新功能都复用了现有的错误码：
用户自定义人设 → 4017 INVALID_PERSONA
思维链系统 → 4000-4006 (LLM 相关错误)
用户画像系统 → 1000 INTERNAL_ERROR
意图分类系统 → 内部逻辑，无新错误
多模态增强 → 4000-4006 (LLM 相关错误)
主动对话系统 → 内部逻辑，无新错误


---

## [1.1.0] - 2025-12-09

### 🎉 Phase 3 AI Agent Evolution 完成

#### 核心架构
- **三层认知架构**: 感知层 → 认知层 → 执行层
- **AgentEvent/AgentAction**: 统一事件和动作枚举
- **AgentCore trait**: 核心处理接口
- **DefaultAgentCore**: 默认实现，支持工具调用和情绪分析

#### 国内服务商优先
- **Provider Manager**: 带 fallback 的多服务商管理
- **支持服务商**: 硅基流动、智谱、月之暗面、通义、文心
- **优先级配置**: 国内服务商优先于国际服务商

#### 语音交互
- **STT 服务**: Whisper API + 阿里云语音识别
- **TTS 服务**: Edge TTS + 阿里云 + OpenAI
- **音频分块**: 支持长音频自动分块处理

#### 智能消息总结
- **SummarizeService**: 消息总结服务
- **多种风格**: Brief, Detailed, Bullet, Narrative
- **缓存支持**: 避免重复生成

#### 语义搜索
- **SemanticSearchService**: 语义搜索服务
- **嵌入管道**: 异步嵌入生成和状态跟踪
- **向量搜索**: 基于相似度的消息检索
- **文本回退**: 向量搜索失败时回退到文本搜索

#### 工具调用系统
- **Tool trait**: 统一工具接口
- **ToolRegistry**: 工具注册和管理
- **内置工具**: 计算器、当前时间、天气查询

#### 情绪分析
- **EmotionAnalyzer trait**: 情绪分析接口
- **RuleBasedEmotionAnalyzer**: 基于规则的分析器
- **LLMEmotionAnalyzer**: 基于 LLM 的分析器
- **HybridEmotionAnalyzer**: 混合分析器

#### 触发评估增强
- **TriggerConfig**: 可配置触发参数
- **冷却机制**: 防止频繁触发
- **专业领域检测**: 基于领域的触发
- **直接回复检测**: 识别对 Agent 的回复

#### 记忆系统增强
- **InMemoryVectorStore**: 内存向量存储
- **EmbeddingMemoryStore**: 嵌入记忆存储
- **对话历史管理**: 限制和修剪

#### 主动消息
- **ProactiveMessenger**: 主动消息服务
- **沉默检测**: 检测群组沉默
- **活动模式分析**: 分析用户活动模式

#### 配额管理
- **QuotaManager**: 配额管理服务
- **多维度限制**: 每用户、每组、日限、月限
- **使用量跟踪**: 记录 token、音频秒数、成本

#### 响应流式传输
- **StreamingService**: 流式响应服务
- **StreamHandle**: 流控制句柄
- **中断支持**: 支持中断正在进行的流
- **消息队列**: 流式传输期间的消息队列

#### 多模态处理增强
- **MultiModalInput**: 图文组合输入
- **优雅降级**: 视觉处理失败时的回退
- **多图片支持**: 支持多张图片处理

#### 人设系统增强
- **VoiceSettings**: TTS 语音配置
- **EmotionBoundaries**: 情绪边界设置
- **StickerPreferences**: 贴纸偏好
- **ResponseStyle**: 响应风格设置
- **PersonaPresets**: 预设人设模板

#### 学习引擎
- **LearningEngine trait**: 学习引擎接口
- **反馈记录**: 记录用户反馈
- **模式学习**: 学习成功的响应模式
- **审核标记**: 标记需要审核的模式
- **数据匿名化**: 保护用户隐私

#### 数据库迁移
- `agent_emotion_history`: 情绪历史表
- `learning_patterns`: 学习模式表
- `user_feedback`: 用户反馈表
- `summary_cache`: 摘要缓存表
- `provider_usage`: 使用量跟踪表
- `ai_agent_persona_fields`: 人设字段扩展

#### 测试
- **357 个测试通过**
- **14 个集成测试**
- **28 个属性测试**

---

## [1.0.3] - 2025-12-08

### 📋 Phase 3 规划完成

#### 规范文档
- **需求文档**: 18 个需求，覆盖三层架构、语音交互、情绪分析等
- **设计文档**: 详细架构设计，28 个正确性属性
- **任务列表**: 18 个任务组，包含属性测试

#### 开发规范 (Steering Files)
- **开发规范**: `.kiro/steering/phase3-development-guidelines.md`
  - 项目结构规范
  - Trait 定义规范
  - 错误处理规范 (错误码 4007-4018)
  - API 响应规范
  - 测试规范 (proptest)
  - 国内服务商优先级
  - 中文支持规范
  
- **实现模式**: `.kiro/steering/phase3-implementation-patterns.md`
  - 三层架构模式
  - Provider Manager 模式
  - Tool 系统模式
  - 情绪分析模式
  - 配额管理模式
  - 学习引擎模式
  - Mock 实现模板
  - 属性测试模板

#### 错误码更新
- 新增 12 个 AI 服务错误码 (4007-4018)
- 更新 `archive/refactor/ERROR_CODES.md`

---

## [1.0.2] - 2025-12-08

### 🐛 Bug 修复

#### 后端修复
- **AI Agent 重复回复**: 修复 AI Agent 在群组中发送两条相同消息的问题
  - 移除 `AIMessageRouter` 中的重复发送代码
  - 在 `AgentResponseWorker` 中添加基于 `reply_to_message_id` 的去重机制
  - 添加详细日志追踪便于调试

#### 客户端修复
- **群消息实时更新**: 修复群组消息无法实时更新的问题
  - 修复数据库实例不一致问题（使用统一的 `userDatabaseProvider`）
  - 创建 `GlobalMessageHandler` 全局持久化 WebSocket 消息
  - `MessagesNotifier` 添加双重监听（持久化流 + WebSocket）
  
- **会话列表重复显示**: 修复同一群组或私聊会话重复显示的问题
  - 使用 `Map<String, Conversation>` 按 ID 去重
  - API 会话优先保留（数据更完整）
  - 为 `ListView.builder` 添加 `ValueKey` 确保正确复用

### 📝 文档
- 新增问题修复文档：
  - `archive/bugfix/AI_Agent重复回复问题.md`
  - `archive/bugfix/群消息实时更新问题.md`
  - `archive/bugfix/消息系统问题修复总结.md`
  - `archive/bugfix/会话列表重复显示问题.md`

---

## [1.0.1] - 2025-12-07

### 🔧 Phase 1 基础设施重构

#### 后端改进
- **统一错误处理**: 新增 `AppError` 枚举和 `error_codes` 模块
- **统一响应格式**: 新增 `ApiResponse<T>` 泛型结构
- **错误转换**: 实现 `From` trait 自动转换各类错误
- **属性测试**: 新增 18 个属性测试验证错误处理正确性

#### 前端改进
- **数据库用户隔离**: `DatabaseService` 实现一用户一库策略
- **错误拦截器**: `ApiClient` 添加 `_ErrorInterceptor`
- **Toast 工具**: 新增 `ToastUtils` 错误提示组件
- **连接状态**: 新增 `ConnectionStatusIndicator` WebSocket 状态组件
- **单元测试**: 新增 34 个前端单元测试

#### 文档
- **错误码文档**: 新增 `archive/refactor/ERROR_CODES.md`
- **项目清理**: 删除临时脚本和过时文件

### 📊 测试统计
- 后端属性测试: 18 个 ✅
- 前端单元测试: 34 个 ✅
- 总测试数: 579 个

---

## [1.0.0] - 2025-12-04

### 🎉 首次发布

完整的即时通讯系统，包含 AI Agent 集成。

### ✨ 新增功能

#### 后端系统
- **用户认证系统** (101 任务)
  - JWT 双令牌认证
  - 密码管理和重置
  - 用户资料管理
  - 在线状态追踪
  - 安全审计日志

- **消息系统** (78 任务)
  - WebSocket 实时通信
  - 消息路由和分发
  - 离线消息同步
  - Redis Pub/Sub 跨实例
  - 高并发处理 (1000+ msg/sec)

- **群聊管理系统** (15 任务)
  - 群组 CRUD 操作
  - 成员管理和权限
  - 邀请码系统
  - 角色管理（群主/管理员/成员）

- **AI Agent 系统** (53 任务) ⭐
  - 多 LLM 提供商支持
    - OpenAI (GPT-3.5, GPT-4)
    - Anthropic (Claude)
    - Google (Gemini)
    - SiliconFlow (国产模型)
  - RAG 记忆系统
  - 智能触发机制
  - 完整人设系统
  - 多模态能力

- **媒体处理系统** (19 任务)
  - 图片压缩和转换
  - 视频压缩
  - 缩略图生成
  - 多云存储支持
  - 文件去重

- **管理系统** (12 任务)
  - 用户角色管理
  - 账户状态管理
  - 系统统计信息
  - 审计日志查询

#### 客户端系统
- **Flutter 客户端** (28 任务)
  - 6 平台支持 (iOS/Android/macOS/Linux/Windows/Web)
  - Neo-Minimalism 设计
  - 本地数据库 (Drift)
  - 状态管理 (Riverpod)
  - 完整 UI 界面
  - 深色模式
  - 离线支持

#### 测试系统
- **单元测试**: 415 个
- **客户端测试**: 41 个
- **E2E 测试**: 25 个
- **性能测试**: 23 个
  - 负载测试 (6 个)
  - 混沌测试 (9 个)
  - 压力测试 (8 个)

#### 部署系统
- **Docker 支持**
  - 多阶段构建
  - 生产环境配置
  - 健康检查

- **Kubernetes 支持**
  - 完整部署配置
  - 自动扩缩容 (HPA)
  - Ingress 配置

- **监控系统**
  - Prometheus 指标收集
  - Grafana 可视化
  - Nginx 反向代理

- **CI/CD**
  - GitHub Actions 流水线
  - 自动化测试
  - 自动部署

### 📊 性能指标

- WebSocket 并发连接: 500+
- 消息吞吐量: 1000+ msg/sec
- Token 验证: 5000+ /sec
- API 响应时间: P99 < 100ms
- 测试速度: 30-60s (优化后)

### 📚 文档

- 项目概览和状态文档
- 完整开发环境配置
- 详细部署指南
- API 参考文档
- 测试指南
- 贡献指南
- 常见问题解答

### 🔧 技术栈

**后端**:
- Rust 1.75+
- Axum 0.7
- PostgreSQL 15
- Redis 7
- Qdrant 1.7

**客户端**:
- Flutter 3.16
- Riverpod
- Drift

**部署**:
- Docker
- Kubernetes
- Nginx
- Prometheus
- Grafana

### 📦 代码统计

- 总代码行数: ~36,000 行
- 测试数量: 504 个
- API 端点: 55+ 个
- 数据库表: 20 个
- 支持平台: 6 个

---

## 版本说明

### 版本号格式

`MAJOR.MINOR.PATCH`

- **MAJOR**: 不兼容的 API 变更
- **MINOR**: 向后兼容的功能新增
- **PATCH**: 向后兼容的问题修复

### 变更类型

- `Added` - 新增功能
- `Changed` - 功能变更
- `Deprecated` - 即将废弃的功能
- `Removed` - 已移除的功能
- `Fixed` - Bug 修复
- `Security` - 安全相关

---

## 路线图

### v1.1.0 (已完成) ✅
- [x] AI Agent 三层认知架构
- [x] 国内服务商优先
- [x] 语音交互 (STT/TTS)
- [x] 智能消息总结
- [x] 工具调用系统
- [x] 情绪分析

### v1.2.0 (已完成) ✅
- [x] 用户自定义人设系统
- [x] 思维链 (Hidden CoT)
- [x] 用户画像系统
- [x] 意图分类系统
- [x] 多模态增强
- [x] 主动对话系统

### v1.3.0 (已完成) ✅
- [x] 客户端基础架构
- [x] 认证系统 (登录/注册)
- [x] QQ简洁风格 UI
- [x] 统一UI组件库
- [x] 代码质量优化

### v1.4.0 (计划中)
- [ ] 客户端聊天功能
- [ ] 会话列表
- [ ] 消息收发
- [ ] 历史消息

### v1.5.0 (计划中)
- [ ] 联系人管理
- [ ] 群组功能
- [ ] AI Agent客户端集成

### v2.0.0 (未来)
- [ ] 端到端加密
- [ ] 去中心化架构
- [ ] 区块链集成
- [ ] Web3 功能

---

## 贡献者

感谢所有为项目做出贡献的人！

---

**最后更新**: 2025-12-12
