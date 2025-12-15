# AI Native IM - 后端 API 文档

## 文档概述

本文档详细描述了 AI Native IM 后端 API 的所有端点、数据结构、错误码和 WebSocket 通信协议，为前端开发提供完整的接口参考。

### 基础信息

- **Base URL**: `http://localhost:8080/api`
- **API 版本**: v1
- **认证方式**: JWT Bearer Token
- **数据格式**: JSON
- **字符编码**: UTF-8

---

## 1. 认证系统

### 1.1 用户注册

**POST** `/api/auth/register`

注册新用户账户。

#### 请求体

```json
{
  "username": "string (3-50字符)",
  "email": "string (有效邮箱格式)",
  "password": "string (至少8字符，包含大小写字母和数字)"
}
```

#### 响应

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
      "online_status": "online|offline|away",
      "created_at": "ISO 8601 datetime"
    },
    "access_token": "jwt_token",
    "refresh_token": "jwt_token"
  }
}
```

#### 错误码

- `1001`: 验证失败（用户名格式、邮箱格式、密码强度等）
- `1003`: 冲突（用户名或邮箱已存在）
- `2003`: 无效凭据

---

### 1.2 用户登录

**POST** `/api/auth/login`

使用用户名/邮箱和密码登录。

#### 请求体

```json
{
  "username": "string (用户名或邮箱)",
  "password": "string"
}
```

#### 响应

同注册响应格式。

#### 错误码

- `2003`: 无效凭据
- `2004`: 账户被锁定
- `2005`: 账户被暂停

---

### 1.3 刷新令牌

**POST** `/api/auth/refresh`

使用刷新令牌获取新的访问令牌。

#### 请求体

```json
{
  "refresh_token": "jwt_token"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "access_token": "new_jwt_token",
    "refresh_token": "new_jwt_token"
  }
}
```

#### 错误码

- `2002`: 令牌已过期
- `2000`: 无效令牌

---

### 1.4 用户登出

**POST** `/api/auth/logout`

登出当前用户并使令牌失效。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "message": "Logged out successfully"
  }
}
```

---

### 1.5 修改密码

**POST** `/api/auth/password/change`

修改当前用户密码。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 请求体

```json
{
  "current_password": "string",
  "new_password": "string"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "message": "Password changed successfully"
  }
}
```

---

### 1.6 重置密码请求

**POST** `/api/auth/password/reset/request`

请求发送密码重置邮件。

#### 请求体

```json
{
  "email": "string"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "message": "Password reset email sent"
  }
}
```

---

### 1.7 确认重置密码

**POST** `/api/auth/password/reset/confirm`

使用重置令牌设置新密码。

#### 请求体

```json
{
  "token": "string",
  "new_password": "string"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "message": "Password reset successfully"
  }
}
```

---

## 2. 用户管理

### 2.1 获取当前用户信息

**GET** `/api/users/me`

获取当前登录用户的详细信息。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "username": "string",
    "email": "string",
    "avatar_url": "string|null",
    "bio": "string|null",
    "online_status": "online|offline|away",
    "last_seen_at": "ISO 8601 datetime|null",
    "role": "user|moderator|admin",
    "created_at": "ISO 8601 datetime"
  }
}
```

---

### 2.2 更新当前用户信息

**PUT** `/api/users/me`

更新当前用户的个人资料。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 请求体

```json
{
  "username": "string|null",
  "avatar_url": "string|null",
  "bio": "string|null"
}
```

#### 响应

同获取用户信息响应。

---

### 2.3 获取用户信息

**GET** `/api/users/:id`

获取指定用户的公开信息。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "username": "string",
    "avatar_url": "string|null",
    "bio": "string|null",
    "online_status": "online|offline|away",
    "last_seen_at": "ISO 8601 datetime|null",
    "role": "user|moderator|admin",
    "created_at": "ISO 8601 datetime"
  }
}
```

#### 错误码

- `3000`: 用户不存在

---

### 2.4 搜索用户

**GET** `/api/users/search`

搜索用户。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 查询参数

