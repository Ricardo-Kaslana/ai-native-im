# AI-Native IM Backend API 完整文档

## 概述

AI-Native IM 是一个基于 Rust 的即时通讯后端系统，集成了先进的 AI Agent 功能、实时消息传递、媒体处理和用户管理。系统使用 Axum 框架构建，支持 WebSocket 连接，提供完整的 RESTful API。

## 核心功能

- **用户认证**: JWT 基础认证，支持注册、登录、密码重置
- **实时消息**: WebSocket 连接，支持文本、图片、视频消息
- **AI Agent**: 智能对话代理，支持个性化配置和学习
- **群组管理**: 完整的群组创建、成员管理、邀请系统
- **媒体处理**: 图片/视频上传、处理、去重
- **联系人系统**: 用户搜索、添加联系人
- **管理功能**: 用户管理、统计信息

## 服务器架构

```
┌─────────────────────────────────────────────────────────────┐
│                    AI-Native IM Server                      │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   Auth Service  │  │ Message Service │  │  AI Agent Service│ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │  Media Service  │  │ Group Service   │  │ Presence Service│ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                      Data Layer                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   PostgreSQL    │  │     Redis       │  │     Qdrant      │ │
│  │   (主数据库)     │  │   (缓存/会话)    │  │   (向量存储)     │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## API 端点总览

### 基础信息
- **Base URL**: `http://localhost:8080` (默认)
- **Content-Type**: `application/json`
- **Authentication**: Bearer Token (JWT)

### 健康检查
```http
GET /health
```
**响应**: `"OK"`

---

## 1. 认证 API (Authentication)

### 1.1 注册用户
```http
POST /auth/register
```

**请求体**:
```json
{
  "username": "string",
  "email": "string",
  "password": "string"
}
```

**响应**:
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "uuid",
      "username": "string",
      "email": "string",
      "avatar_url": "string|null",
      "bio": "string|null",
      "online_status": "offline|online|away",
      "created_at": "datetime"
    },
    "access_token": "string",
    "refresh_token": "string"
  },
  "error": null
}
```

### 1.2 用户登录
```http
POST /auth/login
```

**请求体**:
```json
{
  "username": "string", // 可以是邮箱或用户名
  "password": "string"
}
```

**响应**: 同注册响应

### 1.3 刷新令牌
```http
POST /auth/refresh
```

**请求体**:
```json
{
  "refresh_token": "string"
}
```

**响应**:
```json
{
  "success": true,
  "data": {
    "access_token": "string",
    "refresh_token": "string"
  }
}
```

### 1.4 用户登出 (需要认证)
```http
POST /auth/logout
Authorization: Bearer <token>
```

**响应**:
```json
{
  "success": true,
  "data": {
    "message": "Logged out successfully"
  }
}
```

### 1.5 修改密码 (需要认证)
```http
POST /auth/password/change
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "current_password": "string",
  "new_password": "string"
}
```

### 1.6 请求密码重置
```http
POST /auth/password/reset/request
```

**请求体**:
```json
{
  "email": "string"
}
```

### 1.7 确认密码重置
```http
POST /auth/password/reset/confirm
```

**请求体**:
```json
{
  "token": "string",
  "new_password": "string"
}
```

---

## 2. 用户管理 API (Users)

### 2.1 获取当前用户信息
```http
GET /users/me
Authorization: Bearer <token>
```

**响应**:
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "uuid",
      "username": "string",
      "email": "string",
      "avatar_url": "string|null",
      "bio": "string|null",
      "online_status": "offline|online|away",
      "created_at": "datetime"
    }
  }
}
```

### 2.2 更新当前用户信息
```http
PUT /users/me
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "username": "string|null",
  "avatar_url": "string|null",
  "bio": "string|null"
}
```

### 2.3 根据ID获取用户信息
```http
GET /users/{user_id}
Authorization: Bearer <token>
```

### 2.4 批量获取用户信息
```http
POST /users/batch
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "user_ids": ["uuid1", "uuid2"]
}
```

### 2.5 搜索用户
```http
GET /users/search?q={query}
Authorization: Bearer <token>
```

---

## 3. 群组管理 API (Groups)

### 3.1 创建群组
```http
POST /groups
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "name": "string",
  "description": "string|null"
}
```

