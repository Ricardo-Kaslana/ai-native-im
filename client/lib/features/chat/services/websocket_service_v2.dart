import 'dart:async';
import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:web_socket_channel/web_socket_channel.dart';
import '../../../core/config/app_config.dart';
import '../../../core/storage/storage.dart';
import '../models/message_models.dart';

/// WebSocket 连接状态
enum WebSocketConnectionState {
  disconnected,
  connecting,
  connected,
  reconnecting,
  authFailed,
  error,
}

/// WebSocket 服务 V2 - 完全对齐后端协议
class WebSocketServiceV2 {
  static final WebSocketServiceV2 instance = WebSocketServiceV2._();
  WebSocketServiceV2._();

  WebSocketChannel? _channel;
  Timer? _heartbeatTimer;
  Timer? _reconnectTimer;

  // 消息控制器
  final _messageController = StreamController<ChatMessage>.broadcast();
  final _connectionStateController = StreamController<WebSocketConnectionState>.broadcast();
  final _typingController = StreamController<TypingIndicator>.broadcast();
  final _presenceController = StreamController<PresenceUpdate>.broadcast();
  final _errorController = StreamController<String>.broadcast();

  // 状态
  WebSocketConnectionState _state = WebSocketConnectionState.disconnected;
  String? _currentToken;
  int _reconnectAttempts = 0;
  static const int _maxReconnectAttempts = 10;
  bool _isConnecting = false;
  Completer<void>? _connectCompleter;
  int _heartbeatTimerId = 0; // 用于取消旧的定时器

  // 流
  Stream<ChatMessage> get messageStream => _messageController.stream;
  Stream<WebSocketConnectionState> get connectionStateStream => _connectionStateController.stream;
  Stream<TypingIndicator> get typingStream => _typingController.stream;
  Stream<PresenceUpdate> get presenceStream => _presenceController.stream;
  Stream<String> get errorStream => _errorController.stream;

  WebSocketConnectionState get state => _state;
  bool get isConnected => _state == WebSocketConnectionState.connected;

  /// 连接 WebSocket
  Future<void> connect() async {
    // 如果已经在连接中，等待完成
    if (_isConnecting) {
      debugPrint('WebSocket: Already connecting, waiting...');
      await _connectCompleter?.future;
      return;
    }

    // 如果已经连接，直接返回
    if (_state == WebSocketConnectionState.connected) {
      debugPrint('WebSocket: Already connected');
      return;
    }

    // 标记正在连接
    _isConnecting = true;
    _connectCompleter = Completer<void>();
    _reconnectTimer?.cancel();

    // 获取最新的 Token（确保不是缓存的过期 Token）
    _currentToken = await SecureStorage.instance.getAccessToken();
    if (_currentToken == null || _currentToken!.isEmpty) {
      _setState(WebSocketConnectionState.authFailed);
      _error('No access token available');
      _isConnecting = false;
      _connectCompleter?.complete();
      return;
    }

    _setState(WebSocketConnectionState.connecting);
    debugPrint('WebSocket: Connecting to ${AppConfig.wsBaseUrl}/ws');

    try {
      final uri = Uri.parse('${AppConfig.wsBaseUrl}/ws?token=$_currentToken');
      _channel = WebSocketChannel.connect(uri);

      // 等待连接建立
      await _channel!.ready.timeout(
        const Duration(seconds: 10),
        onTimeout: () {
          throw TimeoutException('WebSocket connection timeout');
        },
      );

      _setState(WebSocketConnectionState.connected);
      _reconnectAttempts = 0;
      _isConnecting = false;
      debugPrint('WebSocket: Connected successfully');

      // 监听消息流
      final subscription = _channel!.stream.listen(
        _handleMessage,
        onError: _handleError,
        onDone: _handleDone,
        cancelOnError: false,
      );

      // 保存订阅以便在断开时取消
      _streamSubscription = subscription;

      // 启动心跳（每5秒发送一次，服务器超时是10秒）
      _startHeartbeat();

      _connectCompleter?.complete();
    } catch (e) {
      debugPrint('WebSocket: Connection failed - $e');
      _error('Connection failed: $e');
      _setState(WebSocketConnectionState.error);
      _isConnecting = false;
      _connectCompleter?.completeError(e);
      _scheduleReconnect();
    }
  }

