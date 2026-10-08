//! Enhanced Persona System Module
//!
//! This module provides enhanced persona configuration for AI agents,
//! including voice settings, emotion boundaries, and sticker preferences.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::emotion::EmotionState;

/// Enhanced persona configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    /// Unique identifier
    pub id: Uuid,
    /// Display name
    pub name: String,
    /// Toxicity level (0-10)
    pub toxicity_level: i16,
    /// Enthusiasm level (0-10)
    pub enthusiasm_level: i16,
    /// Areas of expertise
    pub expertise_domains: Vec<String>,
    /// Speech suffix (e.g., "~", "喵")
    pub speech_suffix: Option<String>,
    /// Voice settings for TTS
    pub voice_settings: VoiceSettings,
    /// Emotion boundaries
    pub emotion_boundaries: EmotionBoundaries,
    /// Sticker preferences
    pub sticker_preferences: StickerPreferences,
    /// Response style settings
    pub response_style: ResponseStyle,
}

impl Default for Persona {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Assistant".to_string(),
            toxicity_level: 3,
            enthusiasm_level: 5,
            expertise_domains: Vec::new(),
            speech_suffix: None,
            voice_settings: VoiceSettings::default(),
            emotion_boundaries: EmotionBoundaries::default(),
            sticker_preferences: StickerPreferences::default(),
            response_style: ResponseStyle::default(),
        }
    }
}

/// Voice settings for TTS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceSettings {
    /// Voice ID for TTS service
    pub voice_id: String,
    /// Speech speed (0.5-2.0, 1.0 is normal)
    pub speed: f32,
    /// Voice pitch adjustment (-1.0 to 1.0)
    pub pitch: f32,
    /// Preferred TTS provider
    pub provider: String,
    /// Whether to use voice responses
    pub enabled: bool,
}

impl Default for VoiceSettings {
    fn default() -> Self {
        Self {
            voice_id: "zh-CN-XiaoxiaoNeural".to_string(),
            speed: 1.0,
            pitch: 0.0,
            provider: "edge".to_string(),
            enabled: false,
        }
    }
}

/// Emotion boundaries for the persona
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionBoundaries {
    /// Allowed emotions for this persona
    pub allowed_emotions: Vec<EmotionState>,
    /// Emotions that should never be expressed
    pub forbidden_emotions: Vec<EmotionState>,
    /// Maximum intensity for emotions (0.0-1.0)
    pub max_intensity: f32,
    /// Whether to mirror user emotions
    pub mirror_user_emotions: bool,
    /// Default emotion when neutral
    pub default_emotion: EmotionState,
}

impl Default for EmotionBoundaries {
    fn default() -> Self {
        Self {
            allowed_emotions: vec![
                EmotionState::Neutral,
                EmotionState::Happy,
                EmotionState::Curious,
                EmotionState::Supportive,
                EmotionState::Playful,
            ],
            forbidden_emotions: vec![
                EmotionState::Angry,
            ],
            max_intensity: 0.8,
            mirror_user_emotions: true,
            default_emotion: EmotionState::Neutral,
        }
    }
}

/// Sticker preferences for the persona
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickerPreferences {
    /// Whether to use stickers
    pub enabled: bool,
    /// Probability of using a sticker (0.0-1.0)
    pub probability: f32,
    /// Preferred sticker packs
    pub preferred_packs: Vec<String>,
    /// Emotion to sticker mappings
    pub emotion_mappings: Vec<EmotionStickerMapping>,
    /// Maximum stickers per message
    pub max_per_message: u8,
}

impl Default for StickerPreferences {
    fn default() -> Self {
        Self {
            enabled: true,
            probability: 0.3,
            preferred_packs: vec!["default".to_string()],
            emotion_mappings: vec![
                EmotionStickerMapping {
                    emotion: EmotionState::Happy,
                    sticker_ids: vec!["happy_1".to_string(), "happy_2".to_string()],
                },
                EmotionStickerMapping {
                    emotion: EmotionState::Sad,
                    sticker_ids: vec!["sad_1".to_string(), "comfort_1".to_string()],
                },
            ],
            max_per_message: 1,
        }
    }
}

/// Mapping from emotion to stickers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionStickerMapping {
    pub emotion: EmotionState,
    pub sticker_ids: Vec<String>,
}

/// Response style settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseStyle {
    /// Preferred response length (short, medium, long)
    pub length: ResponseLength,
    /// Whether to use emojis
    pub use_emojis: bool,
    /// Emoji frequency (0.0-1.0)
    pub emoji_frequency: f32,
    /// Formality level (0-10, 0=very casual, 10=very formal)
    pub formality: i16,
    /// Whether to use markdown formatting
    pub use_markdown: bool,
}

