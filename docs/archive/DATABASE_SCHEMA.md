# Database Schema Documentation

This document describes the complete database schema for the AI-Native IM application.

## Overview

The application uses PostgreSQL as the primary database, Redis for caching and session management, and Qdrant for vector storage (AI agent memory).

## PostgreSQL Schema

### Users Table

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(20) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    avatar_url VARCHAR(2048),
    bio TEXT,
    online_status VARCHAR(20) DEFAULT 'offline' CHECK (online_status IN ('online', 'offline', 'away', 'busy')),
    status_message VARCHAR(500),
    account_status VARCHAR(20) DEFAULT 'active' CHECK (account_status IN ('active', 'suspended', 'deleted')),
    role VARCHAR(20) DEFAULT 'user' CHECK (role IN ('user', 'moderator', 'admin')),
    last_seen_at TIMESTAMP WITH TIME ZONE,
    email_verified BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE
);

-- Indexes
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_online_status ON users(online_status);
CREATE INDEX idx_users_account_status ON users(account_status);
CREATE INDEX idx_users_created_at ON users(created_at);
```

### Groups Table

```sql
CREATE TABLE groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    avatar_url VARCHAR(2048),
    is_public BOOLEAN DEFAULT false,
    member_count INTEGER DEFAULT 0,
    max_capacity INTEGER DEFAULT 500,
    status VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'archived')),
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CONSTRAINT groups_name_length CHECK (length(name) >= 1 AND length(name) <= 100)
);

-- Indexes
CREATE INDEX idx_groups_owner_id ON groups(owner_id);
CREATE INDEX idx_groups_is_public ON groups(is_public);
CREATE INDEX idx_groups_status ON groups(status);
CREATE INDEX idx_groups_created_at ON groups(created_at);
CREATE INDEX idx_groups_name_gin ON groups USING gin(to_tsvector('english', name));
```

### Group Members Table

```sql
CREATE TABLE group_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(20) DEFAULT 'member' CHECK (role IN ('admin', 'moderator', 'member')),
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    invited_by UUID REFERENCES users(id),
    muted_until TIMESTAMP WITH TIME ZONE,
    banned_at TIMESTAMP WITH TIME ZONE,
    banned_by UUID REFERENCES users(id),
    ban_reason TEXT,

    UNIQUE(group_id, user_id)
);

-- Indexes
CREATE INDEX idx_group_members_group_id ON group_members(group_id);
CREATE INDEX idx_group_members_user_id ON group_members(user_id);
CREATE INDEX idx_group_members_role ON group_members(role);
CREATE INDEX idx_group_members_joined_at ON group_members(joined_at);
```

### Group Invite Codes Table

```sql
CREATE TABLE group_invite_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    code VARCHAR(32) UNIQUE NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    max_uses INTEGER,
    use_count INTEGER DEFAULT 0,
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CONSTRAINT invite_codes_max_uses_positive CHECK (max_uses IS NULL OR max_uses > 0)
);

-- Indexes
CREATE INDEX idx_group_invite_codes_group_id ON group_invite_codes(group_id);
CREATE INDEX idx_group_invite_codes_code ON group_invite_codes(code);
CREATE INDEX idx_group_invite_codes_expires_at ON group_invite_codes(expires_at);
```

### Conversations Table

```sql
CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100),
    type VARCHAR(20) NOT NULL DEFAULT 'private' CHECK (type IN ('private', 'group')),
    created_by UUID REFERENCES users(id),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_conversations_type ON conversations(type);
CREATE INDEX idx_conversations_created_by ON conversations(created_by);
CREATE INDEX idx_conversations_created_at ON conversations(created_at);
```

### Conversation Participants Table

```sql
CREATE TABLE conversation_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    left_at TIMESTAMP WITH TIME ZONE,
    last_read_at TIMESTAMP WITH TIME ZONE,
    unread_count INTEGER DEFAULT 0,
    is_muted BOOLEAN DEFAULT false,

    UNIQUE(conversation_id, user_id)
);

