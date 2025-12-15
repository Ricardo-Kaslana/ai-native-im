# 客户端架构文档

## 📋 概述

AI Native IM 客户端采用 Clean Architecture 设计模式，结合 Flutter 的响应式编程特性，构建了一个可维护、可扩展、高性能的移动应用架构。

## 🏗️ 整体架构

### 分层架构图

```
┌─────────────────────────────────────────────────────────────┐
│                    Presentation Layer                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   UI Components  │  │    Screens      │  │    Widgets      │ │
│  │   (Stateless)    │  │  (Stateful)     │  │   (Reusable)    │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    Business Layer                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   Use Cases      │  │  ViewModels      │  │   Providers     │ │
│  │ (Business Logic) │  │ (State Mgmt)    │  │ (Riverpod)      │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                     Data Layer                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   Repositories   │  │ Data Sources    │  │    Models       │ │
│  │ (Data Access)   │  │ (Remote/Local)  │  │ (Entities)      │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                Infrastructure Layer                         │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   Network        │  │   Storage       │  │   DI Container  │ │
│  │ (Dio/WebSocket)  │  │ (Secure/Hive)   │  │ (GetIt)         │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 架构原则

1. **依赖倒置**: 高层模块不依赖低层模块，都依赖抽象
2. **单一职责**: 每个类只有一个变化的理由
3. **开闭原则**: 对扩展开放，对修改关闭
4. **接口隔离**: 客户端不应依赖它不需要的接口
5. **依赖注入**: 使用 GetIt 管理依赖关系

## 📁 目录结构详解

```
lib/
├── src/
│   ├── core/                    # 核心基础设施
│   │   ├── constants/           # 常量定义
│   │   │   ├── api_constants.dart
│   │   │   ├── app_constants.dart
│   │   │   └── storage_constants.dart
│   │   ├── di/                  # 依赖注入
│   │   │   └── injection_container.dart
│   │   ├── errors/              # 错误处理
│   │   │   ├── exceptions/
│   │   │   │   ├── app_exception.dart
│   │   │   │   ├── network_exception.dart
│   │   │   │   └── business_exception.dart
│   │   │   └── handlers/
│   │   │       ├── error_handler.dart
│   │   │       └── logger.dart
│   │   ├── network/             # 网络配置
│   │   │   ├── dio_client.dart
│   │   │   └── interceptors/
│   │   │       ├── auth_interceptor.dart
│   │   │       ├── error_interceptor.dart
│   │   │       └── logging_interceptor.dart
│   │   ├── routing/             # 路由配置
│   │   │   └── app_router.dart
│   │   └── utils/               # 工具类
│   │       ├── date_utils.dart
│   │       ├── validation_utils.dart
│   │       └── image_utils.dart
│   ├── shared/                  # 共享组件
│   │   ├── widgets/             # 通用组件
│   │   │   ├── common/
│   │   │   │   ├── loading_widget.dart
│   │   │   │   ├── error_widget.dart
│   │   │   │   └── empty_widget.dart
│   │   │   ├── chat/
│   │   │   │   ├── message_bubble.dart
│   │   │   │   ├── typing_indicator.dart
│   │   │   │   └── message_input.dart
│   │   │   └── user/
│   │   │       ├── user_avatar.dart
│   │   │       ├── user_status.dart
│   │   │       └── user_info_card.dart
│   │   ├── themes/              # 主题配置
│   │   │   ├── app_theme.dart
│   │   │   ├── light_theme.dart
│   │   │   ├── dark_theme.dart
│   │   │   └── theme_extensions.dart
│   │   └── providers/           # 全局Provider
│   │       ├── app_providers.dart
│   │       ├── theme_provider.dart
│   │       └── locale_provider.dart
│   ├── features/                # 功能模块
│   │   └── auth/                # 认证模块示例
│   │       ├── data/            # 数据层
│   │       │   ├── datasources/
│   │       │   │   ├── auth_local_datasource.dart
│   │       │   │   └── auth_remote_datasource.dart
│   │       │   ├── models/
│   │       │   │   ├── user_model.dart
│   │       │   │   └── auth_response_model.dart
│   │       │   └── repositories/
│   │       │       └── auth_repository_impl.dart
│   │       ├── domain/          # 业务层
│   │       │   ├── entities/
│   │       │   │   ├── user.dart
│   │       │   │   ├── auth_result.dart
│   │       │   │   └── user_status.dart
│   │       │   ├── repositories/
│   │       │   │   └── auth_repository.dart
│   │       │   └── usecases/
│   │       │       ├── login_usecase.dart
│   │       │       ├── register_usecase.dart
│   │       │       └── logout_usecase.dart
│   │       └── presentation/    # 表现层
│   │           ├── pages/
│   │           │   ├── login_page.dart
│   │           │   └── register_page.dart
│   │           ├── widgets/
│   │           │   ├── login_form.dart
│   │           │   └── register_form.dart
│   │           └── providers/
│   │               └── auth_provider.dart
│   └── generated/               # 自动生成代码
│       └── proto/               # Protobuf生成
│           ├── chat.pb.dart
│           ├── chat.pbenum.dart
│           └── chat.pbjson.dart
├── test/                        # 测试文件
│   ├── unit/
│   ├── widget/
│   └── integration/
├── assets/                      # 资源文件
│   ├── images/
│   ├── fonts/
│   └── animations/
└── main.dart                    # 应用入口
```

## 🔧 核心组件详解

### 1. 依赖注入容器

使用 GetIt 管理所有依赖关系：

```dart
// injection_container.dart
Future<void> configureDependencies() async {
  // Core services
  getIt.registerLazySingleton<FlutterSecureStorage>(
    () => const FlutterSecureStorage(),
  );
  
  // Network layer
  getIt.registerLazySingleton<DioClient>(() => DioClient());
  
  // Repositories
  getIt.registerLazySingleton<AuthRepository>(
    () => AuthRepositoryImpl(
      getIt<AuthRemoteDataSource>(),
      getIt<AuthLocalDataSource>(),
      getIt<DioClient>(),
    ),
  );
  
  // Use cases
  getIt.registerLazySingleton<LoginUseCase>(
    () => LoginUseCase(getIt<AuthRepository>()),
  );
  
  // Providers
  getIt.registerFactory<AuthNotifier>(
    () => AuthNotifier(
      getIt<LoginUseCase>(),
      getIt<RegisterUseCase>(),
      getIt<LogoutUseCase>(),
      getIt<GetCurrentUserUseCase>(),
    ),
  );
}
```

### 2. 网络层架构

#### Dio 客户端配置

```dart
class DioClient {
  late final Dio _dio;
  
