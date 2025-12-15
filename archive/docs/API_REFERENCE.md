# 📡 API 参考文档

AI-Native IM 系统的完整 API 参考。

**版本**: 1.2  
**最后更新**: 2025-12-11

## 目录

- [基础信息](#基础信息)
- [认证 API](#认证-api)
- [用户 API](#用户-api)
- [消息 API](#消息-api)
- [群组 API](#群组-api)
- [AI Agent API](#ai-agent-api)
  - [Agent 基础管理](#agent-基础管理)
  - [人设系统 (Phase 4)](#人设系统-phase-4)
  - [用户画像 (Phase 4)](#用户画像-phase-4)
  - [消息总结 (Phase 3)](#消息总结-phase-3)
  - [学习反馈 (Phase 3)](#学习反馈-phase-3)
- [媒体 API](#媒体-api)
- [管理 API](#管理-api)
- [WebSocket API](#websocket-api)
- [错误码参考](#错误码参考)

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

### 统一响应格式

> ⚠️ **Phase 1 重构更新**: 所有 API 现在使用统一的响应格式

**成功响应**:
```json
{
  "success": true,
  "data": { ... }
}
```

**错误响应**:
```json
{
  "success": false,
  "error": {
    "code": 2003,
    "message": "Invalid credentials",
    "details": { ... }
  }
}
```

### HTTP 状态码

| 状态码 | 说明 |
|--------|------|
| `200` | 成功 |
| `201` | 创建成功 |
| `400` | 请求错误 |
| `401` | 未认证 |
| `403` | 无权限 |
| `404` | 资源不存在 |
| `408` | 请求超时 |
| `409` | 资源冲突 |
| `429` | 请求过多 |
| `500` | 服务器错误 |
| `502` | 网关错误 |
| `503` | 服务不可用 |
| `504` | 网关超时 |

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

**成功响应** (201):
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "uuid",
      "username": "john_doe",
      "email": "john@example.com",
      "created_at": "2025-12-08T00:00:00Z"
    },
    "access_token": "eyJ...",
    "refresh_token": "eyJ...",
    "expires_in": 3600
  }
}
```

**错误响应** (400):
```json
{
  "success": false,
  "error": {
    "code": 1001,
    "message": "Validation error",
    "details": {
      "field": "email",
      "reason": "Invalid email format"
    }
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

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "uuid",
      "username": "john_doe",
      "email": "john@example.com",
      "avatar_url": "https://...",
      "status": "online"
    },
    "access_token": "eyJ...",
    "refresh_token": "eyJ...",
    "expires_in": 3600
  }
}
```

**错误响应** (401):
```json
{
  "success": false,
  "error": {
    "code": 2003,
    "message": "Invalid credentials"
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

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "access_token": "eyJ...",
    "refresh_token": "eyJ...",
    "expires_in": 3600
  }
}
```

**错误响应** (401):
```json
{
  "success": false,
  "error": {
    "code": 2002,
    "message": "Token expired"
  }
}
```

### 登出

```http
POST /api/auth/logout
Authorization: Bearer <token>
```

**成功响应** (200):
```json
{
  "success": true,
  "data": null
}
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

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "message": "Password changed successfully"
  }
}
```

---

## 用户 API

### 获取当前用户信息

```http
GET /api/users/me
Authorization: Bearer <token>
```

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "username": "john_doe",
    "email": "john@example.com",
    "avatar_url": "https://...",
    "bio": "Hello world",
    "status": "online",
    "created_at": "2025-01-01T00:00:00Z",
    "updated_at": "2025-12-08T00:00:00Z"
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

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "username": "new_username",
    "avatar_url": "https://...",
    "bio": "New bio"
  }
}
```

### 搜索用户

```http
GET /api/users/search?q=john&limit=20
Authorization: Bearer <token>
```

**查询参数**:
| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| q | string | 是 | 搜索关键词 |
| limit | integer | 否 | 返回数量（默认 20，最大 100）|

**成功响应** (200):
```json
{
  "success": true,
  "data": [
    {
      "id": "uuid",
      "username": "john_doe",
      "avatar_url": "https://...",
      "bio": "Hello"
    }
  ]
}
```

### 获取用户资料

```http
GET /api/users/:user_id
Authorization: Bearer <token>
```

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "username": "john_doe",
    "avatar_url": "https://...",
    "bio": "Hello world",
    "status": "online"
  }
}
```

**错误响应** (404):
```json
{
  "success": false,
  "error": {
    "code": 3000,
    "message": "User not found"
  }
}
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
  "message_type": "text",
  "reply_to": null
}
```

**消息类型** (`message_type`):
- `text` - 文本消息
- `image` - 图片消息
- `video` - 视频消息
- `audio` - 音频消息
- `file` - 文件消息

**成功响应** (201):
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "sender_id": "uuid",
    "receiver_id": "uuid",
    "content": "Hello!",
    "message_type": "text",
    "created_at": "2025-12-08T00:00:00Z"
  }
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
  "message_type": "text",
  "reply_to": null,
  "mentions": ["uuid1", "uuid2"]
}
```

**成功响应** (201):
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "sender_id": "uuid",
    "group_id": "uuid",
    "content": "Hello everyone!",
    "message_type": "text",
    "mentions": ["uuid1", "uuid2"],
    "created_at": "2025-12-08T00:00:00Z"
  }
}
```

### 获取消息历史

```http
GET /api/messages/history?conversation_id=uuid&limit=50&before=timestamp
Authorization: Bearer <token>
```

**查询参数**:
| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| conversation_id | uuid | 是 | 会话 ID |
| limit | integer | 否 | 返回数量（默认 50，最大 100）|
| before | timestamp | 否 | 获取此时间之前的消息 |
| after | timestamp | 否 | 获取此时间之后的消息 |

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "messages": [
      {
        "id": "uuid",
        "sender_id": "uuid",
        "content": "Hello!",
        "message_type": "text",
        "created_at": "2025-12-08T00:00:00Z",
        "read_at": "2025-12-08T00:01:00Z"
      }
    ],
    "has_more": true
  }
}
```

### 标记消息已读

```http
POST /api/messages/:message_id/read
Authorization: Bearer <token>
```

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "message_id": "uuid",
    "read_at": "2025-12-08T00:00:00Z"
  }
}
```

### 删除消息

```http
DELETE /api/messages/:message_id
Authorization: Bearer <token>
```

**成功响应** (200):
```json
{
  "success": true,
  "data": null
}
```

**错误响应** (403):
```json
{
  "success": false,
  "error": {
    "code": 3003,
    "message": "Permission denied"
  }
}
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
  "avatar_url": "https://...",
  "is_public": false
}
```

**成功响应** (201):
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "name": "My Group",
    "description": "Group description",
    "avatar_url": "https://...",
    "owner_id": "uuid",
    "member_count": 1,
    "is_public": false,
    "created_at": "2025-12-08T00:00:00Z"
  }
}
```

### 获取群组列表

```http
GET /api/groups?page=1&limit=20
Authorization: Bearer <token>
```

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "groups": [
      {
        "id": "uuid",
        "name": "My Group",
        "avatar_url": "https://...",
        "member_count": 10,
        "last_message_at": "2025-12-08T00:00:00Z"
      }
    ],
    "total": 5,
    "page": 1,
    "limit": 20
  }
}
```

### 获取群组详情

```http
GET /api/groups/:group_id
Authorization: Bearer <token>
```

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "name": "My Group",
    "description": "Group description",
    "avatar_url": "https://...",
    "owner_id": "uuid",
    "member_count": 10,
    "is_public": false,
    "created_at": "2025-12-08T00:00:00Z",
    "my_role": "admin"
  }
}
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
  "description": "New description",
  "avatar_url": "https://..."
}
```

