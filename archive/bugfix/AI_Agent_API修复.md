# AI Agent API 修复

## 问题描述

编译错误：
```
lib/presentation/providers/ai_agent_provider.dart(9,31): error G4127D1E8: 
The getter 'dio' isn't defined for the type 'ApiClient'.
```

## 问题原因

`AIAgentApi` 类期望接收一个 `Dio` 实例，但 `ApiClient` 类没有公开的 `dio` getter。

**原始代码**：
```dart
// ai_agent_provider.dart
final aiAgentApiProvider = Provider<AIAgentApi>((ref) {
  final apiClient = ref.watch(apiClientProvider);
  return AIAgentApi(apiClient.dio); // ❌ dio getter 不存在
});

// ai_agent_api.dart
class AIAgentApi {
  final Dio _dio;
  AIAgentApi(this._dio);
  // 使用 _dio.get(), _dio.post() 等
}
```

## 解决方案

修改 `AIAgentApi` 使用 `ApiClient` 而不是 `Dio`，这样可以：
1. 利用 `ApiClient` 的拦截器（日志、错误处理）
2. 自动添加认证 token
3. 统一的错误处理

## 修改内容

### 1. 修改 `ai_agent_api.dart`

**导入**：
```dart
import 'api_client.dart';
```

**构造函数**：
```dart
class AIAgentApi {
  final ApiClient _client;  // 改为 ApiClient
  AIAgentApi(this._client);
}
```

**所有方法**：
```dart
// 修改前
final response = await _dio.get('/agents/$agentId');

// 修改后
final response = await _client.get('/agents/$agentId');
```

修改的方法：
- `createAgent()` - `_dio.post` → `_client.post`
- `getAgentForGroup()` - `_dio.get` → `_client.get`
- `getAgent()` - `_dio.get` → `_client.get`
- `deleteAgent()` - `_dio.delete` → `_client.delete`
- `updatePersonality()` - `_dio.put` → `_client.put`
- `getStatistics()` - `_dio.get` → `_client.get`
- `setAffinity()` - `_dio.post` → `_client.post`
- `getAllAgents()` - `_dio.get` → `_client.get`

### 2. 修改 `ai_agent_provider.dart`

```dart
// 修改前
final aiAgentApiProvider = Provider<AIAgentApi>((ref) {
  final apiClient = ref.watch(apiClientProvider);
  return AIAgentApi(apiClient.dio); // ❌
});

// 修改后
final aiAgentApiProvider = Provider<AIAgentApi>((ref) {
  final apiClient = ref.watch(apiClientProvider);
  return AIAgentApi(apiClient); // ✅
});
```

## 优势

使用 `ApiClient` 而不是直接使用 `Dio` 的优势：

1. **统一的日志**：
   - 自动打印请求和响应
   - 包含 🔵 REQUEST、✅ RESPONSE、❌ ERROR 标记

2. **自动认证**：
   - 自动添加 `Authorization: Bearer <token>` 头
   - 无需在每个请求中手动添加

3. **错误处理**：
   - 统一的错误拦截
   - 自动处理 401 未授权错误
   - 可以添加全局错误处理逻辑

4. **一致性**：
   - 与其他 API 客户端（如 `MessageRepository`）保持一致
   - 更容易维护和扩展

## 测试

编译应该成功：
```bash
flutter run -d windows
```

## 状态

✅ 已修复
