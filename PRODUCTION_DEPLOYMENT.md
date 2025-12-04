# 🚀 生产部署指南

AI-Native IM 系统的完整生产部署流程。

## 📋 部署前检查清单

### ✅ 代码准备

- [x] 所有功能已完成 (306/306 任务)
- [x] 所有测试通过 (504/504 测试)
- [x] 代码审查完成
- [x] 文档完整
- [ ] 版本号确认 (v1.0.0)
- [ ] CHANGELOG 更新

### ✅ 安全检查

- [ ] 环境变量配置检查
- [ ] 密钥和证书准备
- [ ] 数据库密码强度
- [ ] JWT Secret 生成
- [ ] API Key 配置
- [ ] SSL/TLS 证书

### ✅ 性能验证

- [x] 负载测试通过
- [x] 压力测试通过
- [x] 性能指标达标
- [ ] 数据库索引优化
- [ ] 缓存策略配置

### ✅ 基础设施

- [ ] 服务器/云资源准备
- [ ] 域名配置
- [ ] CDN 配置
- [ ] 备份策略
- [ ] 监控告警配置

---

## 🔧 第一步：环境准备

### 1. 服务器要求

**最低配置**:
- CPU: 4 核
- 内存: 8GB
- 磁盘: 100GB SSD
- 网络: 100Mbps

**推荐配置**:
- CPU: 8 核
- 内存: 16GB
- 磁盘: 200GB SSD
- 网络: 1Gbps

### 2. 安装必需软件

```bash
# 更新系统
sudo apt update && sudo apt upgrade -y

# 安装 Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# 安装 Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# 验证安装
docker --version
docker-compose --version
```

### 3. 配置防火墙

```bash
# 允许 HTTP/HTTPS
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# 允许 SSH (如果需要)
sudo ufw allow 22/tcp

# 启用防火墙
sudo ufw enable
```

---

## 🔐 第二步：安全配置

### 1. 生成强密码和密钥

```bash
# 生成数据库密码
openssl rand -base64 32

# 生成 Redis 密码
openssl rand -base64 32

# 生成 JWT Secret (至少 32 字符)
openssl rand -base64 48

# 生成 Grafana 密码
openssl rand -base64 16
```

### 2. 配置环境变量

```bash
# 复制示例文件
cp .env.example .env

# 编辑配置
vim .env
```

**必须修改的配置**:
```env
# 数据库
DB_PASSWORD=<生成的强密码>
DATABASE_URL=postgresql://prod_user:${DB_PASSWORD}@postgres:5432/ai_im_prod

# Redis
REDIS_PASSWORD=<生成的强密码>
REDIS_URL=redis://:${REDIS_PASSWORD}@redis:6379

# JWT
JWT_SECRET=<生成的强密钥>

# Grafana
GRAFANA_PASSWORD=<生成的强密码>

# 服务器
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
RUST_LOG=info

# 存储 (根据需要选择)
STORAGE_PROVIDER=s3
AWS_REGION=us-east-1
AWS_ACCESS_KEY_ID=<你的 Access Key>
AWS_SECRET_ACCESS_KEY=<你的 Secret Key>
S3_BUCKET=<你的 Bucket 名称>

# AI (如果使用)
OPENAI_API_KEY=<你的 OpenAI Key>
# 或使用国产模型
SILICONFLOW_API_KEY=<你的 SiliconFlow Key>
```

### 3. 配置 SSL 证书

**方法 1: Let's Encrypt (推荐)**

```bash
# 安装 Certbot
sudo apt install certbot python3-certbot-nginx

# 获取证书
sudo certbot certonly --standalone -d yourdomain.com -d www.yourdomain.com

# 证书位置
# /etc/letsencrypt/live/yourdomain.com/fullchain.pem
# /etc/letsencrypt/live/yourdomain.com/privkey.pem

# 复制到项目
sudo cp /etc/letsencrypt/live/yourdomain.com/fullchain.pem nginx/ssl/cert.pem
sudo cp /etc/letsencrypt/live/yourdomain.com/privkey.pem nginx/ssl/key.pem
```

**方法 2: 自签名证书 (仅测试)**

```bash
mkdir -p nginx/ssl
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout nginx/ssl/key.pem \
  -out nginx/ssl/cert.pem \
  -subj "/CN=yourdomain.com"
```

### 4. 更新 Nginx 配置

编辑 `nginx/nginx.conf`，修改 `server_name`:

```nginx
server {
    listen 443 ssl http2;
    server_name yourdomain.com www.yourdomain.com;
    # ...
}
```

---

## 🚢 第三步：部署后端

### 1. 构建 Docker 镜像

```bash
# 构建后端镜像
docker build -t ai-im-server:v1.0.0 -f server/Dockerfile .

# 标记为 latest
docker tag ai-im-server:v1.0.0 ai-im-server:latest

# (可选) 推送到 Docker Hub
docker tag ai-im-server:v1.0.0 yourusername/ai-im-server:v1.0.0
docker push yourusername/ai-im-server:v1.0.0
```