**响应**:
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "name": "string",
    "description": "string|null",
    "owner_id": "uuid",
    "created_at": "datetime",
    "settings": {}
  }
}
```

### 3.2 获取群组详情
```http
GET /groups/{group_id}
Authorization: Bearer <token>
```

### 3.3 更新群组设置
```http
PUT /groups/{group_id}
Authorization: Bearer <token>
```

### 3.4 解散群组
```http
DELETE /groups/{group_id}
Authorization: Bearer <token>
```

### 3.5 获取用户群组
```http
GET /groups/my
Authorization: Bearer <token>
```

### 3.6 搜索群组
```http
GET /groups/search?q={query}&limit={number}
```

### 3.7 通过邀请码加入群组
```http
POST /groups/join
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "code": "string"
}
```

### 3.8 离开群组
```http
POST /groups/{group_id}/leave
Authorization: Bearer <token>
```

### 3.9 群组成员管理

#### 获取成员列表
```http
GET /groups/{group_id}/members?page={number}&limit={number}
Authorization: Bearer <token>
```

#### 添加成员
```http
POST /groups/{group_id}/members
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "user_id": "uuid"
}
```

#### 移除成员
```http
DELETE /groups/{group_id}/members/{user_id}
Authorization: Bearer <token>
```

#### 更新成员角色
```http
PUT /groups/{group_id}/members/{user_id}/role
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "role": "member|admin|owner"
}
```

#### 静音/取消静音成员
```http
POST /groups/{group_id}/members/{user_id}/mute
POST /groups/{group_id}/members/{user_id}/unmute
Authorization: Bearer <token>
```

#### 封禁/解封成员
```http
POST /groups/{group_id}/members/{user_id}/ban
POST /groups/{group_id}/members/{user_id}/unban
Authorization: Bearer <token>
```

### 3.10 邀请码管理

#### 获取邀请码列表
```http
GET /groups/{group_id}/invite-codes
Authorization: Bearer <token>
```

#### 生成邀请码
```http
POST /groups/{group_id}/invite-codes
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "expires_in_hours": 24,
  "max_uses": 1
}
```

**响应**:
```json
{
  "success": true,
  "data": {
    "code": "string",
    "invite_url": "string",
    "expires_at": "datetime",
    "max_uses": 1,
    "current_uses": 0
  }
}
```

#### 撤销邀请码
```http
DELETE /groups/{group_id}/invite-codes/{code}
Authorization: Bearer <token>
```

---

## 4. 媒体处理 API (Media)

### 4.1 请求图片上传
```http
POST /media/upload/image
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "file_size": 1024000,
  "mime_type": "image/jpeg"
}
```

**响应**:
```json
{
  "media_id": "uuid",
  "upload_url": "string",
  "expires_at": 1640995200
}
```

### 4.2 请求视频上传
```http
POST /media/upload/video
Authorization: Bearer <token>
```

### 4.3 完成上传
```http
POST /media/complete
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "media_id": "uuid",
  "file_hash": "sha256_hash"
}
```

**响应**:
```json
{
  "id": "uuid",
  "media_type": "image|video",
  "mime_type": "string",
  "url": "string",
  "thumbnail_url": "string|null",
  "width": 1920,
  "height": 1080,
  "duration": 120.5,
  "file_size": 1024000,
  "status": "processing|completed|failed",
  "created_at": "datetime"
}
```

### 4.4 获取媒体信息
```http
GET /media/{media_id}
```

### 4.5 获取缩略图
```http
GET /media/{media_id}/thumbnail
```

### 4.6 删除媒体
```http
DELETE /media/{media_id}
Authorization: Bearer <token>
```

---

## 5. AI Agent API

### 5.1 创建 AI Agent
```http
POST /groups/{group_id}/agent
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "name": "string",
  "persona": {
    "name": "string",
    "description": "string",
    "sarcasm_level": 50,
    "enthusiasm_level": 70,
    "expertise_domain": "string",
    "speech_pattern": "string",
    "response_style": "string",
    "forbidden_topics": ["topic1"],
    "preferred_topics": ["topic2"],
    "trigger_threshold": 30,
    "enable_typing_simulation": true,
    "typing_speed_wpm": 60,
    "enable_multimodal": true,
    "emotional_tone": "friendly",
    "catchphrases": ["phrase1"]
  }
}
```

### 5.2 获取群组 AI Agent
```http
GET /groups/{group_id}/agent
Authorization: Bearer <token>
```

### 5.3 获取 AI Agent 信息
```http
GET /agents/{agent_id}
Authorization: Bearer <token>
```

**响应**:
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "name": "string",
    "group_id": "uuid",
    "persona": {},
    "is_active": true,
    "created_at": "datetime",
    "updated_at": "datetime",
    "stats": {
      "total_messages": 1000,
      "total_interactions": 500,
      "average_response_time_ms": 1500,
      "user_satisfaction_score": 4.5,
      "active_days": 30,
      "last_interaction": "datetime"
    }
  }
}
```

