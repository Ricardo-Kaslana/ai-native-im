# AI Native IM - 后端开发白皮书

## 文档概述

本文档是 AI Native IM 后端开发的全面指南，为 Rust 后端开发提供技术规范、架构设计和最佳实践。所有后端开发人员必须遵循本白皮书的规范和指导原则。

### 文档目标
- 建立统一的后端开发标准
- 提供清晰的架构设计指导
- 确保代码质量和可维护性
- 加速团队协作和知识传承

### 适用范围
- Rust 后端服务开发
- API 端点设计和实现
- 数据库设计和迁移
- WebSocket 实时通信
- AI Agent 系统集成

---

## 1. 项目概述

### 1.1 技术栈

#### 核心技术
- **语言**: Rust 1.91.1+
- **Web框架**: Axum 0.7 (基于 Tokio)
- **异步运行时**: Tokio 1.x
- **数据库**: PostgreSQL 16
- **缓存**: Redis 7.0
- **向量数据库**: Qdrant 1.16

#### 依赖库
```toml
# Web Framework
axum = { version = "0.7", features = ["ws"] }
tokio = { version = "1", features = ["full"] }
tower = { version = "0.5", features = ["util"] }
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Database
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "migrate"] }

# Redis
redis = { version = "0.26", features = ["tokio-comp", "connection-manager"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
prost = "0.13"

# Error Handling
thiserror = "2.0"
anyhow = "1.0"

# Utilities
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Security
bcrypt = "0.16"
jsonwebtoken = "9.0"

# HTTP Client (for AI API)
reqwest = { version = "0.12", features = ["json", "stream", "multipart"] }

# Vector Database
qdrant-client = "1.16"

# WebSocket
tokio-tungstenite = "0.24"

# Media Processing
image = { version = "0.25", features = ["webp", "jpeg", "png", "gif"] }
sha2 = "0.10"

# AWS S3
aws-config = "1.1"
aws-sdk-s3 = "1.13"
```

### 1.2 架构设计原则

#### 1. 分层架构
```
┌─────────────────────────────────────────────┐
│            API Layer (Axum)                │
├─────────────────────────────────────────────┤
│          Service Layer                     │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐    │
│  │  Auth   │  │ Message │  │  Group  │    │
│  │ Service │  │ Service │  │ Service │    │
│  └─────────┘  └─────────┘  └─────────┘    │
├─────────────────────────────────────────────┤
│          Data Layer                        │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐    │
│  │PostgreSQL│  │  Redis  │  │ Qdrant  │    │
│  └─────────┘  └─────────┘  └─────────┘    │
└─────────────────────────────────────────────┘
```

#### 2. 模块化设计
- 每个功能模块独立开发和测试
- 清晰的模块边界和接口定义
- 依赖注入和 IoC 容器

#### 3. 异步优先
- 所有 I/O 操作使用 async/await
- 非阻塞的数据库和 HTTP 请求
- 高并发处理能力

---

## 2. 项目结构

### 2.1 目录结构

```
server/
├── src/
│   ├── api/                    # API 路由层
│   │   ├── mod.rs
│   │   ├── auth.rs            # 认证相关 API
│   │   ├── users.rs           # 用户管理 API
│   │   ├── groups.rs          # 群组管理 API
│   │   ├── ai_agents.rs       # AI Agent API
│   │   ├── media.rs           # 媒体处理 API
│   │   ├── admin.rs           # 管理员 API
│   │   ├── message_ws.rs      # WebSocket 消息 API
│   │   └── ws.rs              # WebSocket 连接 API
│   │
│   ├── middleware/             # 中间件
│   │   ├── mod.rs
│   │   ├── jwt_auth.rs        # JWT 认证中间件
│   │   ├── rate_limit.rs      # 速率限制中间件
│   │   └── cors.rs            # CORS 中间件
│   │
│   ├── models/                 # 数据模型
│   │   ├── mod.rs
│   │   ├── user.rs            # 用户模型
│   │   ├── group.rs           # 群组模型
│   │   └── ai_agent.rs        # AI Agent 模型
│   │
│   ├── services/               # 业务逻辑层
│   │   ├── mod.rs
│   │   ├── auth/              # 认证服务
│   │   ├── message/           # 消息服务
│   │   ├── group/             # 群组服务
│   │   ├── media/             # 媒体服务
│   │   ├── ai/                # AI Agent 服务
│   │   ├── presence/          # 在线状态服务
│   │   └── email/             # 邮件服务
│   │
│   ├── error.rs               # 统一错误处理
│   ├── response.rs            # 统一响应格式
│   ├── lib.rs                 # 库入口
│   └── main.rs                # 主入口
│
├── migrations/                 # 数据库迁移
├── tests/                      # 测试文件
├── Cargo.toml                 # 依赖配置
└── Dockerfile                 # Docker 配置
```

### 2.2 模块职责

#### API 层
- 处理 HTTP 请求和响应
- 参数验证和序列化
- 调用服务层方法
- 错误处理和状态码映射

#### 服务层
- 实现业务逻辑
- 数据转换和验证
- 调用数据访问层
- 事务管理

#### 数据层
- 数据库操作
- 缓存管理
- 外部服务调用
- 数据持久化

