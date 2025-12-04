# ⚡ 快速部署指南

5 分钟内完成 AI-Native IM 系统部署。

## 🚀 Docker Compose 部署（推荐新手）

### 1. 克隆项目
```bash
git clone <repository-url>
cd ai-native-im
```

### 2. 配置环境变量
```bash
cp .env.example .env
# 编辑 .env 文件，至少修改以下内容：
# - DB_PASSWORD
# - REDIS_PASSWORD
# - JWT_SECRET
# - GRAFANA_PASSWORD
```

### 3. 生成 SSL 证书（开发环境）
```bash
mkdir -p nginx/ssl
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout nginx/ssl/key.pem \
  -out nginx/ssl/cert.pem \
  -subj "/CN=localhost"
```

### 4. 启动服务
```bash
docker-compose -f docker-compose.prod.yml up -d
```

### 5. 验证部署
```bash
# 检查服务状态
docker-compose -f docker-compose.prod.yml ps

# 测试 API
curl -k https://localhost/health

# 访问监控
# Grafana: http://localhost:3000 (admin/你的密码)
# Prometheus: http://localhost:9090
```

**完成！** 🎉

---

## ☸️ Kubernetes 部署（生产环境）

### 前置要求
- Kubernetes 集群已配置
- kubectl 已安装并配置
- Docker 镜像已构建

### 1. 构建镜像
```bash
docker build -t ai-im-server:latest -f server/Dockerfile .
```

### 2. 创建 Secrets
```bash
kubectl create namespace ai-im

kubectl create secret generic ai-im-secrets \
  --from-literal=database-url='postgresql://prod_user:YOUR_PASSWORD@postgres-service:5432/ai_im_prod' \
  --from-literal=jwt-secret='YOUR_JWT_SECRET_AT_LEAST_32_CHARS' \
  -n ai-im
```

### 3. 部署应用
```bash
chmod +x scripts/deploy.sh
./scripts/deploy.sh production
```

### 4. 验证部署
```bash
kubectl get pods -n ai-im
kubectl get svc -n ai-im
kubectl logs -f deployment/ai-im-server -n ai-im
```

**完成！** 🎉

---

## 📊 访问服务

### API 端点
- **健康检查**: `https://your-domain/health`
- **WebSocket**: `wss://your-domain/ws`
- **API**: `https://your-domain/api/*`

### 监控面板
- **Grafana**: `http://your-domain:3000`
  - 用户名: admin
  - 密码: 你在 .env 中设置的密码

- **Prometheus**: `http://your-domain:9090`

---

## 🔧 常用命令

### Docker Compose

```bash
# 启动服务
docker-compose -f docker-compose.prod.yml up -d

# 停止服务
docker-compose -f docker-compose.prod.yml down

# 查看日志
docker-compose -f docker-compose.prod.yml logs -f server

# 重启服务
docker-compose -f docker-compose.prod.yml restart server

# 扩容
docker-compose -f docker-compose.prod.yml up -d --scale server=5
```

### Kubernetes

```bash
# 查看状态
kubectl get all -n ai-im

# 查看日志
kubectl logs -f deployment/ai-im-server -n ai-im

# 扩容
kubectl scale deployment/ai-im-server --replicas=5 -n ai-im

# 更新镜像
kubectl set image deployment/ai-im-server server=ai-im-server:v2 -n ai-im

# 回滚
kubectl rollout undo deployment/ai-im-server -n ai-im
```

### 备份

```bash
# 手动备份
./scripts/backup.sh

# 设置自动备份（每天凌晨 2 点）
crontab -e
# 添加: 0 2 * * * /path/to/scripts/backup.sh
```

---

## 🐛 故障排查

### 服务无法启动
```bash
# 查看日志
docker-compose logs server

# 检查配置
docker-compose config

# 重新构建
docker-compose build --no-cache server
```

### 数据库连接失败
```bash
# 检查数据库
docker-compose ps postgres
docker-compose logs postgres

# 测试连接
docker exec -it ai-im-postgres psql -U prod_user -d ai_im_prod
```

### 高内存使用
```bash
# 查看资源使用
docker stats

# 重启服务
docker-compose restart server
```

---

## 📚 更多信息

- 完整部署指南: [DEPLOYMENT.md](DEPLOYMENT.md)
- 开发环境配置: [DEV_SETUP.md](DEV_SETUP.md)
- 项目文档: [README.md](README.md)

---

## 🆘 获取帮助

遇到问题？
1. 查看 [DEPLOYMENT.md](DEPLOYMENT.md) 故障排查章节
2. 检查日志: `docker-compose logs -f`
3. 查看 GitHub Issues
