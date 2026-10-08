import 'package:flutter/material.dart';
import 'emoji_data.dart';

/// 表情选择器
class EmojiPicker extends StatefulWidget {
  final void Function(String emoji) onEmojiSelected;
  final int columns; // 每行显示的 emoji 数量

  const EmojiPicker({
    super.key,
    required this.onEmojiSelected,
    this.columns = 8,
  });

  @override
  State<EmojiPicker> createState() => _EmojiPickerState();
}

class _EmojiPickerState extends State<EmojiPicker> with SingleTickerProviderStateMixin {
  late TabController _tabController;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(
      length: EmojiData.categories.length,
      vsync: this,
    );
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final categoryNames = EmojiData.categories.keys.toList();

    return Container(
      height: 280,
      decoration: BoxDecoration(
        color: theme.colorScheme.surface,
        border: Border(
          top: BorderSide(
            color: theme.colorScheme.outline.withValues(alpha: 0.2),
          ),
        ),
      ),
      child: Column(
        children: [
          // 类别标签栏
          TabBar(
            controller: _tabController,
            isScrollable: true,
            tabAlignment: TabAlignment.start,
            labelColor: theme.colorScheme.onSurface,
            unselectedLabelColor: theme.colorScheme.onSurface.withValues(alpha: 0.6),
            indicatorColor: theme.colorScheme.primary,
            tabs: categoryNames.map((name) => Tab(text: name)).toList(),
          ),
          // Emoji 网格
          Expanded(
            child: TabBarView(
              controller: _tabController,
              children: categoryNames.map((category) {
                final emojis = EmojiData.categories[category]!;
                return GridView.builder(
                  padding: const EdgeInsets.all(8),
                  gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                    crossAxisCount: widget.columns,
                    childAspectRatio: 1,
                    crossAxisSpacing: 4,
                    mainAxisSpacing: 4,
                  ),
                  itemCount: emojis.length,
                  itemBuilder: (context, index) {
                    final emoji = emojis[index];
                    return Material(
                      color: Colors.transparent,
                      child: InkWell(
                        onTap: () => widget.onEmojiSelected(emoji),
                        child: Center(
                          child: Text(
                            emoji,
                            style: const TextStyle(fontSize: 24),
                          ),
                        ),
                      ),
                    );
                  },
                );
              }).toList(),
            ),
          ),
        ],
      ),
    );
  }
}

/// 显示 Emoji 选择器的底部弹窗
Future<void> showEmojiPicker({
  required BuildContext context,
  required void Function(String emoji) onEmojiSelected,
  int columns = 8,
}) {
  return showModalBottomSheet<void>(
    context: context,
    backgroundColor: Colors.transparent,
    builder: (context) => EmojiPicker(
      onEmojiSelected: (emoji) {
        // 先执行回调，将 emoji 插入到输入框
        onEmojiSelected(emoji);
        // 使用 post frame callback 确保回调执行完成后再关闭
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (context.mounted) {
            Navigator.pop(context);
          }
        });
      },
      columns: columns,
    ),
  );
}