---

## 3. 开发规范

### 3.1 代码风格

#### 命名规范

1. **模块和文件**
   - 使用 snake_case: `auth_service.rs`
   - 模块名与文件名一致

2. **类型和结构体**
   - 使用 PascalCase: `UserService`, `Message`
   - 错误类型以 `Error` 结尾: `AuthError`

3. **函数和方法**
   - 使用 snake_case: `get_user`, `create_message`
   - 返回 bool 的方法以 `is`, `has`, `can` 开头

4. **常量**
   - 使用 UPPER_SNAKE_CASE: `MAX_FILE_SIZE`, `DEFAULT_TIMEOUT`

#### 代码组织

```rust
// 1. 导入语句
use std::sync::Arc;
use axum::{extract::State, response::Json};
use sqlx::PgPool;

// 2. 模块导入
use crate::error::AppError;
use crate::models::User;
use crate::services::auth::AuthService;

// 3. 类型别名
pub type Result<T> = std::result::Result<T, AppError>;

// 4. 常量定义
const MAX_USERNAME_LENGTH: usize = 50;

// 5. 结构体定义
#[derive(Debug, Clone)]
pub struct UserService {
    pool: PgPool,
    auth_service: Arc<AuthService>,
}

// 6. 实现
impl UserService {
    // 7. 关联函数（构造函数）
    pub fn new(pool: PgPool, auth_service: Arc<AuthService>) -> Self {
        Self { pool, auth_service }
    }
    
    // 8. 公有方法
    pub async fn get_user(&self, id: Uuid) -> Result<User> {
        // 实现
    }
    
    // 9. 私有方法
    async fn validate_user(&self, user: &User) -> Result<()> {
        // 实现
    }
}
```

### 3.2 错误处理规范

#### 错误类型定义

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Validation failed: {0}")]
    Validation(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("External service error: {0}")]
    ExternalService(String),
}

// 转换为应用错误
impl From<ServiceError> for AppError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::Validation(msg) => AppError::Validation(msg),
            ServiceError::NotFound(msg) => AppError::NotFound(msg),
            ServiceError::PermissionDenied(msg) => AppError::PermissionDenied(msg),
            ServiceError::Database(_) => AppError::Internal("Database error".to_string()),
            ServiceError::ExternalService(msg) => AppError::Internal(msg),
        }
    }
}
```

#### 错误处理模式

```rust
// 使用 ? 操作符
pub async fn get_user(&self, id: Uuid) -> Result<User> {
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = $1",
        id
    )
    .fetch_one(&self.pool)
    .await?;
    
    Ok(user)
}

// 使用 match 处理特定错误
pub async fn create_user(&self, data: CreateUser) -> Result<User> {
    match self.check_username_exists(&data.username).await {
        Ok(true) => Err(AppError::Conflict("Username already exists".to_string())),
        Ok(false) => {
            // 继续创建用户
            self.insert_user(data).await
        }
        Err(e) => Err(e),
    }
}
```

### 3.3 数据库操作规范

#### 查询编写

```rust
// ✅ 正确：使用 query_as! 宏
pub async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, email, avatar_url, bio, online_status,
               last_seen_at, account_status, role, created_at, updated_at
        FROM users
        WHERE id = $1 AND account_status != 'deleted'
        "#,
        id
    )
    .fetch_optional(&self.pool)
    .await?;
    
    Ok(user)
}

// ❌ 错误：使用字符串拼接
pub async fn get_user_by_id_bad(&self, id: Uuid) -> Result<Option<User>> {
    let query = format!("SELECT * FROM users WHERE id = '{}'", id);
    let user = sqlx::query_as::<_, User>(&query)
        .fetch_optional(&self.pool)
        .await?;
    Ok(user)
}
```

#### 事务处理

```rust
pub async fn transfer_group_ownership(
    &self,
    group_id: Uuid,
    from_owner: Uuid,
    to_owner: Uuid,
) -> Result<()> {
    let mut tx = self.pool.begin().await?;
    
    // 1. 验证权限
    let group = sqlx::query!(
        "SELECT owner_id FROM groups WHERE id = $1 FOR UPDATE",
        group_id
    )
    .fetch_one(&mut *tx)
    .await?;
    
    if group.owner_id != from_owner {
        return Err(AppError::PermissionDenied("Not group owner".to_string()));
    }
    
    // 2. 转移所有权
    sqlx::query!(
        "UPDATE groups SET owner_id = $1 WHERE id = $2",
        to_owner,
        group_id
    )
    .execute(&mut *tx)
    .await?;
    
    // 3. 更新成员角色
    sqlx::query!(
        "UPDATE group_members SET role = 'member' WHERE group_id = $1 AND user_id = $2",
        group_id,
        from_owner
    )
    .execute(&mut *tx)
    .await?;
    
    sqlx::query!(
        "UPDATE group_members SET role = 'owner' WHERE group_id = $1 AND user_id = $2",
        group_id,
        to_owner
    )
    .execute(&mut *tx)
    .await?;
    
    // 4. 提交事务
    tx.commit().await?;
    
    Ok(())
}
```

### 3.4 异步编程规范

#### 任务创建

```rust
// ✅ 正确：使用 tokio::spawn
pub async fn start_background_workers(&self) {
    // 消息处理工作器
    let message_handler = self.message_handler.clone();
    tokio::spawn(async move {
        message_handler.process_messages().await;
    });
    
    // 清理任务
    let cleanup_service = self.cleanup_service.clone();
    tokio::spawn(async move {
        cleanup_service.run_cleanup().await;
    });
}

