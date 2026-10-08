import 'dart:async';
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import '../models/conversation.dart';
import '../services/conversation_service.dart';
import '../../shared/widgets/user_avatar.dart';
import '../../chat/services/chat_service_v2.dart';
import '../../../core/storage/storage.dart';
import '../../../core/services/unread_count_service.dart';

/// 对话列表页面
class ConversationListPage extends StatefulWidget {
  const ConversationListPage({super.key});

  @override
  State<ConversationListPage> createState() => _ConversationListPageState();
}

class _ConversationListPageState extends State<ConversationListPage> {
  final _service = ConversationService.instance;
  final _chatService = ChatServiceV2.instance;
  final _unreadService = UnreadCountService.instance;
  List<Conversation> _conversations = [];
  bool _isLoading = true;
  String? _error;
  StreamSubscription<int>? _unreadCountSubscription;

  @override
  void initState() {
    super.initState();

    _loadConversations();

    // 监听未读计数变化（当用户从聊天页面返回时，自动刷新会话列表）
    _unreadCountSubscription = _unreadService.unreadCountStream.listen((_) {
      if (mounted) {
        debugPrint('ConversationListPage: unreadCountStream triggered, reloading conversations');
        _loadConversations();
      }
    });

    // 注意：不需要监听 newMessageStream，因为 ChatServiceV2 已经在 _handleIncomingMessage
    // 中调用了 incrementConversationUnread，会触发 unreadCountStream 并自动刷新列表
  }

  @override
  void dispose() {
    _unreadCountSubscription?.cancel();
    super.dispose();
  }

  /// 标记会话为已读
  void _markConversationAsRead(String conversationId) {
    // ChatServiceV2.markAsRead() 会调用 _unreadService.markConversationAsRead()
    _chatService.markAsRead(conversationId);
  }

