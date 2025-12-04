import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/constants/colors.dart';
import '../../../core/constants/spacing.dart';
import '../../../data/models/group.dart';
import '../../providers/group_provider.dart';
import '../../widgets/common/avatar.dart';
import '../ai_agent/ai_agent_screen.dart';

/// Group info screen
class GroupInfoScreen extends ConsumerWidget {
  final Group group;

  const GroupInfoScreen({super.key, required this.group});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final membersAsync = ref.watch(groupMembersProvider(group.id));
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('群组信息'),
        actions: [
          IconButton(
            icon: const Icon(Icons.more_vert),
            onPressed: () => _showOptionsMenu(context),
          ),
        ],
      ),
      body: SingleChildScrollView(
        child: Column(
          children: [
            // Group header
            Container(
              padding: const EdgeInsets.all(Spacing.lg),
              child: Column(
                children: [
                  AppAvatar(
                    imageUrl: group.avatarUrl,
                    name: group.name,
                    size: 80,
                    isGroup: true,
                  ),
                  const SizedBox(height: Spacing.md),
                  Text(
                    group.name,
                    style: theme.textTheme.displaySmall,
                  ),
                  if (group.description != null) ...[
                    const SizedBox(height: Spacing.xs),
                    Text(
                      group.description!,
                      style: theme.textTheme.bodySmall,
                      textAlign: TextAlign.center,
                    ),
                  ],
                  const SizedBox(height: Spacing.sm),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(
                        group.isPublic ? Icons.public : Icons.lock,
                        size: 16,
                        color: theme.textTheme.bodySmall?.color,
                      ),
                      const SizedBox(width: Spacing.xxs),
                      Text(
                        group.isPublic ? '公开群组' : '私密群组',
                        style: theme.textTheme.bodySmall,
                      ),
                      const SizedBox(width: Spacing.md),
                      Icon(
                        Icons.people,
                        size: 16,
                        color: theme.textTheme.bodySmall?.color,
                      ),
                      const SizedBox(width: Spacing.xxs),
                      Text(
                        '${group.memberCount} 人',
                        style: theme.textTheme.bodySmall,
                      ),
                    ],
                  ),
                ],
              ),
            ),

            const Divider(),

            // Invite code
            if (group.inviteCode != null)
              ListTile(
                leading: const Icon(Icons.link),
                title: const Text('邀请码'),
                subtitle: Text(group.inviteCode!),
                trailing: IconButton(
                  icon: const Icon(Icons.copy),
                  onPressed: () {
                    Clipboard.setData(ClipboardData(text: group.inviteCode!));
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(content: Text('邀请码已复制')),
                    );
                  },
                ),
              ),

            // Members section
            Padding(
              padding: const EdgeInsets.all(Spacing.md),
              child: Row(
                children: [
                  Text(
                    '群成员',
                    style: theme.textTheme.labelMedium?.copyWith(
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                  const Spacer(),
                  TextButton.icon(
                    icon: const Icon(Icons.person_add, size: 18),
                    label: const Text('邀请'),
                    onPressed: () {
                      // TODO: Invite members
                    },
                  ),
                ],
              ),
            ),

            // Members list
            membersAsync.when(
              data: (members) => ListView.builder(
                shrinkWrap: true,
                physics: const NeverScrollableScrollPhysics(),
                itemCount: members.length,
                itemBuilder: (context, index) {
                  final member = members[index];
                  return ListTile(
                    leading: Stack(
                      children: [
                        AppAvatar(
                          imageUrl: member.user.avatarUrl,
                          name: member.user.username,
                          size: 40,
                        ),
                        if (member.user.isOnline)
                          Positioned(
                            right: 0,
                            bottom: 0,
                            child: Container(
                              width: 12,
                              height: 12,
                              decoration: BoxDecoration(
                                color: AppColors.online,
                                shape: BoxShape.circle,
                                border: Border.all(
                                  color: theme.scaffoldBackgroundColor,
                                  width: 2,
                                ),
                              ),
                            ),
                          ),
                      ],
                    ),
                    title: Row(
                      children: [
                        Text(member.user.username),
                        if (member.role != GroupRole.member) ...[
                          const SizedBox(width: Spacing.xs),
                          Container(
                            padding: const EdgeInsets.symmetric(
                              horizontal: Spacing.xs,
                              vertical: 2,
                            ),
                            decoration: BoxDecoration(
                              color: member.isOwner
                                  ? Colors.orange.withOpacity(0.2)
                                  : theme.colorScheme.primaryContainer,
                              borderRadius: BorderRadius.circular(4),
                            ),
                            child: Text(
                              member.isOwner ? '群主' : '管理员',
                              style: theme.textTheme.labelSmall?.copyWith(
                                color: member.isOwner
                                    ? Colors.orange
                                    : theme.colorScheme.primary,
                              ),
                            ),
                          ),
                        ],
                      ],
                    ),
                    subtitle: Text(
                      member.user.isOnline ? '在线' : '离线',
                      style: TextStyle(
                        color: member.user.isOnline ? AppColors.online : null,
                      ),
                    ),
                    onTap: () {
                      // TODO: Show member options
                    },
                  );
                },
              ),
              loading: () => const Padding(
                padding: EdgeInsets.all(Spacing.lg),
                child: Center(child: CircularProgressIndicator()),
              ),
              error: (e, _) => Padding(
                padding: const EdgeInsets.all(Spacing.lg),
                child: Center(child: Text('加载失败: $e')),
              ),
            ),

            const Divider(),

            // AI Agent
            ListTile(
              leading: Icon(
                Icons.smart_toy,
                color: theme.colorScheme.primary,
              ),
              title: const Text('AI 助手'),
              subtitle: const Text('管理群组 AI 助手'),
              trailing: const Icon(Icons.chevron_right),
              onTap: () {
                Navigator.push(
                  context,
                  MaterialPageRoute(
                    builder: (_) => AIAgentScreen(
                      groupId: group.id,
                      groupName: group.name,
                    ),
                  ),
                );
              },
            ),

            const Divider(),

            // Actions
            ListTile(
              leading: const Icon(Icons.notifications),
              title: const Text('消息免打扰'),
              trailing: Switch(
                value: false,
                onChanged: (value) {
                  // TODO: Toggle mute
                },
              ),
            ),
            ListTile(
              leading: const Icon(Icons.cleaning_services),
              title: const Text('清空聊天记录'),
              onTap: () => _showClearHistoryDialog(context),
            ),
            ListTile(
              leading: Icon(Icons.exit_to_app, color: theme.colorScheme.error),
              title: Text(
                '退出群组',
                style: TextStyle(color: theme.colorScheme.error),
              ),
              onTap: () => _showLeaveDialog(context),
            ),

            const SizedBox(height: Spacing.xl),
          ],
        ),
      ),
    );
  }

  void _showOptionsMenu(BuildContext context) {
    showModalBottomSheet(
      context: context,
      builder: (context) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(Icons.edit),
              title: const Text('编辑群组信息'),
              onTap: () {
                Navigator.pop(context);
                // TODO: Edit group
              },
            ),
            ListTile(
              leading: const Icon(Icons.qr_code),
              title: const Text('群二维码'),
              onTap: () {
                Navigator.pop(context);
                // TODO: Show QR code
              },
            ),
            ListTile(
              leading: const Icon(Icons.search),
              title: const Text('搜索聊天记录'),
              onTap: () {
                Navigator.pop(context);
                // TODO: Search messages
              },
            ),
          ],
        ),
      ),
    );
  }

  void _showClearHistoryDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('清空聊天记录'),
        content: const Text('确定要清空所有聊天记录吗？此操作不可恢复。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () {
              Navigator.pop(context);
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('聊天记录已清空')),
              );
            },
            child: Text(
              '清空',
              style: TextStyle(color: Theme.of(context).colorScheme.error),
            ),
          ),
        ],
      ),
    );
  }

  void _showLeaveDialog(BuildContext context) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('退出群组'),
        content: const Text('确定要退出该群组吗？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () {
              Navigator.pop(context);
              Navigator.pop(context);
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('已退出群组')),
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
