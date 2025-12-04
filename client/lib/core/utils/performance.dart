import 'package:flutter/foundation.dart';
import 'package:flutter/scheduler.dart';

/// Performance monitoring utilities
class PerformanceMonitor {
  PerformanceMonitor._();

  static bool _isMonitoring = false;
  static final List<double> _frameTimes = [];

  /// Start monitoring frame rate
  static void startMonitoring() {
    if (_isMonitoring) return;
    _isMonitoring = true;

    SchedulerBinding.instance.addPostFrameCallback(_onFrame);
  }

  /// Stop monitoring
  static void stopMonitoring() {
    _isMonitoring = false;
    _frameTimes.clear();
  }

  static void _onFrame(Duration timestamp) {
    if (!_isMonitoring) return;

    final frameTime = timestamp.inMicroseconds / 1000.0;
    _frameTimes.add(frameTime);

    // Keep last 60 frames
    if (_frameTimes.length > 60) {
      _frameTimes.removeAt(0);
    }

    SchedulerBinding.instance.addPostFrameCallback(_onFrame);
  }

  /// Get average FPS
  static double get averageFps {
    if (_frameTimes.length < 2) return 0;

    double totalTime = 0;
    for (int i = 1; i < _frameTimes.length; i++) {
      totalTime += _frameTimes[i] - _frameTimes[i - 1];
    }

    final avgFrameTime = totalTime / (_frameTimes.length - 1);
    return avgFrameTime > 0 ? 1000 / avgFrameTime : 0;
  }

  /// Check if running at 60fps
  static bool get isSmooth => averageFps >= 55;

  /// Log performance info (debug only)
  static void logPerformance() {
    if (kDebugMode) {
      debugPrint('Average FPS: ${averageFps.toStringAsFixed(1)}');
      debugPrint('Is Smooth: $isSmooth');
    }
  }
}

/// Widget rebuild tracker (debug only)
class RebuildTracker {
  static final Map<String, int> _rebuildCounts = {};

  /// Track widget rebuild
  static void track(String widgetName) {
    if (!kDebugMode) return;
    _rebuildCounts[widgetName] = (_rebuildCounts[widgetName] ?? 0) + 1;
  }

  /// Get rebuild count for widget
  static int getCount(String widgetName) {
    return _rebuildCounts[widgetName] ?? 0;
  }

  /// Reset all counts
  static void reset() {
    _rebuildCounts.clear();
  }

  /// Log all rebuild counts
  static void logAll() {
    if (!kDebugMode) return;
    debugPrint('=== Widget Rebuild Counts ===');
    _rebuildCounts.forEach((name, count) {
      debugPrint('$name: $count');
    });
  }
}

/// Memory usage tracker
class MemoryTracker {
  MemoryTracker._();

  /// Log current memory usage (debug only)
  static void logMemoryUsage() {
    if (!kDebugMode) return;
    // Note: Actual memory tracking requires platform-specific implementation
    debugPrint('Memory tracking not available in this build');
  }
}
