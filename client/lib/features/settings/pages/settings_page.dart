import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import '../../../core/storage/storage.dart';
import '../../../core/services/notification_helper.dart';
import '../../../core/services/unread_count_service.dart';
import '../../../core/config/app_config.dart';
import '../../chat/services/chat_service_v2.dart';
import '../../conversations/services/conversation_service.dart';

/// 设置页面
class SettingsPage extends StatefulWidget {
  const SettingsPage({super.key});

  @override
  State<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends State<SettingsPage> {
  final _storage = AppStorage.instance;
  final _notificationHelper = NotificationHelper.instance;
  final _chatService = ChatServiceV2.instance;
  final _conversationService = ConversationService.instance;
  final _unreadService = UnreadCountService.instance;

  bool _notificationsEnabled = true;
  bool _soundEnabled = true;
  bool _vibrationEnabled = true;
  bool _messagePreview = true;
  bool _groupNotificationsEnabled = true;
  bool _privateNotificationsEnabled = true;
  bool _mentionNotificationsEnabled = true;
  bool _hasNotificationPermission = true;
  bool _isRefreshing = false;
  String _themeMode = 'system';
  String _serverUrl = '';

  @override
  void initState() {
    super.initState();
    _loadSettings();
    _checkNotificationPermission();
    _loadServerUrl();
  }

  void _loadSettings() {
    setState(() {
      _notificationsEnabled = _storage.getNotificationsEnabled();
      _soundEnabled = _storage.getSoundEnabled();
      _vibrationEnabled = _storage.getVibrationEnabled();
      _messagePreview = _storage.getMessagePreviewEnabled();
      _groupNotificationsEnabled = _storage.getGroupNotificationsEnabled();
      _privateNotificationsEnabled = _storage.getPrivateNotificationsEnabled();
      _mentionNotificationsEnabled = _storage.getMentionNotificationsEnabled();
      _themeMode = _storage.getThemeMode();
    });
  }

  void _loadServerUrl() {
    setState(() {
      _serverUrl = AppConfig.apiBaseUrl;
    });
  }

  Future<void> _checkNotificationPermission() async {
    final hasPermission = await _notificationHelper.hasPermission();
    if (mounted) {
      setState(() => _hasNotificationPermission = hasPermission);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.pop(),
        ),
        title: const Text('设置'),
      ),
      body: ListView(
        children: [
          _buildSection(
            title: '通知',
            children: [
              // 通知权限状态
              ListTile(
                title: const Text('通知权限'),
                subtitle: Text(
                  _hasNotificationPermission ? '已授权' : '未授权',
                  style: TextStyle(
                    color: _hasNotificationPermission
                        ? Colors.green
                        : Colors.orange,
                  ),
                ),
                trailing: _hasNotificationPermission
                    ? const Icon(Icons.check_circle, color: Colors.green)
                    : const Icon(Icons.warning, color: Colors.orange),
                onTap: _hasNotificationPermission
                    ? null
                    : () => _requestNotificationPermission(),
              ),
              const Divider(height: 1),

              // 全局通知开关
              SwitchListTile(
                title: const Text('消息通知'),
                subtitle: const Text('接收新消息推送'),
                value: _notificationsEnabled,
                onChanged: (value) async {
                  await _storage.setNotificationsEnabled(value);
                  setState(() => _notificationsEnabled = value);
                },
              ),
              SwitchListTile(
                title: const Text('提示音'),
                subtitle: const Text('收到消息时播放提示音'),
                value: _soundEnabled,
                onChanged: (value) async {
                  await _storage.setSoundEnabled(value);
                  setState(() => _soundEnabled = value);
                },
              ),
              SwitchListTile(
                title: const Text('振动'),
                subtitle: const Text('收到消息时振动'),
                value: _vibrationEnabled,
                onChanged: (value) async {
                  await _storage.setVibrationEnabled(value);
                  setState(() => _vibrationEnabled = value);
                },
              ),
              SwitchListTile(
                title: const Text('消息预览'),
                subtitle: const Text('在通知中显示消息内容'),
                value: _messagePreview,
                onChanged: (value) async {
                  await _storage.setMessagePreviewEnabled(value);
                  setState(() => _messagePreview = value);
                },
              ),
              const Divider(height: 1),
              ListTile(
                title: const Text('通知详情'),
                subtitle: const Text('群组、私聊、@提醒'),
                trailing: const Icon(Icons.chevron_right),
                onTap: _showNotificationDetails,
              ),
            ],
          ),
          _buildSection(
            title: '外观',
            children: [
              ListTile(
                title: const Text('主题'),
                subtitle: Text(_getThemeModeText(_themeMode)),
                trailing: const Icon(Icons.chevron_right),
                onTap: _showThemeDialog,
              ),
            ],
          ),
          // 开发者选项（仅开发模式显示）
          if (AppConfig.isDebug)
            _buildSection(
              title: '开发者选项',
              children: [
                ListTile(
                  title: const Text('服务器地址'),
                  subtitle: Text(_serverUrl),
                  trailing: const Icon(Icons.chevron_right),
                  onTap: _showServerConfigDialog,
                ),
                ListTile(
                  title: const Text('当前环境'),
                  subtitle: Text(_getEnvironmentText()),
                ),
              ],
            ),
          _buildSection(
            title: '账号与安全',
            children: [
              ListTile(
                title: const Text('修改密码'),
                trailing: const Icon(Icons.chevron_right),
                onTap: () => _showChangePasswordDialog(),
              ),
              ListTile(
                title: const Text('账号注销'),
                trailing: const Icon(Icons.chevron_right),
                onTap: () {},
              ),
            ],
          ),
          _buildSection(
            title: '应用',
            children: [
              ListTile(
                title: const Text('刷新应用状态'),
                subtitle: const Text('重新加载所有数据'),
                leading: _isRefreshing
                    ? const SizedBox(
                        width: 20,
                        height: 20,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : const Icon(Icons.refresh),
                onTap: _isRefreshing ? null : _refreshApp,
              ),
            ],
          ),
          _buildSection(
            title: '关于',
            children: [
              const ListTile(
                title: Text('版本'),
                trailing: Text('3.0.0'),
              ),
              ListTile(
                title: const Text('隐私政策'),
                trailing: const Icon(Icons.chevron_right),
                onTap: () {},
              ),
              ListTile(
                title: const Text('用户协议'),
                trailing: const Icon(Icons.chevron_right),
                onTap: () {},
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildSection({
    required String title,
    required List<Widget> children,
  }) {
    final theme = Theme.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
          child: Text(
            title,
            style: theme.textTheme.titleSmall?.copyWith(
              color: theme.colorScheme.primary,
            ),
          ),
        ),
        Card(
          margin: const EdgeInsets.symmetric(horizontal: 16),
          child: Column(children: children),
        ),
      ],
    );
  }

  String _getThemeModeText(String mode) {
    switch (mode) {
      case 'light':
        return '浅色';
      case 'dark':
        return '深色';
      default:
        return '跟随系统';
    }
  }

  void _showThemeDialog() {
    showDialog(
      context: context,
      builder: (context) {
        String selectedTheme = _themeMode;
        return StatefulBuilder(
          builder: (context, setDialogState) => AlertDialog(
            title: const Text('选择主题'),
            content: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                SegmentedButton<String>(
                  segments: const [
                    ButtonSegment(
                      value: 'system',
                      label: Text('跟随系统'),
                      icon: Icon(Icons.brightness_auto),
                    ),
                    ButtonSegment(
                      value: 'light',
                      label: Text('浅色'),
                      icon: Icon(Icons.light_mode),
                    ),
                    ButtonSegment(
                      value: 'dark',
                      label: Text('深色'),
                      icon: Icon(Icons.dark_mode),
                    ),
                  ],
                  selected: {selectedTheme},
                  onSelectionChanged: (Set<String> value) {
                    setDialogState(() {
                      selectedTheme = value.first;
                    });
                    _setTheme(value.first);
                  },
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  Future<void> _setTheme(String mode) async {
    await _storage.setThemeMode(mode);
    setState(() => _themeMode = mode);
    if (mounted) Navigator.pop(context);
  }

  /// 请求通知权限
  Future<void> _requestNotificationPermission() async {
    final granted = await _notificationHelper.requestPermissions();
    setState(() => _hasNotificationPermission = granted);
    if (!granted && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: const Text('请在系统设置中开启通知权限'),
          action: SnackBarAction(
            label: '去设置',
            onPressed: () {
              _notificationHelper.openSettings();
            },
          ),
        ),
      );
    }
  }

  /// 显示通知详情设置对话框
  void _showNotificationDetails() {
    showDialog(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: const Text('通知详情'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              SwitchListTile(
                title: const Text('群组消息'),
                subtitle: const Text('接收群组新消息通知'),
                value: _groupNotificationsEnabled,
                onChanged: (value) async {
                  await _storage.setGroupNotificationsEnabled(value);
                  setDialogState(() => _groupNotificationsEnabled = value);
                  setState(() => _groupNotificationsEnabled = value);
                },
              ),
              SwitchListTile(
                title: const Text('私聊消息'),
                subtitle: const Text('接收私聊新消息通知'),
                value: _privateNotificationsEnabled,
                onChanged: (value) async {
                  await _storage.setPrivateNotificationsEnabled(value);
                  setDialogState(() => _privateNotificationsEnabled = value);
                  setState(() => _privateNotificationsEnabled = value);
                },
              ),
              SwitchListTile(
                title: const Text('@提醒'),
                subtitle: const Text('接收@提及通知'),
                value: _mentionNotificationsEnabled,
                onChanged: (value) async {
                  await _storage.setMentionNotificationsEnabled(value);
                  setDialogState(() => _mentionNotificationsEnabled = value);
                  setState(() => _mentionNotificationsEnabled = value);
                },
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('完成'),
            ),
          ],
        ),
      ),
    );
  }

  /// 显示服务器配置对话框（开发者选项）
  void _showServerConfigDialog() {
    final controller = TextEditingController(text: _serverUrl);
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('配置服务器地址'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: controller,
              decoration: const InputDecoration(
                labelText: 'API 地址',
                hintText: 'http://localhost:8080',
              ),
              keyboardType: TextInputType.url,
            ),
            const SizedBox(height: 16),
            const Text(
              '修改后需要重启应用生效',
              style: TextStyle(fontSize: 12, color: Colors.grey),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () {
              final url = controller.text.trim();
              if (url.isNotEmpty) {
                AppConfig.setApiBaseUrl(url);
                setState(() => _serverUrl = url);
              }
              Navigator.pop(context);
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(content: Text('服务器地址已更新: $url\n重启应用后生效')),
              );
            },
            child: const Text('保存'),
          ),
        ],
      ),
    );
  }

  /// 获取环境文本
  String _getEnvironmentText() {
    switch (AppConfig.environment) {
      case AppEnvironment.development:
        return '开发环境';
      case AppEnvironment.staging:
        return '预发布环境';
      case AppEnvironment.production:
        return '生产环境';
    }
  }

  void _showChangePasswordDialog() {
    final currentController = TextEditingController();
    final newController = TextEditingController();
    final confirmController = TextEditingController();

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('修改密码'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              controller: currentController,
              decoration: const InputDecoration(labelText: '当前密码'),
              obscureText: true,
            ),
            const SizedBox(height: 16),
            TextField(
              controller: newController,
              decoration: const InputDecoration(labelText: '新密码'),
              obscureText: true,
            ),
            const SizedBox(height: 16),
            TextField(
              controller: confirmController,
              decoration: const InputDecoration(labelText: '确认新密码'),
              obscureText: true,
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () async {
              if (newController.text != confirmController.text) {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(content: Text('两次输入的密码不一致')),
                );
                return;
              }
              // TODO: 调用修改密码 API
              Navigator.pop(context);
            },
            child: const Text('确定'),
          ),
        ],
      ),
    );
  }

  /// 一键刷新应用状态
  Future<void> _refreshApp() async {
    if (_isRefreshing) return;

    setState(() => _isRefreshing = true);

    try {
      // 1. 确保 WebSocket 已连接
      await _chatService.initialize();

      // 2. 刷新会话列表
      await _conversationService.getConversations();

      // 3. 更新未读计数
      await _unreadService.updateUnreadCount();

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('刷新成功')),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('刷新失败: $e')),
        );
      }
    } finally {
      if (mounted) {
        setState(() => _isRefreshing = false);
      }
    }
  }
}
