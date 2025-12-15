-- Phase 3: AI Agent Evolution - Summary Cache Table
-- Caches generated message summaries for performance

CREATE TABLE IF NOT EXISTS summary_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    
    -- Cache key components
    message_count INTEGER NOT NULL,
    style VARCHAR(50) NOT NULL DEFAULT 'concise',
    start_message_id UUID REFERENCES messages(id) ON DELETE CASCADE,
    end_message_id UUID REFERENCES messages(id) ON DELETE CASCADE,
    
    -- Summary content
    summary_text TEXT NOT NULL,
    key_points JSONB, -- Array of key points
    participants JSONB, -- Array of participant info
    topics JSONB, -- Array of discussed topics
    
    -- Metadata
    token_count INTEGER,
    generation_time_ms INTEGER,
    llm_model VARCHAR(100),
    
    -- Cache control
    cache_key VARCHAR(128) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Unique cache key per group
    UNIQUE(group_id, cache_key)
);

-- Indexes
CREATE INDEX idx_summary_cache_group_id ON summary_cache(group_id);
CREATE INDEX idx_summary_cache_expires ON summary_cache(expires_at);
CREATE INDEX idx_summary_cache_key ON summary_cache(cache_key);

-- Auto-cleanup of expired cache entries
CREATE OR REPLACE FUNCTION cleanup_expired_summaries()
RETURNS void AS $$
BEGIN
    DELETE FROM summary_cache WHERE expires_at < NOW();
END;
$$ LANGUAGE plpgsql;

COMMENT ON TABLE summary_cache IS 'Caches generated message summaries (Phase 3)';
