# AI Agent 重复回复问题

## 状态：✅ 已解决

## 问题描述

AI Agent 在群组中回复消息时，会发送两条相同的消息。

## 错误现象

客户端日志显示：
```
📥 Message received: 135527e3-d914-4316-aedf-77112cde6a07
📨 WebSocket message received for conversation ...: 135527e3-d914-4316-aedf-77112cde6a07
🔄 Skipping duplicate message: 135527e3-d914-4316-aedf-77112cde6a07 (content: 我会一点点，主要是用来打游戏和上网的喵~...)
🌐 GlobalMessageHandler persisted message: 135527e3-d914-4316-aedf-77112cde6a07
🔄 Skipping duplicate message: 135527e3-d914-4316-aedf-77112cde6a07 (content: 我会一点点，主要是用来打游戏和上网的喵~...)
```

同一条消息被接收了两次。

## 根本原因

`AgentResponseWorker` 收到了重复的响应，没有去重机制。虽然 `AgentMessageHandler::process_message()` 只发送一次响应，但由于某些原因（如网络重试、channel 缓冲等），响应可能被处理多次。

## 解决方案

### 1. 移除 AIMessageRouter 中的重复发送代码

```rust
// 修改前
match handler.process_message(agent_message, &recent_messages).await {
    Ok(Some(response)) => {
        if let Some(tx) = response_tx {
            tx.send(response);  // 可能重复发送
        }
    }
}

// 修改后
match handler.process_message(agent_message, &recent_messages).await {
    Ok(Some(response)) => {
        // NOTE: Response is already sent by AgentMessageHandler.process_message()
        // Do NOT send again here to avoid duplicate messages.
    }
}
```

### 2. 在 AgentResponseWorker 中添加去重机制

```rust
pub struct AgentResponseWorker {
    response_rx: mpsc::UnboundedReceiver<AgentResponse>,
    message_sender: Arc<dyn AgentMessageSender>,
    /// Track recently processed message IDs to prevent duplicates
    processed_messages: std::collections::HashSet<Uuid>,
}

impl AgentResponseWorker {
    pub async fn run(mut self) {
        while let Some(response) = self.response_rx.recv().await {
            // Check for duplicate response (same reply_to_message_id means same trigger)
            if self.processed_messages.contains(&response.reply_to_message_id) {
                info!("⏭️ Skipping duplicate response for message {}", response.reply_to_message_id);
                continue;
            }
            
            // Mark as processed
            self.processed_messages.insert(response.reply_to_message_id);
            
            // Clean up old entries if too many (keep last 1000)
            if self.processed_messages.len() > 1000 {
                self.processed_messages.clear();
            }

            // ... send message
        }
    }
}
```

### 3. 添加详细日志追踪

在关键位置添加日志以便追踪消息流程：
- `🔍 AgentMessageHandler.process_message called` - process_message 被调用
- `📤 Sending response to channel` - 响应发送到 channel
- `📥 AgentResponseWorker received response` - worker 收到响应
- `⏭️ Skipping duplicate response` - 检测到重复并跳过
- `✅ Processing response` - 正在处理响应

## 修改的文件

- `server/src/services/ai/ai_message_router.rs` - 移除重复发送代码
- `server/src/services/ai/agent_message_handler.rs` - 添加去重机制和详细日志

## 消息流程

**修复后：**
```
用户消息 → AIMessageRouter.route_message()
              ↓
         AgentMessageHandler.process_message()
              ↓
         response_tx.send(response)
              ↓
         AgentResponseWorker.run()
              ↓
         检查 processed_messages 是否已处理
              ↓
         如果未处理：发送消息，标记为已处理
         如果已处理：跳过
```

## 验证方法

1. 重启服务器
2. 在群组中 @AI Agent 或发送消息触发回复
3. 确认 AI Agent 只回复一条消息
4. 服务器日志应显示相关追踪信息

## 解决日期

2024-12-08