- `q`: 搜索关键词（至少2个字符）
- `limit`: 返回数量限制（默认10，最大50）

#### 响应

```json
{
  "success": true,
  "data": {
    "items": [
      {
        "id": "uuid",
        "username": "string",
        "avatar_url": "string|null",
        "online_status": "online|offline|away"
      }
    ],
    "total": "number"
  }
}
```

---

### 2.5 批量获取用户

**POST** `/api/users/batch`

批量获取多个用户的信息。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 请求体

```json
{
  "user_ids": ["uuid1", "uuid2", "..."]
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "users": [
      {
        "id": "uuid",
        "username": "string",
        "avatar_url": "string|null",
        "online_status": "online|offline|away"
      }
    ]
  }
}
```

---

## 3. 群组管理

### 3.1 创建群组

**POST** `/api/groups`

创建新群组。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 请求体

```json
{
  "name": "string (1-50字符)",
  "description": "string (最多500字符)",
  "avatar_url": "string|null"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "name": "string",
    "description": "string",
    "avatar_url": "string|null",
    "owner_id": "uuid",
    "member_count": "number",
    "max_capacity": "number",
    "created_at": "ISO 8601 datetime"
  }
}
```

---

### 3.2 获取群组信息

**GET** `/api/groups/:id`

获取群组详细信息。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "name": "string",
    "description": "string",
    "avatar_url": "string|null",
    "owner_id": "uuid",
    "member_count": "number",
    "max_capacity": "number",
    "created_at": "ISO 8601 datetime",
    "member_role": "owner|admin|member",
    "is_muted": "boolean",
    "invite_code": "string|null"
  }
}
```

#### 错误码

- `3001`: 群组不存在
- `3003`: 权限不足

---

### 3.3 更新群组信息

**PUT** `/api/groups/:id`

更新群组信息（仅群主和管理员）。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 请求体

```json
{
  "name": "string|null",
  "description": "string|null",
  "avatar_url": "string|null"
}
```

#### 响应

同获取群组信息响应。

---

### 3.4 获取群组成员

**GET** `/api/groups/:id/members`

获取群组成员列表。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 查询参数

- `page`: 页码（默认1）
- `page_size`: 每页数量（默认20，最大100）

#### 响应

```json
{
  "success": true,
  "data": {
    "items": [
      {
        "user": {
          "id": "uuid",
          "username": "string",
          "avatar_url": "string|null",
          "online_status": "online|offline|away"
        },
        "role": "owner|admin|member",
        "joined_at": "ISO 8601 datetime",
        "is_muted": "boolean"
      }
    ],
    "total": "number",
    "page": "number",
    "page_size": "number",
    "total_pages": "number"
  }
}
```

---

### 3.5 邀请用户加入群组

**POST** `/api/groups/:id/invite`

邀请用户加入群组（仅群主和管理员）。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 请求体

```json
{
  "user_id": "uuid"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "message": "User invited successfully"
  }
}
```

---

### 3.6 通过邀请码加入群组

**POST** `/api/groups/join`

使用邀请码加入群组。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 请求体

```json
{
  "invite_code": "string"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "message": "Joined group successfully",
    "group": {
      "id": "uuid",
      "name": "string",
      "avatar_url": "string|null"
    }
  }
}
```

---

### 3.7 生成邀请码

**POST** `/api/groups/:id/invite-code`

生成新的邀请码（仅群主和管理员）。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 请求体

```json
{
  "max_uses": "number|null",
  "expires_at": "ISO 8601 datetime|null"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "invite_code": "string",
    "max_uses": "number|null",
    "expires_at": "ISO 8601 datetime|null"
  }
}
```

---

### 3.8 离开群组

**POST** `/api/groups/:id/leave`

离开群组。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "message": "Left group successfully"
  }
}
```

---

### 3.9 移除成员

**DELETE** `/api/groups/:id/members/:user_id`