  StreamSubscription? _streamSubscription;

  /// 处理接收到的消息
  void _handleMessage(dynamic data) {
    if (data is! String) {
      debugPrint('WebSocket: Received non-text message');
      return;
    }

    debugPrint('WebSocket: Received message - $data');

    try {
      final json = jsonDecode(data) as Map<String, dynamic>;

      // 检查是否是原始消息对象（服务器直接发送的消息格式）
      // 格式：{"id":"...", "conversation_id":"...", "sender_id":"...", "type":"text", ...}
      if (json.containsKey('id') && json.containsKey('sender_id')) {
        // 这是一个原始消息对象，直接解析为 ChatMessage
        try {
          final message = ChatMessage.fromJson(json);
          _messageController.add(message);
          debugPrint('WebSocket: New message from ${message.senderId} (type: ${json['type']})');
        } catch (e) {
          debugPrint('WebSocket: Failed to parse raw message - $e');
        }
        return;
      }

      // 处理 ServerPacket 格式
      final packet = ServerPacket.fromJson(json);

      switch (packet.type) {
        case ServerPacketType.message:
          if (packet.message != null) {
            _messageController.add(packet.message!);
            debugPrint('WebSocket: New message from ${packet.message!.senderId}');
          }
          break;

        case ServerPacketType.typing:
          final typing = TypingIndicator.fromJson(json);
          _typingController.add(typing);
          break;

        case ServerPacketType.presenceUpdate:
        case ServerPacketType.userStatusUpdate:
          final presence = PresenceUpdate.fromJson(json);
          _presenceController.add(presence);
          break;

        case ServerPacketType.syncResponse:
          _handleSyncResponse(json);
          break;

        default:
          debugPrint('WebSocket: Unhandled packet type - ${packet.type}');
      }
    } catch (e) {
      debugPrint('WebSocket: Failed to parse message - $e');
    }
  }

  /// 处理同步响应
  void _handleSyncResponse(Map<String, dynamic> json) {
    final messages = json['messages'] as List?;
    if (messages != null) {
      for (final msg in messages) {
        try {
          final message = ChatMessage.fromJson(msg as Map<String, dynamic>);
          _messageController.add(message);
        } catch (e) {
          debugPrint('WebSocket: Failed to parse sync message - $e');
        }
      }
    }
  }

  /// 处理错误
  void _handleError(Object error) {
    debugPrint('WebSocket: Error - $error');
    _error('WebSocket error: $error');
    _setState(WebSocketConnectionState.error);
    _stopHeartbeat();
    _scheduleReconnect();
  }

  /// 处理连接关闭
  void _handleDone() {
    debugPrint('WebSocket: Connection closed');
    _setState(WebSocketConnectionState.disconnected);
    _stopHeartbeat();
    _scheduleReconnect();
  }

  /// 启动心跳
  void _startHeartbeat() {
    _stopHeartbeat(); // 先停止之前的心跳
    // 每5秒发送一次心跳，服务器超时是10秒，确保不会超时
    _heartbeatTimerId++;
    final currentTimerId = _heartbeatTimerId;
    _heartbeatTimer = Timer.periodic(
      const Duration(seconds: 5),
      (timer) {
        // 只在连接状态且定时器ID匹配时发送
        if (_state == WebSocketConnectionState.connected &&
            _channel != null &&
            _heartbeatTimerId == currentTimerId) {
          _sendHeartbeat();
        } else {
          timer.cancel();
        }
      },
    );
  }

  /// 停止心跳
  void _stopHeartbeat() {
    _heartbeatTimer?.cancel();
    _heartbeatTimer = null;
    _heartbeatTimerId++; // 使旧的定时器失效
  }

