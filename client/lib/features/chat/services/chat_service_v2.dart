import 'dart:async';
import 'package:flutter/foundation.dart';
import '../../../core/network/api_client.dart';
import '../../../core/storage/storage.dart';
import '../../../core/services/notification_helper.dart';
import '../../../core/services/unread_count_service.dart';
import '../models/message_models.dart';
import 'websocket_service_v2.dart';

/// 聊天服务 V2 - 完全对齐后端
class ChatServiceV2 {
  static final ChatServiceV2 instance = ChatServiceV2._();
  ChatServiceV2._();

  final _api = ApiClient.instance;
  final _ws = WebSocketServiceV2.instance;
  final _storage = SecureStorage.instance;
  final _notificationHelper = NotificationHelper.instance;
  final _unreadService = UnreadCountService.instance;

  // 本地消息缓存（按会话ID）
  final _messageCache = <String, List<ChatMessage>>{};

  // 待发送消息的本地ID到内容的映射（用于匹配服务器返回的消息）
  final _pendingMessages = <String, _PendingMessageInfo>{};

  // 私聊会话信息缓存：conversation_id -> {receiverId, conversationId}
  final _privateConversationCache = <String, _PrivateConversationInfo>{};

  // 用户信息缓存：user_id -> {username, avatarUrl}
  final _userInfoCache = <String, _UserInfo>{};

  // 流控制器
  final _newMessageController = StreamController<ChatMessage>.broadcast();
  final _messageUpdateController = StreamController<ChatMessage>.broadcast();

  // 流
  Stream<ChatMessage> get newMessageStream => _newMessageController.stream;
  Stream<ChatMessage> get messageUpdateStream => _messageUpdateController.stream;
  Stream<WebSocketConnectionState> get connectionStateStream => _ws.connectionStateStream;

  // 初始化状态
  bool _isInitialized = false;
  Completer<void>? _initCompleter;

  // 订阅管理
  StreamSubscription<ChatMessage>? _wsMessageSubscription;
  StreamSubscription<WebSocketConnectionState>? _wsStateSubscription;

  /// 初始化服务（确保只初始化一次）
  Future<void> initialize() async {
    // 如果正在初始化，等待完成
    if (_initCompleter != null) {
      await _initCompleter!.future;
      return;
    }

    // 如果已经初始化，直接返回
    if (_isInitialized) {
      debugPrint('ChatService: Already initialized, skipping');
      return;
    }

    // 创建新的 Completer
    _initCompleter = Completer<void>();

    try {
      debugPrint('ChatService: Initializing...');

      // 监听 WebSocket 消息
      _wsMessageSubscription = _ws.messageStream.listen((message) {
        _handleIncomingMessage(message);
      });

      // 监听 WebSocket 连接状态
      _wsStateSubscription = _ws.connectionStateStream.listen((state) {
        debugPrint('ChatService: WebSocket state changed to $state');
      });

      // 连接 WebSocket
      await _ws.connect();

      _isInitialized = true;
      debugPrint('ChatService: Initialization complete');
      _initCompleter!.complete();
    } catch (e) {
      debugPrint('ChatService: Initialization failed - $e');
      _initCompleter!.completeError(e);
      rethrow;
    } finally {
      _initCompleter = null;
    }
  }

