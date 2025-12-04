import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:client/app.dart';

void main() {
  testWidgets('App should render login screen', (WidgetTester tester) async {
    // Build our app and trigger a frame.
    await tester.pumpWidget(
      const ProviderScope(
        child: MyApp(),
      ),
    );

    // Verify that the login screen is displayed
    expect(find.text('AI Native IM'), findsOneWidget);
    expect(find.text('登录'), findsOneWidget);
  });
}
