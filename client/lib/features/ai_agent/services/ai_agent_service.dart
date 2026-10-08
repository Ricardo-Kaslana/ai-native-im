/// AI Agent 服务
///
/// 对齐服务器 API endpoints
library;

import '../../../core/network/api_client.dart';
import '../models/ai_agent.dart';
import '../models/persona_config.dart';

/// AI Agent 服务
class AIAgentService {
  static final AIAgentService instance = AIAgentService._();
  AIAgentService._();

  final _api = ApiClient.instance;

  // ==================== Agent CRUD ====================

  /// 为群组创建 AI Agent
  /// POST /groups/:group_id/agent
  Future<AIAgent> createAgent(
    String groupId,
    CreateAIAgent request,
  ) async {
    return await _api.post<AIAgent>(
      '/groups/$groupId/agent',
      data: request.toJson(),
      fromJson: (json) => AIAgent.fromJson(json as Map<String, dynamic>),
    );
  }

  /// 获取群组的 AI Agent
  /// GET /groups/:group_id/agent
  Future<AIAgent?> getAgentForGroup(String groupId) async {
    return await _api.get<AIAgent?>(
      '/groups/$groupId/agent',
      fromJson: (json) {
        if (json == null) return null;
        // ApiClient 已经提取了 data 字段，json 直接就是 agent 对象或 null
        return AIAgent.fromJson(json as Map<String, dynamic>);
      },
    );
  }

  /// 获取 AI Agent 详情
  /// GET /agents/:agent_id
  Future<AIAgent> getAgent(String agentId) async {
    return await _api.get<AIAgent>(
      '/agents/$agentId',
      fromJson: (json) => AIAgent.fromJson(json as Map<String, dynamic>),
    );
  }

  /// 删除 AI Agent
  /// DELETE /agents/:agent_id
  Future<void> deleteAgent(String agentId) async {
    await _api.deleteEmpty('/agents/$agentId');
  }

  /// 更新 AI Agent 人格
  /// PUT /agents/:agent_id/personality
  Future<AIAgent> updatePersonality(
    String agentId,
    UpdatePersonality request,
  ) async {
    return await _api.put<AIAgent>(
      '/agents/$agentId/personality',
      data: request.toJson(),
      fromJson: (json) => AIAgent.fromJson(json as Map<String, dynamic>),
    );
  }

  /// 获取 AI Agent 统计信息
  /// GET /agents/:agent_id/stats
  Future<AgentStatistics> getStatistics(String agentId) async {
    return await _api.get<AgentStatistics>(
      '/agents/$agentId/stats',
      fromJson: (json) => AgentStatistics.fromJson(json as Map<String, dynamic>),
    );
  }

  /// 设置用户亲和度
  /// POST /agents/:agent_id/affinity
  Future<void> setAffinity(
    String agentId,
    String userId,
    int affinityScore,
  ) async {
    await _api.postEmpty(
      '/agents/$agentId/affinity',
      data: {
        'user_id': userId,
        'affinity_score': affinityScore,
      },
    );
  }

  // ==================== Summary ====================

  /// 生成群组消息总结
  /// POST /groups/:group_id/summary
  Future<Summary> generateSummary(
    String groupId, {
    int messageCount = 100,
    SummaryStyle style = SummaryStyle.brief,
    bool useCache = true,
  }) async {
    return await _api.post<Summary>(
      '/groups/$groupId/summary',
      data: {
        'message_count': messageCount,
        'style': style.value,
        'use_cache': useCache,
      },
      fromJson: (json) => Summary.fromJson(json as Map<String, dynamic>),
    );
  }

  // ==================== Feedback ====================

  /// 记录反馈
  /// POST /messages/:message_id/feedback
  Future<String> recordFeedback(
    String messageId, {
    required String feedbackType,
    String? correction,
  }) async {
    final response = await _api.post<Map<String, dynamic>>(
      '/messages/$messageId/feedback',
      data: {
        'feedback_type': feedbackType,
        if (correction != null) 'correction': correction,
      },
      fromJson: (json) => json as Map<String, dynamic>,
    );
    return response['message'] as String? ?? '反馈已记录';
  }

  /// 获取群组反馈统计
  /// GET /groups/:group_id/feedback/stats
  Future<FeedbackStats> getFeedbackStats(String groupId) async {
    return await _api.get<FeedbackStats>(
      '/groups/$groupId/feedback/stats',
      fromJson: (json) => FeedbackStats.fromJson(json as Map<String, dynamic>),
    );
  }

  // ==================== Persona Presets ====================

