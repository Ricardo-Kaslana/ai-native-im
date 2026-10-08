import '../../../core/network/api_client.dart';
import '../../../core/storage/storage.dart';
import '../models/user.dart';

/// 认证服务
class AuthService {
  static final AuthService instance = AuthService._();
  AuthService._();

  final _api = ApiClient.instance;
  final _storage = SecureStorage.instance;

  User? _currentUser;
  User? get currentUser => _currentUser;

  /// 登录
  Future<User> login({
    required String username,
    required String password,
    bool rememberMe = false,
  }) async {
    final response = await _api.post<Map<String, dynamic>>(
      '/auth/login',
      data: {'username': username, 'password': password},
      fromJson: (json) => json as Map<String, dynamic>,
    );

    await _storage.saveAccessToken(response['access_token']);
    await _storage.saveRefreshToken(response['refresh_token']);

    final user = User.fromJson(response['user']);
    await _storage.saveUserId(user.id);
    _currentUser = user;

    // 保存凭据（如果选择了记住我）
    if (rememberMe) {
      await _storage.saveCredentials(
        username: username,
        password: password,
      );
    } else {
      // 如果取消记住我，清除之前保存的凭据
      await _storage.clearCredentials();
    }

    return user;
  }

  /// 注册
  Future<User> register({
    required String username,
    required String email,
    required String password,
  }) async {
    final response = await _api.post<Map<String, dynamic>>(
      '/auth/register',
      data: {
        'username': username,
        'email': email,
        'password': password,
      },
      fromJson: (json) => json as Map<String, dynamic>,
    );

    await _storage.saveAccessToken(response['access_token']);
    await _storage.saveRefreshToken(response['refresh_token']);
    
    final user = User.fromJson(response['user']);
    await _storage.saveUserId(user.id);
    _currentUser = user;
    
    return user;
  }

  /// 登出
  Future<void> logout() async {
    // 先清理本地状态（最重要）
    await _storage.clearAuth();
    _currentUser = null;

    // 后台调用 logout API（不阻塞）
    _api.postEmpty('/auth/logout').catchError((_) {});
  }

  /// 获取当前用户信息
  Future<User?> getCurrentUser() async {
    try {
      final user = await _api.get<User>(
        '/users/me',
        fromJson: (json) {
          // API 返回 { user: {...} }
          final userData = json['user'] ?? json;
          return User.fromJson(userData as Map<String, dynamic>);
        },
      );
      _currentUser = user;
      return user;
    } catch (_) {
      return null;
    }
  }

  /// 忘记密码 - 发送重置邮件
  Future<void> forgotPassword(String email) async {
    await _api.postEmpty(
      '/auth/password/reset/request',
      data: {'email': email},
    );
  }

  /// 重置密码
  Future<void> resetPassword({
    required String token,
    required String newPassword,
  }) async {
    await _api.postEmpty(
      '/auth/password/reset/confirm',
      data: {'token': token, 'new_password': newPassword},
    );
  }

  /// 修改密码
  Future<void> changePassword({
    required String currentPassword,
    required String newPassword,
  }) async {
    await _api.postEmpty(
      '/auth/password/change',
      data: {
        'current_password': currentPassword,
        'new_password': newPassword,
      },
    );
  }

  /// 检查是否已登录
  Future<bool> isLoggedIn() async {
    final token = await _storage.getAccessToken();
    return token != null && token.isNotEmpty;
  }

  // ========== 记住我功能 ==========

  /// 获取保存的用户名
  Future<String?> getSavedUsername() async {
    return await _storage.getSavedUsername();
  }

  /// 获取保存的密码
  Future<String?> getSavedPassword() async {
    return await _storage.getSavedPassword();
  }

  /// 清除保存的凭据
  Future<void> clearCredentials() async {
    await _storage.clearCredentials();
  }
}