### 2. 启动服务

```bash
# 使用生产配置启动
docker-compose -f docker-compose.prod.yml up -d

# 查看日志
docker-compose -f docker-compose.prod.yml logs -f

# 检查服务状态
docker-compose -f docker-compose.prod.yml ps
```

### 3. 运行数据库迁移

```bash
# 进入服务器容器
docker exec -it ai-im-server bash

# 运行迁移
sqlx migrate run

# 退出容器
exit
```

### 4. 验证部署

```bash
# 健康检查
curl https://yourdomain.com/health

# 应该返回
# {"status":"healthy"}

# 检查 API
curl https://yourdomain.com/api/health

# 检查 WebSocket
wscat -c wss://yourdomain.com/ws
```

---

## 📱 第四步：部署客户端

### Flutter Web 部署

```bash
cd client

# 构建 Web 版本
flutter build web --release

# 部署到服务器
# 方法 1: 使用 Nginx 托管
sudo cp -r build/web/* /var/www/html/

# 方法 2: 使用 CDN
# 上传 build/web/ 到 S3/CloudFront
```

### iOS 部署

```bash
# 1. 配置签名
# 在 Xcode 中配置 Team 和 Bundle ID

# 2. 构建
flutter build ios --release

# 3. 打开 Xcode
open ios/Runner.xcworkspace

# 4. 在 Xcode 中:
# - Product > Archive
# - Distribute App
# - 上传到 App Store Connect
```

### Android 部署

```bash
# 1. 生成签名密钥
keytool -genkey -v -keystore ~/upload-keystore.jks \
  -keyalg RSA -keysize 2048 -validity 10000 \
  -alias upload

# 2. 配置签名
# 编辑 android/key.properties
storePassword=<密码>
keyPassword=<密码>
keyAlias=upload
storeFile=<密钥文件路径>

# 3. 构建 APK
flutter build apk --release

# 4. 构建 App Bundle (推荐)
flutter build appbundle --release

# 5. 上传到 Google Play Console
# build/app/outputs/bundle/release/app-release.aab
```

### macOS 部署

```bash
# 构建
flutter build macos --release

# 创建 DMG
# 使用 create-dmg 工具
npm install -g create-dmg
create-dmg build/macos/Build/Products/Release/AI\ Native\ IM.app
```

---

## 📊 第五步：配置监控

### 1. 访问 Grafana

```bash
# 打开浏览器
https://yourdomain.com:3000

# 登录
用户名: admin
密码: <你在 .env 中设置的密码>
```

### 2. 导入仪表板

1. 点击 "+" → "Import"
2. 上传或粘贴仪表板 JSON
3. 选择 Prometheus 数据源
4. 点击 "Import"

### 3. 配置告警

编辑 `monitoring/prometheus.yml` 添加告警规则：

```yaml
groups:
  - name: ai-im-alerts
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors/sec"

      - alert: HighMemoryUsage
        expr: container_memory_usage_bytes / container_spec_memory_limit_bytes > 0.9
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value }}%"
```

---

## 🔄 第六步：配置备份

### 1. 创建备份脚本

```bash
# 使用项目提供的备份脚本
chmod +x scripts/backup.sh

# 测试备份
./scripts/backup.sh

# 检查备份文件
ls -lh backups/
```

### 2. 配置自动备份

```bash
# 编辑 crontab
crontab -e

# 添加定时任务 (每天凌晨 2 点)
0 2 * * * /path/to/ai-native-im/scripts/backup.sh

# 每周日凌晨 3 点清理旧备份
0 3 * * 0 find /path/to/backups -name "*.sql.gz" -mtime +30 -delete
```

### 3. 配置远程备份

```bash
# 安装 AWS CLI
sudo apt install awscli

# 配置 AWS
aws configure

# 上传备份到 S3
aws s3 sync backups/ s3://your-backup-bucket/ai-im-backups/
```

---

## 🔍 第七步：性能优化

### 1. 数据库优化

```sql
-- 连接到数据库
psql -h localhost -U prod_user -d ai_im_prod

-- 创建索引
CREATE INDEX CONCURRENTLY idx_messages_created_at ON messages(created_at DESC);
CREATE INDEX CONCURRENTLY idx_messages_conversation_id ON messages(conversation_id);
CREATE INDEX CONCURRENTLY idx_users_username ON users(username);

-- 分析表
ANALYZE messages;
ANALYZE users;
ANALYZE groups;

-- 查看慢查询
SELECT * FROM pg_stat_statements 
ORDER BY mean_exec_time DESC 
LIMIT 10;
```

### 2. Redis 优化

```bash
# 连接到 Redis
docker exec -it ai-im-redis redis-cli

# 设置最大内存
CONFIG SET maxmemory 2gb
CONFIG SET maxmemory-policy allkeys-lru

# 启用持久化
CONFIG SET appendonly yes
CONFIG SET save "900 1 300 10 60 10000"

# 保存配置
CONFIG REWRITE
```

