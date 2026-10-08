/// AI Agent 模型
///
/// 对齐服务器端 models/ai_agent.rs
library;

import 'persona_config.dart';

/// AI Agent 实体
class AIAgent {
  final String id;
  final String groupId;
  final String name;
  final int toxicityLevel;
  final int enthusiasmLevel;
  final List<String> expertiseDomains;
  final String? speechSuffix;
  final List<String> triggerKeywords;
  final bool isActive;
  final DateTime createdAt;
  final DateTime updatedAt;

  // Phase 3: Enhanced persona fields
  final VoiceSettings? voiceSettings;
  final EmotionBoundaries? emotionBoundaries;
  final StickerPreferences? stickerPreferences;
  final ResponseStyle? responseStyle;
  final ProactiveConfig? proactiveConfig;

  // Phase 4: User customization fields
  final String? customPrompt;
  final String? backstory;
  final String? avatarUrl;
  final String? greetingMessage;
  final String? farewellMessage;
  final String? personaPreset;

  AIAgent({
    required this.id,
    required this.groupId,
    required this.name,
    required this.toxicityLevel,
    required this.enthusiasmLevel,
    required this.expertiseDomains,
    this.speechSuffix,
    required this.triggerKeywords,
    required this.isActive,
    required this.createdAt,
    required this.updatedAt,
    this.voiceSettings,
    this.emotionBoundaries,
    this.stickerPreferences,
    this.responseStyle,
    this.proactiveConfig,
    this.customPrompt,
    this.backstory,
    this.avatarUrl,
    this.greetingMessage,
    this.farewellMessage,
    this.personaPreset,
  });

  factory AIAgent.fromJson(Map<String, dynamic> json) {
    return AIAgent(
      id: _asString(json['id']) ?? '',
      groupId: _asString(json['group_id']) ?? '',
      name: _asString(json['name']) ?? '',
      toxicityLevel: _asInt(json['toxicity_level']),
      enthusiasmLevel: _asInt(json['enthusiasm_level']),
      expertiseDomains: _asList(json['expertise_domains']),
      speechSuffix: _asString(json['speech_suffix']),
      triggerKeywords: _asList(json['trigger_keywords']),
      isActive: json['is_active'] as bool? ?? false,
      createdAt: DateTime.parse(_asString(json['created_at']) ?? DateTime.now().toIso8601String()),
      updatedAt: DateTime.parse(_asString(json['updated_at']) ?? DateTime.now().toIso8601String()),
      voiceSettings: json['voice_settings'] != null
          ? VoiceSettings.fromJson(json['voice_settings'] as Map<String, dynamic>)
          : null,
      emotionBoundaries: json['emotion_boundaries'] != null
          ? EmotionBoundaries.fromJson(json['emotion_boundaries'] as Map<String, dynamic>)
          : null,
      stickerPreferences: json['sticker_preferences'] != null
          ? StickerPreferences.fromJson(json['sticker_preferences'] as Map<String, dynamic>)
          : null,
      responseStyle: json['response_style'] != null
          ? ResponseStyle.fromJson(json['response_style'] as Map<String, dynamic>)
          : null,
      proactiveConfig: json['proactive_config'] != null
          ? ProactiveConfig.fromJson(json['proactive_config'] as Map<String, dynamic>)
          : null,
      customPrompt: _asString(json['custom_prompt']),
      backstory: _asString(json['backstory']),
      avatarUrl: _asString(json['avatar_url']),
      greetingMessage: _asString(json['greeting_message']),
      farewellMessage: _asString(json['farewell_message']),
      personaPreset: _asString(json['persona_preset']),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'group_id': groupId,
      'name': name,
      'toxicity_level': toxicityLevel,
      'enthusiasm_level': enthusiasmLevel,
      'expertise_domains': expertiseDomains,
      'speech_suffix': speechSuffix,
      'trigger_keywords': triggerKeywords,
      'is_active': isActive,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
      if (voiceSettings != null) 'voice_settings': voiceSettings!.toJson(),
      if (emotionBoundaries != null) 'emotion_boundaries': emotionBoundaries!.toJson(),
      if (stickerPreferences != null) 'sticker_preferences': stickerPreferences!.toJson(),
      if (responseStyle != null) 'response_style': responseStyle!.toJson(),
      if (proactiveConfig != null) 'proactive_config': proactiveConfig!.toJson(),
      if (customPrompt != null) 'custom_prompt': customPrompt,
      if (backstory != null) 'backstory': backstory,
      if (avatarUrl != null) 'avatar_url': avatarUrl,
      if (greetingMessage != null) 'greeting_message': greetingMessage,
      if (farewellMessage != null) 'farewell_message': farewellMessage,
      if (personaPreset != null) 'persona_preset': personaPreset,
    };
  }