移除群组成员（仅群主和管理员）。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "message": "Member removed successfully"
  }
}
```

---

### 3.10 设置成员角色

**PUT** `/api/groups/:id/members/:user_id/role`

设置成员角色（仅群主）。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 请求体

```json
{
  "role": "admin|member"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "message": "Role updated successfully"
  }
}
```

---

### 3.11 禁言/取消禁言

**PUT** `/api/groups/:id/members/:user_id/mute`

禁言或取消禁言成员（仅群主和管理员）。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 请求体

```json
{
  "is_muted": "boolean"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "message": "Mute status updated successfully"
  }
}
```

---

## 4. AI Agent 系统

### 4.1 创建 AI Agent

**POST** `/api/ai/groups/:group_id/agent`

为群组创建 AI Agent。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 请求体

```json
{
  "name": "string (1-100字符)",
  "toxicity_level": "number (0-10)",
  "enthusiasm_level": "number (0-10)",
  "expertise_domains": ["string"],
  "speech_suffix": "string|null",
  "trigger_keywords": ["string"]
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "group_id": "uuid",
    "name": "string",
    "toxicity_level": "number",
    "enthusiasm_level": "number",
    "expertise_domains": ["string"],
    "speech_suffix": "string|null",
    "trigger_keywords": ["string"],
    "is_active": "boolean",
    "created_at": "ISO 8601 datetime"
  }
}
```

#### 错误码

- `4004`: AI Agent 不存在
- `1004`: 无效的人设参数

---

### 4.2 获取群组 AI Agent

**GET** `/api/ai/groups/:group_id/agent`

获取群组的 AI Agent 信息。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "group_id": "uuid",
    "name": "string",
    "toxicity_level": "number",
    "enthusiasm_level": "number",
    "expertise_domains": ["string"],
    "speech_suffix": "string|null",
    "trigger_keywords": ["string"],
    "is_active": "boolean",
    "created_at": "ISO 8601 datetime"
  }
}
```

---

### 4.3 获取 AI Agent 信息

**GET** `/api/ai/agents/:agent_id`

获取指定 AI Agent 的详细信息。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 响应

同获取群组 AI Agent 响应。

---

### 4.4 更新 AI Agent 人设

**PUT** `/api/ai/agents/:agent_id/personality`

更新 AI Agent 的人设参数。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 请求体

```json
{
  "toxicity_level": "number|null",
  "enthusiasm_level": "number|null",
  "expertise_domains": ["string"]|null,
  "speech_suffix": "string|null",
  "trigger_keywords": ["string"]|null
}
```

#### 响应

同获取 AI Agent 信息响应。

---

### 4.5 获取 AI Agent 统计

**GET** `/api/ai/agents/:agent_id/stats`

获取 AI Agent 的使用统计。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "total_messages": "number",
    "avg_response_time_ms": "number",
    "last_message_at": "ISO 8601 datetime|null",
    "trigger_stats": {
      "mention": {
        "count": "number",
        "response_rate": "number"
      },
      "keyword": {
        "count": "number",
        "response_rate": "number"
      },
      "context": {
        "count": "number",
        "response_rate": "number"
      },
      "random": {
        "count": "number",
        "response_rate": "number"
      },
      "cold_start": {
        "count": "number",
        "response_rate": "number"
      }
    }
  }
}
```

---

### 4.6 设置用户亲和度

**POST** `/api/ai/agents/:agent_id/affinity`

设置用户与 AI Agent 的亲和度。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 请求体

```json
{
  "user_id": "uuid",
  "affinity_score": "number (-10 到 10)"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "message": "Affinity set successfully"
  }
}
```

---

### 4.7 生成消息总结

**POST** `/api/ai/groups/:group_id/summary`

生成群组消息的总结。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 请求体

```json
{
  "message_count": "number (默认100)",
  "style": "brief|detailed|topics|action_items",
  "use_cache": "boolean (默认true)"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "summary": "string",
    "style": "string",
    "message_count": "number",
    "time_range": {
      "start": "ISO 8601 datetime",
      "end": "ISO 8601 datetime"
    },
    "cached": "boolean",
    "generated_at": "ISO 8601 datetime"
  }
}
```

---

### 4.8 记录反馈

**POST** `/api/ai/messages/:message_id/feedback`

记录对 AI 消息的反馈。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 请求体

```json
{
  "response_id": "uuid",
  "feedback_type": "positive|negative|correction|ignored",
  "correction": "string|null"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "recorded": "boolean",
    "message": "反馈已记录，感谢您的帮助！"
  }
}
```

---

### 4.9 获取反馈统计

**GET** `/api/ai/groups/:group_id/feedback/stats`

获取群组的反馈统计。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "total_feedback": "number",
    "positive_count": "number",
    "negative_count": "number",
    "correction_count": "number",
    "ignored_count": "number",
    "recent_feedback": [
      {
        "message_id": "uuid",
        "feedback_type": "string",
        "created_at": "ISO 8601 datetime"
      }
    ]
  }
}
```

