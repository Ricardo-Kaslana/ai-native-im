-- Phase 3: AI Agent Evolution - Provider Usage Table
-- Tracks AI service provider usage for quota management and cost tracking

CREATE TABLE IF NOT EXISTS provider_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- User/Group context
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    group_id UUID REFERENCES groups(id) ON DELETE SET NULL,
    agent_id UUID REFERENCES ai_agents(id) ON DELETE SET NULL,
    
    -- Provider info
    provider VARCHAR(50) NOT NULL, -- siliconflow, zhipu, moonshot, openai, etc.
    service_type VARCHAR(30) NOT NULL, -- llm, embedding, stt, tts, vision, image_gen
    model VARCHAR(100),
    
    -- Usage metrics
    tokens_used INTEGER DEFAULT 0,
    input_tokens INTEGER DEFAULT 0,
    output_tokens INTEGER DEFAULT 0,
    audio_seconds REAL DEFAULT 0,
    image_count INTEGER DEFAULT 0,
    
    -- Cost tracking
    cost_estimate REAL DEFAULT 0, -- Estimated cost in USD
    
    -- Request info
    request_id VARCHAR(100),
    latency_ms INTEGER,
    success BOOLEAN NOT NULL DEFAULT TRUE,
    error_message TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Partition by date for efficient querying
    usage_date DATE NOT NULL DEFAULT CURRENT_DATE
);

-- Indexes for common queries
CREATE INDEX idx_provider_usage_user_id ON provider_usage(user_id);
CREATE INDEX idx_provider_usage_group_id ON provider_usage(group_id);
CREATE INDEX idx_provider_usage_provider ON provider_usage(provider);
CREATE INDEX idx_provider_usage_service ON provider_usage(service_type);
CREATE INDEX idx_provider_usage_date ON provider_usage(usage_date);
CREATE INDEX idx_provider_usage_created ON provider_usage(created_at DESC);

-- Composite index for daily user quota checks
CREATE INDEX idx_provider_usage_user_daily ON provider_usage(user_id, service_type, usage_date);

-- Composite index for monthly aggregation
CREATE INDEX idx_provider_usage_monthly ON provider_usage(user_id, service_type, created_at);

-- View for daily usage summary
CREATE OR REPLACE VIEW daily_usage_summary AS
SELECT 
    user_id,
    usage_date,
    service_type,
    provider,
    COUNT(*) as request_count,
    SUM(tokens_used) as total_tokens,
    SUM(audio_seconds) as total_audio_seconds,
    SUM(cost_estimate) as total_cost,
    AVG(latency_ms) as avg_latency_ms,
    SUM(CASE WHEN success THEN 1 ELSE 0 END)::REAL / COUNT(*) as success_rate
FROM provider_usage
GROUP BY user_id, usage_date, service_type, provider;

COMMENT ON TABLE provider_usage IS 'Tracks AI service provider usage for quota and cost management (Phase 3)';