// ✅ 正确：使用 join! 并行执行
pub async fn get_user_with_messages(&self, user_id: Uuid) -> Result<(User, Vec<Message>)> {
    let user_future = self.get_user(user_id);
    let messages_future = self.get_user_messages(user_id, 10);
    
    let (user, messages) = tokio::try_join!(user_future, messages_future)?;
    
    Ok((user, messages))
}
```

#### 超时处理

```rust
use tokio::time::{timeout, Duration};

pub async def call_external_api(&self) -> Result<String> {
    let result = timeout(
        Duration::from_secs(30),
        self.http_client.get("https://api.example.com").send()
    ).await;
    
    match result {
        Ok(Ok(response)) => Ok(response.text().await?),
        Ok(Err(e)) => Err(AppError::ExternalService(e.to_string())),
        Err(_) => Err(AppError::Timeout),
    }
}
```

---

## 4. API 设计规范

### 4.1 RESTful API 设计

#### 资源命名

```
# 用户资源
GET    /api/users           # 获取用户列表
GET    /api/users/:id       # 获取特定用户
POST   /api/users           # 创建用户
PUT    /api/users/:id       # 更新用户
DELETE /api/users/:id       # 删除用户

# 群组资源
GET    /api/groups          # 获取群组列表
GET    /api/groups/:id      # 获取群组详情
POST   /api/groups          # 创建群组
PUT    /api/groups/:id      # 更新群组
DELETE /api/groups/:id      # 删除群组

# 嵌套资源
GET    /api/groups/:id/members     # 获取群组成员
POST   /api/groups/:id/members     # 添加成员
DELETE /api/groups/:id/members/:uid # 移除成员
```

#### HTTP 状态码使用

```rust
// 成功响应
pub async fn get_user(&self, id: Uuid) -> Result<Json<ApiResponse<User>>> {
    let user = self.user_service.get_user(id).await?;
    Ok(Json(ApiResponse::success(user)))
}

// 创建成功（201）
pub async fn create_user(&self, data: CreateUser) -> Result<(StatusCode, Json<ApiResponse<User>>)> {
    let user = self.user_service.create_user(data).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::success(user))))
}

// 无内容（204）
pub async fn delete_user(&self, id: Uuid) -> Result<StatusCode> {
    self.user_service.delete_user(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

### 4.2 请求和响应格式

#### 统一响应格式

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
        }
    }
}

// 使用示例
pub async fn get_user(&self, id: Uuid) -> Result<Json<ApiResponse<User>>> {
    let user = self.user_service.get_user(id).await?;
    Ok(Json(ApiResponse::success(user)))
}
```

#### 请求验证

```rust
use serde::{Deserialize, Validate};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8))]
    pub password: String,
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<User>>> {
    // 验证请求
    request.validate()
        .map_err(|e| AppError::Validation(format!("Validation failed: {}", e)))?;
    
    let user = state.user_service.create_user(request.into()).await?;
    Ok(Json(ApiResponse::success(user)))
}
```

### 4.3 WebSocket 设计

#### 消息格式

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "message")]
    Message {
        id: Uuid,
        sender_id: Uuid,
        group_id: Option<Uuid>,
        receiver_id: Option<Uuid>,
        content: String,
        timestamp: i64,
    },
    
    #[serde(rename = "typing")]
    Typing {
        user_id: Uuid,
        group_id: Uuid,
        is_typing: bool,
    },
    
    #[serde(rename = "presence")]
    Presence {
        user_id: Uuid,
        status: OnlineStatus,
    },
    
    #[serde(rename = "error")]
    Error {
        code: i32,
        message: String,
    },
}
```

#### 连接处理

```rust
pub async fn handle_websocket(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<WsQuery>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, params.token))
}

async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    token: Option<String>,
) {
    // 1. 验证令牌
    let user_id = match validate_token(token).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Token validation failed: {}", e);
            return;
        }
    };
    
    // 2. 创建连接处理器
    let handler = WsHandler::new(user_id, state);
    
    // 3. 处理连接
    if let Err(e) = handler.handle(socket).await {
        tracing::error!("WebSocket handler error: {}", e);
    }
}
```

---

## 5. AI Agent 系统

### 5.1 架构设计

```
┌─────────────────────────────────────────────────┐
│                 AI Agent System                  │
├─────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌──────────┐ │
│  │   Trigger   │  │ Personality │  │  Memory  │ │
│  │  Evaluator  │  │   Manager   │  │  System  │ │
│  └─────────────┘  └─────────────┘  └──────────┘ │
├─────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌──────────┐ │
│  │   LLM       │  │  Embedding  │  │  Tools   │ │
│  │  Providers  │  │   Service   │  │ Registry │ │
│  └─────────────┘  └─────────────┘  └──────────┘ │
└─────────────────────────────────────────────────┘
```

