# AI Agent Extension 类型修复

## 问题描述

创建 AI Agent 时失败，错误信息：
```
❌ ERROR[500] => /groups/bfb2a6ff-1393-4d37-8bdc-1af1dd21b1fc/agent
Response: Missing request extension: Extension of type `uuid::Uuid` was not found.
```

## 根本原因

### JWT 中间件设置
JWT 中间件设置的是 `Extension<UserContext>`：
```rust
// middleware/jwt.rs
pub struct UserContext {
    pub user_id: Uuid,
    pub username: String,
}

// 中间件添加
req.extensions_mut().insert(user_ctx);
```

### AI Agent API 期望
AI Agent API 函数期望的是 `Extension<Uuid>`：
```rust
// api/ai_agents.rs (修复前)
async fn create_agent(
    Extension(user_id): Extension<Uuid>,  // ❌ 错误
    ...
) -> impl IntoResponse {
    ...
}
```

### 类型不匹配
- 中间件提供：`Extension<UserContext>`
- API 期望：`Extension<Uuid>`
- 结果：Axum 找不到 `Extension<Uuid>`，返回 500 错误

## 解决方案

将所有 AI Agent API 函数改为使用 `Extension<UserContext>`：

### 修复的函数

#### 1. create_agent
```rust
// 修复前
async fn create_agent(
    Extension(user_id): Extension<Uuid>,
    ...
) -> impl IntoResponse {
    let _ = user_id;
    ...
}

// 修复后
async fn create_agent(
    Extension(user_ctx): Extension<crate::middleware::UserContext>,
    ...
) -> impl IntoResponse {
    let _ = user_ctx.user_id;  // 从 UserContext 获取 user_id
    ...
}
```

#### 2. delete_agent
```rust
// 修复前
async fn delete_agent(
    Extension(user_id): Extension<Uuid>,
    ...
) -> impl IntoResponse

// 修复后
async fn delete_agent(
    Extension(user_ctx): Extension<crate::middleware::UserContext>,
    ...
) -> impl IntoResponse
```

#### 3. update_personality
```rust
// 修复前
async fn update_personality(
    Extension(user_id): Extension<Uuid>,
    ...
) -> impl IntoResponse

// 修复后
async fn update_personality(
    Extension(user_ctx): Extension<crate::middleware::UserContext>,
    ...
) -> impl IntoResponse
```

#### 4. set_affinity
```rust
// 修复前
async fn set_affinity(
    Extension(user_id): Extension<Uuid>,
    ...
) -> impl IntoResponse

// 修复后
async fn set_affinity(
    Extension(user_ctx): Extension<crate::middleware::UserContext>,
    ...
) -> impl IntoResponse
```

### 不需要修复的函数

这些函数不需要用户认证，所以不需要修改：
- `get_agent()` - 获取 AI Agent 详情
- `get_agent_for_group()` - 获取群组的 AI Agent
- `get_statistics()` - 获取统计信息

## 为什么使用 UserContext？

### 优势

1. **更多信息**：
   ```rust
   pub struct UserContext {
       pub user_id: Uuid,      // 用户 ID
       pub username: String,   // 用户名
   }
   ```

2. **一致性**：
   - 其他 API（群组、消息等）都使用 `UserContext`
   - 保持代码风格统一

3. **可扩展性**：
   - 未来可以添加更多字段（如角色、权限）
   - 不需要修改所有 API 函数

4. **类型安全**：
   - 明确表示这是用户上下文
   - 避免混淆不同的 UUID

### 访问用户 ID

```rust
// 从 UserContext 获取 user_id
let user_id = user_ctx.user_id;

// 从 UserContext 获取 username
let username = user_ctx.username;
```

## 与其他 API 的一致性

### 群组 API
```rust
// server/src/api/groups.rs
async fn create_group(
    Extension(user_ctx): Extension<UserContext>,
    ...
) -> impl IntoResponse
```

### 消息 API
```rust
// server/src/api/conversations.rs
async fn get_conversation_messages(
    Extension(user_ctx): Extension<UserContext>,
    ...
) -> impl IntoResponse
```

### AI Agent API（修复后）
```rust
// server/src/api/ai_agents.rs
async fn create_agent(
    Extension(user_ctx): Extension<crate::middleware::UserContext>,
    ...
) -> impl IntoResponse
```

## 测试

### 重新编译服务器
```bash
cd server
cargo build --release
cargo run --release
```

### 测试创建 AI Agent

1. **打开 AI Agent 界面**：
   - 群组聊天 → ℹ️ 图标 → 🤖 AI 助手

2. **填写表单**：
   - 名称：科莱塔
   - 毒舌程度：5
   - 热情度：5
   - 口癖：喵~

3. **点击创建**

4. **预期结果**：
   - ✅ 创建成功
   - ✅ 显示 AI Agent 详情
   - ✅ 可以查看统计信息

### 验证日志

**客户端**：
```
🔵 REQUEST[POST] => /groups/.../agent
✅ RESPONSE[201] => /groups/.../agent
Data: {success: true, data: {...}}
```

**服务器**：
```
🔵 REQUEST[POST] => /groups/.../agent
✅ AI Agent created: ...
```

不应该看到：
```
❌ ERROR[500] => Missing request extension
```

## 未来改进

### 1. 添加权限检查

目前有 TODO 注释：
```rust
// TODO: Check if user is group owner
let _ = user_ctx.user_id;
```

应该实现：
```rust
// 检查用户是否是群主
let is_owner = check_group_owner(&pool, group_id, user_ctx.user_id).await?;
if !is_owner {
    return Err(StatusCode::FORBIDDEN);
}
```

### 2. 使用 user_ctx.username

可以在日志中使用用户名：
```rust
tracing::info!(
    user_id = %user_ctx.user_id,
    username = %user_ctx.username,
    "Creating AI Agent"
);
```

### 3. 统一导入

在文件顶部添加：
```rust
use crate::middleware::UserContext;
```

然后可以简化为：
```rust
Extension(user_ctx): Extension<UserContext>
```

## 相关问题

这个问题与之前修复的群组 API 问题相同：
- 参考：`probs/群聊创建问题修复.md`
- 原因：Extension 类型不匹配
- 解决方案：统一使用 `UserContext`

## 状态

✅ 已修复
✅ 需要重新编译服务器
⏳ 等待测试

## 检查清单

- [x] 修复 `create_agent`
- [x] 修复 `delete_agent`
- [x] 修复 `update_personality`
- [x] 修复 `set_affinity`
- [x] 验证不需要认证的函数
- [x] 检查编译错误
- [ ] 重新编译服务器
- [ ] 测试创建 AI Agent
- [ ] 测试更新 AI Agent
- [ ] 测试删除 AI Agent