### 5.4 更新 Agent 个性
```http
PUT /agents/{agent_id}/personality
Authorization: Bearer <token>
```

### 5.5 删除 AI Agent
```http
DELETE /agents/{agent_id}
Authorization: Bearer <token>
```

### 5.6 获取 Agent 统计信息
```http
GET /agents/{agent_id}/stats
Authorization: Bearer <token>
```

### 5.7 设置用户亲和度
```http
POST /agents/{agent_id}/affinity
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "user_id": "uuid",
  "affinity_score": 50
}
```

### 5.8 生成群组消息摘要
```http
POST /groups/{group_id}/summary
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "message_count": 100,
  "style": "brief|detailed|topics|action_items",
  "use_cache": true
}
```

**响应**:
```json
{
  "success": true,
  "data": {
    "summary": "string",
    "topics": ["topic1", "topic2"],
    "action_items": ["item1"],
    "generated_at": "datetime"
  }
}
```

### 5.9 记录反馈
```http
POST /messages/{message_id}/feedback
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "response_id": "uuid",
  "feedback_type": "positive|negative|correction|ignored",
  "correction": "string|null"
}
```

### 5.10 获取反馈统计
```http
GET /groups/{group_id}/feedback/stats
Authorization: Bearer <token>
```

### 5.11 获取个性预设
```http
GET /agents/presets
```

### 5.12 测试个性
```http
POST /agents/{agent_id}/test
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "message": "string",
  "sender_name": "string|null"
}
```

### 5.13 用户画像管理

#### 获取用户画像
```http
GET /agents/{agent_id}/profiles/{user_id}
Authorization: Bearer <token>
```

#### 更新用户画像
```http
PUT /agents/{agent_id}/profiles/{user_id}
Authorization: Bearer <token>
```

#### 列出所有用户画像
```http
GET /agents/{agent_id}/profiles
Authorization: Bearer <token>
```

---

## 6. WebSocket 实时通信

### 6.1 连接端点
```
ws://localhost:8080/ws?token=<jwt_token>
```

### 6.2 消息协议

#### 客户端发送消息 (ClientPacket)
```json
{
  "payload": {
    "message": {
      "id": "uuid",
      "sender_id": "uuid",
      "group_id": "uuid",
      "receiver_id": "uuid",
      "type": "TEXT|IMAGE|VIDEO|SYSTEM",
      "content": "string",
      "timestamp": "datetime",
      "is_ai_generated": false
    }
  }
}
```

#### 服务端推送消息 (ServerPacket)
```json
{
  "payload": {
    "message": {}, // ChatMessage
    "sync_response": {}, // SyncResponse
    "typing": {}, // TypingIndicator
    "ack": {}, // MessageAck
    "user_status_update": {}, // UserStatusUpdate
    "group_update": {}, // GroupUpdate
    "conversation_update": {}, // ConversationUpdate
    "presence_update": {}, // PresenceUpdate
    "upload_progress": {}, // MediaUploadProgress
    "ai_message": {}, // AIAgentMessage
    "system_notification": {} // SystemNotification
  }
}
```

### 6.3 消息类型详解

#### ChatMessage - 聊天消息
```json
{
  "id": "uuid",
  "sender_id": "uuid",
  "group_id": "uuid",
  "receiver_id": "uuid",
  "type": "TEXT|IMAGE|VIDEO|SYSTEM",
  "content": "string",
  "timestamp": "datetime",
  "is_ai_generated": false
}
```

#### Heartbeat - 心跳
```json
{
  "timestamp": "datetime"
}
```

#### SyncRequest - 同步请求
```json
{
  "last_sync_timestamp": "datetime",
  "user_id": "uuid"
}
```

#### TypingIndicator - 输入指示器
```json
{
  "user_id": "uuid",
  "group_id": "uuid",
  "is_typing": true
}
```

#### MessageAck - 消息确认
```json
{
  "message_id": "uuid",
  "success": true,
  "error": "string|null"
}
```

