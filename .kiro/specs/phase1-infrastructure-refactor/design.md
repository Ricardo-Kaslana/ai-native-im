# Design Document - Phase 1: Infrastructure Refactor

## Overview

本设计文档定义了 AI-Native-IM 项目 Phase 1 基础设施重构的技术方案。重构目标是建立统一的错误处理机制、API 响应格式和前端数据库用户隔离，为后续的功能开发奠定坚实基础。

## Architecture

### 后端错误处理架构

```
┌─────────────────────────────────────────────────────────────┐
│                      API Layer (Axum)                        │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                   Handler Functions                  │    │
│  │         返回 Result<ApiResponse<T>, AppError>        │    │
│  └─────────────────────────────────────────────────────┘    │
│                            │                                 │
│                            ▼                                 │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              AppError → IntoResponse                 │    │
│  │         自动转换为统一的 JSON 错误响应                │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Service Layer                             │
│         返回 Result<T, AppError>                             │
│         业务逻辑错误转换为 AppError                          │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   Repository Layer                           │
│         返回 Result<T, sqlx::Error>                          │
│         数据库错误通过 From trait 转换                       │
└─────────────────────────────────────────────────────────────┘
```

### 前端数据库隔离架构

```
┌─────────────────────────────────────────────────────────────┐
│                    Flutter App                               │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                 DatabaseService                      │    │
│  │  ┌─────────────┐    ┌─────────────────────────┐     │    │
│  │  │ global.db   │    │ user_{user_id}.db       │     │    │
│  │  │ - accounts  │    │ - users                 │     │    │
│  │  │ - settings  │    │ - conversations         │     │    │
│  │  └─────────────┘    │ - messages              │     │    │
│  │                     │ - media_attachments     │     │    │
│  │                     └─────────────────────────┘     │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### 后端组件

#### 1. AppError 枚举

```rust
// src/error.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// 错误码范围定义
pub mod error_codes {
    // 1000-1999: 通用错误
    pub const INTERNAL_ERROR: i32 = 1000;
    pub const VALIDATION_ERROR: i32 = 1001;
    pub const NOT_FOUND: i32 = 1002;
    pub const CONFLICT: i32 = 1003;
    
    // 2000-2999: 认证错误
    pub const UNAUTHORIZED: i32 = 2000;
    pub const FORBIDDEN: i32 = 2001;
    pub const TOKEN_EXPIRED: i32 = 2002;
    pub const INVALID_CREDENTIALS: i32 = 2003;
    
    // 3000-3999: 业务错误
    pub const USER_NOT_FOUND: i32 = 3000;
    pub const GROUP_NOT_FOUND: i32 = 3001;
    pub const MESSAGE_NOT_FOUND: i32 = 3002;
    pub const PERMISSION_DENIED: i32 = 3003;
    
    // 4000-4999: AI 服务错误
    pub const AI_SERVICE_UNAVAILABLE: i32 = 4000;
    pub const AI_RATE_LIMITED: i32 = 4001;
    pub const AI_TIMEOUT: i32 = 4002;
}

#[derive(Debug, Error)]
pub enum AppError {
    // 通用错误
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    // 认证错误
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    // 数据库错误
    #[error("Database error: {0}")]
    Database(String),
    
    // AI 服务错误
    #[error("AI service unavailable: {0}")]
    AIServiceUnavailable(String),
    
    #[error("AI service rate limited")]
    AIRateLimited,
    
    #[error("AI service timeout")]
    AITimeout,
}

impl AppError {
    pub fn error_code(&self) -> i32 {
        match self {
            AppError::Internal(_) => error_codes::INTERNAL_ERROR,
            AppError::Validation(_) => error_codes::VALIDATION_ERROR,
            AppError::NotFound(_) => error_codes::NOT_FOUND,
            AppError::Conflict(_) => error_codes::CONFLICT,
            AppError::Unauthorized => error_codes::UNAUTHORIZED,
            AppError::Forbidden(_) => error_codes::FORBIDDEN,
            AppError::TokenExpired => error_codes::TOKEN_EXPIRED,
            AppError::InvalidCredentials => error_codes::INVALID_CREDENTIALS,
            AppError::Database(_) => error_codes::INTERNAL_ERROR,
            AppError::AIServiceUnavailable(_) => error_codes::AI_SERVICE_UNAVAILABLE,
            AppError::AIRateLimited => error_codes::AI_RATE_LIMITED,
            AppError::AITimeout => error_codes::AI_TIMEOUT,
        }
    }
    
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Internal(_) | AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Unauthorized | AppError::TokenExpired | AppError::InvalidCredentials => {
                StatusCode::UNAUTHORIZED
            }
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::AIServiceUnavailable(_) | AppError::AIRateLimited | AppError::AITimeout => {
                StatusCode::SERVICE_UNAVAILABLE
            }
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}
```

#### 2. ApiResponse 统一响应格式

```rust
// src/response.rs

