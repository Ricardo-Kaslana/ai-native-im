/// Persona 配置模型
///
/// 对齐服务器端 services/ai/persona/mod.rs
library;

/// TTS 语音设置
class VoiceSettings {
  final String voiceId;
  final double speed;
  final double pitch;
  final String provider;
  final bool enabled;

  const VoiceSettings({
    this.voiceId = 'zh-CN-XiaoxiaoNeural',
    this.speed = 1.0,
    this.pitch = 0.0,
    this.provider = 'edge',
    this.enabled = false,
  });

  factory VoiceSettings.fromJson(Map<String, dynamic> json) {
    return VoiceSettings(
      voiceId: json['voice_id'] as String? ?? 'zh-CN-XiaoxiaoNeural',
      speed: (json['speed'] as num?)?.toDouble() ?? 1.0,
      pitch: (json['pitch'] as num?)?.toDouble() ?? 0.0,
      provider: json['provider'] as String? ?? 'edge',
      enabled: json['enabled'] as bool? ?? false,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'voice_id': voiceId,
      'speed': speed,
      'pitch': pitch,
      'provider': provider,
      'enabled': enabled,
    };
  }

  VoiceSettings copyWith({
    String? voiceId,
    double? speed,
    double? pitch,
    String? provider,
    bool? enabled,
  }) {
    return VoiceSettings(
      voiceId: voiceId ?? this.voiceId,
      speed: speed ?? this.speed,
      pitch: pitch ?? this.pitch,
      provider: provider ?? this.provider,
      enabled: enabled ?? this.enabled,
    );
  }
}

/// 情感边界
class EmotionBoundaries {
  final List<EmotionState> allowedEmotions;
  final List<EmotionState> forbiddenEmotions;
  final double maxIntensity;
  final bool mirrorUserEmotions;
  final EmotionState defaultEmotion;

  const EmotionBoundaries({
    this.allowedEmotions = EmotionState.defaultAllowed,
    this.forbiddenEmotions = const [EmotionState.angry],
    this.maxIntensity = 0.8,
    this.mirrorUserEmotions = true,
    this.defaultEmotion = EmotionState.neutral,
  });

  factory EmotionBoundaries.fromJson(Map<String, dynamic> json) {
    return EmotionBoundaries(
      allowedEmotions: (json['allowed_emotions'] as List?)
              ?.map((e) => EmotionState.fromString(e as String))
              .where((e) => e != null)
              .cast<EmotionState>()
              .toList() ??
          EmotionState.defaultAllowed,
      forbiddenEmotions: (json['forbidden_emotions'] as List?)
              ?.map((e) => EmotionState.fromString(e as String))
              .where((e) => e != null)
              .cast<EmotionState>()
              .toList() ??
          const [EmotionState.angry],
      maxIntensity: (json['max_intensity'] as num?)?.toDouble() ?? 0.8,
      mirrorUserEmotions: json['mirror_user_emotions'] as bool? ?? true,
      defaultEmotion: EmotionState.fromString(json['default_emotion'] as String?) ?? EmotionState.neutral,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'allowed_emotions': allowedEmotions.map((e) => e.value).toList(),
      'forbidden_emotions': forbiddenEmotions.map((e) => e.value).toList(),
      'max_intensity': maxIntensity,
      'mirror_user_emotions': mirrorUserEmotions,
      'default_emotion': defaultEmotion.value,
    };
  }

  EmotionBoundaries copyWith({
    List<EmotionState>? allowedEmotions,
    List<EmotionState>? forbiddenEmotions,
    double? maxIntensity,
    bool? mirrorUserEmotions,
    EmotionState? defaultEmotion,
  }) {
    return EmotionBoundaries(
      allowedEmotions: allowedEmotions ?? this.allowedEmotions,
      forbiddenEmotions: forbiddenEmotions ?? this.forbiddenEmotions,
      maxIntensity: maxIntensity ?? this.maxIntensity,
      mirrorUserEmotions: mirrorUserEmotions ?? this.mirrorUserEmotions,
      defaultEmotion: defaultEmotion ?? this.defaultEmotion,
    );
  }
}

/// 表情偏好
class StickerPreferences {
  final bool enabled;
  final double probability;
  final List<String> preferredPacks;
  final List<EmotionStickerMapping> emotionMappings;
  final int maxPerMessage;

