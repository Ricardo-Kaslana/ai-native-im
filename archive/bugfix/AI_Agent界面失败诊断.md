# AI Agent 界面失败诊断

## 问题描述

进入群聊信息界面成功，但点击 "AI 助手" 后界面失败。

## 可能的原因

### 1. API 请求失败

**检查点**：
- 后端 API 端点是否正常
- 认证 token 是否有效
- 网络连接是否正常

**API 端点**：
```
GET /groups/:group_id/agent
```

### 2. 数据解析错误

**检查点**：
- 后端返回的数据格式是否正确
- 客户端模型是否匹配后端

### 3. 权限问题

**检查点**：
- 用户是否有权限查看 AI Agent
- JWT token 是否包含在请求中

## 调试步骤

### 步骤 1：查看客户端日志

在 Flutter 控制台查找：
```
❌ ERROR[xxx] => /groups/.../agent
```

或者：
```
加载 AI Agent 失败: ...
```

### 步骤 2：查看服务器日志

在服务器控制台查找：
```
🔵 REQUEST[GET] => /groups/.../agent
```

以及对应的响应或错误。

### 步骤 3：检查数据库

```sql
-- 检查 AI Agent 表是否存在
SELECT * FROM ai_agents LIMIT 1;

-- 检查特定群组是否有 AI Agent
SELECT * FROM ai_agents WHERE group_id = '<your_group_id>';
```

### 步骤 4：手动测试 API

使用 curl 或 Postman 测试：

```bash
curl -H "Authorization: Bearer <your_token>" \
  http://localhost:8080/groups/<group_id>/agent
```

预期响应：
```json
{
  "success": true,
  "data": null  // 如果没有 AI Agent
}
```

或者：
```json
{
  "success": true,
  "data": {
    "id": "...",
    "group_id": "...",
    "name": "...",
    ...
  }
}
```

## 常见问题和解决方案

### 问题 1：404 Not Found

**原因**：API 路由未注册或路径错误

**解决方案**：
1. 检查 `server/src/main.rs` 中是否有：
   ```rust
   .merge(api::ai_agent_routes(pool.clone()))
   ```

2. 重启服务器

### 问题 2：401 Unauthorized

**原因**：认证失败

**解决方案**：
1. 检查客户端是否发送了 Authorization 头
2. 检查 token 是否过期
3. 重新登录

### 问题 3：500 Internal Server Error

**原因**：服务器内部错误

**解决方案**：
1. 查看服务器日志获取详细错误
2. 检查数据库连接
3. 检查 AI Agent 表结构

### 问题 4：数据解析错误

**原因**：客户端模型与后端不匹配

**解决方案**：
1. 检查 `AIAgent.fromJson()` 方法
2. 对比后端返回的字段
3. 添加错误处理

## 临时解决方案

如果 AI Agent 功能暂时无法使用，可以：

1. **跳过错误显示**：
   修改 `ai_agent_screen.dart`，在错误时显示"功能开发中"

2. **禁用 AI Agent 入口**：
   暂时隐藏群组信息页面的 AI Agent 选项

3. **使用模拟数据**：
   在客户端返回模拟的 AI Agent 数据

## 需要的信息

为了更好地诊断问题，请提供：

1. **客户端日志**：
   - Flutter 控制台的完整错误信息
   - 特别是 API 请求和响应

2. **服务器日志**：
   - 服务器控制台的相关日志
   - 特别是 `/groups/.../agent` 的请求

3. **群组 ID**：
   - 你尝试访问的群组 ID
   - 用于检查数据库

4. **错误截图**：
   - 如果有错误提示界面

## 快速检查清单

- [ ] 服务器正在运行
- [ ] 客户端已登录
- [ ] 网络连接正常
- [ ] 群组 ID 正确
- [ ] AI Agent 路由已注册
- [ ] 数据库表存在
- [ ] 查看了客户端日志
- [ ] 查看了服务器日志

## 下一步

请运行以下命令并提供输出：

### 客户端
```bash
# 在 Flutter 控制台查看完整日志
# 特别关注包含 "agent" 的行
```

### 服务器
```bash
# 查看服务器日志
# 特别关注 API 请求
```

### 数据库
```sql
-- 检查表是否存在
\dt ai_agents

-- 检查表结构
\d ai_agents

-- 检查数据
SELECT COUNT(*) FROM ai_agents;
```

提供这些信息后，我可以给出更具体的解决方案。
