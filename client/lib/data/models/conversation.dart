import 'package:equatable/equatable.dart';
import 'message.dart';
import 'user.dart';

/// Conversation model
class Conversation extends Equatable {
  final String id;
  final ConversationType type;
  final String? name;
  final String? avatarUrl;
  final List<User> participants;
  final Message? lastMessage;
  final int unreadCount;
  final bool isMuted;
  final bool isPinned;
  final DateTime? lastActivityAt;
  final DateTime createdAt;

  const Conversation({
    required this.id,
    required this.type,
    this.name,
    this.avatarUrl,
    this.participants = const [],
    this.lastMessage,
    this.unreadCount = 0,
    this.isMuted = false,
    this.isPinned = false,
    this.lastActivityAt,
    required this.createdAt,
  });

  /// Get display name for conversation
  String get displayName {
    if (name != null && name!.isNotEmpty) return name!;
    if (type == ConversationType.private && participants.isNotEmpty) {
      return participants.first.displayName;
    }
    return '未命名会话';
  }

  /// Get display avatar
  String? get displayAvatar {
    if (avatarUrl != null) return avatarUrl;
    if (type == ConversationType.private && participants.isNotEmpty) {
      return participants.first.avatarUrl;
    }
    return null;
  }

  /// Check if other user is online (for private chats)
  bool get isOnline {
    if (type == ConversationType.private && participants.isNotEmpty) {
      return participants.first.isOnline;
    }
    return false;
  }

  Conversation copyWith({
    String? id,
    ConversationType? type,
    String? name,
    String? avatarUrl,
    List<User>? participants,
    Message? lastMessage,
    int? unreadCount,
    bool? isMuted,
    bool? isPinned,
    DateTime? lastActivityAt,
    DateTime? createdAt,
  }) {
    return Conversation(
      id: id ?? this.id,
      type: type ?? this.type,
      name: name ?? this.name,
      avatarUrl: avatarUrl ?? this.avatarUrl,
      participants: participants ?? this.participants,
      lastMessage: lastMessage ?? this.lastMessage,
      unreadCount: unreadCount ?? this.unreadCount,
      isMuted: isMuted ?? this.isMuted,
      isPinned: isPinned ?? this.isPinned,
      lastActivityAt: lastActivityAt ?? this.lastActivityAt,
      createdAt: createdAt ?? this.createdAt,
    );
  }

  @override
  List<Object?> get props => [
        id,
        type,
        name,
        avatarUrl,
        participants,
        lastMessage,
        unreadCount,
        isMuted,
        isPinned,
        lastActivityAt,
        createdAt,
      ];
}

/// Conversation type
enum ConversationType {
  private,
  group,
  aiAgent,
}