#### AIAgentMessage - AI Agent 消息
```json
{
  "agent_id": "uuid",
  "group_id": "uuid",
  "message": {}, // ChatMessage
  "thinking_process": "string|null",
  "confidence_score": 0.95,
  "triggered_keywords": ["keyword1"],
  "trigger_type": "AGENT_MENTION|KEYWORD|CONTEXTUAL|RANDOM|IDLE_ACTIVATION|PROACTIVE_GREETING"
}
```

#### PresenceUpdate - 在线状态更新
```json
{
  "user_id": "uuid",
  "status": "OFFLINE|ONLINE|AWAY",
  "status_message": "string|null",
  "is_mobile": false,
  "last_seen": "datetime"
}
```

---

## 7. 联系人 API (Contacts)

### 7.1 获取联系人列表
```http
GET /contacts
Authorization: Bearer <token>
```

**响应**:
```json
{
  "success": true,
  "data": {
    "contacts": [
      {
        "id": "uuid",
        "username": "string",
        "email": "string",
        "avatar_url": "string|null",
        "bio": "string|null",
        "is_online": true,
        "created_at": "datetime"
      }
    ]
  }
}
```

### 7.2 添加联系人
```http
POST /contacts
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "contact_id": "uuid"
}
```

---

## 8. 管理员 API (Admin)

### 8.1 获取统计信息
```http
GET /admin/stats
Authorization: Bearer <token>
```

**响应**:
```json
{
  "success": true,
  "data": {
    "total_users": 1000,
    "active_users": 500,
    "total_groups": 200,
    "total_messages": 50000,
    "total_ai_agents": 50,
    "last_updated": "datetime"
  }
}
```

### 8.2 获取用户列表
```http
GET /admin/users
Authorization: Bearer <token>
```

### 8.3 搜索用户
```http
GET /admin/users/search?q={query}
Authorization: Bearer <token>
```

### 8.4 暂停用户
```http
POST /admin/users/{user_id}/suspend
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "reason": "string|null",
  "duration_days": 7
}
```

### 8.5 激活用户
```http
POST /admin/users/{user_id}/activate
Authorization: Bearer <token>
```

### 8.6 删除用户
```http
DELETE /admin/users/{user_id}
Authorization: Bearer <token>
```

### 8.7 更改用户角色
```http
PUT /admin/users/{user_id}/role
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "role": "user|admin|moderator"
}
```

---

## 9. 对话 API (Conversations)

### 9.1 获取对话列表
```http
GET /conversations
Authorization: Bearer <token>
```

**响应**:
```json
{
  "success": true,
  "data": {
    "conversations": [
      {
        "id": "uuid",
        "conversation_type": "PRIVATE|GROUP",
        "name": "string|null",
        "participants": [
          {
            "id": "uuid",
            "username": "string",
            "avatar_url": "string|null"
          }
        ],
        "created_at": "datetime",
        "updated_at": "datetime"
      }
    ]
  }
}
```

### 9.2 创建对话
```http
POST /conversations
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "conversation_type": "PRIVATE|GROUP",
  "name": "string|null",
  "participant_ids": ["uuid1", "uuid2"]
}
```

---

## 10. API 响应格式

### 成功响应
```json
{
  "success": true,
  "data": {}, // 具体数据
  "error": null
}
```

### 错误响应
```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "ERROR_CODE",
    "message": "错误描述"
  }
}
```

### HTTP 状态码
- `200` - 成功
- `201` - 创建成功
- `400` - 请求参数错误
- `401` - 未认证
- `403` - 无权限
- `404` - 资源不存在
- `409` - 冲突
- `500` - 服务器内部错误

---

## 11. 数据库模型

### User (用户)
- `id`: UUID - 用户唯一标识
- `username`: String - 用户名
- `email`: String - 邮箱
- `password_hash`: String - 密码哈希
- `avatar_url`: Option<String> - 头像URL
- `bio`: Option<String> - 个人简介
- `online_status`: OnlineStatus - 在线状态
- `created_at`: DateTime - 创建时间
- `updated_at`: DateTime - 更新时间

### Group (群组)
- `id`: UUID - 群组唯一标识
- `name`: String - 群组名称
- `description`: Option<String> - 群组描述
- `owner_id`: UUID - 群主ID
- `created_at`: DateTime - 创建时间
- `updated_at`: DateTime - 更新时间

