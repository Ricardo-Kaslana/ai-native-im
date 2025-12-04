import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/constants/spacing.dart';
import '../../../data/models/conversation.dart';
import '../../../data/models/message.dart';
import '../../../data/models/user.dart';
import '../../widgets/common/avatar.dart';

/// Search query provider
final searchQueryProvider = StateProvider<String>((ref) => '');

/// Search results provider
final searchResultsProvider = FutureProvider<SearchResults>((ref) async {
  final query = ref.watch(searchQueryProvider);
  if (query.isEmpty) return const SearchResults();

  await Future.delayed(const Duration(milliseconds: 300));

  // Mock search results
  return SearchResults(
    conversations: [
      Conversation(
        id: '1',
        type: ConversationType.private,
        name: '张三',
        createdAt: DateTime.now(),
      ),
    ],
    messages: [
      Message(
        id: 'msg_1',
        conversationId: '1',
        senderId: 'user_1',
        senderName: '张三',
        type: MessageType.text,
        content: '包含 "$query" 的消息内容',
        status: MessageStatus.read,
        createdAt: DateTime.now().subtract(const Duration(hours: 2)),
      ),
    ],
    contacts: [
      User(
        id: 'user_1',
        username: '张三',
        email: 'zhangsan@example.com',
        createdAt: DateTime.now(),
      ),
    ],
  );
});

/// Search results model
class SearchResults {
  final List<Conversation> conversations;
  final List<Message> messages;
  final List<User> contacts;

  const SearchResults({
    this.conversations = const [],
    this.messages = const [],
    this.contacts = const [],
  });

  bool get isEmpty => conversations.isEmpty && messages.isEmpty && contacts.isEmpty;
}

/// Search screen
class SearchScreen extends ConsumerStatefulWidget {
  const SearchScreen({super.key});

  @override
  ConsumerState<SearchScreen> createState() => _SearchScreenState();
}

class _SearchScreenState extends ConsumerState<SearchScreen> {
  final _searchController = TextEditingController();
  final _focusNode = FocusNode();

  @override
  void initState() {
    super.initState();
    _focusNode.requestFocus();
  }

  @override
  void dispose() {
    _searchController.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final query = ref.watch(searchQueryProvider);
    final resultsAsync = ref.watch(searchResultsProvider);

    return Scaffold(
      appBar: AppBar(
        titleSpacing: 0,
        title: TextField(
          controller: _searchController,
          focusNode: _focusNode,
          decoration: InputDecoration(
            hintText: '搜索消息、联系人、群组...',
            border: InputBorder.none,
            suffixIcon: query.isNotEmpty
                ? IconButton(
                    icon: const Icon(Icons.clear),
                    onPressed: () {
                      _searchController.clear();
                      ref.read(searchQueryProvider.notifier).state = '';
                    },
                  )
                : null,
          ),
          onChanged: (value) {
            ref.read(searchQueryProvider.notifier).state = value;
          },
        ),
      ),
      body: query.isEmpty
          ? _buildEmptyState(context)
          : resultsAsync.when(
              data: (results) {
                if (results.isEmpty) {
                  return _buildNoResults(context, query);
                }
                return _buildResults(context, results, query);
              },
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (e, _) => Center(child: Text('搜索失败: $e')),
            ),
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    final theme = Theme.of(context);
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            Icons.search,
            size: 64,
            color: theme.colorScheme.primary.withOpacity(0.3),
          ),
          const SizedBox(height: Spacing.md),
          Text(
            '输入关键词开始搜索',
            style: theme.textTheme.bodyMedium?.copyWith(
              color: theme.textTheme.bodySmall?.color,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildNoResults(BuildContext context, String query) {
    final theme = Theme.of(context);
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            Icons.search_off,
            size: 64,
            color: theme.colorScheme.primary.withOpacity(0.3),
          ),
          const SizedBox(height: Spacing.md),
          Text(
            '未找到 "$query" 相关结果',
            style: theme.textTheme.bodyMedium,
          ),
        ],
      ),
    );
  }

  Widget _buildResults(BuildContext context, SearchResults results, String query) {
    final theme = Theme.of(context);

    return ListView(
      children: [
        // Contacts section
        if (results.contacts.isNotEmpty) ...[
          _buildSectionHeader(context, '联系人', results.contacts.length),
          ...results.contacts.map((user) => ListTile(
                leading: AppAvatar(name: user.username, size: 40),
                title: _highlightText(user.username, query, theme),
                subtitle: user.email != null ? Text(user.email!) : null,
                onTap: () {
                  // TODO: Navigate to contact
                },
              )),
        ],

        // Conversations section
        if (results.conversations.isNotEmpty) ...[
          _buildSectionHeader(context, '会话', results.conversations.length),
          ...results.conversations.map((conv) => ListTile(
                leading: AppAvatar(
                  name: conv.displayName,
                  size: 40,
                  isGroup: conv.type == ConversationType.group,
                ),
                title: _highlightText(conv.displayName, query, theme),
                onTap: () {
                  // TODO: Navigate to conversation
                },
              )),
        ],

        // Messages section
        if (results.messages.isNotEmpty) ...[
          _buildSectionHeader(context, '消息', results.messages.length),
          ...results.messages.map((msg) => ListTile(
                leading: AppAvatar(name: msg.senderName ?? '?', size: 40),
                title: Text(msg.senderName ?? '未知'),
                subtitle: _highlightText(msg.content, query, theme),
                trailing: Text(
                  _formatTime(msg.createdAt),
                  style: theme.textTheme.labelSmall,
                ),
                onTap: () {
                  // TODO: Navigate to message
                },
              )),
        ],
      ],
    );
  }

  Widget _buildSectionHeader(BuildContext context, String title, int count) {
    final theme = Theme.of(context);
    return Container(
      padding: const EdgeInsets.symmetric(
        horizontal: Spacing.md,
        vertical: Spacing.sm,
      ),
      color: theme.colorScheme.surface,
      child: Row(
        children: [
          Text(
            title,
            style: theme.textTheme.labelMedium?.copyWith(
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(width: Spacing.xs),
          Text(
            '($count)',
            style: theme.textTheme.labelSmall,
          ),
        ],
      ),
    );
  }

  Widget _highlightText(String text, String query, ThemeData theme) {
    if (query.isEmpty) return Text(text);

    final lowerText = text.toLowerCase();
    final lowerQuery = query.toLowerCase();
    final index = lowerText.indexOf(lowerQuery);

    if (index < 0) return Text(text);

    return RichText(
      text: TextSpan(
        style: theme.textTheme.bodyMedium,
        children: [
          TextSpan(text: text.substring(0, index)),
          TextSpan(
            text: text.substring(index, index + query.length),
            style: TextStyle(
              backgroundColor: theme.colorScheme.primaryContainer,
              fontWeight: FontWeight.w600,
            ),
          ),
          TextSpan(text: text.substring(index + query.length)),
        ],
      ),
      maxLines: 2,
      overflow: TextOverflow.ellipsis,
    );
  }

  String _formatTime(DateTime time) {
    final now = DateTime.now();
    final diff = now.difference(time);

    if (diff.inMinutes < 60) return '${diff.inMinutes}分钟前';
    if (diff.inHours < 24) return '${diff.inHours}小时前';
    if (diff.inDays < 7) return '${diff.inDays}天前';
    return '${time.month}/${time.day}';
  }
}
