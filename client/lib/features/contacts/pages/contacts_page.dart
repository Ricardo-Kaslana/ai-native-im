import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import '../models/contact.dart';
import '../services/contact_service.dart';
import '../../shared/widgets/user_avatar.dart';
import '../../conversations/services/conversation_service.dart';
import '../../groups/services/group_service.dart';
import '../../groups/models/group.dart';
import '../../chat/services/chat_service_v2.dart';

/// 联系人列表页面
class ContactsPage extends StatefulWidget {
  const ContactsPage({super.key});

  @override
  State<ContactsPage> createState() => _ContactsPageState();
}

class _ContactsPageState extends State<ContactsPage> with SingleTickerProviderStateMixin {
  late TabController _tabController;
  final _contactService = ContactService.instance;
  final _groupService = GroupService.instance;
  
  List<Contact> _contacts = [];
  List<Group> _groups = [];
  bool _isLoadingContacts = true;
  bool _isLoadingGroups = true;
  String? _contactsError;
  String? _groupsError;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
    _loadContacts();
    _loadGroups();
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  Future<void> _loadContacts() async {
    setState(() {
      _isLoadingContacts = true;
      _contactsError = null;
    });

    try {
      final contacts = await _contactService.getContacts();
      if (mounted) {
        setState(() {
          _contacts = contacts;
          _isLoadingContacts = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _contactsError = e.toString();
          _isLoadingContacts = false;
        });
      }
    }
  }

  Future<void> _loadGroups() async {
    setState(() {
      _isLoadingGroups = true;
      _groupsError = null;
    });

    try {
      final groups = await _groupService.getGroups();
      if (mounted) {
        setState(() {
          _groups = groups;
          _isLoadingGroups = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _groupsError = e.toString();
          _isLoadingGroups = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('通讯录'),
        actions: [
          IconButton(
            icon: const Icon(Icons.person_add_outlined),
            onPressed: () => context.push('/contacts/add'),
          ),
          IconButton(
            icon: const Icon(Icons.search),
            onPressed: () => context.push('/search'),
          ),
        ],
        bottom: TabBar(
          controller: _tabController,
          tabs: const [
            Tab(text: '好友'),
            Tab(text: '群聊'),
          ],
        ),
      ),
      body: TabBarView(
        controller: _tabController,
        children: [
          _buildContactsList(theme),
          _buildGroupsList(theme),
        ],
      ),
    );
  }

  Widget _buildContactsList(ThemeData theme) {
    if (_isLoadingContacts) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_contactsError != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.error_outline, size: 48, color: Colors.grey),
            const SizedBox(height: 16),
            Text(_contactsError!),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: _loadContacts,
              child: const Text('重试'),
            ),
          ],
        ),
      );
    }

    if (_contacts.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.people_outline,
              size: 64,
              color: theme.colorScheme.onSurface.withValues(alpha: 0.3),
            ),
            const SizedBox(height: 16),
            Text(
              '暂无好友',
              style: theme.textTheme.titleMedium?.copyWith(
                color: theme.colorScheme.onSurface.withValues(alpha: 0.5),
              ),
            ),
            const SizedBox(height: 16),
            ElevatedButton.icon(
              onPressed: () => context.push('/contacts/add'),
              icon: const Icon(Icons.person_add),
              label: const Text('添加好友'),
            ),
          ],
        ),
      );
    }

    return RefreshIndicator(
      onRefresh: _loadContacts,
      child: ListView.builder(
        itemCount: _contacts.length,
        itemBuilder: (context, index) {
          final contact = _contacts[index];
          return _buildContactTile(contact, theme);
        },
      ),
    );
  }

  Widget _buildContactTile(Contact contact, ThemeData theme) {
    return ListTile(
      leading: UserAvatar(
        name: contact.displayName,
        avatarUrl: contact.avatarUrl,
        size: 48,
      ),
      title: Text(
        contact.displayName,
        style: theme.textTheme.titleMedium,
      ),
      subtitle: contact.bio != null
          ? Text(
              contact.bio!,
              style: theme.textTheme.bodySmall,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            )
          : null,
      onTap: () async {
        // 创建或获取私聊对话并跳转
        try {
          final conversationService = ConversationService.instance;
          final conversation = await conversationService.createPrivateConversation(contact.id);
          if (mounted) {
            // 预加载私聊会话信息（避免首次发送消息时的额外API调用）
            // contact.id 就是对方的用户ID
            ChatServiceV2.instance.preloadPrivateConversationInfo(
              conversationId: conversation.id,
              receiverId: contact.id,
            );

            // 使用对话中的显示名称（优先）或联系人名称
            final displayName = conversation.displayName.isNotEmpty
                ? conversation.displayName
                : contact.displayName;
            context.push(
              '/chat/${conversation.id}?name=${Uri.encodeComponent(displayName)}&isGroup=false',
            );
          }
        } catch (e) {
          if (mounted) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(content: Text('打开对话失败: $e')),
            );
          }
        }
      },
    );
  }

  Widget _buildGroupsList(ThemeData theme) {
    if (_isLoadingGroups) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_groupsError != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.error_outline, size: 48, color: Colors.grey),
            const SizedBox(height: 16),
            Text(_groupsError!),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: _loadGroups,
              child: const Text('重试'),
            ),
          ],
        ),
      );
    }

    if (_groups.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.group_outlined,
              size: 64,
              color: theme.colorScheme.onSurface.withValues(alpha: 0.3),
            ),
            const SizedBox(height: 16),
            Text(
              '暂无群聊',
              style: theme.textTheme.titleMedium?.copyWith(
                color: theme.colorScheme.onSurface.withValues(alpha: 0.5),
              ),
            ),
            const SizedBox(height: 24),
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                OutlinedButton.icon(
                  onPressed: () async {
                    final result = await context.push('/group/join');
                    if (result == true) _loadGroups();
                  },
                  icon: const Icon(Icons.login),
                  label: const Text('加入群聊'),
                ),
                const SizedBox(width: 16),
                FilledButton.icon(
                  onPressed: () async {
                    final result = await context.push('/group/create');
                    if (result == true) _loadGroups();
                  },
                  icon: const Icon(Icons.group_add),
                  label: const Text('创建群聊'),
                ),
              ],
            ),
          ],
        ),
      );
    }

    return RefreshIndicator(
      onRefresh: _loadGroups,
      child: ListView.builder(
        itemCount: _groups.length,
        itemBuilder: (context, index) {
          final group = _groups[index];
          return _buildGroupTile(group, theme);
        },
      ),
    );
  }

  Widget _buildGroupTile(Group group, ThemeData theme) {
    return ListTile(
      leading: UserAvatar(
        name: group.name,
        avatarUrl: group.avatarUrl,
        size: 48,
      ),
      title: Text(
        group.name,
        style: theme.textTheme.titleMedium,
      ),
      subtitle: Text(
        group.description ?? '${group.memberCount} 成员',
        style: theme.textTheme.bodySmall,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
      trailing: const Icon(Icons.chevron_right),
      onTap: () => context.push(
        '/chat/${group.id}?name=${Uri.encodeComponent(group.name)}&isGroup=true',
      ),
    );
  }
}