  DioClient() {
    _dio = Dio(BaseOptions(
      baseUrl: ApiConstants.baseUrl,
      connectTimeout: AppConstants.apiTimeout,
      receiveTimeout: AppConstants.apiTimeout,
      headers: {
        ApiConstants.contentTypeHeader: ApiConstants.jsonContentType,
      },
    ));
    
    _setupInterceptors();
  }
  
  void _setupInterceptors() {
    _dio.interceptors.addAll([
      LoggingInterceptor(),
      ErrorInterceptor(),
      AuthInterceptor(),
    ]);
  }
}
```

#### 认证拦截器

```dart
class AuthInterceptor extends Interceptor {
  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    _addAuthorizationHeader(options);
    handler.next(options);
  }
  
  @override
  void onError(DioException err, ErrorInterceptorHandler handler) {
    if (err.response?.statusCode == 401) {
      _handleUnauthorizedError(err, handler);
    } else {
      handler.next(err);
    }
  }
  
  Future<void> _handleUnauthorizedError(
    DioException err,
    ErrorInterceptorHandler handler,
  ) async {
    // 自动刷新令牌逻辑
    final newToken = await _refreshToken();
    if (newToken != null) {
      // 重试原始请求
      handler.resolve(await _retryRequest(err.requestOptions, newToken));
    } else {
      handler.next(err);
    }
  }
}
```

### 3. 状态管理架构

#### Riverpod Provider 示例

```dart
// 认证状态
class AuthState {
  const AuthState({
    this.user,
    this.isLoading = false,
    this.error,
    this.isAuthenticated = false,
  });
  
  // 状态属性
  final User? user;
  final bool isLoading;
  final AppException? error;
  final bool isAuthenticated;
  
