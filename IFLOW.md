# AI 原生即时通讯应用 - iFlow CLI 上下文文档

## 项目概述

这是一个基于 Rust 后端和 Flutter 前端的 AI 原生即时通讯应用，核心特色是深度集成的 AI Agent 系统，提供"真实群友"般的智能聊天体验。项目采用极简设计理念，追求高性能和流畅的用户体验。

## 技术架构

### 后端 (Rust)
- **Web 框架**: Axum 0.7 + Tokio 异步运行时
- **数据库**: PostgreSQL (主数据库) + Redis (缓存/消息队列) + Qdrant (向量数据库)
- **序列化**: Protocol Buffers v3 (Prost)
- **认证**: JWT 双令牌机制 + bcrypt 密码加密

### 前端 (Flutter)
- **框架**: Flutter 3.24.5+ 
- **状态管理**: Riverpod
- **本地存储**: Hive + Drift (SQLite)
- **网络**: Dio + Retrofit + WebSocket

### 基础设施
- **容器化**: Docker + Docker Compose
- **存储**: 多云存储支持 (AWS S3, MinIO, CloudFlare R2, 腾讯COS, 阿里OSS)
- **监控**: Prometheus + Grafana

## 项目结构

```
ai-native-im/
├── server/              # Rust 后端服务
│   ├── src/            # 源代码
│   │   ├── api/        # API 端点层
│   │   ├── middleware/ # 中间件层
│   │   ├── models/     # 数据模型
│   │   ├── services/   # 业务服务层
│   │   └── ...
│   ├── migrations/     # 数据库迁移
│   ├── tests/          # 测试文件
│   └── Cargo.toml      # Rust 依赖配置
├── client/             # Flutter 客户端
│   ├── lib/            # Dart 源代码
│   ├── assets/         # 资源文件
│   └── pubspec.yaml    # Flutter 依赖配置
├── protos/             # Protocol Buffers 定义
├── k8s/                # Kubernetes 配置
├── scripts/            # 部署脚本
└── docs/               # 项目文档
```

## 核心功能模块

### 1. 用户认证系统 ✅ 100% 完成
- JWT 双令牌认证 (Access Token 15分钟 + Refresh Token 7天)
- 用户注册、登录、密码重置
- 账户安全 (暴力破解防护、账户锁定)
- 用户资料管理

### 2. 基础消息系统 ✅ 100% 完成
- WebSocket 实时消息传递
- 离线消息队列和同步
- 消息去重和确认机制
- 高并发处理 (1000+ msg/sec)

### 3. 群聊管理系统 ✅ 100% 完成
- 群组创建和设置
- 成员邀请 (直接邀请、邀请码)
- 角色权限管理 (群主、管理员、成员)
- 群组搜索和发现

### 4. AI Agent 系统 ⭐ 核心差异化 (100% 完成)
- **RAG 记忆系统**: 向量数据库存储对话历史，语义检索上下文
- **人设系统**: 高度定制化性格 (毒舌程度、热情度、专业领域)
- **智能触发**: @提及、关键词匹配、上下文相关、随机插话、冷场激活
- **多LLM提供商**: OpenAI, Anthropic Claude, Google Gemini, SiliconFlow
- **多模态能力**: 图片理解、视觉问答
- **打字模拟**: 真实延迟模拟

### 5. 媒体处理系统 ✅ 100% 完成
- 图片/视频压缩和转换
- 缩略图生成
- 多云存储支持
- 文件去重和恶意软件扫描

### 6. 客户端 (65% 完成)
- ✅ **认证流程**: 完整的登录/注册/自动登录
- ✅ **JWT令牌管理**: 自动刷新、安全存储
- ✅ **WebSocket连接**: 基础连接和心跳
- 🔄 **会话列表和聊天界面**: WebSocket完成，UI开发中
- ⏳ **联系人管理**: 待实现
- ⏳ **群组功能**: 待实现
- ⏳ **AI Agent 集成**: 待实现

## 开发环境配置

### 必需工具
- Rust 1.91.1+
- Flutter 3.24.5+
- Docker & Docker Compose
- PostgreSQL 15
- Redis 7
- Qdrant 1.7.4

### 快速启动

1. **启动基础服务**:
```bash
docker compose -f docker-compose.dev.yml up -d
```

2. **运行后端服务**:
```bash
cd server
cargo run
```

3. **运行客户端**:
```bash
cd client
flutter run
```

### 环境变量配置

后端环境变量 (`server/.env`):
```env
DATABASE_URL=postgresql://dev_user:dev_password@localhost:5432/ai_im_dev
REDIS_URL=redis://localhost:6379
QDRANT_URL=http://localhost:6333
JWT_SECRET=your-super-secret-jwt-key-change-in-production
OPENAI_API_KEY=your-openai-api-key
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
```

