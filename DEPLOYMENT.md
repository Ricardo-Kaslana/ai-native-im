# 🚀 部署指南

本文档提供 AI-Native IM 系统的完整部署指南。

## 目录

- [快速开始](#快速开始)
- [Docker Compose 部署](#docker-compose-部署)
- [Kubernetes 部署](#kubernetes-部署)
- [监控配置](#监控配置)
- [备份与恢复](#备份与恢复)
- [故障排查](#故障排查)

---

## 快速开始

### 前置要求

- Docker 20.10+
- Docker Compose 2.0+
- 4GB+ RAM
- 20GB+ 磁盘空间

### 开发环境

```bash
# 启动开发环境
./start-dev.sh

# 停止开发环境
./stop-dev.sh
```

---

## Docker Compose 部署

### 生产环境部署

1. **创建环境变量文件**

```bash
cp .env.example .env
```

编辑 `.env` 文件：

```env
# Database
DB_PASSWORD=your_secure_password_here

# Redis
REDIS_PASSWORD=your_redis_password_here

# JWT
JWT_SECRET=your_jwt_secret_key_here

# Grafana
GRAFANA_PASSWORD=your_grafana_password_here

# Email (可选)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your_email@gmail.com
SMTP_PASSWORD=your_email_password

# Storage (可选)
STORAGE_PROVIDER=s3
AWS_REGION=us-east-1
AWS_ACCESS_KEY_ID=your_access_key
AWS_SECRET_ACCESS_KEY=your_secret_key
S3_BUCKET=your-bucket-name
```

2. **生成 SSL 证书**

```bash
# 自签名证书（开发/测试）
mkdir -p nginx/ssl
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout nginx/ssl/key.pem \
  -out nginx/ssl/cert.pem

# 生产环境建议使用 Let's Encrypt
```

3. **启动服务**

```bash
docker-compose -f docker-compose.prod.yml up -d
```

4. **验证部署**

```bash
# 检查服务状态
docker-compose -f docker-compose.prod.yml ps

# 查看日志
docker-compose -f docker-compose.prod.yml logs -f server

# 健康检查
curl http://localhost:8080/health
```

### 服务访问

- **API**: https://localhost:443
- **Grafana**: http://localhost:3000 (admin/your_password)
- **Prometheus**: http://localhost:9090

---

## Kubernetes 部署

### 前置要求

- Kubernetes 1.24+
- kubectl 配置完成
- Helm 3.0+ (可选)

### 部署步骤

1. **创建 Secrets**

```bash
# 创建数据库密码
kubectl create secret generic ai-im-secrets \
  --from-literal=database-url='postgresql://prod_user:PASSWORD@postgres-service:5432/ai_im_prod' \
  --from-literal=jwt-secret='YOUR_JWT_SECRET' \
  -n ai-im

# 创建 TLS 证书
kubectl create secret tls ai-im-tls \
  --cert=path/to/cert.pem \
  --key=path/to/key.pem \
  -n ai-im
```

2. **部署应用**

```bash
# 使用部署脚本
chmod +x scripts/deploy.sh
./scripts/deploy.sh production

# 或手动部署
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/ingress.yaml
kubectl apply -f k8s/hpa.yaml
```

3. **验证部署**

```bash
# 检查 Pod 状态
kubectl get pods -n ai-im

# 检查服务
kubectl get svc -n ai-im

# 检查 Ingress
kubectl get ingress -n ai-im

# 查看日志
kubectl logs -f deployment/ai-im-server -n ai-im
```

### 扩缩容

```bash
# 手动扩容
kubectl scale deployment/ai-im-server --replicas=5 -n ai-im

# HPA 自动扩容已配置
# - 最小副本数: 3
# - 最大副本数: 10
# - CPU 目标: 70%
# - 内存目标: 80%
```

### 滚动更新

```bash
# 更新镜像
kubectl set image deployment/ai-im-server \
  server=ai-im-server:new-version \
  -n ai-im

# 查看更新状态
kubectl rollout status deployment/ai-im-server -n ai-im

# 回滚
kubectl rollout undo deployment/ai-im-server -n ai-im
```

---

## 监控配置

### Prometheus + Grafana

监控栈已包含在 `docker-compose.prod.yml` 中。

#### 访问 Grafana

1. 打开 http://localhost:3000
2. 登录 (admin / 你的密码)
3. 导入仪表板

#### 关键指标

- **系统指标**
  - CPU 使用率
  - 内存使用率
  - 磁盘 I/O
  - 网络流量

- **应用指标**
  - 请求速率 (RPS)
  - 响应时间 (P50, P95, P99)
  - 错误率
  - WebSocket 连接数

- **数据库指标**
  - 连接池使用率
  - 查询延迟
  - 慢查询数量

- **Redis 指标**
  - 命中率
  - 内存使用
  - 连接数

### 告警配置

编辑 `monitoring/alerts.yml` 添加告警规则：

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
```

---

## 备份与恢复

### 自动备份

```bash
# 设置定时任务
chmod +x scripts/backup.sh

# 添加到 crontab (每天凌晨 2 点)
0 2 * * * /path/to/scripts/backup.sh
```

### 手动备份

```bash
# 备份所有数据
./scripts/backup.sh

# 备份文件位置
ls -lh backups/
```

### 恢复数据

```bash
# 恢复 PostgreSQL
gunzip < backups/postgres_20231204_020000.sql.gz | \
  docker exec -i ai-im-postgres psql -U prod_user ai_im_prod

# 恢复 Redis
docker cp backups/redis_20231204_020000.rdb ai-im-redis:/data/dump.rdb
docker restart ai-im-redis
```

---

## 故障排查

### 常见问题

#### 1. 服务无法启动

```bash
# 查看日志
docker-compose logs server

# 检查配置
docker-compose config

# 检查端口占用
netstat -tulpn | grep 8080
```

#### 2. 数据库连接失败

```bash
# 检查数据库状态
docker-compose ps postgres

# 测试连接
docker exec -it ai-im-postgres psql -U prod_user -d ai_im_prod

# 查看数据库日志
docker-compose logs postgres
```

#### 3. Redis 连接失败

```bash
# 检查 Redis 状态
docker-compose ps redis

# 测试连接
docker exec -it ai-im-redis redis-cli ping

# 查看 Redis 日志
docker-compose logs redis
```

#### 4. 高内存使用

```bash
# 查看容器资源使用
docker stats

# 限制容器内存
docker-compose -f docker-compose.prod.yml up -d --scale server=3
```

#### 5. WebSocket 连接断开

```bash
# 检查 Nginx 配置
docker exec ai-im-nginx nginx -t

# 查看 Nginx 日志
docker-compose logs nginx

# 增加超时时间
# 编辑 nginx/nginx.conf 中的 proxy_read_timeout
```

### 性能优化

#### 数据库优化

```sql
-- 查看慢查询
SELECT * FROM pg_stat_statements 
ORDER BY mean_exec_time DESC 
LIMIT 10;

-- 创建索引
CREATE INDEX CONCURRENTLY idx_messages_user_id ON messages(user_id);
```

#### Redis 优化

```bash
# 查看内存使用
docker exec ai-im-redis redis-cli INFO memory

# 设置最大内存
docker exec ai-im-redis redis-cli CONFIG SET maxmemory 2gb
docker exec ai-im-redis redis-cli CONFIG SET maxmemory-policy allkeys-lru
```

### 日志收集

```bash
# 收集所有日志
docker-compose logs > logs_$(date +%Y%m%d_%H%M%S).txt

# 实时查看特定服务日志
docker-compose logs -f server

# 查看最近 100 行
docker-compose logs --tail=100 server
```

---

## 安全建议

1. **更改默认密码**
   - 数据库密码
   - Redis 密码
   - JWT Secret
   - Grafana 密码

2. **启用 HTTPS**
   - 使用 Let's Encrypt 证书
   - 配置 SSL/TLS

3. **限制网络访问**
   - 使用防火墙规则
   - 配置 IP 白名单

4. **定期更新**
   - 更新 Docker 镜像
   - 更新依赖包
   - 应用安全补丁

5. **监控和告警**
   - 配置告警规则
   - 监控异常行为
   - 定期审查日志

---

## 扩展阅读

- [Kubernetes 官方文档](https://kubernetes.io/docs/)
- [Docker Compose 文档](https://docs.docker.com/compose/)
- [Prometheus 文档](https://prometheus.io/docs/)
- [Grafana 文档](https://grafana.com/docs/)

---

## 支持

如有问题，请查看：
- [项目 README](README.md)
- [开发指南](DEV_SETUP.md)
- [API 文档](API_DOCS.md)