  // 复制方法
  AuthState copyWith({
    User? user,
    bool? isLoading,
    AppException? error,
    bool? isAuthenticated,
  }) {
    return AuthState(
      user: user ?? this.user,
      isLoading: isLoading ?? this.isLoading,
      error: error ?? this.error,
      isAuthenticated: isAuthenticated ?? this.isAuthenticated,
    );
  }
}

// 认证通知器
class AuthNotifier extends StateNotifier<AuthState> {
  final LoginUseCase _loginUseCase;
  final RegisterUseCase _registerUseCase;
  final LogoutUseCase _logoutUseCase;
  
  AuthNotifier(
    this._loginUseCase,
    this._registerUseCase,
    this._logoutUseCase,
  ) : super(const AuthState());
  
  Future<void> login({required String email, required String password}) async {
    state = state.copyWith(isLoading: true, error: null);
    
    final result = await _loginUseCase(LoginParams(email: email, password: password));
    
    result.fold(
      (error) => state = state.copyWith(isLoading: false, error: error),
      (authResult) => state = state.copyWith(
        user: authResult.user,
        isLoading: false,
        isAuthenticated: true,
      ),
    );
  }
}

// Provider 定义
final authProvider = StateNotifierProvider<AuthNotifier, AuthState>((ref) {
  return getIt<AuthNotifier>();
});
```

### 4. 数据流架构

#### Repository 实现

```dart
class AuthRepositoryImpl implements AuthRepository {
  final AuthRemoteDataSource _remoteDataSource;
  final AuthLocalDataSource _localDataSource;
  
  AuthRepositoryImpl(this._remoteDataSource, this._localDataSource);
  
  @override
  Future<Either<AppException, AuthResult>> login({
    required String email,
    required String password,
  }) async {
    try {
      final response = await _remoteDataSource.login(
        LoginRequestModel(email: email, password: password),
      );
      
      // 保存到本地
      await _localDataSource.saveAuthData(response);
      
      return Right(AuthResult.fromModel(response));
    } on AppException catch (e) {
      return Left(e);
    } catch (e) {
      return Left(const SystemException.unknownError);
    }
  }
}
```

#### Use Case 实现

```dart
class LoginUseCase {
  final AuthRepository _repository;
  
  LoginUseCase(this._repository);
  
  Future<Either<AppException, AuthResult>> call(LoginParams params) async {
    // 输入验证
    final validationError = _validateParams(params);
    if (validationError != null) {
      return Left(validationError);
    }
    
    // 调用 Repository
    return await _repository.login(
      email: params.email,
      password: params.password,
    );
  }
  
  AppException? _validateParams(LoginParams params) {
    if (params.email.isEmpty) {
      return const ValidationException.emptyField;
    }
    
    if (!_isValidEmail(params.email)) {
      return const ValidationException.invalidEmail;
    }
    
    return null;
  }
}
```

## 🔄 数据流示例

### 用户登录流程

```
1. UI Layer (LoginPage)
   ↓ 用户点击登录
2. Presentation Layer (AuthNotifier)
   ↓ 调用 Use Case
3. Business Layer (LoginUseCase)
   ↓ 验证输入
4. Data Layer (AuthRepository)
   ↓ 调用 Remote DataSource
5. Infrastructure Layer (AuthRemoteDataSource)
   ↓ 发送 HTTP 请求
6. Backend API
   ↓ 返回响应
7. Infrastructure Layer (AuthRemoteDataSource)
   ↓ 解析响应
8. Data Layer (AuthRepository)
   ↓ 保存到本地存储
9. Business Layer (LoginUseCase)
   ↓ 返回结果
10. Presentation Layer (AuthNotifier)
    ↓ 更新状态
11. UI Layer (LoginPage)
    ↓ 更新界面
