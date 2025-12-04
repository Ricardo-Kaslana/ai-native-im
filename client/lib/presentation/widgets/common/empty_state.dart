import 'package:flutter/material.dart';
import '../../../core/constants/spacing.dart';
import 'app_button.dart';

/// Empty state widget for lists and screens
class EmptyState extends StatelessWidget {
  final IconData icon;
  final String title;
  final String? description;
  final String? actionText;
  final VoidCallback? onAction;

  const EmptyState({
    super.key,
    required this.icon,
    required this.title,
    this.description,
    this.actionText,
    this.onAction,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Center(
      child: Padding(
        padding: const EdgeInsets.all(Spacing.xl),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              width: 80,
              height: 80,
              decoration: BoxDecoration(
                color: theme.colorScheme.primary.withOpacity(0.1),
                shape: BoxShape.circle,
              ),
              child: Icon(
                icon,
                size: 40,
                color: theme.colorScheme.primary,
              ),
            ),
            const SizedBox(height: Spacing.lg),
            Text(
              title,
              style: theme.textTheme.titleLarge,
              textAlign: TextAlign.center,
            ),
            if (description != null) ...[
              const SizedBox(height: Spacing.sm),
              Text(
                description!,
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: theme.textTheme.bodySmall?.color,
                ),
                textAlign: TextAlign.center,
              ),
            ],
            if (actionText != null && onAction != null) ...[
              const SizedBox(height: Spacing.lg),
              AppButton(
                text: actionText!,
                onPressed: onAction,
              ),
            ],
          ],
        ),
      ),
    );
  }

  /// Factory for no conversations state
  factory EmptyState.noConversations({VoidCallback? onStartChat}) {
    return EmptyState(
      icon: Icons.chat_bubble_outline,
      title: '暂无会话',
      description: '开始一段新的对话吧',
      actionText: '发起聊天',
      onAction: onStartChat,
    );
  }

  /// Factory for no messages state
  factory EmptyState.noMessages() {
    return const EmptyState(
      icon: Icons.message_outlined,
      title: '暂无消息',
      description: '发送第一条消息开始聊天',
    );
  }

  /// Factory for no contacts state
  factory EmptyState.noContacts({VoidCallback? onAddContact}) {
    return EmptyState(
      icon: Icons.people_outline,
      title: '暂无联系人',
      description: '添加好友开始聊天',
      actionText: '添加联系人',
      onAction: onAddContact,
    );
  }

  /// Factory for no search results state
  factory EmptyState.noSearchResults() {
    return const EmptyState(
      icon: Icons.search_off,
      title: '未找到结果',
      description: '尝试使用其他关键词搜索',
    );
  }

  /// Factory for offline state
  factory EmptyState.offline({VoidCallback? onRetry}) {
    return EmptyState(
      icon: Icons.wifi_off,
      title: '网络连接失败',
      description: '请检查网络设置后重试',
      actionText: '重试',
      onAction: onRetry,
    );
  }
}
