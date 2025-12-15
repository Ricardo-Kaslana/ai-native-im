# Windows 本地部署指南

## 📋 前置要求

### 必需软件

1. **Rust** (最新稳定版)
   - 下载: https://www.rust-lang.org/tools/install
   - 安装后重启终端

2. **PostgreSQL** (14+)
   - 下载: https://www.postgresql.org/download/windows/
   - 安装时记住密码，默认用户是 `postgres`

3. **Redis** (Windows 版本)
   - 下载: https://github.com/tporadowski/redis/releases
   - 或使用 Memurai: https://www.memurai.com/get-memurai

4. **Qdrant** (向量数据库)
   - 下载: https://github.com/qdrant/qdrant/releases
   - 或使用 Docker Desktop

5. **Git** (可选但推荐)
   - 下载: https://git-scm.com/download/win

### 可选软件

- **Visual Studio Build Tools** (Rust 编译需要)
  - 下载: https://visualstudio.microsoft.com/downloads/
  - 选择 "C++ build tools"

- **Flutter** (如果要运行客户端)
  - 下载: https://flutter.dev/docs/get-started/install/windows

## 🚀 快速部署步骤

### 步骤 1: 安装 PostgreSQL

1. 下载并安装 PostgreSQL
2. 安装时设置密码（例如：`postgres123`）
3. 确保 PostgreSQL 服务已启动

4. 创建数据库和用户：

打开 PowerShell 或 CMD，运行：

```powershell
# 使用 psql 连接（需要输入安装时设置的密码）
psql -U postgres

# 在 psql 中执行以下命令：
CREATE USER dev_user WITH PASSWORD 'dev_password';
CREATE DATABASE ai_im_dev OWNER dev_user;
GRANT ALL PRIVILEGES ON DATABASE ai_im_dev TO dev_user;
\q
```

### 步骤 2: 安装 Redis

**方法 A: 使用 Redis for Windows**

1. 下载 Redis: https://github.com/tporadowski/redis/releases
2. 解压到 `C:\Redis`
3. 运行 `redis-server.exe`

**方法 B: 使用 Memurai (推荐)**

1. 下载: https://www.memurai.com/get-memurai
2. 安装并启动服务

### 步骤 3: 安装 Qdrant

**方法 A: 使用预编译二进制**

1. 下载: https://github.com/qdrant/qdrant/releases
2. 解压并运行 `qdrant.exe`

**方法 B: 使用 Docker Desktop (推荐)**

```powershell
docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant
```

### 步骤 4: 配置环境变量

编辑 `server/.env` 文件（已经配置好了，检查一下）：

```env
DATABASE_URL=postgresql://dev_user:dev_password@localhost:5432/ai_im_dev?sslmode=disable
REDIS_URL=redis://localhost:6379
QDRANT_URL=http://localhost:6333

JWT_SECRET=<REDACTED>
JWT_ACCESS_EXPIRY=900
JWT_REFRESH_EXPIRY=604800

# AI 配置（使用 SiliconFlow，国内可用）
SILICONFLOW_API_KEY=<REDACTED>
SILICONFLOW_MODEL=Qwen/Qwen2.5-7B-Instruct
SILICONFLOW_EMBEDDING_MODEL=BAAI/bge-large-zh-v1.5
SILICONFLOW_VISION_MODEL=Qwen/Qwen2-VL-7B-Instruct

SERVER_HOST=0.0.0.0
SERVER_PORT=8080
RUST_LOG=debug
```

### 步骤 5: 安装 SQLx CLI

```powershell
cargo install sqlx-cli --no-default-features --features postgres
```

### 步骤 6: 运行数据库迁移

```powershell
cd server
sqlx migrate run
```

### 步骤 7: 编译并运行服务器

```powershell
cd server
cargo build
cargo run
```

服务器将运行在 `http://localhost:8080`

## 🧪 测试部署

### 运行测试

```powershell
cd server
cargo test --lib --tests
```

### 快速测试（推荐）

```powershell
cd server
cargo test --lib --tests -- --test-threads=1
```

