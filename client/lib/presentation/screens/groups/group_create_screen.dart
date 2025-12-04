import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/constants/spacing.dart';
import '../../../data/models/user.dart';
import '../../providers/contacts_provider.dart';
import '../../providers/group_provider.dart';
import '../../widgets/common/app_button.dart';
import '../../widgets/common/app_input.dart';
import '../../widgets/common/avatar.dart';

/// Group creation screen
class GroupCreateScreen extends ConsumerStatefulWidget {
  const GroupCreateScreen({super.key});

  @override
  ConsumerState<GroupCreateScreen> createState() => _GroupCreateScreenState();
}

class _GroupCreateScreenState extends ConsumerState<GroupCreateScreen> {
  final _nameController = TextEditingController();
  final _descController = TextEditingController();
  int _currentStep = 0;

  @override
  void dispose() {
    _nameController.dispose();
    _descController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final creationState = ref.watch(groupCreationProvider);
    final selectedMembers = ref.watch(selectedMembersProvider);

    return Scaffold(
      appBar: AppBar(
        title: Text(_currentStep == 0 ? '选择成员' : '群组信息'),
        actions: [
          if (_currentStep == 0 && selectedMembers.isNotEmpty)
            TextButton(
              onPressed: () => setState(() => _currentStep = 1),
              child: Text('下一步 (${selectedMembers.length})'),
            ),
        ],
      ),
      body: _currentStep == 0
          ? _buildMemberSelection(context)
          : _buildGroupInfo(context, creationState, selectedMembers),
    );
  }

  Widget _buildMemberSelection(BuildContext context) {
    final contactsAsync = ref.watch(contactsProvider);
    final selectedMembers = ref.watch(selectedMembersProvider);
    final theme = Theme.of(context);

    return Column(
      children: [
        // Selected members preview
        if (selectedMembers.isNotEmpty)
          Container(
            height: 80,
            padding: const EdgeInsets.symmetric(vertical: Spacing.sm),
            child: ListView.builder(
              scrollDirection: Axis.horizontal,
              padding: const EdgeInsets.symmetric(horizontal: Spacing.md),
              itemCount: selectedMembers.length,
              itemBuilder: (context, index) {
                final user = selectedMembers[index];
                return Padding(
                  padding: const EdgeInsets.only(right: Spacing.sm),
                  child: GestureDetector(
                    onTap: () => _toggleMember(user),
                    child: Column(
                      children: [
                        Stack(
                          children: [
                            AppAvatar(name: user.username, size: 44),
                            Positioned(
                              right: -2,
                              top: -2,
                              child: Container(
                                padding: const EdgeInsets.all(2),
                                decoration: const BoxDecoration(
                                  color: Colors.red,
                                  shape: BoxShape.circle,
                                ),
                                child: const Icon(
                                  Icons.close,
                                  size: 12,
                                  color: Colors.white,
                                ),
                              ),
                            ),
                          ],
                        ),
                        const SizedBox(height: Spacing.xxs),
                        SizedBox(
                          width: 50,
                          child: Text(
                            user.username,
                            style: theme.textTheme.labelSmall,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            textAlign: TextAlign.center,
                          ),
                        ),
                      ],
                    ),
                  ),
                );
              },
            ),
          ),

        if (selectedMembers.isNotEmpty) const Divider(),

        // Contacts list
        Expanded(
          child: contactsAsync.when(
            data: (contacts) => ListView.builder(
              itemCount: contacts.length,
              itemBuilder: (context, index) {
                final user = contacts[index];
                final isSelected = selectedMembers.contains(user);
                return ListTile(
                  leading: AppAvatar(
                    imageUrl: user.avatarUrl,
                    name: user.username,
                    size: 40,
                  ),
                  title: Text(user.username),
                  trailing: Checkbox(
                    value: isSelected,
                    onChanged: (_) => _toggleMember(user),
                  ),
                  onTap: () => _toggleMember(user),
                );
              },
            ),
            loading: () => const Center(child: CircularProgressIndicator()),
            error: (e, _) => Center(child: Text('加载失败: $e')),
          ),
        ),
      ],
    );
  }

