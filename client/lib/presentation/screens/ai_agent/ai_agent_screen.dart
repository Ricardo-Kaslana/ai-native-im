import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../../core/constants/spacing.dart';
import '../../../data/models/ai_agent.dart';
import '../../providers/ai_agent_provider.dart';
import '../../widgets/common/app_button.dart';
import '../../widgets/common/app_input.dart';
import '../../widgets/common/loading_indicator.dart';
import '../../widgets/common/error_view.dart';

/// AI Agent management screen
class AIAgentScreen extends ConsumerStatefulWidget {
  final String groupId;
  final String groupName;

  const AIAgentScreen({
    super.key,
    required this.groupId,
    required this.groupName,
  });

  @override
  ConsumerState<AIAgentScreen> createState() => _AIAgentScreenState();
}

class _AIAgentScreenState extends ConsumerState<AIAgentScreen> {
  @override
  Widget build(BuildContext context) {
    final agentState = ref.watch(aiAgentProvider(widget.groupId));
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('AI 助手'),
      ),
      body: agentState.isLoading
          ? const Center(child: LoadingIndicator())
          : agentState.errorMessage != null
              ? ErrorView(
                  message: agentState.errorMessage!,
                  onRetry: () => ref
                      .read(aiAgentProvider(widget.groupId).notifier)
                      .loadAgentForGroup(widget.groupId),
                )
              : agentState.agent == null
                  ? _buildCreateAgentView(theme)
                  : _buildAgentDetailView(theme, agentState),
    );
  }

  Widget _buildCreateAgentView(ThemeData theme) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(Spacing.lg),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              width: 100,
              height: 100,
              decoration: BoxDecoration(
                color: theme.colorScheme.primary.withOpacity(0.1),
                shape: BoxShape.circle,
              ),
              child: Icon(
                Icons.smart_toy_outlined,
                size: 50,
                color: theme.colorScheme.primary,
              ),
            ),
            const SizedBox(height: Spacing.lg),
            Text(
              '为「${widget.groupName}」添加 AI 助手',
              style: theme.textTheme.titleLarge,
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: Spacing.sm),
            Text(
              'AI 助手可以参与群聊，回答问题，活跃气氛',
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.textTheme.bodySmall?.color,
              ),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: Spacing.xl),
            AppButton(
              text: '创建 AI 助手',
              icon: Icons.add,
              onPressed: () => _showCreateAgentDialog(),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildAgentDetailView(ThemeData theme, AIAgentState agentState) {
    final agent = agentState.agent!;
    final stats = agentState.stats;

    return SingleChildScrollView(
      padding: const EdgeInsets.all(Spacing.lg),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Agent header
          _buildAgentHeader(theme, agent),
          const SizedBox(height: Spacing.xl),

          // Personality section
          _buildSection(
            theme,
            '人设配置',
            Icons.psychology,
            _buildPersonalityCard(theme, agent.personality),
          ),
          const SizedBox(height: Spacing.lg),

          // Trigger keywords
          if (agent.triggerKeywords.isNotEmpty) ...[
            _buildSection(
              theme,
              '触发关键词',
              Icons.tag,
              Wrap(
                spacing: Spacing.xs,
                runSpacing: Spacing.xs,
                children: agent.triggerKeywords
                    .map((k) => Chip(label: Text(k)))
                    .toList(),
              ),
            ),
            const SizedBox(height: Spacing.lg),
          ],

          // Statistics
          if (stats != null) ...[
            _buildSection(
              theme,
              '统计数据',
              Icons.analytics,
              _buildStatsCard(theme, stats),
            ),
            const SizedBox(height: Spacing.lg),
          ],

          // Actions
          _buildActionsSection(theme),
        ],
      ),
    );
  }

  Widget _buildAgentHeader(ThemeData theme, AIAgent agent) {
    return Row(
      children: [
        CircleAvatar(
          radius: 40,
          backgroundColor: theme.colorScheme.primary.withOpacity(0.1),
          backgroundImage:
              agent.avatarUrl != null ? NetworkImage(agent.avatarUrl!) : null,
          child: agent.avatarUrl == null
              ? Icon(
                  Icons.smart_toy,
                  size: 40,
                  color: theme.colorScheme.primary,
                )
              : null,
        ),
        const SizedBox(width: Spacing.md),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                agent.name,
                style: theme.textTheme.headlineSmall,
              ),
              const SizedBox(height: Spacing.xxs),
              Row(
                children: [
                  Container(
                    width: 8,
                    height: 8,
                    decoration: BoxDecoration(
                      color: agent.isActive ? Colors.green : Colors.grey,
                      shape: BoxShape.circle,
                    ),
                  ),
                  const SizedBox(width: Spacing.xs),
                  Text(
                    agent.isActive ? '在线' : '离线',
                    style: theme.textTheme.bodySmall,
                  ),
                ],
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildSection(
    ThemeData theme,
    String title,
    IconData icon,
    Widget content,
  ) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Icon(icon, size: 20, color: theme.colorScheme.primary),
            const SizedBox(width: Spacing.xs),
            Text(title, style: theme.textTheme.titleMedium),
          ],
        ),
        const SizedBox(height: Spacing.sm),
        content,
      ],
    );
  }

  Widget _buildPersonalityCard(ThemeData theme, AIAgentPersonality personality) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(Spacing.md),
        child: Column(
          children: [
            _buildSliderRow(
              theme,
              '毒舌程度',
              personality.toxicityLevel,
              Icons.sentiment_very_dissatisfied,
              Icons.sentiment_very_satisfied,
            ),
            const SizedBox(height: Spacing.md),
            _buildSliderRow(
              theme,
              '热情度',
              personality.enthusiasmLevel,
              Icons.ac_unit,
              Icons.local_fire_department,
            ),
            if (personality.expertiseDomains.isNotEmpty) ...[
              const SizedBox(height: Spacing.md),
              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('专业领域: ', style: theme.textTheme.bodyMedium),
                  Expanded(
                    child: Wrap(
                      spacing: Spacing.xs,
                      children: personality.expertiseDomains
                          .map((d) => Chip(
                                label: Text(d),
                                visualDensity: VisualDensity.compact,
                              ))
                          .toList(),
                    ),
                  ),
                ],
              ),
            ],
            if (personality.speechSuffix != null) ...[
              const SizedBox(height: Spacing.sm),
              Row(
                children: [
                  Text('口癖: ', style: theme.textTheme.bodyMedium),
                  Text(
                    '「${personality.speechSuffix}」',
                    style: theme.textTheme.bodyMedium?.copyWith(
                      fontStyle: FontStyle.italic,
                    ),
                  ),
                ],
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildSliderRow(
    ThemeData theme,
    String label,
    int value,
    IconData lowIcon,
    IconData highIcon,
  ) {
    return Row(
      children: [
        SizedBox(
          width: 80,
          child: Text(label, style: theme.textTheme.bodyMedium),
        ),
        Icon(lowIcon, size: 16, color: theme.textTheme.bodySmall?.color),
        Expanded(
          child: Slider(
            value: value.toDouble(),
            min: 0,
            max: 10,
            divisions: 10,
            onChanged: null, // Read-only display
          ),
        ),
        Icon(highIcon, size: 16, color: theme.textTheme.bodySmall?.color),
        SizedBox(
          width: 24,
          child: Text(
            '$value',
            style: theme.textTheme.bodySmall,
            textAlign: TextAlign.center,
          ),
        ),
      ],
    );
  }

  Widget _buildStatsCard(ThemeData theme, AIAgentStats stats) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(Spacing.md),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.spaceAround,
          children: [
            _buildStatItem(theme, '总消息', stats.totalMessages.toString()),
            _buildStatItem(theme, '总回复', stats.totalResponses.toString()),
            _buildStatItem(
              theme,
              '平均响应',
              '${stats.averageResponseTime.toStringAsFixed(1)}s',
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatItem(ThemeData theme, String label, String value) {
    return Column(
      children: [
        Text(
          value,
          style: theme.textTheme.headlineSmall?.copyWith(
            color: theme.colorScheme.primary,
          ),
        ),
        Text(label, style: theme.textTheme.bodySmall),
      ],
    );
  }

  Widget _buildActionsSection(ThemeData theme) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        AppButton(
          text: '编辑人设',
          icon: Icons.edit,
          isOutlined: true,
          onPressed: () => _showEditPersonalityDialog(),
        ),
        const SizedBox(height: Spacing.sm),
        AppButton(
          text: '删除 AI 助手',
          icon: Icons.delete,
          isOutlined: true,
          onPressed: () => _showDeleteConfirmDialog(),
        ),
      ],
    );
  }

  void _showCreateAgentDialog() {
    showDialog(
      context: context,
      builder: (context) => _CreateAgentDialog(
        groupId: widget.groupId,
        onCreated: () {
          ref
              .read(aiAgentProvider(widget.groupId).notifier)
              .loadAgentForGroup(widget.groupId);
        },
      ),
    );
  }

  void _showEditPersonalityDialog() {
    final agent = ref.read(aiAgentProvider(widget.groupId)).agent;
    if (agent == null) return;

    showDialog(
      context: context,
      builder: (context) => _EditPersonalityDialog(
        groupId: widget.groupId,
        personality: agent.personality,
      ),
    );
  }

  void _showDeleteConfirmDialog() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('删除 AI 助手'),
        content: const Text('确定要删除这个 AI 助手吗？此操作不可撤销。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () async {
              Navigator.pop(context);
              final success = await ref
                  .read(aiAgentProvider(widget.groupId).notifier)
                  .deleteAgent();
              if (success && mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(content: Text('AI 助手已删除')),
                );
              }
            },
            child: const Text('删除', style: TextStyle(color: Colors.red)),
          ),
        ],
      ),
    );
  }
}