---

## 5. 媒体处理

### 5.1 获取上传令牌

**POST** `/api/media/upload-token`

获取媒体文件上传令牌。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 请求体

```json
{
  "file_name": "string",
  "file_size": "number",
  "mime_type": "string",
  "group_id": "uuid|null"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "upload_token": "string",
    "upload_url": "string",
    "expires_at": "ISO 8601 datetime"
  }
}
```

#### 错误码

- `3008`: 媒体文件过大
- `3009`: 不支持的媒体类型

---

### 5.2 确认上传完成

**POST** `/api/media/confirm-upload`

确认媒体文件上传完成。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 请求体

```json
{
  "upload_token": "string",
  "file_hash": "string"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "media_id": "uuid",
    "url": "string",
    "thumbnail_url": "string|null",
    "metadata": {
      "width": "number|null",
      "height": "number|null",
      "duration": "number|null"
    }
  }
}
```

---

### 5.3 获取媒体信息

**GET** `/api/media/:id`

获取媒体文件信息。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "file_name": "string",
    "mime_type": "string",
    "file_size": "number",
    "url": "string",
    "thumbnail_url": "string|null",
    "metadata": {
      "width": "number|null",
      "height": "number|null",
      "duration": "number|null"
    },
    "created_at": "ISO 8601 datetime"
  }
}
```

---

## 6. 管理员功能

### 6.1 获取系统统计

**GET** `/api/admin/stats`

获取系统统计信息（仅管理员）。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "users": {
      "total": "number",
      "active": "number",
      "online": "number"
    },
    "groups": {
      "total": "number",
      "active": "number"
    },
    "messages": {
      "total": "number",
      "today": "number"
    },
    "ai_agents": {
      "total": "number",
      "active": "number"
    }
  }
}
```

---

### 6.2 获取用户列表

**GET** `/api/admin/users`

获取用户列表（仅管理员）。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 查询参数

- `page`: 页码（默认1）
- `page_size`: 每页数量（默认20）
- `status`: 账户状态筛选（active|suspended|deleted）
- `role`: 角色筛选（user|moderator|admin）

#### 响应

```json
{
  "success": true,
  "data": {
    "items": [
      {
        "id": "uuid",
        "username": "string",
        "email": "string",
        "role": "string",
        "account_status": "string",
        "online_status": "string",
        "last_seen_at": "ISO 8601 datetime|null",
        "created_at": "ISO 8601 datetime"
      }
    ],
    "total": "number",
    "page": "number",
    "page_size": "number",
    "total_pages": "number"
  }
}
```

---

### 6.3 暂停用户

**POST** `/api/admin/users/:id/suspend`

暂停用户账户（仅管理员）。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 请求体

```json
{
  "reason": "string"
}
```

#### 响应

```json
{
  "success": true,
  "data": {
    "message": "User suspended successfully"
  }
}
```

---

### 6.4 激活用户

**POST** `/api/admin/users/:id/activate`

激活用户账户（仅管理员）。

#### 请求头

```
Authorization: Bearer <access_token>
```

#### 响应

```json
{
  "success": true,
  "data": {
    "message": "User activated successfully"
  }
}
```

---

## 7. WebSocket 通信