-- Indexes
CREATE INDEX idx_conversation_participants_conv_id ON conversation_participants(conversation_id);
CREATE INDEX idx_conversation_participants_user_id ON conversation_participants(user_id);
CREATE INDEX idx_conversation_participants_unread_count ON conversation_participants(unread_count);
```

### Messages Table

```sql
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sender_id UUID NOT NULL REFERENCES users(id),
    conversation_id UUID REFERENCES conversations(id),
    group_id UUID REFERENCES groups(id),
    content TEXT,
    msg_type INTEGER NOT NULL DEFAULT 0 CHECK (msg_type IN (0, 1, 2, 3, 4)), -- 0=text, 1=image, 2=video, 3=emoji, 4=system
    media_id UUID REFERENCES media_files(id),
    reply_to_id UUID REFERENCES messages(id),
    mentions UUID[] DEFAULT '{}',
    is_ai_generated BOOLEAN DEFAULT false,
    is_edited BOOLEAN DEFAULT false,
    edited_at TIMESTAMP WITH TIME ZONE,
    is_deleted BOOLEAN DEFAULT false,
    deleted_at TIMESTAMP WITH TIME ZONE,
    delivered BOOLEAN DEFAULT false,
    delivered_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CONSTRAINT messages_has_recipient CHECK (
        (conversation_id IS NOT NULL) OR (group_id IS NOT NULL)
    )
);

-- Indexes
CREATE INDEX idx_messages_sender_id ON messages(sender_id);
CREATE INDEX idx_messages_conversation_id ON messages(conversation_id);
CREATE INDEX idx_messages_group_id ON messages(group_id);
CREATE INDEX idx_messages_reply_to_id ON messages(reply_to_id);
CREATE INDEX idx_messages_created_at ON messages(created_at);
CREATE INDEX idx_messages_msg_type ON messages(msg_type);
CREATE INDEX idx_messages_is_ai_generated ON messages(is_ai_generated);
CREATE INDEX idx_messages_content_gin ON messages USING gin(to_tsvector('english', content));
```

### Message Reactions Table

```sql
CREATE TABLE message_reactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    reaction VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    UNIQUE(message_id, user_id, reaction)
);

-- Indexes
CREATE INDEX idx_message_reactions_message_id ON message_reactions(message_id);
CREATE INDEX idx_message_reactions_user_id ON message_reactions(user_id);
```

### Media Files Table

```sql
CREATE TABLE media_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    filename VARCHAR(255) NOT NULL,
    original_filename VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    content_type VARCHAR(100) NOT NULL,
    storage_provider VARCHAR(50) NOT NULL,
    storage_path VARCHAR(2048) NOT NULL,
    thumbnail_url VARCHAR(2048),
    uploaded_by UUID NOT NULL REFERENCES users(id),
    conversation_id UUID REFERENCES conversations(id),
    group_id UUID REFERENCES groups(id),
    is_public BOOLEAN DEFAULT false,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CONSTRAINT media_files_has_context CHECK (
        (conversation_id IS NOT NULL) OR (group_id IS NOT NULL)
    )
);

-- Indexes
CREATE INDEX idx_media_files_uploaded_by ON media_files(uploaded_by);
CREATE INDEX idx_media_files_conversation_id ON media_files(conversation_id);
CREATE INDEX idx_media_files_group_id ON media_files(group_id);
CREATE INDEX idx_media_files_content_type ON media_files(content_type);
CREATE INDEX idx_media_files_created_at ON media_files(created_at);
```

### AI Agents Table

```sql
CREATE TABLE ai_agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    personality JSONB NOT NULL DEFAULT '{}',
    emotion_state JSONB DEFAULT '{}',
    knowledge_base JSONB DEFAULT '{}',
    interaction_history JSONB DEFAULT '{}',
    memory_points JSONB DEFAULT '{}',
    learning_patterns JSONB DEFAULT '{}',
    toxicity_level INTEGER DEFAULT 0 CHECK (toxicity_level >= 0 AND toxicity_level <= 10),
    enthusiasm_level INTEGER DEFAULT 5 CHECK (enthusiasm_level >= 0 AND enthusiasm_level <= 10),
    expertise_domains TEXT[] DEFAULT '{}',
    speech_suffix VARCHAR(500),
    trigger_keywords TEXT[] DEFAULT '{}',
    custom_prompt TEXT,
    backstory TEXT,
    avatar_url VARCHAR(2048),
    response_style VARCHAR(50) DEFAULT 'friendly',
    proactivity_level INTEGER DEFAULT 5 CHECK (proactivity_level >= 0 AND proactivity_level <= 10),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_ai_agents_group_id ON ai_agents(group_id);
