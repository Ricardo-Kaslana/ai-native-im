# Error Codes Reference

本文档列出了 AI-Native-IM 项目中所有的错误码定义、描述和用户提示。

## 错误码范围

| 范围 | 类别 | 描述 |
|------|------|------|
| 1000-1999 | 通用错误 | 系统级别的通用错误 |
| 2000-2999 | 认证错误 | 用户认证和授权相关错误 |
| 3000-3999 | 业务错误 | 业务逻辑相关错误 |
| 4000-4999 | AI 服务错误 | AI 服务相关错误 |

## 通用错误 (1000-1999)

| 错误码 | 名称 | HTTP 状态码 | 描述 | 用户提示 |
|--------|------|-------------|------|----------|
| 1000 | INTERNAL_ERROR | 500 | 内部服务器错误 | 系统繁忙，请稍后重试 |
| 1001 | VALIDATION_ERROR | 400 | 请求参数验证失败 | (使用具体验证消息) |
| 1002 | NOT_FOUND | 404 | 请求的资源不存在 | 请求的资源不存在 |
| 1003 | CONFLICT | 409 | 资源冲突 | 操作冲突，请刷新后重试 |
| 1004 | BAD_REQUEST | 400 | 错误的请求 | 请求参数错误 |
| 1005 | TIMEOUT | 408 | 请求超时 | 请求超时，请检查网络连接 |

## 认证错误 (2000-2999)

| 错误码 | 名称 | HTTP 状态码 | 描述 | 用户提示 |
|--------|------|-------------|------|----------|
| 2000 | UNAUTHORIZED | 401 | 未认证 | 请先登录 |
| 2001 | FORBIDDEN | 403 | 禁止访问 | 没有权限执行此操作 |
| 2002 | TOKEN_EXPIRED | 401 | Token 已过期 | 登录已过期，请重新登录 |
| 2003 | INVALID_CREDENTIALS | 401 | 凭证无效 | 用户名或密码错误 |
| 2004 | ACCOUNT_LOCKED | 401 | 账号已锁定 | 账号已被锁定，请稍后再试 |
| 2005 | ACCOUNT_SUSPENDED | 401 | 账号已停用 | 账号已被停用 |

## 业务错误 (3000-3999)

| 错误码 | 名称 | HTTP 状态码 | 描述 | 用户提示 |
|--------|------|-------------|------|----------|
| 3000 | USER_NOT_FOUND | 404 | 用户不存在 | 用户不存在 |
| 3001 | GROUP_NOT_FOUND | 404 | 群组不存在 | 群组不存在 |
| 3002 | MESSAGE_NOT_FOUND | 404 | 消息不存在 | 消息不存在 |
| 3003 | PERMISSION_DENIED | 403 | 权限不足 | 没有权限执行此操作 |
| 3004 | ALREADY_EXISTS | 409 | 资源已存在 | 资源已存在 |
| 3005 | GROUP_FULL | 400 | 群组已满 | 群组已满 |
| 3006 | INVITE_EXPIRED | 400 | 邀请已过期 | 邀请码已过期或无效 |
| 3007 | MEDIA_NOT_FOUND | 404 | 媒体文件不存在 | 文件不存在 |
| 3008 | MEDIA_TOO_LARGE | 400 | 媒体文件过大 | 文件太大 |
| 3009 | UNSUPPORTED_MEDIA_TYPE | 400 | 不支持的媒体类型 | 不支持的文件类型 |

## AI 服务错误 (4000-4999)

