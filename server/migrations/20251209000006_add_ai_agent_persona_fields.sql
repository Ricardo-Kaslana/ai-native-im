-- Phase 3: AI Agent Evolution - Enhanced Persona Fields
-- Adds voice settings, emotion boundaries, and sticker preferences to AI agents

-- Add new persona-related columns to ai_agents table
ALTER TABLE ai_agents 
ADD COLUMN IF NOT EXISTS voice_settings JSONB DEFAULT '{"voice_id": "zh-CN-XiaoxiaoNeural", "speed": 1.0, "pitch": 0.0, "provider": "edge", "enabled": false}'::jsonb,
ADD COLUMN IF NOT EXISTS emotion_boundaries JSONB DEFAULT '{"allowed_emotions": ["neutral", "happy", "curious", "supportive", "playful"], "forbidden_emotions": ["angry"], "max_intensity": 0.8, "mirror_user_emotions": true, "default_emotion": "neutral"}'::jsonb,
ADD COLUMN IF NOT EXISTS sticker_preferences JSONB DEFAULT '{"enabled": true, "probability": 0.3, "preferred_packs": ["default"], "max_per_message": 1}'::jsonb,
ADD COLUMN IF NOT EXISTS response_style JSONB DEFAULT '{"length": "medium", "use_emojis": true, "emoji_frequency": 0.3, "formality": 5, "use_markdown": false}'::jsonb,
ADD COLUMN IF NOT EXISTS proactive_config JSONB DEFAULT '{"enabled": false, "silence_threshold_minutes": 30, "cooldown_minutes": 60, "daily_limit": 5}'::jsonb;

-- Add quota-related columns
ALTER TABLE ai_agents
ADD COLUMN IF NOT EXISTS daily_token_limit INTEGER DEFAULT 100000,
ADD COLUMN IF NOT EXISTS monthly_token_limit INTEGER DEFAULT 3000000;

-- Index for JSON queries
CREATE INDEX IF NOT EXISTS idx_ai_agents_voice_enabled ON ai_agents((voice_settings->>'enabled'));
CREATE INDEX IF NOT EXISTS idx_ai_agents_proactive_enabled ON ai_agents((proactive_config->>'enabled'));

COMMENT ON COLUMN ai_agents.voice_settings IS 'TTS voice configuration (Phase 3)';
COMMENT ON COLUMN ai_agents.emotion_boundaries IS 'Allowed/forbidden emotions and intensity limits (Phase 3)';
COMMENT ON COLUMN ai_agents.sticker_preferences IS 'Sticker usage preferences (Phase 3)';
COMMENT ON COLUMN ai_agents.response_style IS 'Response formatting preferences (Phase 3)';
COMMENT ON COLUMN ai_agents.proactive_config IS 'Proactive messaging configuration (Phase 3)';