### 7.1 连接建立

**WebSocket** `/api/ws`

建立 WebSocket 连接进行实时通信。

#### 连接参数

- `token`: JWT 访问令牌（查询参数）

#### 连接步骤

1. 客户端发起 WebSocket 连接：`ws://localhost:8080/api/ws?token=<jwt_token>`
2. 服务器验证令牌
3. 验证成功后建立连接

---

### 7.2 消息格式

#### 客户端发送消息

```json
{
  "type": "message|heartbeat|sync",
  "data": {
    // 根据 type 不同而变化
  }
}
```

#### 服务器发送消息

```json
{
  "type": "message|typing|presence|error",
  "data": {
    // 根据 type 不同而变化
  }
}
```

---

### 7.3 消息类型

#### 7.3.1 发送消息

**客户端 → 服务器**

```json
{
  "type": "message",
  "data": {
    "id": "uuid",
    "sender_id": "uuid",
    "receiver_id": "uuid|null",
    "group_id": "uuid|null",
    "msg_type": "0|1|2|3",
    "content": "string",
    "timestamp": "unix_timestamp"
  }
}
```

**服务器 → 客户端**

```json
{
  "type": "message",
  "data": {
    "id": "uuid",
    "sender_id": "uuid",
    "receiver_id": "uuid|null",
    "group_id": "uuid|null",
    "msg_type": "0|1|2|3",
    "content": "string",
    "timestamp": "unix_timestamp",
    "is_ai_generated": "boolean"
  }
}
```

#### 7.3.2 正在输入指示

**客户端 → 服务器**

```json
{
  "type": "typing",
  "data": {
    "group_id": "uuid",
    "is_typing": "boolean"
  }
}
```

**服务器 → 客户端**

```json
{
  "type": "typing",
  "data": {
    "user_id": "uuid",
    "group_id": "uuid",
    "is_typing": "boolean"
  }
}
```

#### 7.3.3 在线状态

**服务器 → 客户端**

```json
{
  "type": "presence",
  "data": {
    "user_id": "uuid",
    "status": "online|offline|away",
    "last_seen_at": "ISO 8601 datetime|null"
  }
}
```

#### 7.3.4 心跳

**客户端 → 服务器**

```json
{
  "type": "heartbeat",
  "data": {
    "timestamp": "unix_timestamp"
  }
}
```

**服务器 → 客户端**

```json
{
  "type": "heartbeat",
  "data": {
    "timestamp": "unix_timestamp"
  }
}
```

---

### 7.4 错误处理

#### 连接错误

```json
{
  "type": "error",
  "data": {
    "code": "number",
    "message": "string"
  }
}
```

#### 常见错误码

- `4001`: WebSocket 连接参数错误
- `4002`: 认证失败
- `4003`: 连接被拒绝
- `4004`: 消息格式错误

---

## 8. 错误码参考

### 8.1 通用错误 (1000-1999)

| 错误码 | 名称 | HTTP状态码 | 描述 |
|--------|------|------------|------|
| 1000 | INTERNAL_ERROR | 500 | 内部服务器错误 |
| 1001 | VALIDATION_ERROR | 400 | 验证失败 |
| 1002 | NOT_FOUND | 404 | 资源不存在 |
| 1003 | CONFLICT | 409 | 资源冲突 |
| 1004 | BAD_REQUEST | 400 | 请求参数错误 |
| 1005 | TIMEOUT | 408 | 请求超时 |

### 8.2 认证错误 (2000-2999)

| 错误码 | 名称 | HTTP状态码 | 描述 |
|--------|------|------------|------|
| 2000 | UNAUTHORIZED | 401 | 未授权访问 |
| 2001 | FORBIDDEN | 403 | 权限不足 |
| 2002 | TOKEN_EXPIRED | 401 | 令牌已过期 |
| 2003 | INVALID_CREDENTIALS | 401 | 无效凭据 |
| 2004 | ACCOUNT_LOCKED | 401 | 账户被锁定 |
| 2005 | ACCOUNT_SUSPENDED | 401 | 账户被暂停 |

