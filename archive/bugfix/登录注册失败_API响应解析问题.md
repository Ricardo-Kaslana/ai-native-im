# 登录注册失败 - API 响应解析问题

## 问题描述

登录和注册 API 返回 200 成功，但前端无法正常跳转到主页。

**错误日志**：
```
🔵 REQUEST[POST] => /auth/login
✅ RESPONSE[200] => /auth/login
Login error: type 'Null' is not a subtype of type 'Map<String, dynamic>' in type cast
#0      new AuthResponse.fromJson (package:client/data/repositories/auth_repository.dart:108:31)
#1      AuthRepository.login (package:client/data/repositories/auth_repository.dart:46:27)
```

## 问题原因

**服务器返回的 JSON 格式**：
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "xxx",
      "username": "xxx",
      "email": "xxx"
    },
    "access_token": "xxx",
    "refresh_token": "xxx"
  }
}
```

**客户端错误的解析方式**：
```dart
// ❌ 错误：直接解析 response.data（整个响应体）
return AuthResponse.fromJson(response.data);
```

`response.data` 是整个 JSON 对象 `{ success, data, error }`，而 `AuthResponse.fromJson` 期望的是 `data` 字段内的对象。

当尝试访问 `json['user']` 时，实际访问的是顶层的 `user` 字段（不存在），返回 `null`，导致类型转换失败。

## 解决方案

修改 `auth_repository.dart`，先提取 `data` 字段再解析：

```dart
// ✅ 正确：先提取 data 字段
final responseData = response.data as Map<String, dynamic>;
final data = responseData['data'] as Map<String, dynamic>;
return AuthResponse.fromJson(data);
```

## 修改内容

### `client/lib/data/repositories/auth_repository.dart`

**修改的方法**：

1. `register()`:
```dart
// 修改前
return AuthResponse.fromJson(response.data);

// 修改后
final responseData = response.data as Map<String, dynamic>;
final data = responseData['data'] as Map<String, dynamic>;
return AuthResponse.fromJson(data);
```

2. `login()`:
```dart
// 修改前
return AuthResponse.fromJson(response.data);

// 修改后
final responseData = response.data as Map<String, dynamic>;
final data = responseData['data'] as Map<String, dynamic>;
return AuthResponse.fromJson(data);
```

3. `refreshToken()`:
```dart
// 修改前
return AuthResponse.fromJson(response.data);

// 修改后
final responseData = response.data as Map<String, dynamic>;
final data = responseData['data'] as Map<String, dynamic>;
return AuthResponse.fromJson(data);
```

4. `getCurrentUser()`:
```dart
// 修改前
return UserInfo.fromJson(response.data);

// 修改后
final responseData = response.data as Map<String, dynamic>;
final data = responseData['data'] as Map<String, dynamic>;
return UserInfo.fromJson(data);
```

## 相关问题

同时发现并修复了另一个问题：登录成功后没有初始化用户数据库。

### `client/lib/presentation/providers/auth_provider.dart`

**添加数据库初始化**：
```dart
// 在 login() 和 register() 成功后添加
await DatabaseService.instance.openUserDatabase(response.userId);

// 在 logout() 中添加
await DatabaseService.instance.closeUserDatabase();
```

## 根本原因

服务器使用统一的 `ApiResponse<T>` 包装器：
```rust
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ErrorInfo>,
}
```

客户端需要先解包 `data` 字段才能获取实际数据。

## 预防措施

1. 可以创建一个通用的响应解析工具函数：
```dart
T parseApiResponse<T>(dynamic responseData, T Function(Map<String, dynamic>) fromJson) {
  final response = responseData as Map<String, dynamic>;
  if (response['success'] != true) {
    throw ApiException(message: response['error']?['message'] ?? 'Unknown error');
  }
  final data = response['data'] as Map<String, dynamic>;
  return fromJson(data);
}
```

2. 在 API 文档中明确说明响应格式

## 状态

✅ 已修复

## 日期

2024-12-07