```

## 🎨 UI 架构模式

### 1. 组件层次结构

```
App (MaterialApp)
├── AppRouter (GoRouter)
├── MainScreen (ConsumerWidget)
│   ├── NavigationRail (导航)
│   ├── ConversationList (会话列表)
│   └── ChatScreen (聊天界面)
│       ├── MessageList (消息列表)
│       ├── MessageInput (输入框)
│       └── MessageBubble (消息气泡)
└── Theme (MaterialApp主题)
```

### 2. 状态管理最佳实践

```dart
// 使用 Consumer 监听状态
class UserProfile extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final user = ref.watch(authProvider).user;
    
    return Column(
      children: [
        CircleAvatar(
          backgroundImage: user?.avatarUrl != null
              ? NetworkImage(user!.avatarUrl!)
              : null,
          child: Text(user?.initials ?? '?'),
        ),
        Text(user?.username ?? ''),
        Text(user?.email ?? ''),
      ],
    );
  }
}

// 使用 ConsumerReader 执行操作
class LoginButton extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return ElevatedButton(
      onPressed: () {
        ref.read(authProvider.notifier).login(
          email: _emailController.text,
          password: _passwordController.text,
        );
      },
      child: Text('登录'),
    );
  }
}
```

## 🧪 测试架构

### 1. 测试分层

```
test/
├── unit/                    # 单元测试
│   ├── domain/
│   │   ├── usecases/
│   │   └── repositories/
│   └── data/
├── widget/                  # Widget测试
│   ├── shared/
│   └── features/
├── integration/             # 集成测试
│   ├── api/
│   └── storage/
└── test_utils/              # 测试工具
    ├── mocks/
    └── fixtures/
```

### 2. 测试示例

```dart
// Use Case 测试
void main() {
  group('LoginUseCase', () {
    late LoginUseCase useCase;
    late MockAuthRepository mockRepository;
    
    setUp(() {
      mockRepository = MockAuthRepository();
      useCase = LoginUseCase(mockRepository);
    });
    
    test('should return AuthResult when login succeeds', () async {
      // Arrange
      final params = LoginParams(email: 'test@example.com', password: 'password');
      final authResult = AuthResult(
        user: User(id: '1', username: 'test', email: 'test@example.com'),
        accessToken: 'token',
        refreshToken: 'refresh',
        expiresAt: DateTime.now().add(Duration(minutes: 15)),
      );
      
      when(mockRepository.login(any))
          .thenAnswer((_) async => Right(authResult));
      
      // Act
      final result = await useCase(params);
      
      // Assert
      expect(result.isRight, true);
      result.fold(
        (error) => fail('Expected success but got error'),
        (result) {
          expect(result.user.username, 'test');
        },
      );
    });
  });
}

// Widget 测试
void main() {
  testWidgets('LoginForm validates input', (tester) async {
    await tester.pumpWidget(
      ProviderScope(
        child: MaterialApp(
          home: Scaffold(
            body: LoginForm(
              emailController: TextEditingController(),
              passwordController: TextEditingController(),
              isPasswordVisible: false,
              onPasswordVisibilityChanged: (visible) {},
              onLogin: (email, password) {},
            ),
          ),
        ),
      ),
    );
    
    // 测试空输入
    await tester.tap(find.byType(ElevatedButton));
    await tester.pump();
    
    expect(find.text('请输入邮箱'), findsOneWidget);
    expect(find.text('请输入密码'), findsOneWidget);
  });
}
```

## 🚀 性能优化策略

### 1. 状态优化

- 使用 `const` 构造函数
- 避免不必要的 widget 重建
- 使用 `select` 监听特定状态

### 2. 图片优化

- 使用 `cached_network_image`
- 实现图片懒加载
- 生成适当尺寸的缩略图

### 3. 列表优化

- 使用 `ListView.builder`
- 实现 item 缓存
- 分页加载大量数据

### 4. 内存管理

- 及时释放资源
- 使用 `AutomaticKeepAliveClientMixin` 谨慎
- 监控内存使用

## 📚 最佳实践总结

1. **保持分层清晰**: 严格遵守分层架构，避免跨层调用
2. **依赖注入**: 使用 GetIt 管理依赖，便于测试和维护
3. **状态管理**: 合理使用 Riverpod，避免过度状态
4. **错误处理**: 统一的错误处理机制，提供友好提示
5. **测试覆盖**: 为每一层编写相应测试
6. **性能优先**: 在开发早期考虑性能问题
7. **代码复用**: 提取可复用的组件和逻辑
8. **文档同步**: 保持代码和文档的同步更新

---

**文档版本**: v1.0  
**最后更新**: 2025-12-12  
**维护者**: 开发团队