### GroupMember (群组成员)
- `group_id`: UUID - 群组ID
- `user_id`: UUID - 用户ID
- `role`: MemberRole - 成员角色
- `muted_until`: Option<DateTime> - 静音到期时间
- `joined_at`: DateTime - 加入时间

### AIAgent (AI Agent)
- `id`: UUID - Agent唯一标识
- `group_id`: UUID - 所属群组ID
- `name`: String - Agent名称
- `persona`: Jsonb - 个性配置
- `is_active`: Boolean - 是否激活
- `created_at`: DateTime - 创建时间
- `updated_at`: DateTime - 更新时间

### Message (消息)
- `id`: UUID - 消息唯一标识
- `sender_id`: UUID - 发送者ID
- `group_id`: Option<UUID> - 群组ID
- `receiver_id`: Option<UUID> - 接收者ID（私聊）
- `content`: String - 消息内容
- `message_type`: MessageType - 消息类型
- `is_ai_generated`: Boolean - 是否AI生成
- `created_at`: DateTime - 创建时间

---

## 12. 环境配置

### 必需的环境变量
```env
# 数据库
DATABASE_URL=postgresql://username:password@localhost:5432/database
REDIS_URL=redis://localhost:6379

# JWT
JWT_SECRET=your-secret-key

# AI 服务 (可选)
OPENAI_API_KEY=sk-xxx
ANTHROPIC_API_KEY=sk-ant-xxx
SILICONFLOW_API_KEY=sk-xxx

# 存储 (可选)
AWS_S3_BUCKET=your-bucket
AWS_ACCESS_KEY_ID=xxx
AWS_SECRET_ACCESS_KEY=xxx
```

### 可选配置
```env
# 服务器
PORT=8080

# 日志
RUST_LOG=ai_im_server=debug

# Qdrant (向量数据库)
QDRANT_URL=http://localhost:6333
```

---

## 13. 部署说明

### Docker 部署 (推荐)
```bash
# 构建镜像
docker build -t ai-native-im-server .

# 运行容器
docker run -d \
  --name ai-im-server \
  -p 8080:8080 \
  --env-file .env \
  ai-native-im-server
```

### 本地开发
```bash
# 安装依赖
cargo build

# 运行服务器
cargo run

# 开发模式 (热重载)
cargo install cargo-watch
cargo watch -x run
```

### 性能优化
- 使用 `cargo build --release` 进行生产构建
- 配置适当的数据库连接池大小
- 启用 Redis 缓存
- 配置反向代理 (nginx/caddy)

---

## 14. 故障排除

### 常见问题

1. **数据库连接失败**
   - 检查 `DATABASE_URL` 配置
   - 确保数据库服务正在运行

2. **Redis 连接失败**
   - 检查 `REDIS_URL` 配置
   - 确保 Redis 服务正在运行

3. **JWT 认证失败**
   - 检查 `JWT_SECRET` 配置
   - 确保令牌未过期

4. **AI Agent 无响应**
   - 检查 AI 服务 API 密钥
   - 查看相关日志错误信息

### 日志级别
```env
RUST_LOG=info          # 基础信息
RUST_LOG=debug         # 调试信息
RUST_LOG=trace         # 详细跟踪
```

---

## 15. SDK 示例

### Rust 示例
```rust
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // 登录
    let login_response = client
        .post("http://localhost:8080/auth/login")
        .json(&json!({
            "username": "user@example.com",
            "password": "password"
        }))
        .send()
        .await?;

    let token = login_response.json::<serde_json::Value>()
        .await?["data"]["access_token"].as_str().unwrap();

    // 获取用户信息
    let user_response = client
        .get("http://localhost:8080/users/me")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    println!("{:#?}", user_response.json::<serde_json::Value>().await?);

    Ok(())
}
```

### JavaScript 示例
```javascript
// 登录
const loginResponse = await fetch('http://localhost:8080/auth/login', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    username: 'user@example.com',
    password: 'password'
  })
});
const { data } = await loginResponse.json();
const token = data.access_token;

// 获取用户信息
const userResponse = await fetch('http://localhost:8080/users/me', {
  headers: {
    'Authorization': `Bearer ${token}`
  }
});
const userData = await userResponse.json();
console.log(userData);
```

---

## 16. 许可证

本项目采用 MIT 许可证。详见 [LICENSE](../LICENSE) 文件。

---

## 17. 贡献

欢迎贡献代码！请提交 Pull Request 或创建 Issue。

---

*最后更新: 2024-12-22*