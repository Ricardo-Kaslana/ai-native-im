import 'dart:async';
import 'package:flutter/foundation.dart';
import '../network/api_client.dart';
import '../../features/conversations/models/conversation.dart';

/// 未读消息计数服务
/// 单例模式，全局管理未读消息数量
class UnreadCountService {
  static final UnreadCountService instance = UnreadCountService._();
  UnreadCountService._();

  final _api = ApiClient.instance;

  // 总未读数
  int _totalUnreadCount = 0;

  // 每个会话的未读计数缓存（conversation_id -> unread_count）
  final Map<String, int> _conversationUnreadCounts = {};

  // 控制器
  final _countController = StreamController<int>.broadcast();

  /// 总未读数流
  Stream<int> get unreadCountStream => _countController.stream;

  /// 当前总未读数
  int get totalUnreadCount => _totalUnreadCount;

  /// 获取指定会话的未读计数
  int getConversationUnreadCount(String conversationId) {
    return _conversationUnreadCounts[conversationId] ?? 0;
  }

  /// 更新未读计数
  /// 注意：这个方法会覆盖总未读数，但不覆盖手动设置的会话未读计数
  Future<void> updateUnreadCount() async {
    try {
      final conversations = await _api.get<List<Conversation>>(
        '/conversations',
        fromJson: (json) {
          final List list;
          if (json is List) {
            list = json;
          } else if (json is Map) {
            list = json['conversations'] as List? ?? json['data'] as List? ?? [];
          } else {
            list = [];
          }
          return list
              .map((e) => Conversation.fromJson(e as Map<String, dynamic>))
              .toList();
        },
      );

      // 服务器不返回 unread_count，所以服务器返回的总数总是 0
      // 我们使用本地手动设置的会话未读计数来计算总数
      final localTotal = _conversationUnreadCounts.values.fold(0, (sum, count) => sum + count);

      if (_totalUnreadCount != localTotal) {
        _totalUnreadCount = localTotal;
        if (!_countController.isClosed) {
          _countController.add(_totalUnreadCount);
        }
      }

      debugPrint('UnreadCountService: updateUnreadCount - localTotal: $localTotal, server conversations: ${conversations.length}');
    } catch (e) {
      debugPrint('UnreadCountService: Failed to update unread count - $e');
    }
  }

  /// 设置未读数（用于手动设置，比如收到新消息时）
  void setUnreadCount(int count) {
    if (_totalUnreadCount != count) {
      _totalUnreadCount = count;
      if (!_countController.isClosed) {
        _countController.add(_totalUnreadCount);
      }
    }
  }

  /// 增加未读数
  void incrementUnreadCount([int amount = 1]) {
    setUnreadCount(_totalUnreadCount + amount);
  }

  /// 增加指定会话的未读数
  void incrementConversationUnread(String conversationId, [int amount = 1]) {
    final oldCount = _conversationUnreadCounts[conversationId] ?? 0;
    final newCount = oldCount + amount;
    _conversationUnreadCounts[conversationId] = newCount;

    // 更新总未读数
    final totalNewCount = _totalUnreadCount + amount;
    if (_totalUnreadCount != totalNewCount) {
      _totalUnreadCount = totalNewCount;
      if (!_countController.isClosed) {
        _countController.add(_totalUnreadCount);
      }
    }

    debugPrint('UnreadCountService: Conversation $conversationId unread: $newCount (total: $_totalUnreadCount)');
  }

  /// 标记会话为已读
  void markConversationAsRead(String conversationId) {
    final oldCount = _conversationUnreadCounts[conversationId] ?? 0;
    if (oldCount > 0) {
      _conversationUnreadCounts[conversationId] = 0;

      // 减少总未读数
      final totalNewCount = _totalUnreadCount - oldCount;
      _totalUnreadCount = totalNewCount < 0 ? 0 : totalNewCount;
      if (!_countController.isClosed) {
        _countController.add(_totalUnreadCount);
      }

      debugPrint('UnreadCountService: Conversation $conversationId marked as read (was $oldCount, total now: $_totalUnreadCount)');
    }
  }

  /// 减少未读数
  void decrementUnreadCount([int amount = 1]) {
    final newCount = _totalUnreadCount - amount;
    setUnreadCount(newCount < 0 ? 0 : newCount);
  }

  /// 清除所有未读
  void clearAll() {
    _conversationUnreadCounts.clear();
    setUnreadCount(0);
  }

  /// 释放资源
  void dispose() {
    _countController.close();
  }
}
