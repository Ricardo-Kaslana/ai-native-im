import 'dart:async';
import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:web_socket_channel/web_socket_channel.dart';
import '../../../core/config/app_config.dart';
import '../../../core/storage/storage.dart';
import '../models/message.dart';

/// WebSocket 服务
class WebSocketService {
  static final WebSocketService instance = WebSocketService._();
  WebSocketService._();

  WebSocketChannel? _channel;
  Timer? _heartbeatTimer;
  Timer? _reconnectTimer;

  final _messageController = StreamController<Message>.broadcast();
  final _connectionController = StreamController<bool>.broadcast();

  Stream<Message> get messageStream => _messageController.stream;
  Stream<bool> get connectionStream => _connectionController.stream;

  bool _isConnected = false;
  bool get isConnected => _isConnected;

  /// 连接 WebSocket
  Future<void> connect() async {
    if (_isConnected) {
      debugPrint('WebSocket: Already connected');
      return;
    }

    final token = await SecureStorage.instance.getAccessToken();
    if (token == null || token.isEmpty) {
      debugPrint('WebSocket: No token available');
      return;
    }

    debugPrint('WebSocket: Connecting to ${AppConfig.wsBaseUrl}/ws');

    try {
      final uri = Uri.parse('${AppConfig.wsBaseUrl}/ws?token=$token');
      _channel = WebSocketChannel.connect(uri);

      // 等待连接建立
      await _channel!.ready.timeout(
        const Duration(seconds: 5),
        onTimeout: () {
          throw TimeoutException('WebSocket connection timeout');
        },
      );

      _isConnected = true;
      _connectionController.add(true);
      debugPrint('WebSocket: Connected successfully');

      // 监听消息流
      _channel!.stream.listen(
        _handleMessage,
        onError: _handleError,
        onDone: _handleDone,
        cancelOnError: false,
      );

      _startHeartbeat();
    } catch (e) {
      debugPrint('WebSocket: Connection failed - $e');
      _isConnected = false;
      _connectionController.add(false);
      _scheduleReconnect();
    }
  }

  void _handleMessage(dynamic data) {
    debugPrint('WebSocket: Received message - $data');
    try {
      final json = jsonDecode(data as String) as Map<String, dynamic>;
      final type = json['type'] as String?;

      switch (type) {
        case 'new_message':
        case 'message':
          final message = Message.fromWebSocket(json);
          debugPrint('WebSocket: New message from ${message.senderId}');
          _messageController.add(message);
          break;
        case 'heartbeat_ack':
        case 'pong':
          debugPrint('WebSocket: Heartbeat ack');
          break;
        default:
          debugPrint('WebSocket: Unknown message type - $type');
          break;
      }
    } catch (e) {
      debugPrint('WebSocket: Failed to parse message - $e');
    }
  }

  void _handleError(Object error) {
    debugPrint('WebSocket: Error - $error');
    _isConnected = false;
    _connectionController.add(false);
    _scheduleReconnect();
  }

  void _handleDone() {
    debugPrint('WebSocket: Connection closed');
    _isConnected = false;
    _connectionController.add(false);
    _stopHeartbeat();
    _scheduleReconnect();
  }

  void _startHeartbeat() {
    _heartbeatTimer?.cancel();
    _heartbeatTimer = Timer.periodic(
      Duration(seconds: AppConfig.wsHeartbeatInterval),
      (_) => _sendPing(),
    );
  }

  void _stopHeartbeat() {
    _heartbeatTimer?.cancel();
    _heartbeatTimer = null;
  }

  void _sendPing() {
    if (_isConnected && _channel != null) {
      final ping = jsonEncode({'type': 'heartbeat'});
      _channel!.sink.add(ping);
      debugPrint('WebSocket: Sent heartbeat');
    }
  }

  void _scheduleReconnect() {
    _reconnectTimer?.cancel();
    debugPrint('WebSocket: Scheduling reconnect in ${AppConfig.wsReconnectDelay}s');
    _reconnectTimer = Timer(
      Duration(seconds: AppConfig.wsReconnectDelay),
      () => connect(),
    );
  }

  /// 发送消息
  Future<void> sendMessage({
    required String conversationId,
    required String content,
    String type = 'text',
    String? replyToId,
    bool isGroup = false,
  }) async {
    if (!_isConnected) {
      debugPrint('WebSocket: Cannot send message - not connected');
      throw Exception('WebSocket 未连接');
    }

    // 后端期望消息字段在根级别，不嵌套在 data 中
    // 格式: {type: "message", receiver_id/group_id, content, msg_type}
    final message = {
      'type': 'message',
      if (isGroup) 'group_id': conversationId else 'receiver_id': conversationId,
      'content': content,
      'msg_type': type == 'text' ? 0 : (type == 'image' ? 1 : (type == 'video' ? 2 : 0)),
      if (replyToId != null) 'reply_to': replyToId,
    };

    final json = jsonEncode(message);
    debugPrint('WebSocket: Sending message - $json');
    _channel?.sink.add(json);
  }

  /// 发送输入状态
  void sendTyping(String conversationId, bool isTyping) {
    if (!_isConnected) return;

    final message = {
      'type': 'typing',
      'data': {
        'conversation_id': conversationId,
        'is_typing': isTyping,
      },
    };

    _channel?.sink.add(jsonEncode(message));
  }

  /// 断开连接
  void disconnect() {
    debugPrint('WebSocket: Disconnecting');
    _stopHeartbeat();
    _reconnectTimer?.cancel();
    _channel?.sink.close();
    _channel = null;
    _isConnected = false;
    _connectionController.add(false);
  }

  /// 销毁
  void dispose() {
    disconnect();
    _messageController.close();
    _connectionController.close();
  }
}