### 获取群组成员

```http
GET /api/groups/:group_id/members?page=1&limit=50
Authorization: Bearer <token>
```

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "members": [
      {
        "user_id": "uuid",
        "username": "john_doe",
        "avatar_url": "https://...",
        "role": "admin",
        "joined_at": "2025-12-08T00:00:00Z"
      }
    ],
    "total": 10
  }
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

**角色类型** (`role`):
- `owner` - 群主
- `admin` - 管理员
- `member` - 普通成员

### 移除成员

```http
DELETE /api/groups/:group_id/members/:user_id
Authorization: Bearer <token>
```

### 更新成员角色

```http
PUT /api/groups/:group_id/members/:user_id
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "role": "admin"
}
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

**成功响应** (201):
```json
{
  "success": true,
  "data": {
    "code": "ABC123XYZ",
    "expires_at": "2025-12-09T00:00:00Z",
    "max_uses": 10,
    "used_count": 0
  }
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
  "invite_code": "ABC123XYZ"
}
```

**错误响应** (400):
```json
{
  "success": false,
  "error": {
    "code": 3006,
    "message": "Invite expired"
  }
}
```

---

## AI Agent API

### Agent 基础管理

#### 创建 AI Agent

```http
POST /api/groups/:group_id/agent
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "name": "小助手",
  "provider": "siliconflow",
  "model": "Qwen/Qwen2.5-7B-Instruct",
  "personality": {
    "toxicity_level": 0.3,
    "enthusiasm_level": 0.7,
    "expertise_domains": ["tech", "coding", "general"],
    "speech_suffix": "喵~"
  },
  "triggers": {
    "mention": true,
    "keywords": ["help", "帮助", "问题"],
    "context_aware": true,
    "random_probability": 0.1,
    "idle_timeout_secs": 300
  }
}
```

**支持的 Provider**:
| Provider | 说明 | 优先级 |
|----------|------|--------|
| `siliconflow` | 硅基流动（推荐） | 1 |
| `zhipu` | 智谱 AI | 2 |
| `moonshot` | 月之暗面 | 3 |
| `tongyi` | 通义千问 | 4 |
| `wenxin` | 文心一言 | 5 |
| `openai` | OpenAI | 10 |
| `anthropic` | Anthropic | 11 |

**成功响应** (201):
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "group_id": "uuid",
    "name": "小助手",
    "provider": "siliconflow",
    "model": "Qwen/Qwen2.5-7B-Instruct",
    "is_active": true,
    "created_at": "2025-12-08T00:00:00Z"
  }
}
```