### 5.2 核心组件

#### 触发评估器

```rust
#[async_trait]
pub trait TriggerEvaluator: Send + Sync {
    async fn evaluate(
        &self,
        message: &str,
        sender_id: &str,
        context: &AgentContext,
    ) -> TriggerDecision;
}

pub struct TriggerDecision {
    pub should_respond: bool,
    pub probability: f64,
    pub trigger_type: TriggerType,
    pub delay_ms: u64,
}

#[derive(Debug, Clone)]
pub enum TriggerType {
    Mention,        // @提及
    Keyword,        // 关键词匹配
    Context,        // 上下文相关
    Random,         // 随机插话
    ColdStart,      // 冷场激活
}
```

#### 人设管理器

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    pub name: String,
    pub toxicity_level: u8,        // 0-10
    pub enthusiasm_level: u8,      // 0-10
    pub expertise_domains: Vec<String>,
    pub speech_suffix: Option<String>,
    pub trigger_keywords: Vec<String>,
    pub emotion_boundaries: EmotionBoundaries,
    pub response_style: ResponseStyle,
}

#[async_trait]
pub trait PersonalityManager: Send + Sync {
    async fn get_personality(&self, agent_id: Uuid) -> Result<Personality>;
    async fn update_personality(&self, agent_id: Uuid, personality: Personality) -> Result<()>;
    async fn apply_affinity(&self, personality: &mut Personality, user_id: Uuid, affinity: i16);
}
```

#### LLM 客户端抽象

```rust
#[async_trait]
pub trait LLMClient: Send + Sync {
    async fn generate_response(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<String>;
    
    async fn generate_stream(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>>;
    
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>>;
}

// Provider 实现
pub struct OpenAIClient {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

#[async_trait]
impl LLMClient for OpenAIClient {
    async fn generate_response(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<String> {
        // 实现调用 OpenAI API
    }
}
```

### 5.3 消息处理流程

```rust
pub struct AgentMessageHandler {
    evaluator: Arc<dyn TriggerEvaluator>,
    personality_manager: Arc<dyn PersonalityManager>,
    llm_client: Arc<dyn LLMClient>,
    memory_store: Arc<dyn MemoryStore>,
    typing_simulator: Arc<dyn TypingSimulator>,
}

impl AgentMessageHandler {
    pub async fn handle_message(
        &self,
        message: &ChatMessage,
        agent: &AIAgent,
    ) -> Result<Option<String>> {
        // 1. 评估触发条件
        let context = self.build_context(message, agent).await?;
        let decision = self.evaluator.evaluate(
            &message.content,
            &message.sender_id.to_string(),
            &context,
        ).await?;
        
        if !decision.should_respond {
            return Ok(None);
        }
        
        // 2. 获取人设
        let personality = self.personality_manager.get_personality(agent.id).await?;
        
        // 3. 检索相关记忆
        let memories = self.memory_store.retrieve(&message.content, 5).await?;
        
        // 4. 构建提示词
        let system_prompt = self.build_system_prompt(&personality, &memories);
        
        // 5. 模拟打字
        if decision.delay_ms > 0 {
            self.typing_simulator.simulate_typing(
                agent.group_id,
                decision.delay_ms,
            ).await?;
        }
        
        // 6. 生成响应
        let response = self.llm_client.generate_response(
            &system_prompt,
            &message.content,
        ).await?;
        
        // 7. 存储记忆
        self.memory_store.store(message, &response).await?;
        
        Ok(Some(response))
    }
}
```

---

## 6. 数据库设计

### 6.1 表设计原则

#### 1. 命名规范

```sql
-- 表名使用复数形式，snake_case
CREATE TABLE users (...);
CREATE TABLE groups (...);
CREATE TABLE messages (...);

-- 列名使用 snake_case
CREATE TABLE users (
    id UUID PRIMARY KEY,
    user_name VARCHAR(50),  -- ❌ 错误：应该是 username
    created_at TIMESTAMPTZ  -- ✅ 正确
);
```

#### 2. 主键和外键

```sql
-- 使用 UUID 作为主键
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- ...
);

-- 外键约束
CREATE TABLE messages (
    id UUID PRIMARY KEY,
    sender_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    group_id UUID REFERENCES groups(id) ON DELETE CASCADE,
    -- ...
);
```

#### 3. 索引设计

```sql
-- 为查询条件创建索引
CREATE INDEX idx_messages_group_time 
    ON messages(group_id, timestamp DESC);

-- 为外键创建索引
CREATE INDEX idx_messages_sender 
    ON messages(sender_id);

-- 部分索引（仅对活跃数据）
CREATE INDEX idx_users_active 
    ON users(username) 
    WHERE account_status = 'active';
```

### 6.2 迁移管理

#### 迁移文件命名

```
migrations/
├── 20251129000001_create_users_table.sql
├── 20251129000002_create_password_reset_tokens_table.sql
├── 20251130000001_add_username_to_audit_log.sql
└── 20251206000001_allow_ai_agent_as_sender.sql
```

#### 迁移编写规范

```sql
-- 1. 使用 IF NOT EXISTS 避免重复创建
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) NOT NULL UNIQUE,
    -- ...
);

-- 2. 添加注释
COMMENT ON TABLE users IS 'User accounts and profiles';
COMMENT ON COLUMN users.id IS 'Unique user identifier (UUID v4)';

-- 3. 创建触发器
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

---

## 7. 缓存策略

### 7.1 Redis 使用规范

#### 键命名规范

```rust
// 格式：{service}:{type}:{id}:{field?}
const USER_CACHE_KEY: &str = "user:profile:{user_id}";
const GROUP_MEMBERS_KEY: &str = "group:members:{group_id}";
const MESSAGE_CACHE_KEY: &str = "message:unread:{user_id}";
const AI_AGENT_CACHE_KEY: &str = "agent:personality:{agent_id}";
```

#### 缓存操作

```rust
use redis::AsyncCommands;

pub struct CacheService {
    client: redis::Client,
}

impl CacheService {
    // 设置缓存
    pub async fn set_user_profile(&self, user_id: Uuid, profile: &UserProfile) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("user:profile:{}", user_id);
        let value = serde_json::to_string(profile)?;
        
        conn.set_ex(&key, value, 3600).await?; // 1小时过期
        Ok(())
    }
    
    // 获取缓存
    pub async fn get_user_profile(&self, user_id: Uuid) -> Result<Option<UserProfile>> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("user:profile:{}", user_id);
        
        let value: Option<String> = conn.get(&key).await?;
        match value {
            Some(json) => {
                let profile: UserProfile = serde_json::from_str(&json)?;
                Ok(Some(profile))
            }
            None => Ok(None),
        }
    }
    
    // 删除缓存
    pub async fn invalidate_user(&self, user_id: Uuid) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let pattern = format!("user:*:{}", user_id);
        
        let keys: Vec<String> = conn.keys(&pattern).await?;
        if !keys.is_empty() {
            conn.del(&keys).await?;
        }
        Ok(())
    }
}
```

### 7.2 缓存策略

#### 1. Cache-Aside 模式

```rust
pub async fn get_user(&self, user_id: Uuid) -> Result<User> {
    // 1. 先从缓存获取
    if let Some(user) = self.cache.get_user(user_id).await? {
        return Ok(user);
    }
    
    // 2. 缓存未命中，从数据库获取
    let user = self.database.get_user(user_id).await?;
    
    // 3. 写入缓存
    self.cache.set_user(user_id, &user).await?;
    
    Ok(user)
}
```

#### 2. Write-Through 模式

```rust
pub async fn update_user(&self, user_id: Uuid, data: UpdateUser) -> Result<User> {
    // 1. 更新数据库
    let user = self.database.update_user(user_id, data).await?;
    
    // 2. 同步更新缓存
    self.cache.set_user(user_id, &user).await?;
    
    Ok(user)
}
```

---

## 8. 安全规范

### 8.1 认证和授权

#### JWT 实现

```rust
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};

pub struct TokenService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl TokenService {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
        }
    }
    
    pub fn generate_token(&self, claims: &Claims) -> Result<String> {
        let header = Header::default();
        encode(&header, claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(format!("Token generation failed: {}", e)))
    }
    
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|_| AppError::Unauthorized)?;
        
        Ok(token_data.claims)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // 用户ID
    pub exp: usize,         // 过期时间
    pub iat: usize,         // 签发时间
    pub role: String,       // 用户角色
}
```

#### 中间件实现

```rust
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn jwt_auth_middleware(
    State(token_service): State<Arc<TokenService>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. 提取令牌
    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(&header[7..])
            } else {
                None
            }
        });
    
    let token = match token {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };
    
    // 2. 验证令牌
    let claims = match token_service.validate_token(token) {
        Ok(claims) => claims,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };
    
    // 3. 将用户信息添加到请求扩展
    request.extensions_mut().insert(UserContext::new(claims));
    
    // 4. 继续处理请求
    Ok(next.run(request).await)
}
```

### 8.2 数据验证

#### 输入验证

```rust
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 50, message = "用户名长度必须在3-50字符之间"))]
    #[validate(regex(path = "USERNAME_REGEX", message = "用户名只能包含字母、数字和下划线"))]
    pub username: String,
    
    #[validate(email(message = "请输入有效的邮箱地址"))]
    pub email: String,
    
    #[validate(length(min = 8, message = "密码至少需要8字符"))]
    #[validate(custom(function = "validate_password_strength", message = "密码必须包含大小写字母和数字"))]
    pub password: String,
}

// 自定义验证函数
fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    
    if !(has_upper && has_lower && has_digit) {
        return Err(ValidationError::new("密码必须包含大小写字母和数字"));
    }
    
    Ok(())
}
```

#### SQL 注入防护

```rust
// ✅ 正确：使用参数化查询
pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE username = $1",
        username
    )
    .fetch_optional(&self.pool)
    .await?;
    
    Ok(user)
}

// ❌ 错误：字符串拼接（SQL注入风险）
pub async fn get_user_by_username_bad(&self, username: &str) -> Result<Option<User>> {
    let query = format!("SELECT * FROM users WHERE username = '{}'", username);
    let user = sqlx::query_as::<_, User>(&query)
        .fetch_optional(&self.pool)
        .await?;
    Ok(user)
}
```

### 8.3 速率限制

#### Redis 实现

```rust
use redis::AsyncCommands;

pub struct RateLimiter {
    client: redis::Client,
}

impl RateLimiter {
    pub async fn check_rate_limit(
        &self,
        key: &str,
        limit: u32,
        window: u64, // 窗口时间（秒）
    ) -> Result<bool> {
        let mut conn = self.client.get_async_connection().await?;
        
        // 使用滑动窗口算法
        let now = chrono::Utc::now().timestamp() as u64;
        let pipeline_start = now - window;
        
        // 清理过期记录
        conn.z_remrangebyscore(key, 0, pipeline_start).await?;
        
        // 获取当前窗口内的请求数
        let count: u64 = conn.z_card(key).await?;
        
        if count >= limit as u64 {
            return Ok(false); // 超出限制
        }
        
        // 记录当前请求
        conn.z_add(key, now, now).await?;
        conn.expire(key, window).await?;
        
        Ok(true)
    }
}

// 中间件使用
pub async fn rate_limit_middleware(
    State(rate_limiter): State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let key = format!("rate_limit:{}", get_client_ip(&request));
    
    if !rate_limiter.check_rate_limit(&key, 10, 60).await? {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    Ok(next.run(request).await)
}
```

---

## 9. 测试规范

### 9.1 单元测试

#### 测试组织

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    // 测试工具函数
    fn create_test_user() -> User {
        User {
            id: Uuid::new_v4(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            // ...
        }
    }
    
    #[tokio::test]
    async fn test_create_user_success() {
        // Arrange
        let pool = create_test_pool().await;
        let service = UserService::new(pool);
        let data = CreateUser {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        
        // Act
        let result = service.create_user(data).await;
        
        // Assert
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
    }
    
    #[tokio::test]
    async fn test_create_user_duplicate_username() {
        // Arrange
        let pool = create_test_pool().await;
        let service = UserService::new(pool);
        
        // 创建第一个用户
        let data1 = CreateUser {
            username: "testuser".to_string(),
            email: "test1@example.com".to_string(),
            password: "password123".to_string(),
        };
        service.create_user(data1).await.unwrap();
        
        // 尝试创建相同用户名的用户
        let data2 = CreateUser {
            username: "testuser".to_string(),
            email: "test2@example.com".to_string(),
            password: "password456".to_string(),
        };
        
        // Act
        let result = service.create_user(data2).await;
        
        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Conflict(_)));
    }
}
```

#### Mock 使用

```rust
use mockall::{mock, predicate::*};