  /// 处理接收到的消息
  void _handleIncomingMessage(ChatMessage message) async {
    final conversationId = message.conversationId;

    // 从缓存中获取发送者信息（如果消息中没有）
    ChatMessage enrichedMessage = message;
    if ((message.senderUsername == null || message.senderAvatarUrl == null) &&
        _userInfoCache.containsKey(message.senderId)) {
      final userInfo = _userInfoCache[message.senderId]!;
      enrichedMessage = message.copyWith(
        senderUsername: userInfo.username,
        senderAvatarUrl: userInfo.avatarUrl,
      );
      debugPrint('ChatService: Enriched message with cached user info for ${message.senderId}');
    }

    // 获取当前用户 ID（用于判断是否为他人发送的消息）
    final currentUserId = _storage.getUserIdSync();

    // 检查这是否是我们自己发送的消息的回显
    // 通过匹配内容、发送者和时间戳来识别
    String? matchedLocalId;
    for (final entry in _pendingMessages.entries) {
      final pendingInfo = entry.value;
      if (pendingInfo.conversationId == conversationId &&
          pendingInfo.content == enrichedMessage.content &&
          pendingInfo.senderId == enrichedMessage.senderId &&
          pendingInfo.msgType == enrichedMessage.msgType) {
        // 检查时间戳是否在合理范围内（30秒内）
        final timeDiff = (enrichedMessage.timestamp - pendingInfo.timestamp).abs();
        if (timeDiff < 30000) {
          matchedLocalId = entry.key;
          debugPrint('ChatService: Received server echo for local message $matchedLocalId -> ${enrichedMessage.id}');
          break;
        }
      }
    }

    if (matchedLocalId != null) {
      // 这是我们发送的消息的回显，替换缓存中的本地消息
      _pendingMessages.remove(matchedLocalId);
      _messageCache.putIfAbsent(conversationId, () => []);

      // 找到并替换缓存中的本地消息
      final index = _messageCache[conversationId]!.indexWhere((m) => m.id == matchedLocalId);
      if (index != -1) {
        // 保留本地消息的 isAiGenerated 值（本地消息肯定是用户发送的，不是 AI）
        // 同时保留服务器返回的其他字段（如 id, delivered 等）
        final updatedMessage = enrichedMessage.copyWith(
          isAiGenerated: false, // 用户发送的消息肯定不是 AI
        );
        _messageCache[conversationId]![index] = updatedMessage;
        debugPrint('ChatService: Replaced local message $matchedLocalId with server message ${enrichedMessage.id}');
      } else {
        // 缓存中没找到，可能是新消息，添加进去
        _messageCache[conversationId]!.add(enrichedMessage);
      }

      // 通知消息更新（不是新消息）
      if (!_messageUpdateController.isClosed) {
        _messageUpdateController.add(enrichedMessage);
      }
    } else {
      // 这是来自其他人的新消息，正常处理
      _messageCache.putIfAbsent(conversationId, () => []);
      _messageCache[conversationId]!.add(enrichedMessage);

      // 通知新消息
      if (!_newMessageController.isClosed) {
        _newMessageController.add(enrichedMessage);
      }

      // ========== 通知处理 ==========
      // 如果消息不是当前用户发送的，显示通知并增加未读计数
      if (currentUserId != null && enrichedMessage.senderId != currentUserId) {
        await _showNotification(enrichedMessage);
        // 增加该会话的未读计数（会自动更新总未读数）
        _unreadService.incrementConversationUnread(conversationId);
      }
    }
  }

  /// 显示消息通知
  Future<void> _showNotification(ChatMessage message) async {
    try {
      final senderName = message.senderUsername ?? '用户';
      final content = _getMessageContentForNotification(message);

      if (message.isGroup) {
        // 群组消息通知
        // 从缓存或 API 获取群组名称
        String groupName = '群聊';
        // TODO: 可以从 conversation 服务获取群组名称
        await _notificationHelper.handleGroupMessage(
          groupName: groupName,
          senderName: senderName,
          message: content,
          conversationId: message.conversationId,
        );
      } else {
        // 私聊消息通知
        await _notificationHelper.handlePrivateMessage(
          senderName: senderName,
          message: content,
          userId: message.senderId,
        );
      }
    } catch (e) {
      debugPrint('ChatService: Failed to show notification - $e');
    }
  }

  /// 获取用于通知显示的消息内容
  String _getMessageContentForNotification(ChatMessage message) {
    switch (message.msgType) {
      case MsgType.text:
        return message.content;
      case MsgType.image:
        return '[图片]';
      case MsgType.video:
        return '[视频]';
      case MsgType.emoji:
        return message.content;
      case MsgType.system:
        return message.content;
    }
  }

