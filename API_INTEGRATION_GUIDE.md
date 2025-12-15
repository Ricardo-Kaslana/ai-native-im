# API 集成指南

## 📋 概述

本指南详细说明了如何将 Flutter 客户端与 AI Native IM 后端 API 进行集成。包括认证、WebSocket 消息传递、错误处理等核心功能的实现细节。

## 🚀 快速开始

### 1. 环境准备

确保后端服务正在运行：
```bash
# 后端服务地址
http://localhost:8080

# WebSocket 地址
ws://localhost:8080/ws
```

### 2. 依赖配置

确保 `pubspec.yaml` 包含必要的依赖：
```yaml
dependencies:
  # 网络请求
  dio: ^5.4.0
  retrofit: ^4.0.3
  
  # 状态管理
  flutter_riverpod: ^2.4.9
  
  # 安全存储
  flutter_secure_storage: ^9.0.0
  
  # Protocol Buffers
  protobuf: ^3.1.0
  
  # WebSocket
  web_socket_channel: ^2.4.0
```

### 3. 初始化依赖注入

```dart
import 'src/core/di/injection_container.dart' as di;

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await di.configureDependencies();
  runApp(MyApp());
}
```

## 🔐 认证集成

### 1. 用户注册

```dart
// 使用 RegisterUseCase
final registerParams = RegisterParams(
  username: 'testuser',
  email: 'test@example.com',
  password: 'Test123456!', // 必须包含大小写字母、数字和特殊字符
);

final result = await registerUseCase(registerParams);

result.fold(
  (error) => print('注册失败: ${error.message}'),
  (authResult) => print('注册成功: ${authResult.user.username}'),
);
```

### 2. 用户登录

```dart
// 使用 LoginUseCase
final loginParams = LoginParams(
  email: 'test@example.com',
  password: 'Test123456!',
);

final result = await loginUseCase(loginParams);

result.fold(
  (error) => print('登录失败: ${error.message}'),
  (authResult) => print('登录成功: ${authResult.user.username}'),
);
```

### 3. 自动登录

应用启动时自动检查登录状态：

```dart
class MyApp extends ConsumerStatefulWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // 初始化认证状态
    useEffect(() {
      ref.read(authProvider.notifier).initialize();
      return null;
    }, []);

    return MaterialApp(
      // 根据认证状态显示不同页面
      home: Consumer(
        builder: (context, ref, child) {
          final authState = ref.watch(authProvider);
          
          if (authState.isLoading) {
            return LoadingScreen();
          }
          
          if (authState.isAuthenticated) {
            return HomeScreen();
          } else {
            return LoginScreen();
          }
        },
      ),
    );
  }
}
```

### 4. JWT 令牌管理

令牌自动存储和刷新：

```dart
// AuthRepository 自动处理令牌刷新
class AuthInterceptor extends Interceptor {
  @override
  void onError(DioException err, ErrorInterceptorHandler handler) {
    if (err.response?.statusCode == 401) {
      // 自动刷新令牌并重试请求
      _refreshToken(err, handler);
    } else {
      handler.next(err);
    }
  }
}
```

## 📡 WebSocket 集成

### 1. 建立连接

```dart
class WebSocketService {
  WebSocketChannel? _channel;
  
  Future<void> connect(String accessToken) async {
    final uri = Uri.parse('ws://localhost:8080/ws?token=$accessToken');
    _channel = WebSocketChannel.connect(uri);
    
    _channel!.stream.listen(
      _handleMessage,
      onError: _handleError,
      onDone: _handleDone,
    );
  }
  
  void _handleMessage(dynamic message) {
    // 解析 Protocol Buffers 消息
    final packet = ServerPacket.fromBuffer(message);
    
    switch (packet.whichPayload()) {
      case ServerPacketPayload.message:
        _handleChatMessage(packet.message);
        break;
      case ServerPacketPayload.syncResponse:
        _handleSyncResponse(packet.syncResponse);
        break;
      // 处理其他消息类型
    }
  }
}
```

### 2. 发送消息

```dart
void sendMessage({
  required String content,
  required String conversationId,
  MsgType type = MsgType.TEXT,
}) {
  final message = ChatMessage(
    id: UUID().v4(),
    senderId: currentUserId,
    content: content,
    type: type,
    timestamp: Timestamp.now(),
  );
  
  final packet = ClientPacket(
    message: message,
  );
  
  _channel?.sink.add(packet.writeToBuffer());
}
```