## 开发规范

### 代码风格
- **Rust**: 使用 `rustfmt` 和 `clippy`
- **Dart/Flutter**: 遵循官方代码规范，使用 `dart format`
- **命名约定**: 
  - Rust: snake_case (变量/函数), PascalCase (类型/结构体)
  - Dart: camelCase (变量/函数), PascalCase (类/类型)

### 错误处理
- **后端**: 统一使用 `AppError` 类型，明确的错误代码 (1000-4999)
- **前端**: 统一的异常处理机制，用户友好的错误提示

### API 设计
- RESTful API 设计原则
- 统一的响应格式 (`ApiResponse<T>`)
- Protocol Buffers 用于高效序列化

## 测试策略

### 后端测试
```bash
cd server
cargo test --lib --tests
```

### 前端测试
```bash
cd client
flutter test
```

### 集成测试
```bash
cd server
cargo test --test integration
```

## 部署

### 开发环境
```bash
./scripts/start-dev.sh
```

### 生产环境
```bash
docker compose -f docker-compose.prod.yml up -d
```

### Kubernetes
```bash
kubectl apply -f k8s/
```

## 性能指标

- **消息延迟**: < 100ms (p99)
- **并发连接**: 10,000+
- **消息吞吐**: 1000+ msg/sec
- **应用启动**: < 3秒 (冷启动)
- **页面切换**: < 500ms

## AI Agent 系统详解

### 核心能力
1. **记忆系统**: 基于向量数据库的长期记忆，支持语义检索
2. **人设配置**: 可调节的性格参数，包括毒舌度、热情度、专业领域等
3. **智能触发**: 多种触发机制，模拟真实用户行为
4. **多模态**: 支持图片理解和视觉问答
5. **学习引擎**: 基于用户反馈的持续优化

### 触发机制
- **@提及**: 100% 触发率
- **关键词匹配**: 60-80% 触发率
- **上下文相关**: 40-60% 触发率 (相似度 > 0.8)
- **随机插话**: 1-5% 触发率
- **冷场激活**: 10-20% 触发率 (2小时+无活动)

### LLM 提供商
- OpenAI (GPT-4, GPT-3.5)
- Anthropic Claude
- Google Gemini
- SiliconFlow (国产模型: Qwen, DeepSeek, GLM, Yi)
- 自定义/本地模型

## 常见问题

### Q: 如何重置数据库？
```bash
cd server
sqlx migrate revert
sqlx migrate run
```

### Q: 如何查看日志？
```bash
# Docker 服务日志
docker compose logs -f

# 特定服务日志
docker compose logs -f postgres
```

### Q: 如何添加新的 AI Provider？
在 `server/src/services/ai/providers/` 目录下创建新的 Provider 实现，并在 `ai_manager.rs` 中注册。

## 开发路线图

### Phase 1: 基础架构 ✅
- [x] 项目初始化和环境配置
- [x] 数据库设计和迁移
- [x] 基础 API 框架

### Phase 2: 核心功能 ✅
- [x] 用户认证系统
- [x] 消息系统
- [x] 群组管理
- [x] AI Agent 系统

### Phase 3: 客户端开发 🚧
- [ ] Flutter 项目架构搭建
- [ ] 认证界面和流程
- [ ] 聊天界面和功能
- [ ] 联系人和群组管理
- [ ] AI Agent 集成

### Phase 4: 优化和部署 ⏳
- [ ] 性能优化
- [ ] 安全加固
- [ ] 监控和日志
- [ ] 生产环境部署

## 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 联系方式

- 项目主页: https://gitee.com/sbgitee250/ai-native-im
- 问题反馈: https://gitee.com/sbgitee250/ai-native-im/issues

---

## 📝 总结

AI Native IM 是一个现代化的即时通讯应用，采用了先进的技术栈和架构设计，特别强调了 AI Agent 的集成，为用户提供智能化的聊天体验。

## 📅 更新日志

### 2025-12-12 - 客户端认证系统完成
- ✅ 完成客户端认证系统完整实现
  - Clean Architecture 分层架构
  - Riverpod 状态管理
  - JWT 令牌自动刷新
  - 自动登录功能
- ✅ WebSocket 基础连接完成
  - Protocol Buffers 集成
  - 心跳检测机制
- ✅ API 集成测试通过
  - 认证 API 测试
  - JWT 令牌刷新测试
  - WebSocket 连接测试
- 📚 文档更新
  - CLIENT_DEVELOPMENT_PLAN.md v2.0
  - CLIENT_INTEGRATION_CHECKLIST.md v2.0
  - 新增 API_INTEGRATION_GUIDE.md

**最后更新**: 2025-12-12