# 批量 API 修复指南

## 修复模式总结

已成功建立了统一的 API 响应处理和错误处理模式，可以快速应用到剩余的方法。

### 1. 标准修复模式

#### 对于有返回值的方法：
```dart
try {
  final response = await _apiClient.xxx<Map<String, dynamic>>('/path', data: data);

  final apiResponse = ApiResponse.fromDioResponse(
    response,
    dataParser: (data) => YourModel.fromJson(data['field']),
  );

  if (apiResponse.isSuccess && apiResponse.data != null) {
    return apiResponse.data!;
  } else {
    final error = apiResponse.error;
    throw ApiException(
      message: error?.message ?? '操作失败',
      code: error?.code ?? -1,
    );
  }
} on DioException catch (e) {
  throw ApiException(
    message: ErrorService.getFriendlyMessage(e),
    code: -1,
  );
} catch (e) {
  throw ApiException(
    message: '操作失败: $e',
    code: -1,
  );
}
```

#### 对于 void 返回值的方法：
```dart
try {
  final response = await _apiClient.xxx<Map<String, dynamic>>('/path', data: data);

  final apiResponse = ApiResponse.fromDioResponse(response);

  if (!apiResponse.isSuccess) {
    final error = apiResponse.error;
    throw ApiException(
      message: error?.message ?? '操作失败',
      code: error?.code ?? -1,
    );
  }
} on DioException catch (e) {
  throw ApiException(
    message: ErrorService.getFriendlyMessage(e),
    code: -1,
  );
} catch (e) {
  throw ApiException(
    message: '操作失败: $e',
    code: -1,
  );
}
```

### 2. 批量修复步骤

#### 剩余需要修复的文件：

1. **groups_remote_data_source.dart**
   - 已修复：6个方法
   - 剩余：约16个方法
   - 重点关注：removeMember, updateMemberRole, banMember, unbanMember, generateInviteCode, joinGroup, searchGroups, getUserGroups, leaveGroup, getGroupStats, uploadGroupAvatar, getInviteCodes

2. **ai_agent_repository.dart**
   - 需要检查并应用统一模式

3. **media_repository.dart**
   - 需要检查并应用统一模式

4. **Service 层文件**
   - sync_service.dart
   - media_service.dart
   - invite_service.dart

### 3. 快速修复技巧

#### 使用搜索替换：
1. 搜索模式：`await _apiClient.`
2. 将每个 API 调用包装在 try-catch 中
3. 添加 ApiResponse 处理
4. 更新错误抛出为 ApiException

#### 常见替换模板：
- `return Model.fromJson(response['data']);`
  → 使用 ApiResponse 包装

- `throw Exception('message')`
  → 改为 `throw ApiException(message: 'message', code: -1)`

### 4. 测试建议

修复完成后需要进行：
1. **语法检查**：确保没有编译错误
2. **单元测试**：测试 API 调用和错误处理
3. **集成测试**：验证与后端的交互正常

### 5. 优先级

1. **高优先级**：groups_remote_data_source.dart（已完成部分）
2. **中优先级**：ai_agent_repository.dart, media_repository.dart
3. **低优先级**：service 层文件

## 总结

核心的修复模式已经建立并验证有效。剩余工作主要是将此模式应用到其他文件。每个方法的修复大约需要2-3分钟，总共约40-50个方法，预计需要2-3小时完成所有修复。