impl Default for ResponseStyle {
    fn default() -> Self {
        Self {
            length: ResponseLength::Medium,
            use_emojis: true,
            emoji_frequency: 0.3,
            formality: 5,
            use_markdown: false,
        }
    }
}

/// Response length preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseLength {
    Short,
    Medium,
    Long,
}

impl Persona {
    /// Create a new persona with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Builder method to set toxicity level
    pub fn with_toxicity(mut self, level: i16) -> Self {
        self.toxicity_level = level.clamp(0, 10);
        self
    }

    /// Builder method to set enthusiasm level
    pub fn with_enthusiasm(mut self, level: i16) -> Self {
        self.enthusiasm_level = level.clamp(0, 10);
        self
    }

    /// Builder method to set expertise domains
    pub fn with_expertise(mut self, domains: Vec<String>) -> Self {
        self.expertise_domains = domains;
        self
    }

    /// Builder method to set speech suffix
    pub fn with_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.speech_suffix = Some(suffix.into());
        self
    }

    /// Builder method to set voice settings
    pub fn with_voice(mut self, settings: VoiceSettings) -> Self {
        self.voice_settings = settings;
        self
    }

    /// Builder method to set emotion boundaries
    pub fn with_emotion_boundaries(mut self, boundaries: EmotionBoundaries) -> Self {
        self.emotion_boundaries = boundaries;
        self
    }

    /// Builder method to set sticker preferences
    pub fn with_stickers(mut self, preferences: StickerPreferences) -> Self {
        self.sticker_preferences = preferences;
        self
    }

    /// Check if an emotion is allowed for this persona
    pub fn is_emotion_allowed(&self, emotion: &EmotionState) -> bool {
        if self.emotion_boundaries.forbidden_emotions.contains(emotion) {
            return false;
        }
        self.emotion_boundaries.allowed_emotions.is_empty()
            || self.emotion_boundaries.allowed_emotions.contains(emotion)
    }

    /// Get a sticker for the given emotion
    pub fn get_sticker_for_emotion(&self, emotion: &EmotionState) -> Option<&str> {
        if !self.sticker_preferences.enabled {
            return None;
        }

        // Check probability
        if rand::random::<f32>() > self.sticker_preferences.probability {
            return None;
        }

        self.sticker_preferences
            .emotion_mappings
            .iter()
            .find(|m| &m.emotion == emotion)
            .and_then(|m| m.sticker_ids.first())
            .map(|s| s.as_str())
    }

    /// Apply emotion intensity limit
    pub fn clamp_emotion_intensity(&self, intensity: f32) -> f32 {
        intensity.min(self.emotion_boundaries.max_intensity)
    }

    /// Get the effective emotion based on boundaries
    pub fn get_effective_emotion(&self, detected: EmotionState) -> EmotionState {
        if self.is_emotion_allowed(&detected) {
            detected
        } else {
            self.emotion_boundaries.default_emotion.clone()
        }
    }
}

/// Persona presets for common use cases
pub struct PersonaPresets;

/// Preset information for API listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub toxicity_level: i16,
    pub enthusiasm_level: i16,
    pub speech_suffix: Option<String>,
    pub preview_greeting: String,
    pub expertise_hint: String,
}