  const StickerPreferences({
    this.enabled = true,
    this.probability = 0.3,
    this.preferredPacks = const ['default'],
    this.emotionMappings = const [
      EmotionStickerMapping(
        emotion: EmotionState.happy,
        stickerIds: ['happy_1', 'happy_2'],
      ),
      EmotionStickerMapping(
        emotion: EmotionState.sad,
        stickerIds: ['sad_1', 'comfort_1'],
      ),
    ],
    this.maxPerMessage = 1,
  });

  factory StickerPreferences.fromJson(Map<String, dynamic> json) {
    return StickerPreferences(
      enabled: json['enabled'] as bool? ?? true,
      probability: (json['probability'] as num?)?.toDouble() ?? 0.3,
      preferredPacks: (json['preferred_packs'] as List?)?.cast<String>() ?? const ['default'],
      emotionMappings: (json['emotion_mappings'] as List?)
              ?.map((e) => EmotionStickerMapping.fromJson(e as Map<String, dynamic>))
              .toList() ??
          const [
            EmotionStickerMapping(
              emotion: EmotionState.happy,
              stickerIds: ['happy_1', 'happy_2'],
            ),
          ],
      maxPerMessage: json['max_per_message'] as int? ?? 1,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'enabled': enabled,
      'probability': probability,
      'preferred_packs': preferredPacks,
      'emotion_mappings': emotionMappings.map((e) => e.toJson()).toList(),
      'max_per_message': maxPerMessage,
    };
  }

  StickerPreferences copyWith({
    bool? enabled,
    double? probability,
    List<String>? preferredPacks,
    List<EmotionStickerMapping>? emotionMappings,
    int? maxPerMessage,
  }) {
    return StickerPreferences(
      enabled: enabled ?? this.enabled,
      probability: probability ?? this.probability,
      preferredPacks: preferredPacks ?? this.preferredPacks,
      emotionMappings: emotionMappings ?? this.emotionMappings,
      maxPerMessage: maxPerMessage ?? this.maxPerMessage,
    );
  }
}

/// 情感到表情的映射
class EmotionStickerMapping {
  final EmotionState emotion;
  final List<String> stickerIds;

  const EmotionStickerMapping({
    required this.emotion,
    required this.stickerIds,
  });

  factory EmotionStickerMapping.fromJson(Map<String, dynamic> json) {
    return EmotionStickerMapping(
      emotion: EmotionState.fromString(json['emotion'] as String?) ?? EmotionState.neutral,
      stickerIds: (json['sticker_ids'] as List?)?.cast<String>() ?? [],
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'emotion': emotion.value,
      'sticker_ids': stickerIds,
    };
  }
}

/// 响应风格
class ResponseStyle {
  final ResponseLength length;
  final bool useEmojis;
  final double emojiFrequency;
  final int formality;
  final bool useMarkdown;

  const ResponseStyle({
    this.length = ResponseLength.medium,
    this.useEmojis = true,
    this.emojiFrequency = 0.3,
    this.formality = 5,
    this.useMarkdown = false,
  });

  factory ResponseStyle.fromJson(Map<String, dynamic> json) {
    final lengthStr = json['length'] as String?;
    return ResponseStyle(
      length: lengthStr != null ? ResponseLength.fromString(lengthStr) : ResponseLength.medium,
      useEmojis: json['use_emojis'] as bool? ?? true,
      emojiFrequency: (json['emoji_frequency'] as num?)?.toDouble() ?? 0.3,
      formality: json['formality'] as int? ?? 5,
      useMarkdown: json['use_markdown'] as bool? ?? false,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'length': length.value,
      'use_emojis': useEmojis,
      'emoji_frequency': emojiFrequency,
      'formality': formality,
      'use_markdown': useMarkdown,
    };
  }

  ResponseStyle copyWith({
    ResponseLength? length,
    bool? useEmojis,
    double? emojiFrequency,
    int? formality,
    bool? useMarkdown,
  }) {
    return ResponseStyle(
      length: length ?? this.length,
      useEmojis: useEmojis ?? this.useEmojis,
      emojiFrequency: emojiFrequency ?? this.emojiFrequency,
      formality: formality ?? this.formality,
      useMarkdown: useMarkdown ?? this.useMarkdown,
    );
  }
}

/// 响应长度
enum ResponseLength {
  short,
  medium,
  long;

