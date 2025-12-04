import 'package:dio/dio.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../core/network/ai_agent_api.dart';
import '../../data/models/ai_agent.dart';

/// Dio provider for AI Agent API
final _aiAgentDioProvider = Provider<Dio>((ref) {
  return Dio(BaseOptions(
    baseUrl: 'http://localhost:8080',
    connectTimeout: const Duration(seconds: 30),
    receiveTimeout: const Duration(seconds: 30),
    headers: {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
    },
  ));
});

/// AI Agent API provider
final aiAgentApiProvider = Provider<AIAgentApi>((ref) {
  final dio = ref.watch(_aiAgentDioProvider);
  return AIAgentApi(dio);
});

/// AI Agent state
class AIAgentState {
  final AIAgent? agent;
  final AIAgentStats? stats;
  final bool isLoading;
  final String? errorMessage;

  const AIAgentState({
    this.agent,
    this.stats,
    this.isLoading = false,
    this.errorMessage,
  });

  AIAgentState copyWith({
    AIAgent? agent,
    AIAgentStats? stats,
    bool? isLoading,
    String? errorMessage,
  }) {
    return AIAgentState(
      agent: agent ?? this.agent,
      stats: stats ?? this.stats,
      isLoading: isLoading ?? this.isLoading,
      errorMessage: errorMessage,
    );
  }
}

/// AI Agent state notifier
class AIAgentNotifier extends StateNotifier<AIAgentState> {
  final AIAgentApi _api;

  AIAgentNotifier(this._api) : super(const AIAgentState());

  /// Load AI Agent for a group
  Future<void> loadAgentForGroup(String groupId) async {
    state = state.copyWith(isLoading: true, errorMessage: null);
    try {
      final agent = await _api.getAgentForGroup(groupId);
      state = state.copyWith(agent: agent, isLoading: false);
      if (agent != null) {
        await loadStats(agent.id);
      }
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        errorMessage: '加载 AI Agent 失败: $e',
      );
    }
  }

  /// Create AI Agent
  Future<bool> createAgent(String groupId, CreateAgentRequest request) async {
    state = state.copyWith(isLoading: true, errorMessage: null);
    try {
      final agent = await _api.createAgent(groupId, request);
      state = state.copyWith(agent: agent, isLoading: false);
      return true;
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        errorMessage: '创建 AI Agent 失败: $e',
      );
      return false;
    }
  }

  /// Update personality
  Future<bool> updatePersonality(AIAgentPersonality personality) async {
    if (state.agent == null) return false;

    state = state.copyWith(isLoading: true, errorMessage: null);
    try {
      final agent = await _api.updatePersonality(state.agent!.id, personality);
      state = state.copyWith(agent: agent, isLoading: false);
      return true;
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        errorMessage: '更新人设失败: $e',
      );
      return false;
    }
  }

  /// Delete AI Agent
  Future<bool> deleteAgent() async {
    if (state.agent == null) return false;

    state = state.copyWith(isLoading: true, errorMessage: null);
    try {
      await _api.deleteAgent(state.agent!.id);
      state = const AIAgentState();
      return true;
    } catch (e) {
      state = state.copyWith(
        isLoading: false,
        errorMessage: '删除 AI Agent 失败: $e',
      );
      return false;
    }
  }

  /// Load statistics
  Future<void> loadStats(String agentId) async {
    try {
      final stats = await _api.getStatistics(agentId);
      state = state.copyWith(stats: stats);
    } catch (e) {
      // Stats loading failure is not critical
    }
  }

  /// Clear state
  void clear() {
    state = const AIAgentState();
  }
}

/// AI Agent provider for a specific group
final aiAgentProvider =
    StateNotifierProvider.family<AIAgentNotifier, AIAgentState, String>(
  (ref, groupId) {
    final api = ref.watch(aiAgentApiProvider);
    final notifier = AIAgentNotifier(api);
    // Auto-load agent for the group
    notifier.loadAgentForGroup(groupId);
    return notifier;
  },
);