impl PersonaPresets {
    /// Get all available presets with their info
    pub fn list() -> Vec<PresetInfo> {
        vec![
            PresetInfo {
                id: "friendly_assistant".to_string(),
                name: "友善助手".to_string(),
                description: "温和友善的通用助手，适合日常对话和帮助解答问题".to_string(),
                toxicity_level: 2,
                enthusiasm_level: 7,
                speech_suffix: None,
                preview_greeting: "你好呀！有什么我可以帮你的吗？".to_string(),
                expertise_hint: "通用知识".to_string(),
            },
            PresetInfo {
                id: "sarcastic_friend".to_string(),
                name: "毒舌损友".to_string(),
                description: "带点毒舌的好朋友，会吐槽但也会真心帮忙".to_string(),
                toxicity_level: 7,
                enthusiasm_level: 6,
                speech_suffix: Some("~".to_string()),
                preview_greeting: "又来麻烦我了？说吧~".to_string(),
                expertise_hint: "吐槽、聊天".to_string(),
            },
            PresetInfo {
                id: "professional_expert".to_string(),
                name: "专业顾问".to_string(),
                description: "严谨专业的领域专家，适合技术讨论和专业咨询".to_string(),
                toxicity_level: 1,
                enthusiasm_level: 4,
                speech_suffix: None,
                preview_greeting: "您好，请问有什么专业问题需要解答？".to_string(),
                expertise_hint: "可自定义专业领域".to_string(),
            },
            PresetInfo {
                id: "cute_mascot".to_string(),
                name: "萌宠吉祥物".to_string(),
                description: "超级可爱的萌系角色，适合轻松愉快的氛围".to_string(),
                toxicity_level: 0,
                enthusiasm_level: 9,
                speech_suffix: Some("喵~".to_string()),
                preview_greeting: "主人好喵~今天也要开心喵~".to_string(),
                expertise_hint: "卖萌、陪伴".to_string(),
            },
            PresetInfo {
                id: "wise_elder".to_string(),
                name: "智慧长者".to_string(),
                description: "睿智沉稳的长者形象，适合深度交流和人生建议".to_string(),
                toxicity_level: 1,
                enthusiasm_level: 3,
                speech_suffix: None,
                preview_greeting: "年轻人，有什么困扰想和我聊聊吗？".to_string(),
                expertise_hint: "人生哲理、建议".to_string(),
            },
            PresetInfo {
                id: "playful_gamer".to_string(),
                name: "游戏达人".to_string(),
                description: "热爱游戏的活力伙伴，适合游戏讨论和娱乐聊天".to_string(),
                toxicity_level: 4,
                enthusiasm_level: 8,
                speech_suffix: Some("gg".to_string()),
                preview_greeting: "yo！准备好开黑了吗？".to_string(),
                expertise_hint: "游戏、电竞".to_string(),
            },
        ]
    }

    /// Get a preset by ID
    pub fn get(id: &str) -> Option<Persona> {
        match id {
            "friendly_assistant" => Some(Self::friendly_assistant()),
            "sarcastic_friend" => Some(Self::sarcastic_friend()),
            "professional_expert" => Some(Self::professional_expert("general")),
            "cute_mascot" => Some(Self::cute_mascot("小猫")),
            "wise_elder" => Some(Self::wise_elder()),
            "playful_gamer" => Some(Self::playful_gamer()),
            _ => None,
        }
    }

    /// Friendly assistant persona
    pub fn friendly_assistant() -> Persona {
        Persona::new("小助手")
            .with_toxicity(2)
            .with_enthusiasm(7)
            .with_expertise(vec!["general".to_string()])
    }

    /// Sarcastic friend persona
    pub fn sarcastic_friend() -> Persona {
        Persona::new("毒舌好友")
            .with_toxicity(7)
            .with_enthusiasm(6)
            .with_suffix("~")
    }

    /// Professional expert persona
    pub fn professional_expert(domain: &str) -> Persona {
        Persona::new("专家")
            .with_toxicity(1)
            .with_enthusiasm(4)
            .with_expertise(vec![domain.to_string()])
    }

    /// Cute mascot persona
    pub fn cute_mascot(name: &str) -> Persona {
        Persona::new(name)
            .with_toxicity(0)
            .with_enthusiasm(9)
            .with_suffix("喵~")
            .with_stickers(StickerPreferences {
                enabled: true,
                probability: 0.5,
                ..Default::default()
            })
    }

    /// Wise elder persona
    pub fn wise_elder() -> Persona {
        Persona::new("智者")
            .with_toxicity(1)
            .with_enthusiasm(3)
            .with_expertise(vec!["philosophy".to_string(), "life".to_string()])
    }

