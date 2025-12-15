# 开发环境配置完成 ✅

## 已安装的工具

| 工具 | 版本 | 状态 |
|------|------|------|
| Rust | 1.91.1 | ✅ |
| Cargo | 1.91.1 | ✅ (已配置阿里云镜像) |
| Docker | 29.1.1 | ✅ |
| Docker Compose | v2.40.3 | ✅ |
| Flutter | 3.24.5 | ✅ |
| SQLx CLI | 0.8.6 | ✅ |
| Protoc | 3.21.12 | ✅ |
| PostgreSQL Client | 16 | ✅ |
| Redis Tools | 7.0.15 | ✅ |
| Git | 2.43.0 | ✅ |

## 运行中的服务

| 服务 | 端口 | 状态 |
|------|------|------|
| PostgreSQL | 5432 | ✅ 运行中 |
| Redis | 6379 | ✅ 运行中 |
| Qdrant | 6333, 6334 | ✅ 运行中 |

## 数据库状态

- ✅ PostgreSQL 连接成功
- ✅ Redis 连接成功
- ✅ Qdrant 连接成功
- ✅ 数据库迁移已完成 (9 个迁移)

## Cargo 配置

已配置阿里云镜像源，加速依赖下载：
```toml
[source.crates-io]
replace-with = "aliyun"

[source.aliyun]
registry = "sparse+https://mirrors.aliyun.com/crates.io-index/"
```

## 快速开始

### 1. 检查环境
```bash
./check-env.sh
```

### 2. 启动开发服务器
```bash
cd server
cargo run
```

服务器将运行在 `http://localhost:8080`

### 3. 运行测试 (后端)
```bash
cd server
cargo test --lib --tests
```

### 4. 启动客户端
```bash
cd client

# 获取依赖
flutter pub get

# 生成代码 (JSON序列化等)
dart run build_runner build --delete-conflicting-outputs

# 运行应用 (Windows)
flutter run -d windows

# 运行应用 (Chrome)
flutter run -d chrome

# 代码规范检查
flutter analyze lib/
```

### 5. 编译检查 (后端)
```bash
cd server
cargo check
```

### 6. 查看 Docker 日志
```bash
# 查看所有服务日志
sudo docker compose logs -f

# 查看特定服务日志
sudo docker compose logs -f postgres
sudo docker compose logs -f redis
sudo docker compose logs -f qdrant
```

### 7. 管理 Docker 服务
```bash
# 启动所有服务
sudo docker compose up -d

# 停止所有服务
sudo docker compose down

# 重启服务
sudo docker compose restart

# 查看服务状态
sudo docker compose ps
```

## 数据库管理

### 连接 PostgreSQL
```bash
PGPASSWORD=dev_password psql -h localhost -U dev_user -d ai_im_dev
```

### 连接 Redis
```bash
redis-cli
```

### 运行数据库迁移
```bash
cd server
sqlx migrate run
```

### 回滚迁移
```bash
cd server
sqlx migrate revert
```

## 项目结构

```
ai-native-im/
├── server/              # Rust 后端
│   ├── src/            # 源代码
│   ├── tests/          # 测试
│   ├── migrations/     # 数据库迁移
│   ├── Cargo.toml      # 依赖配置
│   └── .env            # 环境变量
├── client/             # Flutter 客户端
│   ├── lib/src/        # 源代码
│   │   ├── core/       # 核心基础设施
│   │   ├── features/   # 功能模块 (auth/chat等)
│   │   ├── shared/     # 共享组件
│   │   └── generated/  # 生成代码
│   ├── test/           # 测试
│   └── pubspec.yaml    # 依赖配置
├── protos/             # Protocol Buffers 定义
├── archive/docs/       # 核心文档 (API/错误码等)
├── docker-compose.dev.yml  # Docker 配置
└── changelogs/         # 更新日志
```

## 环境变量

环境变量配置在 `server/.env`：

```env
# 数据库
DATABASE_URL=postgresql://dev_user:dev_password@localhost:5432/ai_im_dev

# Redis
REDIS_URL=redis://localhost:6379

# Qdrant
QDRANT_URL=http://localhost:6333

# JWT
JWT_SECRET=your-super-secret-jwt-key-change-in-production
JWT_ACCESS_EXPIRY=900  # 15 分钟
JWT_REFRESH_EXPIRY=604800  # 7 天

# LLM API
OPENAI_API_KEY=<REDACTED>
OPENAI_MODEL=deepseek-chat

# 服务器
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
RUST_LOG=debug
```

## 常见问题

### Docker 权限问题
如果遇到 Docker 权限问题，运行：
```bash
sudo usermod -aG docker $USER
newgrp docker
```

### 端口被占用
如果端口被占用，检查并停止占用端口的进程：
```bash
# 检查端口占用
sudo lsof -i :5432  # PostgreSQL
sudo lsof -i :6379  # Redis
sudo lsof -i :6333  # Qdrant
sudo lsof -i :8080  # 应用服务器
```

### 数据库连接失败
确保 Docker 容器正在运行：
```bash
sudo docker compose ps
```

如果容器未运行，启动它们：
```bash
sudo docker compose up -d
```

## 下一步

1. ✅ 环境配置完成
2. ✅ 数据库迁移完成
3. ✅ 项目编译成功
4. ✅ 客户端基础架构完成
5. 🚀 开始开发！

查看 [CLIENT_DEVELOPMENT_PLAN.md](CLIENT_DEVELOPMENT_PLAN.md) 了解客户端开发计划。
查看 [archive/docs/PROJECT_OVERVIEW.md](archive/docs/PROJECT_OVERVIEW.md) 了解项目整体架构。
