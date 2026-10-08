import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'core/theme/app_theme.dart';
import 'core/router/app_router.dart';
import 'core/network/api_client.dart';
import 'core/auth/auth_provider.dart';

/// 应用程序入口
class AiimApp extends ConsumerStatefulWidget {
  const AiimApp({super.key});

  @override
  ConsumerState<AiimApp> createState() => _AiimAppState();
}

class _AiimAppState extends ConsumerState<AiimApp> {
  late GoRouter _router;
  late AuthController _authController;

  @override
  void initState() {
    super.initState();
    _authController = ref.read(authControllerProvider);
    _router = AppRouter.createRouter(_authController);
    _setupApiClient();
  }

  void _setupApiClient() {
    ApiClient.instance.onLogout = () async {
      await _authController.logout();
    };
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(
      title: 'AIIM',
      debugShowCheckedModeBanner: false,
      theme: AppTheme.light,
      darkTheme: AppTheme.dark,
      themeMode: ThemeMode.system,
      routerConfig: _router,
    );
  }
}
