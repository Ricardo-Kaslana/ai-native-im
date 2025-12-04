# 🚀 AI Native IM - 即时通讯系统

一个功能完整、安全可靠的即时通讯系统，使用 Rust 后端 + Flutter 客户端构建。

> 📚 **完整文档导航**: [DOCS_INDEX.md](DOCS_INDEX.md) | **快速部署**: [QUICK_DEPLOY.md](QUICK_DEPLOY.md)

## 📊 项目状态

**总体进度**: ✅ 100% 完成 (7/7 个核心模块)  
**后端测试**: ✅ 415 个测试全部通过  
**客户端测试**: ✅ 17 个测试全部通过  
**代码行数**: ~35,000 行  
**最后更新**: 2025-12-04

### 模块完成情况

| 模块 | 状态 | 任务 |
|------|------|------|
| 用户认证系统 | ✅ 100% | 101/101 |
| 消息系统 | ✅ 100% | 78/78 |
| 管理系统 | ✅ 100% | 12/12 |
| 群聊管理系统 | ✅ 100% | 15/15 |
| AI Agent 系统 | ✅ 100% | 53/53 ⭐ |
| 媒体处理系统 | ✅ 100% | 19/19 |
| 客户端 UI 系统 | ✅ 100% | 28/28 🎉 |

## ✨ 核心功能

### 后端 (Rust)

#### 认证系统 ✅
- JWT 双令牌认证（访问令牌 + 刷新令牌）
- 密码管理（修改、重置、强度验证）
- 速率限制和防暴力破解
- 安全审计日志

#### 消息系统 ✅
- WebSocket 实时通信
- 消息路由（私聊/群聊）
- 离线消息同步
- Redis Pub/Sub 跨实例
- 高并发处理 (1000+ msg/sec)

#### 群聊系统 ✅
- 群组 CRUD 操作
- 成员管理和权限
- 邀请码系统
- 角色管理（群主/管理员/成员）

#### AI Agent 系统 ✅ ⭐ 核心差异化
- 多 LLM 提供商（OpenAI, Anthropic, Google, SiliconFlow）
- 国产模型支持（Qwen, DeepSeek, GLM, Yi）
- RAG 记忆系统（向量存储 + 语义搜索）
- 智能触发机制（5种触发类型）
- 完整人设系统（毒舌度、热情度、专业领域）
- 多模态能力（图片理解）
- 打字模拟（真实延迟）

#### 媒体处理系统 ✅
- 图片/视频压缩和转换
- 缩略图生成
- 多云存储（S3, MinIO, R2, 腾讯COS, 阿里OSS）
- 文件去重
- 恶意软件扫描

### 客户端 (Flutter) ✅ 新完成

#### 平台支持
- ✅ iOS
- ✅ Android
- ✅ macOS
- ✅ Linux
- ✅ Windows
- ✅ Web

#### 功能特性
- Neo-Minimalism 设计系统
- 深色模式 + OLED 优化
- 本地数据库（Drift）
- Riverpod 状态管理
- WebSocket 实时通信
- 媒体处理（选择、上传、查看）
- 离线支持
- 60/120fps 流畅动画

## 🛠️ 技术栈

### 后端
- **语言**: Rust
- **Web框架**: Axum
- **数据库**: PostgreSQL + SQLx
- **缓存**: Redis
- **向量数据库**: Qdrant
- **认证**: JWT + bcrypt
- **AI/LLM**: OpenAI, Anthropic, Google, SiliconFlow

### 客户端
- **框架**: Flutter 3.24.5
- **语言**: Dart 3.5.4
- **状态管理**: Riverpod
- **本地数据库**: Drift
- **网络**: Dio + WebSocket

## 🚀 快速开始

### 运行后端

```bash
# 1. 配置环境
cd server && cp .env.example .env

# 2. 运行迁移
sqlx migrate run

# 3. 启动服务器
cargo run
```

服务器运行在 `http://localhost:8080`

### 运行客户端

```bash
cd client

# 安装依赖
flutter pub get

# 运行应用
flutter run

# 或指定平台
flutter run -d macos
flutter run -d chrome
```

## 🧪 测试

### 后端测试

```bash
cd server

# 快速测试（推荐，30-60秒）
make test-fast

# 标准测试
cargo test --lib --tests
```

### 客户端测试

```bash
cd client

# 运行所有测试
flutter test

# 运行特定测试
flutter test test/widget/
flutter test test/integration/
```

### 测试覆盖

| 类别 | 数量 |
|------|------|
| 后端单元测试 | 415 |
| 客户端测试 | 41 |
| E2E 测试 | 25 |
| 性能测试 | 23 |
| **总计** | **504** |

## 📁 项目结构

