import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'app.dart';
import 'core/storage/storage.dart';
import 'core/services/notification_helper.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // 初始化存储
  await AppStorage.instance.init();
  // 初始化安全存储缓存（用于同步访问用户ID和Token）
  await SecureStorage.instance.initCache();
  // 初始化通知服务
  await NotificationHelper.instance.initialize();

  runApp(const ProviderScope(child: AiimApp()));
}