  String get value {
    switch (this) {
      case ResponseLength.short:
        return 'short';
      case ResponseLength.medium:
        return 'medium';
      case ResponseLength.long:
        return 'long';
    }
  }

  static ResponseLength fromString(String? value) {
    switch (value) {
      case 'short':
        return ResponseLength.short;
      case 'medium':
        return ResponseLength.medium;
      case 'long':
        return ResponseLength.long;
      default:
        return ResponseLength.medium;
    }
  }

  String get displayName {
    switch (this) {
      case ResponseLength.short:
        return '简短';
      case ResponseLength.medium:
        return '中等';
      case ResponseLength.long:
        return '详细';
    }
  }
}

/// 主动消息配置
class ProactiveConfig {
  final bool enabled;
  final int silenceThresholdMinutes;
  final int cooldownMinutes;
  final int dailyLimit;

  const ProactiveConfig({
    this.enabled = false,
    this.silenceThresholdMinutes = 120,
    this.cooldownMinutes = 30,
    this.dailyLimit = 10,
  });

  factory ProactiveConfig.fromJson(Map<String, dynamic> json) {
    return ProactiveConfig(
      enabled: json['enabled'] as bool? ?? false,
      silenceThresholdMinutes: json['silence_threshold_minutes'] as int? ?? 120,
      cooldownMinutes: json['cooldown_minutes'] as int? ?? 30,
      dailyLimit: json['daily_limit'] as int? ?? 10,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'enabled': enabled,
      'silence_threshold_minutes': silenceThresholdMinutes,
      'cooldown_minutes': cooldownMinutes,
      'daily_limit': dailyLimit,
    };
  }

  ProactiveConfig copyWith({
    bool? enabled,
    int? silenceThresholdMinutes,
    int? cooldownMinutes,
    int? dailyLimit,
  }) {
    return ProactiveConfig(
      enabled: enabled ?? this.enabled,
      silenceThresholdMinutes: silenceThresholdMinutes ?? this.silenceThresholdMinutes,
      cooldownMinutes: cooldownMinutes ?? this.cooldownMinutes,
      dailyLimit: dailyLimit ?? this.dailyLimit,
    );
  }
}

/// 情感状态
enum EmotionState {
  neutral,
  happy,
  sad,
  angry,
  curious,
  supportive,
  playful,
  excited,
  tired,
  surprised;

  static const List<EmotionState> defaultAllowed = [
    EmotionState.neutral,
    EmotionState.happy,
    EmotionState.curious,
    EmotionState.supportive,
    EmotionState.playful,
  ];

  String get value {
    switch (this) {
      case EmotionState.neutral:
        return 'neutral';
      case EmotionState.happy:
        return 'happy';
      case EmotionState.sad:
        return 'sad';
      case EmotionState.angry:
        return 'angry';
      case EmotionState.curious:
        return 'curious';
      case EmotionState.supportive:
        return 'supportive';
      case EmotionState.playful:
        return 'playful';
      case EmotionState.excited:
        return 'excited';
      case EmotionState.tired:
        return 'tired';
      case EmotionState.surprised:
        return 'surprised';
    }
  }

  String get displayName {
    switch (this) {
      case EmotionState.neutral:
        return '平静';
      case EmotionState.happy:
        return '开心';
      case EmotionState.sad:
        return '难过';
      case EmotionState.angry:
        return '生气';
      case EmotionState.curious:
        return '好奇';
      case EmotionState.supportive:
        return '支持';
      case EmotionState.playful:
        return '调皮';
      case EmotionState.excited:
        return '兴奋';
      case EmotionState.tired:
        return '疲惫';
      case EmotionState.surprised:
        return '惊讶';
    }
  }

  static EmotionState? fromString(String? value) {
    if (value == null) return null;
    switch (value) {
      case 'neutral':
        return EmotionState.neutral;
      case 'happy':
        return EmotionState.happy;
      case 'sad':
        return EmotionState.sad;
      case 'angry':
        return EmotionState.angry;
      case 'curious':
        return EmotionState.curious;
      case 'supportive':
        return EmotionState.supportive;
      case 'playful':
        return EmotionState.playful;
      case 'excited':
        return EmotionState.excited;
      case 'tired':
        return EmotionState.tired;
      case 'surprised':
        return EmotionState.surprised;
      default:
        return null;
    }
  }
}

