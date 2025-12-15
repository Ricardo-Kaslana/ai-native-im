# Qdrant 向量维度不匹配问题

## 问题描述

**错误信息**：
```
Vector dimension error: expected dim: 1536, got 1024
gRPC /qdrant.Points/Search failed with Client specified an invalid argument
```

## 问题原因

Qdrant 集合创建时配置的向量维度与实际使用的 embedding 模型输出维度不匹配：

- **集合配置维度**：1536（OpenAI text-embedding-3-small）
- **实际向量维度**：1024（SiliconFlow BAAI/bge-large-zh-v1.5）

## 常见 Embedding 模型维度

| 模型 | 维度 |
|------|------|
| OpenAI text-embedding-3-small | 1536 |
| OpenAI text-embedding-ada-002 | 1536 |
| SiliconFlow BAAI/bge-large-zh-v1.5 | 1024 |
| SiliconFlow BAAI/bge-m3 | 1024 |

## 解决方案

### 方案 1：删除现有集合（推荐）

删除现有的 Qdrant 集合，让服务器用正确的维度重新创建：

```bash
# 通过 Qdrant API 删除集合
curl -X DELETE "http://localhost:6333/collections/chat_messages"
```

或者重启 Qdrant 并清除数据：
```bash
docker-compose down
docker volume rm ai-native-im_qdrant_data
docker-compose up -d
```

### 方案 2：设置环境变量

在 `.env` 文件中设置 `EMBEDDING_DIMENSION`：

```bash
# 如果使用 SiliconFlow embedding
EMBEDDING_DIMENSION=1024

# 如果使用 OpenAI embedding
EMBEDDING_DIMENSION=1536
```

## 代码修改

已修改 `server/src/services/ai/vector_store.rs`：

1. 添加 `get_embedding_dimension()` 函数，支持从环境变量读取维度
2. 默认维度改为 1024（兼容 SiliconFlow）
3. 创建集合时使用动态维度

```rust
/// 获取 embedding 维度，支持环境变量配置
pub fn get_embedding_dimension() -> u64 {
    std::env::var("EMBEDDING_DIMENSION")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1024) // 默认 1024 兼容 SiliconFlow
}
```

## 注意事项

1. 修改维度后必须删除现有集合，否则会继续报错
2. 确保 embedding 模型和维度配置一致
3. 如果切换 embedding 模型，需要重新索引所有数据

## 状态

✅ 已修复

## 日期

2025-12-07
