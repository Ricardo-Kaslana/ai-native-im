import 'package:flutter/services.dart';

/// Haptic feedback utilities
class Haptics {
  Haptics._();

  /// Light impact feedback (for button taps)
  static void light() {
    HapticFeedback.lightImpact();
  }

  /// Medium impact feedback (for selections)
  static void medium() {
    HapticFeedback.mediumImpact();
  }

  /// Heavy impact feedback (for important actions)
  static void heavy() {
    HapticFeedback.heavyImpact();
  }

  /// Selection click feedback
  static void selection() {
    HapticFeedback.selectionClick();
  }

  /// Vibrate feedback (for errors)
  static void vibrate() {
    HapticFeedback.vibrate();
  }
}