#### 获取 AI Agent 信息

```http
GET /api/groups/:group_id/agent
Authorization: Bearer <token>
```

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "group_id": "uuid",
    "name": "小助手",
    "provider": "siliconflow",
    "model": "Qwen/Qwen2.5-7B-Instruct",
    "personality": {
      "toxicity_level": 0.3,
      "enthusiasm_level": 0.7,
      "expertise_domains": ["tech", "coding"],
      "speech_suffix": "喵~"
    },
    "triggers": {
      "mention": true,
      "keywords": ["help", "帮助"],
      "context_aware": true
    },
    "is_active": true,
    "stats": {
      "total_messages": 1234,
      "avg_response_time_ms": 850
    }
  }
}
```

**错误响应** (404):
```json
{
  "success": false,
  "error": {
    "code": 4004,
    "message": "AI agent not found"
  }
}
```

#### 更新 AI Agent

```http
PUT /api/groups/:group_id/agent
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "name": "新名字",
  "personality": {
    "toxicity_level": 0.5
  },
  "is_active": true
}
```

#### 删除 AI Agent

```http
DELETE /api/groups/:group_id/agent
Authorization: Bearer <token>
```

#### 测试 AI Agent

```http
POST /api/groups/:group_id/agent/test
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "message": "你好，能帮我解答一个问题吗？"
}
```

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "response": "当然可以喵~ 请问有什么问题呢？",
    "trigger_reason": "keyword_match",
    "response_time_ms": 850
  }
}
```

**错误响应** (503):
```json
{
  "success": false,
  "error": {
    "code": 4007,
    "message": "All providers failed"
  }
}
```

#### 获取 AI Agent 记忆

