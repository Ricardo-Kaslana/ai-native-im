# WebSocket API Guide

This guide covers the WebSocket implementations for real-time features in the AI-Native IM application.

## Overview

The application uses two separate WebSocket connections for different purposes:

1. **Presence WebSocket** (`/ws`) - For user online/offline status and presence management
2. **Message WebSocket** (`/ws?token=<jwt>`) - For real-time messaging, typing indicators, and AI agent interactions

## Connection Management

### Connecting to Presence WebSocket

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');
```

### Connecting to Message WebSocket

```javascript
const token = getJWTToken(); // Your JWT token
const ws = new WebSocket(`ws://localhost:8080/ws?token=${token}`);
```

### Connection Lifecycle

1. **Connection Open**: Send initial subscription/presence updates
2. **Heartbeat**: Send ping every 30 seconds to maintain connection
3. **Reconnection**: Implement exponential backoff for reconnection attempts
4. **Graceful Close**: Send close frame before terminating

## Presence WebSocket Protocol

### Message Types

#### Ping/Pong
```json
// Client -> Server
{
  "type": "Ping",
  "timestamp": 1234567890
}

// Server -> Client
{
  "type": "Pong",
  "timestamp": 1234567890
}
```

#### Subscribe to User Presence
```json
// Client -> Server
{
  "type": "Subscribe",
  "user_ids": ["uuid1", "uuid2", "uuid3"]
}

// Server -> Client (confirmation)
{
  "type": "Subscribed",
  "user_ids": ["uuid1", "uuid2", "uuid3"]
}
```

#### Presence Updates
```json
// Server -> Client (when subscribed user status changes)
{
  "type": "PresenceUpdate",
  "data": {
    "user_id": "uuid",
    "online": true,
    "last_seen": "2025-01-01T12:00:00Z",
    "status": "online" // online|offline|away|busy
  }
}
```

#### Initial Status Broadcast
```json
// Client -> Server (when connecting)
{
  "type": "StatusUpdate",
  "data": {
    "status": "online",
    "status_message": "Working on project"
  }
}
```

### Complete Example

```javascript
class PresenceManager {
  constructor(userId) {
    this.userId = userId;
    this.ws = new WebSocket('ws://localhost:8080/ws');
    this.subscribedUsers = new Set();
    this.setupWebSocket();
  }

  setupWebSocket() {
    this.ws.onopen = () => {
      console.log('Presence WebSocket connected');
      // Start heartbeat
      this.startHeartbeat();
    };

    this.ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      this.handleMessage(message);
    };

