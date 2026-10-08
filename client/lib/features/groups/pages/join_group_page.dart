import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import '../services/group_service.dart';
import '../models/group.dart';
import '../../shared/widgets/user_avatar.dart';

/// 加入群聊页面
class JoinGroupPage extends StatefulWidget {
  const JoinGroupPage({super.key});

  @override
  State<JoinGroupPage> createState() => _JoinGroupPageState();
}

class _JoinGroupPageState extends State<JoinGroupPage> with SingleTickerProviderStateMixin {
  late TabController _tabController;
  final _service = GroupService.instance;
  
  // 邀请码
  final _codeController = TextEditingController();
  bool _isJoining = false;
  
  // 搜索
  final _searchController = TextEditingController();
  List<Group> _searchResults = [];
  bool _isSearching = false;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
    _codeController.dispose();
    _searchController.dispose();
    super.dispose();
  }

  Future<void> _joinByCode() async {
    final code = _codeController.text.trim();
    if (code.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请输入邀请码')),
      );
      return;
    }

    setState(() => _isJoining = true);

    try {
      final group = await _service.joinByInviteCode(code);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('成功加入群聊: ${group.name}')),
        );
        context.pop(true); // 返回并刷新列表
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('加入失败: $e')),
        );
      }
    } finally {
      if (mounted) {
        setState(() => _isJoining = false);
      }
    }
  }

  Future<void> _searchGroups() async {
    final query = _searchController.text.trim();
    if (query.isEmpty) {
      setState(() => _searchResults = []);
      return;
    }

    setState(() => _isSearching = true);

    try {
      final results = await _service.searchGroups(query);
      if (mounted) {
        setState(() => _searchResults = results);
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('搜索失败: $e')),
        );
      }
    } finally {
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
        title: const Text('加入群聊'),
        bottom: TabBar(
          controller: _tabController,
          tabs: const [
            Tab(text: '邀请码'),
            Tab(text: '搜索群组'),
          ],
        ),
      ),
      body: TabBarView(
        controller: _tabController,
        children: [
          _buildInviteCodeTab(theme),
          _buildSearchTab(theme),
        ],
      ),
    );
  }

  Widget _buildInviteCodeTab(ThemeData theme) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Icon(
            Icons.qr_code_2,
            size: 80,
            color: theme.colorScheme.primary.withValues(alpha: 0.6),
          ),
          const SizedBox(height: 24),
          Text(
            '输入邀请码加入群聊',
            style: theme.textTheme.titleLarge,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 8),
          Text(
            '请向群成员获取邀请码',
            style: theme.textTheme.bodyMedium?.copyWith(
              color: theme.colorScheme.onSurface.withValues(alpha: 0.6),
            ),
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 32),
          TextField(
            controller: _codeController,
            decoration: InputDecoration(
              labelText: '邀请码',
              hintText: '请输入邀请码',
              prefixIcon: const Icon(Icons.vpn_key_outlined),
              border: OutlineInputBorder(
                borderRadius: BorderRadius.circular(12),
              ),
            ),
            textCapitalization: TextCapitalization.characters,
            onSubmitted: (_) => _joinByCode(),
          ),
          const SizedBox(height: 24),
          FilledButton.icon(
            onPressed: _isJoining ? null : _joinByCode,
            icon: _isJoining
                ? const SizedBox(
                    width: 20,
                    height: 20,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Icon(Icons.group_add),
            label: Text(_isJoining ? '加入中...' : '加入群聊'),
          ),
        ],
      ),
    );
  }

  Widget _buildSearchTab(ThemeData theme) {
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: TextField(
            controller: _searchController,
            decoration: InputDecoration(
              hintText: '搜索群组名称',
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
              border: OutlineInputBorder(
                borderRadius: BorderRadius.circular(12),
              ),
            ),
            onChanged: (value) {
              if (value.length >= 2) {
                _searchGroups();
              } else {
                setState(() => _searchResults = []);
              }
            },
            onSubmitted: (_) => _searchGroups(),
          ),
        ),
        Expanded(
          child: _buildSearchResults(theme),
        ),
      ],
    );
  }

  Widget _buildSearchResults(ThemeData theme) {
    if (_isSearching) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_searchController.text.isEmpty) {
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
              '输入群组名称搜索',
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onSurface.withValues(alpha: 0.5),
              ),
            ),
          ],
        ),
      );
    }

    if (_searchResults.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.search_off,
              size: 64,
              color: theme.colorScheme.onSurface.withValues(alpha: 0.3),
            ),
            const SizedBox(height: 16),
            Text(
              '未找到相关群组',
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onSurface.withValues(alpha: 0.5),
              ),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      itemCount: _searchResults.length,
      itemBuilder: (context, index) {
        final group = _searchResults[index];
        return ListTile(
          leading: UserAvatar(
            name: group.name,
            avatarUrl: group.avatarUrl,
            size: 48,
          ),
          title: Text(group.name),
          subtitle: group.description != null
              ? Text(
                  group.description!,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                )
              : Text('${group.memberCount} 成员'),
          trailing: const Icon(Icons.chevron_right),
          onTap: () {
            // 显示群组详情或申请加入
            _showGroupPreview(group);
          },
        );
      },
    );
  }

  void _showGroupPreview(Group group) {
    showModalBottomSheet(
      context: context,
      builder: (context) => SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              UserAvatar(
                name: group.name,
                avatarUrl: group.avatarUrl,
                size: 72,
              ),
              const SizedBox(height: 16),
              Text(
                group.name,
                style: Theme.of(context).textTheme.titleLarge,
              ),
              if (group.description != null) ...[
                const SizedBox(height: 8),
                Text(
                  group.description!,
                  style: Theme.of(context).textTheme.bodyMedium,
                  textAlign: TextAlign.center,
                ),
              ],
              const SizedBox(height: 8),
              Text(
                '${group.memberCount} 成员',
                style: Theme.of(context).textTheme.bodySmall,
              ),
              const SizedBox(height: 24),
              Text(
                '此群组可能需要邀请码才能加入',
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.6),
                ),
              ),
              const SizedBox(height: 16),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                children: [
                  OutlinedButton(
                    onPressed: () => Navigator.pop(context),
                    child: const Text('取消'),
                  ),
                  FilledButton.icon(
                    onPressed: () {
                      Navigator.pop(context);
                      _tabController.animateTo(0); // 切换到邀请码 tab
                    },
                    icon: const Icon(Icons.vpn_key),
                    label: const Text('输入邀请码'),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
