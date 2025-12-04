import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/constants/spacing.dart';
import '../../providers/theme_provider.dart';

/// Settings screen
class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeMode = ref.watch(themeModeProvider);
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('设置'),
      ),
      body: ListView(
        children: [
          // Appearance section
          _buildSectionHeader(context, '外观'),
          ListTile(
            leading: const Icon(Icons.dark_mode),
            title: const Text('深色模式'),
            trailing: DropdownButton<ThemeMode>(
              value: themeMode,
              underline: const SizedBox(),
              items: const [
                DropdownMenuItem(
                  value: ThemeMode.system,
                  child: Text('跟随系统'),
                ),
                DropdownMenuItem(
                  value: ThemeMode.light,
                  child: Text('浅色'),
                ),
                DropdownMenuItem(
                  value: ThemeMode.dark,
                  child: Text('深色'),
                ),
              ],
              onChanged: (mode) {
                if (mode != null) {
                  ref.read(themeModeProvider.notifier).setThemeMode(mode);
                }
              },
            ),
          ),

          const Divider(),

          // Notifications section
          _buildSectionHeader(context, '通知'),
          SwitchListTile(
            secondary: const Icon(Icons.notifications),
            title: const Text('消息通知'),
            subtitle: const Text('接收新消息推送'),
            value: true,
            onChanged: (value) {
              // TODO: Toggle notifications
            },
          ),
          SwitchListTile(
            secondary: const Icon(Icons.volume_up),
            title: const Text('消息声音'),
            subtitle: const Text('收到消息时播放提示音'),
            value: true,
            onChanged: (value) {
              // TODO: Toggle sound
            },
          ),
          SwitchListTile(
            secondary: const Icon(Icons.vibration),
            title: const Text('振动'),
            subtitle: const Text('收到消息时振动'),
            value: true,
            onChanged: (value) {
              // TODO: Toggle vibration
            },
          ),

          const Divider(),

          // Privacy section
          _buildSectionHeader(context, '隐私'),
          ListTile(
            leading: const Icon(Icons.lock),
            title: const Text('修改密码'),
            trailing: const Icon(Icons.chevron_right),
            onTap: () {
              // TODO: Change password
            },
          ),
          SwitchListTile(
            secondary: const Icon(Icons.visibility),
            title: const Text('显示在线状态'),
            subtitle: const Text('让其他人看到你的在线状态'),
            value: true,
            onChanged: (value) {
              // TODO: Toggle online status visibility
            },
          ),
          SwitchListTile(
            secondary: const Icon(Icons.done_all),
            title: const Text('已读回执'),
            subtitle: const Text('让对方知道你已读消息'),
            value: true,
            onChanged: (value) {
              // TODO: Toggle read receipts
            },
          ),

          const Divider(),

          // Storage section
          _buildSectionHeader(context, '存储'),
          ListTile(
            leading: const Icon(Icons.storage),
            title: const Text('清除缓存'),
            subtitle: const Text('释放存储空间'),
            trailing: const Icon(Icons.chevron_right),
            onTap: () {
              _showClearCacheDialog(context);
            },
          ),
          ListTile(
            leading: const Icon(Icons.download),
            title: const Text('自动下载'),
            subtitle: const Text('Wi-Fi 下自动下载媒体'),
            trailing: const Icon(Icons.chevron_right),
            onTap: () {
              // TODO: Auto download settings
            },
          ),

          const Divider(),

          // About section
          _buildSectionHeader(context, '关于'),
          ListTile(
            leading: const Icon(Icons.info),
            title: const Text('版本'),
            trailing: Text(
              '1.0.0',
              style: theme.textTheme.bodySmall,
            ),
          ),
          ListTile(
            leading: const Icon(Icons.description),
            title: const Text('用户协议'),
            trailing: const Icon(Icons.chevron_right),
            onTap: () {
              // TODO: Show terms
            },
          ),
          ListTile(
            leading: const Icon(Icons.privacy_tip),
            title: const Text('隐私政策'),
            trailing: const Icon(Icons.chevron_right),
            onTap: () {
              // TODO: Show privacy policy
            },
          ),

          const SizedBox(height: Spacing.xl),
        ],
      ),
    );
  }

  Widget _buildSectionHeader(BuildContext context, String title) {
    final theme = Theme.of(context);
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        Spacing.md,
        Spacing.md,
        Spacing.md,
        Spacing.xs,
      ),
      child: Text(
        title,
        style: theme.textTheme.labelMedium?.copyWith(
          color: theme.colorScheme.primary,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }

  void _showClearCacheDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('清除缓存'),
        content: const Text('确定要清除所有缓存数据吗？这不会删除你的聊天记录。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () {
              Navigator.of(context).pop();
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('缓存已清除')),
              );
            },
            child: const Text('清除'),
          ),
        ],
      ),
    );
  }
}
