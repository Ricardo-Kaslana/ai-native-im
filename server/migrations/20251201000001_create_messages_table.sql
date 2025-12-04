-- Create messages table for message persistence
CREATE TABLE messages (
    id UUID PRIMARY KEY,
    sender_id UUID NOT NULL REFERENCES users(id),
    receiver_id UUID REFERENCES users(id),  -- NULL for group messages
    group_id UUID,                           -- NULL for private messages
    msg_type SMALLINT NOT NULL DEFAULT 0,    -- 0=TEXT, 1=IMAGE, 2=VIDEO, 3=EMOJI
    content TEXT NOT NULL,
    timestamp BIGINT NOT NULL,
    is_ai_generated BOOLEAN DEFAULT FALSE,
    delivered BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    -- Ensure message is either private or group, not both
    CONSTRAINT chk_message_target CHECK (
        (receiver_id IS NOT NULL AND group_id IS NULL) OR
        (receiver_id IS NULL AND group_id IS NOT NULL)
    )
);

-- Index for offline message retrieval (undelivered messages for a user)
CREATE INDEX idx_messages_receiver_undelivered 
    ON messages(receiver_id, timestamp DESC) 
    WHERE delivered = FALSE AND receiver_id IS NOT NULL;

-- Index for group message queries
CREATE INDEX idx_messages_group_time 
    ON messages(group_id, timestamp DESC) 
    WHERE group_id IS NOT NULL;

-- Index for sender message history
CREATE INDEX idx_messages_sender_time 
    ON messages(sender_id, timestamp DESC);

-- Index for message lookup by ID (for deduplication)
CREATE INDEX idx_messages_id_lookup 
    ON messages(id);

-- Index for timestamp-based queries
CREATE INDEX idx_messages_timestamp 
    ON messages(timestamp DESC);

COMMENT ON TABLE messages IS 'Stores all chat messages for persistence and offline delivery';
COMMENT ON COLUMN messages.msg_type IS '0=TEXT, 1=IMAGE, 2=VIDEO, 3=EMOJI';
COMMENT ON COLUMN messages.delivered IS 'Whether the message has been delivered to the recipient';