  Future<void> _loadConversations() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    try {
      final conversations = await _service.getConversations();
      if (mounted) {
        setState(() {
          // 合并全局服务中的未读计数到会话数据中
          _conversations = conversations.map((conv) {
            final localUnread = _unreadService.getConversationUnreadCount(conv.id);
            debugPrint('ConversationListPage: ${conv.name} (${conv.id}) - localUnread: $localUnread, server unread: ${conv.unreadCount}');
            // 始终使用全局服务的未读计数（即使为0）
            return Conversation(
              id: conv.id,
              type: conv.type,
              name: conv.name,
              avatarUrl: conv.avatarUrl,
              lastMessage: conv.lastMessage,
              lastMessageAt: conv.lastMessageAt,
              unreadCount: localUnread, // 使用全局服务的未读计数
              createdAt: conv.createdAt,
              updatedAt: conv.updatedAt,
              memberIds: conv.memberIds,
              participants: conv.participants,
            );
          }).toList();
          _isLoading = false;
        });
        debugPrint('ConversationListPage: Loaded ${_conversations.length} conversations');
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _error = e.toString();
          _isLoading = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('消息'),
        actions: [
          IconButton(
            icon: const Icon(Icons.search),
            onPressed: () => context.push('/search'),
          ),
          IconButton(
            icon: const Icon(Icons.add),
            onPressed: () => _showNewChatOptions(),
          ),
        ],
      ),
      body: _buildBody(theme),
    );
  }

  Widget _buildBody(ThemeData theme) {
    if (_isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_error != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.error_outline, size: 48, color: Colors.grey),
            const SizedBox(height: 16),
            Text(_error!),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: _loadConversations,
              child: const Text('重试'),
            ),
          ],
        ),
      );
    }

    if (_conversations.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.chat_bubble_outline,
              size: 64,
              color: theme.colorScheme.onSurface.withValues(alpha: 0.3),
            ),
            const SizedBox(height: 16),
            Text(
              '暂无消息',
              style: theme.textTheme.titleMedium?.copyWith(
                color: theme.colorScheme.onSurface.withValues(alpha: 0.5),
              ),
            ),
            const SizedBox(height: 8),
            Text(
              '开始一段新对话吧',
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onSurface.withValues(alpha: 0.4),
              ),
            ),
          ],
        ),
      );
    }

    return RefreshIndicator(
      onRefresh: _loadConversations,
      child: ListView.builder(
        itemCount: _conversations.length,
        itemBuilder: (context, index) {
          final conversation = _conversations[index];
          return _buildConversationTile(conversation, theme);
        },
      ),
    );
  }

  Widget _buildConversationTile(Conversation conversation, ThemeData theme) {
    return ListTile(
      leading: Stack(
        clipBehavior: Clip.none,
        children: [
          UserAvatar(
            name: conversation.displayName,
            avatarUrl: conversation.avatarUrl,
            size: 48,
          ),
          if (conversation.unreadCount > 0)
            Positioned(
              right: -4,
              top: -4,
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                decoration: BoxDecoration(
                  color: Colors.red,
                  borderRadius: BorderRadius.circular(10),
                ),
                constraints: const BoxConstraints(
                  minWidth: 18,
                  minHeight: 18,
                ),
                child: Text(
                  conversation.unreadCount > 99
                      ? '99+'
                      : conversation.unreadCount.toString(),
                  style: const TextStyle(
                    color: Colors.white,
                    fontSize: 10,
                    fontWeight: FontWeight.bold,
                  ),
                  textAlign: TextAlign.center,
                ),
              ),
            ),
        ],
      ),
      title: Row(
        children: [
          Expanded(
            child: Text(
              conversation.displayName,
              style: theme.textTheme.titleMedium?.copyWith(
                fontWeight: conversation.unreadCount > 0
                    ? FontWeight.bold
                    : FontWeight.normal,
              ),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
          ),
          if (conversation.updatedAt != null)
            Text(
              _formatTime(conversation.updatedAt!),
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.onSurface.withValues(alpha: 0.5),
              ),
            ),
        ],
      ),
      subtitle: Text(
        conversation.lastMessage ?? '',
        style: theme.textTheme.bodyMedium?.copyWith(
          color: theme.colorScheme.onSurface.withValues(alpha: 0.6),
        ),
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
      onTap: () {
        // 标记会话为已读
        _markConversationAsRead(conversation.id);

        // 对于私聊，预加载会话信息（避免首次发送消息时的额外API调用）
        if (!conversation.isGroup && conversation.participants != null) {
          final currentUserId = SecureStorage.instance.getUserIdSync();
          // 找到对方用户的ID
          final otherParticipant = conversation.participants!.firstWhere(
            (p) => p.id != currentUserId,
            orElse: () => conversation.participants!.first,
          );
          ChatServiceV2.instance.preloadPrivateConversationInfo(
            conversationId: conversation.id,
            receiverId: otherParticipant.id,
          );
        }
        context.push(
          '/chat/${conversation.id}?name=${Uri.encodeComponent(conversation.displayName)}&isGroup=${conversation.isGroup}',
        );
      },
    );
  }

  String _formatTime(DateTime time) {
    final now = DateTime.now();
    final diff = now.difference(time);

    if (diff.inDays == 0) {
      return '${time.hour.toString().padLeft(2, '0')}:${time.minute.toString().padLeft(2, '0')}';
    } else if (diff.inDays == 1) {
      return '昨天';
    } else if (diff.inDays < 7) {
      const weekdays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];
      return weekdays[time.weekday % 7];
    } else {
      return '${time.month}/${time.day}';
    }
  }

  void _showNewChatOptions() {
    showModalBottomSheet(
      context: context,
      builder: (context) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(Icons.chat),
              title: const Text('发起私聊'),
              onTap: () {
                Navigator.pop(context);
                context.push('/contacts');
              },
            ),
            ListTile(
              leading: const Icon(Icons.group_add),
              title: const Text('创建群聊'),
              onTap: () {
                Navigator.pop(context);
                context.push('/group/create');
              },
            ),
          ],
        ),
      ),
    );
  }
}