mock! {
    pub EmailService {}
    
    #[async_trait]
    impl EmailServiceTrait for EmailService {
        async fn send_password_reset(&self, email: &str, token: &str) -> Result<()>;
    }
}

#[tokio::test]
async fn test_password_reset_flow() {
    // Arrange
    let mut mock_email = MockEmailService::new();
    mock_email
        .expect_send_password_reset()
        .with(eq("test@example.com"), always())
        .times(1)
        .returning(|_, _| Ok(()));
    
    let pool = create_test_pool().await;
    let service = AuthService::new(pool, Arc::new(mock_email));
    
    // Act
    let result = service.request_password_reset("test@example.com").await;
    
    // Assert
    assert!(result.is_ok());
}
```

### 9.2 集成测试

#### API 测试

```rust
use axum_test::TestServer;
use serde_json::json;

#[tokio::test]
async fn test_user_registration_api() {
    // Arrange
    let app = create_app().await;
    let server = TestServer::new(app).unwrap();
    
    let request_body = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });
    
    // Act
    let response = server
        .post("/api/auth/register")
        .json(&request_body)
        .await;
    
    // Assert
    assert_eq!(response.status_code(), 201);
    
    let body: ApiResponse<AuthResponseData> = response.json();
    assert!(body.success);
    assert!(body.data.is_some());
    
    let data = body.data.unwrap();
    assert_eq!(data.user.username, "testuser");
    assert!(data.access_token.len() > 0);
}
```

#### WebSocket 测试

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::test]
async fn test_websocket_connection() {
    // Arrange
    let app = create_app().await;
    let server = TestServer::new(app).unwrap();
    let ws_url = server.ws_url("/api/ws").replace("http", "ws");
    
    // Act
    let (ws_stream, _) = connect_async(&format!("{}?token={}", ws_url, TEST_TOKEN))
        .await
        .unwrap();
    
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    // 发送心跳
    ws_sender.send(Message::Text(json!({
        "type": "heartbeat",
        "data": { "timestamp": 1234567890 }
    }).to_string())).await.unwrap();
    
    // 接收响应
    let response = ws_receiver.next().await.unwrap().unwrap();
    
    // Assert
    match response {
        Message::Text(text) => {
            let msg: WsMessage = serde_json::from_str(&text).unwrap();
            assert!(matches!(msg, WsMessage::Heartbeat { .. }));
        }
        _ => panic!("Expected text message"),
    }
}
```