  /// 发送心跳
  void _sendHeartbeat() {
    if (_state == WebSocketConnectionState.connected && _channel != null) {
      final heartbeat = jsonEncode({'type': 'heartbeat'});
      try {
        _channel!.sink.add(heartbeat);
        debugPrint('WebSocket: Heartbeat sent');
      } catch (e) {
        debugPrint('WebSocket: Failed to send heartbeat - $e');
        // 发送失败说明连接已断开
        _handleDone();
      }
    }
  }

  /// 安排重连
  void _scheduleReconnect() {
    _reconnectTimer?.cancel();

    if (_reconnectAttempts >= _maxReconnectAttempts) {
      debugPrint('WebSocket: Max reconnect attempts reached');
      _setState(WebSocketConnectionState.error);
      return;
    }

    _reconnectAttempts++;
    final delay = Duration(seconds: _calculateBackoff(_reconnectAttempts));

    debugPrint('WebSocket: Scheduling reconnect #$_reconnectAttempts in ${delay.inSeconds}s');

    _setState(WebSocketConnectionState.reconnecting);

    _reconnectTimer = Timer(delay, () {
      _isConnecting = false;
      connect();
    });
  }

  /// 计算退避延迟
  int _calculateBackoff(int attempt) {
    final backoff = (1 << attempt).clamp(1, 60);
    return backoff;
  }

  /// 发送消息（简化版：不等待服务器 ACK，因为服务器当前不发送 ACK）
  Future<void> sendMessage(ChatMessage message) async {
    if (_state != WebSocketConnectionState.connected) {
      throw Exception('WebSocket not connected. State: $_state');
    }

    try {
      final json = jsonEncode(message.toWebSocket());
      _channel?.sink.add(json);
      debugPrint('WebSocket: Message sent - $json');
    } catch (e) {
      debugPrint('WebSocket: Failed to send message - $e');
      rethrow;
    }
  }

  /// 发送消息（不等待确认）
  void sendMessageNoAck(ChatMessage message) {
    if (_state != WebSocketConnectionState.connected) {
      debugPrint('WebSocket: Not connected, cannot send message');
      return;
    }

    try {
      final json = jsonEncode(message.toWebSocket());
      _channel?.sink.add(json);
      debugPrint('WebSocket: Message sent (no ack) - $json');
    } catch (e) {
      debugPrint('WebSocket: Failed to send message - $e');
    }
  }

  /// 发送输入状态
  void sendTypingIndicator({
    required String conversationId,
    required bool isTyping,
    bool isGroup = false,
  }) {
    if (_state != WebSocketConnectionState.connected) return;

    final typing = TypingIndicator(
      userId: '',
      groupId: isGroup ? conversationId : null,
      isTyping: isTyping,
    );

    try {
      _channel?.sink.add(jsonEncode(typing.toJson()));
    } catch (e) {
      debugPrint('WebSocket: Failed to send typing indicator - $e');
    }
  }

  /// 断开连接
  void disconnect() {
    debugPrint('WebSocket: Disconnecting');
    _isConnecting = false;
    _stopHeartbeat();
    _reconnectTimer?.cancel();
    _connectCompleter?.complete();
    _streamSubscription?.cancel();
    _streamSubscription = null;
    _channel?.sink.close();
    _channel = null;
    _setState(WebSocketConnectionState.disconnected);
    _reconnectAttempts = 0;
  }

  /// 设置状态
  void _setState(WebSocketConnectionState newState) {
    if (_state != newState) {
      _state = newState;
      if (!_connectionStateController.isClosed) {
        _connectionStateController.add(newState);
      }
      debugPrint('WebSocket: State changed to $newState');
    }
  }

  /// 发送错误
  void _error(String message) {
    if (!_errorController.isClosed) {
      _errorController.add(message);
    }
  }

  /// 销毁
  void dispose() {
    disconnect();
    _messageController.close();
    _connectionStateController.close();
    _typingController.close();
    _presenceController.close();
    _errorController.close();
  }
}
