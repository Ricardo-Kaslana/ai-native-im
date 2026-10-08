import '../../../core/network/api_client.dart';
import '../models/conversation.dart';

/// 对话服务
class ConversationService {
  static final ConversationService instance = ConversationService._();
  ConversationService._();

  final _api = ApiClient.instance;

  /// 获取对话列表
  Future<List<Conversation>> getConversations() async {
    return await _api.get<List<Conversation>>(
      '/conversations',
      fromJson: (json) {
        // 后端返回 { success: true, data: [...] }，ApiClient 提取后是数组
        final List list;
        if (json is List) {
          list = json;
        } else if (json is Map) {
          list = json['conversations'] as List? ?? json['data'] as List? ?? [];
        } else {
          list = [];
        }
        return list
            .map((e) => Conversation.fromJson(e as Map<String, dynamic>))
            .toList();
      },
    );
  }

  /// 获取单个对话
  Future<Conversation> getConversation(String id) async {
    return await _api.get<Conversation>(
      '/conversations/$id',
      fromJson: (json) => Conversation.fromJson(json as Map<String, dynamic>),
    );
  }

  /// 创建私聊对话
  Future<Conversation> createPrivateConversation(String userId) async {
    return await _api.post<Conversation>(
      '/conversations',
      data: {
        'conversation_type': 'private',  // 使用小写，匹配数据库约束
        'participant_ids': [userId],
      },
      fromJson: (json) {
        final convData = json['conversation'] ?? json;
        return Conversation.fromJson(Map<String, dynamic>.from(convData));
      },
    );
  }

  /// 创建群聊对话
  Future<Conversation> createGroupConversation({
    required String name,
    required List<String> memberIds,
  }) async {
    return await _api.post<Conversation>(
      '/conversations',
      data: {
        'conversation_type': 'group',  // 使用小写，匹配数据库约束
        'name': name,
        'participant_ids': memberIds,
      },
      fromJson: (json) {
        final convData = json['conversation'] ?? json;
        return Conversation.fromJson(Map<String, dynamic>.from(convData));
      },
    );
  }

  /// 标记对话为已读
  Future<void> markAsRead(String conversationId) async {
    await _api.postEmpty('/conversations/$conversationId/read');
  }

  /// 删除对话
  Future<void> deleteConversation(String conversationId) async {
    await _api.deleteEmpty('/conversations/$conversationId');
  }
}
