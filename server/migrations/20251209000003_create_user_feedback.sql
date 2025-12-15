-- Phase 3: AI Agent Evolution - User Feedback Table
-- Records user feedback on AI agent responses

CREATE TABLE IF NOT EXISTS user_feedback (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- References
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL REFERENCES ai_agents(id) ON DELETE CASCADE,
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    response_message_id UUID REFERENCES messages(id) ON DELETE SET NULL,
    
    -- Feedback details
    feedback_type VARCHAR(20) NOT NULL CHECK (feedback_type IN ('positive', 'negative', 'neutral', 'correction')),
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    correction_text TEXT, -- User-provided correction for the response
    
    -- Context for learning
    original_message TEXT,
    agent_response TEXT,
    
    -- Metadata
    is_processed BOOLEAN NOT NULL DEFAULT FALSE,
    processed_at TIMESTAMPTZ,
    pattern_id UUID REFERENCES learning_patterns(id) ON DELETE SET NULL,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_user_feedback_user_id ON user_feedback(user_id);
CREATE INDEX idx_user_feedback_agent_id ON user_feedback(agent_id);
CREATE INDEX idx_user_feedback_group_id ON user_feedback(group_id);
CREATE INDEX idx_user_feedback_message_id ON user_feedback(message_id);
CREATE INDEX idx_user_feedback_type ON user_feedback(feedback_type);
CREATE INDEX idx_user_feedback_unprocessed ON user_feedback(is_processed) WHERE is_processed = FALSE;
CREATE INDEX idx_user_feedback_created_at ON user_feedback(created_at DESC);

COMMENT ON TABLE user_feedback IS 'Records user feedback on AI agent responses (Phase 3)';