  /// 判断是否为AI消息（通过sender_id判断）
  bool get isAiMessage => false;

  /// 获取显示名称
  String get displayName => name;

  /// 获取角色类型
  String get roleType => 'ai_agent';
}

/// 创建 AI Agent 请求
class CreateAIAgent {
  final String name;
  final int? toxicityLevel;
  final int? enthusiasmLevel;
  final List<String>? expertiseDomains;
  final String? speechSuffix;
  final List<String>? triggerKeywords;
  final String? preset;

  CreateAIAgent({
    required this.name,
    this.toxicityLevel,
    this.enthusiasmLevel,
    this.expertiseDomains,
    this.speechSuffix,
    this.triggerKeywords,
    this.preset,
  });

  Map<String, dynamic> toJson() {
    return {
      'name': name,
      if (toxicityLevel != null) 'toxicity_level': toxicityLevel,
      if (enthusiasmLevel != null) 'enthusiasm_level': enthusiasmLevel,
      if (expertiseDomains != null) 'expertise_domains': expertiseDomains,
      if (speechSuffix != null) 'speech_suffix': speechSuffix,
      if (triggerKeywords != null) 'trigger_keywords': triggerKeywords,
      if (preset != null) 'preset': preset,
    };
  }
}

/// 更新 AI Agent 人格请求
class UpdatePersonality {
  // Basic fields
  final String? name;
  final int? toxicityLevel;
  final int? enthusiasmLevel;
  final List<String>? expertiseDomains;
  final String? speechSuffix;
  final List<String>? triggerKeywords;

  // Phase 3: Enhanced persona settings
  final VoiceSettings? voiceSettings;
  final EmotionBoundaries? emotionBoundaries;
  final StickerPreferences? stickerPreferences;
  final ResponseStyle? responseStyle;
  final ProactiveConfig? proactiveConfig;

  // Phase 4: User customization
  final String? customPrompt;
  final String? backstory;
  final String? avatarUrl;
  final String? greetingMessage;
  final String? farewellMessage;

  /// Apply a preset template (overwrites other fields)
  final String? preset;

  UpdatePersonality({
    this.name,
    this.toxicityLevel,
    this.enthusiasmLevel,
    this.expertiseDomains,
    this.speechSuffix,
    this.triggerKeywords,
    this.voiceSettings,
    this.emotionBoundaries,
    this.stickerPreferences,
    this.responseStyle,
    this.proactiveConfig,
    this.customPrompt,
    this.backstory,
    this.avatarUrl,
    this.greetingMessage,
    this.farewellMessage,
    this.preset,
  });

  Map<String, dynamic> toJson() {
    return {
      if (name != null) 'name': name,
      if (toxicityLevel != null) 'toxicity_level': toxicityLevel,
      if (enthusiasmLevel != null) 'enthusiasm_level': enthusiasmLevel,
      if (expertiseDomains != null) 'expertise_domains': expertiseDomains,
      if (speechSuffix != null) 'speech_suffix': speechSuffix,
      if (triggerKeywords != null) 'trigger_keywords': triggerKeywords,
      if (voiceSettings != null) 'voice_settings': voiceSettings!.toJson(),
      if (emotionBoundaries != null) 'emotion_boundaries': emotionBoundaries!.toJson(),
      if (stickerPreferences != null) 'sticker_preferences': stickerPreferences!.toJson(),
      if (responseStyle != null) 'response_style': responseStyle!.toJson(),
      if (proactiveConfig != null) 'proactive_config': proactiveConfig!.toJson(),
      if (customPrompt != null) 'custom_prompt': customPrompt,
      if (backstory != null) 'backstory': backstory,
      if (avatarUrl != null) 'avatar_url': avatarUrl,
      if (greetingMessage != null) 'greeting_message': greetingMessage,
      if (farewellMessage != null) 'farewell_message': farewellMessage,
      if (preset != null) 'preset': preset,
    };
  }
}