| 错误码 | 名称 | HTTP 状态码 | 描述 | 用户提示 |
|--------|------|-------------|------|----------|
| 4000 | AI_SERVICE_UNAVAILABLE | 503 | AI 服务不可用 | AI 服务暂时不可用 |
| 4001 | AI_RATE_LIMITED | 429 | AI 服务限流 | 请求太频繁，请稍后再试 |
| 4002 | AI_TIMEOUT | 504 | AI 服务超时 | AI 响应超时，请重试 |
| 4003 | AI_INVALID_RESPONSE | 502 | AI 响应无效 | AI 服务异常 |
| 4004 | AI_AGENT_NOT_FOUND | 404 | AI 助手不存在 | AI 助手不存在 |
| 4005 | EMBEDDING_ERROR | 500 | 向量嵌入错误 | AI 服务异常 |
| 4006 | VECTOR_DB_ERROR | 500 | 向量数据库错误 | AI 服务异常 |
| 4007 | ALL_PROVIDERS_FAILED | 503 | 所有服务商均失败 | AI 服务暂时不可用，请稍后再试 |
| 4008 | STT_ERROR | 500 | 语音识别失败 | 语音识别失败，请重新发送或使用文字 |
| 4009 | TTS_ERROR | 500 | 语音合成失败 | 语音合成失败，已为您显示文字回复 |
| 4010 | QUOTA_EXCEEDED | 429 | 配额已用尽 | 您今日的 AI 对话次数已用完，明天再来吧 |
| 4011 | TOOL_NOT_FOUND | 404 | 工具不存在 | AI 工具不可用 |
| 4012 | TOOL_EXECUTION_ERROR | 500 | 工具执行失败 | AI 工具执行失败，请重试 |
| 4013 | MEMORY_ERROR | 500 | 记忆系统错误 | AI 服务异常 |
| 4014 | SUMMARY_ERROR | 500 | 消息总结失败 | 消息总结失败，请稍后再试 |
| 4015 | EMOTION_ANALYSIS_ERROR | 500 | 情绪分析失败 | AI 服务异常 |
| 4016 | LEARNING_ERROR | 500 | 学习系统错误 | AI 服务异常 |
| 4017 | INVALID_PERSONA | 400 | 人设配置无效 | AI 助手配置无效 |
| 4018 | STREAMING_INTERRUPTED | 500 | 流式响应中断 | AI 响应中断，请重试 |

## 错误响应格式

### 统一错误响应

```json
{
  "success": false,
  "error": {
    "code": 2003,
    "message": "Invalid credentials",
    "details": {
      "field": "password"
    }
  }
}
```

### 字段说明

| 字段 | 类型 | 必填 | 描述 |
|------|------|------|------|
| success | boolean | 是 | 固定为 `false` |
| error.code | integer | 是 | 错误码 |
| error.message | string | 是 | 错误消息（英文，供开发调试） |
| error.details | object | 否 | 额外的错误详情 |

## 错误处理最佳实践

### 后端

1. 使用 `AppError` 枚举定义所有错误类型
2. 通过 `IntoResponse` trait 自动转换为统一响应格式
3. 使用 `From` trait 进行错误类型转换

```rust
// 示例：返回错误
return Err(AppError::NotFound("User not found".to_string()));

// 示例：错误转换
let user = db.get_user(id).await?; // sqlx::Error 自动转换为 AppError
```

### 前端

1. 使用 `ApiError` 类解析错误响应
2. 使用 `userMessage` 获取用户友好的错误提示
3. 检查 `isRetryable` 决定是否显示重试按钮
4. 检查 `requiresReauth` 决定是否需要重新登录

```dart
// 示例：处理 API 错误
try {
  await apiClient.post('/api/login', data: credentials);
} on DioException catch (e) {
  final apiError = e.apiError;
  if (apiError != null) {
    showErrorToast(context, apiError.userMessage);
    if (apiError.requiresReauth) {
      // 跳转到登录页
    }
  }
}
```

## 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| 1.0 | 2025-12-07 | 初始版本 |
| 1.1 | 2025-12-08 | 添加 Phase 3 AI Agent Evolution 相关错误码 (4007-4018) |
| 1.2 | 2025-12-11 | Phase 4 完成，错误码无需新增（复用现有错误类型） |