### 3. 心跳机制

```dart
Timer? _heartbeatTimer;

void _startHeartbeat() {
  _heartbeatTimer = Timer.periodic(Duration(seconds: 30), (_) {
    final heartbeat = Heartbeat(
      timestamp: Timestamp.now(),
    );
    
    final packet = ClientPacket(
      heartbeat: heartbeat,
    );
    
    _channel?.sink.add(packet.writeToBuffer());
  });
}
```

## 🛠️ API 调用示例

### 1. 获取用户信息

```dart
class UserRepository {
  final DioClient _dioClient;
  
  Future<User> getCurrentUser() async {
    try {
      final response = await _dioClient.get<Map<String, dynamic>>(
        '/users/me',
      );
      
      return UserModel.fromJson(response.data!['data']['user']);
    } on DioException catch (e) {
      throw AppException.fromDioError(e);
    }
  }
}
```

### 2. 搜索用户

```dart
Future<List<User>> searchUsers(String query) async {
  try {
    final response = await _dioClient.get<Map<String, dynamic>>(
      '/users/search',
      queryParameters: {
        'query': query,
        'limit': 20,
        'offset': 0,
      },
    );
    
    final usersList = response.data!['data']['users'] as List;
    return usersList.map((json) => UserModel.fromJson(json)).toList();
  } on DioException catch (e) {
    throw AppException.fromDioError(e);
  }
}
```

### 3. 创建群组

```dart
Future<Group> createGroup({
  required String name,
  String? description,
}) async {
  try {
    final response = await _dioClient.post<Map<String, dynamic>>(
      '/groups',
      data: {
        'name': name,
        if (description != null) 'description': description,
      },
    );
    
    return GroupModel.fromJson(response.data!['data']['group']);
  } on DioException catch (e) {
    throw AppException.fromDioError(e);
  }
}
```

## ⚠️ 错误处理

### 1. 统一错误类型

```dart
abstract class AppException implements Exception {
  final String code;
  final String message;
  final dynamic details;
  
  const AppException(this.code, this.message, [this.details]);
  
  factory AppException.fromDioError(DioException error) {
    switch (error.type) {
      case DioExceptionType.connectionTimeout:
        return NetworkException.timeout;
      case DioExceptionType.connectionError:
        return NetworkException.noConnection;
      case DioExceptionType.badResponse:
        return _handleHttpError(error.response?.statusCode ?? 0, error.response?.data);
      default:
        return SystemException.unknownError;
    }
  }
  
  static AppException _handleHttpError(int statusCode, dynamic responseData) {
    final message = responseData?['error']?['message'] ?? 'HTTP错误';
    
    switch (statusCode) {
      case 400:
        return BusinessException.invalidRequest;
      case 401:
        return BusinessException.unauthorized;
      case 403:
        return BusinessException.forbidden;
      case 404:
        return BusinessException.notFound;
      case 500:
        return NetworkException.serverError;
      default:
        return NetworkException.serverError;
    }
  }
}
```

### 2. 错误处理策略

```dart
class ErrorHandler {
  static void handleError(Object error, StackTrace? stackTrace) {
    // 记录错误
    Logger.error(error.toString(), error: error, stackTrace: stackTrace);
    
    // 显示用户友好的错误提示
    final userMessage = _getUserFriendlyMessage(error);
    _showErrorToUser(userMessage);
  }
  
  static String _getUserFriendlyMessage(Object error) {
    if (error is NetworkException) {
      return '网络连接异常，请检查网络设置';
    } else if (error is BusinessException) {
      return error.message;
    } else {
      return '发生未知错误，请稍后重试';
    }
  }
}
```

## 📱 UI 集成示例

### 1. 登录表单

