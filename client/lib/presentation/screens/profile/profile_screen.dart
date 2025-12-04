import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/constants/spacing.dart';
import '../../providers/auth_provider.dart';
import '../../widgets/common/avatar.dart';
import '../auth/login_screen.dart';
import 'settings_screen.dart';

/// Profile screen
class ProfileScreen extends ConsumerWidget {
  const ProfileScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final authState = ref.watch(authProvider);
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('我的'),
        actions: [
          IconButton(
            icon: const Icon(Icons.settings),
            onPressed: () {
              Navigator.of(context).push(
                MaterialPageRoute(builder: (_) => const SettingsScreen()),
              );
            },
          ),
        ],
      ),
      body: SingleChildScrollView(
        child: Column(
          children: [
            // Profile header
            Container(
              padding: const EdgeInsets.all(Spacing.lg),
              child: Column(
                children: [
                  // Avatar
                  GestureDetector(
                    onTap: () {
                      // TODO: Change avatar
                    },
                    child: Stack(
                      children: [
                        AppAvatar(
                          name: authState.username ?? '用户',
                          size: Spacing.avatarXl,
                        ),
                        Positioned(
                          right: 0,
                          bottom: 0,
                          child: Container(
                            padding: const EdgeInsets.all(Spacing.xxs),
                            decoration: BoxDecoration(
                              color: theme.colorScheme.primary,
                              shape: BoxShape.circle,
                            ),
                            child: const Icon(
                              Icons.camera_alt,
                              size: 16,
                              color: Colors.white,
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(height: Spacing.md),
                  // Username
                  Text(
                    authState.username ?? '用户',
                    style: theme.textTheme.displaySmall,
                  ),
                  const SizedBox(height: Spacing.xxs),
                  // Email
                  Text(
                    authState.email ?? '',
                    style: theme.textTheme.bodySmall,
                  ),
                ],
              ),
            ),

            const Divider(),

            // Menu items
            _buildMenuItem(
              context,
              icon: Icons.person_outline,
              title: '编辑资料',
              onTap: () {
                // TODO: Edit profile
              },
            ),
            _buildMenuItem(
              context,
              icon: Icons.qr_code,
              title: '我的二维码',
              onTap: () {
                // TODO: Show QR code
              },
            ),
            _buildMenuItem(
              context,
              icon: Icons.bookmark_outline,
              title: '收藏',
              onTap: () {
                // TODO: Favorites
              },
            ),
            _buildMenuItem(
              context,
              icon: Icons.folder_outlined,
              title: '文件',
              onTap: () {
                // TODO: Files
              },
            ),

            const Divider(),

            _buildMenuItem(
              context,
              icon: Icons.help_outline,
              title: '帮助与反馈',
              onTap: () {
                // TODO: Help
              },
            ),
            _buildMenuItem(
              context,
              icon: Icons.info_outline,
              title: '关于',
              onTap: () {
                _showAboutDialog(context);
              },
            ),

            const Divider(),

            // Logout button
            _buildMenuItem(
              context,
              icon: Icons.logout,
              title: '退出登录',
              isDestructive: true,
              onTap: () {
                _showLogoutDialog(context, ref);
              },
            ),

            const SizedBox(height: Spacing.xl),
          ],
        ),
      ),
    );
  }

  Widget _buildMenuItem(
    BuildContext context, {
    required IconData icon,
    required String title,
    required VoidCallback onTap,
    bool isDestructive = false,
  }) {
    final theme = Theme.of(context);
    final color = isDestructive ? theme.colorScheme.error : null;

    return ListTile(
      leading: Icon(icon, color: color),
      title: Text(
        title,
        style: TextStyle(color: color),
      ),
      trailing: isDestructive ? null : const Icon(Icons.chevron_right),
      onTap: onTap,
    );
  }

  void _showAboutDialog(BuildContext context) {
    showAboutDialog(
      context: context,
      applicationName: 'AI Native IM',
      applicationVersion: '1.0.0',
      applicationIcon: Icon(
        Icons.chat_bubble_rounded,
        size: 48,
        color: Theme.of(context).colorScheme.primary,
      ),
      children: [
        const Text('智能即时通讯应用'),
        const SizedBox(height: Spacing.sm),
        const Text('使用 Flutter 构建'),
      ],
    );
  }

  void _showLogoutDialog(BuildContext context, WidgetRef ref) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('退出登录'),
        content: const Text('确定要退出登录吗？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () {
              ref.read(authProvider.notifier).logout();
              Navigator.of(context).pushAndRemoveUntil(
                MaterialPageRoute(builder: (_) => const LoginScreen()),
                (route) => false,
              );
            },
            child: Text(
              '退出',
              style: TextStyle(color: Theme.of(context).colorScheme.error),
            ),
          ),
        ],
      ),
    );
  }
}
