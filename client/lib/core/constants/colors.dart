import 'package:flutter/material.dart';

/// App color palette following Neo-Minimalism design
class AppColors {
  AppColors._();

  // Light Theme Colors
  static const lightPrimary = Color(0xFF2196F3);
  static const lightBackground = Color(0xFFFFFFFF);
  static const lightSurface = Color(0xFFF5F5F5);
  static const lightText = Color(0xFF212121);
  static const lightTextSecondary = Color(0xFF757575);
  static const lightDivider = Color(0xFFE0E0E0);
  static const lightSuccess = Color(0xFF4CAF50);
  static const lightError = Color(0xFFF44336);
  static const lightWarning = Color(0xFFFF9800);

  // Dark Theme Colors (OLED-friendly)
  static const darkPrimary = Color(0xFF64B5F6);
  static const darkBackground = Color(0xFF000000);
  static const darkSurface = Color(0xFF121212);
  static const darkText = Color(0xFFFFFFFF);
  static const darkTextSecondary = Color(0xFFB0B0B0);
  static const darkDivider = Color(0xFF2C2C2C);
  static const darkSuccess = Color(0xFF81C784);
  static const darkError = Color(0xFFE57373);
  static const darkWarning = Color(0xFFFFB74D);

  // Message Bubble Colors
  static const sentBubbleLight = Color(0xFF2196F3);
  static const sentBubbleDark = Color(0xFF1976D2);
  static const receivedBubbleLight = Color(0xFFF5F5F5);
  static const receivedBubbleDark = Color(0xFF1E1E1E);

  // Online Status
  static const online = Color(0xFF4CAF50);
  static const offline = Color(0xFF9E9E9E);
  static const away = Color(0xFFFF9800);
}
