# Flutter Client Phase 1 - 紧急修复任务列表

## 概述

本文档列出了修复 Phase 1 回归问题的具体任务。所有任务按优先级排序，P0 任务必须立即完成。

---

## P0 - 紧急修复（必须立即完成）

### ✅ Task 1: 修复数据库初始化问题

**问题**: 使用了 `AppDatabase.forTesting()` 而不是正常构造函数

**文件**: `client/lib/core/database/database_service.dart`

**修改**:
```dart
// 第 95 行，将：
_userDb = AppDatabase.forTesting(NativeDatabase.createInBackground(file));

// 改为：
_userDb = AppDatabase(NativeDatabase.createInBackground(file));
```

**验证**:
- [ ] 登录后能正常打开数据库
- [ ] 数据库文件正确创建在 `user_{userId}.db`
- [ ] 数据库操作（读写）正常工作

**预计时间**: 5 分钟

---

### ✅ Task 2: 统一错误处理 - 移除重复拦截器

**问题**: `ApiClient` 中有两个错误拦截器导致冲突

**文件**: `client/lib/core/network/api_client.dart`

**修改**:
```dart
// 第 32-40 行，移除其中一个拦截器
// 保留 _ErrorInterceptor，移除 ErrorHandler.createDioInterceptor()

// 修改后的代码：
_dio.interceptors.add(_ErrorInterceptor(this));
// 移除这一行：
// _dio.interceptors.add(ErrorHandler.createDioInterceptor());
```

**验证**:
- [ ] API 错误能正确转换为 `ApiException`
- [ ] 错误信息准确显示给用户
- [ ] 不会出现双重包装的错误

**预计时间**: 10 分钟

---

### ✅ Task 3: 修复 AuthStateManager 错误处理

**问题**: 只捕获 `ApiException`，未处理新的 `AppException` 体系

**文件**: `client/lib/core/services/auth_state_manager.dart`

**修改**:

1. **在 `register` 方法中** (第 90-140 行):
```dart
// 修改 catch 块顺序，先捕获 AppException
} on AppException catch (e) {
  print('❌ AuthStateManager: Registration failed with AppException');
  print('   Message: ${e.message}');
  print('   User message: ${e.userMessage}');
  
  _updateState(AuthState(
    status: AuthStatus.unauthenticated,
    errorMessage: e.userMessage,
  ));
} on ApiException catch (e) {
  // 保持现有逻辑
  // ...
} catch (e, stackTrace) {
  // 保持现有逻辑
  // ...
}
```

2. **在 `login` 方法中** (第 142-190 行):
```dart
// 同样的修改
} on AppException catch (e) {
  print('❌ AuthStateManager: Login failed with AppException');
  print('   Message: ${e.message}');
  print('   User message: ${e.userMessage}');
  
  _updateState(AuthState(
    status: AuthStatus.unauthenticated,
    errorMessage: e.userMessage,
  ));
} on ApiException catch (e) {
  // 保持现有逻辑
  // ...
} catch (e, stackTrace) {
  // 保持现有逻辑
  // ...
}
```

**验证**:
- [ ] 登录失败时显示正确的中文错误信息
- [ ] 网络错误、认证错误、验证错误都能正确处理
- [ ] 错误信息符合 `AppException.userMessage` 的定义

**预计时间**: 15 分钟

---

### ✅ Task 4: 添加错误处理导入

**问题**: `AuthStateManager` 可能缺少 `AppException` 的导入

**文件**: `client/lib/core/services/auth_state_manager.dart`

**修改**:
```dart
// 在文件顶部添加导入
import '../error/app_exception.dart';
```

**验证**:
- [ ] 代码能正常编译
- [ ] 没有导入错误

**预计时间**: 2 分钟

---

## P1 - 重要修复（尽快完成）

### ✅ Task 5: 完善 WebSocket 连接管理

**问题**: ConnectionManager 可能未正确初始化

