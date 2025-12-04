import 'package:drift/drift.dart';
import '../local/database.dart';
import '../models/message.dart' as model;

/// Message repository for data access
class MessageRepository {
  final AppDatabase _db;

  MessageRepository(this._db);

  /// Get messages by conversation
  Future<List<model.Message>> getMessages(
    String conversationId, {
    int limit = 50,
    int offset = 0,
  }) async {
    final messages = await _db.getMessagesByConversation(
      conversationId,
      limit: limit,
      offset: offset,
    );
    return messages.map(_mapToModel).toList();
  }

  /// Watch messages stream
  Stream<List<model.Message>> watchMessages(String conversationId) {
    return _db.watchMessages(conversationId).map(
      (messages) => messages.map(_mapToModel).toList(),
    );
  }

  /// Insert message
  Future<void> insertMessage(model.Message message) async {
    await _db.insertMessage(
      MessagesCompanion(
        id: Value(message.id),
        conversationId: Value(message.conversationId),
        senderId: Value(message.senderId),
        senderName: Value(message.senderName),
        type: Value(message.type.name),
        content: Value(message.content),
        status: Value(message.status.name),
        replyToId: Value(message.replyToId),
        isDeleted: Value(message.isDeleted),
        createdAt: Value(message.createdAt),
        editedAt: Value(message.editedAt),
      ),
    );
  }

  /// Update message status
  Future<void> updateMessageStatus(String id, model.MessageStatus status) async {
    await (_db.update(_db.messages)..where((m) => m.id.equals(id)))
        .write(MessagesCompanion(status: Value(status.name)));
  }

  /// Mark message as deleted
  Future<void> markAsDeleted(String id) async {
    await (_db.update(_db.messages)..where((m) => m.id.equals(id)))
        .write(const MessagesCompanion(isDeleted: Value(true)));
  }

  /// Delete message
  Future<void> deleteMessage(String id) async {
    await _db.deleteMessage(id);
  }

  /// Delete all messages in conversation
  Future<void> deleteConversationMessages(String conversationId) async {
    await (_db.delete(_db.messages)
      ..where((m) => m.conversationId.equals(conversationId)))
        .go();
  }

  model.Message _mapToModel(Message msg) {
    return model.Message(
      id: msg.id,
      conversationId: msg.conversationId,
      senderId: msg.senderId,
      senderName: msg.senderName,
      type: model.MessageType.values.firstWhere(
        (t) => t.name == msg.type,
        orElse: () => model.MessageType.text,
      ),
      content: msg.content,
      status: model.MessageStatus.values.firstWhere(
        (s) => s.name == msg.status,
        orElse: () => model.MessageStatus.sent,
      ),
      replyToId: msg.replyToId,
      createdAt: msg.createdAt,
      editedAt: msg.editedAt,
      isDeleted: msg.isDeleted,
    );
  }
}