  Widget _buildGroupInfo(
    BuildContext context,
    GroupCreationState state,
    List<User> members,
  ) {
    final theme = Theme.of(context);

    return SingleChildScrollView(
      padding: const EdgeInsets.all(Spacing.lg),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Group avatar
          Center(
            child: GestureDetector(
              onTap: () {
                // TODO: Select group avatar
              },
              child: Stack(
                children: [
                  AppAvatar(
                    name: _nameController.text.isEmpty ? '群' : _nameController.text,
                    size: 80,
                    isGroup: true,
                  ),
                  Positioned(
                    right: 0,
                    bottom: 0,
                    child: Container(
                      padding: const EdgeInsets.all(Spacing.xxs),
                      decoration: BoxDecoration(
                        color: theme.colorScheme.primary,
                        shape: BoxShape.circle,
                      ),
                      child: const Icon(
                        Icons.camera_alt,
                        size: 16,
                        color: Colors.white,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: Spacing.xl),

          // Group name
          AppInput(
            label: '群组名称',
            hint: '请输入群组名称',
            controller: _nameController,
            errorText: state.error,
            onChanged: (value) {
              ref.read(groupCreationProvider.notifier).setName(value);
              setState(() {});
            },
          ),
          const SizedBox(height: Spacing.md),

          // Description
          AppInput(
            label: '群组简介（可选）',
            hint: '介绍一下这个群组',
            controller: _descController,
            maxLines: 3,
            onChanged: (value) {
              ref.read(groupCreationProvider.notifier).setDescription(value);
            },
          ),
          const SizedBox(height: Spacing.lg),

          // Public switch
          SwitchListTile(
            title: const Text('公开群组'),
            subtitle: const Text('任何人都可以搜索并加入'),
            value: state.isPublic,
            onChanged: (value) {
              ref.read(groupCreationProvider.notifier).setPublic(value);
            },
            contentPadding: EdgeInsets.zero,
          ),
          const Divider(),

          // Members preview
          Padding(
            padding: const EdgeInsets.symmetric(vertical: Spacing.md),
            child: Row(
              children: [
                Text(
                  '群成员 (${members.length + 1})',
                  style: theme.textTheme.labelMedium,
                ),
                const Spacer(),
                TextButton(
                  onPressed: () => setState(() => _currentStep = 0),
                  child: const Text('编辑'),
                ),
              ],
            ),
          ),
          Wrap(
            spacing: Spacing.sm,
            runSpacing: Spacing.sm,
            children: [
              // Current user (owner)
              _buildMemberChip(context, '我', isOwner: true),
              ...members.map((m) => _buildMemberChip(context, m.username)),
            ],
          ),

          const SizedBox(height: Spacing.xl),

          // Create button
          AppButton(
            text: '创建群组',
            isLoading: state.isLoading,
            onPressed: _createGroup,
            width: double.infinity,
          ),
        ],
      ),
    );
  }

  Widget _buildMemberChip(BuildContext context, String name, {bool isOwner = false}) {
    final theme = Theme.of(context);
    return Chip(
      avatar: AppAvatar(name: name, size: 24),
      label: Text(name),
      backgroundColor: isOwner ? theme.colorScheme.primaryContainer : null,
    );
  }

  void _toggleMember(User user) {
    final selected = ref.read(selectedMembersProvider);
    if (selected.contains(user)) {
      ref.read(selectedMembersProvider.notifier).state =
          selected.where((u) => u.id != user.id).toList();
    } else {
      ref.read(selectedMembersProvider.notifier).state = [...selected, user];
    }
  }

  Future<void> _createGroup() async {
    final members = ref.read(selectedMembersProvider);
    final group = await ref.read(groupCreationProvider.notifier).createGroup(members);

    if (group != null && mounted) {
      ref.read(groupCreationProvider.notifier).reset();
      ref.read(selectedMembersProvider.notifier).state = [];

      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('群组 "${group.name}" 创建成功')),
      );
      Navigator.of(context).pop(group);
    }
  }
}
