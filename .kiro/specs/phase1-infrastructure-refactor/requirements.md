# Requirements Document

## Introduction

本规范定义了 AI-Native-IM 项目 Phase 1 基础设施重构的需求。重构目标是解决当前代码库中最紧急的问题：后端错误处理不统一、前端数据库用户隔离缺失、API 响应格式混乱。这些问题直接影响系统的健壮性和可维护性。

## Glossary

- **AppError**: 统一的应用错误类型，包含错误码、错误消息和 HTTP 状态码映射
- **AppResult**: `Result<T, AppError>` 的类型别名
- **ErrorResponse**: 统一的 JSON 错误响应格式
- **User Database**: 按用户 ID 隔离的本地 SQLite 数据库文件
- **Global Database**: 存储全局配置和账号列表的共享数据库

## Requirements

### Requirement 1: 统一后端错误处理

**User Story:** As a 开发者, I want 后端有统一的错误处理机制, so that 前端能获得一致且有意义的错误信息，便于调试和用户提示。

#### Acceptance Criteria

1. WHEN 后端发生任何错误 THEN the System SHALL 返回统一格式的 JSON 错误响应，包含 error_code、message 和 details 字段
2. WHEN 后端返回错误 THEN the System SHALL 根据错误类型映射到正确的 HTTP 状态码（400/401/403/404/500）
3. WHEN 数据库操作失败 THEN the System SHALL 将 sqlx::Error 转换为 AppError::Database 并返回 500 状态码
4. WHEN 认证失败 THEN the System SHALL 返回 AppError::Unauthorized 和 401 状态码
5. WHEN 请求参数无效 THEN the System SHALL 返回 AppError::Validation 和 400 状态码，包含具体的验证失败原因
6. WHEN AI 服务调用失败 THEN the System SHALL 返回 AppError::AIService 和 503 状态码

### Requirement 2: 统一 API 响应格式

**User Story:** As a 前端开发者, I want 所有 API 返回一致的响应格式, so that 我可以用统一的方式处理成功和失败响应。

#### Acceptance Criteria

1. WHEN API 调用成功 THEN the System SHALL 返回 `{ "success": true, "data": {...} }` 格式
2. WHEN API 调用失败 THEN the System SHALL 返回 `{ "success": false, "error": { "code": "...", "message": "...", "details": {...} } }` 格式
3. WHEN 返回列表数据 THEN the System SHALL 在 data 字段中包含 items 数组和 total 计数
4. WHEN 返回分页数据 THEN the System SHALL 在 data 字段中包含 page、page_size、total_pages 信息

### Requirement 3: 前端数据库用户隔离

**User Story:** As a 用户, I want 我的聊天数据与其他用户完全隔离, so that 切换账号时不会看到其他人的消息。

#### Acceptance Criteria

1. WHEN 用户登录成功 THEN the System SHALL 创建或打开该用户专属的数据库文件 `user_{user_id}.db`
2. WHEN 用户登出 THEN the System SHALL 关闭当前用户数据库连接
3. WHEN 切换用户 THEN the System SHALL 关闭旧用户数据库并打开新用户数据库
4. WHEN 应用启动未登录 THEN the System SHALL 只访问全局数据库 `global.db`
5. WHEN 全局数据库初始化 THEN the System SHALL 创建账号列表表存储登录过的用户信息

### Requirement 4: 前端错误处理优化

**User Story:** As a 用户, I want 应用在出错时显示友好的提示, so that 我不会看到吓人的红色错误页面。

#### Acceptance Criteria

1. WHEN 发生未捕获的异常 THEN the System SHALL 显示自定义错误页面而非 Flutter 默认红屏
2. WHEN 网络请求失败 THEN the System SHALL 显示 Toast 提示并提供重试选项
3. WHEN WebSocket 断开连接 THEN the System SHALL 显示连接状态指示器并自动重连
4. WHEN API 返回错误 THEN the System SHALL 解析统一错误格式并显示对应的用户友好消息

### Requirement 5: 错误码规范

**User Story:** As a 开发者, I want 有明确的错误码规范, so that 前后端可以基于错误码进行精确的错误处理。

#### Acceptance Criteria

1. THE System SHALL 定义错误码范围：1000-1999 通用错误，2000-2999 认证错误，3000-3999 业务错误，4000-4999 AI 服务错误
2. WHEN 定义新错误码 THEN the System SHALL 在错误码文档中记录错误码、描述和建议的用户提示
3. THE System SHALL 在 AppError 枚举中为每个错误变体分配唯一的错误码
