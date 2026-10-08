/// 消息类型枚举，对齐后端 MsgType
enum MsgType {
  text(0),
  image(1),
  video(2),
  emoji(3),
  system(99);

  final int value;
  const MsgType(this.value);

  static MsgType fromValue(int value) {
    switch (value) {
      case 0:
        return text;
      case 1:
        return image;
      case 2:
        return video;
      case 3:
        return emoji;
      default:
        return text;
    }
  }

  String toJson() => value.toString();
}

/// 消息状态
enum MessageStatus {
  sending,
  sent,
  delivered,
  failed,
}

/// 聊天消息模型，对齐后端 Message 结构
class ChatMessage {
  final String id;
  final String senderId;
  final String? senderUsername;  // 发送者用户名
  final String? senderAvatarUrl; // 发送者头像
  final String? _conversationId; // 服务器直接返回的 conversation_id
  final String? receiverId; // 私聊时的接收者ID
  final String? groupId; // 群聊时的群组ID
  final MsgType msgType;
  final String content;
  final int timestamp; // 毫秒时间戳
  final bool isAiGenerated;
  final bool delivered;
  final String createdAt;
  final MessageStatus status; // 本地状态，不在后端

  ChatMessage({
    required this.id,
    required this.senderId,
    this.senderUsername,
    this.senderAvatarUrl,
    String? conversationId,
    this.receiverId,
    this.groupId,
    required this.msgType,
    required this.content,
    required this.timestamp,
    this.isAiGenerated = false,
    this.delivered = false,
    required this.createdAt,
    this.status = MessageStatus.sent,
  }) : _conversationId = conversationId;

  /// 是否为私聊消息
  bool get isPrivate => receiverId != null;

  /// 是否为群聊消息
  bool get isGroup => groupId != null;

  /// 获取会话ID
  /// 优先使用服务器返回的 conversation_id，其次是 groupId 或 receiverId
  String get conversationId => _conversationId ?? groupId ?? receiverId ?? id;

  /// 从 JSON 创建（后端响应格式）
  factory ChatMessage.fromJson(Map<String, dynamic> json) {
    // 辅助函数
    String asString(dynamic value, {String defaultValue = ''}) {
      if (value == null) return defaultValue;
      if (value is String) return value;
      return value.toString();
    }

    int asInt(dynamic value, {int defaultValue = 0}) {
      if (value == null) return defaultValue;
      if (value is int) return value;
      if (value is String) return int.tryParse(value) ?? defaultValue;
      if (value is double) return value.toInt();
      return defaultValue;
    }

    return ChatMessage(
      id: asString(json['id'] ?? ''),
      senderId: asString(json['sender_id'] ?? ''),
      senderUsername: json['sender_username'] as String?,
      senderAvatarUrl: json['sender_avatar_url'] as String?,
      conversationId: json['conversation_id'] != null ? asString(json['conversation_id']) : null,
      receiverId: json['receiver_id'] != null ? asString(json['receiver_id']) : null,
      groupId: json['group_id'] != null ? asString(json['group_id']) : null,
      msgType: MsgType.fromValue(asInt(json['msg_type'], defaultValue: 0)),
      content: asString(json['content'] ?? ''),
      timestamp: asInt(json['timestamp'], defaultValue: DateTime.now().millisecondsSinceEpoch),
      isAiGenerated: json['is_ai_generated'] as bool? ?? false,
      delivered: json['delivered'] as bool? ?? false,
      createdAt: asString(json['created_at'] ?? DateTime.now().toIso8601String()),
      status: json['delivered'] == true ? MessageStatus.delivered : MessageStatus.sent,
    );
  }

