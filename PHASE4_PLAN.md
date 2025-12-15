# Phase 4 开发计划 - 联系人与群组功能

## 阶段目标
实现完整的联系人管理和群组功能，包括联系人添加、群组创建、成员管理、权限控制等核心社交功能。

## 功能范围

### 1. 联系人管理
- **搜索联系人**：通过手机号/ID搜索用户
- **添加联系人**：发送好友请求、处理好友请求
- **联系人列表**：展示所有联系人（首字母排序）
- **联系人详情**：查看联系人信息、修改备注
- **黑名单**：屏蔽指定用户

### 2. 群组管理
- **创建群组**：群组名称、头像、简介设置
- **加入群组**：通过邀请码或邀请链接加入
- **群组信息**：查看群组详情、成员列表
- **群组设置**：修改群组信息、设置管理员
- **退出群组**：主动退出或被移出

### 3. 成员管理
- **邀请成员**：邀请联系人加入群组
- **移除成员**：群主/管理员移除群成员
- **权限管理**：设置管理员权限
- **成员角色**：群主、管理员、普通成员

### 4. 群聊功能
- **群组聊天**：多人群聊消息收发
- **@提及功能**：@特定成员或@全体成员
- **群公告**：发布和管理群公告
- **消息免打扰**：设置群组消息提醒

## 实现计划

### Week 1：联系人管理（2025-01-14 ~ 2025-01-20）
- [ ] 联系人搜索功能实现
- [ ] 好友请求系统
- [ ] 联系人列表页面
- [ ] 联系人详情页面
- [ ] 添加/删除联系人

### Week 2：群组基础功能（2025-01-21 ~ 2025-01-27）
- [ ] 创建群组页面
- [ ] 群组信息页面
- [ ] 加入群组功能
- [ ] 群组设置页面
- [ ] 群组列表展示

### Week 3：成员管理（2025-01-28 ~ 2025-02-03）
- [ ] 邀请成员功能
- [ ] 成员列表页面
- [ ] 权限管理系统
- [ ] 移除成员功能
- [ ] 成员角色设置

### Week 4：群聊功能（2025-02-04 ~ 2025-02-10）
- [ ] 群组消息收发
- [ ] @提及功能
- [ ] 群公告系统
- [ ] 消息免打扰设置
- [ ] 群聊特殊功能优化

## 技术设计

### 数据模型
```dart
// 联系人
class Contact {
  final String id;
  final String userId;
  final String name;
  final String avatar;
  final String remark;
  final List<String> tags;
  final bool isBlocked;
  final DateTime addedAt;
}

// 好友请求
class FriendRequest {
  final String id;
  final String fromUserId;
  final String toUserId;
  final String message;
  final FriendRequestStatus status;
  final DateTime createdAt;
}

// 群组
class Group {
  final String id;
  final String name;
  final String avatar;
  final String description;
  final String ownerId;
  final List<GroupMember> members;
  final GroupSettings settings;
  final DateTime createdAt;
}

// 群成员
class GroupMember {
  final String userId;
  final String nickname;
  final GroupRole role;
  final DateTime joinedAt;
  final bool isMuted;
}

// 群组设置
class GroupSettings {
  final bool allowInvite;
  final bool needApproval;
  final bool allowMembersInvite;
  final String inviteCode;
  final DateTime? inviteCodeExpireAt;
}
```

### API 接口设计
```
# 联系人相关
POST /api/contacts/search         # 搜索联系人
POST /api/contacts/add            # 添加联系人
DELETE /api/contacts/:id          # 删除联系人
PUT /api/contacts/:id/remark      # 修改备注

# 好友请求
POST /api/friend-requests/send    # 发送好友请求
POST /api/friend-requests/accept  # 接受请求
POST /api/friend-requests/reject   # 拒绝请求

# 群组相关
POST /api/groups                   # 创建群组
GET /api/groups/:id                # 获取群信息
PUT /api/groups/:id                # 更新群信息
POST /api/groups/:id/join          # 加入群组
DELETE /api/groups/:id/members/:userId  # 移除成员

# 群成员
POST /api/groups/:id/invite        # 邀请成员
GET /api/groups/:id/members        # 获取成员列表
PUT /api/groups/:id/members/:userId/role  # 设置成员角色
```

### 页面路由
```
/contacts                    # 联系人列表
/contacts/search             # 联系人搜索
/contacts/:id                # 联系人详情
/groups                      # 群组列表
/groups/create              # 创建群组
/groups/:id                  # 群组信息/聊天
/groups/:id/members          # 成员列表
/groups/:id/settings         # 群组设置
/invites                     # 好友请求列表
```

## UI 设计要点

### 1. 联系人页面
- 顶部搜索框
- 字母索引导航
- 在线状态显示
- 长按显示操作菜单

### 2. 群组页面
- 群头像和基本信息
- 成员数量显示
- 群消息预览
- 未读消息角标

### 3. 成员管理页面
- 成员列表（角色标识）
- 搜索成员功能
- 批量操作支持
- 权限设置开关

### 4. 群聊页面
- 群名称和成员数
- @成员自动完成
- 群公告条显示
- 管理员工具菜单

## 开发优先级

### 高优先级
1. 联系人搜索和添加
2. 创建群组
3. 邀请成员
4. 基础群聊功能

### 中优先级
1. 联系人备注和标签
2. 群组设置
3. 权限管理
4. @提及功能

### 低优先级
1. 黑名单功能
2. 消息免打扰
3. 群公告
4. 高级权限设置

## 测试计划

### 单元测试
- [ ] Contact 数据模型测试
- [ ] FriendRequest 状态转换测试
- [ ] Group 权限验证测试

### 集成测试
- [ ] 添加联系人流程测试
- [ ] 创建群组流程测试
- [ ] 邀请成员流程测试

### UI 测试
- [ ] 联系人页面交互测试
- [ ] 群组设置页面测试
- [ ] 成员管理功能测试

## 风险评估

### 技术风险
- WebSocket 群消息广播复杂性
- 大量群成员的性能问题
- 权限系统的并发控制

### 时间风险
- 功能较多，可能需要调整优先级
- 复杂交互可能需要更多时间

### 缓解措施
- 先实现核心功能，后续迭代优化
- 使用分页加载处理大量数据
- 提前进行性能测试