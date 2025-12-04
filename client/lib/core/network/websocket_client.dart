import 'dart:async';
import 'dart:convert';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

/// WebSocket connection state
enum WebSocketState { disconnected, connecting, connected, reconnecting }

/// WebSocket client provider
final webSocketClientProvider = Provider<WebSocketClient>((ref) {
  final client = WebSocketClient();
  ref.onDispose(() => client.disconnect());
  return client;
});

/// WebSocket connection state provider
final webSocketStateProvider = StateProvider<WebSocketState>((ref) {
  return WebSocketState.disconnected;
});

/// WebSocket client for real-time messaging
class WebSocketClient {
  static const String _baseUrl = 'ws://localhost:8080/ws';
  static const Duration _reconnectDelay = Duration(seconds: 3);
  static const int _maxReconnectAttempts = 5;

  WebSocketChannel? _channel;
  StreamSubscription? _subscription;
  String? _accessToken;
  int _reconnectAttempts = 0;
  bool _shouldReconnect = true;

  final _messageController = StreamController<Map<String, dynamic>>.broadcast();
  final _stateController = StreamController<WebSocketState>.broadcast();

  Stream<Map<String, dynamic>> get messages => _messageController.stream;
  Stream<WebSocketState> get state => _stateController.stream;

  void setAccessToken(String? token) {
    _accessToken = token;
  }

  Future<void> connect() async {
    if (_channel != null) return;

    _stateController.add(WebSocketState.connecting);

    try {
      final uri = Uri.parse('$_baseUrl?token=$_accessToken');
      _channel = WebSocketChannel.connect(uri);

      _subscription = _channel!.stream.listen(
        _onMessage,
        onError: _onError,
        onDone: _onDone,
      );

      _reconnectAttempts = 0;
      _stateController.add(WebSocketState.connected);
    } catch (e) {
      _stateController.add(WebSocketState.disconnected);
      _scheduleReconnect();
    }
  }

  void disconnect() {
    _shouldReconnect = false;
    _subscription?.cancel();
    _channel?.sink.close();
    _channel = null;
    _stateController.add(WebSocketState.disconnected);
  }

  void send(Map<String, dynamic> message) {
    if (_channel != null) {
      _channel!.sink.add(jsonEncode(message));
    }
  }

  void sendTypingIndicator(String conversationId, bool isTyping) {
    send({
      'type': 'typing',
      'conversation_id': conversationId,
      'is_typing': isTyping,
    });
  }

  void sendMessage(String conversationId, String content, {String type = 'text'}) {
    send({
      'type': 'message',
      'conversation_id': conversationId,
      'content': content,
      'message_type': type,
    });
  }

  void _onMessage(dynamic data) {
    try {
      final message = jsonDecode(data as String) as Map<String, dynamic>;
      _messageController.add(message);
    } catch (e) {
      // Invalid message format
    }
  }

  void _onError(dynamic error) {
    _stateController.add(WebSocketState.disconnected);
    _scheduleReconnect();
  }

  void _onDone() {
    _channel = null;
    _stateController.add(WebSocketState.disconnected);
    _scheduleReconnect();
  }

  void _scheduleReconnect() {
    if (!_shouldReconnect) return;
    if (_reconnectAttempts >= _maxReconnectAttempts) return;

    _reconnectAttempts++;
    _stateController.add(WebSocketState.reconnecting);

    Future.delayed(_reconnectDelay * _reconnectAttempts, () {
      if (_shouldReconnect && _channel == null) {
        connect();
      }
    });
  }

  void dispose() {
    disconnect();
    _messageController.close();
    _stateController.close();
  }
}
