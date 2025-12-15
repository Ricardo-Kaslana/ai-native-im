# 测试指南

## 📋 概述

本指南详细说明了 AI Native IM 项目的测试策略、测试工具、测试流程和最佳实践。

## 🧪 测试策略

### 测试金字塔

```
           /\
          /  \
         / E2E \        <- 少量端到端测试
        /______\
       /        \
      / Integration \   <- 适量集成测试
     /____________\
    /                \
   /   Unit Tests     \  <- 大量单元测试
  /__________________\
```

### 测试类型分布

| 测试类型 | 数量占比 | 执行频率 | 目标 |
|---------|---------|---------|------|
| 单元测试 | 70% | 每次提交 | 验证业务逻辑 |
| 集成测试 | 20% | 每日构建 | 验证模块交互 |
| 端到端测试 | 10% | 发布前 | 验证用户流程 |

## 🛠️ 测试工具和框架

### Flutter 测试框架

```yaml
dev_dependencies:
  flutter_test:
    sdk: flutter
  mockito: ^5.4.4
  integration_test:
    sdk: flutter
  golden_toolkit: ^0.15.0
  flutter_driver: ^3.3.0
```

### 测试工具配置

#### 1. 测试配置文件

```yaml
# test/test_config.dart
class TestConfig {
  static const Duration defaultTimeout = Duration(seconds: 30);
  static const Duration longTimeout = Duration(minutes: 5);
  
  static const String testEmail = 'test@example.com';
  static const String testPassword = 'Test123456!';
  static const String testUsername = 'testuser';
}
```

#### 2. 测试工具类

```dart
// test/test_utils/test_helpers.dart
class TestHelpers {
  static Widget createTestApp({Widget? child}) {
    return ProviderScope(
      child: MaterialApp(
        home: child ?? Container(),
      ),
    );
  }
  
  static Future<void> pumpAndSettle(
    WidgetTester tester, {
    Duration duration = const Duration(milliseconds: 100),
  }) async {
    await tester.pump();
    await tester.pump(duration);
  }
  
  static Finder findByKey(Key key) {
    return find.byKey(key);
  }
}
```

## 📝 单元测试

### 1. Use Case 测试

```dart
// test/unit/features/auth/domain/usecases/login_usecase_test.dart
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:dartz/dartz.dart';

import 'package:ai_native_im_client/src/core/errors/exceptions.dart';
import 'package:ai_native_im_client/src/features/auth/domain/usecases/login_usecase.dart';
import 'package:ai_native_im_client/src/features/auth/domain/repositories/auth_repository.dart';

class MockAuthRepository extends Mock implements AuthRepository {}

void main() {
  group('LoginUseCase', () {
    late LoginUseCase useCase;
    late MockAuthRepository mockRepository;
    
    setUp(() {
      mockRepository = MockAuthRepository();
      useCase = LoginUseCase(mockRepository);
    });
    
    group('login', () {
      test('should return AuthResult when login succeeds', () async {
        // Arrange
        const params = LoginParams(email: 'test@example.com', password: 'password123');
        final authResult = AuthResult(
          user: User(
            id: '1',
            username: 'testuser',
            email: 'test@example.com',
            status: const UserStatus.online(),
            createdAt: DateTime.now(),
          ),
          accessToken: 'access_token',
          refreshToken: 'refresh_token',
          expiresAt: DateTime.now().add(const Duration(minutes: 15)),
        );
        
        when(mockRepository.login(any))
            .thenAnswer((_) async => Right(authResult));
        
        // Act
        final result = await useCase(params);
        
        // Assert
        expect(result.isRight, true);
        result.fold(
          (error) => fail('Expected success but got error: $error'),
          (result) {
            expect(result.user.username, 'testuser');
            expect(result.user.email, 'test@example.com');
          },
        );
        
        verify(mockRepository.login(params)).called(1);
      });
      
      test('should return ValidationException when email is empty', () async {
        // Arrange
        const params = LoginParams(email: '', password: 'password123');
        
        // Act
        final result = await useCase(params);
        
        // Assert
        expect(result.isLeft, true);
        result.fold(
          (error) => expect(error, isA<ValidationException>()),
          (success) => fail('Expected error but got success'),
        );
        
        verifyNever(mockRepository.login(any));
      });
      
      test('should return ValidationException when email is invalid', () async {
        // Arrange
        const params = LoginParams(email: 'invalid-email', password: 'password123');
        
        // Act
        final result = await useCase(params);
        
        // Assert
        expect(result.isLeft, true);
        result.fold(
          (error) => expect(error, isA<ValidationException>()),
          (success) => fail('Expected error but got success'),
        );
        
        verifyNever(mockRepository.login(any));
      });
      
      test('should return ValidationException when password is too short', () async {
        // Arrange
        const params = LoginParams(email: 'test@example.com', password: '123');
        
        // Act
        final result = await useCase(params);
        
        // Assert
        expect(result.isLeft, true);
        result.fold(
          (error) => expect(error, isA<BusinessException>()),
          (success) => fail('Expected error but got success'),
        );
        
        verifyNever(mockRepository.login(any));
      });
    });
  });
}
```

