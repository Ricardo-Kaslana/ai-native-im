# Phase 4 联系人与群组功能架构总结

## 概述

本文档总结了Phase 4（联系人与群组功能）的架构设计，遵循Clean Architecture原则和项目的开发规范。

## 联系人模块架构

### 1. 目录结构

```
lib/src/features/contacts/
├── data/                          # 数据层
│   ├── datasources/              # 数据源
│   │   ├── contact_remote_datasource.dart       # 接口定义
│   │   └── contact_remote_datasource_impl.dart  # 实现
│   ├── models/                   # 数据模型
│   │   ├── contact_model.dart                  # 联系人模型
│   │   └── friend_request_model.dart           # 好友请求模型
│   └── repositories/             # 仓储实现
│       └── contact_repository_impl.dart       # 联系人仓储实现
├── domain/                        # 业务层
│   ├── entities/                 # 实体
│   │   ├── contact.dart                         # 联系人实体
│   │   └── friend_request.dart                  # 好友请求实体
│   ├── repositories/             # 仓储接口
│   │   └── contact_repository.dart              # 联系人仓储接口
│   └── usecases/                 # 用例
│       └── get_contacts_usecase.dart            # 获取联系人用例
└── presentation/                 # 表现层
    ├── pages/                    # 页面
    │   └── contacts_page.dart                    # 联系人页面
    ├── widgets/                 # 组件
    └── providers/               # 状态管理
```

### 2. 核心实体

#### Contact (联系人)
```dart
class Contact {
  final String id;
  final String userId;
  final String username;
  final ContactStatus status;
  final bool isOnline;
  // ... 其他字段

  String get displayName; // 优先显示备注名
  bool get isFriend;        // 是否是好友
  String get statusDescription; // 状态描述
}
```

#### FriendRequest (好友请求)
```dart
class FriendRequest {
  final String id;
  final ContactUser fromUser;
  final ContactUser toUser;
  final FriendRequestStatus status;
  final DateTime createdAt;
  // ... 其他字段
}
```

### 3. 数据层实现

#### Repository接口定义
- 获取联系人列表（支持分页、搜索、筛选）
- 根据ID获取联系人
- 发送/处理好友请求
- 更新联系人备注
- 删除/拉黑联系人
- 流式数据监听

#### RemoteDataSource
- 使用Dio进行HTTP请求
- 支持查询参数和请求体
- 统一错误处理
- JSON序列化/反序列化

### 4. UI层设计

#### 页面结构
- 自定义AppBar（与整体设计保持一致）
- 搜索栏（支持实时搜索）
- Tab切换（好友/请求/发现）
- 空状态提示

## 群组模块架构（待实现）

### 1. 核心功能
- 群组创建/管理
- 成员管理（邀请/移除）
- 权限控制（群主/管理员/成员）
- 群组设置
- 邀请码系统

### 2. 目录结构（规划）
```
lib/src/features/groups/
├── data/
│   ├── datasources/
│   ├── models/
│   └── repositories/
├── domain/
│   ├── entities/
│   │   ├── group.dart
│   │   ├── group_member.dart
│   │   └── group_invite.dart
│   ├── repositories/
│   └── usecases/
└── presentation/
    ├── pages/
    │   ├── groups_page.dart
    │   ├── group_create_page.dart
    │   └── group_settings_page.dart
    └── widgets/
```

## 技术实现要点

### 1. 状态管理
使用Riverpod进行状态管理：
- 联系人列表状态
- 好友请求状态
- 搜索状态
- 加载/错误状态

### 2. 错误处理
统一的错误处理机制：
- 网络错误
- 业务逻辑错误
- 用户友好的错误提示

### 3. 性能优化
- 分页加载联系人列表
- 图片缓存（复用OptimizedImage组件）
- 懒加载列表项
- 防抖搜索

### 4. 实时更新
- WebSocket监听好友请求
- 联系人状态变化
- 群组消息同步

## 待完成功能

### 联系人模块
1. ✅ 基础架构搭建
2. ✅ 数据模型定义
3. ✅ Repository层实现
4. ✅ 基础UI页面
5. ⏳ 搜索功能实现
6. ⏳ 好友请求处理
7. ⏳ 联系人详情页
8. ⏳ 聊天入口

### 群组模块
1. ⏳ 群组数据模型
2. ⏳ 群组CRUD操作
3. ⏳ 成员管理
4. ⏳ 权限控制
5. ⏳ 群组设置页面

## 需要修复的问题

1. **构建问题**
   - JSON序列化版本冲突
   - Drift模型分析错误
   - 版本兼容性问题

2. **依赖更新**
   ```yaml
   dependencies:
     json_annotation: ^4.9.0

   dev_dependencies:
     analyzer: ^9.0.0
   ```

## 下一步计划

1. **修复构建问题**
   - 更新依赖版本
   - 修复模型定义
   - 重新生成代码

2. **完善联系人功能**
   - 实现搜索功能
   - 处理好友请求
   - 添加联系人详情页

3. **开发群组功能**
   - 创建群组数据模型
   - 实现群组管理
   - 开发群组聊天

4. **集成测试**
   - API集成测试
   - UI交互测试
   - 性能测试

## 总结

Phase 4的联系人模块架构已经搭建完成，遵循了Clean Architecture原则，实现了分层设计。虽然还有一些功能需要完善，但基础架构已经为后续开发提供了良好的基础。

---

**创建时间**: 2025-12-13
**状态**: 架构完成，功能开发中