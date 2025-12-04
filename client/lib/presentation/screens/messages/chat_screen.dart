import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/constants/spacing.dart';
import '../../../data/models/conversation.dart';
import '../../providers/message_provider.dart';
import '../../widgets/common/avatar.dart';
import '../../widgets/message/message_bubble.dart';
import '../../widgets/message/chat_input.dart';
import '../../widgets/message/typing_indicator.dart';

/// Chat screen
class ChatScreen extends ConsumerStatefulWidget {
  final Conversation conversation;

  const ChatScreen({
    super.key,
    required this.conversation,
  });

  @override
  ConsumerState<ChatScreen> createState() => _ChatScreenState();
}

class _ChatScreenState extends ConsumerState<ChatScreen> {
  final _scrollController = ScrollController();

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final messagesAsync = ref.watch(messagesProvider(widget.conversation.id));
    final isTyping = ref.watch(typingIndicatorProvider(widget.conversation.id));
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        titleSpacing: 0,
        title: Row(
          children: [
            AppAvatar(
              imageUrl: widget.conversation.displayAvatar,
              name: widget.conversation.displayName,
              size: 36,
              isGroup: widget.conversation.type == ConversationType.group,
              isAI: widget.conversation.type == ConversationType.aiAgent,
            ),
            const SizedBox(width: Spacing.sm),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    widget.conversation.displayName,
                    style: theme.textTheme.bodyMedium?.copyWith(
                      fontWeight: FontWeight.w600,
                    ),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                  if (widget.conversation.type == ConversationType.private)
                    Text(
                      widget.conversation.isOnline ? '在线' : '离线',
                      style: theme.textTheme.labelSmall?.copyWith(
                        color: widget.conversation.isOnline
                            ? Colors.green
                            : theme.textTheme.bodySmall?.color,
                      ),
                    ),
                ],
              ),
            ),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.call),
            onPressed: () {
              // TODO: Voice call
            },
          ),
          IconButton(
            icon: const Icon(Icons.more_vert),
            onPressed: () {
              // TODO: Show chat options
            },
          ),
        ],
      ),
      body: Column(
        children: [
          // Messages list
          Expanded(
            child: messagesAsync.when(
              data: (messages) {
                if (messages.isEmpty) {
                  return _buildEmptyState(context);
                }
                return ListView.builder(
                  controller: _scrollController,
                  reverse: true,
                  padding: const EdgeInsets.symmetric(vertical: Spacing.sm),
                  itemCount: messages.length + (isTyping ? 1 : 0),
                  itemBuilder: (context, index) {
                    if (isTyping && index == 0) {
                      return const TypingIndicator();
                    }
                    final messageIndex = isTyping ? index - 1 : index;
                    final message = messages[messageIndex];
                    final isSent = message.senderId == 'me';

                    // Check if we should show date separator
                    bool showDateSeparator = false;
                    if (messageIndex < messages.length - 1) {
                      final nextMessage = messages[messageIndex + 1];
                      showDateSeparator = !_isSameDay(
                        message.createdAt,
                        nextMessage.createdAt,
                      );
                    } else {
                      showDateSeparator = true;
                    }

                    return Column(
                      children: [
                        if (showDateSeparator)
                          _buildDateSeparator(context, message.createdAt),
                        MessageBubble(
                          message: message,
                          isSent: isSent,
                          showAvatar: !isSent &&
                              widget.conversation.type != ConversationType.private,
                        ),
                      ],
                    );
                  },
                );
              },
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (error, stack) => Center(
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Icon(
                      Icons.error_outline,
                      size: 48,
                      color: theme.colorScheme.error,
                    ),
                    const SizedBox(height: Spacing.md),
                    const Text('加载消息失败'),
                    const SizedBox(height: Spacing.md),
                    ElevatedButton(
                      onPressed: () => ref.invalidate(
                        messagesProvider(widget.conversation.id),
                      ),
                      child: const Text('重试'),
                    ),
                  ],
                ),
              ),
            ),
          ),

          // Chat input
          ChatInput(
            onSend: (text) {
              ref.read(messagesProvider(widget.conversation.id).notifier)
                  .sendMessage(text);
              // Scroll to bottom
              _scrollController.animateTo(
                0,
                duration: const Duration(milliseconds: 300),
                curve: Curves.easeOut,
              );
            },
            onMediaTap: () {
              // TODO: Open media picker
            },
          ),
        ],
      ),
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    final theme = Theme.of(context);
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          AppAvatar(
            imageUrl: widget.conversation.displayAvatar,
            name: widget.conversation.displayName,
            size: 80,
            isGroup: widget.conversation.type == ConversationType.group,
            isAI: widget.conversation.type == ConversationType.aiAgent,
          ),
          const SizedBox(height: Spacing.lg),
          Text(
            widget.conversation.displayName,
            style: theme.textTheme.displaySmall,
          ),
          const SizedBox(height: Spacing.xs),
          Text(
            '开始聊天吧',
            style: theme.textTheme.bodySmall,
          ),
        ],
      ),
    );
  }

  Widget _buildDateSeparator(BuildContext context, DateTime date) {
    final theme = Theme.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: Spacing.md),
      child: Center(
        child: Container(
          padding: const EdgeInsets.symmetric(
            horizontal: Spacing.sm,
            vertical: Spacing.xxs,
          ),
          decoration: BoxDecoration(
            color: theme.colorScheme.surface,
            borderRadius: BorderRadius.circular(Spacing.radiusSm),
          ),
          child: Text(
            _formatDate(date),
            style: theme.textTheme.labelSmall,
          ),
        ),
      ),
    );
  }

  bool _isSameDay(DateTime a, DateTime b) {
    return a.year == b.year && a.month == b.month && a.day == b.day;
  }

  String _formatDate(DateTime date) {
    final now = DateTime.now();
    final diff = now.difference(date);

    if (_isSameDay(date, now)) {
      return '今天';
    } else if (diff.inDays == 1) {
      return '昨天';
    } else if (diff.inDays < 7) {
      const weekdays = ['周一', '周二', '周三', '周四', '周五', '周六', '周日'];
      return weekdays[date.weekday - 1];
    } else {
      return '${date.year}/${date.month}/${date.day}';
    }
  }
}
