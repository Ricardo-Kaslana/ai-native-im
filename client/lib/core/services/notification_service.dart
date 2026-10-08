import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:timezone/timezone.dart' as tz;
import 'package:timezone/data/latest.dart' as tz_data;

/// 通知服务 - 处理本地消息通知
class NotificationService {
  NotificationService._();
  static final NotificationService instance = NotificationService._();

  final _notifications = FlutterLocalNotificationsPlugin();
  bool _initialized = false;

  /// 初始化通知服务
  Future<void> initialize() async {
    if (_initialized) return;

    // 初始化时区
    tz_data.initializeTimeZones();
    tz.setLocalLocation(tz.local);

    // Android 初始化设置
    const androidSettings = AndroidInitializationSettings('@mipmap/ic_launcher');

    // iOS 初始化设置
    const iosSettings = DarwinInitializationSettings(
      requestAlertPermission: false,
      requestBadgePermission: false,
      requestSoundPermission: false,
    );

    const initSettings = InitializationSettings(
      android: androidSettings,
      iOS: iosSettings,
    );

    await _notifications.initialize(
      initSettings,
      onDidReceiveNotificationResponse: _onNotificationTap,
    );

    _initialized = true;
  }

  /// 请求通知权限
  Future<bool> requestPermissions() async {
    if (kDebugMode) {
      // Debug 模式下直接返回 true（桌面平台可能不支持）
      if (!Platform.isAndroid && !Platform.isIOS) {
        return true;
      }
    }

    // Android 13+ 需要运行时权限
    if (Platform.isAndroid) {
      final status = await _notifications
          .resolvePlatformSpecificImplementation<
              AndroidFlutterLocalNotificationsPlugin>()
          ?.requestNotificationsPermission();

      // 如果用户拒绝，引导到设置
      if (status != null && !status) {
        return false;
      }

      // 检查精确通知权限（Android 14+）
      final exactAlarmStatus = await Permission.scheduleExactAlarm.isGranted;
      if (!exactAlarmStatus) {
        await Permission.scheduleExactAlarm.request();
      }
    }

    // iOS 权限
    if (Platform.isIOS) {
      final bool? granted = await _notifications
          .resolvePlatformSpecificImplementation<
              IOSFlutterLocalNotificationsPlugin>()
          ?.requestPermissions(
            alert: true,
            badge: true,
            sound: true,
          );
      return granted ?? false;
    }

    return true;
  }

  /// 检查通知权限状态
  Future<bool> areNotificationsEnabled() async {
    if (kDebugMode) {
      if (!Platform.isAndroid && !Platform.isIOS) {
        return true;
      }
    }

    if (Platform.isAndroid) {
      return await Permission.notification.isGranted;
    }

    if (Platform.isIOS) {
      return await Permission.notification.isGranted;
    }

    return true;
  }

  /// 打开系统通知设置
  Future<void> openNotificationSettings() async {
    if (Platform.isAndroid) {
      // 打开应用通知设置
      await openAppSettings();
    } else if (Platform.isIOS) {
      await openAppSettings();
    }
  }

  /// 显示即时消息通知
  Future<void> showMessageNotification({
    required String title,
    required String body,
    String? payload,
    String? channelId = 'messages',
    String? channelName = '消息通知',
    String? channelDescription = '接收新消息的通知',
  }) async {
    if (!await areNotificationsEnabled()) return;

    // Android 通知渠道
    final androidChannel = AndroidNotificationChannel(
      channelId!,
      channelName!,
      description: channelDescription,
      importance: Importance.high,
      showBadge: true,
      playSound: true,
      enableVibration: true,
    );

    // 创建通知渠道（仅 Android）
    if (Platform.isAndroid) {
      await _notifications
          .resolvePlatformSpecificImplementation<
              AndroidFlutterLocalNotificationsPlugin>()
          ?.createNotificationChannel(androidChannel);
    }

    // Android 通知详情
    final androidDetails = AndroidNotificationDetails(
      channelId,
      channelName,
      channelDescription: channelDescription,
      importance: Importance.high,
      showWhen: true,
      icon: '@mipmap/ic_launcher',
      styleInformation: const BigTextStyleInformation(''),
    );

    // iOS 通知详情
    const iosDetails = DarwinNotificationDetails(
      presentAlert: true,
      presentBadge: true,
      presentSound: true,
    );

    final details = NotificationDetails(
      android: androidDetails,
      iOS: iosDetails,
    );

    await _notifications.show(
      DateTime.now().millisecondsSinceEpoch % 100000, // 生成唯一 ID
      title,
      body,
      details,
      payload: payload,
    );
  }

  /// 显示群组消息通知
  Future<void> showGroupMessageNotification({
    required String groupName,
    required String senderName,
    required String message,
    required String conversationId,
  }) async {
    await showMessageNotification(
      title: groupName,
      body: '$senderName: $message',
      payload: 'conversation:$conversationId',
      channelId: 'group_messages',
      channelName: '群组消息',
      channelDescription: '群组新消息通知',
    );
  }

  /// 显示私聊消息通知
  Future<void> showPrivateMessageNotification({
    required String senderName,
    required String message,
    required String userId,
  }) async {
    await showMessageNotification(
      title: senderName,
      body: message,
      payload: 'user:$userId',
      channelId: 'private_messages',
      channelName: '私聊消息',
      channelDescription: '私聊新消息通知',
    );
  }

  /// 显示系统通知
  Future<void> showSystemNotification({
    required String title,
    required String message,
  }) async {
    await showMessageNotification(
      title: title,
      body: message,
      channelId: 'system',
      channelName: '系统通知',
      channelDescription: '系统相关通知',
    );
  }

  /// 清除所有通知
  Future<void> clearAllNotifications() async {
    await _notifications.cancelAll();
  }

  /// 清除指定通知
  Future<void> clearNotification(int id) async {
    await _notifications.cancel(id);
  }

  /// 处理通知点击事件
  void _onNotificationTap(NotificationResponse response) {
    // 这里可以通过路由跳转到对应的聊天页面
    // 例如: Get.toNamed('/chat/${response.payload}');
    debugPrint('通知点击: ${response.payload}');
  }

  /// 更新应用角标（仅 iOS）
  /// 注意：flutter_local_notifications 17.x 版本移除了 setBadge 方法
  /// 角标现在需要在发送通知时通过 presentationOptions 控制
  Future<void> setBadgeCount(int count) async {
    if (!Platform.isIOS) return;

    // 当前版本暂不支持直接设置角标
    // 可以通过发送带有 badge 数量的通知来间接实现
    debugPrint('设置角标: $count (当前版本暂不支持)');
  }

  /// 重置应用角标
  Future<void> resetBadge() async {
    await setBadgeCount(0);
  }
}
