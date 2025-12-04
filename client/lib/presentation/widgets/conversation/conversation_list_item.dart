import 'package:flutter/material.dart';
import '../../../core/constants/colors.dart';
import '../../../core/constants/spacing.dart';
import '../../../data/models/conversation.dart';
import '../common/avatar.dart';
import 'unread_badge.dart';

/// Conversation list item widget
class ConversationListItem extends StatelessWidget {
  final Conversation conversation;
  final VoidCallback? onTap;
  final VoidCallback? onLongPress;

  const ConversationListItem({
    super.key,
    required this.conversation,
    this.onTap,
    this.onLongPress,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;

    return InkWell(
      onTap: onTap,
      onLongPress: onLongPress,
      child: Container(
        padding: const EdgeInsets.symmetric(
          horizontal: Spacing.md,
          vertical: Spacing.sm,
        ),
        decoration: BoxDecoration(
          color: conversation.isPinned
              ? (isDark
                  ? AppColors.darkSurface.withOpacity(0.5)
                  : AppColors.lightSurface.withOpacity(0.5))
              : null,
        ),
        child: Row(
          children: [
            // Avatar with online indicator
            Stack(
              children: [
                AppAvatar(
                  imageUrl: conversation.displayAvatar,
                  name: conversation.displayName,
                  size: Spacing.avatarMd,
                  isGroup: conversation.type == ConversationType.group,
                  isAI: conversation.type == ConversationType.aiAgent,
                ),
                if (conversation.type == ConversationType.private &&
                    conversation.isOnline)
                  Positioned(
                    right: 0,
                    bottom: 0,
                    child: Container(
                      width: 14,
                      height: 14,
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
            const SizedBox(width: Spacing.md),

            // Content
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // Name and time row
                  Row(
                    children: [
                      Expanded(
                        child: Row(
                          children: [
                            Flexible(
                              child: Text(
                                conversation.displayName,
                                style: theme.textTheme.bodyMedium?.copyWith(
                                  fontWeight: conversation.unreadCount > 0
                                      ? FontWeight.w600
                                      : FontWeight.w500,
                                ),
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                              ),
                            ),
                            if (conversation.isMuted) ...[
                              const SizedBox(width: Spacing.xxs),
                              Icon(
                                Icons.notifications_off,
                                size: 14,
                                color: theme.textTheme.bodySmall?.color,
                              ),
                            ],
                          ],
                        ),
                      ),
                      Text(
                        _formatTime(conversation.lastActivityAt),
                        style: theme.textTheme.labelSmall?.copyWith(
                          color: conversation.unreadCount > 0
                              ? theme.colorScheme.primary
                              : null,
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: Spacing.xxs),

                  // Last message and unread badge row
                  Row(
                    children: [
                      Expanded(
                        child: Text(
                          _getLastMessagePreview(),
                          style: theme.textTheme.bodySmall?.copyWith(
                            color: conversation.unreadCount > 0
                                ? theme.textTheme.bodyMedium?.color
                                : null,
                          ),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                      if (conversation.unreadCount > 0)
                        UnreadBadge(count: conversation.unreadCount),
                    ],
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  String _getLastMessagePreview() {
    final lastMessage = conversation.lastMessage;
    if (lastMessage == null) return '';

    // For group chats, show sender name
    if (conversation.type == ConversationType.group &&
        lastMessage.senderName != null) {
      return '${lastMessage.senderName}: ${lastMessage.preview}';
    }

    return lastMessage.preview;
  }

  String _formatTime(DateTime? dateTime) {
    if (dateTime == null) return '';

    final now = DateTime.now();
    final diff = now.difference(dateTime);

    if (diff.inMinutes < 1) {
      return '刚刚';
    } else if (diff.inHours < 1) {
      return '${diff.inMinutes}分钟前';
    } else if (diff.inDays < 1) {
      return '${dateTime.hour.toString().padLeft(2, '0')}:${dateTime.minute.toString().padLeft(2, '0')}';
    } else if (diff.inDays < 7) {
      const weekdays = ['周一', '周二', '周三', '周四', '周五', '周六', '周日'];
      return weekdays[dateTime.weekday - 1];
    } else {
      return '${dateTime.month}/${dateTime.day}';
    }
  }
}
