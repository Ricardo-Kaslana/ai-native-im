-- Phase 4: User Profile System
-- Stores user traits and relationship scores for AI agent personalization

-- User profiles table - stores personality traits and preferences learned by AI
CREATE TABLE IF NOT EXISTS agent_user_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES ai_agents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Relationship scores (-10 to 10)
    intimacy_score SMALLINT NOT NULL DEFAULT 0 CHECK (intimacy_score BETWEEN -10 AND 10),
    respect_score SMALLINT NOT NULL DEFAULT 0 CHECK (respect_score BETWEEN -10 AND 10),
    trust_score SMALLINT NOT NULL DEFAULT 0 CHECK (trust_score BETWEEN -10 AND 10),
    
    -- Personality traits (JSONB for flexibility)
    traits JSONB DEFAULT '[]'::jsonb,
    
    -- Communication preferences
    preferred_tone VARCHAR(50) DEFAULT 'neutral', -- friendly, formal, casual, playful
    preferred_response_length VARCHAR(20) DEFAULT 'medium', -- short, medium, long
    
    -- Interests and topics
    interests JSONB DEFAULT '[]'::jsonb,
    sensitive_topics JSONB DEFAULT '[]'::jsonb,
    
    -- Interaction history summary
    total_interactions INTEGER NOT NULL DEFAULT 0,
    positive_interactions INTEGER NOT NULL DEFAULT 0,
    negative_interactions INTEGER NOT NULL DEFAULT 0,
    last_interaction_at TIMESTAMPTZ,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Unique constraint
    UNIQUE(agent_id, user_id)
);

-- Create trigger for updated_at
CREATE TRIGGER update_agent_user_profiles_updated_at
    BEFORE UPDATE ON agent_user_profiles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Indexes
CREATE INDEX idx_agent_user_profiles_agent ON agent_user_profiles(agent_id);
CREATE INDEX idx_agent_user_profiles_user ON agent_user_profiles(user_id);
CREATE INDEX idx_agent_user_profiles_intimacy ON agent_user_profiles(intimacy_score);
CREATE INDEX idx_agent_user_profiles_last_interaction ON agent_user_profiles(last_interaction_at DESC);

-- User interaction events for profile learning
CREATE TABLE IF NOT EXISTS user_interaction_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES ai_agents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Event details
    event_type VARCHAR(50) NOT NULL, -- message, reaction, mention, feedback
    sentiment VARCHAR(20), -- positive, negative, neutral
    context_summary TEXT,
    
    -- Learned information
    detected_traits JSONB DEFAULT '[]'::jsonb,
    detected_interests JSONB DEFAULT '[]'::jsonb,
    
    -- Reference
    message_id UUID,
    
    -- Timestamp
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for event queries
CREATE INDEX idx_user_interaction_events_agent_user ON user_interaction_events(agent_id, user_id);
CREATE INDEX idx_user_interaction_events_created ON user_interaction_events(created_at DESC);

-- Comments
COMMENT ON TABLE agent_user_profiles IS 'Stores user personality profiles learned by AI agents';
COMMENT ON COLUMN agent_user_profiles.intimacy_score IS 'How close the relationship is (-10 to 10)';
COMMENT ON COLUMN agent_user_profiles.respect_score IS 'Mutual respect level (-10 to 10)';
COMMENT ON COLUMN agent_user_profiles.trust_score IS 'Trust level (-10 to 10)';
COMMENT ON COLUMN agent_user_profiles.traits IS 'Learned personality traits (e.g., ["humorous", "technical", "patient"])';
COMMENT ON COLUMN agent_user_profiles.interests IS 'Detected user interests (e.g., ["programming", "gaming", "music"])';
COMMENT ON COLUMN agent_user_profiles.sensitive_topics IS 'Topics to be careful about';
COMMENT ON TABLE user_interaction_events IS 'Raw interaction events for profile learning';