  /// 获取会话消息列表
  Future<List<ChatMessage>> getMessages(
    String conversationId, {
    int limit = 50,
    int offset = 0,
    bool forceRefresh = false,
  }) async {
    // 确保已初始化
    await initialize();

    // 如果有缓存且不强制刷新，返回缓存
    if (!forceRefresh && _messageCache.containsKey(conversationId)) {
      final cached = _messageCache[conversationId]!;
      if (cached.length >= offset + limit) {
        return cached.skip(offset).take(limit).toList();
      }
    }

    try {
      // 直接使用 conversationId 获取消息（群组用 group_id，私聊用 conversation_id）
      final response = await _api.get<List<dynamic>>(
        '/conversations/$conversationId/messages',
        queryParameters: {
          'limit': limit.toString(),
          'offset': offset.toString(),
        },
        fromJson: (json) {
          final list = json['messages'] as List? ?? json as List;
          return list.map((e) => e as Map<String, dynamic>).toList();
        },
      );

      final messages = response
          .map((json) => ChatMessage.fromJson(json as Map<String, dynamic>))
          .toList();

      // 从消息中提取并缓存用户信息
      for (final msg in messages) {
        if (msg.senderUsername != null || msg.senderAvatarUrl != null) {
          _userInfoCache[msg.senderId] = _UserInfo(
            username: msg.senderUsername ?? msg.senderId,
            avatarUrl: msg.senderAvatarUrl,
          );
        }
      }

      // 从消息中提取私聊的 receiver_id 并缓存（如果是私聊）
      if (messages.isNotEmpty) {
        final firstMsg = messages.first;
        if (firstMsg.receiverId != null) {
          // 这是一个私聊消息，缓存 receiver_id
          // receiver_id 是对方用户的 ID（如果是当前用户发送的消息）
          // 或者是当前用户的 ID（如果是对方发送的消息）
          final currentUserId = await _storage.getUserId();
          String? otherUserId;

          for (final msg in messages) {
            if (msg.senderId != currentUserId) {
              otherUserId = msg.senderId;
              break;
            } else if (msg.receiverId != null && msg.receiverId != currentUserId) {
              otherUserId = msg.receiverId;
              break;
            }
          }

          if (otherUserId != null) {
            _privateConversationCache[conversationId] = _PrivateConversationInfo(
              receiverId: otherUserId,
              conversationId: conversationId,
            );
            debugPrint('ChatService: Cached receiverId=$otherUserId for $conversationId from messages');
          }
        }
      }

      // 合并本地待发送的消息（如果有）和服务器返回的消息
      final mergedMessages = _mergeLocalAndServerMessages(conversationId, messages);

      // 更新缓存
      _messageCache[conversationId] = mergedMessages;

      return mergedMessages;
    } catch (e) {
      debugPrint('ChatService: Failed to get messages for $conversationId - $e');
      // 如果请求失败且有缓存，返回缓存
      if (_messageCache.containsKey(conversationId)) {
        return _messageCache[conversationId]!;
      }
      rethrow;
    }
  }

  /// 合并本地消息和服务器消息，移除重复的本地消息
  List<ChatMessage> _mergeLocalAndServerMessages(
    String conversationId,
    List<ChatMessage> serverMessages,
  ) {
    final result = <ChatMessage>[];
    final matchedLocalIds = <String>{};

    for (final serverMsg in serverMessages) {
      // 检查是否有待发送的本地消息与此服务器消息匹配
      for (final entry in _pendingMessages.entries) {
        final localId = entry.key;
        final pendingInfo = entry.value;

        if (pendingInfo.conversationId == conversationId &&
            pendingInfo.content == serverMsg.content &&
            pendingInfo.senderId == serverMsg.senderId &&
            pendingInfo.msgType == serverMsg.msgType) {
          // 检查时间戳是否在合理范围内（5分钟内）
          final timeDiff = (serverMsg.timestamp - pendingInfo.timestamp).abs();
          if (timeDiff < 300000) {
            matchedLocalIds.add(localId);
            debugPrint('ChatService: Matched local message $localId to server message ${serverMsg.id}');
            break;
          }
        }
      }

      result.add(serverMsg);
    }

    // 添加未匹配的本地待发送消息（移除5分钟限制，避免消息丢失）
    for (final entry in _pendingMessages.entries) {
      if (!matchedLocalIds.contains(entry.key) && entry.value.conversationId == conversationId) {
        // 检查缓存中是否有此本地消息
        final cached = _messageCache[conversationId];
        if (cached != null) {
          final localMsg = cached.firstWhere(
            (m) => m.id == entry.key,
            orElse: () => entry.value.toChatMessage(),
          );
          // 添加本地待发送消息，不论时间（避免消息丢失）
          result.add(localMsg);
          debugPrint('ChatService: Adding unmatched local message ${entry.key}');
        }
      }
    }

    // 按时间戳排序
    result.sort((a, b) => a.timestamp.compareTo(b.timestamp));
    return result;
  }

  /// 发送文本消息
  Future<ChatMessage> sendTextMessage({
    required String conversationId,
    required String content,
    bool isGroup = false,
  }) async {
    return _sendMessage(
      conversationId: conversationId,
      content: content,
      msgType: MsgType.text,
      isGroup: isGroup,
    );
  }

