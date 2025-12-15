# WebSocket 连接重置问题

## 错误信息

```
2025-12-06T13:27:20.320110Z ERROR ai_im_server::services::message::websocket_handler: 
WebSocket error user_id=bf3d6024-0714-48b5-9cc3-7373754af750 
error=WebSocket protocol error: Connection reset without closing handshake
```

## 问题分析

### 这不是真正的错误！

这是一个 **正常的网络行为**，表示 WebSocket 连接被突然关闭，没有经过标准的关闭握手。

### 常见原因

#### 1. 开发环境（最常见）
- ✅ **Flutter 热重载** (`r` 命令)
- ✅ **Flutter 热重启** (`R` 命令)
- ✅ **应用重启**
- ✅ **调试器断开**

#### 2. 用户行为
- ✅ **应用最小化**
- ✅ **切换到其他应用**
- ✅ **用户登出**
- ✅ **关闭应用**

#### 3. 网络问题
- ✅ **网络切换**（WiFi → 移动网络）
- ✅ **网络波动**
- ✅ **超时**
- ✅ **防火墙/代理干扰**

### 为什么会发生？

**正常关闭流程**：
1. 客户端发送 WebSocket Close 帧
2. 服务器响应 Close 帧
3. TCP 连接关闭

**连接重置**：
1. TCP 连接直接关闭（RST 包）
2. 没有 WebSocket Close 帧
3. 服务器检测到连接断开

## 客户端行为

客户端有 **自动重连机制**：

```
🔌 WebSocket disconnected
🔄 Scheduling reconnect attempt 1 in 3s
🔌 Connecting to WebSocket: ws://localhost:8080/ws?token=...
✅ WebSocket connected
```

- 断开后等待 3 秒
- 自动重新连接
- 最多尝试 5 次
- 连接成功后恢复正常

## 解决方案

### 方案 1：降低日志级别（已实施）✅

将 "Connection reset" 错误改为 DEBUG 级别：

```rust
Some(Err(e)) => {
    // Connection reset is normal (client closed, network issue, etc.)
    if e.to_string().contains("Connection reset") {
        tracing::debug!(
            user_id = %user_id,
            "WebSocket connection reset (normal)"
        );
    } else {
        tracing::error!(
            user_id = %user_id,
            error = %e,
            "WebSocket error"
        );
    }
    break;
}
```

**效果**：
- 正常的连接重置不再显示为 ERROR
- 真正的错误仍然会记录
- 日志更清晰

### 方案 2：客户端优雅关闭（已实现）✅

客户端已经使用正确的关闭状态码：

```dart
void disconnect() {
  _channel?.sink.close(status.goingAway);  // ✅ 正确
}
```

### 方案 3：增加心跳超时时间

如果频繁断开，可以增加心跳超时：

```rust
pub struct WebSocketConfig {
    pub heartbeat_timeout_secs: u64,  // 默认 10 秒
}
```

可以改为 30 秒或更长。

## 何时需要关注？

### ⚠️ 需要关注的情况

1. **频繁断开**（每分钟多次）
   - 可能是网络不稳定
   - 检查网络质量

2. **无法重连**
   - 检查服务器是否正常
   - 检查认证 token 是否有效

3. **其他 WebSocket 错误**
   - 不是 "Connection reset" 的错误
   - 需要具体分析

### ✅ 不需要关注的情况

1. **开发环境中的断开**
   - 热重载导致
   - 完全正常

2. **偶尔的断开**（几分钟一次）
   - 网络波动
   - 自动重连会处理

3. **用户操作导致的断开**
   - 登出、关闭应用
   - 预期行为

## 监控建议

### 正常指标

- **重连成功率** > 95%
- **平均重连时间** < 5 秒
- **连接持续时间** > 5 分钟

### 异常指标

- **重连失败率** > 10%
- **频繁断开**（< 1 分钟）
- **长时间无法连接** > 30 秒

## 生产环境优化

### 1. 添加连接质量监控

```rust
// 记录连接统计
struct ConnectionStats {
    total_connections: u64,
    total_disconnects: u64,
    reset_disconnects: u64,
    normal_disconnects: u64,
}
```

### 2. 区分断开原因

```rust
enum DisconnectReason {
    Normal,           // 正常关闭
    Reset,            // 连接重置
    Timeout,          // 超时
    Error(String),    // 其他错误
}
```

### 3. 添加重连退避策略

客户端已实现：
- 第 1 次：3 秒
- 第 2 次：3 秒
- 第 3 次：3 秒
- ...
- 最多 5 次

可以改为指数退避：
- 第 1 次：1 秒
- 第 2 次：2 秒
- 第 3 次：4 秒
- 第 4 次：8 秒
- 第 5 次：16 秒

## 总结

### 当前状态

✅ **这是正常现象，不需要担心**

- 客户端有自动重连
- 服务器正确处理断开
- 日志级别已优化

### 建议

1. **开发环境**：忽略这些日志
2. **生产环境**：监控重连成功率
3. **优化**：如果频繁断开，考虑增加心跳超时

### 何时需要修复？

- ❌ 开发环境中的偶尔断开 → 不需要
- ❌ 热重载导致的断开 → 不需要
- ⚠️ 生产环境中频繁断开 → 需要调查
- ⚠️ 无法重连 → 需要修复

## 状态

✅ 已优化日志级别
✅ 客户端重连机制正常
✅ 不影响功能使用