/// Dialog to create AI Agent
class _CreateAgentDialog extends ConsumerStatefulWidget {
  final String groupId;
  final VoidCallback onCreated;

  const _CreateAgentDialog({
    required this.groupId,
    required this.onCreated,
  });

  @override
  ConsumerState<_CreateAgentDialog> createState() => _CreateAgentDialogState();
}

class _CreateAgentDialogState extends ConsumerState<_CreateAgentDialog> {
  final _nameController = TextEditingController();
  final _speechSuffixController = TextEditingController();
  final _keywordsController = TextEditingController();
  int _toxicityLevel = 5;
  int _enthusiasmLevel = 5;
  final List<String> _expertiseDomains = [];
  bool _isLoading = false;

  @override
  void dispose() {
    _nameController.dispose();
    _speechSuffixController.dispose();
    _keywordsController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('创建 AI 助手'),
      content: SingleChildScrollView(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            AppInput(
              label: '名称',
              hint: '给 AI 助手起个名字',
              controller: _nameController,
            ),
            const SizedBox(height: Spacing.md),
            _buildSlider('毒舌程度', _toxicityLevel, (v) {
              setState(() => _toxicityLevel = v.round());
            }),
            _buildSlider('热情度', _enthusiasmLevel, (v) {
              setState(() => _enthusiasmLevel = v.round());
            }),
            const SizedBox(height: Spacing.sm),
            AppInput(
              label: '口癖（可选）',
              hint: '例如：喵~',
              controller: _speechSuffixController,
            ),
            const SizedBox(height: Spacing.md),
            AppInput(
              label: '触发关键词（可选）',
              hint: '用逗号分隔，例如：帮助,问题',
              controller: _keywordsController,
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('取消'),
        ),
        TextButton(
          onPressed: _isLoading ? null : _createAgent,
          child: _isLoading
              ? const SizedBox(
                  width: 16,
                  height: 16,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Text('创建'),
        ),
      ],
    );
  }