  /// 快速发送文本消息（不等待确认，UI 响应更快）
  Future<ChatMessage> sendTextMessageFast({
    required String conversationId,
    required String content,
    bool isGroup = false,
  }) async {
    // 注意：这里会获取用户ID（如果缓存未初始化会异步读取）
    // 用于快速响应但确保有有效的用户ID

    // 先尝试从缓存获取
    var currentUserId = _storage.getUserIdSync();
    if (currentUserId == null) {
      // 如果缓存为空，异步读取
      currentUserId = await _storage.getUserId();
      if (currentUserId == null) {
        throw Exception('User not logged in');
      }
    }

    // 对于私聊，需要获取真实的 receiver_id
    // 如果 conversationId 实际上是 conversation UUID，需要查询 participants
    String? receiverId;
    String? groupId;
    String actualConversationId = conversationId;

    if (!isGroup) {
      // 私聊：获取真实的 receiver_id 和 conversation_id
      final participantInfo = await _getPrivateConversationInfo(conversationId);
      receiverId = participantInfo.receiverId;
      actualConversationId = participantInfo.conversationId;
      debugPrint('ChatService: Private chat - receiverId=$receiverId, conversationId=$actualConversationId');
    } else {
      // 群聊
      groupId = conversationId;
    }

    // 创建本地消息
    final message = ChatMessage.create(
      senderId: currentUserId,
      receiverId: receiverId,
      groupId: groupId,
      msgType: MsgType.text,
      content: content,
      conversationId: actualConversationId,  // 使用真实的 conversation_id
    );

    // 存储待发送消息信息（用于后续匹配服务器返回的消息）
    final cid = message.conversationId;
    _pendingMessages[message.id] = _PendingMessageInfo(
      localId: message.id,
      conversationId: cid,
      senderId: message.senderId,
      content: message.content,
      msgType: message.msgType,
      timestamp: message.timestamp,
      isGroup: isGroup,
      receiverId: receiverId,  // 存储真实的 receiver_id
    );

    // 添加到缓存
    _messageCache.putIfAbsent(cid, () => []);
    _messageCache[cid]!.add(message);

    // 通知 UI
    if (!_messageUpdateController.isClosed) {
      _messageUpdateController.add(message);
    }

    // 通过 WebSocket 发送（不等待，后台处理）
    try {
      _ws.sendMessageNoAck(message);
      // 更新为已发送状态
      final sentMessage = message.copyWith(status: MessageStatus.sent);
      _updateMessageInCache(sentMessage);
      if (!_messageUpdateController.isClosed) {
        _messageUpdateController.add(sentMessage);
      }
      return sentMessage;
    } catch (e) {
      // 发送失败，清理待发送消息
      _pendingMessages.remove(message.id);
      final failedMessage = message.copyWith(status: MessageStatus.failed);
      _updateMessageInCache(failedMessage);
      if (!_messageUpdateController.isClosed) {
        _messageUpdateController.add(failedMessage);
      }
      rethrow;
    }
  }

  /// 获取私聊的真实信息：receiver_id 和 conversation_id
  /// 重要：私聊必须使用真实的 receiver_id（用户ID），不能使用 conversation_id
  Future<_PrivateConversationInfo> _getPrivateConversationInfo(String conversationId) async {
    // 检查缓存
    if (_privateConversationCache.containsKey(conversationId)) {
      final cached = _privateConversationCache[conversationId]!;
      debugPrint('ChatService._getPrivateConversationInfo: Using cached receiverId=${cached.receiverId} for $conversationId');
      return cached;
    }

    debugPrint('ChatService._getPrivateConversationInfo: Fetching info for $conversationId');

    try {
      // 尝试作为 conversation UUID 查询
      final response = await _api.get<Map<String, dynamic>>(
        '/conversations/$conversationId',
        fromJson: (json) {
          debugPrint('ChatService._getPrivateConversationInfo: GET /conversations/$conversationId response: $json');
          return json as Map<String, dynamic>;
        },
      );

      // 成功获取到 conversation，提取参与者信息
      final conversation = response['conversation'] as Map<String, dynamic>?;
      if (conversation != null) {
        final participants = conversation['participants'] as List?;
        final currentUserId = await _storage.getUserId();
        debugPrint('ChatService._getPrivateConversationInfo: Current user=$currentUserId, participants count=${participants?.length}');

        if (participants != null && participants.length >= 2) {
          // 查找对方用户ID（不等于当前用户的参与者）
          for (var p in participants) {
            final participant = p as Map<String, dynamic>;
            final pid = participant['id'] as String;
            debugPrint('ChatService._getPrivateConversationInfo: Checking participant $pid');
            if (pid != currentUserId) {
              // 缓存结果
              final info = _PrivateConversationInfo(
                receiverId: pid,
                conversationId: conversationId,
              );
              _privateConversationCache[conversationId] = info;
              debugPrint('ChatService._getPrivateConversationInfo: Found and cached receiverId=$pid for $conversationId');
              return info;
            }
          }
        } else {
          debugPrint('ChatService._getPrivateConversationInfo: ERROR - participants count is ${participants?.length}, expected >= 2');
        }
      }

      // 如果没找到对方用户ID，这是错误状态
      debugPrint('ChatService._getPrivateConversationInfo: ERROR - Could not find receiver in conversation');
      throw Exception('Could not find other participant in conversation');
    } catch (e) {
      debugPrint('ChatService._getPrivateConversationInfo: Exception - $e');
      debugPrint('ChatService._getPrivateConversationInfo: Cannot determine receiver_id for $conversationId');
      rethrow;
    }
  }

