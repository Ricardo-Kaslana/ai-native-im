-- AI Agent configuration table
CREATE TABLE IF NOT EXISTS ai_agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    toxicity_level SMALLINT NOT NULL DEFAULT 5 CHECK (toxicity_level BETWEEN 0 AND 10),
    enthusiasm_level SMALLINT NOT NULL DEFAULT 5 CHECK (enthusiasm_level BETWEEN 0 AND 10),
    expertise_domains TEXT[] NOT NULL DEFAULT '{}',
    speech_suffix VARCHAR(10),
    trigger_keywords TEXT[] NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(group_id)  -- One agent per group
);

-- Create trigger for ai_agents updated_at
CREATE TRIGGER update_ai_agents_updated_at
    BEFORE UPDATE ON ai_agents
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Agent user affinities (how the agent feels about specific users)
CREATE TABLE IF NOT EXISTS agent_user_affinities (
    agent_id UUID NOT NULL REFERENCES ai_agents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    affinity_score SMALLINT NOT NULL DEFAULT 0 CHECK (affinity_score BETWEEN -10 AND 10),
    PRIMARY KEY (agent_id, user_id)
);

-- Trigger evaluation logs for statistics
CREATE TABLE IF NOT EXISTS agent_trigger_logs (
    id BIGSERIAL PRIMARY KEY,
    agent_id UUID NOT NULL REFERENCES ai_agents(id) ON DELETE CASCADE,
    group_id UUID NOT NULL,
    message_id UUID NOT NULL,
    trigger_type VARCHAR(50) NOT NULL,
    did_respond BOOLEAN NOT NULL,
    probability REAL NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_trigger_logs_agent_time ON agent_trigger_logs(agent_id, created_at DESC);
CREATE INDEX idx_trigger_logs_group ON agent_trigger_logs(group_id, created_at DESC);

-- Message statistics per agent
CREATE TABLE IF NOT EXISTS agent_message_stats (
    agent_id UUID PRIMARY KEY REFERENCES ai_agents(id) ON DELETE CASCADE,
    total_messages BIGINT NOT NULL DEFAULT 0,
    last_message_at TIMESTAMPTZ,
    avg_response_time_ms INTEGER
);

-- Add comments
COMMENT ON TABLE ai_agents IS 'AI Agent configurations for group chats';
COMMENT ON TABLE agent_user_affinities IS 'User-specific affinity scores for AI agents';
COMMENT ON TABLE agent_trigger_logs IS 'Logs of trigger evaluations for statistics';
COMMENT ON TABLE agent_message_stats IS 'Aggregated message statistics per agent';
