-- Migration: Sync existing groups with conversations table
-- This migration creates conversation records for existing groups
-- and adds group members as conversation participants

-- Create conversations for existing groups that don't have one
INSERT INTO conversations (id, conversation_type, name, created_by, created_at, updated_at)
SELECT 
    g.id, 
    'group', 
    g.name, 
    g.owner_id, 
    g.created_at, 
    g.updated_at
FROM groups g
WHERE NOT EXISTS (
    SELECT 1 FROM conversations c WHERE c.id = g.id
);

-- Add group members as conversation participants
INSERT INTO conversation_participants (conversation_id, user_id, joined_at)
SELECT 
    gm.group_id, 
    gm.user_id, 
    gm.joined_at
FROM group_members gm
WHERE NOT EXISTS (
    SELECT 1 FROM conversation_participants cp 
    WHERE cp.conversation_id = gm.group_id 
    AND cp.user_id = gm.user_id
);

