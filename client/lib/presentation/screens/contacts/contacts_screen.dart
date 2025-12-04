import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/constants/colors.dart';
import '../../../core/constants/spacing.dart';
import '../../../data/models/user.dart';
import '../../providers/contacts_provider.dart';
import '../../widgets/common/avatar.dart';

/// Contacts screen
class ContactsScreen extends ConsumerStatefulWidget {
  const ContactsScreen({super.key});

  @override
  ConsumerState<ContactsScreen> createState() => _ContactsScreenState();
}

class _ContactsScreenState extends ConsumerState<ContactsScreen> {
  final _searchController = TextEditingController();
  bool _isSearching = false;

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final groupedContactsAsync = ref.watch(groupedContactsProvider);
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: _isSearching
            ? TextField(
                controller: _searchController,
                autofocus: true,
                decoration: const InputDecoration(
                  hintText: '搜索联系人...',
                  border: InputBorder.none,
                ),
                onChanged: (value) {
                  ref.read(contactSearchQueryProvider.notifier).state = value;
                },
              )
            : const Text('联系人'),
        actions: [
          IconButton(
            icon: Icon(_isSearching ? Icons.close : Icons.search),
            onPressed: () {
              setState(() {
                _isSearching = !_isSearching;
                if (!_isSearching) {
                  _searchController.clear();
                  ref.read(contactSearchQueryProvider.notifier).state = '';
                }
              });
            },
          ),
          IconButton(
            icon: const Icon(Icons.person_add),
            onPressed: () {
              // TODO: Add contact
            },
          ),
        ],
      ),
      body: groupedContactsAsync.when(
        data: (groupedContacts) {
          if (groupedContacts.isEmpty) {
            return _buildEmptyState(context);
          }
          return ListView.builder(
            itemCount: groupedContacts.length,
            itemBuilder: (context, index) {
              final letter = groupedContacts.keys.elementAt(index);
              final contacts = groupedContacts[letter]!;
              return _buildSection(context, letter, contacts);
            },
          );
        },
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, stack) => Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(Icons.error_outline, size: 48, color: theme.colorScheme.error),
              const SizedBox(height: Spacing.md),
              const Text('加载失败'),
              const SizedBox(height: Spacing.md),
              ElevatedButton(
                onPressed: () => ref.invalidate(contactsProvider),
                child: const Text('重试'),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildSection(BuildContext context, String letter, List<User> contacts) {
    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Section header
        Container(
          padding: const EdgeInsets.symmetric(
            horizontal: Spacing.md,
            vertical: Spacing.xs,
          ),
          color: isDark ? AppColors.darkSurface : AppColors.lightSurface,
          child: Text(
            letter,
            style: theme.textTheme.labelMedium?.copyWith(
              color: theme.colorScheme.primary,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),
        // Contacts in section
        ...contacts.map((contact) => _buildContactItem(context, contact)),
      ],
    );
  }

  Widget _buildContactItem(BuildContext context, User contact) {
    final theme = Theme.of(context);

    return InkWell(
      onTap: () {
        // TODO: Navigate to chat or profile
      },
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: Spacing.md,
          vertical: Spacing.sm,
        ),
        child: Row(
          children: [
            // Avatar with online indicator
            Stack(
              children: [
                AppAvatar(
                  imageUrl: contact.avatarUrl,
                  name: contact.username,
                  size: Spacing.avatarMd,
                ),
                if (contact.isOnline)
                  Positioned(
                    right: 0,
                    bottom: 0,
                    child: Container(
                      width: 14,
                      height: 14,
                      decoration: BoxDecoration(
                        color: AppColors.online,
                        shape: BoxShape.circle,
                        border: Border.all(
                          color: theme.scaffoldBackgroundColor,
                          width: 2,
                        ),
                      ),
                    ),
                  ),
              ],
            ),
            const SizedBox(width: Spacing.md),
            // Name and status
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    contact.username,
                    style: theme.textTheme.bodyMedium?.copyWith(
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                  const SizedBox(height: Spacing.xxs),
                  Text(
                    contact.isOnline ? '在线' : _formatLastSeen(contact.lastSeenAt),
                    style: theme.textTheme.bodySmall?.copyWith(
                      color: contact.isOnline
                          ? AppColors.online
                          : theme.textTheme.bodySmall?.color,
                    ),
                  ),
                ],
              ),
            ),
            // Actions
            IconButton(
              icon: const Icon(Icons.chat_bubble_outline),
              onPressed: () {
                // TODO: Start chat
              },
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    final theme = Theme.of(context);
    final query = ref.watch(contactSearchQueryProvider);

    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            query.isEmpty ? Icons.people_outline : Icons.search_off,
            size: 80,
            color: theme.colorScheme.primary.withOpacity(0.5),
          ),
          const SizedBox(height: Spacing.lg),
          Text(
            query.isEmpty ? '暂无联系人' : '未找到匹配的联系人',
            style: theme.textTheme.bodyLarge,
          ),
          if (query.isEmpty) ...[
            const SizedBox(height: Spacing.xs),
            Text(
              '点击右上角添加新联系人',
              style: theme.textTheme.bodySmall,
            ),
          ],
        ],
      ),
    );
  }

  String _formatLastSeen(DateTime? lastSeen) {
    if (lastSeen == null) return '离线';

    final diff = DateTime.now().difference(lastSeen);
    if (diff.inMinutes < 1) return '刚刚在线';
    if (diff.inMinutes < 60) return '${diff.inMinutes}分钟前在线';
    if (diff.inHours < 24) return '${diff.inHours}小时前在线';
    if (diff.inDays < 7) return '${diff.inDays}天前在线';
    return '很久未上线';
  }
}
