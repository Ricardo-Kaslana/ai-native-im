import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import '../../auth/models/user.dart';
import '../services/contact_service.dart';
import '../../shared/widgets/user_avatar.dart';

/// 添加联系人页面
class AddContactPage extends StatefulWidget {
  const AddContactPage({super.key});

  @override
  State<AddContactPage> createState() => _AddContactPageState();
}

class _AddContactPageState extends State<AddContactPage> {
  final _searchController = TextEditingController();
  final _contactService = ContactService.instance;

  List<User> _searchResults = [];
  bool _isSearching = false;
  final Set<String> _addingIds = {};
  final Set<String> _addedIds = {};

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  Future<void> _search(String query) async {
    if (query.trim().isEmpty) {
      setState(() => _searchResults = []);
      return;
    }

    setState(() => _isSearching = true);

    try {
      final users = await _contactService.searchUsers(query);
      if (mounted) {
        setState(() {
          _searchResults = users;
          _isSearching = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() => _isSearching = false);
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('搜索失败: $e')),
        );
      }
    }
  }

  Future<void> _addContact(User user) async {
    if (_addingIds.contains(user.id) || _addedIds.contains(user.id)) return;

    setState(() => _addingIds.add(user.id));

    try {
      await _contactService.addContact(user.id);
      if (mounted) {
        setState(() {
          _addingIds.remove(user.id);
          _addedIds.add(user.id);
        });
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('已添加 ${user.displayName} 为好友')),
        );
      }
    } catch (e) {
      if (mounted) {
        setState(() => _addingIds.remove(user.id));
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('添加失败: $e')),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.pop(),
        ),
        title: const Text('添加好友'),
      ),
      body: Column(
        children: [
          // 搜索框
          Padding(
            padding: const EdgeInsets.all(16),
            child: TextField(
              controller: _searchController,
              decoration: InputDecoration(
                hintText: '搜索用户名或邮箱',
                prefixIcon: const Icon(Icons.search),
                suffixIcon: _searchController.text.isNotEmpty
                    ? IconButton(
                        icon: const Icon(Icons.clear),
                        onPressed: () {
                          _searchController.clear();
                          setState(() => _searchResults = []);
                        },
                      )
                    : null,
              ),
              onChanged: (value) {
                if (value.length >= 2) {
                  _search(value);
                }
              },
              onSubmitted: _search,
            ),
          ),

          // 搜索结果
          Expanded(
            child: _isSearching
                ? const Center(child: CircularProgressIndicator())
                : _searchResults.isEmpty
                    ? _buildEmptyState(theme)
                    : ListView.builder(
                        itemCount: _searchResults.length,
                        itemBuilder: (context, index) {
                          final user = _searchResults[index];
                          return _buildUserTile(user, theme);
                        },
                      ),
          ),
        ],
      ),
    );
  }

  Widget _buildEmptyState(ThemeData theme) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            Icons.person_search,
            size: 64,
            color: theme.colorScheme.onSurface.withValues(alpha: 0.3),
          ),
          const SizedBox(height: 16),
          Text(
            '搜索用户添加好友',
            style: theme.textTheme.bodyLarge?.copyWith(
              color: theme.colorScheme.onSurface.withValues(alpha: 0.5),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildUserTile(User user, ThemeData theme) {
    final isAdding = _addingIds.contains(user.id);
    final isAdded = _addedIds.contains(user.id);

    return ListTile(
      leading: UserAvatar(
        name: user.displayName,
        avatarUrl: user.avatarUrl,
        size: 48,
      ),
      title: Text(user.displayName),
      subtitle: Text('@${user.username}'),
      trailing: isAdded
          ? const Icon(Icons.check, color: Colors.green)
          : isAdding
              ? const SizedBox(
                  width: 24,
                  height: 24,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : OutlinedButton(
                  onPressed: () => _addContact(user),
                  child: const Text('添加'),
                ),
    );
  }
}