```dart
class LoginForm extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final authState = ref.watch(authProvider);
    final authNotifier = ref.read(authProvider.notifier);
    
    return Column(
      children: [
        TextFormField(
          decoration: InputDecoration(labelText: '邮箱'),
          validator: (value) {
            if (value == null || value.isEmpty) return '请输入邮箱';
            if (!RegExp(r'^[^@]+@[^@]+\.[^@]+').hasMatch(value)) {
              return '请输入有效的邮箱地址';
            }
            return null;
          ),
        ),
        
        TextFormField(
          decoration: InputDecoration(labelText: '密码'),
          obscureText: true,
          validator: (value) {
            if (value == null || value.isEmpty) return '请输入密码';
            if (value.length < 6) return '密码至少需要6个字符';
            return null;
          },
        ),
        
        const SizedBox(height: 16),
        
        LoadingButton(
          isLoading: authState.isLoading,
          onPressed: () {
            if (_formKey.currentState!.validate()) {
              authNotifier.login(
                email: _emailController.text,
                password: _passwordController.text,
              );
            }
          },
          child: Text('登录'),
        ),
        
        if (authState.error != null)
          Padding(
            padding: const EdgeInsets.only(top: 16),
            child: Text(
              authState.error!.message,
              style: TextStyle(color: Theme.of(context).colorScheme.error),
            ),
          ),
      ],
    );
  }
}
```

### 2. 消息列表

```dart
class MessageList extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final messages = ref.watch(messageProvider);
    
    return ListView.builder(
      reverse: true,
      itemCount: messages.length,
      itemBuilder: (context, index) {
        final message = messages[index];
        return MessageBubble(
          message: message,
          isMe: message.senderId == currentUserId,
        );
      },
    );
  }
}
```

## 🧪 测试指南

### 1. API 测试

使用提供的测试脚本验证 API 连接：

```bash
# 运行认证 API 测试
dart run test_auth_api.dart

# 运行 JWT 令牌刷新测试
dart run test_jwt_refresh.dart

# 运行 WebSocket 连接测试
dart run test_websocket.dart
```

### 2. 单元测试

```dart
void main() {
  group('AuthRepository', () {
    late AuthRepository repository;
    late MockAuthRemoteDataSource mockRemoteDataSource;
    late MockAuthLocalDataSource mockLocalDataSource;
    
    setUp(() {
      mockRemoteDataSource = MockAuthRemoteDataSource();
      mockLocalDataSource = MockAuthLocalDataSource();
      repository = AuthRepositoryImpl(
        mockRemoteDataSource,
        mockLocalDataSource,
        DioClient(),
      );
    });
    
    test('should login successfully', () async {
      // Arrange
      final request = LoginRequestModel(
        email: 'test@example.com',
        password: 'password123',
      );
      final response = AuthResponseModel(
        accessToken: 'token',
        refreshToken: 'refresh',
        user: UserModel(
          id: '1',
          username: 'test',
          email: 'test@example.com',
          status: UserStatus.online,
          createdAt: DateTime.now(),
        ),
        expiresIn: 900,
      );
      
      when(mockRemoteDataSource.login(any))
          .thenAnswer((_) async => response);
      
      // Act
      final result = await repository.login(
        email: 'test@example.com',
        password: 'password123',
      );
      
      // Assert
      expect(result.isRight, true);
      result.fold(
        (error) => fail('Expected success but got error'),
        (authResult) {
          expect(authResult.user.username, 'test');
          expect(authResult.accessToken, 'token');
        },
      );
    });
  });
}
```

## 📚 最佳实践

### 1. 状态管理

- 使用 Riverpod 进行状态管理
- 保持状态不可变
- 使用 `ConsumerWidget` 或 `Consumer` 监听状态变化

### 2. 网络请求

- 使用 Dio 拦截器处理认证和错误
- 实现请求重试机制
- 添加超时设置

### 3. 错误处理

- 统一的错误类型和处理
- 用户友好的错误提示
- 详细的错误日志

### 4. 性能优化

- 使用 `const` 构造函数
- 避免不必要的重建
- 实现图片懒加载

## 🔧 故障排除

### 常见问题

1. **认证失败**
   - 检查令牌是否过期
   - 验证请求头格式
   - 确认后端服务运行正常

2. **WebSocket 连接失败**
   - 检查令牌是否有效
   - 确认 WebSocket 地址正确
   - 验证网络连接

3. **消息发送失败**
   - 检查 Protocol Buffers 序列化
   - 验证消息格式
   - 确认连接状态

### 调试技巧

1. 使用 `flutter logs` 查看日志
2. 在 Dio 拦截器中添加请求/响应日志
3. 使用 Chrome DevTools 检查网络请求

---

**文档版本**: v1.0  
**最后更新**: 2025-12-12  
**维护者**: 开发团队