### 2. Repository 测试

```dart
// test/unit/features/auth/data/repositories/auth_repository_impl_test.dart
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';

import 'package:ai_native_im_client/src/features/auth/data/repositories/auth_repository_impl.dart';
import 'package:ai_native_im_client/src/features/auth/data/datasources/auth_local_datasource.dart';
import 'package:ai_native_im_client/src/features/auth/data/datasources/auth_remote_datasource.dart';
import 'package:ai_native_im_client/src/core/network/dio_client.dart';

class MockAuthRemoteDataSource extends Mock implements AuthRemoteDataSource {}
class MockAuthLocalDataSource extends Mock implements AuthLocalDataSource {}
class MockDioClient extends Mock implements DioClient {}

void main() {
  group('AuthRepositoryImpl', () {
    late AuthRepositoryImpl repository;
    late MockAuthRemoteDataSource mockRemoteDataSource;
    late MockAuthLocalDataSource mockLocalDataSource;
    late MockDioClient mockDioClient;
    
    setUp(() {
      mockRemoteDataSource = MockAuthRemoteDataSource();
      mockLocalDataSource = MockAuthLocalDataSource();
      mockDioClient = MockDioClient();
      repository = AuthRepositoryImpl(
        mockRemoteDataSource,
        mockLocalDataSource,
        mockDioClient,
      );
    });
    
    group('login', () {
      test('should save auth data and return AuthResult on successful login', () async {
        // Arrange
        const request = LoginRequestModel(
          email: 'test@example.com',
          password: 'password123',
        );
        final response = AuthResponseModel(
          accessToken: 'access_token',
          refreshToken: 'refresh_token',
          user: UserModel(
            id: '1',
            username: 'testuser',
            email: 'test@example.com',
            status: UserStatus.online,
            createdAt: DateTime.now(),
          ),
          expiresIn: 900,
        );
        
        when(mockRemoteDataSource.login(request))
            .thenAnswer((_) async => response);
        
        // Act
        final result = await repository.login(
          email: 'test@example.com',
          password: 'password123',
        );
        
        // Assert
        expect(result.isRight, true);
        result.fold(
          (error) => fail('Expected success but got error: $error'),
          (authResult) {
            expect(authResult.user.username, 'testuser');
            expect(authResult.accessToken, 'access_token');
          },
        );
        
        verify(mockRemoteDataSource.login(request)).called(1);
        verify(mockLocalDataSource.saveAuthData(response)).called(1);
        verify(mockDioClient.updateAuthToken('access_token')).called(1);
      });
      
      test('should return error on login failure', () async {
        // Arrange
        const request = LoginRequestModel(
          email: 'test@example.com',
          password: 'wrongpassword',
        );
        final error = const BusinessException.invalidCredentials;
        
        when(mockRemoteDataSource.login(request))
            .thenThrow(error);
        
        // Act
        final result = await repository.login(
          email: 'test@example.com',
          password: 'wrongpassword',
        );
        
        // Assert
        expect(result.isLeft, true);
        result.fold(
          (actualError) => expect(actualError, error),
          (success) => fail('Expected error but got success'),
        );
        
        verify(mockRemoteDataSource.login(request)).called(1);
        verifyNever(mockLocalDataSource.saveAuthData(any));
        verifyNever(mockDioClient.updateAuthToken(any));
      });
    });
  });
}
```

### 3. Provider 测试

