import 'dart:io';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path_provider/path_provider.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Cache manager provider
final cacheManagerProvider = Provider<CacheManager>((ref) {
  return CacheManager();
});

/// Cache statistics provider
final cacheStatsProvider = FutureProvider<CacheStats>((ref) async {
  final manager = ref.watch(cacheManagerProvider);
  return manager.getCacheStats();
});

/// Cache statistics
class CacheStats {
  final int imageCacheSize;
  final int messageCacheSize;
  final int totalSize;
  final int fileCount;

  const CacheStats({
    this.imageCacheSize = 0,
    this.messageCacheSize = 0,
    this.totalSize = 0,
    this.fileCount = 0,
  });

  String get formattedSize {
    if (totalSize < 1024) return '$totalSize B';
    if (totalSize < 1024 * 1024) return '${(totalSize / 1024).toStringAsFixed(1)} KB';
    if (totalSize < 1024 * 1024 * 1024) {
      return '${(totalSize / (1024 * 1024)).toStringAsFixed(1)} MB';
    }
    return '${(totalSize / (1024 * 1024 * 1024)).toStringAsFixed(1)} GB';
  }
}

/// Cache manager for managing app cache
class CacheManager {
  static const int maxCacheSize = 500 * 1024 * 1024; // 500 MB
  static const String _imageCacheDir = 'image_cache';
  static const String _messageCacheDir = 'message_cache';

  /// Get cache statistics
  Future<CacheStats> getCacheStats() async {
    final cacheDir = await getTemporaryDirectory();
    final imageDir = Directory('${cacheDir.path}/$_imageCacheDir');
    final messageDir = Directory('${cacheDir.path}/$_messageCacheDir');

    int imageCacheSize = 0;
    int messageCacheSize = 0;
    int fileCount = 0;

    if (await imageDir.exists()) {
      final stats = await _getDirectoryStats(imageDir);
      imageCacheSize = stats.$1;
      fileCount += stats.$2;
    }

    if (await messageDir.exists()) {
      final stats = await _getDirectoryStats(messageDir);
      messageCacheSize = stats.$1;
      fileCount += stats.$2;
    }

    return CacheStats(
      imageCacheSize: imageCacheSize,
      messageCacheSize: messageCacheSize,
      totalSize: imageCacheSize + messageCacheSize,
      fileCount: fileCount,
    );
  }

  /// Clear all cache
  Future<void> clearAllCache() async {
    await clearImageCache();
    await clearMessageCache();
    await _clearPreferences();
  }

  /// Clear image cache
  Future<void> clearImageCache() async {
    final cacheDir = await getTemporaryDirectory();
    final imageDir = Directory('${cacheDir.path}/$_imageCacheDir');
    if (await imageDir.exists()) {
      await imageDir.delete(recursive: true);
    }
  }

  /// Clear message cache
  Future<void> clearMessageCache() async {
    final cacheDir = await getTemporaryDirectory();
    final messageDir = Directory('${cacheDir.path}/$_messageCacheDir');
    if (await messageDir.exists()) {
      await messageDir.delete(recursive: true);
    }
  }

  /// Clear old cache if exceeds limit
  Future<void> trimCache() async {
    final stats = await getCacheStats();
    if (stats.totalSize > maxCacheSize) {
      // Clear oldest files first
      await _trimDirectory(await getTemporaryDirectory(), maxCacheSize ~/ 2);
    }
  }

  Future<(int, int)> _getDirectoryStats(Directory dir) async {
    int size = 0;
    int count = 0;

    await for (final entity in dir.list(recursive: true)) {
      if (entity is File) {
        size += await entity.length();
        count++;
      }
    }

    return (size, count);
  }

  Future<void> _trimDirectory(Directory dir, int targetSize) async {
    final files = <File>[];

    await for (final entity in dir.list(recursive: true)) {
      if (entity is File) {
        files.add(entity);
      }
    }

    // Sort by last modified (oldest first)
    files.sort((a, b) {
      final aStat = a.statSync();
      final bStat = b.statSync();
      return aStat.modified.compareTo(bStat.modified);
    });

    int currentSize = 0;
    for (final file in files) {
      currentSize += await file.length();
    }

    // Delete oldest files until under target size
    for (final file in files) {
      if (currentSize <= targetSize) break;
      final fileSize = await file.length();
      await file.delete();
      currentSize -= fileSize;
    }
  }

  Future<void> _clearPreferences() async {
    final prefs = await SharedPreferences.getInstance();
    // Only clear cache-related preferences
    final keysToRemove = prefs.getKeys().where((key) => key.startsWith('cache_'));
    for (final key in keysToRemove) {
      await prefs.remove(key);
    }
  }
}
