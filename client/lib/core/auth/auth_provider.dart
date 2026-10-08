import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../storage/storage.dart';

/// 认证状态
enum AuthStatus {
  authenticated,
  unauthenticated,
  unknown,
}

/// 认证控制器 - 继承 ChangeNotifier 以支持 GoRouter 的 refreshListenable
class AuthController extends ChangeNotifier {
  AuthStatus _status = AuthStatus.unknown;
  String? _userId;

  AuthStatus get status => _status;
  String? get userId => _userId;
  bool get isAuthenticated => _status == AuthStatus.authenticated;
  bool get isUnknown => _status == AuthStatus.unknown;
  bool get isUnauthenticated => _status == AuthStatus.unauthenticated;

  AuthController() {
    _initAuthState();
  }

  /// 初始化认证状态
  Future<void> _initAuthState() async {
    await checkAuthStatus();
  }

  /// 检查认证状态
  Future<bool> checkAuthStatus() async {
    final token = await SecureStorage.instance.getAccessToken();
    final userId = await SecureStorage.instance.getUserId();

    _status = (token != null && token.isNotEmpty)
        ? AuthStatus.authenticated
        : AuthStatus.unauthenticated;
    _userId = userId;

    notifyListeners();
    return _status == AuthStatus.authenticated;
  }

  /// 登录成功
  Future<void> loginSuccess(String userId) async {
    _userId = userId;
    _status = AuthStatus.authenticated;
    notifyListeners();
  }

  /// 登出
  Future<void> logout() async {
    await SecureStorage.instance.clearAuth();
    _status = AuthStatus.unauthenticated;
    _userId = null;
    notifyListeners();
  }
}

/// Riverpod Provider - 提供 AuthController
final authControllerProvider = Provider<AuthController>((ref) {
  return AuthController();
});

/// 便捷访问认证状态的 Provider
final authStatusProvider = Provider<AuthStatus>((ref) {
  return ref.watch(authControllerProvider).status;
});

/// 便捷访问用户 ID 的 Provider
final currentUserIdProvider = Provider<String?>((ref) {
  return ref.watch(authControllerProvider).userId;
});