```dart
// test/unit/features/auth/presentation/providers/auth_provider_test.dart
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:ai_native_im_client/src/features/auth/presentation/providers/auth_provider.dart';
import 'package:ai_native_im_client/src/features/auth/domain/usecases/login_usecase.dart';
import 'package:ai_native_im_client/src/features/auth/domain/usecases/register_usecase.dart';
import 'package:ai_native_im_client/src/features/auth/domain/usecases/logout_usecase.dart';
import 'package:ai_native_im_client/src/features/auth/domain/usecases/get_current_user_usecase.dart';

class MockLoginUseCase extends Mock implements LoginUseCase {}
class MockRegisterUseCase extends Mock implements RegisterUseCase {}
class MockLogoutUseCase extends Mock implements LogoutUseCase {}
class MockGetCurrentUserUseCase extends Mock implements GetCurrentUserUserUseCase {}

void main() {
  group('AuthNotifier', () {
    late AuthNotifier authNotifier;
    late MockLoginUseCase mockLoginUseCase;
    late MockRegisterUseCase mockRegisterUseCase;
    late MockLogoutUseCase mockLogoutUseCase;
    late MockGetCurrentUserUseCase mockGetCurrentUserUseCase;
    
    setUp(() {
      mockLoginUseCase = MockLoginUseCase();
      mockRegisterUseCase = MockRegisterUseCase();
      mockLogoutUseCase = MockLogoutUseCase();
      mockGetCurrentUserUseCase = MockGetCurrentUserUseCase();
      
      authNotifier = AuthNotifier(
        mockLoginUseCase,
        mockRegisterUseCase,
        mockLogoutUseCase,
        mockGetCurrentUserUseCase,
      );
    });
    
    test('initial state should be correct', () {
      expect(authNotifier.state.user, null);
      expect(authNotifier.state.isLoading, false);
      expect(authNotifier.state.isAuthenticated, false);
      expect(authNotifier.state.error, null);
    });
    
    test('should set loading to true when login starts', () async {
      // Arrange
      when(mockLoginUseCase(any))
          .thenAnswer((_) async => Future.delayed(Duration(seconds: 1)));
      
      // Act
      authNotifier.login(email: 'test@example.com', password: 'password');
      
      // Assert
      expect(authNotifier.state.isLoading, true);
      expect(authNotifier.state.error, null);
    });
    
    test('should update state on successful login', () async {
      // Arrange
      final authResult = AuthResult(
        user: User(
          id: '1',
          username: 'testuser',
          email: 'test@example.com',
          status: const UserStatus.online(),
          createdAt: DateTime.now(),
        ),
        accessToken: 'token',
        refreshToken: 'refresh',
        expiresAt: DateTime.now().add(Duration(minutes: 15)),
      );
      
      when(mockLoginUseCase(any))
          .thenAnswer((_) async => Right(authResult));
      
      // Act
      await authNotifier.login(email: 'test@example.com', password: 'password');
      
      // Assert
      expect(authNotifier.state.isLoading, false);
      expect(authNotifier.state.isAuthenticated, true);
      expect(authNotifier.state.user?.username, 'testuser');
      expect(authNotifier.state.error, null);
    });
    
    test('should update state on login failure', () async {
      // Arrange
      const error = BusinessException.invalidCredentials;
      
      when(mockLoginUseCase(any))
          .thenAnswer((_) async => Left(error));
      
      // Act
      await authNotifier.login(email: 'test@example.com', password: 'wrongpassword');
      
      // Assert
      expect(authNotifier.state.isLoading, false);
      expect(authNotifier.state.isAuthenticated, false);
      expect(authNotifier.state.user, null);
      expect(authNotifier.state.error, error);
    });
    
    test('should clear error when clearError is called', () {
      // Arrange
      const error = BusinessException.invalidCredentials;
      authNotifier.state = authNotifier.state.copyWith(error: error);
      
      // Act
      authNotifier.clearError();
      
      // Assert
      expect(authNotifier.state.error, null);
    });
  });
}
```

## 🎨 Widget 测试

### 1. 基础 Widget 测试

