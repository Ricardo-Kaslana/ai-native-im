-- Phase 3: AI Agent Evolution - Emotion History Table
-- Tracks emotion analysis history for AI agents

CREATE TABLE IF NOT EXISTS agent_emotion_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES ai_agents(id) ON DELETE CASCADE,
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    message_id UUID REFERENCES messages(id) ON DELETE SET NULL,
    
    -- Emotion analysis results
    detected_emotion VARCHAR(50) NOT NULL,
    confidence REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    sentiment_score REAL CHECK (sentiment_score >= -1.0 AND sentiment_score <= 1.0),
    
    -- Response emotion
    response_emotion VARCHAR(50),
    emotion_intensity REAL CHECK (emotion_intensity >= 0.0 AND emotion_intensity <= 1.0),
    
    -- Context
    user_message_preview TEXT,
    agent_response_preview TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX idx_emotion_history_agent_id ON agent_emotion_history(agent_id);
CREATE INDEX idx_emotion_history_group_id ON agent_emotion_history(group_id);
CREATE INDEX idx_emotion_history_created_at ON agent_emotion_history(created_at DESC);
CREATE INDEX idx_emotion_history_detected ON agent_emotion_history(detected_emotion);

-- Comment
COMMENT ON TABLE agent_emotion_history IS 'Tracks emotion analysis history for AI agents (Phase 3)';
