import 'package:drift/drift.dart';
import '../local/database.dart';
import '../models/conversation.dart' as model;
import '../models/user.dart' as user_model;
import '../models/message.dart' as message_model;

/// Conversation repository for data access
class ConversationRepository {
  final AppDatabase _db;

  ConversationRepository(this._db);

  /// Get all conversations with last message
  Future<List<model.Conversation>> getAllConversations() async {
    final conversations = await _db.getAllConversations();
    final result = <model.Conversation>[];

    for (final conv in conversations) {
      final participants = await _getParticipants(conv.id);
      final lastMessage = await _getLastMessage(conv.id);
      result.add(_mapToModel(conv, participants, lastMessage));
    }

    return result;
  }

  /// Watch conversations stream
  Stream<List<model.Conversation>> watchConversations() {
    return _db.watchConversations().asyncMap((conversations) async {
      final result = <model.Conversation>[];
      for (final conv in conversations) {
        final participants = await _getParticipants(conv.id);
        final lastMessage = await _getLastMessage(conv.id);
        result.add(_mapToModel(conv, participants, lastMessage));
      }
      return result;
    });
  }

  /// Get conversation by ID
  Future<model.Conversation?> getConversationById(String id) async {
    final conv = await _db.getConversationById(id);
    if (conv == null) return null;

    final participants = await _getParticipants(id);
    final lastMessage = await _getLastMessage(id);
    return _mapToModel(conv, participants, lastMessage);
  }

  /// Insert or update conversation
  Future<void> upsertConversation(model.Conversation conversation) async {
    await _db.into(_db.conversations).insertOnConflictUpdate(
      ConversationsCompanion(
        id: Value(conversation.id),
        type: Value(conversation.type.name),
        name: Value(conversation.name),
        avatarUrl: Value(conversation.avatarUrl),
        unreadCount: Value(conversation.unreadCount),
        isMuted: Value(conversation.isMuted),
        isPinned: Value(conversation.isPinned),
        lastActivityAt: Value(conversation.lastActivityAt),
        createdAt: Value(conversation.createdAt),
        updatedAt: Value(DateTime.now()),
      ),
    );
  }

  /// Update unread count
  Future<void> updateUnreadCount(String id, int count) async {
    await (_db.update(_db.conversations)..where((c) => c.id.equals(id)))
        .write(ConversationsCompanion(unreadCount: Value(count)));
  }

  /// Mark as read
  Future<void> markAsRead(String id) async {
    await updateUnreadCount(id, 0);
  }

  /// Delete conversation
  Future<void> deleteConversation(String id) async {
    await _db.deleteConversation(id);
  }

  Future<List<user_model.User>> _getParticipants(String conversationId) async {
    final query = _db.select(_db.conversationParticipants).join([
      innerJoin(_db.users, _db.users.id.equalsExp(_db.conversationParticipants.userId)),
    ])..where(_db.conversationParticipants.conversationId.equals(conversationId));

    final results = await query.get();
    return results.map((row) {
      final user = row.readTable(_db.users);
      return user_model.User(
        id: user.id,
        username: user.username,
        email: user.email,
        avatarUrl: user.avatarUrl,
        bio: user.bio,
        isOnline: user.isOnline,
        lastSeenAt: user.lastSeenAt,
        createdAt: user.createdAt,
      );
    }).toList();
  }

  Future<message_model.Message?> _getLastMessage(String conversationId) async {
    final messages = await _db.getMessagesByConversation(conversationId, limit: 1);
    if (messages.isEmpty) return null;

    final msg = messages.first;
    return message_model.Message(
      id: msg.id,
      conversationId: msg.conversationId,
      senderId: msg.senderId,
      senderName: msg.senderName,
      type: message_model.MessageType.values.firstWhere(
        (t) => t.name == msg.type,
        orElse: () => message_model.MessageType.text,
      ),
      content: msg.content,
      status: message_model.MessageStatus.values.firstWhere(
        (s) => s.name == msg.status,
        orElse: () => message_model.MessageStatus.sent,
      ),
      replyToId: msg.replyToId,
      createdAt: msg.createdAt,
      editedAt: msg.editedAt,
      isDeleted: msg.isDeleted,
    );
  }

  model.Conversation _mapToModel(
    Conversation conv,
    List<user_model.User> participants,
    message_model.Message? lastMessage,
  ) {
    return model.Conversation(
      id: conv.id,
      type: model.ConversationType.values.firstWhere(
        (t) => t.name == conv.type,
        orElse: () => model.ConversationType.private,
      ),
      name: conv.name,
      avatarUrl: conv.avatarUrl,
      participants: participants,
      lastMessage: lastMessage,
      unreadCount: conv.unreadCount,
      isMuted: conv.isMuted,
      isPinned: conv.isPinned,
      lastActivityAt: conv.lastActivityAt,
      createdAt: conv.createdAt,
    );
  }
}