  /// 从 WebSocket 消息创建
  factory ChatMessage.fromWebSocket(Map<String, dynamic> json) {
    // WebSocket 格式: WsMessage
    return ChatMessage(
      id: json['id'] as String,
      senderId: json['sender_id'] as String,
      conversationId: json['conversation_id'] as String?,
      receiverId: json['receiver_id'] as String?,
      groupId: json['group_id'] as String?,
      msgType: _parseMsgType(json['type'] as String? ?? 'text'),
      content: json['content'] as String? ?? '',
      timestamp: DateTime.parse(json['created_at'] as String? ?? DateTime.now().toIso8601String())
          .millisecondsSinceEpoch,
      isAiGenerated: json['is_ai_generated'] as bool? ?? false,
      delivered: json['status'] == 'delivered',
      createdAt: json['created_at'] as String? ?? DateTime.now().toIso8601String(),
      status: json['status'] == 'delivered' ? MessageStatus.delivered : MessageStatus.sent,
    );
  }

  static MsgType _parseMsgType(String type) {
    switch (type) {
      case 'text':
        return MsgType.text;
      case 'image':
        return MsgType.image;
      case 'video':
        return MsgType.video;
      case 'emoji':
        return MsgType.emoji;
      default:
        return MsgType.text;
    }
  }

  /// 创建待发送的消息（本地生成ID）
  factory ChatMessage.create({
    required String senderId,
    String? conversationId,  // 可选：真实的 conversation_id
    String? receiverId,
    String? groupId,
    required MsgType msgType,
    required String content,
  }) {
    final now = DateTime.now();
    return ChatMessage(
      id: _generateLocalId(),
      senderId: senderId,
      conversationId: conversationId,
      receiverId: receiverId,
      groupId: groupId,
      msgType: msgType,
      content: content,
      timestamp: now.millisecondsSinceEpoch,
      isAiGenerated: false,
      delivered: false,
      createdAt: now.toIso8601String(),
      status: MessageStatus.sending,
    );
  }

  /// 转换为 WebSocket 发送格式
  Map<String, dynamic> toWebSocket() {
    final json = <String, dynamic>{
      'type': 'message',
      'id': id,
      'sender_id': senderId,
      'msg_type': msgType.value,
      'content': content,
      // Include conversation_id for proper server-side matching
      'conversation_id': conversationId,
    };

    if (groupId != null) {
      json['group_id'] = groupId;
    } else if (receiverId != null) {
      json['receiver_id'] = receiverId;
    }

    return json;
  }

  /// 转换为 JSON
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'sender_id': senderId,
      if (_conversationId != null) 'conversation_id': _conversationId,
      if (receiverId != null) 'receiver_id': receiverId,
      if (groupId != null) 'group_id': groupId,
      'msg_type': msgType.value,
      'content': content,
      'timestamp': timestamp,
      'is_ai_generated': isAiGenerated,
      'delivered': delivered,
      'created_at': createdAt,
    };
  }

  /// 复制并更新部分字段
  ChatMessage copyWith({
    String? id,
    String? senderId,
    String? senderUsername,
    String? senderAvatarUrl,
    String? conversationId,
    String? receiverId,
    String? groupId,
    MsgType? msgType,
    String? content,
    int? timestamp,
    bool? isAiGenerated,
    bool? delivered,
    String? createdAt,
    MessageStatus? status,
  }) {
    return ChatMessage(
      id: id ?? this.id,
      senderId: senderId ?? this.senderId,
      senderUsername: senderUsername ?? this.senderUsername,
      senderAvatarUrl: senderAvatarUrl ?? this.senderAvatarUrl,
      conversationId: conversationId ?? _conversationId,
      receiverId: receiverId ?? this.receiverId,
      groupId: groupId ?? this.groupId,
      msgType: msgType ?? this.msgType,
      content: content ?? this.content,
      timestamp: timestamp ?? this.timestamp,
      isAiGenerated: isAiGenerated ?? this.isAiGenerated,
      delivered: delivered ?? this.delivered,
      createdAt: createdAt ?? this.createdAt,
      status: status ?? this.status,
    );
  }

  /// 生成本地临时ID
  static String _generateLocalId() {
    return 'local_${DateTime.now().millisecondsSinceEpoch}_${_randomString(8)}';
  }

  static String _randomString(int length) {
    const chars = 'abcdefghijklmnopqrstuvwxyz0123456789';
    final random = DateTime.now().millisecondsSinceEpoch;
    final sb = StringBuffer();
    for (var i = 0; i < length; i++) {
      sb.write(chars[(random + i) % chars.length]);
    }
    return sb.toString();
  }

  @override
  String toString() {
    return 'ChatMessage(id: $id, senderId: $senderId, content: $content, type: $msgType, status: $status)';
  }
}

