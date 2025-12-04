# ❓ 常见问题 (FAQ)

AI-Native IM 项目的常见问题解答。

## 目录

- [通用问题](#通用问题)
- [安装和配置](#安装和配置)
- [开发相关](#开发相关)
- [部署相关](#部署相关)
- [性能和优化](#性能和优化)
- [故障排查](#故障排查)

---

## 通用问题

### Q: 这个项目是做什么的？

A: AI-Native IM 是一个功能完整的即时通讯系统，核心特色是集成了 AI Agent，可以在群聊中智能参与对话。使用 Rust 后端 + Flutter 客户端构建，支持 6 个平台。

### Q: 项目的主要特色是什么？

A: 
- ⭐ **AI Agent 系统** - 支持多个 LLM 提供商，智能触发机制
- 🚀 **高性能** - Rust 后端，消息延迟 < 100ms
- 📱 **跨平台** - 支持 iOS/Android/macOS/Linux/Windows/Web
- 🔒 **安全可靠** - JWT 认证，完整的权限系统
- 🧪 **高质量** - 504 个测试，100% 通过率

### Q: 项目使用什么技术栈？

A:
- **后端**: Rust + Axum + PostgreSQL + Redis + Qdrant
- **客户端**: Flutter + Riverpod + Drift
- **部署**: Docker + Kubernetes + Nginx
- **监控**: Prometheus + Grafana

### Q: 项目是否开源？

A: 是的，项目完全开源。欢迎贡献代码！

### Q: 支持哪些 AI 模型？

A: 支持多个提供商：
- OpenAI (GPT-3.5, GPT-4)
- Anthropic (Claude)
- Google (Gemini)
- SiliconFlow (国产模型)
- 自定义 API 端点

---

## 安装和配置

### Q: 如何快速开始？

A: 三步开始：
```bash
# 1. 克隆项目
git clone <repository-url>

# 2. 启动开发环境
./start-dev.sh

# 3. 运行服务器
cd server && cargo run
```

详见 [README.md](README.md)

### Q: 需要什么前置条件？

A:
- **后端**: Rust 1.75+, Docker, PostgreSQL, Redis
- **客户端**: Flutter 3.16+
- **系统**: 4GB+ RAM, 20GB+ 磁盘空间

### Q: Docker 服务启动失败怎么办？

A: 检查以下几点：
```bash
# 1. 检查 Docker 是否运行
docker ps

# 2. 检查端口占用
sudo lsof -i :5432  # PostgreSQL
sudo lsof -i :6379  # Redis

# 3. 查看日志
docker-compose logs postgres
docker-compose logs redis

# 4. 重启服务
docker-compose restart
```

### Q: 如何配置环境变量？

A:
```bash
# 1. 复制示例文件
cp .env.example .env

# 2. 编辑配置
vim .env

# 3. 至少修改以下内容
# - DB_PASSWORD
# - REDIS_PASSWORD
# - JWT_SECRET
```

详见 [.env.example](.env.example)

### Q: 如何配置 AI API Key？

A: 在 `.env` 文件中添加：
```env
OPENAI_API_KEY=sk-your-key
ANTHROPIC_API_KEY=sk-ant-your-key
GOOGLE_API_KEY=your-key
SILICONFLOW_API_KEY=your-key
```

---

## 开发相关

### Q: 如何运行测试？

A:
```bash
# 后端快速测试（推荐）
cd server && make test-fast

# 客户端测试
cd client && flutter test

# 性能测试
cd server && cargo test --test load_tests
```

详见 [server/TESTING.md](server/TESTING.md)

### Q: 测试很慢怎么办？

A: 使用优化后的测试命令：
```bash
# 使用 nextest（5-10倍速度提升）
make test-fast

# 仅运行单元测试
make test-quick

# 监视模式（开发时）
make test-watch
```

### Q: 如何调试后端？

A:
```bash
# 1. 设置日志级别
export RUST_LOG=debug

# 2. 运行服务器
cargo run

# 3. 查看详细日志
# 日志会输出到控制台
```

### Q: 如何调试客户端？

A:
```bash
# 1. 启动调试模式
flutter run --debug

# 2. 使用 DevTools
flutter pub global activate devtools
flutter pub global run devtools

# 3. 查看日志
flutter logs
```

### Q: 如何添加新的 API 端点？

A:
1. 在 `server/src/api/` 添加路由
2. 在 `server/src/services/` 添加业务逻辑
3. 在 `server/src/models/` 添加数据模型
4. 添加测试
5. 更新 API 文档

示例见 [CONTRIBUTING.md](CONTRIBUTING.md)

### Q: 如何添加数据库迁移？

A:
```bash
cd server

# 创建迁移
sqlx migrate add create_new_table

# 编辑 migrations/xxx_create_new_table.sql

# 运行迁移
sqlx migrate run

# 回滚
sqlx migrate revert
```

---

## 部署相关

### Q: 如何部署到生产环境？

A: 使用 Docker Compose：
```bash
# 1. 配置环境变量
cp .env.example .env
vim .env

# 2. 生成 SSL 证书
mkdir -p nginx/ssl
# 使用 Let's Encrypt 或自签名证书

# 3. 启动服务
docker-compose -f docker-compose.prod.yml up -d
```

详见 [QUICK_DEPLOY.md](QUICK_DEPLOY.md)

### Q: 如何部署到 Kubernetes？

A:
```bash
# 1. 构建镜像
docker build -t ai-im-server:latest -f server/Dockerfile .

# 2. 创建 Secrets
kubectl create secret generic ai-im-secrets \
  --from-literal=database-url='...' \
  --from-literal=jwt-secret='...'

# 3. 部署
./scripts/deploy.sh production
```

详见 [DEPLOYMENT.md](DEPLOYMENT.md)

### Q: 如何配置 HTTPS？

A: 
**方法 1: Let's Encrypt**
```bash
# 使用 certbot
sudo certbot --nginx -d yourdomain.com
```

**方法 2: 自签名证书**
```bash
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout nginx/ssl/key.pem \
  -out nginx/ssl/cert.pem
```

### Q: 如何扩容服务？

A:
**Docker Compose**:
```bash
docker-compose up -d --scale server=5
```

**Kubernetes**:
```bash
kubectl scale deployment/ai-im-server --replicas=5 -n ai-im
```

### Q: 如何备份数据？

A:
```bash
# 使用备份脚本
./scripts/backup.sh

# 手动备份 PostgreSQL
docker exec ai-im-postgres pg_dump -U prod_user ai_im_prod > backup.sql

# 手动备份 Redis
docker exec ai-im-redis redis-cli SAVE
docker cp ai-im-redis:/data/dump.rdb backup.rdb
```

---

## 性能和优化

### Q: 系统能支持多少并发用户？

A: 性能测试结果：
- WebSocket 并发连接: 500+
- 消息吞吐量: 1000+ msg/sec
- Token 验证: 5000+ /sec
- API 响应时间: P99 < 100ms

单机可支持 1000+ 并发用户，通过水平扩展可支持更多。

### Q: 如何优化数据库性能？

A:
```sql
-- 1. 添加索引
CREATE INDEX idx_messages_user_id ON messages(user_id);
CREATE INDEX idx_messages_created_at ON messages(created_at);

-- 2. 分析慢查询
SELECT * FROM pg_stat_statements 
ORDER BY mean_exec_time DESC 
LIMIT 10;

-- 3. 优化查询
-- 使用 EXPLAIN ANALYZE 分析查询计划
```

### Q: 如何优化 Redis 性能？

A:
```bash
# 1. 设置最大内存
redis-cli CONFIG SET maxmemory 2gb

# 2. 设置淘汰策略
redis-cli CONFIG SET maxmemory-policy allkeys-lru

# 3. 启用持久化
redis-cli CONFIG SET appendonly yes
```

### Q: 消息延迟高怎么办？

A: 检查以下几点：
1. 网络延迟 - 使用 `ping` 测试
2. 数据库性能 - 检查慢查询
3. Redis 性能 - 检查内存使用
4. 服务器负载 - 使用 `htop` 查看
5. WebSocket 连接 - 检查连接数

### Q: 如何监控系统性能？

A: 使用 Prometheus + Grafana：
```bash
# 访问 Grafana
http://localhost:3000

# 查看指标
# - CPU/内存使用率
# - 请求速率和延迟
# - 数据库连接池
# - Redis 命中率
```

---

## 故障排查

### Q: 服务器无法启动

A:
```bash
# 1. 检查日志
cargo run 2>&1 | tee server.log

# 2. 检查端口占用
sudo lsof -i :8080

# 3. 检查数据库连接
psql -h localhost -U dev_user -d ai_im_dev

# 4. 检查环境变量
echo $DATABASE_URL
```

### Q: WebSocket 连接失败

A:
```bash
# 1. 检查 Nginx 配置
docker exec ai-im-nginx nginx -t

# 2. 查看 Nginx 日志
docker logs ai-im-nginx

# 3. 测试 WebSocket
wscat -c ws://localhost:8080/ws?token=<token>

# 4. 检查防火墙
sudo ufw status
```

### Q: 数据库连接失败

A:
```bash
# 1. 检查数据库状态
docker-compose ps postgres

# 2. 测试连接
psql -h localhost -U dev_user -d ai_im_dev

# 3. 检查密码
# 确保 .env 中的密码正确

# 4. 重启数据库
docker-compose restart postgres
```

### Q: Redis 连接失败

A:
```bash
# 1. 检查 Redis 状态
docker-compose ps redis

# 2. 测试连接
redis-cli ping

# 3. 查看日志
docker logs ai-im-redis

# 4. 重启 Redis
docker-compose restart redis
```

### Q: AI Agent 不响应

A: 检查以下几点：
1. API Key 是否正确配置
2. 网络是否可以访问 AI API
3. 触发条件是否满足
4. 查看服务器日志中的错误信息

```bash
# 测试 AI API
curl https://api.openai.com/v1/models \
  -H "Authorization: Bearer $OPENAI_API_KEY"
```

### Q: 媒体上传失败

A:
```bash
# 1. 检查存储配置
echo $STORAGE_PROVIDER

# 2. 检查文件大小限制
# 默认 100MB

# 3. 检查存储空间
df -h

# 4. 查看日志
docker logs ai-im-server | grep media
```

### Q: 测试失败

A:
```bash
# 1. 清理并重新构建
cargo clean
cargo build

# 2. 更新依赖
cargo update

# 3. 检查数据库
# 确保测试数据库可用

# 4. 运行单个测试
cargo test test_name -- --nocapture
```

---

## 更多帮助

### 找不到答案？

- 📖 查看 [完整文档](DOCS_INDEX.md)
- 🐛 [报告 Bug](https://github.com/your-repo/issues)
- 💬 加入讨论区
- 📧 联系维护者

### 相关文档

- [README.md](README.md) - 项目概览
- [DEV_SETUP.md](DEV_SETUP.md) - 开发环境
- [DEPLOYMENT.md](DEPLOYMENT.md) - 部署指南
- [API_REFERENCE.md](API_REFERENCE.md) - API 文档
- [CONTRIBUTING.md](CONTRIBUTING.md) - 贡献指南

---

**最后更新**: 2025-12-04