### 9.3 性能测试

#### 负载测试

```rust
use std::time::{Duration, Instant};
use tokio::task::JoinSet;

#[tokio::test]
async fn test_concurrent_message_sending() {
    // Arrange
    let pool = create_test_pool().await;
    let service = MessageService::new(pool);
    let concurrent_requests = 100;
    
    let mut set = JoinSet::new();
    let start = Instant::now();
    
    // Act
    for i in 0..concurrent_requests {
        let service = service.clone();
        set.spawn(async move {
            let message = CreateMessage {
                sender_id: Uuid::new_v4(),
                group_id: Uuid::new_v4(),
                content: format!("Test message {}", i),
                msg_type: MsgType::Text,
            };
            
            service.send_message(message).await
        });
    }
    
    // 等待所有请求完成
    let mut success_count = 0;
    while let Some(result) = set.join_next().await {
        match result.unwrap() {
            Ok(_) => success_count += 1,
            Err(e) => eprintln!("Request failed: {}", e),
        }
    }
    
    let duration = start.elapsed();
    
    // Assert
    assert_eq!(success_count, concurrent_requests);
    assert!(duration < Duration::from_secs(5));
    
    println!(
        "Processed {} requests in {:?} ({:.2} req/s)",
        concurrent_requests,
        duration,
        concurrent_requests as f64 / duration.as_secs_f64()
    );
}
```