  /// 发送图片消息
  Future<ChatMessage> sendImageMessage({
    required String conversationId,
    required String imageUrl,
    bool isGroup = false,
  }) async {
    return _sendMessage(
      conversationId: conversationId,
      content: imageUrl,
      msgType: MsgType.image,
      isGroup: isGroup,
    );
  }

  /// 发送视频消息
  Future<ChatMessage> sendVideoMessage({
    required String conversationId,
    required String videoUrl,
    bool isGroup = false,
  }) async {
    return _sendMessage(
      conversationId: conversationId,
      content: videoUrl,
      msgType: MsgType.video,
      isGroup: isGroup,
    );
  }

  /// 发送表情消息
  Future<ChatMessage> sendEmojiMessage({
    required String conversationId,
    required String emoji,
    bool isGroup = false,
  }) async {
    return _sendMessage(
      conversationId: conversationId,
      content: emoji,
      msgType: MsgType.emoji,
      isGroup: isGroup,
    );
  }

  /// 通用发送消息方法（减少代码重复）
  Future<ChatMessage> _sendMessage({
    required String conversationId,
    required String content,
    required MsgType msgType,
    bool isGroup = false,
  }) async {
    await initialize();

    final currentUserId = await _storage.getUserId();
    if (currentUserId == null) {
      throw Exception('User not logged in');
    }

    // 对于私聊，需要获取真实的 receiver_id
    String? receiverId;
    String? groupId;
    String actualConversationId = conversationId;

    if (!isGroup) {
      final participantInfo = await _getPrivateConversationInfo(conversationId);
      receiverId = participantInfo.receiverId;
      actualConversationId = participantInfo.conversationId;
    } else {
      groupId = conversationId;
    }

    final message = ChatMessage.create(
      senderId: currentUserId,
      receiverId: receiverId,
      groupId: groupId,
      msgType: msgType,
      content: content,
      conversationId: actualConversationId,
    );

    final cid = message.conversationId;
    _messageCache.putIfAbsent(cid, () => []);
    _messageCache[cid]!.add(message);
    if (!_messageUpdateController.isClosed) {
      _messageUpdateController.add(message);
    }

    try {
      await _ws.sendMessage(message);
      final sentMessage = message.copyWith(status: MessageStatus.sent);
      _updateMessageInCache(sentMessage);
      if (!_messageUpdateController.isClosed) {
        _messageUpdateController.add(sentMessage);
      }
      return sentMessage;
    } catch (e) {
      final failedMessage = message.copyWith(status: MessageStatus.failed);
      _updateMessageInCache(failedMessage);
      if (!_messageUpdateController.isClosed) {
        _messageUpdateController.add(failedMessage);
      }
      rethrow;
    }
  }

  /// 重试发送失败的消息
  Future<ChatMessage> retryMessage(ChatMessage message) async {
    // 移除旧消息
    final cid = message.conversationId;
    _messageCache[cid]?.removeWhere((m) => m.id == message.id);

    // 对于私聊，conversationId 是正确的会话 UUID
    // 对于群聊，groupId 是正确的群组 ID
    final isGroup = message.groupId != null;
    final targetId = isGroup ? message.groupId! : message.conversationId;

    switch (message.msgType) {
      case MsgType.text:
        return sendTextMessage(
          conversationId: targetId,
          content: message.content,
          isGroup: isGroup,
        );
      case MsgType.image:
        return sendImageMessage(
          conversationId: targetId,
          imageUrl: message.content,
          isGroup: isGroup,
        );
      case MsgType.video:
        return sendVideoMessage(
          conversationId: targetId,
          videoUrl: message.content,
          isGroup: isGroup,
        );
      case MsgType.emoji:
        return sendEmojiMessage(
          conversationId: targetId,
          emoji: message.content,
          isGroup: isGroup,
        );
      default:
        throw Exception('Unknown message type');
    }
  }

