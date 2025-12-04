/// AI Agent model
class AIAgent {
  final String id;
  final String groupId;
  final String name;
  final String? avatarUrl;
  final AIAgentPersonality personality;
  final List<String> triggerKeywords;
  final bool isActive;
  final DateTime createdAt;
  final DateTime updatedAt;

  const AIAgent({
    required this.id,
    required this.groupId,
    required this.name,
    this.avatarUrl,
    required this.personality,
    this.triggerKeywords = const [],
    this.isActive = true,
    required this.createdAt,
    required this.updatedAt,
  });

  factory AIAgent.fromJson(Map<String, dynamic> json) {
    return AIAgent(
      id: json['id'] as String,
      groupId: json['group_id'] as String,
      name: json['name'] as String,
      avatarUrl: json['avatar_url'] as String?,
      personality: AIAgentPersonality.fromJson(
        json['personality'] as Map<String, dynamic>,
      ),
      triggerKeywords: (json['trigger_keywords'] as List<dynamic>?)
              ?.map((e) => e as String)
              .toList() ??
          [],
      isActive: json['is_active'] as bool? ?? true,
      createdAt: DateTime.parse(json['created_at'] as String),
      updatedAt: DateTime.parse(json['updated_at'] as String),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'group_id': groupId,
      'name': name,
      'avatar_url': avatarUrl,
      'personality': personality.toJson(),
      'trigger_keywords': triggerKeywords,
      'is_active': isActive,
      'created_at': createdAt.toIso8601String(),
      'updated_at': updatedAt.toIso8601String(),
    };
  }

  AIAgent copyWith({
    String? id,
    String? groupId,
    String? name,
    String? avatarUrl,
    AIAgentPersonality? personality,
    List<String>? triggerKeywords,
    bool? isActive,
    DateTime? createdAt,
    DateTime? updatedAt,
  }) {
    return AIAgent(
      id: id ?? this.id,
      groupId: groupId ?? this.groupId,
      name: name ?? this.name,
      avatarUrl: avatarUrl ?? this.avatarUrl,
      personality: personality ?? this.personality,
      triggerKeywords: triggerKeywords ?? this.triggerKeywords,
      isActive: isActive ?? this.isActive,
      createdAt: createdAt ?? this.createdAt,
      updatedAt: updatedAt ?? this.updatedAt,
    );
  }
}

/// AI Agent personality configuration
class AIAgentPersonality {
  final int toxicityLevel; // 0-10, 毒舌程度
  final int enthusiasmLevel; // 0-10, 热情度
  final List<String> expertiseDomains; // 专业领域
  final String? speechSuffix; // 口癖/语气词
  final String? customPrompt; // 自定义提示词

  const AIAgentPersonality({
    this.toxicityLevel = 5,
    this.enthusiasmLevel = 5,
    this.expertiseDomains = const [],
    this.speechSuffix,
    this.customPrompt,
  });

  factory AIAgentPersonality.fromJson(Map<String, dynamic> json) {
    return AIAgentPersonality(
      toxicityLevel: json['toxicity_level'] as int? ?? 5,
      enthusiasmLevel: json['enthusiasm_level'] as int? ?? 5,
      expertiseDomains: (json['expertise_domains'] as List<dynamic>?)
              ?.map((e) => e as String)
              .toList() ??
          [],
      speechSuffix: json['speech_suffix'] as String?,
      customPrompt: json['custom_prompt'] as String?,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'toxicity_level': toxicityLevel,
      'enthusiasm_level': enthusiasmLevel,
      'expertise_domains': expertiseDomains,
      'speech_suffix': speechSuffix,
      'custom_prompt': customPrompt,
    };
  }

  AIAgentPersonality copyWith({
    int? toxicityLevel,
    int? enthusiasmLevel,
    List<String>? expertiseDomains,
    String? speechSuffix,
    String? customPrompt,
  }) {
    return AIAgentPersonality(
      toxicityLevel: toxicityLevel ?? this.toxicityLevel,
      enthusiasmLevel: enthusiasmLevel ?? this.enthusiasmLevel,
      expertiseDomains: expertiseDomains ?? this.expertiseDomains,
      speechSuffix: speechSuffix ?? this.speechSuffix,
      customPrompt: customPrompt ?? this.customPrompt,
    );
  }
}

/// AI Agent statistics
class AIAgentStats {
  final int totalMessages;
  final int totalResponses;
  final double averageResponseTime;
  final Map<String, int> triggerCounts;

  const AIAgentStats({
    this.totalMessages = 0,
    this.totalResponses = 0,
    this.averageResponseTime = 0,
    this.triggerCounts = const {},
  });

  factory AIAgentStats.fromJson(Map<String, dynamic> json) {
    return AIAgentStats(
      totalMessages: json['total_messages'] as int? ?? 0,
      totalResponses: json['total_responses'] as int? ?? 0,
      averageResponseTime:
          (json['average_response_time'] as num?)?.toDouble() ?? 0,
      triggerCounts: (json['trigger_counts'] as Map<String, dynamic>?)
              ?.map((k, v) => MapEntry(k, v as int)) ??
          {},
    );
  }
}

/// Request to create AI Agent
class CreateAgentRequest {
  final String name;
  final String? avatarUrl;
  final int toxicityLevel;
  final int enthusiasmLevel;
  final List<String> expertiseDomains;
  final String? speechSuffix;
  final List<String> triggerKeywords;

  const CreateAgentRequest({
    required this.name,
    this.avatarUrl,
    this.toxicityLevel = 5,
    this.enthusiasmLevel = 5,
    this.expertiseDomains = const [],
    this.speechSuffix,
    this.triggerKeywords = const [],
  });

  Map<String, dynamic> toJson() {
    return {
      'name': name,
      'avatar_url': avatarUrl,
      'toxicity_level': toxicityLevel,
      'enthusiasm_level': enthusiasmLevel,
      'expertise_domains': expertiseDomains,
      'speech_suffix': speechSuffix,
      'trigger_keywords': triggerKeywords,
    };
  }
}
