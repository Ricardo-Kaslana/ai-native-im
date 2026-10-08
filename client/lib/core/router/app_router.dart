import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import '../auth/auth_provider.dart';

// 页面导入
import '../../features/auth/pages/login_page.dart';
import '../../features/auth/pages/register_page.dart';
import '../../features/auth/pages/forgot_password_page.dart';
import '../../features/main/pages/main_page.dart';
import '../../features/chat/pages/chat_page.dart';
import '../../features/profile/pages/profile_page.dart';
import '../../features/profile/pages/edit_profile_page.dart';
import '../../features/settings/pages/settings_page.dart';
import '../../features/groups/pages/create_group_page.dart';
import '../../features/groups/pages/group_detail_page.dart';
import '../../features/groups/pages/join_group_page.dart';
import '../../features/search/pages/search_page.dart';
import '../../features/contacts/pages/add_contact_page.dart';
import '../../features/contacts/pages/contacts_page.dart';

/// 应用路由配置
class AppRouter {
  static final _rootNavigatorKey = GlobalKey<NavigatorState>();

  static GoRouter createRouter(AuthController authController) {
    return GoRouter(
      navigatorKey: _rootNavigatorKey,
      initialLocation: '/login',
      refreshListenable: authController,  // 监听认证状态变化
      redirect: (context, state) => _redirect(state, authController),
      routes: [
        // 认证页面
        GoRoute(
          path: '/login',
          name: 'login',
          builder: (context, state) => const LoginPage(),
        ),
        GoRoute(
          path: '/register',
          name: 'register',
          builder: (context, state) => const RegisterPage(),
        ),
        GoRoute(
          path: '/forgot-password',
          name: 'forgot-password',
          builder: (context, state) => const ForgotPasswordPage(),
        ),

        // 主页
        GoRoute(
          path: '/',
          name: 'main',
          builder: (context, state) => const MainPage(),
          routes: [
            // 聊天页面
            GoRoute(
              path: 'chat/:conversationId',
              name: 'chat',
              builder: (context, state) {
                final conversationId = state.pathParameters['conversationId']!;
                final title = state.uri.queryParameters['name'] ?? '';
                final isGroup = state.uri.queryParameters['isGroup'] == 'true';
                return ChatPage(
                  conversationId: conversationId,
                  title: title,
                  isGroup: isGroup,
                );
              },
            ),

            // 个人资料
            GoRoute(
              path: 'profile',
              name: 'profile',
              builder: (context, state) => const ProfilePage(),
            ),
            GoRoute(
              path: 'profile/edit',
              name: 'edit-profile',
              builder: (context, state) => const EditProfilePage(),
            ),

            // 设置
            GoRoute(
              path: 'settings',
              name: 'settings',
              builder: (context, state) => const SettingsPage(),
            ),

            // 群组
            GoRoute(
              path: 'group/create',
              name: 'create-group',
              builder: (context, state) => const CreateGroupPage(),
            ),
            GoRoute(
              path: 'group/join',
              name: 'join-group',
              builder: (context, state) => const JoinGroupPage(),
            ),
            GoRoute(
              path: 'group/:groupId',
              name: 'group-detail',
              builder: (context, state) {
                final groupId = state.pathParameters['groupId']!;
                return GroupDetailPage(groupId: groupId);
              },
            ),

            // 搜索
            GoRoute(
              path: 'search',
              name: 'search',
              builder: (context, state) => const SearchPage(),
            ),

            // 添加联系人
            GoRoute(
              path: 'contacts/add',
              name: 'add-contact',
              builder: (context, state) => const AddContactPage(),
            ),
            // 通讯录
            GoRoute(
              path: 'contacts',
              name: 'contacts',
              builder: (context, state) => const ContactsPage(),
            ),
          ],
        ),
      ],
    );
  }

  /// 同步的重定向逻辑
  static String? _redirect(GoRouterState state, AuthController authController) {
    final isAuthRoute = state.matchedLocation == '/login' ||
        state.matchedLocation == '/register' ||
        state.matchedLocation == '/forgot-password';

    // 如果认证状态未知，不重定向（等待初始化完成）
    if (authController.isUnknown) {
      return null;
    }

    // 未登录且不是认证页面 -> 重定向到登录页
    if (!authController.isAuthenticated && !isAuthRoute) {
      return '/login';
    }

    // 已登录且是认证页面 -> 重定向到主页
    if (authController.isAuthenticated && isAuthRoute) {
      return '/';
    }

    return null;
  }
}