/// 输入状态指示器
class TypingIndicator {
  final String userId;
  final String? groupId;
  final bool isTyping;

  TypingIndicator({
    required this.userId,
    this.groupId,
    required this.isTyping,
  });

  factory TypingIndicator.fromJson(Map<String, dynamic> json) {
    return TypingIndicator(
      userId: json['user_id'] as String,
      groupId: json['group_id'] as String?,
      isTyping: json['is_typing'] as bool? ?? false,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'type': 'typing',
      'user_id': userId,
      if (groupId != null) 'group_id': groupId,
      'is_typing': isTyping,
    };
  }
}

/// 在线状态更新
class PresenceUpdate {
  final String userId;
  final String status; // online, offline, away
  final String? statusMessage;
  final bool? isMobile;
  final DateTime? lastSeen;

  PresenceUpdate({
    required this.userId,
    required this.status,
    this.statusMessage,
    this.isMobile,
    this.lastSeen,
  });

  factory PresenceUpdate.fromJson(Map<String, dynamic> json) {
    return PresenceUpdate(
      userId: json['user_id'] as String,
      status: json['status'] as String,
      statusMessage: json['status_message'] as String?,
      isMobile: json['is_mobile'] as bool?,
      lastSeen: json['last_seen'] != null
          ? DateTime.parse(json['last_seen'] as String)
          : null,
    );
  }
}

/// WebSocket 服务器包类型
enum ServerPacketType {
  message,
  syncResponse,
  typing,
  ack,
  userStatusUpdate,
  groupUpdate,
  conversationUpdate,
  presenceUpdate,
  uploadProgress,
  aiMessage,
  systemNotification,
}

/// WebSocket 服务器包
class ServerPacket {
  final ServerPacketType? type;
  final ChatMessage? message;
  final Map<String, dynamic>? rawData;

  ServerPacket({
    this.type,
    this.message,
    this.rawData,
  });

  factory ServerPacket.fromJson(Map<String, dynamic> json) {
    final typeStr = json['type'] as String?;
    ServerPacketType? type;

    if (typeStr != null) {
      switch (typeStr) {
        case 'message':
        case 'new_message':
          type = ServerPacketType.message;
          break;
        case 'sync_response':
          type = ServerPacketType.syncResponse;
          break;
        case 'typing':
          type = ServerPacketType.typing;
          break;
        case 'ack':
          type = ServerPacketType.ack;
          break;
        case 'user_status_update':
          type = ServerPacketType.userStatusUpdate;
          break;
        case 'group_update':
          type = ServerPacketType.groupUpdate;
          break;
        case 'conversation_update':
          type = ServerPacketType.conversationUpdate;
          break;
        case 'presence_update':
          type = ServerPacketType.presenceUpdate;
          break;
        case 'upload_progress':
          type = ServerPacketType.uploadProgress;
          break;
        case 'ai_message':
          type = ServerPacketType.aiMessage;
          break;
        case 'system_notification':
          type = ServerPacketType.systemNotification;
          break;
      }
    }

    ChatMessage? message;
    if (type == ServerPacketType.message && (json['id'] != null || json['sender_id'] != null)) {
      try {
        message = ChatMessage.fromWebSocket(json);
      } catch (_) {
        // 解析失败，保持 message 为 null
      }
    }

    return ServerPacket(
      type: type,
      message: message,
      rawData: json,
    );
  }
}
