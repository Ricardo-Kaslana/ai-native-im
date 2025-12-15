# AI Agent 群消息不显示给群组成员问题

## 问题描述

AI Agent 发送消息到群组后，服务器日志显示消息发送成功（`ack: Success`），但群组中的其他成员看不到该消息。

## 错误现象

服务器日志：
```
Agent 弗洛洛 sent message ... to group ... (ack: Success)
```

但客户端没有收到消息。

## 根本原因

`MessageRouterImpl` 的 `route_to_group` 方法没有数据库连接池，无法查询群组成员列表。原实现只会尝试向消息发送者（即 AI Agent 本身）投递消息，而不是向所有群组成员投递。

原代码逻辑：
```rust
async fn route_to_group(&self, group_id: &Uuid, message: &Message) -> Result<u32, RouterError> {
    // 只向 sender_id 投递，而 sender_id 是 AI Agent，不是真实用户
    if self.deliver_locally(&message.sender_id, message).await {
        delivered_count = 1;
    }
}
```

## 解决方案

### 1. 添加数据库连接池字段 (`message_router.rs`)

```rust
pub struct MessageRouterImpl {
    connection_manager: Arc<InMemoryConnectionManager>,
    dedup_cache: Arc<InMemoryDedupCache>,
    persister: Arc<dyn MessagePersister>,
    redis_client: Option<Arc<redis::aio::ConnectionManager>>,
    pool: Option<sqlx::PgPool>,  // 新增
}
```

### 2. 添加 with_pool 方法

```rust
pub fn with_pool(mut self, pool: sqlx::PgPool) -> Self {
    self.pool = Some(pool);
    self
}
```

### 3. 添加查询群组成员方法

```rust
async fn get_group_members(&self, group_id: &Uuid) -> Vec<Uuid> {
    if let Some(ref pool) = self.pool {
        match sqlx::query_scalar!(
            r#"SELECT user_id FROM group_members WHERE group_id = $1"#,
            group_id
        )
        .fetch_all(pool)
        .await
        {
            Ok(members) => members,
            Err(e) => {
                tracing::warn!("Failed to get group members: {}", e);
                vec![]
            }
        }
    } else {
        vec![]
    }
}
```

### 4. 修改 route_to_group 方法

```rust
async fn route_to_group(&self, group_id: &Uuid, message: &Message) -> Result<u32, RouterError> {
    let channel = format!("group:{}", group_id);
    self.publish_to_redis(&channel, message).await?;

    // 获取所有群组成员并向在线成员投递
    let members = self.get_group_members(group_id).await;
    let mut delivered_count = 0u32;
    
    for member_id in &members {
        if self.deliver_locally(member_id, message).await {
            delivered_count += 1;
        }
    }
    
    // 如果没有成员（无数据库连接池），至少向发送者投递
    if members.is_empty() {
        if self.deliver_locally(&message.sender_id, message).await {
            delivered_count = 1;
        }
    }

    tracing::debug!(
        group_id = %group_id,
        message_id = %message.id,
        member_count = members.len(),
        delivered_count = delivered_count,
        "Group message delivered to members"
    );

    Ok(delivered_count)
}
```

### 5. 在 main.rs 中传递数据库连接池

```rust
let base_router = Arc::new(
    MessageRouterImpl::new(
        message_connection_manager.clone(),
        dedup_cache,
        message_persister,
    ).with_pool(pool.clone())  // 传递数据库连接池
);
```

## 修改的文件

- `server/src/services/message/message_router.rs`
- `server/src/main.rs`

## 验证方法

1. 重启服务器
2. 在群组中 @AI Agent 或发送消息触发 AI 回复
3. 检查群组中其他成员是否能实时收到 AI Agent 的消息
4. 查看服务器日志中的 `member_count` 和 `delivered_count` 字段

## 日期

2024-12-07
