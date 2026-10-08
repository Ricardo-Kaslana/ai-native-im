import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../../../core/config/app_config.dart';
import '../../../core/storage/storage.dart';
import '../../../core/utils/url_utils.dart';
import '../models/message_models.dart';
import '../services/chat_service_v2.dart';
import '../services/websocket_service_v2.dart';
import '../widgets/emoji_picker.dart';
import '../../shared/widgets/user_avatar.dart';

/// 聊天页面 V2 - 完全对齐后端
class ChatPage extends ConsumerStatefulWidget {
  final String conversationId;
  final String title;
  final bool isGroup;

  const ChatPage({
    super.key,
    required this.conversationId,
    required this.title,
    this.isGroup = false,
  });

  @override
  ConsumerState<ChatPage> createState() => _ChatPageState();
}

class _ChatPageState extends ConsumerState<ChatPage> {
  final _chatService = ChatServiceV2.instance;
  final _messageController = TextEditingController();
  final _scrollController = ScrollController();

  List<ChatMessage> _messages = [];
  bool _isLoading = true;
  String? _error;
  bool _isConnected = false;
  String? _currentUserId;
  StreamSubscription<ChatMessage>? _newMessageSubscription;
  StreamSubscription<ChatMessage>? _messageUpdateSubscription;
  StreamSubscription<WebSocketConnectionState>? _connectionSubscription;

  @override
  void initState() {
    super.initState();
    _initChat();
  }

  Future<void> _initChat() async {
    // 获取当前用户 ID
    _currentUserId = await SecureStorage.instance.getUserId();

    // 先监听连接状态（在初始化之前设置监听，避免错过状态更新）
    _connectionSubscription = _chatService.connectionStateStream.listen((state) {
      if (mounted) {
        setState(() {
          _isConnected = state == WebSocketConnectionState.connected;
        });
      }
    });

    // 获取当前连接状态（确保初始状态正确）
    _isConnected = _chatService.isConnected;

    // 初始化聊天服务（会自动连接 WebSocket）
    if (!_chatService.isConnected) {
      await _chatService.initialize();
      // 初始化后再次更新连接状态
      if (mounted) {
        setState(() {
          _isConnected = _chatService.isConnected;
        });
      }
    }

    // 监听新消息
    _newMessageSubscription = _chatService.newMessageStream.listen((message) {
      if (message.conversationId == widget.conversationId && mounted) {
        setState(() {
          // 避免重复消息
          if (!_messages.any((m) => m.id == message.id)) {
            _messages.add(message);
          }
        });
        _scrollToBottom();
      }
    });

    // 监听消息状态更新（发送状态变化）
    _messageUpdateSubscription = _chatService.messageUpdateStream.listen((message) {
      if (message.conversationId == widget.conversationId && mounted) {
        setState(() {
          final index = _messages.indexWhere((m) => m.id == message.id);
          if (index != -1) {
            _messages[index] = message;
          }
        });
      }
    });

    // 加载历史消息
    _loadMessages();

    // 标记会话为已读（会自动触发 unreadCountStream 更新未读计数）
    _chatService.markAsRead(widget.conversationId);
  }