```http
GET /api/groups/:group_id/agent/memories?limit=10
Authorization: Bearer <token>
```

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "memories": [
      {
        "id": "uuid",
        "content": "用户喜欢讨论技术话题",
        "importance": 0.8,
        "created_at": "2025-12-08T00:00:00Z"
      }
    ]
  }
}
```

---

### 人设系统 (Phase 4)

> ✨ **Phase 4 新功能**: 用户自定义人设系统，允许精细控制 AI Agent 的性格和行为。

#### 获取人设预设

```http
GET /api/agents/presets
Authorization: Bearer <token>
```

**描述**: 获取系统预设的人设模板列表。

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "presets": [
      {
        "id": "friendly_helper",
        "name": "友善助手",
        "description": "热情友善，乐于助人，语气温柔",
        "traits": ["热情", "耐心", "专业"],
        "toxicity_level": 0,
        "enthusiasm_level": 9,
        "sample_response": "很高兴能帮到你！请告诉我你需要什么帮助～"
      },
      {
        "id": "sarcastic_genius",
        "name": "毒舌天才",
        "description": "聪明但毒舌，喜欢用讽刺的方式说出真相",
        "traits": ["聪明", "毒舌", "直接"],
        "toxicity_level": 7,
        "enthusiasm_level": 3,
        "sample_response": "哦，这个问题吗？让我猜猜，你连文档都没看吧？"
      },
      {
        "id": "tech_expert",
        "name": "技术专家",
        "description": "专业严谨，擅长技术问题，回答精准",
        "traits": ["专业", "严谨", "细致"],
        "toxicity_level": 1,
        "enthusiasm_level": 6,
        "sample_response": "这个问题涉及到并发处理，建议使用互斥锁来避免竞争条件。"
      },
      {
        "id": "casual_friend",
        "name": "随性朋友",
        "description": "轻松随意，像朋友一样聊天",
        "traits": ["随性", "幽默", "亲切"],
        "toxicity_level": 2,
        "enthusiasm_level": 7,
        "sample_response": "哈哈，这个我懂！其实很简单啦，我来跟你说说。"
      }
    ]
  }
}
```

**预设字段说明**:
| 字段 | 类型 | 说明 |
|------|------|------|
| id | string | 预设 ID |
| name | string | 预设名称 |
| description | string | 预设描述 |
| traits | string[] | 性格特质列表 |
| toxicity_level | integer | 毒舌等级 (0-10) |
| enthusiasm_level | integer | 热情等级 (0-10) |
| sample_response | string | 示例回复 |

#### 更新 AI Agent 人设

```http
PUT /api/agents/:agent_id/personality
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "traits": ["幽默", "专业", "热心"],
  "toxicity_level": 3,
  "enthusiasm_level": 8,
  "custom_prompt": "你是一个技术极客，热爱编程，喜欢用简洁的代码解决问题。"
}
```

**字段说明**:
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| traits | string[] | 否 | 性格特质列表 (最多 5 个) |
| toxicity_level | integer | 否 | 毒舌等级 (0-10，0=温柔，10=极度毒舌) |
| enthusiasm_level | integer | 否 | 热情等级 (0-10，0=冷淡，10=极度热情) |
| custom_prompt | string | 否 | 自定义人设描述 (最多 500 字) |

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "agent_id": "uuid",
    "personality": {
      "traits": ["幽默", "专业", "热心"],
      "toxicity_level": 3,
      "enthusiasm_level": 8,
      "custom_prompt": "你是一个技术极客..."
    },
    "system_prompt": "你是 AI助手，性格特点：幽默、专业、热心...",
    "updated_at": "2025-12-11T00:00:00Z"
  }
}
```

**错误响应** (400):
```json
{
  "success": false,
  "error": {
    "code": 4017,
    "message": "Invalid persona",
    "details": {
      "field": "toxicity_level",
      "reason": "Must be between 0 and 10"
    }
  }
}
```

#### 测试人设配置

```http
POST /api/agents/:agent_id/test
Authorization: Bearer <token>
```

**描述**: 在不保存的情况下测试人设配置的效果。

**请求体**:
```json
{
  "message": "你好，能帮我解答一个问题吗？",
  "test_personality": {
    "traits": ["毒舌", "聪明"],
    "toxicity_level": 7,
    "enthusiasm_level": 3
  }
}
```

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "response": "哦，又是一个问题。说吧，什么问题？不过我猜你可能连搜索都懒得搜。",
    "personality_used": {
      "traits": ["毒舌", "聪明"],
      "toxicity_level": 7,
      "enthusiasm_level": 3
    },
    "response_time_ms": 850
  }
}
```

---

### 用户画像 (Phase 4)

> ✨ **Phase 4 新功能**: AI 会自动学习和记录用户的性格、兴趣和交互习惯，实现个性化交互。

#### 获取用户画像

```http
GET /api/agents/:agent_id/profiles/:user_id
Authorization: Bearer <token>
```

