import '../../../core/storage/storage.dart';

/// 参与者信息
class Participant {
  final String id;
  final String? username;
  final String? avatarUrl;

  Participant({
    required this.id,
    this.username,
    this.avatarUrl,
  });

  factory Participant.fromJson(Map<String, dynamic> json) {
    return Participant(
      id: _asString(json['id']) ?? '',
      username: _asString(json['username']),
      avatarUrl: _asString(json['avatar_url']),
    );
  }

  /// 安全转换为 String
  static String? _asString(dynamic value) {
    if (value == null) return null;
    if (value is String) return value;
    if (value is num) return value.toString();
    return value.toString();
  }
}

/// 对话模型
class Conversation {
  final String id;
  final String type; // 'private' or 'group'
  final String? name;
  final String? avatarUrl;
  final String? lastMessage;
  final DateTime? lastMessageAt;
  final int unreadCount;
  final DateTime createdAt;
  final DateTime? updatedAt;
  final List<String>? memberIds;
  final List<Participant>? participants;

  Conversation({
    required this.id,
    required this.type,
    this.name,
    this.avatarUrl,
    this.lastMessage,
    this.lastMessageAt,
    this.unreadCount = 0,
    required this.createdAt,
    this.updatedAt,
    this.memberIds,
    this.participants,
  });

  bool get isGroup => type.toUpperCase() == 'GROUP';
  
  String get displayName {
    if (name != null && name!.isNotEmpty) return name!;
    return '未命名对话';
  }

  /// 安全转换为 String
  static String? _asString(dynamic value) {
    if (value == null) return null;
    if (value is String) return value;
    // 处理数字类型（int, double, num）
    if (value is num) return value.toString();
    // 其他类型转字符串
    return value.toString();
  }

  /// 安全转换为 int
  static int _asInt(dynamic value, [int defaultValue = 0]) {
    if (value == null) return defaultValue;
    if (value is int) return value;
    if (value is String) return int.tryParse(value) ?? defaultValue;
    return defaultValue;
  }

  factory Conversation.fromJson(Map<String, dynamic> json) {
    // API 返回 conversation_type (PRIVATE/GROUP) 或 type
    String type = _asString(json['conversation_type']) ??
                  _asString(json['type']) ?? 'PRIVATE';
    // 统一转小写
    type = type.toLowerCase();
    if (type == 'private') type = 'private';
    if (type == 'group') type = 'group';

    // 从 participants 提取显示名称和头像
    String? displayName = _asString(json['name']);
    String? avatarUrl = _asString(json['avatar_url']);

    // 后端返回 participants 为对象数组 [{id, username, avatar_url}, ...]
    final participantsData = json['participants'];
    List<String>? memberIds;
    List<Participant>? participants;

    if (participantsData is List && participantsData.isNotEmpty) {
      // 安全解析参与者列表
      final ids = <String>[];
      final parsedParticipants = <Participant>[];

      for (final p in participantsData) {
        // p 可能是 Map<String, dynamic> 或 Map<dynamic, dynamic>
        if (p is Map) {
          final participant = Participant.fromJson(Map<String, dynamic>.from(p));
          final idStr = participant.id;
          if (idStr.isNotEmpty) {
            ids.add(idStr);
            parsedParticipants.add(participant);
          }
        }
      }

      memberIds = ids.isNotEmpty ? ids : null;
      participants = parsedParticipants.isNotEmpty ? parsedParticipants : null;

      // 如果没有显示名称，从参与者中提取
      if ((displayName == null || displayName.isEmpty) && participants != null && participants.isNotEmpty) {
        if (type == 'private') {
          // 私聊：显示对方用户（非当前用户）
          final currentUserId = SecureStorage.instance.getUserIdSync();
          Participant otherUser;
          try {
            otherUser = participants.firstWhere((p) => p.id != currentUserId);
          } catch (_) {
            otherUser = participants.first;
          }
          displayName = otherUser.username ?? '未知用户';
          // 对于私聊，直接使用对方的头像（覆盖可能存在的旧值）
          avatarUrl = otherUser.avatarUrl;
        } else {
          // 群聊显示群名或所有成员名称
          final usernames = participants
              .map((p) => p.username)
              .where((u) => u != null && u.isNotEmpty)
              .cast<String>()
              .toList();
          if (usernames.isNotEmpty) {
            displayName = usernames.join(', ');
          }
        }
      }
    }

    return Conversation(
      id: _asString(json['id']) ?? '',
      type: type,
      name: displayName,
      avatarUrl: avatarUrl,
      lastMessage: _asString(json['last_message']),
      lastMessageAt: json['last_message_at'] != null
          ? DateTime.tryParse(_asString(json['last_message_at']) ?? '')
          : null,
      unreadCount: _asInt(json['unread_count']),
      createdAt: DateTime.tryParse(_asString(json['created_at']) ?? '') ?? DateTime.now(),
      updatedAt: json['updated_at'] != null
          ? DateTime.tryParse(_asString(json['updated_at']) ?? '')
          : null,
      memberIds: memberIds,
      participants: participants,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'type': type,
      'name': name,
      'avatar_url': avatarUrl,
      'last_message': lastMessage,
      'last_message_at': lastMessageAt?.toIso8601String(),
      'unread_count': unreadCount,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt?.toIso8601String(),
      'member_ids': memberIds,
      'participants': participants?.map((p) => {
        'id': p.id,
        'username': p.username,
        'avatar_url': p.avatarUrl,
      }).toList(),
    };
  }
}