  @override
  void dispose() {
    _newMessageSubscription?.cancel();
    _messageUpdateSubscription?.cancel();
    _connectionSubscription?.cancel();
    _messageController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  Future<void> _loadMessages() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    try {
      final messages = await _chatService.getMessages(
        widget.conversationId,
        limit: 50,
        forceRefresh: true,
      );
      if (mounted) {
        setState(() {
          _messages = messages;
          _isLoading = false;
        });
        _scrollToBottom();
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

  void _scrollToBottom() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 300),
          curve: Curves.easeOut,
        );
      }
    });
  }

  Future<void> _sendMessage() async {
    final text = _messageController.text.trim();
    if (text.isEmpty) return;

    _messageController.clear();

    // 使用快速发送模式（不等待确认，UI 响应更快）
    try {
      await _chatService.sendTextMessageFast(
        conversationId: widget.conversationId,
        content: text,
        isGroup: widget.isGroup,
      );
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('发送失败: $e')),
        );
        _messageController.text = text;
      }
    }
  }

  /// 重试发送失败的消息
  void _retryMessage(ChatMessage message) {
    _chatService.retryMessage(message);
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.pop(),
        ),
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(widget.title),
            _buildConnectionStatus(theme),
          ],
        ),
        actions: [
          if (widget.isGroup)
            IconButton(
              icon: const Icon(Icons.group),
              onPressed: () => context.push('/group/${widget.conversationId}'),
            ),
          IconButton(
            icon: const Icon(Icons.more_vert),
            onPressed: () {},
          ),
        ],
      ),
      body: Column(
        children: [
          Expanded(child: _buildMessageList(theme)),
          _buildInputBar(theme),
        ],
      ),
    );
  }

  Widget _buildConnectionStatus(ThemeData theme) {
    String statusText;
    Color statusColor;

    switch (_chatService.connectionState) {
      case WebSocketConnectionState.connected:
        statusText = '在线';
        statusColor = Colors.green;
        break;
      case WebSocketConnectionState.connecting:
        statusText = '连接中...';
        statusColor = Colors.orange;
        break;
      case WebSocketConnectionState.reconnecting:
        statusText = '重连中...';
        statusColor = Colors.orange;
        break;
      case WebSocketConnectionState.disconnected:
        statusText = '未连接';
        statusColor = Colors.grey;
        break;
      case WebSocketConnectionState.authFailed:
        statusText = '认证失败';
        statusColor = theme.colorScheme.error;
        break;
      case WebSocketConnectionState.error:
        statusText = '连接错误';
        statusColor = theme.colorScheme.error;
        break;
    }

    return InkWell(
      onTap: () {
        // 显示连接详情
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('状态: $_chatService.connectionState\nURL: ${AppConfig.wsBaseUrl}/ws'),
            duration: const Duration(seconds: 2),
          ),
        );
      },
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            width: 8,
            height: 8,
            decoration: BoxDecoration(
              color: statusColor,
              shape: BoxShape.circle,
            ),
          ),
          const SizedBox(width: 4),
          Text(
            statusText,
            style: theme.textTheme.bodySmall?.copyWith(
              color: statusColor,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildMessageList(ThemeData theme) {
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
              onPressed: _loadMessages,
              child: const Text('重试'),
            ),
          ],
        ),
      );
    }

    if (_messages.isEmpty) {
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
              '暂无消息，打个招呼吧~',
              style: theme.textTheme.bodyLarge?.copyWith(
                color: theme.colorScheme.onSurface.withValues(alpha: 0.5),
              ),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      controller: _scrollController,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      itemCount: _messages.length,
      itemBuilder: (context, index) {
        final message = _messages[index];
        return _buildMessageBubble(message, theme);
      },
    );
  }

  Widget _buildMessageBubble(ChatMessage message, ThemeData theme) {
    final isMe = message.senderId == _currentUserId;
    final isGroup = widget.isGroup;

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: isMe ? MainAxisAlignment.end : MainAxisAlignment.start,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (!isMe) ...[
            UserAvatar(
              name: message.senderUsername ?? message.senderId,
              avatarUrl: message.senderAvatarUrl,
              size: 36,
            ),
            const SizedBox(width: 8),
          ],
          Flexible(
            child: Column(
              crossAxisAlignment: isMe ? CrossAxisAlignment.end : CrossAxisAlignment.start,
              children: [
                // 群聊中显示发送者用户名
                if (isGroup && !isMe && message.senderUsername != null)
                  Padding(
                    padding: const EdgeInsets.only(bottom: 2, left: 14),
                    child: Text(
                      message.senderUsername!,
                      style: theme.textTheme.bodySmall?.copyWith(
                        color: theme.colorScheme.onSurface.withValues(alpha: 0.6),
                      ),
                    ),
                  ),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
                  decoration: BoxDecoration(
                    color: isMe
                        ? theme.colorScheme.primary
                        : theme.colorScheme.surfaceContainerHighest,
                    borderRadius: BorderRadius.only(
                      topLeft: const Radius.circular(16),
                      topRight: const Radius.circular(16),
                      bottomLeft: Radius.circular(isMe ? 16 : 4),
                      bottomRight: Radius.circular(isMe ? 4 : 16),
                    ),
                  ),
                  child: _buildMessageContent(message, theme, isMe),
                ),
                // 发送状态
                if (isMe && message.status != MessageStatus.delivered)
                  _buildMessageStatus(message, theme),
              ],
            ),
          ),
          if (isMe) const SizedBox(width: 8),
        ],
      ),
    );
  }

  Widget _buildMessageContent(ChatMessage message, ThemeData theme, bool isMe) {
    switch (message.msgType) {
      case MsgType.text:
        return Text(
          message.content,
          style: theme.textTheme.bodyMedium?.copyWith(
            color: isMe
                ? theme.colorScheme.onPrimary
                : theme.colorScheme.onSurface,
          ),
        );
      case MsgType.image:
        final imageUrl = UrlUtils.processMediaUrl(message.content);
        return ClipRRect(
          borderRadius: BorderRadius.circular(8),
          child: Image.network(
            imageUrl,
            width: 200,
            fit: BoxFit.cover,
            errorBuilder: (context, error, stackTrace) {
              return Container(
                width: 200,
                height: 150,
                color: theme.colorScheme.surfaceContainerHighest,
                child: const Icon(Icons.broken_image),
              );
            },
          ),
        );
      case MsgType.video:
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Icon(Icons.videocam, size: 48),
            const SizedBox(height: 4),
            Text(
              '视频: ${message.content}',
              style: theme.textTheme.bodySmall?.copyWith(
                color: isMe
                    ? theme.colorScheme.onPrimary.withValues(alpha: 0.7)
                    : theme.colorScheme.onSurface.withValues(alpha: 0.7),
              ),
            ),
          ],
        );
      case MsgType.emoji:
        return Text(
          message.content,
          style: const TextStyle(fontSize: 32),
        );
      case MsgType.system:
        return Text(
          message.content,
          style: theme.textTheme.bodySmall?.copyWith(
            color: theme.colorScheme.onSurface.withValues(alpha: 0.5),
            fontStyle: FontStyle.italic,
          ),
        );
    }
  }

  Widget _buildMessageStatus(ChatMessage message, ThemeData theme) {
    switch (message.status) {
      case MessageStatus.sending:
        return Padding(
          padding: const EdgeInsets.only(top: 4, left: 14),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              SizedBox(
                width: 12,
                height: 12,
                child: CircularProgressIndicator(
                  strokeWidth: 2,
                  color: theme.colorScheme.outline,
                ),
              ),
              const SizedBox(width: 4),
              Text(
                '发送中...',
                style: theme.textTheme.bodySmall?.copyWith(
                  color: theme.colorScheme.outline,
                ),
              ),
            ],
          ),
        );
      case MessageStatus.sent:
        return Padding(
          padding: const EdgeInsets.only(top: 4, left: 14),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                Icons.done,
                size: 14,
                color: theme.colorScheme.outline,
              ),
              const SizedBox(width: 4),
              Text(
                '已发送',
                style: theme.textTheme.bodySmall?.copyWith(
                  color: theme.colorScheme.outline,
                ),
              ),
            ],
          ),
        );
      case MessageStatus.delivered:
        return Padding(
          padding: const EdgeInsets.only(top: 4, left: 14),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                Icons.done_all,
                size: 14,
                color: theme.colorScheme.primary,
              ),
              const SizedBox(width: 4),
              Text(
                '已送达',
                style: theme.textTheme.bodySmall?.copyWith(
                  color: theme.colorScheme.primary,
                ),
              ),
            ],
          ),
        );
      case MessageStatus.failed:
        return Padding(
          padding: const EdgeInsets.only(top: 4, left: 14),
          child: InkWell(
            onTap: () => _retryMessage(message),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(
                  Icons.error_outline,
                  size: 14,
                  color: theme.colorScheme.error,
                ),
                const SizedBox(width: 4),
                Text(
                  '发送失败，点击重试',
                  style: theme.textTheme.bodySmall?.copyWith(
                    color: theme.colorScheme.error,
                  ),
                ),
              ],
            ),
          ),
        );
    }
  }

  Widget _buildInputBar(ThemeData theme) {
    final canSend = _isConnected;

    return Container(
      padding: EdgeInsets.only(
        left: 16,
        right: 8,
        top: 8,
        bottom: MediaQuery.of(context).padding.bottom + 8,
      ),
      decoration: BoxDecoration(
        color: theme.colorScheme.surface,
        border: Border(
          top: BorderSide(
            color: theme.colorScheme.outline.withValues(alpha: 0.2),
          ),
        ),
      ),
      child: Row(
        children: [
          IconButton(
            icon: const Icon(Icons.add_circle_outline),
            onPressed: canSend ? () {} : null,
          ),
          Expanded(
            child: TextField(
              controller: _messageController,
              enabled: canSend,
              decoration: InputDecoration(
                hintText: canSend ? '输入消息...' : '离线状态，无法发送',
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(24),
                  borderSide: BorderSide.none,
                ),
                filled: true,
                fillColor: canSend
                    ? theme.colorScheme.surfaceContainerHighest
                    : theme.colorScheme.surfaceContainerHighest.withValues(alpha: 0.5),
                contentPadding: const EdgeInsets.symmetric(
                  horizontal: 16,
                  vertical: 10,
                ),
              ),
              textInputAction: TextInputAction.send,
              onSubmitted: canSend ? (_) => _sendMessage() : null,
            ),
          ),
          // Emoji 按钮
          IconButton(
            icon: const Icon(Icons.emoji_emotions_outlined),
            onPressed: canSend ? _showEmojiPicker : null,
          ),
          const SizedBox(width: 8),
          IconButton(
            icon: Icon(
              Icons.send,
              color: canSend ? theme.colorScheme.primary : theme.colorScheme.outline,
            ),
            onPressed: canSend ? _sendMessage : null,
          ),
        ],
      ),
    );
  }

  /// 显示 Emoji 选择器
  void _showEmojiPicker() {
    showEmojiPicker(
      context: context,
      onEmojiSelected: (emoji) {
        // 将 emoji 插入到当前光标位置
        final currentValue = _messageController.text;
        final cursorPosition = _messageController.selection.baseOffset;

        // 如果光标位置无效（-1），默认追加到末尾
        final insertPosition = cursorPosition < 0 ? currentValue.length : cursorPosition;

        final newText = currentValue.substring(0, insertPosition) +
            emoji +
            currentValue.substring(insertPosition);
        _messageController.value = TextEditingValue(
          text: newText,
          selection: TextSelection.collapsed(
            offset: insertPosition + emoji.length,
          ),
        );
      },
    );
  }
}
