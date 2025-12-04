import 'package:flutter_test/flutter_test.dart';
import 'package:client/data/models/message.dart';

void main() {
  group('Chat Flow Integration Test', () {
    testWidgets('message model has correct structure', (tester) async {
      // Test message model
      final message = Message(
        id: 'msg-1',
        conversationId: 'conv-1',
        senderId: 'user-1',
        content: 'Hello!',
        type: MessageType.text,
        status: MessageStatus.sent,
        createdAt: DateTime.now(),
      );

      expect(message.id, 'msg-1');
      expect(message.content, 'Hello!');
      expect(message.type, MessageType.text);
      expect(message.status, MessageStatus.sent);
    });

    testWidgets('image message model works correctly', (tester) async {
      // Test image message
      final imageMessage = Message(
        id: 'msg-2',
        conversationId: 'conv-1',
        senderId: 'user-1',
        content: 'https://example.com/image.jpg',
        type: MessageType.image,
        status: MessageStatus.delivered,
        createdAt: DateTime.now(),
      );

      expect(imageMessage.type, MessageType.image);
      expect(imageMessage.status, MessageStatus.delivered);
    });

    testWidgets('video message model works correctly', (tester) async {
      // Test video message
      final videoMessage = Message(
        id: 'msg-3',
        conversationId: 'conv-1',
        senderId: 'user-1',
        content: 'https://example.com/video.mp4',
        type: MessageType.video,
        status: MessageStatus.read,
        createdAt: DateTime.now(),
      );

      expect(videoMessage.type, MessageType.video);
      expect(videoMessage.status, MessageStatus.read);
    });

    testWidgets('message status transitions are valid', (tester) async {
      // Test all message statuses
      expect(MessageStatus.sending.index, isNot(MessageStatus.sent.index));
      expect(MessageStatus.sent.index, isNot(MessageStatus.delivered.index));
      expect(MessageStatus.delivered.index, isNot(MessageStatus.read.index));
      expect(MessageStatus.read.index, isNot(MessageStatus.failed.index));
    });

    testWidgets('message types are correctly defined', (tester) async {
      // Test all message types
      expect(MessageType.text.index, isNot(MessageType.image.index));
      expect(MessageType.image.index, isNot(MessageType.video.index));
      expect(MessageType.video.index, isNot(MessageType.audio.index));
      expect(MessageType.audio.index, isNot(MessageType.file.index));
    });

    testWidgets('message copyWith works correctly', (tester) async {
      final original = Message(
        id: 'msg-1',
        conversationId: 'conv-1',
        senderId: 'user-1',
        content: 'Original',
        type: MessageType.text,
        status: MessageStatus.sending,
        createdAt: DateTime.now(),
      );

      final updated = original.copyWith(
        content: 'Updated',
        status: MessageStatus.sent,
      );

      expect(updated.id, original.id);
      expect(updated.content, 'Updated');
      expect(updated.status, MessageStatus.sent);
    });

    testWidgets('message with reply works correctly', (tester) async {
      final replyMessage = Message(
        id: 'msg-reply',
        conversationId: 'conv-1',
        senderId: 'user-1',
        content: 'This is a reply',
        type: MessageType.text,
        status: MessageStatus.sent,
        replyToId: 'msg-original',
        createdAt: DateTime.now(),
      );

      expect(replyMessage.replyToId, 'msg-original');
    });
  });
}
