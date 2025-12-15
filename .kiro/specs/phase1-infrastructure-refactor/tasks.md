# Implementation Plan - Phase 1: Infrastructure Refactor

## 1. 后端统一错误处理

- [x] 1.1 创建 AppError 枚举和错误码模块
  - 在 `server/src/` 下创建 `error.rs` 文件
  - 定义 `AppError` 枚举，包含所有错误变体
  - 定义 `error_codes` 模块，包含错误码常量
  - 实现 `error_code()` 和 `status_code()` 方法
  - _Requirements: 1.1, 1.2, 5.1, 5.3_

- [x] 1.2 编写 AppError 属性测试
  - **Property 2: 错误码到状态码映射正确性**
  - **Property 6: 错误码唯一性**
  - **Property 7: 错误码范围正确性**
  - **Validates: Requirements 1.2, 5.1, 5.3**

- [x] 1.3 实现 AppError 的 IntoResponse trait
  - 实现 `IntoResponse for AppError`
  - 返回统一的 JSON 错误响应格式
  - 设置正确的 HTTP 状态码
  - _Requirements: 1.1, 1.2_

- [x] 1.4 编写错误响应格式属性测试
  - **Property 1: 错误响应格式一致性**
  - **Validates: Requirements 1.1, 2.2**

- [x] 1.5 实现 From trait 进行错误转换
  - 实现 `From<sqlx::Error> for AppError`
  - 实现 `From<AIAgentError> for AppError`
  - 实现 `From<GroupError> for AppError`
  - 实现 `From<MediaError> for AppError`
  - _Requirements: 1.3, 1.6_

- [x] 1.6 在 lib.rs 中导出错误模块
  - 添加 `pub mod error;`
  - 导出 `AppError` 和 `error_codes`
  - _Requirements: 1.1_

## 2. 后端统一响应格式

- [x] 2.1 创建 ApiResponse 响应结构
  - 在 `server/src/` 下创建 `response.rs` 文件
  - 定义 `ApiResponse<T>` 泛型结构
  - 定义 `ErrorDetail` 结构
  - 定义 `ListResponse<T>` 和 `PaginatedResponse<T>`
  - 实现 `success()` 和 `error()` 构造方法
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [x] 2.2 编写响应格式属性测试
  - **Property 3: 成功响应格式一致性**
  - **Property 4: 列表响应格式一致性**
  - **Validates: Requirements 2.1, 2.3**

- [x] 2.3 在 lib.rs 中导出响应模块
  - 添加 `pub mod response;`
  - 导出 `ApiResponse`, `ListResponse`, `PaginatedResponse`
  - _Requirements: 2.1_

## 3. 迁移现有 API 使用新错误处理

- [x] 3.1 迁移 auth API
  - 修改 `api/auth.rs` 使用 `AppError`
  - 修改返回类型为 `Result<Json<ApiResponse<T>>, AppError>`
  - 移除直接返回 `StatusCode` 的代码
  - _Requirements: 1.1, 2.1_

- [x] 3.2 迁移 users API
  - 修改 `api/users.rs` 使用 `AppError`
  - 统一响应格式
  - _Requirements: 1.1, 2.1_

- [x] 3.3 迁移 conversations API
  - 修改 `api/conversations.rs` 使用 `AppError`
  - 统一响应格式
  - _Requirements: 1.1, 2.1_

- [x] 3.4 迁移 groups API
  - 修改 `api/groups.rs` 使用 `AppError`
  - 统一响应格式（已有自定义 ApiResponse，需要统一）
  - _Requirements: 1.1, 2.1_

- [x] 3.5 迁移 ai_agents API
  - 修改 `api/ai_agents.rs` 使用 `AppError`
  - 统一响应格式
  - _Requirements: 1.1, 2.1_

- [x] 3.6 迁移 contacts API
  - 修改 `api/contacts.rs` 使用 `AppError`
  - 统一响应格式
  - _Requirements: 1.1, 2.1_

- [x] 4. Checkpoint - 确保后端测试通过
  - Ensure all tests pass, ask the user if questions arise.

## 5. 前端数据库用户隔离