```dart
// test/widget/shared/widgets/common/loading_widget_test.dart
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:ai_native_im_client/src/shared/widgets/common/loading_widget.dart';

void main() {
  group('LoadingWidget', () {
    testWidgets('should display loading indicator with message', (tester) async {
      // Arrange
      const message = 'Loading...';
      
      // Act
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: LoadingWidget(message: message),
          ),
        ),
      );
      
      // Assert
      expect(find.byType(CircularProgressIndicator), findsOneWidget);
      expect(find.text(message), findsOneWidget);
    });
    
    testWidgets('should display different sizes', (tester) async {
      // Arrange & Act
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Column(
              children: [
                LoadingWidget(size: LoadingSize.small),
                LoadingWidget(size: LoadingSize.medium),
                LoadingWidget(size: LoadingSize.large),
              ],
            ),
          ),
        ),
      );
      
      // Assert
      final indicators = find.byType(CircularProgressIndicator);
      expect(indicators, findsNWidgets(3));
    });
  });
}
```

### 2. 表单 Widget 测试

```dart
// test/widget/features/auth/presentation/widgets/login_form_test.dart
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:ai_native_im_client/src/features/auth/presentation/widgets/login_form.dart';

void main() {
  group('LoginForm', () {
    late TextEditingController emailController;
    late TextEditingController passwordController;
    
    setUp(() {
      emailController = TextEditingController();
      passwordController = TextEditingController();
    });
    
    tearDown(() {
      emailController.dispose();
      passwordController.dispose();
    });
    
    testWidgets('should validate empty fields', (tester) async {
      // Arrange
      bool loginCalled = false;
      
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: LoginForm(
              emailController: emailController,
              passwordController: passwordController,
              isPasswordVisible: false,
              onPasswordVisibilityChanged: (visible) {},
              onLogin: (email, password) {
                loginCalled = true;
              },
            ),
          ),
        ),
      );
      
      // Act
      await tester.tap(find.byType(ElevatedButton));
      await tester.pump();
      
      // Assert
      expect(find.text('请输入邮箱'), findsOneWidget);
      expect(find.text('请输入密码'), findsOneWidget);
      expect(loginCalled, false);
    });
    
    testWidgets('should validate email format', (tester) async {
      // Arrange
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: LoginForm(
              emailController: emailController,
              passwordController: passwordController,
              isPasswordVisible: false,
              onPasswordVisibilityChanged: (visible) {},
              onLogin: (email, password) {},
            ),
          ),
        ),
      );
      
      // Act
      await tester.enterText(find.byKey(Key('email-field')), 'invalid-email');
      await tester.tap(find.byType(ElevatedButton));
      await tester.pump();
      
      // Assert
      expect(find.text('请输入有效的邮箱地址'), findsOneWidget);
    });
    
    testWidgets('should validate password length', (tester) async {
      // Arrange
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: LoginForm(
              emailController: emailController,
              passwordController: passwordController,
              isPasswordVisible: false,
              onPasswordVisibilityChanged: (visible) {},
              onLogin: (email, password) {},
            ),
          ),
        ),
      );
      
      // Act
      await tester.enterText(find.byKey(Key('email-field')), 'test@example.com');
      await tester.enterText(find.byKey(Key('password-field')), '123');
      await tester.tap(find.byType(ElevatedButton));
      await tester.pump();
      
      // Assert
      expect(find.text('密码至少需要6个字符'), findsOneWidget);
    });
    
    testWidgets('should call onLogin with valid input', (tester) async {
      // Arrange
      String? calledEmail;
      String? calledPassword;
      
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: LoginForm(
              emailController: emailController,
              passwordController: passwordController,
              isPasswordVisible: false,
              onPasswordVisibilityChanged: (visible) {},
              onLogin: (email, password) {
                calledEmail = email;
                calledPassword = password;
              },
            ),
          ),
        ),
      );
      
      // Act
      await tester.enterText(find.byKey(Key('email-field')), 'test@example.com');
      await tester.enterText(find.byKey(Key('password-field')), 'password123');
      await tester.tap(find.byType(ElevatedButton));
      await tester.pump();
      
      // Assert
      expect(calledEmail, 'test@example.com');
      expect(calledPassword, 'password123');
    });
  });
}
```

## 🔗 集成测试

### 1. API 集成测试