---

## 10. 监控和日志

### 10.1 日志规范

#### 日志级别

```rust
use tracing::{debug, error, info, warn, instrument};

impl UserService {
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn get_user(&self, user_id: Uuid) -> Result<User> {
        debug!("Fetching user from database");
        
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Database error while fetching user {}: {}", user_id, e);
            AppError::Database(e.to_string())
        })?;
        
        match user {
            Some(user) => {
                info!("Successfully fetched user {}", user_id);
                Ok(user)
            }
            None => {
                warn!("User {} not found", user_id);
                Err(AppError::UserNotFound(user_id.to_string()))
            }
        }
    }
}
```

#### 结构化日志

```rust
use tracing::info;

#[instrument(skip(self))]
pub async fn process_message(&self, message: &ChatMessage) -> Result<()> {
    info!(
        message_id = %message.id,
        sender_id = %message.sender_id,
        group_id = %message.group_id,
        content_length = message.content.len(),
        "Processing message"
    );
    
    // 处理逻辑
    
    info!(
        message_id = %message.id,
        "Message processed successfully"
    );
    
    Ok(())
}
```

### 10.2 指标收集

#### Prometheus 集成

```rust
use prometheus::{Counter, Histogram, IntGauge, Registry};

pub struct Metrics {
    pub requests_total: Counter,
    pub request_duration: Histogram,
    pub active_connections: IntGauge,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            requests_total: Counter::new(
                "http_requests_total",
                "Total number of HTTP requests"
            ).unwrap(),
            request_duration: Histogram::with_opts(
                prometheus::HistogramOpts::new(
                    "http_request_duration_seconds",
                    "HTTP request duration in seconds"
                ).buckets(vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0])
            ).unwrap(),
            active_connections: IntGauge::new(
                "websocket_active_connections",
                "Number of active WebSocket connections"
            ).unwrap(),
        }
    }
    
    pub fn register(&self, registry: &Registry) {
        registry.register(Box::new(self.requests_total.clone())).unwrap();
        registry.register(Box::new(self.request_duration.clone())).unwrap();
        registry.register(Box::new(self.active_connections.clone())).unwrap();
    }
}

// 在中间件中使用
pub async fn metrics_middleware(
    State(metrics): State<Arc<Metrics>>,
    request: Request,
    next: Next,
) -> Response {
    let timer = metrics.request_duration.start_timer();
    let response = next.run(request).await;
    timer.observe_duration();
    
    metrics.requests_total.inc();
    
    response
}
```

---

## 11. 部署和运维

### 11.1 Docker 配置

#### Dockerfile

```dockerfile
# 构建阶段
FROM rust:1.91.1 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# 构建应用
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复制构建产物
COPY --from=builder /app/target/release/ai-im-server /app/ai-im-server
COPY --from=builder /app/migrations ./migrations

# 创建非 root 用户
RUN useradd -m -u 1000 appuser && chown -R appuser:appuser /app
USER appuser

# 暴露端口
EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# 启动命令
CMD ["./ai-im-server"]
```

#### docker-compose.yml

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://postgres:password@db:5432/ai_im
      - REDIS_URL=redis://redis:6379
      - QDRANT_URL=http://qdrant:6333
      - RUST_LOG=info
    depends_on:
      - db
      - redis
      - qdrant
    restart: unless-stopped

  db:
    image: postgres:16
    environment:
      - POSTGRES_DB=ai_im
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    restart: unless-stopped

  qdrant:
    image: qdrant/qdrant:latest
    ports:
      - "6333:6333"
    volumes:
      - qdrant_data:/qdrant/storage
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
  qdrant_data:
```

### 11.2 环境配置

#### 环境变量

```env
# 数据库配置
DATABASE_URL=postgresql://user:password@localhost:5432/ai_im
DATABASE_MAX_CONNECTIONS=20

# Redis 配置
REDIS_URL=redis://localhost:6379
REDIS_POOL_SIZE=10

