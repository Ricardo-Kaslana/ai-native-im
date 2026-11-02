# AI-Native IM (AIIM)

![status](https://img.shields.io/badge/status-archived-lightgrey)
![license](https://img.shields.io/badge/license-Apache--2.0-blue)
![backend](https://img.shields.io/badge/backend-Rust%20%2F%20Axum-orange)
![client](https://img.shields.io/badge/client-Flutter-02569B)

> **English TL;DR** — AI-Native IM (AIIM) is an archived 2025–2026 experiment: a Rust/Axum + Flutter instant messenger with LLM-powered agents. It was built rapidly with heavy AI assistance ("vibe coding"), is incomplete and buggy, and is published for archival/reference only. **Not production-ready, not maintained, no support.** Licensed under Apache-2.0.

> **归档说明**
>
> 本项目已归档（2026-11-02），不再维护，也不接收 Issue / PR。
>
> 它是 2025 年的一次早期实验：在 AI 编程工具的高强度辅助下快速搭建，功能覆盖看似完整，但代码质量、完成度和一致性都存在明显问题。公开此仓库的目的仅是**存档与学习参考**，不是提供一个可用的即时通讯产品。
>
> **请勿将本项目部署到公网或用于生产环境。**

## 项目概览

AI-Native IM 试图把 AI Agent 作为“群成员”融入即时通讯：用户可以在群聊里创建具备人设、情绪、记忆和主动发言能力的 AI 角色，让它参与日常对话。

- **服务端**：Rust + Axum，PostgreSQL / Redis / 可选 Qdrant
- **客户端**：Flutter，覆盖 Android / iOS / Windows / Web
- **协议**：REST + WebSocket；`protos/` 中保留了服务端使用的 protobuf 定义
- **AI**：支持多家国内外 LLM 提供商，内置人设、情绪、记忆、主动消息、打字模拟等实验性模块

## 项目状态（诚实版）

| 模块 | 状态 | 说明 |
| --- | --- | --- |
| 用户认证 | 基本可用 | JWT 双令牌、bcrypt、限流、审计日志；密码重置邮件为 Mock |
| 实时消息 | 原型可用 | WebSocket（JSON）、私聊/群聊、离线落库、Redis Pub/Sub；客户端联调不完整 |
| 群组 / 邀请码 | 原型可用 | 群组 CRUD、成员角色、邀请码 |
| 联系人 / 会话 | 原型可用 | 搜索用户、添加联系人、会话列表 |
| AI Agent | 实验性 | 多提供商、人设、情绪、记忆、主动消息；依赖外部 API，行为不稳定 |
| 媒体处理 | 部分可用 | 本地头像/文件上传；腾讯 COS / 阿里 OSS provider 为占位实现 |
| 管理 / 指标 | 部分可用 | 管理接口与 Prometheus 指标骨架 |
| Flutter 客户端 | 部分可用 | 登录、会话、聊天、联系人、群组、设置等页面；部分接口未对齐 |
| 部署 / CI | 无 | Docker、K8s、Nginx、CI 配置已在归档前移除 |

> 服务端包含较多测试文件，但归档时**未重新执行构建或测试**；客户端 `test/` 目录为空。请勿把测试数量当作质量保证。

## 主要功能

### 认证与用户

- 注册、登录、登出、刷新令牌
- 修改密码、密码重置流程（邮件为 Mock，不会真正发送）
- bcrypt 密码哈希、JWT 访问/刷新令牌、登录限流
- 安全审计日志、用户资料与头像

### 消息与群组

- WebSocket 实时消息（客户端当前使用 JSON 消息，不是 protobuf 二进制）
- 私聊与群聊、消息落库、离线同步
- Redis Pub/Sub 多实例消息转发
- 群组 CRUD、成员角色（群主/管理员/成员）、邀请码
- 联系人、用户搜索、会话列表、未读数

### AI Agent（核心实验）

- LLM 提供商：SiliconFlow、智谱 AI、Moonshot（Kimi）、通义千问、文心一言、OpenAI、Anthropic、Google Gemini，以及自定义 OpenAI 兼容端点
- 人设配置：名称、专业领域、说话风格、毒舌度/热情度等
- 情绪系统、向量记忆、会话摘要、用户画像、主动消息、打字延迟模拟
- 流式响应、图片理解、STT/TTS 等模块骨架

> AI 模块是项目中最大也最不稳定的部分。多数能力依赖外部 API Key，部分模块只有接口或测试，未形成可靠的产品闭环。

### 媒体与客户端

- 本地头像/文件上传、访问控制、图片处理骨架
- Flutter 客户端包含登录、注册、会话列表、聊天、联系人、群组、个人资料、设置、AI Agent 等页面
- 支持 Android、iOS、Windows、Web；当前仓库没有 macOS/Linux 客户端工程
- 客户端使用 Riverpod + go_router + Dio + WebSocket

## 技术栈

**服务端**

- Rust 2021、Axum 0.7、Tokio
- SQLx 0.8 + PostgreSQL、Redis、可选 Qdrant
- Prost / Protocol Buffers、JWT、bcrypt
- SQLx migrations、cargo-nextest 配置、Makefile / 构建脚本

**客户端**

- Flutter 3.x、Dart ^3.9
- Riverpod、go_router、Dio、web_socket_channel
- flutter_secure_storage、shared_preferences、image_picker

**其他**

- 无 Docker、K8s、Nginx、CI 配置（历史中曾存在，归档前已移除）
- `protoc` 二进制不随仓库提供，需要自行下载

## 仓库结构

```text
.
├── server/          # Rust 服务端（API、服务层、数据库迁移、测试）
├── client/          # Flutter 客户端（Android / iOS / Windows / Web）
├── protos/          # Protocol Buffers 定义
├── template/        # 环境变量模板（历史配置，部分变量未使用）
├── docs/            # 现行文档 + archive 历史文档
├── CHANGELOG.md     # 开发期变更记录
├── README.md
└── LICENSE
```

以下内容仅存在于本地或被忽略，不会出现在仓库中：`server/.env`、`server/target/`、`client/build/`、`protoc.exe`、`_local-archive/`、`.workbuddy-ai/`。

## 快速开始（未验证）

> 以下步骤整理自开发期文档，归档时未重新验证。依赖版本较旧，构建可能失败；本项目不提供支持。

### 环境要求

- Rust（edition 2021）
- PostgreSQL 与 Redis（必需）
- Qdrant（可选，用于 AI 向量记忆）
- Flutter 3.x / Dart ^3.9
- `protoc`（构建服务端时需要）

### 服务端

```bash
# 1. 准备 PostgreSQL 数据库与 Redis
createdb ai_im

# 2. 配置环境变量
cd server
cp ../template/.env.example .env
# 编辑 .env：至少设置 DATABASE_URL、REDIS_URL、JWT_SECRET
# 如需 AI 功能，再配置一个 LLM API Key

# 3. 运行数据库迁移（需要 sqlx-cli）
sqlx migrate run

# 4. 启动服务端
# 需要 protoc：设置 PROTOC 环境变量，或将 protoc.exe 放到 server/ 下
cargo run
```

服务端默认监听 `8080` 端口，端口通过 `PORT` 环境变量覆盖。健康检查：`GET /health`。

### protoc 说明

`server/build.rs` 会编译 `protos/chat.proto`，需要 `protoc`：

```bash
# Windows：将 protoc.exe 放到 server/ 下后
server\build.bat

# 或手动设置
set PROTOC=path\to\protoc.exe
cargo build
```

请从 Protocol Buffers 官方 release 页面下载与你的环境匹配的 `protoc`。

### 客户端

```bash
cd client
flutter pub get

# 默认连接 http://localhost:8080
flutter run -d windows

# 覆盖 API 地址
flutter run -d windows --dart-define=API_BASE_URL=http://localhost:8080
```

Android 模拟器访问宿主机时使用 `http://10.0.2.2:8080`：

```bash
flutter run -d emulator-5554 --dart-define=API_BASE_URL=http://10.0.2.2:8080
```

## 配置

服务端通过环境变量配置，开发期使用 `server/.env`（已 gitignore）。主要变量：

| 变量 | 必需 | 说明 |
| --- | --- | --- |
| `DATABASE_URL` | 是 | PostgreSQL 连接串 |
| `REDIS_URL` | 是 | Redis 连接串 |
| `JWT_SECRET` | 是 | 32 位以上随机字符串；不要使用默认值 |
| `PORT` | 否 | HTTP 端口，默认 8080 |
| `QDRANT_URL` | 否 | 向量数据库地址，AI 记忆使用 |
| `STORAGE_PATH` | 否 | 本地上传文件目录，默认 `./storage` |
| `MEDIA_BASE_URL` | 否 | 媒体文件对外地址 |
| `SILICONFLOW_API_KEY` 等 | 否 | 各 LLM 提供商凭据；至少配置一个才能使用 AI |

`template/.env.example` 是开发期模板，包含重复配置块和部分当前代码未使用的变量（例如 `ADMIN_EMAIL` / `ADMIN_PASSWORD`），仅供历史参考。不要把其中的默认密码、占位密钥用于任何真实环境。

## 文档

- [API 文档](docs/API_DOC.md)
- [AI Agent 模块分析](docs/AI_AGENT_ANALYSIS.md)
- [AI Agent 工作流](docs/AI_AGENT_WORKFLOW.md)
- [文档索引](docs/README.md)
- [开发期 AI 笔记（可能过时）](docs/archive/CLAUDE.md)
- [变更日志](CHANGELOG.md)

历史文档保存在 `docs/archive/`，内容可能与当前代码不一致。

## 已知问题与限制

- 早期 AI 辅助开发，代码风格、错误处理、模块完成度不一致。
- 客户端与服务端接口/协议可能未完全对齐；部分页面和交互未完成。
- 客户端使用 JSON WebSocket；Dart protobuf 代码未生成或未接入。
- 腾讯 COS / 阿里 OSS 存储 provider 是占位实现，云媒体上传不可用。
- 邮件服务为 Mock，密码重置不会真正发送邮件。
- 管理后台、RBAC、配额、告警等只有部分骨架。
- 没有 Docker / CI / 部署配置；没有客户端测试；服务端测试未在归档时验证。
- 依赖版本较旧，`cargo build` / `flutter build` 可能失败。
- 存在硬编码默认值、重复代码、未使用代码和潜在安全问题。
- 没有国际化、无障碍适配和完整的移动端适配。
- **不适合任何生产或公网部署场景。**

## 安全与凭据

- 项目已归档，不提供安全维护；详见 [SECURITY.md](SECURITY.md)。
- 开发期历史提交和公开镜像曾暴露 API Key、JWT 密钥和服务器 IP。发布到 GitHub 前已重写 Git 历史清理这些值，因此提交哈希与旧仓库不同。
- 清理历史**不等于**凭据安全：任何曾公开过的 Key / Secret 都必须视为已泄漏，请立即作废并轮换。
- `server/.env` 被 gitignore；请勿提交任何真实凭据。默认 JWT 回退值和宽松 CORS 仅适合本地实验。

## 许可证

本项目使用 [Apache License 2.0](LICENSE)。版权与归属信息见 [NOTICE](NOTICE)。

## 致谢

- 2025–2026 年使用 AI 编程工具高强度辅助开发，属于典型的早期 “vibe coding” 实验。
- 感谢 Rust、Axum、Flutter、PostgreSQL、Redis、Qdrant 及所有开源依赖的作者。
- 文中出现的第三方名称与商标归其各自所有者所有。

本项目按“现状”提供，不附带任何明示或暗示的担保。