```dart
// test/integration/api/auth_api_test.dart
import 'package:flutter_test/flutter_test.dart';
import 'package:dio/dio.dart';

import 'package:ai_native_im_client/src/core/network/dio_client.dart';

void main() {
  group('Auth API Integration', () {
    late DioClient dioClient;
    
    setUpAll(() {
      dioClient = DioClient();
    });
    
    test('should register new user successfully', () async {
      // Arrange
      final timestamp = DateTime.now().millisecondsSinceEpoch;
      
      // Act
      try {
        final response = await dioClient.post<Map<String, dynamic>>(
        '/auth/register',
        data: {
          'username': 'testuser_$timestamp',
          'email': 'test_$timestamp@example.com',
          'password': 'Test123456!',
        },
      );
        
        // Assert
        expect(response.statusCode, 200);
        expect(response.data!['success'], true);
        expect(response.data!['data']['user']['username'], 'testuser_$timestamp');
      } catch (e) {
        fail('API call failed: $e');
      }
    });
    
    test('should login with valid credentials', () async {
      // Arrange
      final timestamp = DateTime.now().millisecondsSinceEpoch;
      
      // First register user
      await dioClient.post<Map<String, dynamic>>(
        '/auth/register',
        data: {
          'username': 'testuser_$timestamp',
          'email': 'test_$timestamp@example.com',
          'password': 'Test123456!',
        },
      );
      
      // Act
      try {
        final response = await dioClient.post<Map<String, dynamic>>(
          '/auth/login',
          data: {
            'email': 'test_$timestamp@example.com',
            'password': 'Test123456!',
          },
        );
        
        // Assert
        expect(response.statusCode, 200);
        expect(response.data!['success'], true);
        expect(response.data!['data']['access_token'], isNotEmpty);
        expect(response.data!['data']['refresh_token'], isNotEmpty);
      } catch (e) {
        fail('Login failed: $e');
      }
    });
    
    test('should reject invalid credentials', () async {
      // Act
      try {
        final response = await dioClient.post<Map<String, dynamic>>(
          '/auth/login',
          data: {
            'email': 'nonexistent@example.com',
            'password': 'wrongpassword',
          },
        );
        
        // Assert
        expect(response.statusCode, 401);
        expect(response.data!['success'], false);
        expect(response.data!['error']['code'], 2003);
      } catch (e) {
        fail('API call failed: $e');
      }
    });
  });
}
```

### 2. WebSocket 集成测试

```dart
// test/integration/websocket/websocket_test.dart
import 'package:flutter_test/flutter_test.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

import 'package:ai_native_im_client/src/core/network/dio_client.dart';

void main() {
  group('WebSocket Integration', () {
    late String accessToken;
    
    setUpAll(() async {
      // Get access token by registering and logging in
      final dioClient = DioClient();
      
      final timestamp = DateTime.now().millisecondsSinceEpoch;
      final registerResponse = await dioClient.post<Map<String, dynamic>>(
        '/auth/register',
        data: {
          'username': 'wstest_$timestamp',
          'email': 'wstest_$timestamp@example.com',
          'password': 'Test123456!',
        },
      );
      
      accessToken = registerResponse.data!['data']['access_token'];
    });
    
    test('should establish WebSocket connection', () async {
      // Arrange
      final uri = Uri.parse('ws://localhost:8080/ws?token=$accessToken');
      final channel = WebSocketChannel.connect(uri);
      
      // Act & Assert
      expect(channel.ready, completes);
      
      // Cleanup
      await channel.sink.close();
    });
    
    test('should send and receive heartbeat', () async {
      // Arrange
      final uri = Uri.parse('ws://localhost:8080/ws?token=$accessToken');
      final channel = WebSocketChannel.connect(uri);
      
      bool messageReceived = false;
      
      channel.stream.listen(
        (message) {
          messageReceived = true;
        },
        onError: (error) => fail('WebSocket error: $error'),
      );
      
      // Act
      final heartbeat = {
        'heartbeat': {
          'timestamp': DateTime.now().toIso8601String(),
        }
      };
      
      channel.sink.add(jsonEncode(heartbeat));
      
      // Wait for response
      await Future.delayed(Duration(seconds: 2));
      
      // Assert
      expect(messageReceived, true);
      
      // Cleanup
      await channel.sink.close();
    });
  });
}
```

## 📱 端到端测试

### 1. 用户流程测试

