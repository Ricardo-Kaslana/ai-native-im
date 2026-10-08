import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import '../../auth/models/user.dart';
import '../../contacts/services/contact_service.dart';
import '../../groups/models/group.dart';
import '../../groups/services/group_service.dart';
import '../../shared/widgets/user_avatar.dart';

/// 全局搜索页面
class SearchPage extends StatefulWidget {
  const SearchPage({super.key});

  @override
  State<SearchPage> createState() => _SearchPageState();
}

class _SearchPageState extends State<SearchPage> with SingleTickerProviderStateMixin {
  late TabController _tabController;
  final _searchController = TextEditingController();
  final _contactService = ContactService.instance;
  final _groupService = GroupService.instance;

  List<User> _users = [];
  List<Group> _groups = [];
  bool _isSearching = false;
  String _query = '';

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
    _searchController.dispose();
    super.dispose();
  }

  Future<void> _search(String query) async {
    if (query.trim().isEmpty) {
      setState(() {
        _users = [];
        _groups = [];
        _query = '';
      });
      return;
    }

    setState(() {
      _isSearching = true;
      _query = query;
    });

    try {
      final users = await _contactService.searchUsers(query);
      final groups = await _groupService.getGroups();
      final filteredGroups = groups.where(
        (g) => g.name.toLowerCase().contains(query.toLowerCase()),
      ).toList();

      if (mounted) {
        setState(() {
          _users = users;
          _groups = filteredGroups;
          _isSearching = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() => _isSearching = false);
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
        title: TextField(
          controller: _searchController,
          autofocus: true,
          decoration: InputDecoration(
            hintText: '搜索用户或群组',
            border: InputBorder.none,
            hintStyle: theme.textTheme.bodyLarge?.copyWith(
              color: theme.colorScheme.onSurface.withValues(alpha: 0.5),
            ),
          ),
          onChanged: (value) {
            if (value.length >= 2) {
              _search(value);
            } else if (value.isEmpty) {
              setState(() {
                _users = [];
                _groups = [];
              });
            }
          },
          onSubmitted: _search,
        ),
        actions: [
          if (_searchController.text.isNotEmpty)
            IconButton(
              icon: const Icon(Icons.clear),
              onPressed: () {
                _searchController.clear();
                setState(() {
                  _users = [];
                  _groups = [];
                  _query = '';
                });
              },
            ),
        ],
        bottom: TabBar(
          controller: _tabController,
          tabs: [
            Tab(text: '用户 (${_users.length})'),
            Tab(text: '群组 (${_groups.length})'),
          ],
        ),
      ),
      body: _isSearching
          ? const Center(child: CircularProgressIndicator())
          : _query.isEmpty
              ? _buildEmptyState(theme)
              : TabBarView(
                  controller: _tabController,
                  children: [
                    _buildUserList(theme),
                    _buildGroupList(theme),
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
            Icons.search,
            size: 64,
            color: theme.colorScheme.onSurface.withValues(alpha: 0.3),
          ),
          const SizedBox(height: 16),
          Text(
            '输入关键词搜索',
            style: theme.textTheme.bodyLarge?.copyWith(
              color: theme.colorScheme.onSurface.withValues(alpha: 0.5),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildUserList(ThemeData theme) {
    if (_users.isEmpty) {
      return Center(
        child: Text(
          '未找到相关用户',
          style: theme.textTheme.bodyLarge?.copyWith(
            color: theme.colorScheme.onSurface.withValues(alpha: 0.5),
          ),
        ),
      );
    }

    return ListView.builder(
      itemCount: _users.length,
      itemBuilder: (context, index) {
        final user = _users[index];
        return ListTile(
          leading: UserAvatar(
            name: user.displayName,
            avatarUrl: user.avatarUrl,
            size: 48,
          ),
          title: Text(user.displayName),
          subtitle: Text('@${user.username}'),
          onTap: () {
            // TODO: 查看用户资料或添加好友
          },
        );
      },
    );
  }

  Widget _buildGroupList(ThemeData theme) {
    if (_groups.isEmpty) {
      return Center(
        child: Text(
          '未找到相关群组',
          style: theme.textTheme.bodyLarge?.copyWith(
            color: theme.colorScheme.onSurface.withValues(alpha: 0.5),
          ),
        ),
      );
    }

    return ListView.builder(
      itemCount: _groups.length,
      itemBuilder: (context, index) {
        final group = _groups[index];
        return ListTile(
          leading: UserAvatar(
            name: group.name,
            avatarUrl: group.avatarUrl,
            size: 48,
          ),
          title: Text(group.name),
          subtitle: Text('${group.memberCount} 名成员'),
          onTap: () => context.push('/group/${group.id}'),
        );
      },
    );
  }
}
