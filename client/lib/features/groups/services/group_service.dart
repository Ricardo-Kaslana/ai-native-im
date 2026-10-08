import '../../../core/network/api_client.dart';
import '../models/group.dart';

/// 群组服务
class GroupService {
  static final GroupService instance = GroupService._();
  GroupService._();

  final _api = ApiClient.instance;

  /// 获取我的群组列表
  Future<List<Group>> getGroups() async {
    return await _api.get<List<Group>>(
      '/groups/my',
      fromJson: (json) {
        // 后端返回 { success: true, data: [...] }，ApiClient 提取后是数组
        final List list;
        if (json is List) {
          list = json;
        } else if (json is Map) {
          list = json['groups'] as List? ?? json['data'] as List? ?? [];
        } else {
          list = [];
        }
        return list
            .map((e) => Group.fromJson(e as Map<String, dynamic>))
            .toList();
      },
    );
  }

  /// 获取单个群组
  Future<Group> getGroup(String groupId) async {
    return await _api.get<Group>(
      '/groups/$groupId',
      fromJson: (json) {
        final groupData = json['group'] ?? json;
        return Group.fromJson(groupData as Map<String, dynamic>);
      },
    );
  }

  /// 创建群组
  Future<Group> createGroup({
    required String name,
    String? description,
    bool? isPublic,
    required List<String> memberIds,
  }) async {
    return await _api.post<Group>(
      '/groups',
      data: {
        'name': name,
        if (description != null) 'description': description,
        if (isPublic != null) 'is_public': isPublic,
        'member_ids': memberIds,
      },
      fromJson: (json) => Group.fromJson(json as Map<String, dynamic>),
    );
  }

  /// 更新群组
  Future<Group> updateGroup(
    String groupId, {
    String? name,
    String? description,
  }) async {
    return await _api.put<Group>(
      '/groups/$groupId',
      data: {
        if (name != null) 'name': name,
        if (description != null) 'description': description,
      },
      fromJson: (json) => Group.fromJson(json as Map<String, dynamic>),
    );
  }

  /// 获取群组成员
  Future<List<GroupMember>> getMembers(String groupId) async {
    return await _api.get<List<GroupMember>>(
      '/groups/$groupId/members',
      fromJson: (json) {
        // 后端返回 { success: true, data: [...] }，ApiClient 提取后是数组
        final List list;
        if (json is List) {
          list = json;
        } else if (json is Map) {
          list = json['members'] as List? ?? json['data'] as List? ?? [];
        } else {
          list = [];
        }
        return list
            .map((e) => GroupMember.fromJson(e as Map<String, dynamic>))
            .toList();
      },
    );
  }

  /// 添加成员（单个）
  Future<void> addMember(String groupId, String userId) async {
    await _api.postEmpty(
      '/groups/$groupId/members',
      data: {'user_id': userId},
    );
  }

  /// 添加多个成员
  Future<void> addMembers(String groupId, List<String> userIds) async {
    // 后端只支持单个添加，需要循环调用
    for (final userId in userIds) {
      await addMember(groupId, userId);
    }
  }

  /// 移除成员
  Future<void> removeMember(String groupId, String userId) async {
    await _api.deleteEmpty('/groups/$groupId/members/$userId');
  }

  /// 退出群组
  Future<void> leaveGroup(String groupId) async {
    await _api.postEmpty('/groups/$groupId/leave');
  }

  /// 解散群组
  Future<void> deleteGroup(String groupId) async {
    await _api.deleteEmpty('/groups/$groupId');
  }

  /// 生成邀请码
  Future<String> generateInviteCode(String groupId, {int? expiresInHours, int? maxUses}) async {
    final result = await _api.post<Map<String, dynamic>>(
      '/groups/$groupId/invite-codes',
      data: {
        if (expiresInHours != null) 'expires_in_hours': expiresInHours,
        if (maxUses != null) 'max_uses': maxUses,
      },
      fromJson: (json) => json as Map<String, dynamic>,
    );
    return result['code'] as String;
  }

  /// 通过邀请码加入群组
  Future<Group> joinByInviteCode(String code) async {
    return await _api.post<Group>(
      '/groups/join',
      data: {'code': code},
      fromJson: (json) {
        final groupData = json['group'] ?? json;
        return Group.fromJson(groupData as Map<String, dynamic>);
      },
    );
  }

  /// 搜索群组
  Future<List<Group>> searchGroups(String query, {int limit = 20}) async {
    return await _api.get<List<Group>>(
      '/groups/search',
      queryParameters: {'q': query, 'limit': limit.toString()},
      fromJson: (json) {
        // 后端返回 { success: true, data: [...] }，ApiClient 提取后是数组
        final List list;
        if (json is List) {
          list = json;
        } else if (json is Map) {
          list = json['groups'] as List? ?? json['data'] as List? ?? [];
        } else {
          list = [];
        }
        return list
            .map((e) => Group.fromJson(e as Map<String, dynamic>))
            .toList();
      },
    );
  }
}