    this.ws.onclose = () => {
      console.log('Presence WebSocket disconnected');
      // Implement reconnection logic
      this.reconnect();
    };
  }

  subscribeToUsers(userIds) {
    this.ws.send(JSON.stringify({
      type: 'Subscribe',
      user_ids: userIds
    }));
    userIds.forEach(id => this.subscribedUsers.add(id));
  }

  updateStatus(status, message = '') {
    this.ws.send(JSON.stringify({
      type: 'StatusUpdate',
      data: {
        status: status,
        status_message: message
      }
    }));
  }

  startHeartbeat() {
    this.heartbeatInterval = setInterval(() => {
      this.ws.send(JSON.stringify({
        type: 'Ping',
        timestamp: Date.now()
      }));
    }, 30000);
  }

  handleMessage(message) {
    switch (message.type) {
      case 'Pong':
        // Handle pong
        break;
      case 'PresenceUpdate':
        this.onPresenceUpdate(message.data);
        break;
      case 'Subscribed':
        console.log('Subscribed to users:', message.user_ids);
        break;
    }
  }

  onPresenceUpdate(data) {
    // Update UI with user presence
    console.log(`User ${data.user_id} is ${data.online ? 'online' : 'offline'}`);
  }

  reconnect() {
    setTimeout(() => {
      this.ws = new WebSocket('ws://localhost:8080/ws');
      this.setupWebSocket();
    }, 1000);
  }
}
```

## Message WebSocket Protocol

### Authentication

The Message WebSocket requires a valid JWT token as a query parameter:
```
ws://localhost:8080/ws?token=<your_jwt_token>
```

### Message Types

#### 1. Chat Messages

##### Sending a Message
```json
// Client -> Server
{
  "type": "message",
  "data": {
    "conversation_id": "uuid", // For private chats
    "group_id": "uuid",        // For group chats
    "content": "Hello, world!",
    "msg_type": 0,            // 0=text, 1=image, 2=video, 3=emoji
    "media_id": "uuid",       // For media messages
    "reply_to": "uuid",       // For reply messages
    "mentions": ["uuid1"],    // User mentions
    "metadata": {}            // Additional metadata
  }
}
```

##### Receiving a Message
```json
// Server -> Client
{
  "type": "message",
  "data": {
    "id": "uuid",
    "sender_id": "uuid",
    "sender_name": "John Doe",
    "sender_avatar": "url",
    "conversation_id": "uuid",
    "group_id": "uuid",
    "content": "Hello, world!",
    "msg_type": 0,
    "media_id": "uuid",
    "media_url": "url",
    "reply_to": {
      "id": "uuid",
      "content": "Original message",
      "sender_name": "Jane Doe"
    },
    "mentions": ["uuid1"],
    "is_ai_generated": false,
    "timestamp": "2025-01-01T12:00:00Z",
    "delivered": true,
    "read": false
  }
}
```

#### 2. AI Agent Messages

##### Proactive AI Message
```json
// Server -> Client
{
  "type": "ai_message",
  "data": {
    "agent_id": "uuid",
    "agent_name": "Assistant",
    "group_id": "uuid",
    "content": "How is everyone doing today?",
    "thinking_process": "I noticed the chat has been quiet for a while and want to engage the group",
    "confidence": 0.95,
    "is_proactive": true,
    "emotion": "friendly",
    "trigger_type": "periodic_check",
    "timestamp": "2025-01-01T12:00:00Z"
  }
}
```

##### AI Response to User
```json
// Server -> Client
{
  "type": "ai_message",
  "data": {
    "agent_id": "uuid",
    "agent_name": "Assistant",
    "group_id": "uuid",
    "content": "That's an interesting question! Let me help...",
    "thinking_process": "User is asking about X, I should provide information about Y and Z",
    "confidence": 0.88,
    "is_proactive": false,
    "emotion": "helpful",
    "response_to": "uuid", // Message ID this responds to
    "timestamp": "2025-01-01T12:00:00Z"
  }
}
```

#### 3. Typing Indicators

##### Send Typing Status
```json
// Client -> Server
{
  "type": "typing",
  "data": {
    "conversation_id": "uuid",
    "is_typing": true
  }
}
```

##### Receive Typing Status
```json
// Server -> Client
{
  "type": "typing",
  "data": {
    "user_id": "uuid",
    "user_name": "John Doe",
    "conversation_id": "uuid",
    "is_typing": true,
    "timestamp": "2025-01-01T12:00:00Z"
  }
}
```

#### 4. Message Synchronization

##### Request Sync
```json
// Client -> Server
{
  "type": "sync_request",
  "data": {
    "conversation_id": "uuid",
    "since": "2025-01-01T10:00:00Z", // ISO datetime
    "limit": 50 // Optional, default 100
  }
}
```

##### Sync Response
```json
// Server -> Client
{
  "type": "sync_response",
  "data": {
    "messages": [
      // Message objects...
    ],
    "has_more": true,
    "next_cursor": "cursor_string"
  }
}
```

#### 5. Message Status Updates

##### Mark as Read
```json
// Client -> Server
{
  "type": "mark_as_read",
  "data": {
    "message_id": "uuid",
    "conversation_id": "uuid"
  }
}
```

##### Message Acknowledgment
```json
// Server -> Client
{
  "type": "message_ack",
  "data": {
    "message_id": "uuid",
    "status": "delivered" // delivered|read
  }
}
```

#### 6. Media Upload

##### Start Upload
```json
// Client -> Server
{
  "type": "upload_start",
  "data": {
    "filename": "photo.jpg",
    "file_size": 1024000,
    "content_type": "image/jpeg",
    "conversation_id": "uuid"
  }
}
```

##### Upload Response
```json
// Server -> Client
{
  "type": "upload_response",
  "data": {
    "upload_id": "uuid",
    "upload_url": "https://storage.url/upload/...",
    "expires_at": "2025-01-01T12:05:00Z"
  }
}
```

#### 7. Heartbeat

```json
// Client -> Server (every 30 seconds)
{
  "type": "heartbeat"
}

// Server -> Client (response)
{
  "type": "heartbeat_ack"
}
```

#### 8. System Notifications

```json
// Server -> Client
{
  "type": "system_notification",
  "data": {
    "title": "User Joined",
    "message": "John Doe joined the group",
    "level": "info", // info|warning|error
    "action_url": "optional_url",
    "timestamp": "2025-01-01T12:00:00Z"
  }
}
```

#### 9. Group Updates

```json
// Server -> Client
{
  "type": "group_update",
  "data": {
    "group_id": "uuid",
    "update_type": "member_joined", // member_joined|member_left|name_changed|etc.
    "data": {
      "user_id": "uuid",
      "user_name": "John Doe"
    },
    "timestamp": "2025-01-01T12:00:00Z"
  }
}
```

## Complete WebSocket Client Implementation

```javascript
class ChatWebSocket {
  constructor(token) {
    this.token = token;
    this.ws = null;
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 10;
    this.reconnectDelay = 1000;
    this.heartbeatInterval = null;
    this.messageQueue = [];
    this.connect();
  }

