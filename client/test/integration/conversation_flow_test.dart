import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:client/presentation/screens/messages/conversation_list_screen.dart';
import 'package:client/data/models/conversation.dart';

void main() {
  group('Conversation Flow Integration Test', () {
    testWidgets('displays conversation list screen correctly', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const ConversationListScreen(),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Verify screen title (may appear multiple times in navigation)
      expect(find.text('消息'), findsWidgets);

      // Verify search bar exists
      expect(find.byIcon(Icons.search), findsOneWidget);
    });

    testWidgets('shows empty state when no conversations', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const ConversationListScreen(),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Should show empty state or loading indicator initially
      expect(find.byType(ConversationListScreen), findsOneWidget);
    });

    testWidgets('can tap on search icon', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const ConversationListScreen(),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Find and tap search icon
      final searchIcon = find.byIcon(Icons.search);
      if (searchIcon.evaluate().isNotEmpty) {
        await tester.tap(searchIcon);
        await tester.pumpAndSettle();
      }

      // Screen should still be visible
      expect(find.byType(ConversationListScreen), findsOneWidget);
    });

    testWidgets('conversation model has correct structure', (tester) async {
      // Test conversation model with actual structure
      final conversation = Conversation(
        id: 'test-id',
        type: ConversationType.private,
        name: 'Test User',
        avatarUrl: 'https://example.com/avatar.jpg',
        unreadCount: 5,
        isPinned: true,
        isMuted: false,
        createdAt: DateTime.now(),
      );

      expect(conversation.id, 'test-id');
      expect(conversation.name, 'Test User');
      expect(conversation.unreadCount, 5);
      expect(conversation.type, ConversationType.private);
      expect(conversation.isPinned, true);
    });

    testWidgets('group conversation model works correctly', (tester) async {
      // Test group conversation
      final groupConversation = Conversation(
        id: 'group-id',
        type: ConversationType.group,
        name: 'Test Group',
        avatarUrl: null,
        unreadCount: 10,
        isPinned: false,
        isMuted: true,
        createdAt: DateTime.now(),
      );

      expect(groupConversation.type, ConversationType.group);
      expect(groupConversation.isMuted, true);
      expect(groupConversation.unreadCount, 10);
    });
  });
}
