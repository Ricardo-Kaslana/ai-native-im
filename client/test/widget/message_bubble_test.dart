import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:client/data/models/message.dart';
import 'package:client/presentation/widgets/message/message_bubble.dart';

void main() {
  group('MessageBubble', () {
    testWidgets('renders sent message correctly', (tester) async {
      final message = Message(
        id: 'test_1',
        conversationId: 'conv_1',
        senderId: 'me',
        type: MessageType.text,
        content: 'Hello, World!',
        status: MessageStatus.sent,
        createdAt: DateTime.now(),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: MessageBubble(
              message: message,
              isSent: true,
            ),
          ),
        ),
      );

      expect(find.text('Hello, World!'), findsOneWidget);
    });

    testWidgets('renders received message correctly', (tester) async {
      final message = Message(
        id: 'test_2',
        conversationId: 'conv_1',
        senderId: 'other',
        senderName: '张三',
        type: MessageType.text,
        content: 'Hi there!',
        status: MessageStatus.read,
        createdAt: DateTime.now(),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: MessageBubble(
              message: message,
              isSent: false,
              showAvatar: true,
            ),
          ),
        ),
      );

      expect(find.text('Hi there!'), findsOneWidget);
    });

    testWidgets('shows message status icon for sent messages', (tester) async {
      final message = Message(
        id: 'test_3',
        conversationId: 'conv_1',
        senderId: 'me',
        type: MessageType.text,
        content: 'Test message',
        status: MessageStatus.delivered,
        createdAt: DateTime.now(),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: MessageBubble(
              message: message,
              isSent: true,
            ),
          ),
        ),
      );

      // Should find the done_all icon for delivered status
      expect(find.byIcon(Icons.done_all), findsOneWidget);
    });

    testWidgets('displays image placeholder for image messages', (tester) async {
      final message = Message(
        id: 'test_4',
        conversationId: 'conv_1',
        senderId: 'me',
        type: MessageType.image,
        content: '',
        status: MessageStatus.sent,
        createdAt: DateTime.now(),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: MessageBubble(
              message: message,
              isSent: true,
            ),
          ),
        ),
      );

      expect(find.byIcon(Icons.image), findsOneWidget);
    });
  });
}
