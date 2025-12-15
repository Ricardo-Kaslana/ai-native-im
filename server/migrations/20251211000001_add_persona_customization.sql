-- Phase 4: User Customizable Persona System
-- Adds custom prompt, backstory, avatar, greeting/farewell messages

-- Add user customization fields to ai_agents table
ALTER TABLE ai_agents 
ADD COLUMN IF NOT EXISTS custom_prompt TEXT,
ADD COLUMN IF NOT EXISTS backstory TEXT,
ADD COLUMN IF NOT EXISTS avatar_url VARCHAR(500),
ADD COLUMN IF NOT EXISTS greeting_message TEXT,
ADD COLUMN IF NOT EXISTS farewell_message TEXT,
ADD COLUMN IF NOT EXISTS persona_preset VARCHAR(50) DEFAULT 'custom';

-- Add constraints
ALTER TABLE ai_agents
ADD CONSTRAINT chk_backstory_length CHECK (backstory IS NULL OR LENGTH(backstory) <= 2000),
ADD CONSTRAINT chk_custom_prompt_length CHECK (custom_prompt IS NULL OR LENGTH(custom_prompt) <= 5000),
ADD CONSTRAINT chk_greeting_length CHECK (greeting_message IS NULL OR LENGTH(greeting_message) <= 500),
ADD CONSTRAINT chk_farewell_length CHECK (farewell_message IS NULL OR LENGTH(farewell_message) <= 500);

-- Add index for preset queries
CREATE INDEX IF NOT EXISTS idx_ai_agents_preset ON ai_agents(persona_preset);

-- Add comments
COMMENT ON COLUMN ai_agents.custom_prompt IS 'User-defined system prompt override (max 5000 chars)';
COMMENT ON COLUMN ai_agents.backstory IS 'Agent backstory/background for personality context (max 2000 chars)';
COMMENT ON COLUMN ai_agents.avatar_url IS 'Custom avatar URL for the agent';
COMMENT ON COLUMN ai_agents.greeting_message IS 'Message sent when agent is first activated or user joins';
COMMENT ON COLUMN ai_agents.farewell_message IS 'Message sent when agent is deactivated';
COMMENT ON COLUMN ai_agents.persona_preset IS 'Preset template name: friendly_assistant, sarcastic_friend, professional_expert, cute_mascot, custom';
