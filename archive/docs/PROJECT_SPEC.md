# Project Specification: AI-Native Minimalist IM

## 1. 项目愿景 (Project Vision)
构建一个极致轻量、美观且跨平台的即时通讯应用。
*   **核心理念**：极简（Minimalism）、AI 原生（AI-Native）、高性能（High Performance）。
*   **关键特性**：
    *   仅保留通讯录、个人个性主页、私聊与群聊。
    *   支持发送文字、表情、图片、视频。
    *   高度定制的审美。
    *   快速小巧。
    *   **AI 深度融合**：群聊中包含高度定制化的 AI Agent（拟人化、具备记忆、非指令式主动交互\[通过接入DeepSeek、OpenAI、Gemini、Qwen等API\]）。
*   **目标平台**：iOS, Android, Desktop (macOS/Windows/Linux).

## 2. 技术栈 (Tech Stack)

### 2.1 客户端 (Client) - Flutter
*   **Framework**: Flutter (Latest Stable).
*   **State Management**: **Riverpod** (更加健壮，适合 IM 复杂状态流).
*   **Local Database**: **Drift** (SQLite ORM) 或 **Isar** (高性能 NoSQL，推荐).
*   **Network**: `web_socket_channel` (WS 连接), `dio` (HTTP 请求).
*   **Serialization**: **Protobuf** (使用 `protobuf` dart 库).
*   **Media**: `ffmpeg_kit_flutter` (视频压缩), `flutter_image_compress`.
*   **UI Style**: Cupertino (iOS) & Material 3 (Android) 混合微调，追求极致留白与流畅动画。

### 2.2 服务端 (Server) - Rust
*   **Language**: Rust (Latest Stable).
*   **Web Framework**: **Axum** (轻量、人体工程学好、基于 Tokio).
*   **Async Runtime**: **Tokio**.
*   **Database ORM**: **SQLx** (编译时检查 SQL，性能极佳) 或 **SeaORM**.
*   **WebSockets**: `axum` extractors / `tokio-tungstenite`.
*   **Serialization**: `prost` (Protobuf implementation for Rust).
*   **AI/LLM Integration**: `reqwest` (调用 OpenAI/Claude/Qwen/DeepSeek API), `qdrant-client` (向量数据库交互).

### 2.3 基础设施 (Infrastructure)
*   **Database**: PostgreSQL (主要存储), Redis (会话缓存/PubSub).
*   **Vector DB**: Qdrant 或 Pgvector (用于 AI 长期记忆/RAG).
*   **Protocol**: Protocol Buffers (v3).

## 3. 目录结构 (Directory Structure)

建议采用 Monorepo 结构，方便 Protobuf 共享。

## 4. 通讯协议设计 (Protocol Buffers)

请在 `/protos/chat.proto` 中定义：


## 5. 核心功能开发规范 (Development Guidelines)

### 5.1 服务端开发规范 (Rust)
1.  **并发模型**：每个 WebSocket 连接生成一个 Tokio Task。使用 `Broadcast` channel 或 Redis Pub/Sub 进行消息分发。
2.  **错误处理**：使用 `thiserror` 定义统一错误枚举，使用 `anyhow` 处理不可恢复错误。
3.  **性能优化**：
    *   消息体尽量使用 Protobuf 二进制传输。
    *   数据库查询必须使用 `ConnectPool`。
4.  **AI Bot 逻辑 (The "Soul")**：
    *   **监听器**：群聊消息进入 pipeline 后，异步触发 AI 分析模块。
    *   **触发判定 (Trigger Logic)**：
        *   直接 @Bot -> 100% 触发。
        *   关键词命中 (Regex) -> 高概率触发。
        *   上下文相关性 (Vector Search 相似度 > 0.8) -> 中概率触发。
        *   随机游走 (Random) -> 极低概率触发 (模拟潜水员偶尔插嘴)。
    *   **延迟模拟**：AI 生成回复后，不要立即推送。计算 `len(response) * 50ms` 的延迟，发送 `Typing` 状态，然后再推送消息。

### 5.2 客户端开发规范 (Flutter)
1.  **UI/UX**：
    *   列表滚动必须保持 60/120fps，使用 `ListView.builder`。
    *   图片加载使用 `cached_network_image` 并配合服务端缩略图 API。
2.  **本地存储**：
    *   所有消息接收后立即存入本地 DB (Drift/Isar)。
    *   UI 数据源永远是本地 DB（Single Source of Truth），网络消息仅用于更新 DB。
3.  **媒体处理**：
    *   发送图片前：统一压缩至 WebP，最大边长不超过 1920px。
    *   发送视频前：使用 ffmpeg 压缩为 H.264/H.265，CRF 23-28。

## 核心亮点与竞争力

你的核心竞争力在于 **“AI与人的无缝融合”** 以及 **“反臃肿的用户体验”**。
### 6.1 AI 驱动：打造“真实群友” (核心差异化)

这是杀手级功能。不要把 Bot 做成“@Bot 也就是查天气”的工具人，而要让它像一个**有性格的潜水员**。

**1. 架构设计：Agent 而非单纯的 Chatbot**
*   **RAG (检索增强生成)：** 这一点至关重要。Bot 需要拥有“记忆”。当群友聊到“上次咱们说的那家拉面店”时，Bot 需要能通过向量数据库检索之前的聊天记录，接上话茬，而不是傻傻地问“哪家店？”。
*   **Personality System (人设系统)：** 允许群主高度定制 Bot。
    *   *参数：* 毒舌程度、热情程度、专业领域、说话口癖（比如句尾加“喵”）、对谁更友好。
    *   *提示词工程：* 后台需要一套复杂的 System Prompt 模板。

**2. 触发机制 (Triggering)**
*   **拒绝“每句必回”：** 真实的群友不会每一句话都回。
*   **主动插嘴模型：** 在服务端运行一个小型的分类模型或逻辑判断。
    *   只有当话题涉及 Bot 的“兴趣点”，或者群里冷场了，或者被明确 @ 时，Bot 才发言。
    *   引入**随机延迟**：Bot 收到消息后，不要秒回。根据回复长度计算模拟打字时间，甚至显示“对方正在输入...”，增加真实感。

**3. 多模态能力**
*   Bot 不仅能发文字，还能发表情包（从你的表情包库里检索最贴切的）、生成图片。

### 6.2 “美观”与 UI/UX 

**1. 设计风格：Neo-Minimalism (新极简主义)**
*   **留白：** 大量的留白，不要用复杂的分割线，用间距来区分模块。
*   **字体：** 跨平台也要保证字体美学。iOS 用 San Francisco，Android 用 Product Sans 或系统默认，但要调整字重和行高（Line-height 设为 1.5 倍会显得很高级）。
*   **微交互：**
    *   消息发出的“咻”声配合极轻微的触感反馈（Haptic Feedback）。
    *   长按消息时的缩放动画。
    *   AI 生成回复时的流式打字机效果等。

**2. 聊天体验细节**
*   **沉浸式：** 只有聊天，没有“发现”、“朋友圈”、“购物”。底部 Tab 最多两个：`消息` 和 `通讯录` (或者 `设置`)。
*   **引用与回复：** 参考 Telegram 的交互，丝滑的左滑回复。