use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetail>,
}

#[derive(Serialize)]
pub struct ErrorDetail {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct ListResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    pub fn error(code: i32, message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(ErrorDetail {
                code,
                message: message.into(),
                details: None,
            }),
        }
    }
}
```

### 前端组件

#### 1. DatabaseService

```dart
// lib/core/database/database_service.dart

class DatabaseService {
  static DatabaseService? _instance;
  
  AppDatabase? _globalDb;
  AppDatabase? _userDb;
  String? _currentUserId;
  
  static DatabaseService get instance {
    _instance ??= DatabaseService._();
    return _instance!;
  }
  
  DatabaseService._();
  
  /// 获取全局数据库（账号列表、设置等）
  Future<AppDatabase> getGlobalDatabase() async {
    _globalDb ??= await _openDatabase('global.db');
    return _globalDb!;
  }
  
  /// 获取当前用户数据库
  AppDatabase? get userDatabase => _userDb;
  
  /// 用户登录后打开用户专属数据库
  Future<void> openUserDatabase(String userId) async {
    if (_currentUserId == userId && _userDb != null) {
      return; // 已经打开
    }
    
    // 关闭旧的用户数据库
    await closeUserDatabase();
    
    // 打开新的用户数据库
    _userDb = await _openDatabase('user_$userId.db');
    _currentUserId = userId;
  }
  
  /// 关闭用户数据库（登出时调用）
  Future<void> closeUserDatabase() async {
    if (_userDb != null) {
      await _userDb!.close();
      _userDb = null;
      _currentUserId = null;
    }
  }
  
  Future<AppDatabase> _openDatabase(String fileName) async {
    final dbFolder = await getApplicationDocumentsDirectory();
    final file = File(p.join(dbFolder.path, 'aiim', fileName));
    return AppDatabase(NativeDatabase.createInBackground(file));
  }
}
```

#### 2. ErrorHandler

```dart
// lib/core/error/error_handler.dart

class ErrorHandler {
  static void initialize() {
    // 捕获 Flutter 框架错误
    FlutterError.onError = (details) {
      _logError(details.exception, details.stack);
      // 不显示红屏，而是记录日志
    };
    
    // 捕获异步错误
    PlatformDispatcher.instance.onError = (error, stack) {
      _logError(error, stack);
      return true;
    };
  }
  
  static void _logError(Object error, StackTrace? stack) {
    // TODO: 发送到错误追踪服务
    debugPrint('Error: $error');
    debugPrint('Stack: $stack');
  }
}

/// 自定义错误页面
class AppErrorWidget extends StatelessWidget {
  final FlutterErrorDetails details;
  
  const AppErrorWidget({required this.details});
  
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.error_outline, size: 64, color: Colors.grey),
            SizedBox(height: 16),
            Text('系统开小差了', style: TextStyle(fontSize: 18)),
            SizedBox(height: 8),
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: Text('返回'),
            ),
          ],
        ),
      ),
    );
  }
}
```

#### 3. ApiError 解析

```dart
// lib/core/network/api_error.dart

class ApiError implements Exception {
  final int code;
  final String message;
  final Map<String, dynamic>? details;
  
  ApiError({
    required this.code,
    required this.message,
    this.details,
  });
  
  factory ApiError.fromJson(Map<String, dynamic> json) {
    final error = json['error'] as Map<String, dynamic>;
    return ApiError(
      code: error['code'] as int,
      message: error['message'] as String,
      details: error['details'] as Map<String, dynamic>?,
    );
  }
  
