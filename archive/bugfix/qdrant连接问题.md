# Qdrant 连接问题

## 问题描述
后端启动时出现 Qdrant 连接警告：
```
Failed to obtain server version. Unable to check client-server compatibility.
Set check_compatibility=false to skip version check.
⚠️ Failed to connect to Qdrant: Vector database error: Failed to check collection: 
Error in the response: The operation was cancelled operation was canceled
Using mock vector store as fallback
```

## 原因分析
1. Qdrant 客户端版本与服务器版本不匹配
2. 客户端尝试检查版本兼容性时失败
3. 可能是 Qdrant 服务器未正确响应版本检查请求

## 根本原因
经过测试发现：
- Qdrant 服务正在运行（端口 6333 可访问）
- qdrant-client 1.16 默认使用 gRPC 协议（端口 6334）
- 当前配置使用 HTTP URL (`http://localhost:6333`)
- 客户端尝试通过 HTTP 端口进行 gRPC 通信导致失败

## 解决方案

### 方案 1: 使用 gRPC 端口（推荐）✅
修改 `.env` 文件中的 QDRANT_URL：
```env
# 从 HTTP 端口改为 gRPC 端口
QDRANT_URL=http://localhost:6334
```

### 方案 2: 降级客户端版本
如果 Qdrant 服务器不支持 gRPC，可以降级到使用 HTTP 的旧版本：
```toml
# 在 server/Cargo.toml 中
qdrant-client = "1.7"
```

### 方案 3: 配置客户端使用 HTTP（需要代码修改）
在 `server/src/services/ai/vector_store.rs` 中明确指定使用 REST API 而非 gRPC。

## 当前状态
- ✅ 已升级 qdrant-client 到 1.16 版本
- ✅ 代码编译成功
- ✅ 后端服务已启动
- ⚠️ Qdrant 连接失败（协议不匹配）
- ✅ 系统自动降级使用 MockVectorStore，基本功能正常

## 影响范围
- AI Agent 的语义搜索功能
- 历史消息的向量存储和检索
- 不影响基本的消息收发功能

## 测试步骤
1. 确保 Qdrant 服务正在运行
   - HTTP API: localhost:6333 ✅
   - gRPC API: localhost:6334 ✅
2. 修改 `.env` 文件中的 QDRANT_URL 为 `http://localhost:6334`
3. 重启后端服务
4. 查看启动日志中的 Qdrant 连接状态

## 验证结果
- ✅ 端口 6333 (HTTP) 可访问
- ✅ 端口 6334 (gRPC) 可访问
- ⏳ 需要用户修改 `.env` 文件并重启服务进行最终验证

## 相关文件
- `server/Cargo.toml` - 依赖版本配置
- `server/src/services/ai/vector_store.rs` - Qdrant 客户端初始化
- `server/src/services/ai/system.rs` - AI 系统初始化，包含 fallback 逻辑
