import 'dart:async';
import 'package:flutter/material.dart';
import '../../../core/theme/app_theme.dart';
import '../../../core/services/unread_count_service.dart';
import '../../chat/services/chat_service_v2.dart';
import '../../chat/services/websocket_service_v2.dart';
import '../../conversations/pages/conversation_list_page.dart';
import '../../contacts/pages/contacts_page.dart';
import '../../profile/pages/profile_page.dart';

/// 主页 - 底部导航
class MainPage extends StatefulWidget {
  const MainPage({super.key});

  @override
  State<MainPage> createState() => _MainPageState();
}

class _MainPageState extends State<MainPage> {
  int _currentIndex = 0;

  final _pages = const [
    ConversationListPage(),
    ContactsPage(),
    ProfilePage(),
  ];

  final _chatService = ChatServiceV2.instance;
  final _unreadService = UnreadCountService.instance;
  WebSocketConnectionState _connectionState = WebSocketConnectionState.disconnected;
  StreamSubscription<WebSocketConnectionState>? _connectionSubscription;
  StreamSubscription<int>? _unreadSubscription;
  int _unreadCount = 0;

  @override
  void initState() {
    super.initState();
    _monitorConnection();
    _monitorUnreadCount();
  }

  @override
  void dispose() {
    _connectionSubscription?.cancel();
    _unreadSubscription?.cancel();
    super.dispose();
  }

  /// 监听 WebSocket 连接状态
  void _monitorConnection() {
    _connectionSubscription = _chatService.connectionStateStream.listen((state) {
      if (mounted) {
        setState(() => _connectionState = state);
      }
    });
    // 获取初始状态
    _connectionState = _chatService.connectionState;
  }

  /// 监听未读消息数量
  void _monitorUnreadCount() {
    _unreadSubscription = _unreadService.unreadCountStream.listen((count) {
      if (mounted) {
        setState(() => _unreadCount = count);
      }
    });
    // 获取初始未读数
    _unreadService.updateUnreadCount();
    _unreadCount = _unreadService.totalUnreadCount;
  }

  @override
  Widget build(BuildContext context) {
    final isDesktop = Breakpoints.isDesktop(context);

    if (isDesktop) {
      return _buildDesktopLayout();
    }
    return _buildMobileLayout();
  }

  Widget _buildMobileLayout() {
    return Scaffold(
      body: SafeArea(
        bottom: false,
        child: Column(
          children: [
            // 连接状态指示器（仅断开时显示）
            if (_connectionState != WebSocketConnectionState.connected)
              _buildConnectionBanner(),
            Expanded(
              child: IndexedStack(
                index: _currentIndex,
                children: _pages,
              ),
            ),
          ],
        ),
      ),
      bottomNavigationBar: NavigationBar(
        selectedIndex: _currentIndex,
        onDestinationSelected: (index) => setState(() => _currentIndex = index),
        destinations: [
          NavigationDestination(
            icon: _buildBadgeIcon(
              icon: const Icon(Icons.chat_bubble_outline),
              count: _unreadCount,
              showBadge: _unreadCount > 0,
            ),
            selectedIcon: const Icon(Icons.chat_bubble),
            label: '消息',
          ),
          const NavigationDestination(
            icon: Icon(Icons.people_outline),
            selectedIcon: Icon(Icons.people),
            label: '通讯录',
          ),
          const NavigationDestination(
            icon: Icon(Icons.person_outline),
            selectedIcon: Icon(Icons.person),
            label: '我的',
          ),
        ],
      ),
    );
  }

  Widget _buildDesktopLayout() {
    return Scaffold(
      body: Column(
        children: [
          // 连接状态指示器（仅断开时显示）
          if (_connectionState != WebSocketConnectionState.connected)
            _buildConnectionBanner(),
          Expanded(
            child: Row(
              children: [
                NavigationRail(
                  selectedIndex: _currentIndex,
                  onDestinationSelected: (index) => setState(() => _currentIndex = index),
                  labelType: NavigationRailLabelType.all,
                  leading: Padding(
                    padding: const EdgeInsets.symmetric(vertical: 16),
                    child: Icon(
                      Icons.chat_bubble_rounded,
                      size: 32,
                      color: Theme.of(context).colorScheme.primary,
                    ),
                  ),
                  destinations: [
                    NavigationRailDestination(
                      icon: _buildBadgeIcon(
                        icon: const Icon(Icons.chat_bubble_outline),
                        count: _unreadCount,
                        showBadge: _unreadCount > 0,
                      ),
                      selectedIcon: const Icon(Icons.chat_bubble),
                      label: Text('消息'),
                    ),
                    const NavigationRailDestination(
                      icon: Icon(Icons.people_outline),
                      selectedIcon: Icon(Icons.people),
                      label: Text('通讯录'),
                    ),
                    const NavigationRailDestination(
                      icon: Icon(Icons.person_outline),
                      selectedIcon: Icon(Icons.person),
                      label: Text('我的'),
                    ),
                  ],
                ),
                const VerticalDivider(thickness: 1, width: 1),
                Expanded(
                  child: _pages[_currentIndex],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  /// 构建带红点的图标
  Widget _buildBadgeIcon({
    required Widget icon,
    required int count,
    required bool showBadge,
  }) {
    return Stack(
      clipBehavior: Clip.none,
      children: [
        icon,
        if (showBadge)
          Positioned(
            right: -6,
            top: -6,
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
              decoration: BoxDecoration(
                color: Colors.red,
                borderRadius: BorderRadius.circular(10),
              ),
              constraints: const BoxConstraints(
                minWidth: 18,
                minHeight: 18,
              ),
              child: Text(
                count > 99 ? '99+' : count.toString(),
                style: const TextStyle(
                  color: Colors.white,
                  fontSize: 10,
                  fontWeight: FontWeight.bold,
                ),
                textAlign: TextAlign.center,
              ),
            ),
          ),
      ],
    );
  }

  /// 构建连接状态横幅
  Widget _buildConnectionBanner() {
    final theme = Theme.of(context);
    String message;
    Color backgroundColor;
    Color textColor;
    IconData icon;

    switch (_connectionState) {
      case WebSocketConnectionState.disconnected:
        message = '未连接 - 消息可能无法实时接收';
        backgroundColor = Colors.grey.shade800;
        textColor = Colors.white;
        icon = Icons.cloud_off;
        break;
      case WebSocketConnectionState.connecting:
        message = '连接中...';
        backgroundColor = Colors.orange;
        textColor = Colors.white;
        icon = Icons.sync;
        break;
      case WebSocketConnectionState.reconnecting:
        message = '重新连接中...';
        backgroundColor = Colors.orange;
        textColor = Colors.white;
        icon = Icons.sync;
        break;
      case WebSocketConnectionState.authFailed:
        message = '认证失败 - 请重新登录';
        backgroundColor = theme.colorScheme.error;
        textColor = Colors.white;
        icon = Icons.error;
        break;
      case WebSocketConnectionState.error:
        message = '连接错误 - 请检查网络';
        backgroundColor = theme.colorScheme.error;
        textColor = Colors.white;
        icon = Icons.warning;
        break;
      case WebSocketConnectionState.connected:
        // 已连接时不显示横幅
        return const SizedBox.shrink();
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
      color: backgroundColor,
      child: Row(
        children: [
          Icon(icon, color: textColor, size: 14),
          const SizedBox(width: 6),
          Expanded(
            child: Text(
              message,
              style: TextStyle(color: textColor, fontSize: 11),
            ),
          ),
        ],
      ),
    );
  }
}
