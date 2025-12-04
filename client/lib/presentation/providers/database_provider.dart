import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../data/local/database.dart';
import '../../data/repositories/conversation_repository.dart';
import '../../data/repositories/message_repository.dart';
import '../../data/repositories/user_repository.dart';

/// Database provider (singleton)
final databaseProvider = Provider<AppDatabase>((ref) {
  final db = AppDatabase();
  ref.onDispose(() => db.close());
  return db;
});

/// Conversation repository provider
final conversationRepositoryProvider = Provider<ConversationRepository>((ref) {
  final db = ref.watch(databaseProvider);
  return ConversationRepository(db);
});

/// Message repository provider
final messageRepositoryProvider = Provider<MessageRepository>((ref) {
  final db = ref.watch(databaseProvider);
  return MessageRepository(db);
});

/// User repository provider
final userRepositoryProvider = Provider<UserRepository>((ref) {
  final db = ref.watch(databaseProvider);
  return UserRepository(db);
});
