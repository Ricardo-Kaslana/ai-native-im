-- Phase 3: AI Agent Evolution - Learning Patterns Table
-- Stores learned patterns from user feedback for AI improvement

CREATE TABLE IF NOT EXISTS learning_patterns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    
    -- Pattern identification
    context_hash VARCHAR(64) NOT NULL,
    context_embedding VECTOR(1024), -- For semantic similarity search
    
    -- Pattern content
    context_text TEXT NOT NULL,
    successful_response TEXT,
    
    -- Feedback tracking
    feedback_type VARCHAR(20) NOT NULL CHECK (feedback_type IN ('positive', 'negative', 'neutral', 'correction')),
    frequency INTEGER NOT NULL DEFAULT 1,
    
    -- Review status
    needs_review BOOLEAN NOT NULL DEFAULT FALSE,
    reviewed_at TIMESTAMPTZ,
    reviewed_by UUID REFERENCES users(id) ON DELETE SET NULL,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Unique constraint per group and context
    UNIQUE(group_id, context_hash)
);

-- Indexes
CREATE INDEX idx_learning_patterns_group_id ON learning_patterns(group_id);
CREATE INDEX idx_learning_patterns_feedback ON learning_patterns(feedback_type);
CREATE INDEX idx_learning_patterns_needs_review ON learning_patterns(needs_review) WHERE needs_review = TRUE;
CREATE INDEX idx_learning_patterns_frequency ON learning_patterns(frequency DESC);
CREATE INDEX idx_learning_patterns_last_used ON learning_patterns(last_used DESC);

-- Vector similarity index (requires pgvector extension)
-- CREATE INDEX idx_learning_patterns_embedding ON learning_patterns USING ivfflat (context_embedding vector_cosine_ops);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_learning_patterns_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_learning_patterns_updated_at
    BEFORE UPDATE ON learning_patterns
    FOR EACH ROW
    EXECUTE FUNCTION update_learning_patterns_updated_at();

COMMENT ON TABLE learning_patterns IS 'Stores learned patterns from user feedback (Phase 3)';