```
ai-native-im/
├── server/                    # Rust 后端
│   ├── src/
│   │   ├── api/              # API 端点
│   │   ├── services/         # 业务服务
│   │   ├── models/           # 数据模型
│   │   ├── ai/               # AI Agent 系统
│   │   └── media/            # 媒体处理
│   └── tests/                # 测试文件
│
├── client/                    # Flutter 客户端
│   ├── lib/
│   │   ├── core/             # 核心功能
│   │   │   ├── constants/    # 常量
│   │   │   ├── theme/        # 主题
│   │   │   ├── network/      # 网络
│   │   │   └── utils/        # 工具
│   │   ├── data/             # 数据层
│   │   │   ├── local/        # 本地数据库
│   │   │   ├── models/       # 数据模型
│   │   │   └── repositories/ # 仓库
│   │   └── presentation/     # 表现层
│   │       ├── providers/    # 状态管理
│   │       ├── screens/      # 页面
│   │       └── widgets/      # 组件
│   └── test/                 # 测试文件
│
├── protos/                    # Protobuf 定义
└── .kiro/specs/              # 规范文档
```

## 📡 API 端点

### 认证 (7个)
- `POST /auth/register` - 用户注册
- `POST /auth/login` - 用户登录
- `POST /auth/logout` - 用户登出 🔒
- `POST /auth/refresh` - 刷新令牌
- `POST /auth/password/change` - 修改密码 🔒
- `POST /auth/password/reset/request` - 请求重置密码
- `POST /auth/password/reset/confirm` - 确认重置密码

### 用户 (5个) 🔒
- `GET /users/me` - 获取当前用户
- `PUT /users/me` - 更新用户资料
- `GET /users/:id` - 获取用户信息
- `GET /users/search` - 搜索用户
- `POST /users/batch` - 批量获取用户

### 群组 (17个) 🔒
- 群组 CRUD、成员管理、邀请码等

### 管理员 (6个) 🔒👑
- 用户管理、统计、审计日志等

🔒 = 需要认证 | 👑 = 需要管理员权限

## 🤖 AI Agent 快速开始

```bash
# 1. 配置 API Key
export OPENAI_API_KEY=sk-...
# 或使用国产模型
export SILICONFLOW_API_KEY=sk-...

# 2. 创建 AI Agent
curl -X POST http://localhost:8080/groups/{group_id}/agent \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "小助手",
    "toxicity_level": 5,
    "enthusiasm_level": 8,
    "expertise_domains": ["编程", "AI"]
  }'
```

## 📚 文档导航

### 🚀 快速开始
- [QUICK_DEPLOY.md](QUICK_DEPLOY.md) - 5分钟快速部署
- [CLIENT_QUICK_START.md](CLIENT_QUICK_START.md) - 客户端快速开始
- [FAQ.md](FAQ.md) - 常见问题解答

### 📖 核心文档
- [DOCS_INDEX.md](DOCS_INDEX.md) - 完整文档索引 ⭐
- [PROJECT_STATUS.md](PROJECT_STATUS.md) - 项目状态总览
- [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md) - 项目架构
- [CHANGELOG.md](CHANGELOG.md) - 更新日志

### 🛠️ 开发文档
- [DEV_SETUP.md](DEV_SETUP.md) - 开发环境配置
- [server/TESTING.md](server/TESTING.md) - 测试指南
- [API_REFERENCE.md](API_REFERENCE.md) - API 参考
- [CONTRIBUTING.md](CONTRIBUTING.md) - 贡献指南

### 🚢 部署文档
- [DEPLOYMENT.md](DEPLOYMENT.md) - 完整部署指南
- [docker-compose.prod.yml](docker-compose.prod.yml) - 生产配置
- [k8s/](k8s/) - Kubernetes 配置

### 开发文档
- [DEV_SETUP.md](DEV_SETUP.md) - 开发环境配置
- [CLIENT_QUICK_START.md](CLIENT_QUICK_START.md) - 客户端快速开始
- [server/TESTING.md](server/TESTING.md) - 测试指南

### 规范文档
- `.kiro/specs/user-auth-system/` - 用户认证系统
- `.kiro/specs/message-system/` - 消息系统
- `.kiro/specs/group-chat-system/` - 群聊系统
- `.kiro/specs/ai-agent-system/` - AI Agent 系统
- `.kiro/specs/media-processing-system/` - 媒体处理系统
- `.kiro/specs/client-ui-system/` - 客户端 UI 系统

## 🎉 里程碑

- ✅ 2025-11-30: 用户认证系统完成
- ✅ 2025-12-01: 消息系统完成
- ✅ 2025-12-01: 管理系统完成
- ✅ 2025-12-02: 群聊管理系统完成
- ✅ 2025-12-03: AI Agent 系统完成 ⭐
- ✅ 2025-12-04: 媒体处理系统完成
- ✅ 2025-12-04: 客户端 UI 系统完成 🎉
- ✅ 2025-12-04: **项目全部完成** 🎊

---

**项目状态**: ✅ 全部完成  
**完成日期**: 2025-12-04

