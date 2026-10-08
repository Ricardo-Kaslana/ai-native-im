import '../../../core/network/api_client.dart';
import '../../auth/models/user.dart';

/// 用户服务
class UserService {
  static final UserService instance = UserService._();
  UserService._();

  final _api = ApiClient.instance;

  /// 获取用户信息
  Future<User> getUser(String userId) async {
    return await _api.get<User>(
      '/users/$userId',
      fromJson: (json) {
        final userData = json['user'] ?? json;
        return User.fromJson(userData as Map<String, dynamic>);
      },
    );
  }

  /// 获取当前用户信息
  Future<User> getCurrentUser() async {
    return await _api.get<User>(
      '/users/me',
      fromJson: (json) {
        final userData = json['user'] ?? json;
        return User.fromJson(userData as Map<String, dynamic>);
      },
    );
  }

  /// 更新个人资料
  Future<User> updateProfile({
    String? username,
    String? bio,
    String? avatarUrl,
  }) async {
    final data = <String, dynamic>{};
    if (username != null) data['username'] = username;
    if (bio != null) data['bio'] = bio;
    if (avatarUrl != null) data['avatar_url'] = avatarUrl;

    return await _api.put<User>(
      '/users/me',
      data: data,
      fromJson: (json) {
        final userData = json['user'] ?? json;
        return User.fromJson(userData as Map<String, dynamic>);
      },
    );
  }

  /// 搜索用户
  Future<List<User>> searchUsers(String query) async {
    return await _api.get<List<User>>(
      '/users/search',
      queryParameters: {'q': query},
      fromJson: (json) {
        final list = json['users'] as List? ?? json as List;
        return list
            .map((e) => User.fromJson(e as Map<String, dynamic>))
            .toList();
      },
    );
  }
}