  Widget _buildSlider(String label, int value, ValueChanged<double> onChanged) {
    return Row(
      children: [
        SizedBox(width: 80, child: Text(label)),
        Expanded(
          child: Slider(
            value: value.toDouble(),
            min: 0,
            max: 10,
            divisions: 10,
            label: value.toString(),
            onChanged: onChanged,
          ),
        ),
        SizedBox(width: 24, child: Text('$value')),
      ],
    );
  }

  Future<void> _createAgent() async {
    if (_nameController.text.trim().isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请输入名称')),
      );
      return;
    }

    setState(() => _isLoading = true);

    final keywords = _keywordsController.text
        .split(',')
        .map((k) => k.trim())
        .where((k) => k.isNotEmpty)
        .toList();

    final request = CreateAgentRequest(
      name: _nameController.text.trim(),
      toxicityLevel: _toxicityLevel,
      enthusiasmLevel: _enthusiasmLevel,
      speechSuffix: _speechSuffixController.text.trim().isEmpty
          ? null
          : _speechSuffixController.text.trim(),
      triggerKeywords: keywords,
      expertiseDomains: _expertiseDomains,
    );

    final success = await ref
        .read(aiAgentProvider(widget.groupId).notifier)
        .createAgent(widget.groupId, request);

    if (mounted) {
      setState(() => _isLoading = false);
      if (success) {
        Navigator.pop(context);
        widget.onCreated();
      }
    }
  }
}