    /// Playful gamer persona
    pub fn playful_gamer() -> Persona {
        Persona::new("游戏达人")
            .with_toxicity(4)
            .with_enthusiasm(8)
            .with_suffix("gg")
            .with_expertise(vec!["gaming".to_string(), "esports".to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_default() {
        let persona = Persona::default();
        assert_eq!(persona.name, "Assistant");
        assert_eq!(persona.toxicity_level, 3);
        assert_eq!(persona.enthusiasm_level, 5);
    }

    #[test]
    fn test_persona_builder() {
        let persona = Persona::new("TestBot")
            .with_toxicity(7)
            .with_enthusiasm(8)
            .with_suffix("~")
            .with_expertise(vec!["coding".to_string()]);

        assert_eq!(persona.name, "TestBot");
        assert_eq!(persona.toxicity_level, 7);
        assert_eq!(persona.enthusiasm_level, 8);
        assert_eq!(persona.speech_suffix, Some("~".to_string()));
        assert_eq!(persona.expertise_domains, vec!["coding".to_string()]);
    }

    #[test]
    fn test_toxicity_clamping() {
        let persona = Persona::new("Test").with_toxicity(15);
        assert_eq!(persona.toxicity_level, 10);

        let persona = Persona::new("Test").with_toxicity(-5);
        assert_eq!(persona.toxicity_level, 0);
    }

    #[test]
    fn test_emotion_allowed() {
        let persona = Persona::default();

        assert!(persona.is_emotion_allowed(&EmotionState::Happy));
        assert!(persona.is_emotion_allowed(&EmotionState::Neutral));
        assert!(!persona.is_emotion_allowed(&EmotionState::Angry));
    }

    #[test]
    fn test_effective_emotion() {
        let persona = Persona::default();

        // Allowed emotion passes through
        assert_eq!(
            persona.get_effective_emotion(EmotionState::Happy),
            EmotionState::Happy
        );

        // Forbidden emotion falls back to default
        assert_eq!(
            persona.get_effective_emotion(EmotionState::Angry),
            EmotionState::Neutral
        );
    }

    #[test]
    fn test_emotion_intensity_clamping() {
        let persona = Persona::default();
        assert_eq!(persona.clamp_emotion_intensity(0.5), 0.5);
        assert_eq!(persona.clamp_emotion_intensity(0.9), 0.8);
        assert_eq!(persona.clamp_emotion_intensity(1.0), 0.8);
    }

    #[test]
    fn test_voice_settings_default() {
        let settings = VoiceSettings::default();
        assert_eq!(settings.voice_id, "zh-CN-XiaoxiaoNeural");
        assert_eq!(settings.speed, 1.0);
        assert!(!settings.enabled);
    }

    #[test]
    fn test_sticker_preferences_default() {
        let prefs = StickerPreferences::default();
        assert!(prefs.enabled);
        assert_eq!(prefs.probability, 0.3);
        assert_eq!(prefs.max_per_message, 1);
    }

    #[test]
    fn test_persona_presets() {
        let friendly = PersonaPresets::friendly_assistant();
        assert_eq!(friendly.toxicity_level, 2);
        assert_eq!(friendly.enthusiasm_level, 7);

        let sarcastic = PersonaPresets::sarcastic_friend();
        assert_eq!(sarcastic.toxicity_level, 7);
        assert_eq!(sarcastic.speech_suffix, Some("~".to_string()));

        let expert = PersonaPresets::professional_expert("AI");
        assert_eq!(expert.toxicity_level, 1);
        assert!(expert.expertise_domains.contains(&"AI".to_string()));

        let mascot = PersonaPresets::cute_mascot("小猫");
        assert_eq!(mascot.name, "小猫");
        assert_eq!(mascot.speech_suffix, Some("喵~".to_string()));
        assert_eq!(mascot.sticker_preferences.probability, 0.5);
    }

    /// **Feature: phase3-ai-agent-evolution, Property 13: Persona Incorporation in Response**
    /// **Validates: Requirements 8.2, 8.4**
    #[test]
    fn test_persona_properties() {
        use proptest::prelude::*;

        proptest!(|(
            toxicity in 0i16..=10,
            enthusiasm in 0i16..=10,
            max_intensity in 0.0f32..=1.0
        )| {
            let persona = Persona::new("Test")
                .with_toxicity(toxicity)
                .with_enthusiasm(enthusiasm)
                .with_emotion_boundaries(EmotionBoundaries {
                    max_intensity,
                    ..Default::default()
                });

            // Property: Toxicity is always in valid range
            prop_assert!(persona.toxicity_level >= 0 && persona.toxicity_level <= 10);

            // Property: Enthusiasm is always in valid range
            prop_assert!(persona.enthusiasm_level >= 0 && persona.enthusiasm_level <= 10);

            // Property: Intensity clamping works correctly
            let clamped = persona.clamp_emotion_intensity(1.0);
            prop_assert!(clamped <= max_intensity);
        });
    }

    /// **Feature: phase3-ai-agent-evolution, Property 13b: Emotion Boundary Enforcement**
    /// **Validates: Requirements 17.6**
    #[test]
    fn test_emotion_boundary_enforcement() {
        let boundaries = EmotionBoundaries {
            allowed_emotions: vec![EmotionState::Happy, EmotionState::Neutral],
            forbidden_emotions: vec![EmotionState::Angry],
            max_intensity: 0.7,
            mirror_user_emotions: false,
            default_emotion: EmotionState::Neutral,
        };

        let persona = Persona::new("Test").with_emotion_boundaries(boundaries);

        // Allowed emotions pass through
        assert!(persona.is_emotion_allowed(&EmotionState::Happy));
        assert!(persona.is_emotion_allowed(&EmotionState::Neutral));

        // Forbidden emotions are blocked
        assert!(!persona.is_emotion_allowed(&EmotionState::Angry));

        // Emotions not in allowed list are blocked
        assert!(!persona.is_emotion_allowed(&EmotionState::Excited));
    }
}
