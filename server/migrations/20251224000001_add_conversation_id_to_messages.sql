-- Add conversation_id to messages table for proper client-side message matching
-- This field links messages to the conversations table for efficient querying

-- Add the column (optional, since existing messages don't have it)
ALTER TABLE messages ADD COLUMN IF NOT EXISTS conversation_id UUID REFERENCES conversations(id);

-- Create index for conversation-based queries
CREATE INDEX IF NOT EXISTS idx_messages_conversation_time
    ON messages(conversation_id, timestamp DESC)
    WHERE conversation_id IS NOT NULL;

-- Add comment
COMMENT ON COLUMN messages.conversation_id IS 'Links message to conversations table for proper message matching and retrieval';
