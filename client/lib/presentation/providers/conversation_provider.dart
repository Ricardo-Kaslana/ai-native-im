import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../data/models/conversation.dart';
import '../../data/models/message.dart';
import '../../data/models/user.dart';

/// Conversations provider with mock data
final conversationsProvider = FutureProvider<List<Conversation>>((ref) async {
  // Simulate network delay
  await Future.delayed(const Duration(milliseconds: 500));

  // Return mock data
  return _mockConversations;
});

/// Selected conversation provider
final selectedConversationProvider = StateProvider<Conversation?>((ref) => null);

/// Mock conversations for development
final _mockConversations = [
  Conversation(
    id: '1',
    type: ConversationType.private,
    participants: [
      User(
        id: 'user_1',
        username: '张三',
        avatarUrl: null,
        isOnline: true,
        createdAt: DateTime.now().subtract(const Duration(days: 30)),
      ),
    ],
    lastMessage: Message(
      id: 'msg_1',
      conversationId: '1',
      senderId: 'user_1',
      type: MessageType.text,
      content: '好的，明天见！',
      status: MessageStatus.read,
      createdAt: DateTime.now().subtract(const Duration(minutes: 5)),
    ),
    unreadCount: 0,
    isPinned: true,
    lastActivityAt: DateTime.now().subtract(const Duration(minutes: 5)),
    createdAt: DateTime.now().subtract(const Duration(days: 7)),
  ),
  Conversation(
    id: '2',
    type: ConversationType.group,
    name: '项目讨论组',
    participants: [
      User(
        id: 'user_2',
        username: '李四',
        isOnline: true,
        createdAt: DateTime.now().subtract(const Duration(days: 20)),
      ),
      User(
        id: 'user_3',
        username: '王五',
        isOnline: false,
        createdAt: DateTime.now().subtract(const Duration(days: 15)),
      ),
    ],
    lastMessage: Message(
      id: 'msg_2',
      conversationId: '2',
      senderId: 'user_2',
      senderName: '李四',
      type: MessageType.text,
      content: '新版本已经发布了',
      status: MessageStatus.delivered,
      createdAt: DateTime.now().subtract(const Duration(hours: 1)),
    ),
    unreadCount: 3,
    lastActivityAt: DateTime.now().subtract(const Duration(hours: 1)),
    createdAt: DateTime.now().subtract(const Duration(days: 14)),
  ),
  Conversation(
    id: '3',
    type: ConversationType.aiAgent,
    name: 'AI 助手',
    lastMessage: Message(
      id: 'msg_3',
      conversationId: '3',
      senderId: 'ai_agent_1',
      type: MessageType.text,
      content: '有什么我可以帮助你的吗？',
      status: MessageStatus.read,
      createdAt: DateTime.now().subtract(const Duration(hours: 3)),
    ),
    unreadCount: 0,
    lastActivityAt: DateTime.now().subtract(const Duration(hours: 3)),
    createdAt: DateTime.now().subtract(const Duration(days: 5)),
  ),
  Conversation(
    id: '4',
    type: ConversationType.private,
    participants: [
      User(
        id: 'user_4',
        username: '赵六',
        isOnline: false,
        lastSeenAt: DateTime.now().subtract(const Duration(hours: 2)),
        createdAt: DateTime.now().subtract(const Duration(days: 10)),
      ),
    ],
    lastMessage: Message(
      id: 'msg_4',
      conversationId: '4',
      senderId: 'me',
      type: MessageType.image,
      content: '',
      status: MessageStatus.delivered,
      createdAt: DateTime.now().subtract(const Duration(days: 1)),
    ),
    unreadCount: 0,
    lastActivityAt: DateTime.now().subtract(const Duration(days: 1)),
    createdAt: DateTime.now().subtract(const Duration(days: 10)),
  ),
];