  connect() {
    const wsUrl = `ws://localhost:8080/ws?token=${this.token}`;
    this.ws = new WebSocket(wsUrl);
    this.setupEventHandlers();
  }

  setupEventHandlers() {
    this.ws.onopen = () => {
      console.log('WebSocket connected');
      this.reconnectAttempts = 0;
      this.startHeartbeat();

      // Send queued messages
      while (this.messageQueue.length > 0) {
        this.send(this.messageQueue.shift());
      }
    };

    this.ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      this.handleMessage(message);
    };

    this.ws.onclose = () => {
      console.log('WebSocket disconnected');
      this.stopHeartbeat();
      this.handleReconnect();
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };
  }

  send(message) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    } else {
      // Queue message if not connected
      this.messageQueue.push(message);
    }
  }

  sendMessage(conversationId, content, type = 0, mediaId = null) {
    this.send({
      type: 'message',
      data: {
        conversation_id: conversationId,
        content: content,
        msg_type: type,
        media_id: mediaId
      }
    });
  }

  sendTyping(conversationId, isTyping) {
    this.send({
      type: 'typing',
      data: {
        conversation_id: conversationId,
        is_typing: isTyping
      }
    });
  }

  markAsRead(messageId, conversationId) {
    this.send({
      type: 'mark_as_read',
      data: {
        message_id: messageId,
        conversation_id: conversationId
      }
    });
  }

  requestSync(conversationId, since) {
    this.send({
      type: 'sync_request',
      data: {
        conversation_id: conversationId,
        since: since
      }
    });
  }

  startHeartbeat() {
    this.heartbeatInterval = setInterval(() => {
      this.send({ type: 'heartbeat' });
    }, 30000);
  }

  stopHeartbeat() {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }
  }

  handleReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);
      setTimeout(() => {
        this.reconnectAttempts++;
        console.log(`Reconnecting... Attempt ${this.reconnectAttempts}`);
        this.connect();
      }, delay);
    } else {
      console.error('Max reconnection attempts reached');
    }
  }

  handleMessage(message) {
    switch (message.type) {
      case 'message':
        this.onMessage(message.data);
        break;
      case 'ai_message':
        this.onAIMessage(message.data);
        break;
      case 'typing':
        this.onTyping(message.data);
        break;
      case 'sync_response':
        this.onSyncResponse(message.data);
        break;
      case 'message_ack':
        this.onMessageAck(message.data);
        break;
      case 'system_notification':
        this.onSystemNotification(message.data);
        break;
      case 'group_update':
        this.onGroupUpdate(message.data);
        break;
      case 'heartbeat_ack':
        // Heartbeat response received
        break;
    }
  }

  // Event handlers (to be implemented by the UI)
  onMessage(message) {
    console.log('New message:', message);
  }

  onAIMessage(message) {
    console.log('AI message:', message);
  }

  onTyping(data) {
    console.log('Typing status:', data);
  }

  onSyncResponse(data) {
    console.log('Sync messages:', data);
  }

  onMessageAck(data) {
    console.log('Message acknowledged:', data);
  }

  onSystemNotification(data) {
    console.log('System notification:', data);
  }

  onGroupUpdate(data) {
    console.log('Group update:', data);
  }

  close() {
    this.stopHeartbeat();
    if (this.ws) {
      this.ws.close();
    }
  }
}
```

## Best Practices

### 1. Connection Management
- Always implement reconnection logic with exponential backoff
- Clean up intervals and event listeners when closing
- Handle network changes gracefully

### 2. Message Handling
- Validate incoming messages structure
- Handle message ordering with sequence numbers if needed
- Implement local caching for offline scenarios

### 3. Performance
- Batch multiple operations when possible
- Use message queues for offline scenarios
- Implement lazy loading for message history

### 4. Security
- Validate JWT tokens server-side
- Sanitize message content
- Rate limit message sending

### 5. Error Handling
- Log errors appropriately
- Show user-friendly error messages
- Implement fallback mechanisms

## Testing

### WebSocket Testing Tools
- `wscat` - Command-line WebSocket client
- WebSocket King - Chrome extension
- Postman - WebSocket testing support

### Example Test with wscat
```bash
# Connect to presence WebSocket
wscat -c ws://localhost:8080/ws

# Connect to message WebSocket with token
wscat -c "ws://localhost:8080/ws?token=your_jwt_token"
```

## Troubleshooting

### Common Issues
1. **Connection refused**: Check if server is running
2. **Authentication failed**: Verify JWT token validity
3. **Frequent disconnections**: Check heartbeat implementation
4. **Messages not delivered**: Verify network connectivity

### Debug Mode
Set `RUST_LOG=debug` environment variable to enable detailed WebSocket logging on the server.