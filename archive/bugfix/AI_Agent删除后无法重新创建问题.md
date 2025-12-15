# AI Agent 删除后无法重新创建问题

## 问题描述

用户删除 AI Agent 后，尝试为同一个群组创建新的 AI Agent 时，出现错误提示："Group already has an AI Agent"。

## 错误信息

```
Response: {success: false, data: null, error: Group already has an AI Agent}
```

## 根本原因

**位置**：`server/src/services/ai/agent_service.rs` - `create_agent` 方法

**原因**：
1. `delete_agent` 方法使用软删除，只是将 `is_active` 设置为 `FALSE`，并未真正删除记录
2. `create_agent` 方法检查群组是否已有 AI Agent 时，使用的查询是：
   ```sql
   SELECT EXISTS(SELECT 1 FROM ai_agents WHERE group_id = $1)
   ```
3. 这个查询没有检查 `is_active` 状态，所以即使 Agent 已被软删除，仍然会被检测到
4. 导致无法为同一个群组创建新的 AI Agent

## 代码分析

### delete_agent 方法（软删除）

```rust
pub async fn delete_agent(&self, agent_id: Uuid) -> Result<(), AIAgentError> {
    let result = sqlx::query!(
        r#"UPDATE ai_agents SET is_active = FALSE WHERE id = $1"#,
        agent_id
    )
    .execute(&self.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AIAgentError::AgentNotFound(agent_id));
    }

    info!("Deactivated AI Agent {}", agent_id);
    Ok(())
}
```

### create_agent 方法（问题代码）

**修改前**：
```rust
pub async fn create_agent(
    &self,
    group_id: Uuid,
    request: CreateAIAgent,
) -> Result<AIAgent, AIAgentError> {
    // ...

    // Check if group already has an agent
    let existing = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM ai_agents WHERE group_id = $1) as "exists!""#,
        group_id
    )
    .fetch_one(&self.pool)
    .await?;

    if existing {
        return Err(AIAgentError::AgentAlreadyExists);  // ❌ 即使是软删除的也会被检测到
    }

    // ...
}
```

## 解决方案

**文件**：`server/src/services/ai/agent_service.rs`

**修改**：只检查活跃的 AI Agent

```rust
pub async fn create_agent(
    &self,
    group_id: Uuid,
    request: CreateAIAgent,
) -> Result<AIAgent, AIAgentError> {
    Self::validate_name(&request.name)?;

    let toxicity = request.toxicity_level.unwrap_or(5);
    let enthusiasm = request.enthusiasm_level.unwrap_or(5);
    Self::validate_levels(toxicity, enthusiasm)?;

    // Check if group already has an active agent
    let existing = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM ai_agents WHERE group_id = $1 AND is_active = TRUE) as "exists!""#,
        group_id
    )
    .fetch_one(&self.pool)
    .await?;

    if existing {
        return Err(AIAgentError::AgentAlreadyExists);
    }

    // ... rest of the code
}
```

## 关键变化

| 修改前 | 修改后 |
|-------|-------|
| `WHERE group_id = $1` | `WHERE group_id = $1 AND is_active = TRUE` |

## 软删除设计说明

软删除的优点：
- 保留历史数据和统计信息
- 可以恢复已删除的 Agent
- 保持数据完整性（外键引用）

软删除的注意事项：
- 所有查询都需要考虑 `is_active` 状态
- 唯一性约束需要包含 `is_active` 字段
- 创建新记录前要检查是否有活跃记录

## 相关方法

### get_agent_for_group（已正确实现）

```rust
pub async fn get_agent_for_group(&self, group_id: Uuid) -> Result<Option<AIAgent>, AIAgentError> {
    let agent = sqlx::query_as!(
        AIAgent,
        r#"
        SELECT id, group_id, name, toxicity_level, enthusiasm_level, expertise_domains, speech_suffix, trigger_keywords, is_active, created_at, updated_at
        FROM ai_agents WHERE group_id = $1 AND is_active = TRUE
        "#,
        group_id
    )
    .fetch_optional(&self.pool)
    .await?;

    Ok(agent)
}
```

这个方法已经正确地只查询活跃的 Agent。

## 测试验证

1. 创建一个 AI Agent
2. 删除该 AI Agent
3. 尝试为同一个群组创建新的 AI Agent
4. 验证创建成功
5. 检查数据库中旧的 Agent 记录 `is_active = FALSE`
6. 检查数据库中新的 Agent 记录 `is_active = TRUE`

## 数据库查询示例

```sql
-- 查看群组的所有 AI Agent（包括已删除的）
SELECT id, name, is_active, created_at, updated_at 
FROM ai_agents 
WHERE group_id = 'xxx-xxx-xxx';

-- 查看群组的活跃 AI Agent
SELECT id, name, is_active, created_at, updated_at 
FROM ai_agents 
WHERE group_id = 'xxx-xxx-xxx' AND is_active = TRUE;
```

## 相关文件

- `server/src/services/ai/agent_service.rs` - AI Agent 服务
- `server/src/api/ai_agents.rs` - AI Agent API
- `client/lib/presentation/providers/ai_agent_provider.dart` - 客户端 AI Agent provider

## 状态

✅ 已修复

## 日期

2024-12-06
