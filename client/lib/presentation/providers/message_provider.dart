import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:uuid/uuid.dart';
import '../../data/models/message.dart';

/// Messages provider family (by conversation ID)
final messagesProvider = StateNotifierProvider.family<MessagesNotifier, AsyncValue<List<Message>>, String>(
  (ref, conversationId) => MessagesNotifier(conversationId),
);

/// Typing indicator provider
final typingIndicatorProvider = StateProvider.family<bool, String>(
  (ref, conversationId) => false,
);

/// Messages state notifier
class MessagesNotifier extends StateNotifier<AsyncValue<List<Message>>> {
  final String conversationId;
  final _uuid = const Uuid();

  MessagesNotifier(this.conversationId) : super(const AsyncValue.loading()) {
    _loadMessages();
  }

  Future<void> _loadMessages() async {
    try {
      // Simulate network delay
      await Future.delayed(const Duration(milliseconds: 300));

      // Return mock messages
      state = AsyncValue.data(_getMockMessages());
    } catch (e, st) {
      state = AsyncValue.error(e, st);
    }
  }

  Future<void> sendMessage(String content) async {
    if (content.trim().isEmpty) return;

    final newMessage = Message(
      id: _uuid.v4(),
      conversationId: conversationId,
      senderId: 'me',
      type: MessageType.text,
      content: content.trim(),
      status: MessageStatus.sending,
      createdAt: DateTime.now(),
    );

    // Add message optimistically
    state.whenData((messages) {
      state = AsyncValue.data([newMessage, ...messages]);
    });

    // Simulate sending
    await Future.delayed(const Duration(milliseconds: 500));

    // Update status to sent
    state.whenData((messages) {
      final index = messages.indexWhere((m) => m.id == newMessage.id);
      if (index != -1) {
        final updatedMessages = [...messages];
        updatedMessages[index] = newMessage.copyWith(status: MessageStatus.sent);
        state = AsyncValue.data(updatedMessages);
      }
    });

    // Simulate delivery
    await Future.delayed(const Duration(seconds: 1));
    state.whenData((messages) {
      final index = messages.indexWhere((m) => m.id == newMessage.id);
      if (index != -1) {
        final updatedMessages = [...messages];
        updatedMessages[index] = newMessage.copyWith(status: MessageStatus.delivered);
        state = AsyncValue.data(updatedMessages);
      }
    });
  }

  Future<void> loadMore() async {
    // TODO: Implement pagination
  }

  List<Message> _getMockMessages() {
    final now = DateTime.now();
    return [
      Message(
        id: 'msg_1',
        conversationId: conversationId,
        senderId: 'other',
        senderName: '对方',
        type: MessageType.text,
        content: '你好！最近怎么样？',
        status: MessageStatus.read,
        createdAt: now.subtract(const Duration(minutes: 30)),
      ),
      Message(
        id: 'msg_2',
        conversationId: conversationId,
        senderId: 'me',
        type: MessageType.text,
        content: '挺好的，谢谢关心！你呢？',
        status: MessageStatus.read,
        createdAt: now.subtract(const Duration(minutes: 28)),
      ),
      Message(
        id: 'msg_3',
        conversationId: conversationId,
        senderId: 'other',
        senderName: '对方',
        type: MessageType.text,
        content: '我也不错，最近在忙一个新项目',
        status: MessageStatus.read,
        createdAt: now.subtract(const Duration(minutes: 25)),
      ),
      Message(
        id: 'msg_4',
        conversationId: conversationId,
        senderId: 'me',
        type: MessageType.text,
        content: '听起来很有意思！是什么项目？',
        status: MessageStatus.read,
        createdAt: now.subtract(const Duration(minutes: 20)),
      ),
      Message(
        id: 'msg_5',
        conversationId: conversationId,
        senderId: 'other',
        senderName: '对方',
        type: MessageType.text,
        content: '是一个 AI 聊天应用，使用 Flutter 开发的',
        status: MessageStatus.read,
        createdAt: now.subtract(const Duration(minutes: 15)),
      ),
      Message(
        id: 'msg_6',
        conversationId: conversationId,
        senderId: 'me',
        type: MessageType.text,
        content: '太棒了！Flutter 确实很适合做跨平台应用',
        status: MessageStatus.delivered,
        createdAt: now.subtract(const Duration(minutes: 10)),
      ),
    ];
  }
}