CREATE INDEX idx_ai_agents_name ON ai_agents(name);
CREATE INDEX idx_ai_agents_expertise_domains ON ai_agents USING gin(expertise_domains);
CREATE INDEX idx_ai_agents_trigger_keywords ON ai_agents USING gin(trigger_keywords);
```

### AI Agent User Profiles Table

```sql
CREATE TABLE ai_agent_user_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES ai_agents(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    nickname VARCHAR(100),
    relationship_type VARCHAR(50),
    interaction_count INTEGER DEFAULT 0,
    last_interaction_at TIMESTAMP WITH TIME ZONE,
    affinity_score DECIMAL(3,2) DEFAULT 0.0 CHECK (affinity_score >= -1.0 AND affinity_score <= 1.0),
    communication_preferences JSONB DEFAULT '{}',
    relationship_notes TEXT,
    shared_interests TEXT[] DEFAULT '{}',
    conflict_history JSONB DEFAULT '{}',

    UNIQUE(agent_id, user_id)
);

-- Indexes
CREATE INDEX idx_ai_agent_user_profiles_agent_id ON ai_agent_user_profiles(agent_id);
CREATE INDEX idx_ai_agent_user_profiles_user_id ON ai_agent_user_profiles(user_id);
CREATE INDEX idx_ai_agent_user_profiles_affinity ON ai_agent_user_profiles(affinity_score);
```

### AI Message Feedback Table

```sql
CREATE TABLE ai_message_feedback (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_id UUID REFERENCES ai_agents(id),
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    helpful BOOLEAN,
    feedback_type VARCHAR(50),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    UNIQUE(message_id, user_id)
);

-- Indexes
CREATE INDEX idx_ai_message_feedback_message_id ON ai_message_feedback(message_id);
CREATE INDEX idx_ai_message_feedback_agent_id ON ai_message_feedback(agent_id);
CREATE INDEX idx_ai_message_feedback_rating ON ai_message_feedback(rating);
CREATE INDEX idx_ai_message_feedback_created_at ON ai_message_feedback(created_at);
```

### Contacts Table

```sql
CREATE TABLE contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    contact_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    nickname VARCHAR(100),
    added_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    UNIQUE(user_id, contact_user_id),
    CONSTRAINT contacts_no_self_reference CHECK (user_id != contact_user_id)
);

-- Indexes
CREATE INDEX idx_contacts_user_id ON contacts(user_id);
CREATE INDEX idx_contacts_contact_user_id ON contacts(contact_user_id);
```

### Sessions Table (JWT Refresh Tokens)

```sql
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token VARCHAR(255) UNIQUE NOT NULL,
    device_info JSONB DEFAULT '{}',
    ip_address INET,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    last_used_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_refresh_token ON sessions(refresh_token);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
```

### Password Reset Tokens Table

```sql
CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(255) UNIQUE NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    used_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);
CREATE INDEX idx_password_reset_tokens_token ON password_reset_tokens(token);
CREATE INDEX idx_password_reset_tokens_expires_at ON password_reset_tokens(expires_at);
```

### Audit Logs Table

```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50),
    resource_id UUID,
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
```

### Rate Limits Table

```sql
CREATE TABLE rate_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    identifier VARCHAR(255) NOT NULL, -- IP address, user ID, etc.
    action VARCHAR(100) NOT NULL,
    count INTEGER NOT NULL DEFAULT 1,
    window_start TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    window_end TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    UNIQUE(identifier, action)
);

-- Indexes
CREATE INDEX idx_rate_limits_identifier ON rate_limits(identifier);
CREATE INDEX idx_rate_limits_action ON rate_limits(action);
CREATE INDEX idx_rate_limits_window_end ON rate_limits(window_end);
```

## Redis Schema

### Key Naming Conventions
```
session:{session_id}           - User session data
user:online:{user_id}          - User online status with TTL
presence:subscriptions:{user_id} - Users this user is subscribed to
message:queue:{user_id}        - Undelivered messages queue
typing:{conversation_id}       - Typing indicators for conversations
ai:agent:{agent_id}:state      - AI agent current state
ai:agent:{agent_id}:memory     - AI agent short-term memory
rate_limit:{identifier}:{action} - Rate limiting counters
cache:api:{endpoint}:{params}  - API response cache
lock:{resource_id}             - Distributed locks
```

### Data Structures

#### User Online Status
```
Key: user:online:{user_id}
Type: Hash
TTL: 300 seconds
Fields:
  - status: "online"|"offline"|"away"|"busy"
  - last_seen: Unix timestamp
  - status_message: Optional text