# Qdrant 配置
QDRANT_URL=http://localhost:6333
QDRANT_API_KEY=

# JWT 配置
JWT_SECRET=your-super-secret-jwt-key-change-in-production
JWT_ACCESS_EXPIRY=900  # 15分钟
JWT_REFRESH_EXPIRY=604800  # 7天

# 服务器配置
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# 日志配置
RUST_LOG=info

# AI 服务配置
OPENAI_API_KEY=sk-xxx
OPENAI_MODEL=gpt-3.5-turbo

# 文件存储
STORAGE_TYPE=s3  # s3, local
S3_BUCKET=ai-im-files
S3_REGION=us-east-1
S3_ACCESS_KEY=xxx
S3_SECRET_KEY=xxx

# 媒体处理
MAX_IMAGE_SIZE=10485760  # 10MB
MAX_VIDEO_SIZE=104857600  # 100MB
```

### 11.3 监控配置

#### Prometheus 配置

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'ai-im-server'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 5s
```

#### Grafana 仪表板

```json
{
  "dashboard": {
    "title": "AI Native IM - Backend Metrics",
    "panels": [
      {
        "title": "HTTP Requests",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])",
            "legendFormat": "{{method}} {{endpoint}}"
          }
        ]
      },
      {
        "title": "Response Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          }
        ]
      },
      {
        "title": "WebSocket Connections",
        "type": "singlestat",
        "targets": [
          {
            "expr": "websocket_active_connections",
            "legendFormat": "Active Connections"
          }
        ]
      }
    ]
  }
}
```

---

## 12. 故障排查

### 12.1 常见问题

#### 1. 数据库连接失败

```
Error: Database error: connection refused
```

排查步骤：
1. 检查数据库服务是否运行
2. 验证连接字符串配置
3. 检查网络连通性
4. 查看数据库日志

#### 2. Redis 连接超时

```
Error: Redis error: Connection timed out
```

排查步骤：
1. 检查 Redis 服务状态
2. 验证 Redis URL 配置
3. 检查防火墙设置
4. 监控 Redis 内存使用

#### 3. AI 服务响应慢

```
Warning: AI service timeout
```

排查步骤：
1. 检查 API 密钥配置
2. 监控 API 调用频率
3. 检查网络延迟
4. 考虑使用本地模型

### 12.2 日志分析

#### 查看错误日志

```bash
# 查看最近的错误
tail -f /var/log/ai-im/error.log | grep ERROR

# 统计错误类型
grep ERROR /var/log/ai-im/error.log | awk '{print $5}' | sort | uniq -c

# 查看特定时间段
grep "2025-12-09 10:" /var/log/ai-im/error.log
```

#### 性能分析

```bash
# 查看慢查询
grep "slow query" /var/log/ai-im/postgresql.log

# 分析响应时间
grep "request_duration" /var/log/ai-im/access.log | awk '{print $NF}' | sort -n
```

---

## 13. 最佳实践

### 13.1 代码质量

1. **使用 cargo clippy**
   ```bash
   cargo clippy -- -D warnings
   ```

2. **运行 cargo fmt**
   ```bash
   cargo fmt --check
   ```

3. **编写测试**
   - 单元测试覆盖率 > 80%
   - 集成测试覆盖主要流程
   - 性能测试验证关键指标

### 13.2 性能优化

1. **数据库优化**
   - 使用索引加速查询
   - 避免 N+1 查询
   - 使用连接池

2. **缓存策略**
   - 热数据缓存
   - 合理设置过期时间
   - 缓存预热

3. **异步优化**
   - 使用 tokio 并发处理
   - 避免阻塞操作
   - 合理设置超时

### 13.3 安全加固

1. **认证授权**
   - 强密码策略
   - 定期轮换密钥
   - 最小权限原则

2. **数据保护**
   - 敏感数据加密
   - 传输层安全
   - 定期备份

3. **访问控制**
   - IP 白名单
   - 速率限制
   - 审计日志

---

## 14. 附录

### 14.1 推荐工具

- **IDE**: VS Code + rust-analyzer
- **API 测试**: Postman, Insomnia
- **数据库**: pgAdmin, DBeaver
- **监控**: Prometheus + Grafana
- **日志**: ELK Stack (Elasticsearch, Logstash, Kibana)

### 14.2 学习资源

- [Rust 官方文档](https://doc.rust-lang.org/)
- [Tokio 教程](https://tokio.rs/tokio/tutorial)
- [Axum 示例](https://github.com/tokio-rs/axum/tree/main/examples)
- [SQLx 指南](https://github.com/launchbadge/sqlx)

### 14.3 社区支持

- [Rust 用户论坛](https://users.rust-lang.org/)
- [Tokio Discord](https://discord.gg/tokio)
- [Axum GitHub](https://github.com/tokio-rs/axum)

---

## 结语

本白皮书为 AI Native IM 后端开发提供了全面的指导规范。遵循这些规范将确保代码质量、系统稳定性和团队协作效率。

随着项目的发展，本白皮书将持续更新，以反映最新的技术实践和经验总结。

**记住：优秀的代码不仅在于功能实现，更在于可维护性和可扩展性。**