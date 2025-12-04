import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Authentication state
enum AuthStatus { initial, loading, authenticated, unauthenticated, error }

/// Auth state class
class AuthState {
  final AuthStatus status;
  final String? userId;
  final String? username;
  final String? email;
  final String? errorMessage;
  final String? accessToken;

  const AuthState({
    this.status = AuthStatus.initial,
    this.userId,
    this.username,
    this.email,
    this.errorMessage,
    this.accessToken,
  });

  AuthState copyWith({
    AuthStatus? status,
    String? userId,
    String? username,
    String? email,
    String? errorMessage,
    String? accessToken,
  }) {
    return AuthState(
      status: status ?? this.status,
      userId: userId ?? this.userId,
      username: username ?? this.username,
      email: email ?? this.email,
      errorMessage: errorMessage ?? this.errorMessage,
      accessToken: accessToken ?? this.accessToken,
    );
  }

  bool get isAuthenticated => status == AuthStatus.authenticated;
  bool get isLoading => status == AuthStatus.loading;
}

/// Auth state notifier
class AuthNotifier extends StateNotifier<AuthState> {
  AuthNotifier() : super(const AuthState());

  Future<void> login(String email, String password) async {
    state = state.copyWith(status: AuthStatus.loading, errorMessage: null);

    try {
      // TODO: Implement actual API call
      await Future.delayed(const Duration(seconds: 1));

      // Mock successful login
      state = state.copyWith(
        status: AuthStatus.authenticated,
        userId: 'user_123',
        username: 'testuser',
        email: email,
        accessToken: 'mock_token',
      );
    } catch (e) {
      state = state.copyWith(
        status: AuthStatus.error,
        errorMessage: e.toString(),
      );
    }
  }

  Future<void> register(String username, String email, String password) async {
    state = state.copyWith(status: AuthStatus.loading, errorMessage: null);

    try {
      // TODO: Implement actual API call
      await Future.delayed(const Duration(seconds: 1));

      // Mock successful registration
      state = state.copyWith(
        status: AuthStatus.authenticated,
        userId: 'user_new',
        username: username,
        email: email,
        accessToken: 'mock_token',
      );
    } catch (e) {
      state = state.copyWith(
        status: AuthStatus.error,
        errorMessage: e.toString(),
      );
    }
  }

  void logout() {
    state = const AuthState(status: AuthStatus.unauthenticated);
  }

  void clearError() {
    state = state.copyWith(errorMessage: null);
  }
}

/// Auth provider
final authProvider = StateNotifierProvider<AuthNotifier, AuthState>(
  (ref) => AuthNotifier(),
);