  /// 获取所有人设预设
  /// GET /agents/presets
  Future<List<PersonaPresetInfo>> getPersonaPresets() async {
    return await _api.get<List<PersonaPresetInfo>>(
      '/agents/presets',
      fromJson: (json) {
        final list = json as List;
        return list
            .map((e) => PersonaPresetInfo.fromJson(e as Map<String, dynamic>))
            .toList();
      },
    );
  }

  /// 测试人设
  /// POST /agents/:agent_id/test
  Future<TestPersonaResponse> testPersona(
    String agentId,
    TestPersonaRequest request,
  ) async {
    return await _api.post<TestPersonaResponse>(
      '/agents/$agentId/test',
      data: request.toJson(),
      fromJson: (json) => TestPersonaResponse.fromJson(json as Map<String, dynamic>),
    );
  }

  // ==================== User Profiles ====================

  /// 获取用户在 AI Agent 的档案
  /// GET /agents/:agent_id/profiles/:user_id
  Future<UserProfile> getUserProfile(String agentId, String userId) async {
    return await _api.get<UserProfile>(
      '/agents/$agentId/profiles/$userId',
      fromJson: (json) => UserProfile.fromJson(json as Map<String, dynamic>),
    );
  }

  /// 更新用户档案
  /// PUT /agents/:agent_id/profiles/:user_id
  Future<UserProfile> updateUserProfile(
    String agentId,
    String userId, {
    String? nickname,
    int? affinityScore,
    List<String>? preferredTopics,
    String? notes,
  }) async {
    final data = <String, dynamic>{};
    if (nickname != null) data['nickname'] = nickname;
    if (affinityScore != null) data['affinity_score'] = affinityScore;
    if (preferredTopics != null) data['preferred_topics'] = preferredTopics;
    if (notes != null) data['notes'] = notes;

    return await _api.put<UserProfile>(
      '/agents/$agentId/profiles/$userId',
      data: data,
      fromJson: (json) => UserProfile.fromJson(json as Map<String, dynamic>),
    );
  }

  /// 列出 AI Agent 的所有用户档案
  /// GET /agents/:agent_id/profiles
  Future<List<UserProfile>> listUserProfiles(String agentId) async {
    return await _api.get<List<UserProfile>>(
      '/agents/$agentId/profiles',
      fromJson: (json) {
        final list = json as List;
        return list
            .map((e) => UserProfile.fromJson(e as Map<String, dynamic>))
            .toList();
      },
    );
  }
}

/// 用户档案
class UserProfile {
  final String agentId;
  final String userId;
  final String? nickname;
  final int affinityScore;
  final List<String> preferredTopics;
  final String? notes;
  final DateTime createdAt;
  final DateTime? updatedAt;

  UserProfile({
    required this.agentId,
    required this.userId,
    this.nickname,
    required this.affinityScore,
    this.preferredTopics = const [],
    this.notes,
    required this.createdAt,
    this.updatedAt,
  });

  factory UserProfile.fromJson(Map<String, dynamic> json) {
    return UserProfile(
      agentId: json['agent_id'] as String? ?? '',
      userId: json['user_id'] as String? ?? '',
      nickname: json['nickname'] as String?,
      affinityScore: json['affinity_score'] as int? ?? 0,
      preferredTopics: (json['preferred_topics'] as List?)?.cast<String>() ?? [],
      notes: json['notes'] as String?,
      createdAt: DateTime.parse(json['created_at'] as String? ?? DateTime.now().toIso8601String()),
      updatedAt: json['updated_at'] != null
          ? DateTime.parse(json['updated_at'] as String)
          : null,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'agent_id': agentId,
      'user_id': userId,
      if (nickname != null) 'nickname': nickname,
      'affinity_score': affinityScore,
      'preferred_topics': preferredTopics,
      if (notes != null) 'notes': notes,
      'created_at': createdAt.toIso8601String(),
      if (updatedAt != null) 'updated_at': updatedAt!.toIso8601String(),
    };
  }

  UserProfile copyWith({
    String? agentId,
    String? userId,
    String? nickname,
    int? affinityScore,
    List<String>? preferredTopics,
    String? notes,
    DateTime? createdAt,
    DateTime? updatedAt,
  }) {
    return UserProfile(
      agentId: agentId ?? this.agentId,
      userId: userId ?? this.userId,
      nickname: nickname ?? this.nickname,
      affinityScore: affinityScore ?? this.affinityScore,
      preferredTopics: preferredTopics ?? this.preferredTopics,
      notes: notes ?? this.notes,
      createdAt: createdAt ?? this.createdAt,
      updatedAt: updatedAt ?? this.updatedAt,
    );
  }
}
