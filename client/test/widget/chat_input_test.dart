import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:client/presentation/widgets/message/chat_input.dart';

void main() {
  group('ChatInput', () {
    testWidgets('renders correctly', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ChatInput(
              onSend: (_) {},
            ),
          ),
        ),
      );

      expect(find.byType(TextField), findsOneWidget);
      expect(find.byIcon(Icons.add_photo_alternate_outlined), findsOneWidget);
    });

    testWidgets('shows send button when text is entered', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ChatInput(
              onSend: (_) {},
            ),
          ),
        ),
      );

      await tester.enterText(find.byType(TextField), 'Hello');
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 200));

      // After entering text, send button should appear
      expect(find.byIcon(Icons.send), findsOneWidget);
    });

    testWidgets('calls onMediaTap when media button is pressed', (tester) async {
      bool mediaTapped = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ChatInput(
              onSend: (_) {},
              onMediaTap: () => mediaTapped = true,
            ),
          ),
        ),
      );

      await tester.tap(find.byIcon(Icons.add_photo_alternate_outlined));
      await tester.pump();

      expect(mediaTapped, isTrue);
    });
  });
}
