/// 消息模型
class Message {
  final String id;
  final String conversationId;
  final String senderId;
  final String? senderName;
  final String? senderAvatar;
  final String content;
  final String type;
  final DateTime createdAt;
  final bool isRead;
  final String? replyToId;

  Message({
    required this.id,
    required this.conversationId,
    required this.senderId,
    this.senderName,
    this.senderAvatar,
    required this.content,
    this.type = 'text',
    required this.createdAt,
    this.isRead = false,
    this.replyToId,
  });

  /// 是否是自己发送的消息
  bool get isMe => _currentUserId == senderId;
  static String? _currentUserId;
  static void setCurrentUserId(String userId) => _currentUserId = userId;

  /// 安全转换为 String
  static String? _asString(dynamic value) {
    if (value == null) return null;
    if (value is String) return value;
    // 处理数字类型（int, double, num）
    if (value is num) return value.toString();
    // 其他类型转字符串
    return value.toString();
  }

  /// 安全转换为 bool
  static bool _asBool(dynamic value, [bool defaultValue = false]) {
    if (value == null) return defaultValue;
    if (value is bool) return value;
    if (value is String) return value.toLowerCase() == 'true';
    if (value is num) return value != 0;
    return defaultValue;
  }

  factory Message.fromJson(Map<String, dynamic> json) {
    // 处理 msg_type 数字类型
    String msgType = 'text';
    final rawType = json['type'] ?? json['msg_type'] ?? json['message_type'];
    if (rawType is int) {
      switch (rawType) {
        case 0: msgType = 'text'; break;
        case 1: msgType = 'image'; break;
        case 2: msgType = 'video'; break;
        case 3: msgType = 'emoji'; break;
        default: msgType = 'text';
      }
    } else if (rawType is String) {
      msgType = rawType.toLowerCase();
    }

    // 处理 created_at （可能是字符串或 timestamp）
    DateTime createdAt;
    final rawCreatedAt = json['created_at'] ?? json['timestamp'];
    if (rawCreatedAt is int) {
      createdAt = DateTime.fromMillisecondsSinceEpoch(rawCreatedAt);
    } else if (rawCreatedAt is String) {
      createdAt = DateTime.tryParse(rawCreatedAt) ?? DateTime.now();
    } else {
      createdAt = DateTime.now();
    }

    return Message(
      id: _asString(json['id']) ?? '',
      conversationId: _asString(json['conversation_id']) ?? _asString(json['group_id']) ?? '',
      senderId: _asString(json['sender_id']) ?? '',
      senderName: _asString(json['sender_name']),
      senderAvatar: _asString(json['sender_avatar']),
      content: _asString(json['content']) ?? '',
      type: msgType,
      createdAt: createdAt,
      isRead: _asBool(json['is_read']) || _asBool(json['read']),
      replyToId: _asString(json['reply_to_id']) ?? _asString(json['reply_to']),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'conversation_id': conversationId,
      'sender_id': senderId,
      'sender_name': senderName,
      'sender_avatar': senderAvatar,
      'content': content,
      'type': type,
      'created_at': createdAt.toIso8601String(),
      'is_read': isRead,
      'reply_to_id': replyToId,
    };
  }

  /// 从 WebSocket 消息创建
  factory Message.fromWebSocket(Map<String, dynamic> json) {
    // 后端发送格式: { id, conversation_id, sender_id, sender_name, type, content, status, created_at }
    final payload = json['data'] is Map
        ? json['data']
        : json;

    return Message(
      id: _asString(payload['id']) ?? '',
      conversationId: _asString(payload['conversation_id']) ?? '',
      senderId: _asString(payload['sender_id']) ?? '',
      senderName: _asString(payload['sender_name']),
      senderAvatar: _asString(payload['sender_avatar']),
      content: _asString(payload['content']) ?? '',
      type: _asString(payload['type']) ?? 'text',
      createdAt: _parseDateTime(payload['created_at']),
      isRead: payload['status'] == 'delivered',
      replyToId: _asString(payload['reply_to_id']) ?? _asString(payload['reply_to']),
    );
  }

  /// 安全解析 DateTime
  static DateTime _parseDateTime(dynamic value) {
    if (value == null) return DateTime.now();
    if (value is DateTime) return value;
    if (value is String) {
      return DateTime.tryParse(value) ?? DateTime.now();
    }
    if (value is int) {
      return DateTime.fromMillisecondsSinceEpoch(value);
    }
    return DateTime.now();
  }
}