/// 人设预设信息
class PersonaPresetInfo {
  final String id;
  final String name;
  final String description;
  final int toxicityLevel;
  final int enthusiasmLevel;
  final String? speechSuffix;
  final String previewGreeting;
  final String expertiseHint;

  const PersonaPresetInfo({
    required this.id,
    required this.name,
    required this.description,
    required this.toxicityLevel,
    required this.enthusiasmLevel,
    this.speechSuffix,
    required this.previewGreeting,
    required this.expertiseHint,
  });

  factory PersonaPresetInfo.fromJson(Map<String, dynamic> json) {
    return PersonaPresetInfo(
      id: json['id'] as String? ?? '',
      name: json['name'] as String? ?? '',
      description: json['description'] as String? ?? '',
      toxicityLevel: json['toxicity_level'] as int? ?? 5,
      enthusiasmLevel: json['enthusiasm_level'] as int? ?? 5,
      speechSuffix: json['speech_suffix'] as String?,
      previewGreeting: json['preview_greeting'] as String? ?? '',
      expertiseHint: json['expertise_hint'] as String? ?? '',
    );
  }
}

/// 测试人设请求
class TestPersonaRequest {
  final String message;
  final String? senderName;

  TestPersonaRequest({
    required this.message,
    this.senderName,
  });

  Map<String, dynamic> toJson() {
    return {
      'message': message,
      if (senderName != null) 'sender_name': senderName,
    };
  }
}

/// 测试人设响应
class TestPersonaResponse {
  final String response;
  final String emotion;
  final int processingTimeMs;

  TestPersonaResponse({
    required this.response,
    required this.emotion,
    required this.processingTimeMs,
  });

  factory TestPersonaResponse.fromJson(Map<String, dynamic> json) {
    return TestPersonaResponse(
      response: json['response'] as String? ?? '',
      emotion: json['emotion'] as String? ?? 'neutral',
      processingTimeMs: json['processing_time_ms'] as int? ?? 0,
    );
  }
}

/// 消息总结
class Summary {
  final String content;
  final String style;
  final int messageCount;
  final DateTime generatedAt;

  Summary({
    required this.content,
    required this.style,
    required this.messageCount,
    required this.generatedAt,
  });

  factory Summary.fromJson(Map<String, dynamic> json) {
    return Summary(
      content: json['content'] as String? ?? '',
      style: json['style'] as String? ?? 'brief',
      messageCount: json['message_count'] as int? ?? 0,
      generatedAt: DateTime.parse(json['generated_at'] as String? ?? DateTime.now().toIso8601String()),
    );
  }
}

/// 总结风格
enum SummaryStyle {
  brief,
  detailed,
  topics,
  actionItems;

  String get value {
    switch (this) {
      case SummaryStyle.brief:
        return 'brief';
      case SummaryStyle.detailed:
        return 'detailed';
      case SummaryStyle.topics:
        return 'topics';
      case SummaryStyle.actionItems:
        return 'action_items';
    }
  }

  String get displayName {
    switch (this) {
      case SummaryStyle.brief:
        return '简短';
      case SummaryStyle.detailed:
        return '详细';
      case SummaryStyle.topics:
        return '话题列表';
      case SummaryStyle.actionItems:
        return '行动项';
    }
  }

  static SummaryStyle fromString(String value) {
    switch (value) {
      case 'brief':
        return SummaryStyle.brief;
      case 'detailed':
        return SummaryStyle.detailed;
      case 'topics':
        return SummaryStyle.topics;
      case 'action_items':
        return SummaryStyle.actionItems;
      default:
        return SummaryStyle.brief;
    }
  }
}

/// 反馈统计
class FeedbackStats {
  final int positiveCount;
  final int negativeCount;
  final int correctionCount;
  final int ignoredCount;
  final int totalFeedback;

  const FeedbackStats({
    this.positiveCount = 0,
    this.negativeCount = 0,
    this.correctionCount = 0,
    this.ignoredCount = 0,
    this.totalFeedback = 0,
  });

  factory FeedbackStats.fromJson(Map<String, dynamic> json) {
    return FeedbackStats(
      positiveCount: json['positive_count'] as int? ?? 0,
      negativeCount: json['negative_count'] as int? ?? 0,
      correctionCount: json['correction_count'] as int? ?? 0,
      ignoredCount: json['ignored_count'] as int? ?? 0,
      totalFeedback: json['total_feedback'] as int? ?? 0,
    );
  }
}
