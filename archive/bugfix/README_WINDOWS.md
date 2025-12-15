# Windows 本地部署 - 3 分钟快速开始 ⚡

## 🎯 最快部署方式

### 前提条件（必须安装）

1. **Rust** - https://www.rust-lang.org/tools/install
2. **PostgreSQL** - https://www.postgresql.org/download/windows/
3. **Redis** - https://github.com/tporadowski/redis/releases 或 https://www.memurai.com/
4. **Qdrant** - 使用 Docker 或下载二进制文件

---

## 🚀 三步部署

### 第 1 步：安装软件（首次）

#### 1.1 安装 Rust
```
访问: https://www.rust-lang.org/tools/install
下载并运行 rustup-init.exe
重启终端
```

#### 1.2 安装 PostgreSQL
```
访问: https://www.postgresql.org/download/windows/
下载并安装
安装时设置密码: postgres123 (记住这个密码)
```

创建数据库：
```cmd
psql -U postgres
```

在 psql 中执行：
```sql
CREATE USER dev_user WITH PASSWORD 'dev_password';
CREATE DATABASE ai_im_dev OWNER dev_user;
GRANT ALL PRIVILEGES ON DATABASE ai_im_dev TO dev_user;
\q
```

#### 1.3 安装 Redis

**方法 A: Memurai (推荐)**
```
访问: https://www.memurai.com/get-memurai
下载并安装
自动作为 Windows 服务运行
```

**方法 B: Redis for Windows**
```
访问: https://github.com/tporadowski/redis/releases
下载最新版本
解压到 C:\Redis
运行: C:\Redis\redis-server.exe
```

#### 1.4 安装 Qdrant

**方法 A: Docker (推荐)**
```cmd
docker run -d -p 6333:6333 -p 6334:6334 --name qdrant qdrant/qdrant
```

**方法 B: 二进制文件**
```
访问: https://github.com/qdrant/qdrant/releases
下载 Windows 版本
解压并运行 qdrant.exe
```

---

### 第 2 步：检查环境

双击运行：
```
check-env.bat
```

或在命令行：
```cmd
check-env.bat
```

确保所有服务显示 ✅

---

### 第 3 步：启动应用

双击运行：
```
start-windows.bat
```

或在命令行：
```cmd
start-windows.bat
```

✅ 完成！服务器运行在 http://localhost:8080

---

## 🧪 测试部署

### 测试 API

打开新的命令行窗口：

```cmd
REM 健康检查
curl http://localhost:8080/health

REM 注册用户
curl -X POST http://localhost:8080/auth/register ^
  -H "Content-Type: application/json" ^
  -d "{\"username\":\"test\",\"email\":\"test@example.com\",\"password\":\"Test123456\"}"

REM 登录
curl -X POST http://localhost:8080/auth/login ^
  -H "Content-Type: application/json" ^
  -d "{\"email\":\"test@example.com\",\"password\":\"Test123456\"}"
```

### 运行测试

```cmd
cd server
cargo test --lib --tests
```

---

## 🔧 常见问题

### PostgreSQL 连接失败

```cmd
REM 检查服务
services.msc
REM 找到 postgresql-x64-xx 并启动

REM 测试连接
set PGPASSWORD=dev_password
psql -U dev_user -d ai_im_dev -h localhost
```

### Redis 连接失败

```cmd
REM 启动 Redis
C:\Redis\redis-server.exe

REM 测试连接
redis-cli ping
```

### Qdrant 连接失败

```cmd
REM 使用 Docker
docker start qdrant

REM 或运行二进制
C:\Qdrant\qdrant.exe

REM 测试连接
curl http://localhost:6333/healthz
```

### 端口被占用

```cmd
REM 查找占用端口的进程
netstat -ano | findstr :8080

REM 结束进程 (替换 <PID>)
taskkill /PID <PID> /F
```

### Rust 编译错误 (linker not found)

安装 Visual Studio Build Tools:
```
访问: https://visualstudio.microsoft.com/downloads/
下载 "Build Tools for Visual Studio"
选择 "Desktop development with C++"
```

---

## 📝 手动启动（如果脚本不工作）

### 1. 启动所有服务

```cmd
REM 启动 PostgreSQL (Windows 服务，通常自动启动)
REM 在 services.msc 中检查

REM 启动 Redis
start C:\Redis\redis-server.exe

REM 启动 Qdrant
docker start qdrant
REM 或
start C:\Qdrant\qdrant.exe
```

### 2. 运行数据库迁移

```cmd
cd server
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run
```

### 3. 启动服务器

```cmd
cargo run
```

---

## 🎯 验证成功

运行 `check-env.bat`，应该看到：

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

## 📚 更多文档

- **详细部署指南**: [WINDOWS_SETUP.md](WINDOWS_SETUP.md)
- **快速开始**: [WINDOWS_QUICK_START.md](WINDOWS_QUICK_START.md)
- **API 文档**: [API_REFERENCE.md](API_REFERENCE.md)
- **常见问题**: [FAQ.md](FAQ.md)

---

## 💡 提示

### 加速编译

创建 `%USERPROFILE%\.cargo\config.toml`:

```toml
[source.crates-io]
replace-with = "aliyun"

[source.aliyun]
registry = "sparse+https://mirrors.aliyun.com/crates.io-index/"
```

### 自动重新编译

```cmd
cargo install cargo-watch
cargo watch -x run
```

### 使用更好的终端

- 安装 Windows Terminal: https://aka.ms/terminal
- 或使用 PowerShell 7: https://github.com/PowerShell/PowerShell

---

## 🎉 成功！

现在你可以：

1. ✅ 访问 API: http://localhost:8080
2. 📖 查看 API 文档
3. 🧪 运行测试
4. 📱 启动 Flutter 客户端

**祝你开发愉快！** 🚀