### 测试 API

使用 PowerShell 或 curl：

```powershell
# 注册用户
Invoke-RestMethod -Uri "http://localhost:8080/auth/register" -Method POST -ContentType "application/json" -Body '{"username":"testuser","email":"test@example.com","password":"Test123456"}'

# 登录
Invoke-RestMethod -Uri "http://localhost:8080/auth/login" -Method POST -ContentType "application/json" -Body '{"email":"test@example.com","password":"Test123456"}'
```

## 🔧 常见问题

### 1. PostgreSQL 连接失败

**问题**: `connection refused` 或 `password authentication failed`

**解决**:
- 检查 PostgreSQL 服务是否运行：`services.msc` 查找 `postgresql-x64-xx`
- 检查 `pg_hba.conf` 文件（通常在 `C:\Program Files\PostgreSQL\xx\data\`）
- 确保有这一行：`host all all 127.0.0.1/32 md5`

### 2. Redis 连接失败

**问题**: `Connection refused`

**解决**:
- 确保 Redis 服务正在运行
- 检查端口 6379 是否被占用：`netstat -ano | findstr :6379`

### 3. Qdrant 连接失败

**问题**: `Connection refused`

**解决**:
- 确保 Qdrant 正在运行
- 访问 http://localhost:6333/dashboard 检查

### 4. Rust 编译错误

**问题**: `linker 'link.exe' not found`

**解决**:
- 安装 Visual Studio Build Tools
- 或安装完整的 Visual Studio Community

### 5. 端口被占用

**问题**: `Address already in use`

**解决**:
```powershell
# 查找占用端口的进程
netstat -ano | findstr :8080

# 结束进程（替换 PID）
taskkill /PID <PID> /F
```

## 📝 Windows 特定优化

### 1. 加速 Cargo 编译

创建或编辑 `%USERPROFILE%\.cargo\config.toml`：

```toml
[source.crates-io]
replace-with = "aliyun"

[source.aliyun]
registry = "sparse+https://mirrors.aliyun.com/crates.io-index/"

[build]
jobs = 4  # 根据 CPU 核心数调整

[net]
git-fetch-with-cli = true
```

### 2. 设置环境变量

在 PowerShell 中：

```powershell
$env:RUST_LOG = "debug"
$env:DATABASE_URL = "postgresql://dev_user:dev_password@localhost:5432/ai_im_dev"
```

或永久设置（系统属性 → 环境变量）

## 🎯 验证部署成功

运行以下命令检查所有服务：

```powershell
# 检查 PostgreSQL
psql -U dev_user -d ai_im_dev -c "SELECT 1;"

# 检查 Redis
redis-cli ping

# 检查 Qdrant
curl http://localhost:6333/healthz

# 检查服务器
curl http://localhost:8080/health
```

## 🚀 启动脚本

创建 `start-windows.ps1`：

```powershell
# 启动所有服务
Write-Host "启动 Redis..." -ForegroundColor Green
Start-Process "C:\Redis\redis-server.exe"

Write-Host "启动 Qdrant..." -ForegroundColor Green
Start-Process "C:\Qdrant\qdrant.exe"

Write-Host "等待服务启动..." -ForegroundColor Yellow
Start-Sleep -Seconds 3

Write-Host "启动应用服务器..." -ForegroundColor Green
cd server
cargo run
```

## 📚 下一步

1. ✅ 服务器运行成功
2. 📱 运行 Flutter 客户端（可选）
3. 🧪 运行完整测试套件
4. 📖 查看 API 文档：[API_REFERENCE.md](API_REFERENCE.md)

## 💡 提示

- 使用 PowerShell 7+ 获得更好的体验
- 考虑使用 Windows Terminal
- 开发时使用 `cargo watch` 自动重新编译
- 使用 Postman 或 Insomnia 测试 API

## 🆘 获取帮助

如果遇到问题：
1. 查看日志：`RUST_LOG=debug cargo run`
2. 检查服务状态
3. 查看 [FAQ.md](FAQ.md)
