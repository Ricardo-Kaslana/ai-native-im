import 'dart:io';
import 'package:flutter/foundation.dart';

/// Platform utilities
class PlatformUtils {
  PlatformUtils._();

  /// Check if running on iOS
  static bool get isIOS => !kIsWeb && Platform.isIOS;

  /// Check if running on Android
  static bool get isAndroid => !kIsWeb && Platform.isAndroid;

  /// Check if running on macOS
  static bool get isMacOS => !kIsWeb && Platform.isMacOS;

  /// Check if running on Windows
  static bool get isWindows => !kIsWeb && Platform.isWindows;

  /// Check if running on Linux
  static bool get isLinux => !kIsWeb && Platform.isLinux;

  /// Check if running on Web
  static bool get isWeb => kIsWeb;

  /// Check if running on mobile (iOS or Android)
  static bool get isMobile => isIOS || isAndroid;

  /// Check if running on desktop (macOS, Windows, Linux)
  static bool get isDesktop => isMacOS || isWindows || isLinux;

  /// Get platform name
  static String get platformName {
    if (kIsWeb) return 'Web';
    if (Platform.isIOS) return 'iOS';
    if (Platform.isAndroid) return 'Android';
    if (Platform.isMacOS) return 'macOS';
    if (Platform.isWindows) return 'Windows';
    if (Platform.isLinux) return 'Linux';
    return 'Unknown';
  }

  /// Check if platform supports haptic feedback
  static bool get supportsHaptics => isMobile;

  /// Check if platform supports push notifications
  static bool get supportsPushNotifications => isMobile;

  /// Check if platform supports biometric authentication
  static bool get supportsBiometrics => isMobile;
}