**描述**: 获取 AI Agent 为特定用户生成的画像信息。

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "user_id": "uuid",
    "agent_id": "uuid",
    "personality_traits": ["外向", "好奇", "技术爱好者"],
    "interests": ["编程", "AI", "游戏", "音乐"],
    "communication_style": "直接且简洁，喜欢技术讨论",
    "activity_pattern": {
      "most_active_hours": [9, 10, 14, 20, 21],
      "preferred_days": ["Monday", "Wednesday", "Friday"],
      "avg_messages_per_day": 15
    },
    "emotional_tendency": "积极乐观，偶尔焦虑",
    "interaction_history": {
      "total_messages": 245,
      "avg_response_time_secs": 120,
      "favorite_topics": ["后端开发", "Rust", "数据库"],
      "last_interaction": "2025-12-11T00:00:00Z"
    },
    "relationship_level": "GoodFriend",
    "intimacy_score": 68,
    "created_at": "2025-11-01T00:00:00Z",
    "updated_at": "2025-12-11T00:00:00Z"
  }
}
```

**关系等级** (`relationship_level`):
| 等级 | 分数范围 | 说明 |
|------|---------|------|
| Stranger | 0-20 | 陌生人 |
| Acquaintance | 21-40 | 熟人 |
| Friend | 41-60 | 朋友 |
| GoodFriend | 61-80 | 好友 |
| CloseFriend | 81-100 | 密友 |
| BestFriend | 100+ | 至友 |

#### 更新用户画像

```http
PUT /api/agents/:agent_id/profiles/:user_id
Authorization: Bearer <token>
```

**描述**: 手动更新用户画像信息（通常由 AI 自动更新）。

**请求体**:
```json
{
  "interests": ["编程", "AI", "游戏", "音乐", "电影"],
  "communication_style": "直接且简洁",
  "notes": "用户最近对机器学习感兴趣"
}
```

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "user_id": "uuid",
    "updated_fields": ["interests", "communication_style", "notes"],
    "updated_at": "2025-12-11T00:00:00Z"
  }
}
```

#### 列出所有用户画像

```http
GET /api/agents/:agent_id/profiles?page=1&limit=20&min_relationship=Friend
Authorization: Bearer <token>
```

**查询参数**:
| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| page | integer | 否 | 页码（默认 1） |
| limit | integer | 否 | 每页数量（默认 20，最大 100） |
| min_relationship | string | 否 | 最低关系等级过滤 |
| sort_by | string | 否 | 排序字段（intimacy_score, last_interaction） |

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "profiles": [
      {
        "user_id": "uuid",
        "username": "john_doe",
        "relationship_level": "GoodFriend",
        "intimacy_score": 68,
        "interests": ["编程", "AI"],
        "last_interaction": "2025-12-11T00:00:00Z"
      }
    ],
    "total": 15,
    "page": 1,
    "limit": 20
  }
}
```

---

### 消息总结 (Phase 3)

> 📝 **Phase 3 功能**: AI 可以自动生成对话总结，帮助用户快速了解历史讨论。

#### 生成消息总结

```http
POST /api/groups/:group_id/summary
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "message_count": 100,
  "style": "detailed",
  "use_cache": true
}
```

**总结风格** (`style`):
| 风格 | 说明 |
|------|------|
| brief | 简洁总结（3-5 句话） |
| detailed | 详细总结（分段描述） |
| topics | 按话题分类 |
| action_items | 提取行动项 |

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "summary_id": "uuid",
    "group_id": "uuid",
    "content": "过去 100 条消息中，大家主要讨论了项目的数据库设计和 API 接口规范...",
    "style": "detailed",
    "message_count": 100,
    "time_range": {
      "start": "2025-12-10T00:00:00Z",
      "end": "2025-12-11T00:00:00Z"
    },
    "key_topics": ["数据库设计", "API 规范", "性能优化"],
    "participants": ["alice", "bob", "charlie"],
    "created_at": "2025-12-11T00:00:00Z",
    "cached": false
  }
}
```

---

### 学习反馈 (Phase 3)

> 🎯 **Phase 3 功能**: 用户可以对 AI 回复进行反馈，AI 会根据反馈持续学习和改进。

#### 提交反馈

```http
POST /api/messages/:message_id/feedback
Authorization: Bearer <token>
```

**请求体**:
```json
{
  "response_id": "uuid",
  "feedback_type": "helpful",
  "correction": null
}
```