/// AI Agent 统计信息
class AgentStatistics {
  final String agentId;
  final int totalMessages;
  final DateTime? lastMessageAt;
  final int? avgResponseTimeMs;
  final TriggerCounts triggerCounts;

  AgentStatistics({
    required this.agentId,
    required this.totalMessages,
    this.lastMessageAt,
    this.avgResponseTimeMs,
    required this.triggerCounts,
  });

  factory AgentStatistics.fromJson(Map<String, dynamic> json) {
    return AgentStatistics(
      agentId: _asString(json['agent_id']) ?? '',
      totalMessages: _asInt(json['total_messages']),
      lastMessageAt: json['last_message_at'] != null
          ? DateTime.parse(_asString(json['last_message_at'])!)
          : null,
      avgResponseTimeMs: json['avg_response_time_ms'] != null
          ? _asInt(json['avg_response_time_ms'])
          : null,
      triggerCounts: TriggerCounts.fromJson(
          json['trigger_counts'] as Map<String, dynamic>? ?? const {}),
    );
  }
}

/// 触发计数
class TriggerCounts {
  final int mention;
  final int keyword;
  final int contextSimilarity;
  final int random;
  final int silenceBreaker;
  final int totalEvaluations;
  final int totalResponses;

  const TriggerCounts({
    this.mention = 0,
    this.keyword = 0,
    this.contextSimilarity = 0,
    this.random = 0,
    this.silenceBreaker = 0,
    this.totalEvaluations = 0,
    this.totalResponses = 0,
  });

  factory TriggerCounts.fromJson(Map<String, dynamic> json) {
    return TriggerCounts(
      mention: _asInt(json['mention']),
      keyword: _asInt(json['keyword']),
      contextSimilarity: _asInt(json['context_similarity']),
      random: _asInt(json['random']),
      silenceBreaker: _asInt(json['silence_breaker']),
      totalEvaluations: _asInt(json['total_evaluations']),
      totalResponses: _asInt(json['total_responses']),
    );
  }
}

/// 触发类型
enum TriggerType {
  mention,
  keyword,
  contextSimilarity,
  random,
  silenceBreaker;

  String get value {
    switch (this) {
      case TriggerType.mention:
        return 'mention';
      case TriggerType.keyword:
        return 'keyword';
      case TriggerType.contextSimilarity:
        return 'context_similarity';
      case TriggerType.random:
        return 'random';
      case TriggerType.silenceBreaker:
        return 'silence_breaker';
    }
  }

  static TriggerType? fromString(String value) {
    switch (value) {
      case 'mention':
        return TriggerType.mention;
      case 'keyword':
        return TriggerType.keyword;
      case 'context_similarity':
        return TriggerType.contextSimilarity;
      case 'random':
        return TriggerType.random;
      case 'silence_breaker':
        return TriggerType.silenceBreaker;
      default:
        return null;
    }
  }
}

// 辅助函数
String? _asString(dynamic value) {
  if (value == null) return null;
  if (value is String) return value;
  return value.toString();
}

int _asInt(dynamic value, [int defaultValue = 0]) {
  if (value == null) return defaultValue;
  if (value is int) return value;
  if (value is String) return int.tryParse(value) ?? defaultValue;
  if (value is double) return value.toInt();
  return defaultValue;
}

List<String> _asList(dynamic value) {
  if (value == null) return [];
  if (value is List) {
    return value.map((e) => e?.toString() ?? '').toList();
  }
  return [];
}