  /// 获取用户友好的错误消息
  String get userMessage {
    switch (code) {
      case 2000: return '请先登录';
      case 2002: return '登录已过期，请重新登录';
      case 2003: return '用户名或密码错误';
      case 3003: return '没有权限执行此操作';
      case 4000: return 'AI 服务暂时不可用';
      case 4001: return '请求太频繁，请稍后再试';
      default: return message;
    }
  }
}
```

## Data Models

### 后端数据模型

无新增数据模型，主要是错误处理和响应格式的结构定义。

### 前端数据模型

#### GlobalDatabase Schema

```dart
// 全局数据库表
class Accounts extends Table {
  TextColumn get userId => text()();
  TextColumn get username => text()();
  TextColumn get avatarUrl => text().nullable()();
  DateTimeColumn get lastLoginAt => dateTime()();
  
  @override
  Set<Column> get primaryKey => {userId};
}

class AppSettings extends Table {
  TextColumn get key => text()();
  TextColumn get value => text()();
  
  @override
  Set<Column> get primaryKey => {key};
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: 错误响应格式一致性
*For any* AppError 变体，转换为 HTTP 响应后，响应体 SHALL 包含 `success: false` 和 `error` 对象，其中 `error` 对象包含 `code` (int) 和 `message` (string) 字段。
**Validates: Requirements 1.1, 2.2**

### Property 2: 错误码到状态码映射正确性
*For any* AppError 变体，其 `status_code()` 方法返回的 HTTP 状态码 SHALL 与错误类型语义一致：认证错误返回 401/403，验证错误返回 400，未找到返回 404，服务错误返回 500/503。
**Validates: Requirements 1.2**

### Property 3: 成功响应格式一致性
*For any* 成功的 API 响应，响应体 SHALL 包含 `success: true` 和 `data` 字段。
**Validates: Requirements 2.1**

### Property 4: 列表响应格式一致性
*For any* 返回列表的 API 响应，`data` 字段 SHALL 包含 `items` 数组和 `total` 计数。
**Validates: Requirements 2.3**

### Property 5: 用户数据库隔离
*For any* 两个不同的用户 ID，调用 `openUserDatabase` 后，数据库文件路径 SHALL 不同，且文件名包含对应的用户 ID。
**Validates: Requirements 3.1, 3.3**

### Property 6: 错误码唯一性
*For any* 两个不同的 AppError 变体（除了参数化变体如 Internal(String)），其 `error_code()` 返回值 SHALL 不同。
**Validates: Requirements 5.1, 5.3**

### Property 7: 错误码范围正确性
*For any* AppError 变体，其 `error_code()` 返回值 SHALL 在定义的范围内：通用错误 1000-1999，认证错误 2000-2999，业务错误 3000-3999，AI 错误 4000-4999。
**Validates: Requirements 5.1**

### Property 8: WebSocket 重连行为
*For any* WebSocket 断开事件，系统 SHALL 在指定的重试间隔后尝试重连，直到达到最大重试次数或连接成功。
**Validates: Requirements 4.3**

## Error Handling

### 后端错误处理流程

1. **Repository 层**: 返回 `Result<T, sqlx::Error>`
2. **Service 层**: 将 `sqlx::Error` 转换为 `AppError`，添加业务上下文
3. **Handler 层**: 返回 `Result<Json<ApiResponse<T>>, AppError>`
4. **Axum 框架**: 通过 `IntoResponse` trait 自动转换 `AppError` 为 HTTP 响应

### 前端错误处理流程

1. **网络层**: Dio 拦截器捕获 HTTP 错误，解析为 `ApiError`
2. **Repository 层**: 将 `ApiError` 转换为业务异常或返回 `Result` 类型
3. **Provider 层**: 处理异常，更新 UI 状态
4. **UI 层**: 显示 Toast 或错误页面

## Testing Strategy

### 单元测试

- 测试 `AppError` 的 `error_code()` 和 `status_code()` 方法
- 测试 `ApiResponse` 的序列化
- 测试 `From<sqlx::Error> for AppError` 转换
- 测试前端 `ApiError.fromJson` 解析
- 测试 `DatabaseService` 的数据库打开/关闭逻辑

### 属性测试 (Property-Based Testing)

使用 `proptest` (Rust) 进行属性测试：

- 生成随机 AppError 变体，验证响应格式
- 生成随机错误码，验证范围约束
- 生成随机用户 ID，验证数据库文件名生成

### 集成测试

- 测试完整的 API 请求-响应流程
- 测试错误场景下的响应格式
- 测试前端数据库切换流程