- [x] 5.1 创建 DatabaseService 单例
  - 在 `client/lib/core/database/` 下创建 `database_service.dart`
  - 实现单例模式
  - 实现 `getGlobalDatabase()` 方法
  - 实现 `openUserDatabase(String userId)` 方法
  - 实现 `closeUserDatabase()` 方法
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 5.2 编写数据库隔离属性测试
  - **Property 5: 用户数据库隔离**
  - **Validates: Requirements 3.1, 3.3**
  - 测试文件: `client/test/unit/database_isolation_test.dart`

- [x] 5.3 创建全局数据库 Schema
  - 在 `database.dart` 中添加 `Accounts` 表
  - 添加 `AppSettings` 表
  - 生成 Drift 代码
  - _Requirements: 3.5_

- [x] 5.4 修改现有数据库使用方式
  - 修改 `database_provider.dart` 使用 `DatabaseService`
  - 添加 `openUserDatabaseProvider` 和 `closeUserDatabaseProvider`
  - 在登录成功后调用 `openUserDatabase`
  - 在登出时调用 `closeUserDatabase`
  - _Requirements: 3.1, 3.2_

- [x] 5.5 迁移现有数据到用户数据库
  - DatabaseService 自动处理用户数据库创建
  - 旧数据库文件保留，新用户使用新的隔离数据库
  - _Requirements: 3.1_

## 6. 前端错误处理优化

- [x] 6.1 创建 ErrorHandler 类
  - 在 `client/lib/core/error/` 下创建 `error_handler.dart`
  - 实现 `initialize()` 方法捕获全局错误
  - 实现错误日志记录
  - _Requirements: 4.1_

- [x] 6.2 创建自定义错误页面
  - 创建 `AppErrorWidget` 组件
  - 设计友好的错误提示 UI
  - 在 `main.dart` 中配置 `ErrorWidget.builder`
  - _Requirements: 4.1_

- [x] 6.3 创建 ApiError 类
  - 在 `client/lib/core/network/` 下创建 `api_error.dart`
  - 实现 `fromJson` 工厂方法
  - 实现 `userMessage` getter 进行错误码到用户消息的映射
  - _Requirements: 4.4_

- [x] 6.4 编写错误消息映射属性测试
  - **Property: 错误码到用户消息映射**
  - **Validates: Requirements 4.4**
  - 测试文件: `client/test/unit/error_mapping_test.dart`

- [x] 6.5 修改 ApiClient 使用新错误处理
  - 添加 `_ErrorInterceptor` 在 Dio 拦截器中解析错误响应
  - 将 DioException 转换为 `ApiError`
  - 添加 `AuthErrorCallback` 处理认证错误
  - _Requirements: 4.2, 4.4_

- [x] 6.6 添加 Toast 错误提示
  - 创建 `ToastUtils` 工具类
  - 实现 `showError`, `showApiError`, `showNetworkError` 方法
  - 提供重试选项
  - 文件: `client/lib/core/utils/toast_utils.dart`
  - _Requirements: 4.2_

- [x] 6.7 优化 WebSocket 断连处理
  - 创建 `ConnectionStatusIndicator` 组件
  - 创建 `ConnectionStatusDot` 紧凑指示器
  - 创建 `ConnectionStatusBanner` 滑动横幅
  - 显示重连进度
  - 文件: `client/lib/presentation/widgets/connection_status_indicator.dart`
  - _Requirements: 4.3_

- [x] 6.8 编写 WebSocket 重连属性测试
  - **Property 8: WebSocket 重连行为**
  - **Validates: Requirements 4.3**
  - 测试文件: `client/test/unit/websocket_reconnect_test.dart`

## 7. 创建错误码文档

- [x] 7.1 创建错误码参考文档
  - 在 `archive/refactor/` 下创建 `ERROR_CODES.md`
  - 列出所有错误码、描述和用户提示
  - 包含错误响应格式说明和最佳实践
  - _Requirements: 5.2_

## 8. Final Checkpoint - 确保所有测试通过
  - [x] 运行后端测试: `cargo test --test error_property_tests` - 18 tests passed ✅
  - [x] 运行前端测试: `flutter test test/unit/` - 34 tests passed ✅
  - All tests pass!
