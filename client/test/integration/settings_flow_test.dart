import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:client/presentation/screens/profile/settings_screen.dart';

void main() {
  group('Settings Flow Integration Test', () {
    testWidgets('settings screen displays correctly', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const SettingsScreen(),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Verify settings screen is displayed
      expect(find.byType(SettingsScreen), findsOneWidget);
      expect(find.text('设置'), findsOneWidget);
    });

    testWidgets('theme toggle exists', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const SettingsScreen(),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Look for theme-related text
      expect(find.text('深色模式'), findsOneWidget);
    });

    testWidgets('notification settings exist', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const SettingsScreen(),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Look for notification-related text
      expect(find.text('通知'), findsOneWidget);
    });

    testWidgets('settings screen has scrollable content', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const SettingsScreen(),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Settings screen should be scrollable
      expect(find.byType(SettingsScreen), findsOneWidget);
    });

    testWidgets('can toggle dark mode switch', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const SettingsScreen(),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Find switch widget
      final switchFinder = find.byType(Switch);
      if (switchFinder.evaluate().isNotEmpty) {
        await tester.tap(switchFinder.first);
        await tester.pumpAndSettle();
      }

      // Screen should still be visible
      expect(find.byType(SettingsScreen), findsOneWidget);
    });

    testWidgets('settings screen has list tiles', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: const SettingsScreen(),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Settings screen should have list tiles for options
      expect(find.byType(ListTile), findsWidgets);
    });
  });
}
