import '../../../core/network/api_client.dart';
import '../models/message.dart';

/// 聊天服务
class ChatService {
  static final ChatService instance = ChatService._();
  ChatService._();

  final _api = ApiClient.instance;

  /// 获取消息列表
  Future<List<Message>> getMessages(
    String conversationId, {
    int? limit,
    String? before,
  }) async {
    final params = <String, dynamic>{};
    if (limit != null) params['limit'] = limit.toString();
    if (before != null) params['before'] = before;

    return await _api.get<List<Message>>(
      '/conversations/$conversationId/messages',
      queryParameters: params.isNotEmpty ? params : null,
      fromJson: (json) {
        final list = json['messages'] as List? ?? json as List;
        return list
            .map((e) => Message.fromJson(e as Map<String, dynamic>))
            .toList();
      },
    );
  }

  /// 发送消息（HTTP 备用）
  Future<Message> sendMessage({
    required String conversationId,
    required String content,
    String type = 'text',
    String? replyToId,
  }) async {
    return await _api.post<Message>(
      '/conversations/$conversationId/messages',
      data: {
        'content': content,
        'type': type,
        if (replyToId != null) 'reply_to_id': replyToId,
      },
      fromJson: (json) => Message.fromJson(json as Map<String, dynamic>),
    );
  }

  /// 标记消息为已读
  Future<void> markAsRead(String conversationId) async {
    await _api.postEmpty('/conversations/$conversationId/read');
  }
}