/// Dialog to edit AI Agent personality
class _EditPersonalityDialog extends ConsumerStatefulWidget {
  final String groupId;
  final AIAgentPersonality personality;

  const _EditPersonalityDialog({
    required this.groupId,
    required this.personality,
  });

  @override
  ConsumerState<_EditPersonalityDialog> createState() =>
      _EditPersonalityDialogState();
}

class _EditPersonalityDialogState
    extends ConsumerState<_EditPersonalityDialog> {
  late int _toxicityLevel;
  late int _enthusiasmLevel;
  late TextEditingController _speechSuffixController;
  bool _isLoading = false;

  @override
  void initState() {
    super.initState();
    _toxicityLevel = widget.personality.toxicityLevel;
    _enthusiasmLevel = widget.personality.enthusiasmLevel;
    _speechSuffixController =
        TextEditingController(text: widget.personality.speechSuffix ?? '');
  }

  @override
  void dispose() {
    _speechSuffixController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('编辑人设'),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          _buildSlider('毒舌程度', _toxicityLevel, (v) {
            setState(() => _toxicityLevel = v.round());
          }),
          _buildSlider('热情度', _enthusiasmLevel, (v) {
            setState(() => _enthusiasmLevel = v.round());
          }),
          const SizedBox(height: Spacing.sm),
          AppInput(
            label: '口癖',
            hint: '例如：喵~',
            controller: _speechSuffixController,
          ),
        ],
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('取消'),
        ),
        TextButton(
          onPressed: _isLoading ? null : _savePersonality,
          child: _isLoading
              ? const SizedBox(
                  width: 16,
                  height: 16,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Text('保存'),
        ),
      ],
    );
  }

  Widget _buildSlider(String label, int value, ValueChanged<double> onChanged) {
    return Row(
      children: [
        SizedBox(width: 80, child: Text(label)),
        Expanded(
          child: Slider(
            value: value.toDouble(),
            min: 0,
            max: 10,
            divisions: 10,
            label: value.toString(),
            onChanged: onChanged,
          ),
        ),
        SizedBox(width: 24, child: Text('$value')),
      ],
    );
  }

  Future<void> _savePersonality() async {
    setState(() => _isLoading = true);

    final newPersonality = widget.personality.copyWith(
      toxicityLevel: _toxicityLevel,
      enthusiasmLevel: _enthusiasmLevel,
      speechSuffix: _speechSuffixController.text.trim().isEmpty
          ? null
          : _speechSuffixController.text.trim(),
    );

    final success = await ref
        .read(aiAgentProvider(widget.groupId).notifier)
        .updatePersonality(newPersonality);

    if (mounted) {
      setState(() => _isLoading = false);
      if (success) {
        Navigator.pop(context);
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('人设已更新')),
        );
      }
    }
  }
}
