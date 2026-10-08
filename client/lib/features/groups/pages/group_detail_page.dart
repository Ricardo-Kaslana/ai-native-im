import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import '../models/group.dart';
import '../services/group_service.dart';
import '../../shared/widgets/user_avatar.dart';
import '../../ai_agent/models/ai_agent.dart';
import '../../ai_agent/models/persona_config.dart';
import '../../ai_agent/services/ai_agent_service.dart';

/// 群组详情页面
class GroupDetailPage extends StatefulWidget {
  final String groupId;

  const GroupDetailPage({super.key, required this.groupId});

  @override
  State<GroupDetailPage> createState() => _GroupDetailPageState();
}

class _GroupDetailPageState extends State<GroupDetailPage> {
  final _groupService = GroupService.instance;
  final _aiAgentService = AIAgentService.instance;

  Group? _group;
  List<GroupMember> _members = [];
  AIAgent? _aiAgent;
  bool _isLoading = true;
  bool _isLoadingAiAgent = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadData();
  }

  Future<void> _loadData() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    try {
      final group = await _groupService.getGroup(widget.groupId);
      final members = await _groupService.getMembers(widget.groupId);

      if (mounted) {
        setState(() {
          _group = group;
          _members = members;
          _isLoading = false;
        });
      }

      // 加载 AI Agent（如果存在）
      _loadAiAgent();
    } catch (e) {
      if (mounted) {
        setState(() {
          _error = e.toString();
          _isLoading = false;
        });
      }
    }
  }

  Future<void> _loadAiAgent() async {
    setState(() {
      _isLoadingAiAgent = true;
    });

    try {
      final agent = await _aiAgentService.getAgentForGroup(widget.groupId);
      if (mounted) {
        setState(() {
          _aiAgent = agent;
          _isLoadingAiAgent = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _isLoadingAiAgent = false;
        });
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
        title: const Text('群组详情'),
        actions: [
          PopupMenuButton<String>(
            onSelected: _handleMenuAction,
            itemBuilder: (context) => [
              const PopupMenuItem(value: 'ai_agent', child: Text('AI 助手')),
              const PopupMenuDivider(),
              const PopupMenuItem(value: 'invite', child: Text('邀请成员')),
              const PopupMenuItem(value: 'code', child: Text('邀请码')),
              const PopupMenuItem(value: 'leave', child: Text('退出群组')),
            ],
          ),
        ],
      ),
      body: _buildBody(theme),
    );
  }

  Widget _buildBody(ThemeData theme) {
    if (_isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_error != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.error_outline, size: 48, color: Colors.grey),
            const SizedBox(height: 16),
            Text(_error!),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: _loadData,
              child: const Text('重试'),
            ),
          ],
        ),
      );
    }

    return ListView(
      children: [
        // 群组信息
        Container(
          padding: const EdgeInsets.all(20),
          child: Column(
            children: [
              UserAvatar(
                name: _group!.name,
                avatarUrl: _group!.avatarUrl,
                size: 80,
              ),
              const SizedBox(height: 16),
              Text(
                _group!.name,
                style: theme.textTheme.headlineSmall,
              ),
              if (_group!.description != null) ...[
                const SizedBox(height: 8),
                Text(
                  _group!.description!,
                  style: theme.textTheme.bodyMedium?.copyWith(
                    color: theme.colorScheme.onSurface.withValues(alpha: 0.6),
                  ),
                  textAlign: TextAlign.center,
                ),
              ],
              const SizedBox(height: 8),
              Text(
                '${_group!.memberCount} 名成员',
                style: theme.textTheme.bodySmall,
              ),
            ],
          ),
        ),
        const Divider(height: 1),

        // AI Agent 区域
        _buildAiAgentSection(theme),
        const Divider(height: 1),

        // 成员列表标题
        Padding(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              Text(
                '群成员',
                style: theme.textTheme.titleMedium,
              ),
              const Spacer(),
              TextButton.icon(
                onPressed: () {},
                icon: const Icon(Icons.person_add, size: 18),
                label: const Text('添加'),
              ),
            ],
          ),
        ),

        // 成员列表
        ..._members.map((member) => ListTile(
          leading: UserAvatar(
            name: member.displayName,
            avatarUrl: member.avatarUrl,
            size: 40,
          ),
          title: Text(member.displayName),
          subtitle: member.isOwner
              ? Text('群主', style: TextStyle(color: theme.colorScheme.primary))
              : member.isAdmin
                  ? const Text('管理员')
                  : null,
          trailing: member.isOwner ? null : PopupMenuButton<String>(
            onSelected: (action) => _handleMemberAction(action, member),
            itemBuilder: (context) => [
              const PopupMenuItem(value: 'chat', child: Text('发消息')),
              const PopupMenuItem(value: 'remove', child: Text('移出群聊')),
            ],
          ),
        )),
      ],
    );
  }

  void _handleMenuAction(String action) async {
    switch (action) {
      case 'ai_agent':
        _showAiAgentDialog();
        break;
      case 'invite':
        // TODO: 跳转到邀请成员页面
        break;
      case 'code':
        _showInviteCodeDialog();
        break;
      case 'leave':
        _confirmLeaveGroup();
        break;
    }
  }

  void _handleMemberAction(String action, GroupMember member) async {
    switch (action) {
      case 'chat':
        // TODO: 跳转到私聊
        break;
      case 'remove':
        _confirmRemoveMember(member);
        break;
    }
  }

  void _showInviteCodeDialog() async {
    try {
      final code = await _groupService.generateInviteCode(widget.groupId);
      if (mounted) {
        showDialog(
          context: context,
          builder: (context) => AlertDialog(
            title: const Text('邀请码'),
            content: SelectableText(
              code,
              style: const TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context),
                child: const Text('关闭'),
              ),
            ],
          ),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('获取邀请码失败: $e')),
        );
      }
    }
  }

  void _confirmLeaveGroup() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('退出群组'),
        content: const Text('确定要退出该群组吗？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () async {
              Navigator.pop(context);
              try {
                await _groupService.leaveGroup(widget.groupId);
                if (mounted) {
                  if (context.mounted) {
                    context.go('/');
                  }
                }
              } catch (e) {
                if (mounted && context.mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(content: Text('退出失败: $e')),
                  );
                }
              }
            },
            child: const Text('确定'),
          ),
        ],
      ),
    );
  }

  void _confirmRemoveMember(GroupMember member) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('移出群聊'),
        content: Text('确定要将 ${member.displayName} 移出群聊吗？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () async {
              Navigator.pop(context);
              try {
                await _groupService.removeMember(widget.groupId, member.id);
                _loadData();
              } catch (e) {
                if (mounted && context.mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(content: Text('移除失败: $e')),
                  );
                }
              }
            },
            child: const Text('确定'),
          ),
        ],
      ),
    );
  }

  // ==================== AI Agent 相关 ====================

  Widget _buildAiAgentSection(ThemeData theme) {
    if (_isLoadingAiAgent) {
      return const Padding(
        padding: EdgeInsets.all(16),
        child: Center(child: CircularProgressIndicator()),
      );
    }

    if (_aiAgent == null) {
      // 没有创建 AI Agent
      return ListTile(
        leading: const Icon(Icons.smart_toy_outlined),
        title: const Text('添加 AI 助手'),
        subtitle: const Text('为群组添加智能助手，自动响应消息'),
        trailing: const Icon(Icons.add_circle_outline),
        onTap: _showCreateAiAgentDialog,
      );
    }

    // 显示 AI Agent 信息
    return ListTile(
      leading: CircleAvatar(
        backgroundColor: theme.colorScheme.primaryContainer,
        child: Icon(
          Icons.smart_toy,
          color: theme.colorScheme.primary,
        ),
      ),
      title: Text(_aiAgent!.name),
      subtitle: Text(
        '毒舌度: ${_aiAgent!.toxicityLevel}/10 | 热情度: ${_aiAgent!.enthusiasmLevel}/10'
        '${_aiAgent!.speechSuffix != null ? " | 后缀: ${_aiAgent!.speechSuffix}" : ""}',
      ),
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            _aiAgent!.isActive ? Icons.check_circle : Icons.cancel,
            color: _aiAgent!.isActive ? Colors.green : Colors.grey,
            size: 20,
          ),
          const SizedBox(width: 8),
          const Icon(Icons.chevron_right),
        ],
      ),
      onTap: _showAiAgentDialog,
    );
  }

  void _showCreateAiAgentDialog() {
    final nameController = TextEditingController(text: '小助手');
    int toxicityLevel = 3;
    int enthusiasmLevel = 7;
    String? speechSuffix;
    final expertiseController = TextEditingController();

    showDialog(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) {
          final theme = Theme.of(context);
          return AlertDialog(
            title: const Text('创建 AI 助手'),
            content: SingleChildScrollView(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  TextField(
                    controller: nameController,
                    decoration: const InputDecoration(
                      labelText: '名称',
                      hintText: '例如：小助手',
                    ),
                  ),
                  const SizedBox(height: 16),
                  Text('毒舌度: $toxicityLevel/10',
                      style: theme.textTheme.bodyMedium),
                  Slider(
                    value: toxicityLevel.toDouble(),
                    min: 0,
                    max: 10,
                    divisions: 10,
                    label: '$toxicityLevel',
                    onChanged: (value) {
                      setDialogState(() {
                        toxicityLevel = value.round();
                      });
                    },
                  ),
                  const SizedBox(height: 8),
                  Text('热情度: $enthusiasmLevel/10',
                      style: theme.textTheme.bodyMedium),
                  Slider(
                    value: enthusiasmLevel.toDouble(),
                    min: 0,
                    max: 10,
                    divisions: 10,
                    label: '$enthusiasmLevel',
                    onChanged: (value) {
                      setDialogState(() {
                        enthusiasmLevel = value.round();
                      });
                    },
                  ),
                  const SizedBox(height: 16),
                  TextField(
                    controller: expertiseController,
                    decoration: const InputDecoration(
                      labelText: '专业领域（逗号分隔）',
                      hintText: '例如：聊天, 游戏, 技术',
                    ),
                  ),
                  const SizedBox(height: 16),
                  TextField(
                    decoration: InputDecoration(
                      labelText: '说话后缀',
                      hintText: '例如：~, 喵~',
                      suffixText: speechSuffix ?? '无',
                    ),
                    onChanged: (value) {
                      setDialogState(() {
                        speechSuffix = value.isEmpty ? null : value;
                      });
                    },
                  ),
                ],
              ),
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context),
                child: const Text('取消'),
              ),
              ElevatedButton(
                onPressed: () async {
                  final request = CreateAIAgent(
                    name: nameController.text.trim(),
                    toxicityLevel: toxicityLevel,
                    enthusiasmLevel: enthusiasmLevel,
                    speechSuffix: speechSuffix,
                    expertiseDomains: expertiseController.text
                        .split(',')
                        .map((e) => e.trim())
                        .where((e) => e.isNotEmpty)
                        .toList(),
                  );

                  Navigator.pop(context);

                  try {
                    await _aiAgentService.createAgent(widget.groupId, request);
                    _loadAiAgent();
                    if (mounted && context.mounted) {
                      ScaffoldMessenger.of(context).showSnackBar(
                        const SnackBar(content: Text('AI 助手创建成功')),
                      );
                    }
                  } catch (e) {
                    if (mounted && context.mounted) {
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(content: Text('创建失败: $e')),
                      );
                    }
                  }
                },
                child: const Text('创建'),
              ),
            ],
          );
        },
      ),
    );
  }

  void _showAiAgentDialog() {
    if (_aiAgent == null) {
      _showCreateAiAgentDialog();
      return;
    }

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(_aiAgent!.name),
        content: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _buildInfoRow('ID', '${_aiAgent!.id.substring(0, 8)}...'),
              _buildInfoRow('毒舌度', '${_aiAgent!.toxicityLevel}/10'),
              _buildInfoRow('热情度', '${_aiAgent!.enthusiasmLevel}/10'),
              if (_aiAgent!.speechSuffix != null)
                _buildInfoRow('说话后缀', _aiAgent!.speechSuffix!),
              if (_aiAgent!.expertiseDomains.isNotEmpty)
                _buildInfoRow('专业领域', _aiAgent!.expertiseDomains.join(', ')),
              if (_aiAgent!.triggerKeywords.isNotEmpty)
                _buildInfoRow('触发关键词', _aiAgent!.triggerKeywords.join(', ')),
              const Divider(),
              Text('状态: ${_aiAgent!.isActive ? "启用" : "禁用"}'),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('关闭'),
          ),
          TextButton(
            onPressed: () async {
              Navigator.pop(context);
              _showAiAgentMenu();
            },
            child: const Text('更多'),
          ),
          TextButton(
            onPressed: () async {
              Navigator.pop(context);
              _confirmDeleteAiAgent();
            },
            child: const Text('删除', style: TextStyle(color: Colors.red)),
          ),
        ],
      ),
    );
  }

  /// 显示 AI Agent 功能菜单
  void _showAiAgentMenu() {
    showModalBottomSheet(
      context: context,
      builder: (context) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(Icons.edit),
              title: const Text('编辑人格'),
              onTap: () {
                Navigator.pop(context);
                _showEditAiAgentDialog();
              },
            ),
            ListTile(
              leading: const Icon(Icons.bar_chart),
              title: const Text('统计信息'),
              onTap: () {
                Navigator.pop(context);
                _showAgentStats();
              },
            ),
            ListTile(
              leading: const Icon(Icons.psychology),
              title: const Text('人设预设'),
              onTap: () {
                Navigator.pop(context);
                _showPersonaPresets();
              },
            ),
            ListTile(
              leading: const Icon(Icons.science),
              title: const Text('测试人设'),
              onTap: () {
                Navigator.pop(context);
                _showTestPersona();
              },
            ),
            ListTile(
              leading: const Icon(Icons.summarize),
              title: const Text('消息总结'),
              onTap: () {
                Navigator.pop(context);
                _showSummaryOptions();
              },
            ),
            ListTile(
              leading: const Icon(Icons.person),
              title: const Text('用户档案'),
              onTap: () {
                Navigator.pop(context);
                _showUserProfiles();
              },
            ),
          ],
        ),
      ),
    );
  }

  /// 显示统计信息
  Future<void> _showAgentStats() async {
    if (_aiAgent == null) return;

    try {
      final stats = await _aiAgentService.getStatistics(_aiAgent!.id);
      if (!mounted) return;

      showDialog(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('统计信息'),
          content: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _buildInfoRow('总消息数', '${stats.totalMessages}'),
                if (stats.lastMessageAt != null)
                  _buildInfoRow(
                    '最后消息',
                    _formatDateTime(stats.lastMessageAt!),
                  ),
                if (stats.avgResponseTimeMs != null)
                  _buildInfoRow(
                    '平均响应时间',
                    '${stats.avgResponseTimeMs!}ms',
                  ),
                const Divider(),
                const Text('触发统计', style: TextStyle(fontWeight: FontWeight.bold)),
                const SizedBox(height: 8),
                _buildInfoRow('@提及', '${stats.triggerCounts.mention}'),
                _buildInfoRow('关键词', '${stats.triggerCounts.keyword}'),
                _buildInfoRow('上下文相似', '${stats.triggerCounts.contextSimilarity}'),
                _buildInfoRow('随机', '${stats.triggerCounts.random}'),
                _buildInfoRow('破冰', '${stats.triggerCounts.silenceBreaker}'),
                const Divider(),
                _buildInfoRow('总评估', '${stats.triggerCounts.totalEvaluations}'),
                _buildInfoRow('总响应', '${stats.triggerCounts.totalResponses}'),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('关闭'),
            ),
          ],
        ),
      );
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('获取统计信息失败: $e')),
        );
      }
    }
  }

  /// 显示人设预设
  Future<void> _showPersonaPresets() async {
    try {
      final presets = await _aiAgentService.getPersonaPresets();
      if (!mounted) return;

      showDialog(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('人设预设'),
          content: SizedBox(
            width: double.maxFinite,
            child: ListView.builder(
              shrinkWrap: true,
              itemCount: presets.length,
              itemBuilder: (context, index) {
                final preset = presets[index];
                return ListTile(
                  title: Text(preset.name),
                  subtitle: Text(preset.description),
                  trailing: OutlinedButton(
                    onPressed: () => _applyPreset(preset.id),
                    child: const Text('应用'),
                  ),
                  onTap: () => _showPresetDetails(preset),
                );
              },
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('关闭'),
            ),
          ],
        ),
      );
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('获取预设失败: $e')),
        );
      }
    }
  }

  /// 显示预设详情
  void _showPresetDetails(PersonaPresetInfo preset) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(preset.name),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(preset.description),
            const SizedBox(height: 16),
            _buildInfoRow('毒舌度', '${preset.toxicityLevel}/10'),
            _buildInfoRow('热情度', '${preset.enthusiasmLevel}/10'),
            if (preset.speechSuffix != null)
              _buildInfoRow('说话后缀', preset.speechSuffix!),
            const SizedBox(height: 16),
            const Text('预览问候:', style: TextStyle(fontWeight: FontWeight.bold)),
            const SizedBox(height: 8),
            Text(preset.previewGreeting),
            if (preset.expertiseHint.isNotEmpty) ...[
              const SizedBox(height: 16),
              const Text('专业提示:', style: TextStyle(fontWeight: FontWeight.bold)),
              const SizedBox(height: 8),
              Text(preset.expertiseHint),
            ],
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('关闭'),
          ),
          ElevatedButton(
            onPressed: () {
              Navigator.pop(context);
              _applyPreset(preset.id);
            },
            child: const Text('应用此预设'),
          ),
        ],
      ),
    );
  }

  /// 应用人设预设
  Future<void> _applyPreset(String presetId) async {
    if (_aiAgent == null) return;

    try {
      final request = UpdatePersonality(preset: presetId);
      await _aiAgentService.updatePersonality(_aiAgent!.id, request);
      await _loadAiAgent();
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('预设已应用')),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('应用预设失败: $e')),
        );
      }
    }
  }

  /// 测试人设
  void _showTestPersona() {
    if (_aiAgent == null) return;

    final messageController = TextEditingController();
    final nameController = TextEditingController(text: '用户');

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('测试人设'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: nameController,
              decoration: const InputDecoration(
                labelText: '发送者名称',
                hintText: '例如：小明',
              ),
            ),
            const SizedBox(height: 16),
            TextField(
              controller: messageController,
              decoration: const InputDecoration(
                labelText: '测试消息',
                hintText: '例如：你好，在吗？',
              ),
              maxLines: 3,
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('取消'),
          ),
          ElevatedButton(
            onPressed: () async {
              if (messageController.text.trim().isEmpty) {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(content: Text('请输入测试消息')),
                );
                return;
              }

              final request = TestPersonaRequest(
                message: messageController.text.trim(),
                senderName: nameController.text.trim().isEmpty
                    ? null
                    : nameController.text.trim(),
              );

              try {
                final response = await _aiAgentService.testPersona(
                  _aiAgent!.id,
                  request,
                );
                if (!mounted) return;

                if (context.mounted) {
                  Navigator.pop(context);
                  _showTestResponse(response);
                }
              } catch (e) {
                if (mounted && context.mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(content: Text('测试失败: $e')),
                  );
                }
              }
            },
            child: const Text('发送'),
          ),
        ],
      ),
    );
  }

  /// 显示测试响应
  void _showTestResponse(TestPersonaResponse response) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('AI 响应'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (response.emotion.isNotEmpty)
              _buildInfoRow('情感', response.emotion),
            _buildInfoRow(
              '处理时间',
              '${response.processingTimeMs}ms',
            ),
            const Divider(),
            const SizedBox(height: 8),
            Text(response.response),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('关闭'),
          ),
        ],
      ),
    );
  }

  /// 显示总结选项
  void _showSummaryOptions() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('消息总结'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              title: const Text('简短总结'),
              subtitle: const Text('快速获取对话要点'),
              onTap: () => _generateSummary(SummaryStyle.brief),
            ),
            ListTile(
              title: const Text('详细总结'),
              subtitle: const Text('完整的对话摘要'),
              onTap: () => _generateSummary(SummaryStyle.detailed),
            ),
            ListTile(
              title: const Text('话题列表'),
              subtitle: const Text('提取讨论的话题'),
              onTap: () => _generateSummary(SummaryStyle.topics),
            ),
            ListTile(
              title: const Text('行动项'),
              subtitle: const Text('提取待办事项'),
              onTap: () => _generateSummary(SummaryStyle.actionItems),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('取消'),
          ),
        ],
      ),
    );
  }

  /// 生成总结
  Future<void> _generateSummary(SummaryStyle style) async {
    Navigator.pop(context);

    try {
      final summary = await _aiAgentService.generateSummary(
        widget.groupId,
        style: style,
      );
      if (!mounted) return;

      showDialog(
        context: context,
        builder: (context) => AlertDialog(
          title: Text('${style.displayName}总结'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _buildInfoRow('消息数', '${summary.messageCount}'),
              _buildInfoRow(
                '生成时间',
                _formatDateTime(summary.generatedAt),
              ),
              const Divider(),
              const SizedBox(height: 8),
              Text(summary.content),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('关闭'),
            ),
          ],
        ),
      );
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('生成总结失败: $e')),
        );
      }
    }
  }

  /// 显示用户档案列表
  Future<void> _showUserProfiles() async {
    if (_aiAgent == null) return;

    try {
      final profiles = await _aiAgentService.listUserProfiles(_aiAgent!.id);
      if (!mounted) return;

      showDialog(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('用户档案'),
          content: SizedBox(
            width: double.maxFinite,
            child: profiles.isEmpty
                ? const Center(
                    child: Padding(
                      padding: EdgeInsets.all(32),
                      child: Text('暂无用户档案'),
                    ),
                  )
                : ListView.builder(
                    shrinkWrap: true,
                    itemCount: profiles.length,
                    itemBuilder: (context, index) {
                      final profile = profiles[index];
                      return ListTile(
                        title: Text(profile.nickname ?? '用户 ${profile.userId.substring(0, 8)}'),
                        subtitle: Text('亲和度: ${profile.affinityScore}'),
                        trailing: const Icon(Icons.chevron_right),
                        onTap: () {
                          Navigator.pop(context);
                          _showUserProfileDetail(profile);
                        },
                      );
                    },
                  ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('关闭'),
            ),
          ],
        ),
      );
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('获取用户档案失败: $e')),
        );
      }
    }
  }

  /// 显示用户档案详情
  void _showUserProfileDetail(UserProfile profile) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(profile.nickname ?? '用户档案'),
        content: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _buildInfoRow('用户ID', profile.userId.substring(0, 8)),
              _buildInfoRow('昵称', profile.nickname ?? '未设置'),
              _buildInfoRow('亲和度', '${profile.affinityScore}'),
              if (profile.preferredTopics.isNotEmpty)
                _buildInfoRow(
                  '偏好话题',
                  profile.preferredTopics.join(', '),
                ),
              if (profile.notes != null && profile.notes!.isNotEmpty)
                _buildInfoRow('备注', profile.notes!),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('关闭'),
          ),
        ],
      ),
    );
  }

  /// 格式化日期时间
  String _formatDateTime(DateTime dt) {
    return '${dt.month}/${dt.day} ${dt.hour.toString().padLeft(2, '0')}:${dt.minute.toString().padLeft(2, '0')}';
  }

  /// 显示编辑 AI Agent 对话框
  void _showEditAiAgentDialog() {
    if (_aiAgent == null) return;

    final nameController = TextEditingController(text: _aiAgent!.name);
    int toxicityLevel = _aiAgent!.toxicityLevel;
    int enthusiasmLevel = _aiAgent!.enthusiasmLevel;
    String? speechSuffix = _aiAgent!.speechSuffix;
    final expertiseController = TextEditingController(
      text: _aiAgent!.expertiseDomains.join(', '),
    );
    final keywordsController = TextEditingController(
      text: _aiAgent!.triggerKeywords.join(', '),
    );

    showDialog(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) {
          final theme = Theme.of(context);
          return AlertDialog(
            title: const Text('编辑 AI 助手'),
            content: SingleChildScrollView(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  TextField(
                    controller: nameController,
                    decoration: const InputDecoration(
                      labelText: '名称',
                      hintText: '例如：小助手',
                    ),
                  ),
                  const SizedBox(height: 16),
                  Text('毒舌度: $toxicityLevel/10',
                      style: theme.textTheme.bodyMedium),
                  Slider(
                    value: toxicityLevel.toDouble(),
                    min: 0,
                    max: 10,
                    divisions: 10,
                    label: '$toxicityLevel',
                    onChanged: (value) {
                      setDialogState(() {
                        toxicityLevel = value.round();
                      });
                    },
                  ),
                  const SizedBox(height: 8),
                  Text('热情度: $enthusiasmLevel/10',
                      style: theme.textTheme.bodyMedium),
                  Slider(
                    value: enthusiasmLevel.toDouble(),
                    min: 0,
                    max: 10,
                    divisions: 10,
                    label: '$enthusiasmLevel',
                    onChanged: (value) {
                      setDialogState(() {
                        enthusiasmLevel = value.round();
                      });
                    },
                  ),
                  const SizedBox(height: 16),
                  TextField(
                    controller: expertiseController,
                    decoration: const InputDecoration(
                      labelText: '专业领域（逗号分隔）',
                      hintText: '例如：聊天, 游戏, 技术',
                    ),
                  ),
                  const SizedBox(height: 16),
                  TextField(
                    controller: keywordsController,
                    decoration: const InputDecoration(
                      labelText: '触发关键词（逗号分隔）',
                      hintText: '例如：帮助, 怎么, @ai',
                    ),
                  ),
                  const SizedBox(height: 16),
                  TextField(
                    decoration: InputDecoration(
                      labelText: '说话后缀',
                      hintText: '例如：~, 喵~',
                      suffixText: speechSuffix ?? '无',
                    ),
                    onChanged: (value) {
                      setDialogState(() {
                        speechSuffix = value.isEmpty ? null : value;
                      });
                    },
                  ),
                ],
              ),
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context),
                child: const Text('取消'),
              ),
              ElevatedButton(
                onPressed: () async {
                  final request = UpdatePersonality(
                    name: nameController.text.trim().isEmpty
                        ? null
                        : nameController.text.trim(),
                    toxicityLevel: toxicityLevel,
                    enthusiasmLevel: enthusiasmLevel,
                    speechSuffix: speechSuffix,
                    expertiseDomains: expertiseController.text
                        .split(',')
                        .map((e) => e.trim())
                        .where((e) => e.isNotEmpty)
                        .toList(),
                    triggerKeywords: keywordsController.text
                        .split(',')
                        .map((e) => e.trim())
                        .where((e) => e.isNotEmpty)
                        .toList(),
                  );

                  Navigator.pop(context);

                  try {
                    await _aiAgentService.updatePersonality(_aiAgent!.id, request);
                    _loadAiAgent();
                    if (mounted && context.mounted) {
                      ScaffoldMessenger.of(context).showSnackBar(
                        const SnackBar(content: Text('AI 助手已更新')),
                      );
                    }
                  } catch (e) {
                    if (mounted && context.mounted) {
                      ScaffoldMessenger.of(context).showSnackBar(
                        SnackBar(content: Text('更新失败: $e')),
                      );
                    }
                  }
                },
                child: const Text('保存'),
              ),
            ],
          );
        },
      ),
    );
  }

  Widget _buildInfoRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 80,
            child: Text(
              '$label:',
              style: const TextStyle(fontWeight: FontWeight.bold),
            ),
          ),
          Expanded(child: Text(value)),
        ],
      ),
    );
  }

  void _confirmDeleteAiAgent() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('删除 AI 助手'),
        content: Text('确定要删除 ${_aiAgent!.name} 吗？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () async {
              Navigator.pop(context);
              try {
                await _aiAgentService.deleteAgent(_aiAgent!.id);
                _loadAiAgent();
                if (mounted && context.mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(content: Text('AI 助手已删除')),
                  );
                }
              } catch (e) {
                if (mounted && context.mounted) {
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(content: Text('删除失败: $e')),
                  );
                }
              }
            },
            child: const Text('确定', style: TextStyle(color: Colors.red)),
          ),
        ],
      ),
    );
  }
}
