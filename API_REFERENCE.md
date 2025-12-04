# 📡 API 参考文档

AI-Native IM 系统的完整 API 参考。

## 目录

- [认证 API](#认证-api)
- [用户 API](#用户-api)
- [消息 API](#消息-api)
- [群组 API](#群组-api)
- [AI Agent API](#ai-agent-api)
- [媒体 API](#媒体-api)
- [管理 API](#管理-api)
- [WebSocket API](#websocket-api)

---

## 基础信息

### Base URL
```
开发环境: http://localhost:8080
生产环境: https://api.yourdomain.com
```

### 认证方式
```http
Authorization: Bearer <access_token>
```

### 响应格式
所有 API 返回 JSON 格式：

**成功响应**:
```json
{
  "data": { ... },
  "message": "Success"
}
```

**错误响应**:
```json
{
  "error": "Error message",
  "code": "ERROR_CODE"
}
```

### HTTP 状态码
- `200` - 成功
- `201` - 创建成功
- `400` - 请求错误
- `401` - 未认证
- `403` - 无权限
- `404` - 资源不存在
- `429` - 请求过多
- `500` - 服务器错误

---

## 认证 API

### 注册用户

```http
POST /api/auth/register
```

**请求体**:
```json
{
  "username": "john_doe",
  "email": "john@example.com",
  "password": "SecurePass123!"
}
```

**响应**:
```json
{
  "data": {
    "user": {
      "id": "uuid",
      "username": "john_doe",
      "email": "john@example.com"
    },
    "access_token": "eyJ...",
    "refresh_token": "eyJ..."
  }
}
```

### 登录

```http
POST /api/auth/login
```

**请求体**:
```json
{
  "username": "john_doe",
  "password": "SecurePass123!"
}
```

**响应**:
```json
{
  "data": {
    "user": { ... },
    "access_token": "eyJ...",
    "refresh_token": "eyJ..."
  }
}
```

### 刷新令牌

```http
POST /api/auth/refresh
```

**请求体**:
```json
{
  "refresh_token": "eyJ..."
}
```

### 登出

```http
POST /api/auth/logout
Authorization: Bearer <token>
```

### 修改密码

```http
POST /api/auth/change-password
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "old_password": "OldPass123!",
  "new_password": "NewPass123!"
}
```

---

## 用户 API

### 获取当前用户信息

```http
GET /api/users/me
Authorization: Bearer <token>
```

**响应**:
```json
{
  "data": {
    "id": "uuid",
    "username": "john_doe",
    "email": "john@example.com",
    "avatar_url": "https://...",
    "bio": "Hello world",
    "status": "online",
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

### 更新个人资料

```http
PUT /api/users/me
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "username": "new_username",
  "avatar_url": "https://...",
  "bio": "New bio"
}
```

### 搜索用户

```http
GET /api/users/search?q=john
Authorization: Bearer <token>
```

**响应**:
```json
{
  "data": [
    {
      "id": "uuid",
      "username": "john_doe",
      "avatar_url": "https://..."
    }
  ]
}
```

### 获取用户资料

```http
GET /api/users/:user_id
Authorization: Bearer <token>
```

---

## 消息 API

### 发送私聊消息

```http
POST /api/messages/direct
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "receiver_id": "uuid",
  "content": "Hello!",
  "message_type": "text"
}
```

### 发送群聊消息

```http
POST /api/messages/group
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "group_id": "uuid",
  "content": "Hello everyone!",
  "message_type": "text"
}
```

### 获取消息历史

```http
GET /api/messages/history?conversation_id=uuid&limit=50&before=timestamp
Authorization: Bearer <token>
```

**查询参数**:
- `conversation_id` - 会话 ID（必需）
- `limit` - 返回数量（默认 50）
- `before` - 时间戳，获取此时间之前的消息

### 标记消息已读

```http
POST /api/messages/:message_id/read
Authorization: Bearer <token>
```

### 删除消息

```http
DELETE /api/messages/:message_id
Authorization: Bearer <token>
```

---

## 群组 API

### 创建群组

```http
POST /api/groups
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "name": "My Group",
  "description": "Group description",
  "avatar_url": "https://..."
}
```

### 获取群组列表

```http
GET /api/groups
Authorization: Bearer <token>
```

### 获取群组详情

```http
GET /api/groups/:group_id
Authorization: Bearer <token>
```

### 更新群组信息

```http
PUT /api/groups/:group_id
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "name": "New Name",
  "description": "New description"
}
```

### 添加成员

```http
POST /api/groups/:group_id/members
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "user_id": "uuid",
  "role": "member"
}
```

### 移除成员

```http
DELETE /api/groups/:group_id/members/:user_id
Authorization: Bearer <token>
```

### 生成邀请码

```http
POST /api/groups/:group_id/invite-code
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "expires_in": 86400,
  "max_uses": 10
}
```

### 通过邀请码加入

```http
POST /api/groups/join
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "invite_code": "ABC123"
}
```

---

## AI Agent API

### 创建 AI Agent

```http
POST /api/groups/:group_id/agent
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "name": "Assistant",
  "model": "gpt-4",
  "provider": "openai",
  "personality": {
    "toxicity": 0.3,
    "enthusiasm": 0.7,
    "expertise": ["tech", "coding"]
  },
  "triggers": {
    "mention": true,
    "keywords": ["help", "question"],
    "context": true
  }
}
```

### 获取 AI Agent 信息

```http
GET /api/groups/:group_id/agent
Authorization: Bearer <token>
```

### 更新 AI Agent

```http
PUT /api/groups/:group_id/agent
Authorization: Bearer <token>
```

### 删除 AI Agent

```http
DELETE /api/groups/:group_id/agent
Authorization: Bearer <token>
```

### 测试 AI Agent

```http
POST /api/groups/:group_id/agent/test
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "message": "Hello, can you help me?"
}
```

---

## 媒体 API

### 请求上传

```http
POST /api/media/upload-request
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "filename": "image.jpg",
  "content_type": "image/jpeg",
  "size": 1024000,
  "dedup_hash": "sha256_hash"
}
```

**响应**:
```json
{
  "data": {
    "upload_id": "uuid",
    "upload_url": "https://...",
    "file_exists": false
  }
}
```

### 完成上传

```http
POST /api/media/upload-complete
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "upload_id": "uuid"
}
```

### 获取媒体文件

```http
GET /api/media/:file_id
Authorization: Bearer <token>
```

### 删除媒体文件

```http
DELETE /api/media/:file_id
Authorization: Bearer <token>
```

---

## 管理 API

### 获取系统统计

```http
GET /api/admin/stats
Authorization: Bearer <admin_token>
```

**响应**:
```json
{
  "data": {
    "total_users": 1000,
    "active_users": 500,
    "total_messages": 50000,
    "total_groups": 100
  }
}
```

### 封禁用户

```http
POST /api/admin/users/:user_id/ban
Authorization: Bearer <admin_token>
```

**请求体**:
```json
{
  "reason": "Violation of terms",
  "duration": 86400
}
```

### 解封用户

```http
POST /api/admin/users/:user_id/unban
Authorization: Bearer <admin_token>
```

---

## WebSocket API

### 连接

```
ws://localhost:8080/ws?token=<access_token>
```

### 消息格式

**客户端 → 服务器**:
```json
{
  "type": "message",
  "data": {
    "conversation_id": "uuid",
    "content": "Hello",
    "message_type": "text"
  }
}
```

**服务器 → 客户端**:
```json
{
  "type": "message",
  "data": {
    "id": "uuid",
    "sender_id": "uuid",
    "content": "Hello",
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

### 事件类型

- `message` - 新消息
- `message_read` - 消息已读
- `typing` - 正在输入
- `presence` - 在线状态变化
- `group_update` - 群组更新

### 心跳

客户端应每 30 秒发送 ping：
```json
{
  "type": "ping"
}
```

服务器响应：
```json
{
  "type": "pong"
}
```

---

## 速率限制

| 端点类型 | 限制 |
|---------|------|
| 认证 API | 10 req/min |
| 普通 API | 100 req/min |
| 消息发送 | 50 msg/min |
| 媒体上传 | 10 uploads/min |

超过限制返回 `429 Too Many Requests`。

---

## 错误码

| 错误码 | 说明 |
|--------|------|
| `INVALID_CREDENTIALS` | 用户名或密码错误 |
| `TOKEN_EXPIRED` | 令牌已过期 |
| `INSUFFICIENT_PERMISSIONS` | 权限不足 |
| `RESOURCE_NOT_FOUND` | 资源不存在 |
| `VALIDATION_ERROR` | 验证失败 |
| `RATE_LIMIT_EXCEEDED` | 超过速率限制 |
| `INTERNAL_ERROR` | 服务器内部错误 |

---

## 示例代码

### cURL

```bash
# 注册
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"john","email":"john@example.com","password":"Pass123!"}'

# 登录
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"john","password":"Pass123!"}'

# 获取用户信息
curl http://localhost:8080/api/users/me \
  -H "Authorization: Bearer <token>"
```

### JavaScript (Fetch)

```javascript
// 登录
const response = await fetch('http://localhost:8080/api/auth/login', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    username: 'john',
    password: 'Pass123!'
  })
});

const data = await response.json();
const token = data.data.access_token;

// 发送消息
await fetch('http://localhost:8080/api/messages/direct', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${token}`
  },
  body: JSON.stringify({
    receiver_id: 'uuid',
    content: 'Hello!',
    message_type: 'text'
  })
});
```

### Python (requests)

```python
import requests

# 登录
response = requests.post('http://localhost:8080/api/auth/login', json={
    'username': 'john',
    'password': 'Pass123!'
})

token = response.json()['data']['access_token']

# 获取用户信息
headers = {'Authorization': f'Bearer {token}'}
response = requests.get('http://localhost:8080/api/users/me', headers=headers)
user = response.json()['data']
```

---

## 更多信息

- [完整部署指南](DEPLOYMENT.md)
- [开发环境配置](DEV_SETUP.md)
- [测试指南](server/TESTING.md)

**最后更新**: 2025-12-04
