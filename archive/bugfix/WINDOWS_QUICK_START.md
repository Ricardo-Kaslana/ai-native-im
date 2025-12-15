# Windows 快速开始指南 🚀

## 最简单的部署方式（推荐新手）

### 方案 A: 使用 Docker Desktop（最简单）

1. **安装 Docker Desktop**
   - 下载: https://www.docker.com/products/docker-desktop/
   - 安装并启动 Docker Desktop

2. **安装 Rust**
   - 下载: https://www.rust-lang.org/tools/install
   - 运行安装程序，重启终端

3. **启动所有服务**
   ```powershell
   # 在项目根目录运行
   docker compose up -d
   ```

4. **运行应用**
   ```powershell
   cd server
   sqlx migrate run
   cargo run
   ```

✅ 完成！服务器运行在 http://localhost:8080

---

### 方案 B: 原生安装（更快，但步骤多）

#### 1. 安装必需软件

**Rust** (必需)
```powershell
# 访问 https://www.rust-lang.org/tools/install
# 下载并运行 rustup-init.exe
# 重启 PowerShell
```

**PostgreSQL** (必需)
```powershell
# 下载: https://www.postgresql.org/download/windows/
# 安装时设置密码: postgres123
# 记住这个密码！
```

**Redis** (必需)
```powershell
# 方法 1: 下载 Memurai (推荐)
# https://www.memurai.com/get-memurai

# 方法 2: 下载 Redis for Windows
# https://github.com/tporadowski/redis/releases
# 解压到 C:\Redis
```

**Qdrant** (必需)
```powershell
# 使用 Docker (最简单)
docker run -d -p 6333:6333 -p 6334:6334 --name qdrant qdrant/qdrant

# 或下载二进制文件
# https://github.com/qdrant/qdrant/releases
```

#### 2. 配置数据库

打开 PowerShell 或 CMD：

```powershell
# 连接到 PostgreSQL (输入安装时设置的密码)
psql -U postgres

# 在 psql 中执行:
CREATE USER dev_user WITH PASSWORD 'dev_password';
CREATE DATABASE ai_im_dev OWNER dev_user;
GRANT ALL PRIVILEGES ON DATABASE ai_im_dev TO dev_user;
\q
```

#### 3. 启动服务

```powershell
# 启动 Redis (如果使用 Redis for Windows)
C:\Redis\redis-server.exe

# 启动 Qdrant (如果使用二进制)
C:\Qdrant\qdrant.exe

# 或使用 Docker
docker start qdrant
```

#### 4. 检查环境

```powershell
# 运行环境检查脚本
.\check-env.ps1
```

#### 5. 运行应用

```powershell
# 使用自动启动脚本
.\start-windows.ps1

# 或手动启动
cd server
sqlx migrate run
cargo run
```

---

## 🧪 测试部署

### 快速测试

```powershell
cd server
cargo test --lib --tests -- --test-threads=1
```

### 测试 API

```powershell
# 健康检查
curl http://localhost:8080/health

# 注册用户
Invoke-RestMethod -Uri "http://localhost:8080/auth/register" `
  -Method POST `
  -ContentType "application/json" `
  -Body '{"username":"test","email":"test@example.com","password":"Test123456"}'

# 登录
Invoke-RestMethod -Uri "http://localhost:8080/auth/login" `
  -Method POST `
  -ContentType "application/json" `
  -Body '{"email":"test@example.com","password":"Test123456"}'
```

---

## 🔧 常见问题速查

### PostgreSQL 连接失败

```powershell
# 检查服务是否运行
Get-Service | Where-Object {$_.Name -like "*postgres*"}

# 启动服务
Start-Service postgresql-x64-xx  # 替换 xx 为版本号

# 测试连接
psql -U dev_user -d ai_im_dev -h localhost
```

### Redis 连接失败

```powershell
# 检查端口
netstat -ano | findstr :6379

# 启动 Redis
C:\Redis\redis-server.exe

# 测试连接
redis-cli ping
```

### Qdrant 连接失败

```powershell
# 使用 Docker 启动
docker run -d -p 6333:6333 -p 6334:6334 --name qdrant qdrant/qdrant

# 检查状态
docker ps | findstr qdrant

# 测试连接
curl http://localhost:6333/healthz
```

### 端口被占用

```powershell
# 查找占用端口的进程
netstat -ano | findstr :8080

# 结束进程 (替换 <PID>)
taskkill /PID <PID> /F
```

### Rust 编译错误

```powershell
# 安装 Visual Studio Build Tools
# https://visualstudio.microsoft.com/downloads/
# 选择 "Desktop development with C++"
```

---

## 📝 配置文件说明

`server/.env` 文件已经配置好了，主要配置：

```env
# 数据库连接
DATABASE_URL=postgresql://dev_user:dev_password@localhost:5432/ai_im_dev

# Redis 连接
REDIS_URL=redis://localhost:6379

# Qdrant 连接
QDRANT_URL=http://localhost:6333

# AI 配置 (使用国内可用的 SiliconFlow)
SILICONFLOW_API_KEY=<REDACTED>
```

---

## 🎯 验证部署成功

运行以下命令，全部显示 ✅ 表示成功：

```powershell
.\check-env.ps1
```

预期输出：
```
✅ 已安装的工具:
  - Rust:          rustc 1.xx.x
  - Cargo:         cargo 1.xx.x

✅ 服务连接测试:
  - PostgreSQL:    ✅ 连接成功
  - Redis:         ✅ 连接成功
  - Qdrant:        ✅ 连接成功

✅ 所有服务运行正常！
```

---

## 🚀 下一步

1. ✅ 服务器运行成功
2. 📖 查看 API 文档: [API_REFERENCE.md](API_REFERENCE.md)
3. 🧪 运行完整测试: `cargo test`
4. 📱 运行 Flutter 客户端: 查看 [CLIENT_QUICK_START.md](CLIENT_QUICK_START.md)

---

## 💡 实用脚本

### 一键启动所有服务

```powershell
.\start-windows.ps1
```

### 仅运行测试

```powershell
.\start-windows.ps1 -TestOnly
```

### 跳过服务检查（服务已启动）

```powershell
.\start-windows.ps1 -SkipServices
```

---

## 🆘 需要帮助？

1. 查看详细文档: [WINDOWS_SETUP.md](WINDOWS_SETUP.md)
2. 查看常见问题: [FAQ.md](FAQ.md)
3. 查看项目状态: [PROJECT_STATUS.md](PROJECT_STATUS.md)

---

## 📊 性能优化建议

### 加速 Cargo 编译

创建 `%USERPROFILE%\.cargo\config.toml`：

```toml
[source.crates-io]
replace-with = "aliyun"

[source.aliyun]
registry = "sparse+https://mirrors.aliyun.com/crates.io-index/"

[build]
jobs = 4  # 根据 CPU 核心数调整
```

### 使用 cargo-watch 自动重新编译

```powershell
cargo install cargo-watch
cargo watch -x run
```

---

**祝你部署顺利！** 🎉
