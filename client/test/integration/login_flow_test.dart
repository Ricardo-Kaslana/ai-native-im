import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:client/app.dart';

void main() {
  group('Login Flow Integration Test', () {
    testWidgets('shows login screen on app start', (tester) async {
      await tester.pumpWidget(
        const ProviderScope(
          child: MyApp(),
        ),
      );

      // Verify login screen is displayed
      expect(find.text('AI Native IM'), findsOneWidget);
      // Find the login button specifically (AppButton with text '登录')
      expect(find.widgetWithText(ElevatedButton, '登录'), findsOneWidget);
    });

    testWidgets('can navigate to register screen', (tester) async {
      // Set a larger screen size to ensure all elements are visible
      tester.view.physicalSize = const Size(1080, 1920);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      await tester.pumpWidget(
        const ProviderScope(
          child: MyApp(),
        ),
      );

      // Scroll down to make sure the register link is visible
      await tester.drag(find.byType(SingleChildScrollView), const Offset(0, -200));
      await tester.pumpAndSettle();

      // Tap on register link
      final registerLink = find.text('立即注册');
      expect(registerLink, findsOneWidget);
      await tester.tap(registerLink);
      await tester.pumpAndSettle();

      // Verify register screen is displayed
      expect(find.text('创建账号'), findsOneWidget);
      // Find the register button specifically
      expect(find.widgetWithText(ElevatedButton, '注册'), findsOneWidget);
    });

    testWidgets('shows validation errors for empty fields', (tester) async {
      // Set a larger screen size
      tester.view.physicalSize = const Size(1080, 1920);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      await tester.pumpWidget(
        const ProviderScope(
          child: MyApp(),
        ),
      );

      // Scroll to make sure the login button is visible
      await tester.drag(find.byType(SingleChildScrollView), const Offset(0, -100));
      await tester.pumpAndSettle();

      // Tap login button (ElevatedButton) without entering credentials
      final loginButton = find.widgetWithText(ElevatedButton, '登录');
      expect(loginButton, findsOneWidget);
      await tester.tap(loginButton);
      await tester.pumpAndSettle();

      // Verify validation errors are shown (at least one instance of each error)
      // Note: '请输入邮箱' appears once as error, '请输入密码' appears twice (hint + error)
      expect(find.text('请输入邮箱'), findsWidgets);
      expect(find.text('请输入密码'), findsWidgets);
    });

    testWidgets('shows email validation error for invalid email', (tester) async {
      // Set a larger screen size
      tester.view.physicalSize = const Size(1080, 1920);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      await tester.pumpWidget(
        const ProviderScope(
          child: MyApp(),
        ),
      );

      // Enter invalid email
      await tester.enterText(
        find.byType(TextFormField).first,
        'invalid-email',
      );
      await tester.enterText(
        find.byType(TextFormField).last,
        'password123',
      );

      // Scroll to make sure the login button is visible
      await tester.drag(find.byType(SingleChildScrollView), const Offset(0, -100));
      await tester.pumpAndSettle();

      // Tap login button (ElevatedButton)
      final loginButton = find.widgetWithText(ElevatedButton, '登录');
      expect(loginButton, findsOneWidget);
      await tester.tap(loginButton);
      await tester.pumpAndSettle();

      // Verify email validation error
      expect(find.text('请输入有效的邮箱地址'), findsOneWidget);
    });

    testWidgets('can navigate back from register screen', (tester) async {
      // Set a larger screen size
      tester.view.physicalSize = const Size(1080, 1920);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(() {
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
      });

      await tester.pumpWidget(
        const ProviderScope(
          child: MyApp(),
        ),
      );

      // Navigate to register screen
      await tester.drag(find.byType(SingleChildScrollView), const Offset(0, -200));
      await tester.pumpAndSettle();
      await tester.tap(find.text('立即注册'));
      await tester.pumpAndSettle();

      // Verify on register screen
      expect(find.text('创建账号'), findsOneWidget);

      // Tap back button or login link
      await tester.tap(find.text('立即登录'));
      await tester.pumpAndSettle();

      // Verify back on login screen
      expect(find.text('AI Native IM'), findsOneWidget);
    });
  });
}