```dart
// integration_test/app_test.dart
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'package:ai_native_im_client/main.dart' as app;

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  
  group('App E2E Tests', () {
    testWidgets('complete user registration and login flow', (tester) async {
      // Start app
      app.main();
      await tester.pumpAndSettle();
      
      // Should see login screen
      expect(find.byType(LoginPage), findsOneWidget);
      
      // Navigate to registration
      await tester.tap(find.text('还没有账号？ 立即注册'));
      await tester.pumpAndSettle();
      
      // Should see registration screen
      expect(find.byType(RegisterPage), findsOneWidget);
      
      // Fill registration form
      await tester.enterText(find.byKey(Key('username-field')), 'testuser');
      await tester.enterText(find.byKey(Key('email-field')), 'test@example.com');
      await tester.enterText(find.byKey(Key('password-field')), 'Test123456!');
      await tester.enterText(find.byKey(Key('confirm-password-field')), 'Test123456!');
      
      // Submit registration
      await tester.tap(find.byType(ElevatedButton));
      await tester.pumpAndSettle(Duration(seconds: 3));
      
      // Should be logged in and see home screen
      expect(find.byType(HomeScreen), findsOneWidget);
      expect(find.text('testuser'), findsOneWidget);
      
      // Logout
      await tester.tap(find.byIcon(Icons.logout));
      await tester.pumpAndSettle();
      
      // Should be back to login screen
      expect(find.byType(LoginPage), findsOneWidget);
    });
    
    testWidgets('login with invalid credentials shows error', (tester) async {
      // Start app
      app.main();
      await tester.pumpAndSettle();
      
      // Try to login with invalid credentials
      await tester.enterText(find.byKey(Key('email-field')), 'invalid@example.com');
      await tester.enterText(find.byKey(Key('password-field')), 'wrongpassword');
      await tester.tap(find.byType(ElevatedButton));
      await tester.pumpAndSettle();
      
      // Should show error message
      expect(find.text('用户名或密码错误'), findsOneWidget);
      expect(find.byType(LoginPage), findsOneWidget);
    });
  });
}
```

## 🏃 性能测试

### 1. 启动性能测试

```dart
// test/performance/startup_performance_test.dart
import 'package:flutter/material.dart';
import 'package:flutter_driver/flutter_driver.dart';
import 'package:test/performance/performance_test_utils.dart';

void main() {
  group('Startup Performance', () {
    FlutterDriver driver;
    
    setUpAll(() async {
      driver = await FlutterDriver.connect();
    });
    
    tearDownAll(() async {
      if (driver != null) {
        await driver?.close();
      }
    });
    
    test('cold startup time should be under 3 seconds', () async {
      // Record startup timeline
      final timeline = await driver.traceAction(() async {
        // Start app
        await driver.waitUntilNoTransientCallbacks();
      });
      
      // Analyze timeline
      final startupTime = timeline.events
          .where((event) => event.name == 'Frame')
          .length;
      
      // Assert startup time (assuming 60 FPS)
      expect(startupTime, lessThan(180)); // 3 seconds at 60 FPS
    });
    
    test('hot restart time should be under 1 second', () async {
      // Perform hot restart
      await driver.requestData('restart');
      await driver.waitUntilNoTransientCallbacks();
      
      // Record timeline
      final timeline = await driver.traceAction(() async {
        // Wait for app to be ready
        await driver.waitFor(find.byType(MaterialApp));
      });
      
      // Analyze timeline
      final restartTime = timeline.events
          .where((event) => event.name == 'Frame')
          .length;
      
      // Assert restart time
      expect(restartTime, lessThan(60)); // 1 second at 60 FPS
    });
  });
}
```

### 2. 滚动性能测试