**反馈类型** (`feedback_type`):
| 类型 | 说明 |
|------|------|
| helpful | 有帮助 |
| unhelpful | 没帮助 |
| incorrect | 信息错误 |
| inappropriate | 不恰当 |
| too_verbose | 过于冗长 |
| too_brief | 过于简略 |

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "recorded": true,
    "message": "感谢反馈！AI 将根据您的反馈持续改进。"
  }
}
```

#### 获取反馈统计

```http
GET /api/groups/:group_id/feedback/stats
Authorization: Bearer <token>
```

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "total_feedbacks": 156,
    "breakdown": {
      "helpful": 98,
      "unhelpful": 12,
      "incorrect": 8,
      "inappropriate": 2,
      "too_verbose": 20,
      "too_brief": 16
    },
    "satisfaction_rate": 0.78,
    "recent_improvements": [
      "回复长度更加合适",
      "技术细节更准确"
    ]
  }
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
  "dedup_hash": "sha256_hash_of_file"
}
```

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "upload_id": "uuid",
    "upload_url": "https://storage.example.com/upload?token=...",
    "file_exists": false,
    "expires_at": "2025-12-08T01:00:00Z"
  }
}
```

**文件已存在响应** (200):
```json
{
  "success": true,
  "data": {
    "upload_id": null,
    "file_exists": true,
    "file_id": "uuid",
    "url": "https://cdn.example.com/files/uuid"
  }
}
```

**错误响应** (400):
```json
{
  "success": false,
  "error": {
    "code": 3008,
    "message": "Media too large",
    "details": {
      "max_size": 104857600,
      "actual_size": 209715200
    }
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

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "file_id": "uuid",
    "url": "https://cdn.example.com/files/uuid",
    "thumbnail_url": "https://cdn.example.com/thumbs/uuid",
    "content_type": "image/jpeg",
    "size": 1024000
  }
}
```

### 获取媒体文件信息

```http
GET /api/media/:file_id
Authorization: Bearer <token>
```

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "filename": "image.jpg",
    "url": "https://cdn.example.com/files/uuid",
    "thumbnail_url": "https://cdn.example.com/thumbs/uuid",
    "content_type": "image/jpeg",
    "size": 1024000,
    "created_at": "2025-12-08T00:00:00Z"
  }
}
```

### 删除媒体文件

```http
DELETE /api/media/:file_id
Authorization: Bearer <token>
```

---

## 管理 API

> ⚠️ 以下 API 需要管理员权限

### 获取系统统计

```http
GET /api/admin/stats
Authorization: Bearer <admin_token>
```

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "users": {
      "total": 1000,
      "active_today": 500,
      "new_today": 50
    },
    "messages": {
      "total": 500000,
      "today": 10000
    },
    "groups": {
      "total": 100,
      "active": 80
    },
    "ai_agents": {
      "total": 50,
      "active": 45,
      "messages_today": 5000
    },
    "storage": {
      "total_bytes": 10737418240,
      "used_bytes": 5368709120
    }
  }
}
```

### 获取用户列表

```http
GET /api/admin/users?page=1&limit=20&status=active
Authorization: Bearer <admin_token>
```

**查询参数**:
| 参数 | 类型 | 说明 |
|------|------|------|
| page | integer | 页码 |
| limit | integer | 每页数量 |
| status | string | 状态筛选：active, suspended, banned |
| q | string | 搜索关键词 |

### 封禁用户

```http
POST /api/admin/users/:user_id/ban
Authorization: Bearer <admin_token>
```

**请求体**:
```json
{
  "reason": "Violation of terms",
  "duration_secs": 86400
}
```

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "user_id": "uuid",
    "status": "banned",
    "banned_until": "2025-12-09T00:00:00Z",
    "reason": "Violation of terms"
  }
}
```

### 解封用户

```http
POST /api/admin/users/:user_id/unban
Authorization: Bearer <admin_token>
```

### 获取审计日志

```http
GET /api/admin/audit-logs?page=1&limit=50
Authorization: Bearer <admin_token>
```

**成功响应** (200):
```json
{
  "success": true,
  "data": {
    "logs": [
      {
        "id": "uuid",
        "user_id": "uuid",
        "action": "user_login",
        "ip_address": "192.168.1.1",
        "user_agent": "Mozilla/5.0...",
        "details": {},
        "created_at": "2025-12-08T00:00:00Z"
      }
    ],
    "total": 1000
  }
}
```

---

## WebSocket API

### 连接

```
ws://localhost:8080/ws?token=<access_token>
```

或使用 Header:
```
ws://localhost:8080/ws
Authorization: Bearer <access_token>
```

### 消息格式

所有 WebSocket 消息使用 JSON 格式：