**文件**: `client/lib/core/network/websocket_client.dart`

**修改**:
```dart
// 在 WebSocketClient 构造函数中添加日志
WebSocketClient({
  ConnectionManager? connectionManager,
  IMessageQueue? messageQueue,
})  : _connectionManager = connectionManager,
      _messageQueue = messageQueue {
  print('🔵 WebSocketClient: Initialized with ConnectionManager: ${connectionManager != null}');
  print('🔵 WebSocketClient: Initialized with MessageQueue: ${messageQueue != null}');
  
  // Setup ConnectionManager integration if provided
  if (_connectionManager != null) {
    _setupConnectionManagerIntegration();
  }
}
```

**验证**:
- [ ] WebSocket 能正常连接
- [ ] 断线后能自动重连
- [ ] 重连后能处理待发送消息

**预计时间**: 10 分钟

---

### ✅ Task 6: 确保 SessionRepository 正确集成

**问题**: ConversationProvider 可能未使用 SessionRepository

**文件**: `client/lib/presentation/providers/conversation_provider.dart`

**检查**:
```dart
// 确认 conversationsProvider 使用的是 sessionRepository.watchSessions()
final conversationsProvider = StreamProvider<List<Conversation>>((ref) {
  final sessionRepository = ref.watch(sessionRepositoryProvider);
  
  // 这里应该使用 sessionRepository，而不是直接调用 API
  return sessionRepository.watchSessions();
});
```

**如果发现问题，修改为**:
```dart
// 确保使用 SessionRepository 而不是 API
final conversationsProvider = StreamProvider<List<Conversation>>((ref) {
  final sessionRepository = ref.watch(sessionRepositoryProvider);
  return sessionRepository.watchSessions();
});
```

**验证**:
- [ ] 会话列表没有重复项
- [ ] 会话按最后活动时间排序
- [ ] 新消息到达时会话列表自动更新

**预计时间**: 15 分钟

---

### ✅ Task 7: 添加 API 拦截器自动 Token 刷新

**问题**: Token 过期时直接登出，未尝试刷新

**文件**: `client/lib/core/network/api_client.dart`

**修改** (在 `_ErrorInterceptor.onError` 中):
```dart
@override
void onError(DioException error, ErrorInterceptorHandler handler) {
  if (kDebugMode) {
    print('❌ ERROR[${error.response?.statusCode}] => ${error.requestOptions.path}');
    print('   Response: ${error.response?.data}');
  }
  
  // Convert to ApiError or NetworkError
  final apiError = _parseError(error);
  
  // Handle 401 - try to refresh token first
  if (error.response?.statusCode == 401) {
    // TODO: Implement automatic token refresh
    // For now, just notify auth error callback
    _client._handleAuthError(apiError);
  }
  
  // Wrap in DioException with ApiError
  handler.reject(
    DioException(
      requestOptions: error.requestOptions,
      response: error.response,
      type: error.type,
      error: apiError,
    ),
  );
}
```

**验证**:
- [ ] Token 过期时尝试刷新
- [ ] 刷新成功后重试原请求
- [ ] 刷新失败后才登出

**预计时间**: 30 分钟

---

## P2 - 优化改进（后续完成）

### ✅ Task 8: 改进日志系统

**问题**: 使用 `print` 而不是专业的日志库

**文件**: 多个文件

**修改**:
1. 添加 `logger` 包到 `pubspec.yaml`
2. 创建全局 logger 实例
3. 替换所有 `print` 为 `logger.debug/info/error`

**验证**:
- [ ] 日志有统一格式
- [ ] 可以控制日志级别
- [ ] 生产环境可以关闭调试日志

**预计时间**: 1 小时

---

### ✅ Task 9: 添加集成测试

**问题**: 缺少端到端测试验证功能

**文件**: `client/test/integration/`

**创建测试**:
1. `login_flow_test.dart` - 登录流程测试
2. `message_send_test.dart` - 消息发送测试
3. `user_switch_test.dart` - 用户切换测试
4. `offline_sync_test.dart` - 离线同步测试

