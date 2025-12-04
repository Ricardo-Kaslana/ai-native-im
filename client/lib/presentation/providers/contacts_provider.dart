import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../data/models/user.dart';

/// Contacts provider with mock data
final contactsProvider = FutureProvider<List<User>>((ref) async {
  // Simulate network delay
  await Future.delayed(const Duration(milliseconds: 300));
  return _mockContacts;
});

/// Search query provider
final contactSearchQueryProvider = StateProvider<String>((ref) => '');

/// Filtered contacts provider
final filteredContactsProvider = Provider<AsyncValue<List<User>>>((ref) {
  final contactsAsync = ref.watch(contactsProvider);
  final query = ref.watch(contactSearchQueryProvider).toLowerCase();

  return contactsAsync.whenData((contacts) {
    if (query.isEmpty) return contacts;
    return contacts.where((user) {
      return user.username.toLowerCase().contains(query) ||
          (user.email?.toLowerCase().contains(query) ?? false);
    }).toList();
  });
});

/// Grouped contacts by first letter
final groupedContactsProvider = Provider<AsyncValue<Map<String, List<User>>>>((ref) {
  final filteredAsync = ref.watch(filteredContactsProvider);

  return filteredAsync.whenData((contacts) {
    final grouped = <String, List<User>>{};
    for (final contact in contacts) {
      final firstLetter = contact.username.isNotEmpty
          ? contact.username[0].toUpperCase()
          : '#';
      grouped.putIfAbsent(firstLetter, () => []).add(contact);
    }
    // Sort keys alphabetically
    final sortedKeys = grouped.keys.toList()..sort();
    return {for (final key in sortedKeys) key: grouped[key]!};
  });
});

/// Mock contacts data
final _mockContacts = [
  User(
    id: 'user_1',
    username: '张三',
    email: 'zhangsan@example.com',
    isOnline: true,
    createdAt: DateTime.now().subtract(const Duration(days: 30)),
  ),
  User(
    id: 'user_2',
    username: '李四',
    email: 'lisi@example.com',
    isOnline: true,
    createdAt: DateTime.now().subtract(const Duration(days: 20)),
  ),
  User(
    id: 'user_3',
    username: '王五',
    email: 'wangwu@example.com',
    isOnline: false,
    lastSeenAt: DateTime.now().subtract(const Duration(hours: 2)),
    createdAt: DateTime.now().subtract(const Duration(days: 15)),
  ),
  User(
    id: 'user_4',
    username: '赵六',
    email: 'zhaoliu@example.com',
    isOnline: false,
    lastSeenAt: DateTime.now().subtract(const Duration(days: 1)),
    createdAt: DateTime.now().subtract(const Duration(days: 10)),
  ),
  User(
    id: 'user_5',
    username: 'Alice',
    email: 'alice@example.com',
    isOnline: true,
    createdAt: DateTime.now().subtract(const Duration(days: 25)),
  ),
  User(
    id: 'user_6',
    username: 'Bob',
    email: 'bob@example.com',
    isOnline: false,
    lastSeenAt: DateTime.now().subtract(const Duration(minutes: 30)),
    createdAt: DateTime.now().subtract(const Duration(days: 18)),
  ),
  User(
    id: 'user_7',
    username: 'Charlie',
    email: 'charlie@example.com',
    isOnline: true,
    createdAt: DateTime.now().subtract(const Duration(days: 12)),
  ),
];
