import 'package:flutter/foundation.dart';
import '../storage/storage.dart';
import 'notification_service.dart';

/// 通知助手 - 根据用户设置决定是否显示通知
class NotificationHelper {
  NotificationHelper._();
  static final NotificationHelper instance = NotificationHelper._();

  final _notificationService = NotificationService.instance;
  final _storage = AppStorage.instance;

  /// 处理私聊消息通知
  Future<void> handlePrivateMessage({
    required String senderName,
    required String message,
    required String userId,
  }) async {
    // 检查全局通知开关
    if (!_storage.getNotificationsEnabled()) return;

    // 检查私聊通知开关
    if (!_storage.getPrivateNotificationsEnabled()) return;

    await _notificationService.showPrivateMessageNotification(
      senderName: senderName,
      message: message,
      userId: userId,
    );
  }

  /// 处理群组消息通知
  Future<void> handleGroupMessage({
    required String groupName,
    required String senderName,
    required String message,
    required String conversationId,
    bool isMention = false,
  }) async {
    // 检查全局通知开关
    if (!_storage.getNotificationsEnabled()) return;

    // 如果是 @mention，检查提及通知开关
    if (isMention) {
      if (!_storage.getMentionNotificationsEnabled()) return;
    } else {
      // 普通群组消息检查群组通知开关
      if (!_storage.getGroupNotificationsEnabled()) return;
    }

    await _notificationService.showGroupMessageNotification(
      groupName: groupName,
      senderName: senderName,
      message: message,
      conversationId: conversationId,
    );
  }

  /// 处理系统通知
  Future<void> handleSystemNotification({
    required String title,
    required String message,
  }) async {
    // 系统通知不受用户设置影响
    await _notificationService.showSystemNotification(
      title: title,
      message: message,
    );
  }

  /// 初始化通知服务
  Future<void> initialize() async {
    await _notificationService.initialize();

    // 检查权限状态
    final hasPermission = await _notificationService.areNotificationsEnabled();
    debugPrint('通知权限状态: $hasPermission');
  }

  /// 请求通知权限
  Future<bool> requestPermissions() async {
    return await _notificationService.requestPermissions();
  }

  /// 检查是否有通知权限
  Future<bool> hasPermission() async {
    return await _notificationService.areNotificationsEnabled();
  }

  /// 打开系统通知设置
  Future<void> openSettings() async {
    await _notificationService.openNotificationSettings();
  }

  /// 清除所有通知
  Future<void> clearAll() async {
    await _notificationService.clearAllNotifications();
  }

  /// 获取当前通知设置摘要
  NotificationSettingsSummary getSettingsSummary() {
    return NotificationSettingsSummary(
      notificationsEnabled: _storage.getNotificationsEnabled(),
      soundEnabled: _storage.getSoundEnabled(),
      vibrationEnabled: _storage.getVibrationEnabled(),
      messagePreviewEnabled: _storage.getMessagePreviewEnabled(),
      groupNotificationsEnabled: _storage.getGroupNotificationsEnabled(),
      privateNotificationsEnabled: _storage.getPrivateNotificationsEnabled(),
      mentionNotificationsEnabled: _storage.getMentionNotificationsEnabled(),
    );
  }
}

/// 通知设置摘要
class NotificationSettingsSummary {
  final bool notificationsEnabled;
  final bool soundEnabled;
  final bool vibrationEnabled;
  final bool messagePreviewEnabled;
  final bool groupNotificationsEnabled;
  final bool privateNotificationsEnabled;
  final bool mentionNotificationsEnabled;

  NotificationSettingsSummary({
    required this.notificationsEnabled,
    required this.soundEnabled,
    required this.vibrationEnabled,
    required this.messagePreviewEnabled,
    required this.groupNotificationsEnabled,
    required this.privateNotificationsEnabled,
    required this.mentionNotificationsEnabled,
  });

  @override
  String toString() {
    return 'NotificationSettingsSummary('
        'notificationsEnabled: $notificationsEnabled, '
        'soundEnabled: $soundEnabled, '
        'vibrationEnabled: $vibrationEnabled, '
        'messagePreviewEnabled: $messagePreviewEnabled, '
        'groupNotificationsEnabled: $groupNotificationsEnabled, '
        'privateNotificationsEnabled: $privateNotificationsEnabled, '
        'mentionNotificationsEnabled: $mentionNotificationsEnabled)';
  }
}
