import 'package:dio/dio.dart';
import '../../data/models/ai_agent.dart';

/// AI Agent API client
class AIAgentApi {
  final Dio _dio;

  AIAgentApi(this._dio);

  /// Create AI Agent for a group
  Future<AIAgent> createAgent(
    String groupId,
    CreateAgentRequest request,
  ) async {
    final response = await _dio.post(
      '/groups/$groupId/agent',
      data: request.toJson(),
    );
    return AIAgent.fromJson(response.data);
  }

  /// Get AI Agent for a group
  Future<AIAgent?> getAgentForGroup(String groupId) async {
    try {
      final response = await _dio.get('/groups/$groupId/agent');
      return AIAgent.fromJson(response.data);
    } on DioException catch (e) {
      if (e.response?.statusCode == 404) {
        return null;
      }
      rethrow;
    }
  }

  /// Get AI Agent by ID
  Future<AIAgent> getAgent(String agentId) async {
    final response = await _dio.get('/agents/$agentId');
    return AIAgent.fromJson(response.data);
  }

  /// Delete AI Agent
  Future<void> deleteAgent(String agentId) async {
    await _dio.delete('/agents/$agentId');
  }

  /// Update AI Agent personality
  Future<AIAgent> updatePersonality(
    String agentId,
    AIAgentPersonality personality,
  ) async {
    final response = await _dio.put(
      '/agents/$agentId/personality',
      data: personality.toJson(),
    );
    return AIAgent.fromJson(response.data);
  }

  /// Get AI Agent statistics
  Future<AIAgentStats> getStatistics(String agentId) async {
    final response = await _dio.get('/agents/$agentId/stats');
    return AIAgentStats.fromJson(response.data);
  }

  /// Set user affinity with AI Agent
  Future<void> setAffinity(
    String agentId,
    String userId,
    int affinityLevel,
  ) async {
    await _dio.post(
      '/agents/$agentId/affinity',
      data: {
        'user_id': userId,
        'affinity_level': affinityLevel,
      },
    );
  }
}