```json
{
  "type": "message_type",
  "data": { ... },
  "timestamp": "2025-12-08T00:00:00Z"
}
```

### 客户端 → 服务器 消息类型

#### 发送消息
```json
{
  "type": "send_message",
  "data": {
    "conversation_id": "uuid",
    "content": "Hello",
    "message_type": "text",
    "reply_to": null
  }
}
```

#### 标记已读
```json
{
  "type": "mark_read",
  "data": {
    "conversation_id": "uuid",
    "message_id": "uuid"
  }
}
```

#### 正在输入
```json
{
  "type": "typing",
  "data": {
    "conversation_id": "uuid",
    "is_typing": true
  }
}
```

#### 心跳
```json
{
  "type": "ping"
}
```

### 服务器 → 客户端 消息类型

#### 新消息
```json
{
  "type": "new_message",
  "data": {
    "id": "uuid",
    "conversation_id": "uuid",
    "sender_id": "uuid",
    "sender_name": "john_doe",
    "content": "Hello",
    "message_type": "text",
    "created_at": "2025-12-08T00:00:00Z"
  }
}
```

#### AI Agent 消息
```json
{
  "type": "agent_message",
  "data": {
    "id": "uuid",
    "conversation_id": "uuid",
    "agent_id": "uuid",
    "agent_name": "小助手",
    "content": "你好喵~",
    "reply_to": "uuid",
    "created_at": "2025-12-08T00:00:00Z"
  }
}
```

#### 消息已读回执
```json
{
  "type": "message_read",
  "data": {
    "message_id": "uuid",
    "reader_id": "uuid",
    "read_at": "2025-12-08T00:00:00Z"
  }
}
```

#### 用户正在输入
```json
{
  "type": "user_typing",
  "data": {
    "conversation_id": "uuid",
    "user_id": "uuid",
    "username": "john_doe",
    "is_typing": true
  }
}
```

#### 在线状态变化
```json
{
  "type": "presence",
  "data": {
    "user_id": "uuid",
    "status": "online",
    "last_seen": "2025-12-08T00:00:00Z"
  }
}
```

#### 群组更新
```json
{
  "type": "group_update",
  "data": {
    "group_id": "uuid",
    "update_type": "member_joined",
    "details": {
      "user_id": "uuid",
      "username": "new_member"
    }
  }
}
```

#### 心跳响应
```json
{
  "type": "pong"
}
```

#### 错误
```json
{
  "type": "error",
  "data": {
    "code": 2002,
    "message": "Token expired"
  }
}
```

### 心跳机制

- 客户端应每 **30 秒** 发送 `ping` 消息
- 服务器在 **60 秒** 内未收到心跳将断开连接
- 断开后客户端应自动重连（建议使用指数退避）

---

## 错误码参考

> 完整错误码列表请参考 [ERROR_CODES.md](archive/refactor/ERROR_CODES.md)

### 错误码范围

| 范围 | 类别 | 描述 |
|------|------|------|
| 1000-1999 | 通用错误 | 系统级别的通用错误 |
| 2000-2999 | 认证错误 | 用户认证和授权相关错误 |
| 3000-3999 | 业务错误 | 业务逻辑相关错误 |
| 4000-4999 | AI 服务错误 | AI 服务相关错误 |

### 常用错误码

| 错误码 | 名称 | HTTP | 说明 |
|--------|------|------|------|
| 1000 | INTERNAL_ERROR | 500 | 内部服务器错误 |
| 1001 | VALIDATION_ERROR | 400 | 请求参数验证失败 |
| 1002 | NOT_FOUND | 404 | 请求的资源不存在 |
| 2000 | UNAUTHORIZED | 401 | 未认证 |
| 2002 | TOKEN_EXPIRED | 401 | Token 已过期 |
| 2003 | INVALID_CREDENTIALS | 401 | 凭证无效 |
| 3000 | USER_NOT_FOUND | 404 | 用户不存在 |
| 3001 | GROUP_NOT_FOUND | 404 | 群组不存在 |
| 3003 | PERMISSION_DENIED | 403 | 权限不足 |
| 4000 | AI_SERVICE_UNAVAILABLE | 503 | AI 服务不可用 |
| 4004 | AI_AGENT_NOT_FOUND | 404 | AI 助手不存在 |
| 4007 | ALL_PROVIDERS_FAILED | 503 | 所有服务商均失败 |
| 4010 | QUOTA_EXCEEDED | 429 | 配额已用尽 |