### 3. Nginx 优化

编辑 `nginx/nginx.conf`:

```nginx
# 增加 worker 进程
worker_processes auto;

# 增加连接数
events {
    worker_connections 4096;
}

# 启用缓存
http {
    proxy_cache_path /var/cache/nginx levels=1:2 keys_zone=my_cache:10m max_size=1g inactive=60m;
    
    server {
        location /api/ {
            proxy_cache my_cache;
            proxy_cache_valid 200 5m;
        }
    }
}
```

---

## 🧪 第八步：生产验证

### 1. 功能测试

```bash
# 注册用户
curl -X POST https://yourdomain.com/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","email":"test@example.com","password":"Test123!"}'

# 登录
curl -X POST https://yourdomain.com/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"Test123!"}'

# 获取用户信息
curl https://yourdomain.com/api/users/me \
  -H "Authorization: Bearer <token>"
```

### 2. 性能测试

```bash
# 使用 Apache Bench
ab -n 1000 -c 100 https://yourdomain.com/health

# 使用 wrk
wrk -t12 -c400 -d30s https://yourdomain.com/health
```

### 3. 安全测试

```bash
# SSL 测试
curl -I https://yourdomain.com

# 检查 SSL 评级
# 访问 https://www.ssllabs.com/ssltest/

# 检查安全头
curl -I https://yourdomain.com | grep -i "security\|x-frame\|x-content"
```

---

## 📈 第九步：扩容策略

### 水平扩容

```bash
# Docker Compose 扩容
docker-compose -f docker-compose.prod.yml up -d --scale server=5

# Kubernetes 扩容
kubectl scale deployment/ai-im-server --replicas=10 -n ai-im
```

### 垂直扩容

编辑 `docker-compose.prod.yml`:

```yaml
services:
  server:
    deploy:
      resources:
        limits:
          cpus: '4'
          memory: 4G
        reservations:
          cpus: '2'
          memory: 2G
```

---

## 🚨 第十步：故障恢复

### 数据库恢复

```bash
# 停止服务
docker-compose -f docker-compose.prod.yml stop server

# 恢复数据库
gunzip < backups/postgres_20231204_020000.sql.gz | \
  docker exec -i ai-im-postgres psql -U prod_user ai_im_prod

# 重启服务
docker-compose -f docker-compose.prod.yml start server
```

### Redis 恢复

```bash
# 停止 Redis
docker-compose -f docker-compose.prod.yml stop redis

# 恢复数据
docker cp backups/redis_20231204_020000.rdb ai-im-redis:/data/dump.rdb

# 重启 Redis
docker-compose -f docker-compose.prod.yml start redis
```

### 回滚部署

```bash
# Docker Compose 回滚
docker-compose -f docker-compose.prod.yml down
docker-compose -f docker-compose.prod.yml up -d

# Kubernetes 回滚
kubectl rollout undo deployment/ai-im-server -n ai-im
```

---

## ✅ 部署完成检查

### 最终验证清单

- [ ] 所有服务运行正常
- [ ] 健康检查通过
- [ ] SSL 证书有效
- [ ] 监控正常工作
- [ ] 告警配置完成
- [ ] 备份正常运行
- [ ] 性能指标达标
- [ ] 客户端可以连接
- [ ] 文档已更新
- [ ] 团队已培训

### 性能指标验证

| 指标 | 目标 | 实际 | 状态 |
|------|------|------|------|
| API 响应时间 (P95) | < 200ms | ___ | [ ] |
| WebSocket 延迟 | < 100ms | ___ | [ ] |
| 并发连接数 | 1000+ | ___ | [ ] |
| 消息吞吐量 | 1000+ msg/s | ___ | [ ] |
| 错误率 | < 0.1% | ___ | [ ] |
| 可用性 | > 99.9% | ___ | [ ] |

---

## 📞 支持和维护

### 日常维护

```bash
# 查看日志
docker-compose -f docker-compose.prod.yml logs -f --tail=100

# 查看资源使用
docker stats

# 重启服务
docker-compose -f docker-compose.prod.yml restart server

# 更新服务
docker-compose -f docker-compose.prod.yml pull
docker-compose -f docker-compose.prod.yml up -d
```

### 监控指标

定期检查：
- CPU 使用率 < 70%
- 内存使用率 < 80%
- 磁盘使用率 < 80%
- 网络带宽充足
- 数据库连接池健康
- Redis 内存使用正常

### 获取帮助

- 📖 [完整文档](DOCS_INDEX.md)
- 🐛 [报告问题](https://github.com/your-repo/issues)
- 💬 [社区讨论](https://github.com/your-repo/discussions)
- 📧 联系支持团队

---

## 🎉 恭喜！

你已经成功部署 AI-Native IM 到生产环境！

**下一步**:
1. 监控系统运行状态
2. 收集用户反馈
3. 持续优化性能
4. 定期更新系统

**最后更新**: 2025-12-04  
**版本**: v1.0.0
