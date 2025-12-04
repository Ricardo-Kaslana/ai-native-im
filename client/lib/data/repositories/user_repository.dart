import 'package:drift/drift.dart';
import '../local/database.dart';
import '../models/user.dart' as model;

/// User repository for data access
class UserRepository {
  final AppDatabase _db;

  UserRepository(this._db);

  /// Get all users
  Future<List<model.User>> getAllUsers() async {
    final users = await _db.getAllUsers();
    return users.map(_mapToModel).toList();
  }

  /// Get user by ID
  Future<model.User?> getUserById(String id) async {
    final user = await _db.getUserById(id);
    return user != null ? _mapToModel(user) : null;
  }

  /// Insert or update user
  Future<void> upsertUser(model.User user) async {
    await _db.into(_db.users).insertOnConflictUpdate(
      UsersCompanion(
        id: Value(user.id),
        username: Value(user.username),
        email: Value(user.email),
        avatarUrl: Value(user.avatarUrl),
        bio: Value(user.bio),
        isOnline: Value(user.isOnline),
        lastSeenAt: Value(user.lastSeenAt),
        createdAt: Value(user.createdAt),
        updatedAt: Value(DateTime.now()),
      ),
    );
  }

  /// Delete user
  Future<void> deleteUser(String id) async {
    await _db.deleteUser(id);
  }

  /// Update online status
  Future<void> updateOnlineStatus(String id, bool isOnline) async {
    await (_db.update(_db.users)..where((u) => u.id.equals(id)))
        .write(UsersCompanion(
          isOnline: Value(isOnline),
          lastSeenAt: isOnline ? const Value.absent() : Value(DateTime.now()),
        ));
  }

  model.User _mapToModel(User user) {
    return model.User(
      id: user.id,
      username: user.username,
      email: user.email,
      avatarUrl: user.avatarUrl,
      bio: user.bio,
      isOnline: user.isOnline,
      lastSeenAt: user.lastSeenAt,
      createdAt: user.createdAt,
    );
  }
}