```

#### Typing Indicators
```
Key: typing:{conversation_id}
Type: Set
TTL: 10 seconds per member
Members: user_ids currently typing
```

#### AI Agent State
```
Key: ai:agent:{agent_id}:state
Type: Hash
TTL: 3600 seconds
Fields:
  - current_emotion: Current emotion state
  - last_interaction: Timestamp of last interaction
  - conversation_context: Active conversation context
  - pending_actions: JSON array of pending actions
```

#### Message Queue
```
Key: message:queue:{user_id}
Type: List
Description: Queue of undelivered messages for offline users
Elements: JSON message objects
```

## Qdrant Vector Database

### Collections

#### AI Agent Memory
```
Collection: agent_memory_{agent_id}
Vector Size: 1024 (for SiliconFlow compatibility)

Points Structure:
{
  "id": "uuid",
  "vector": [1024-dimensional embedding],
  "payload": {
    "type": "conversation"|"fact"|"preference"|"emotion",
    "content": "Original text/content",
    "timestamp": ISO datetime,
    "context": {
      "conversation_id": "uuid",
      "user_id": "uuid",
      "importance": 0.0-1.0,
      "emotion": "emotion_tag",
      "keywords": ["keyword1", "keyword2"]
    },
    "metadata": {
      "source": "user_message"|"ai_response"|"observation",
      "decay_factor": 0.0-1.0,
      "access_count": integer
    }
  }
}
```

#### User Interaction Patterns
```
Collection: user_patterns_{user_id}
Vector Size: 512

Points Structure:
{
  "id": "uuid",
  "vector": [512-dimensional embedding],
  "payload": {
    "pattern_type": "communication_style"|"topic_preference"|"interaction_timing",
    "features": {
      "response_time": milliseconds,
      "message_length": average,
      "emoji_usage": frequency,
      "active_hours": [array],
      "preferred_topics": ["topic1", "topic2"]
    },
    "last_updated": ISO datetime,
    "confidence": 0.0-1.0
  }
}
```

## Database Migrations

### Migration Files
Migrations are stored in `server/migrations/` directory with naming convention:
```
YYYYMMDDHHMMSS_description.sql
```

### Example Migration
```sql
-- Migration: 20250101000000_add_ai_agents.sql
BEGIN;

CREATE TABLE ai_agents (
    -- Table definition as above
);

-- Add indexes
CREATE INDEX idx_ai_agents_group_id ON ai_agents(group_id);

-- Insert default data
INSERT INTO ai_agents (group_id, name, personality)
SELECT id, 'Default Assistant', '{}' FROM groups;

COMMIT;
```

## Performance Optimizations

### Indexing Strategy
1. **Primary Keys**: All tables use UUID primary keys
2. **Foreign Keys**: All foreign key columns are indexed
3. **Query Patterns**: Composite indexes for common query patterns
4. **Full-text Search**: GIN indexes for text search
5. **JSONB**: GIN indexes for JSONB queries

### Partitioning
Consider partitioning large tables:
- Messages by month (based on created_at)
- Audit logs by month
- Message reactions by message_id range

### Caching Strategy
1. **Redis**: Hot data with TTL
2. **Materialized Views**: For complex aggregations
3. **Query Caching**: API responses with short TTL
4. **Connection Pooling**: PgBouncer for high concurrency

## Backup and Recovery

### Backup Strategy
```bash
# Full backup
pg_dump -h localhost -U postgres -d ai_native_im > backup_full_$(date +%Y%m%d).sql

# Schema only
pg_dump -s -h localhost -U postgres -d ai_native_im > backup_schema_$(date +%Y%m%d).sql

# Data only
pg_dump -a -h localhost -U postgres -d ai_native_im > backup_data_$(date +%Y%m%d).sql
```

### Point-in-Time Recovery
- Enable WAL logging in PostgreSQL
- Configure archive retention (e.g., 30 days)
- Regularly test recovery procedures

## Security Considerations

### Data Encryption
- Enable encryption at rest (TDE)
- Use TLS for database connections
- Encrypt sensitive columns (password_hash, tokens)

### Access Control
- Separate read/write users
- Row-level security for multi-tenant data
- Database audit logging

### PII Handling
- Mask email addresses in logs
- Implement data retention policies
- Right to deletion compliance