### 8.3 业务逻辑错误 (3000-3999)

| 错误码 | 名称 | HTTP状态码 | 描述 |
|--------|------|------------|------|
| 3000 | USER_NOT_FOUND | 404 | 用户不存在 |
| 3001 | GROUP_NOT_FOUND | 404 | 群组不存在 |
| 3002 | MESSAGE_NOT_FOUND | 404 | 消息不存在 |
| 3003 | PERMISSION_DENIED | 403 | 权限不足 |
| 3004 | ALREADY_EXISTS | 409 | 资源已存在 |
| 3005 | GROUP_FULL | 400 | 群组已满 |
| 3006 | INVITE_EXPIRED | 400 | 邀请码过期 |
| 3007 | MEDIA_NOT_FOUND | 404 | 媒体文件不存在 |
| 3008 | MEDIA_TOO_LARGE | 400 | 媒体文件过大 |
| 3009 | UNSUPPORTED_MEDIA_TYPE | 400 | 不支持的媒体类型 |

### 8.4 AI 服务错误 (4000-4999)

| 错误码 | 名称 | HTTP状态码 | 描述 |
|--------|------|------------|------|
| 4000 | AI_SERVICE_UNAVAILABLE | 503 | AI 服务不可用 |
| 4001 | AI_RATE_LIMITED | 503 | AI 服务频率限制 |
| 4002 | AI_TIMEOUT | 503 | AI 服务超时 |
| 4003 | AI_INVALID_RESPONSE | 503 | AI 响应无效 |
| 4004 | AI_AGENT_NOT_FOUND | 404 | AI Agent 不存在 |
| 4005 | EMBEDDING_ERROR | 503 | 嵌入生成失败 |
| 4006 | VECTOR_DB_ERROR | 503 | 向量数据库错误 |

---

## 9. 限制和配额

### 9.1 请求限制

- 认证接口：每分钟最多 10 次请求
- 消息发送：每秒最多 5 条消息
- 媒体上传：每分钟最多 10 个文件
- 搜索接口：每分钟最多 30 次请求

### 9.2 文件限制

- 图片文件：最大 10MB
- 视频文件：最大 100MB
- 支持的图片格式：JPEG, PNG, WebP, GIF
- 支持的视频格式：MP4, WebM, MOV

### 9.3 群组限制

- 群组名称：1-50 字符
- 群组描述：最多 500 字符
- 最大成员数：500 人
- 邀请码有效期：最多 30 天

---

## 10. 开发指南

### 10.1 环境配置

开发环境配置请参考 [DEV_SETUP.md](DEV_SETUP.md)。

### 10.2 本地测试

```bash
# 启动后端服务
cd server
cargo run

# 运行测试
cargo test

# 查看日志
RUST_LOG=debug cargo run
```

### 10.3 API 测试

可以使用以下工具测试 API：

- Postman
- curl
- HTTPie

示例：

```bash
# 用户注册
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "password123"
  }'

# 获取用户信息
curl -X GET http://localhost:8080/api/users/me \
  -H "Authorization: Bearer <access_token>"
```

---

## 11. 常见问题

### Q: 如何处理令牌过期？

A: 当收到 401 错误且错误码为 2002 时，使用刷新令牌调用 `/api/auth/refresh` 获取新的访问令牌。

### Q: WebSocket 连接断开如何处理？

A: 实现自动重连机制，使用指数退避算法避免频繁重连。

### Q: 如何处理大文件上传？

A: 使用分片上传，先将文件分成小块，然后逐块上传。

### Q: AI Agent 不响应怎么办？

A: 检查 AI Agent 是否激活，查看触发关键词配置，确认消息内容符合触发条件。

---

## 12. 更新日志

### v1.0.0 (2025-12-09)

- 初始版本发布
- 完整的用户认证系统
- 消息和群组管理
- AI Agent 系统集成
- 媒体处理功能
- WebSocket 实时通信

---

## 13. 联系方式

如有问题或建议，请联系开发团队或查看项目文档。