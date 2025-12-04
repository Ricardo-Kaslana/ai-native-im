import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:client/data/models/conversation.dart';
import 'package:client/data/models/message.dart';
import 'package:client/data/models/user.dart';
import 'package:client/presentation/widgets/conversation/conversation_list_item.dart';

void main() {
  group('ConversationListItem', () {
    testWidgets('renders private conversation correctly', (tester) async {
      final conversation = Conversation(
        id: 'conv_1',
        type: ConversationType.private,
        participants: [
          User(
            id: 'user_1',
            username: '张三',
            isOnline: true,
            createdAt: DateTime.now(),
          ),
        ],
        lastMessage: Message(
          id: 'msg_1',
          conversationId: 'conv_1',
          senderId: 'user_1',
          type: MessageType.text,
          content: 'Hello!',
          status: MessageStatus.read,
          createdAt: DateTime.now(),
        ),
        unreadCount: 0,
        createdAt: DateTime.now(),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ConversationListItem(
              conversation: conversation,
            ),
          ),
        ),
      );

      // Find at least one widget with the name
      expect(find.textContaining('张三'), findsWidgets);
      expect(find.text('Hello!'), findsOneWidget);
    });

    testWidgets('shows unread badge when unread count > 0', (tester) async {
      final conversation = Conversation(
        id: 'conv_2',
        type: ConversationType.private,
        participants: [
          User(
            id: 'user_2',
            username: '李四',
            createdAt: DateTime.now(),
          ),
        ],
        unreadCount: 5,
        createdAt: DateTime.now(),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ConversationListItem(
              conversation: conversation,
            ),
          ),
        ),
      );

      expect(find.text('5'), findsOneWidget);
    });

    testWidgets('shows group name for group conversations', (tester) async {
      final conversation = Conversation(
        id: 'conv_3',
        type: ConversationType.group,
        name: '项目讨论组',
        createdAt: DateTime.now(),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ConversationListItem(
              conversation: conversation,
            ),
          ),
        ),
      );

      expect(find.textContaining('项目讨论组'), findsWidgets);
    });

    testWidgets('calls onTap when tapped', (tester) async {
      bool tapped = false;
      final conversation = Conversation(
        id: 'conv_4',
        type: ConversationType.private,
        participants: [
          User(
            id: 'user_3',
            username: '王五',
            createdAt: DateTime.now(),
          ),
        ],
        createdAt: DateTime.now(),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ConversationListItem(
              conversation: conversation,
              onTap: () => tapped = true,
            ),
          ),
        ),
      );

      await tester.tap(find.byType(ConversationListItem));
      expect(tapped, isTrue);
    });
  });
}
