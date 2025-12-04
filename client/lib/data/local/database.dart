import 'dart:io';
import 'package:drift/drift.dart';
import 'package:drift/native.dart';
import 'package:path_provider/path_provider.dart';
import 'package:path/path.dart' as p;

part 'database.g.dart';

/// Users table
class Users extends Table {
  TextColumn get id => text()();
  TextColumn get username => text()();
  TextColumn get email => text().nullable()();
  TextColumn get avatarUrl => text().nullable()();
  TextColumn get bio => text().nullable()();
  BoolColumn get isOnline => boolean().withDefault(const Constant(false))();
  DateTimeColumn get lastSeenAt => dateTime().nullable()();
  DateTimeColumn get createdAt => dateTime()();
  DateTimeColumn get updatedAt => dateTime().nullable()();

  @override
  Set<Column> get primaryKey => {id};
}

/// Conversations table
class Conversations extends Table {
  TextColumn get id => text()();
  TextColumn get type => text()(); // private, group, aiAgent
  TextColumn get name => text().nullable()();
  TextColumn get avatarUrl => text().nullable()();
  IntColumn get unreadCount => integer().withDefault(const Constant(0))();
  BoolColumn get isMuted => boolean().withDefault(const Constant(false))();
  BoolColumn get isPinned => boolean().withDefault(const Constant(false))();
  DateTimeColumn get lastActivityAt => dateTime().nullable()();
  DateTimeColumn get createdAt => dateTime()();
  DateTimeColumn get updatedAt => dateTime().nullable()();

  @override
  Set<Column> get primaryKey => {id};
}

/// Conversation participants (many-to-many)
class ConversationParticipants extends Table {
  TextColumn get conversationId => text().references(Conversations, #id)();
  TextColumn get userId => text().references(Users, #id)();
  DateTimeColumn get joinedAt => dateTime()();

  @override
  Set<Column> get primaryKey => {conversationId, userId};
}

/// Messages table
class Messages extends Table {
  TextColumn get id => text()();
  TextColumn get conversationId => text().references(Conversations, #id)();
  TextColumn get senderId => text()();
  TextColumn get senderName => text().nullable()();
  TextColumn get type => text()(); // text, image, video, audio, file, system
  TextColumn get content => text()();
  TextColumn get status => text()(); // sending, sent, delivered, read, failed
  TextColumn get replyToId => text().nullable()();
  BoolColumn get isDeleted => boolean().withDefault(const Constant(false))();
  DateTimeColumn get createdAt => dateTime()();
  DateTimeColumn get editedAt => dateTime().nullable()();

  @override
  Set<Column> get primaryKey => {id};
}

/// Media attachments table
class MediaAttachments extends Table {
  TextColumn get id => text()();
  TextColumn get messageId => text().references(Messages, #id)();
  TextColumn get url => text()();
  TextColumn get thumbnailUrl => text().nullable()();
  TextColumn get type => text()(); // image, video, audio, file
  TextColumn get fileName => text().nullable()();
  IntColumn get fileSize => integer().nullable()();
  IntColumn get width => integer().nullable()();
  IntColumn get height => integer().nullable()();
  IntColumn get duration => integer().nullable()();

  @override
  Set<Column> get primaryKey => {id};
}

/// App database
@DriftDatabase(tables: [Users, Conversations, ConversationParticipants, Messages, MediaAttachments])
class AppDatabase extends _$AppDatabase {
  AppDatabase() : super(_openConnection());

  @override
  int get schemaVersion => 1;

  // User operations
  Future<List<User>> getAllUsers() => select(users).get();
  
  Future<User?> getUserById(String id) =>
      (select(users)..where((u) => u.id.equals(id))).getSingleOrNull();

  Future<int> insertUser(UsersCompanion user) => into(users).insert(user);
  
  Future<bool> updateUser(UsersCompanion user) =>
      update(users).replace(user);

  Future<int> deleteUser(String id) =>
      (delete(users)..where((u) => u.id.equals(id))).go();

  // Conversation operations
  Future<List<Conversation>> getAllConversations() =>
      (select(conversations)
        ..orderBy([(c) => OrderingTerm.desc(c.lastActivityAt)]))
      .get();

  Future<Conversation?> getConversationById(String id) =>
      (select(conversations)..where((c) => c.id.equals(id))).getSingleOrNull();

  Future<int> insertConversation(ConversationsCompanion conversation) =>
      into(conversations).insert(conversation);

  Future<bool> updateConversation(ConversationsCompanion conversation) =>
      update(conversations).replace(conversation);

  Future<int> deleteConversation(String id) =>
      (delete(conversations)..where((c) => c.id.equals(id))).go();

  Stream<List<Conversation>> watchConversations() =>
      (select(conversations)
        ..orderBy([(c) => OrderingTerm.desc(c.lastActivityAt)]))
      .watch();

  // Message operations
  Future<List<Message>> getMessagesByConversation(String conversationId, {int limit = 50, int offset = 0}) =>
      (select(messages)
        ..where((m) => m.conversationId.equals(conversationId))
        ..orderBy([(m) => OrderingTerm.desc(m.createdAt)])
        ..limit(limit, offset: offset))
      .get();

  Future<int> insertMessage(MessagesCompanion message) =>
      into(messages).insert(message);

  Future<bool> updateMessage(MessagesCompanion message) =>
      update(messages).replace(message);

  Future<int> deleteMessage(String id) =>
      (delete(messages)..where((m) => m.id.equals(id))).go();

  Stream<List<Message>> watchMessages(String conversationId) =>
      (select(messages)
        ..where((m) => m.conversationId.equals(conversationId))
        ..orderBy([(m) => OrderingTerm.desc(m.createdAt)]))
      .watch();

  // Clear all data
  Future<void> clearAll() async {
    await delete(mediaAttachments).go();
    await delete(messages).go();
    await delete(conversationParticipants).go();
    await delete(conversations).go();
    await delete(users).go();
  }
}

LazyDatabase _openConnection() {
  return LazyDatabase(() async {
    final dbFolder = await getApplicationDocumentsDirectory();
    final file = File(p.join(dbFolder.path, 'aiim.db'));
    return NativeDatabase.createInBackground(file);
  });
}