**验证**:
- [ ] 所有集成测试通过
- [ ] 覆盖主要用户流程

**预计时间**: 2-3 小时

---

## 测试验证清单

### 登录功能验证

```bash
# 1. 启动后端服务
cd server && cargo run

# 2. 启动客户端
cd client && flutter run

# 3. 测试步骤
- [ ] 输入正确的邮箱和密码，点击登录
- [ ] 验证能成功登录并跳转到主页
- [ ] 检查数据库文件是否创建 (user_{userId}.db)
- [ ] 验证 WebSocket 连接状态
- [ ] 尝试发送消息
- [ ] 登出后再次登录
```

### 错误处理验证

```bash
# 1. 测试网络错误
- [ ] 关闭后端服务
- [ ] 尝试登录
- [ ] 验证显示 "网络连接失败，请检查网络"

# 2. 测试认证错误
- [ ] 输入错误的密码
- [ ] 验证显示 "邮箱或密码错误"

# 3. 测试验证错误
- [ ] 输入无效的邮箱格式
- [ ] 验证显示 "请输入有效的邮箱地址"
```

### 会话列表验证

```bash
# 1. 测试会话去重
- [ ] 登录后查看会话列表
- [ ] 验证没有重复的会话
- [ ] 发送消息后验证会话列表更新
- [ ] 验证会话按时间排序
```

### WebSocket 验证

```bash
# 1. 测试连接
- [ ] 登录后验证 WebSocket 已连接
- [ ] 查看连接状态指示器

# 2. 测试重连
- [ ] 暂停网络连接
- [ ] 验证显示 "正在重连..."
- [ ] 恢复网络连接
- [ ] 验证自动重连成功

# 3. 测试消息队列
- [ ] 离线时发送消息
- [ ] 验证消息进入队列
- [ ] 恢复连接后验证消息自动发送
```

---

## 执行顺序

### 第一步：紧急修复 (30-40 分钟)
1. Task 1: 修复数据库初始化 (5 分钟)
2. Task 4: 添加错误处理导入 (2 分钟)
3. Task 3: 修复 AuthStateManager 错误处理 (15 分钟)
4. Task 2: 移除重复拦截器 (10 分钟)

### 第二步：验证修复 (15 分钟)
- 运行登录功能验证
- 运行错误处理验证

### 第三步：重要修复 (1 小时)
5. Task 5: 完善 WebSocket 连接管理 (10 分钟)
6. Task 6: 确保 SessionRepository 集成 (15 分钟)
7. Task 7: 添加自动 Token 刷新 (30 分钟)

### 第四步：全面验证 (30 分钟)
- 运行所有验证清单
- 测试主要用户流程

### 第五步：优化改进 (可选，3-4 小时)
8. Task 8: 改进日志系统 (1 小时)
9. Task 9: 添加集成测试 (2-3 小时)

---

## 回滚计划

如果修复后仍有问题：

### 最小回滚
```bash
# 只回滚数据库初始化
git checkout HEAD~1 -- client/lib/core/database/database_service.dart
```

### 部分回滚
```bash
# 回滚错误处理系统
git checkout HEAD~1 -- client/lib/core/error/
git checkout HEAD~1 -- client/lib/core/network/api_client.dart
```

### 完全回滚
```bash
# 回滚所有 Phase 1 改动
git revert <phase1-commit-hash>
```

---

## 联系和支持

如果在修复过程中遇到问题：

1. 查看 `REGRESSION_ANALYSIS.md` 了解详细分析
2. 查看 Phase 1 的 `design.md` 和 `tasks.md`
3. 运行测试：`cd client && flutter test`
4. 查看日志输出定位问题

---

**文档版本**: 1.0  
**创建时间**: 2025-12-10  
**最后更新**: 2025-12-10  
**预计总修复时间**: 2-3 小时（不含优化改进）