  /// 更新缓存中的消息
  void _updateMessageInCache(ChatMessage updatedMessage) {
    final cid = updatedMessage.conversationId;
    if (!_messageCache.containsKey(cid)) return;

    final index = _messageCache[cid]!.indexWhere((m) => m.id == updatedMessage.id);
    if (index != -1) {
      _messageCache[cid]![index] = updatedMessage;
    }
  }

  /// 获取缓存的会话消息
  List<ChatMessage> getCachedMessages(String conversationId) {
    return _messageCache[conversationId] ?? [];
  }

  /// 清除会话消息缓存
  void clearMessageCache(String conversationId) {
    _messageCache.remove(conversationId);
    _privateConversationCache.remove(conversationId);
  }

  /// 清除所有缓存
  void clearAllCache() {
    _messageCache.clear();
    _privateConversationCache.clear();
    _pendingMessages.clear();
  }

  /// 预加载私聊会话信息（从外部调用，例如从联系人页面跳转前）
  /// 这样可以避免首次发送消息时的额外 API 调用
  void preloadPrivateConversationInfo({
    required String conversationId,
    required String receiverId,
  }) {
    _privateConversationCache[conversationId] = _PrivateConversationInfo(
      receiverId: receiverId,
      conversationId: conversationId,
    );
    debugPrint('ChatService: Preloaded receiverId=$receiverId for $conversationId');
  }

  /// 标记消息为已读（仅更新本地未读计数）
  /// 注意：服务器没有已读状态 API，这里只清除本地未读计数
  void markAsRead(String conversationId) {
    _unreadService.markConversationAsRead(conversationId);
  }

  /// 发送输入状态
  void sendTypingIndicator({
    required String conversationId,
    required bool isTyping,
    bool isGroup = false,
  }) {
    _ws.sendTypingIndicator(
      conversationId: conversationId,
      isTyping: isTyping,
      isGroup: isGroup,
    );
  }

  /// 获取 WebSocket 连接状态
  WebSocketConnectionState get connectionState => _ws.state;

  /// 是否已连接
  bool get isConnected => _ws.isConnected;

  /// 是否已初始化
  bool get isInitialized => _isInitialized;

  /// 销毁服务
  void dispose() {
    _wsMessageSubscription?.cancel();
    _wsStateSubscription?.cancel();
    _newMessageController.close();
    _messageUpdateController.close();
    _pendingMessages.clear();
    _privateConversationCache.clear();
    _messageCache.clear();
  }
}

/// 待发送消息信息
class _PendingMessageInfo {
  final String localId;
  final String conversationId;
  final String senderId;
  final String content;
  final MsgType msgType;
  final int timestamp;
  final bool isGroup;
  final String? receiverId;  // 私聊时的真实 receiver_id

  _PendingMessageInfo({
    required this.localId,
    required this.conversationId,
    required this.senderId,
    required this.content,
    required this.msgType,
    required this.timestamp,
    required this.isGroup,
    this.receiverId,
  });

  ChatMessage toChatMessage() {
    return ChatMessage(
      id: localId,
      senderId: senderId,
      receiverId: isGroup ? null : receiverId,
      groupId: isGroup ? conversationId : null,
      msgType: msgType,
      content: content,
      timestamp: timestamp,
      status: MessageStatus.sent,
      isAiGenerated: false,
      delivered: false,
      createdAt: DateTime.fromMillisecondsSinceEpoch(timestamp).toIso8601String(),
      conversationId: conversationId,
    );
  }
}

/// 私聊会话信息（缓存）
class _PrivateConversationInfo {
  final String receiverId;  // 对方用户ID
  final String conversationId;  // 会话UUID

  _PrivateConversationInfo({
    required this.receiverId,
    required this.conversationId,
  });
}

/// 用户信息缓存
class _UserInfo {
  final String username;
  final String? avatarUrl;

  _UserInfo({
    required this.username,
    this.avatarUrl,
  });
}