```dart
// test/performance/scrolling_performance_test.dart
import 'package:flutter/material.dart';
import 'package:flutter_driver/flutter_driver.dart';

void main() {
  group('Scrolling Performance', () {
    FlutterDriver driver;
    
    setUpAll(() async {
      driver = await FlutterDriver.connect();
    });
    
    tearDownAll(() async {
      if (driver != null) {
        await driver?.close();
      }
    });
    
    test('scrolling should maintain 60 FPS', () async {
      // Navigate to message list
      await driver.tap(find.byTooltip('Messages'));
      await driver.waitUntilNoTransientCallbacks();
      
      // Record scrolling timeline
      final timeline = await driver.traceAction(() async {
        // Scroll down
        for (int i = 0; i < 10; i++) {
          await driver.scroll(find.byType(ListView), 0, 300);
          await driver.pump(Duration(milliseconds: 100));
        }
      });
      
      // Analyze frame times
      final frames = timeline.events
          .where((event) => event.name == 'Frame')
          .toList();
      
      // Calculate average frame time
      final avgFrameTime = frames.fold<int>(0, (sum, frame) {
        return sum + frame.duration.inMicroseconds;
      }) / frames.length;
      
      // Convert to FPS
      final avgFPS = 1000000 / avgFrameTime;
      
      // Assert performance
      expect(avgFPS, greaterThan(55)); // At least 55 FPS
    });
  });
}
```

## 📊 测试覆盖率

### 配置覆盖率收集

```yaml
# pubspec.yaml
dev_dependencies:
  flutter_test:
    sdk: flutter
  test_coverage: ^0.2.0
```

### 运行覆盖率测试

```bash
# 运行所有测试并生成覆盖率报告
flutter test --coverage

# 生成 HTML 覆盖率报告
genhtml coverage/lcov.info -o coverage/html

# 查看覆盖率报告
open coverage/html/index.html
```

### 覆盖率目标

| 模块 | 目标覆盖率 | 当前覆盖率 |
|------|-----------|-----------|
| Domain (Use Cases) | 95% | 92% |
| Data (Repositories) | 90% | 88% |
| Presentation (Providers) | 85% | 83% |
| UI (Widgets) | 80% | 75% |
| Utils | 90% | 91% |

## 🚀 CI/CD 集成

### GitHub Actions 配置

```yaml
# .github/workflows/test.yml
name: Flutter Tests

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Flutter
      uses: subosito/flutter-action@v2
      with:
        flutter-version: '3.24.5'
        
    - name: Install dependencies
      run: flutter pub get
      
    - name: Run unit tests
      run: flutter test --coverage
      
    - name: Run widget tests
      run: flutter test --coverage integration_test/
      
    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v3
      with:
        file: coverage/lcov.info
```

## 📋 测试检查清单

### 提交前检查

- [ ] 所有单元测试通过
- [ ] 代码覆盖率达标
- [ ] 新功能包含对应测试
- [ ] 测试文档已更新

### 发布前检查

- [ ] 所有集成测试通过
- [ ] 端到端测试通过
- [ ] 性能测试达标
- [ ] 设备兼容性测试通过

## 🔧 测试最佳实践

### 1. 测试编写原则

- **AAA 模式**: Arrange, Act, Assert
- **单一职责**: 每个测试只验证一个功能
- **独立性**: 测试之间不应相互依赖
- **可读性**: 测试名称应清晰描述测试内容

### 2. Mock 使用原则

- 只 mock 外部依赖
- 保持 mock 行为简单
- 使用 `verify()` 验证交互
- 避免过度 mock

### 3. 测试数据管理

- 使用工厂模式创建测试数据
- 保持测试数据的一致性
- 避免硬编码测试值
- 使用常量定义测试配置

### 4. 异步测试

- 使用 `await` 等待异步操作
- 使用 `pumpAndSettle()` 等待 UI 更新
- 设置合理的超时时间
- 处理异常情况

## 📚 常见问题解决

### 1. 测试超时

```dart
// 增加超时设置
testWidgets('async test with timeout', (tester) async {
  await tester.pumpAndSettle(Duration(seconds: 10));
}, timeout: Timeout(Duration(seconds: 15)));
```

### 2. Widget 查找失败

```dart
// 使用 Key 定位 Widget
ElevatedButton(
  key: Key('login-button'),
  onPressed: () {},
  child: Text('Login'),
),

// 在测试中查找
await tester.tap(find.byKey(Key('login-button')));
```

### 3. Provider 测试

```dart
// 使用 ProviderScope 包装 Widget
testWidgets('Provider test', (tester) async {
  await tester.pumpWidget(
    ProviderScope(
      overrides: [
        authProvider.overrideWithValue(mockAuthProvider),
      ],
      child: MyApp(),
    ),
  );
});
```

---

**文档版本**: v1.0  
**最后更新**: 2025-12-12  
**维护者**: 开发团队