---

## 速率限制

| 端点类型 | 限制 | 说明 |
|---------|------|------|
| 认证 API | 10 req/min | 防止暴力破解 |
| 普通 API | 100 req/min | 一般接口限制 |
| 消息发送 | 60 msg/min | 防止刷屏 |
| 媒体上传 | 10 uploads/min | 防止滥用存储 |
| AI Agent | 30 req/min | 控制 AI 调用成本 |

超过限制返回 `429 Too Many Requests`：

```json
{
  "success": false,
  "error": {
    "code": 4001,
    "message": "Rate limit exceeded",
    "details": {
      "retry_after": 60
    }
  }
}
```

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

# 获取用户信息（使用返回的 token）
curl http://localhost:8080/api/users/me \
  -H "Authorization: Bearer eyJ..."

# 发送消息
curl -X POST http://localhost:8080/api/messages/direct \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer eyJ..." \
  -d '{"receiver_id":"uuid","content":"Hello!","message_type":"text"}'
```

### JavaScript (Fetch)

```javascript
// 登录
const loginResponse = await fetch('http://localhost:8080/api/auth/login', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    username: 'john',
    password: 'Pass123!'
  })
});

const loginData = await loginResponse.json();
if (!loginData.success) {
  console.error('Login failed:', loginData.error.message);
  return;
}

const token = loginData.data.access_token;

// 发送消息
const messageResponse = await fetch('http://localhost:8080/api/messages/direct', {
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

const messageData = await messageResponse.json();
if (messageData.success) {
  console.log('Message sent:', messageData.data.id);
}

// WebSocket 连接
const ws = new WebSocket(`ws://localhost:8080/ws?token=${token}`);

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  switch (message.type) {
    case 'new_message':
      console.log('New message:', message.data);
      break;
    case 'pong':
      console.log('Heartbeat OK');
      break;
  }
};

// 心跳
setInterval(() => {
  ws.send(JSON.stringify({ type: 'ping' }));
}, 30000);
```

### Python (requests)

```python
import requests
import json

BASE_URL = 'http://localhost:8080'

# 登录
response = requests.post(f'{BASE_URL}/api/auth/login', json={
    'username': 'john',
    'password': 'Pass123!'
})

data = response.json()
if not data['success']:
    print(f"Login failed: {data['error']['message']}")
    exit(1)

token = data['data']['access_token']
headers = {'Authorization': f'Bearer {token}'}

# 获取用户信息
response = requests.get(f'{BASE_URL}/api/users/me', headers=headers)
user = response.json()['data']
print(f"Logged in as: {user['username']}")

# 发送消息
response = requests.post(
    f'{BASE_URL}/api/messages/direct',
    headers=headers,
    json={
        'receiver_id': 'uuid',
        'content': 'Hello from Python!',
        'message_type': 'text'
    }
)

result = response.json()
if result['success']:
    print(f"Message sent: {result['data']['id']}")
else:
    print(f"Failed: {result['error']['message']}")
```

### Dart (Flutter)

```dart
import 'package:dio/dio.dart';

final dio = Dio(BaseOptions(baseUrl: 'http://localhost:8080'));

// 登录
Future<String?> login(String username, String password) async {
  try {
    final response = await dio.post('/api/auth/login', data: {
      'username': username,
      'password': password,
    });
    
    if (response.data['success']) {
      return response.data['data']['access_token'];
    }
    return null;
  } on DioException catch (e) {
    final error = e.response?.data['error'];
    print('Login failed: ${error?['message']}');
    return null;
  }
}

// 发送消息
Future<void> sendMessage(String token, String receiverId, String content) async {
  dio.options.headers['Authorization'] = 'Bearer $token';
  
  final response = await dio.post('/api/messages/direct', data: {
    'receiver_id': receiverId,
    'content': content,
    'message_type': 'text',
  });
  
  if (response.data['success']) {
    print('Message sent: ${response.data['data']['id']}');
  }
}
```

---

## 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| 1.0 | 2025-12-04 | 初始版本 |
| 1.1 | 2025-12-08 | 更新统一响应格式、错误码、AI Agent API |
| 1.2 | 2025-12-11 | 添加 Phase 4 新功能 API：人设系统、用户画像、消息总结、学习反馈 |

**最后更新**: 2025-12-11
