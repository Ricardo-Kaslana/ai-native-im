import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import '../services/group_service.dart';
import '../../contacts/models/contact.dart';
import '../../contacts/services/contact_service.dart';
import '../../shared/widgets/user_avatar.dart';

/// 创建群组页面
class CreateGroupPage extends StatefulWidget {
  const CreateGroupPage({super.key});

  @override
  State<CreateGroupPage> createState() => _CreateGroupPageState();
}

class _CreateGroupPageState extends State<CreateGroupPage> {
  final _nameController = TextEditingController();
  final _groupService = GroupService.instance;
  final _contactService = ContactService.instance;

  List<Contact> _contacts = [];
  final Set<String> _selectedIds = {};
  bool _isLoading = true;
  bool _isCreating = false;
  bool _isPublic = false;

  @override
  void initState() {
    super.initState();
    _loadContacts();
  }

  @override
  void dispose() {
    _nameController.dispose();
    super.dispose();
  }

  Future<void> _loadContacts() async {
    setState(() => _isLoading = true);
    try {
      final contacts = await _contactService.getContacts();
      if (mounted) {
        setState(() {
          _contacts = contacts;
          _isLoading = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() => _isLoading = false);
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('加载联系人失败: $e')),
        );
      }
    }
  }

  Future<void> _createGroup() async {
    final name = _nameController.text.trim();
    if (name.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请输入群名称')),
      );
      return;
    }

    if (_selectedIds.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请选择至少一个成员')),
      );
      return;
    }

    setState(() => _isCreating = true);

    try {
      final group = await _groupService.createGroup(
        name: name,
        isPublic: _isPublic,
        memberIds: _selectedIds.toList(),
      );
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('群组创建成功')),
        );
        context.go('/chat/${group.id}?name=${Uri.encodeComponent(group.name)}&isGroup=true');
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('创建失败: $e')),
        );
      }
    } finally {
      if (mounted) {
        setState(() => _isCreating = false);
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
        title: const Text('创建群聊'),
        actions: [
          TextButton(
            onPressed: _isCreating ? null : _createGroup,
            child: _isCreating
                ? const SizedBox(
                    width: 20,
                    height: 20,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : Text('创建 (${_selectedIds.length})'),
          ),
        ],
      ),
      body: Column(
        children: [
          // 群名称
          Padding(
            padding: const EdgeInsets.all(16),
            child: TextField(
              controller: _nameController,
              decoration: const InputDecoration(
                labelText: '群名称',
                hintText: '请输入群名称',
              ),
            ),
          ),
          const Divider(height: 1),
          // 公开/私密设置
          SwitchListTile(
            title: const Text('公开群组'),
            subtitle: const Text('开启后其他用户可以通过搜索找到此群组'),
            value: _isPublic,
            onChanged: (value) {
              setState(() => _isPublic = value);
            },
          ),
          const Divider(height: 1),
          
          // 选择成员
          Padding(
            padding: const EdgeInsets.all(16),
            child: Row(
              children: [
                Text(
                  '选择成员',
                  style: theme.textTheme.titleMedium,
                ),
                const Spacer(),
                Text(
                  '已选择 ${_selectedIds.length} 人',
                  style: theme.textTheme.bodySmall,
                ),
              ],
            ),
          ),

          // 联系人列表
          Expanded(
            child: _isLoading
                ? const Center(child: CircularProgressIndicator())
                : _contacts.isEmpty
                    ? Center(
                        child: Text(
                          '暂无联系人',
                          style: theme.textTheme.bodyLarge?.copyWith(
                            color: theme.colorScheme.onSurface.withValues(alpha: 0.5),
                          ),
                        ),
                      )
                    : ListView.builder(
                        itemCount: _contacts.length,
                        itemBuilder: (context, index) {
                          final contact = _contacts[index];
                          final isSelected = _selectedIds.contains(contact.id);
                          
                          return CheckboxListTile(
                            value: isSelected,
                            onChanged: (value) {
                              setState(() {
                                if (value == true) {
                                  _selectedIds.add(contact.id);
                                } else {
                                  _selectedIds.remove(contact.id);
                                }
                              });
                            },
                            secondary: UserAvatar(
                              name: contact.displayName,
                              avatarUrl: contact.avatarUrl,
                              size: 40,
                            ),
                            title: Text(contact.displayName),
                          );
                        },
                      ),
          ),
        ],
      ),
    );
  }
}
