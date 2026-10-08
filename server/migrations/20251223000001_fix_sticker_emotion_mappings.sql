-- Fix sticker_preferences to include emotion_mappings field
-- This field was added to the Rust struct but is missing from the database default

-- Update the default value to include emotion_mappings
ALTER TABLE ai_agents
ALTER COLUMN sticker_preferences SET DEFAULT '{"enabled": true, "probability": 0.3, "preferred_packs": ["default"], "emotion_mappings": [], "max_per_message": 1}'::jsonb;

-- Update existing records to include the emotion_mappings field
UPDATE ai_agents
SET sticker_preferences = sticker_preferences || '{"emotion_mappings": []}'
WHERE sticker_preferences IS NOT NULL
  AND sticker_preferences ? 'emotion_mappings' IS NULL;

-- Alternative: Set to NULL to use Rust Default implementation
-- UPDATE ai_agents SET sticker_preferences = NULL WHERE sticker_preferences IS NOT NULL;

-- Verify
SELECT id, name, sticker_preferences FROM ai_agents WHERE sticker_preferences IS NOT NULL;
