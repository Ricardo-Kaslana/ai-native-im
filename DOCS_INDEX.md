# 📚 文档索引

AI-Native IM 项目的完整文档导航。

## 🚀 快速开始

| 文档 | 说明 | 适用人群 |
|------|------|----------|
| [README.md](README.md) | 项目概览和快速开始 | 所有人 ⭐ |
| [QUICK_DEPLOY.md](QUICK_DEPLOY.md) | 5分钟快速部署 | 运维人员 |
| [CLIENT_QUICK_START.md](CLIENT_QUICK_START.md) | 客户端快速开始 | 前端开发 |
| [FAQ.md](FAQ.md) | 常见问题解答 | 所有人 |

## 📖 核心文档

### 项目概览
- [PROJECT_STATUS.md](PROJECT_STATUS.md) - 项目状态总览
- [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md) - 详细架构和设计
- [PROJECT_SPEC.md](PROJECT_SPEC.md) - 完整技术规范

### 开发指南
- [DEV_SETUP.md](DEV_SETUP.md) - 开发环境配置
- [server/TESTING.md](server/TESTING.md) - 测试指南
- [client/README.md](client/README.md) - 客户端开发指南
- [API_REFERENCE.md](API_REFERENCE.md) - API 参考文档
- [CONTRIBUTING.md](CONTRIBUTING.md) - 贡献指南

### 部署运维
- [DEPLOYMENT.md](DEPLOYMENT.md) - 完整部署指南
- [PRODUCTION_DEPLOYMENT.md](PRODUCTION_DEPLOYMENT.md) - 生产部署指南 ⭐
- [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) - 发布检查清单
- [QUICK_DEPLOY.md](QUICK_DEPLOY.md) - 快速部署
- [docker-compose.prod.yml](docker-compose.prod.yml) - 生产环境配置

### 帮助文档
- [FAQ.md](FAQ.md) - 常见问题解答
- [CONTRIBUTING.md](CONTRIBUTING.md) - 如何贡献

## 🎯 按角色分类

### 新手入门
1. 阅读 [README.md](README.md) 了解项目
2. 按照 [DEV_SETUP.md](DEV_SETUP.md) 配置环境
3. 查看 [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md) 理解架构

### 后端开发
1. [server/TESTING.md](server/TESTING.md) - 测试指南
2. [.kiro/specs/](./kiro/specs/) - 各模块规范
3. [PROJECT_SPEC.md](PROJECT_SPEC.md) - 技术规范

### 前端开发
1. [CLIENT_QUICK_START.md](CLIENT_QUICK_START.md) - 快速开始
2. [client/README.md](client/README.md) - 开发指南
3. [.kiro/specs/client-ui-system/](./kiro/specs/client-ui-system/) - UI 规范

### 运维人员
1. [PRODUCTION_DEPLOYMENT.md](PRODUCTION_DEPLOYMENT.md) - 生产部署指南 ⭐
2. [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) - 发布检查清单
3. [DEPLOYMENT.md](DEPLOYMENT.md) - 完整部署指南
4. [QUICK_DEPLOY.md](QUICK_DEPLOY.md) - 快速部署
5. [k8s/](k8s/) - Kubernetes 配置

### 测试人员
1. [server/TESTING.md](server/TESTING.md) - 测试指南
2. [.kiro/specs/e2e-integration-tests/](./kiro/specs/e2e-integration-tests/) - E2E 测试
3. [.kiro/specs/performance-reliability/](./kiro/specs/performance-reliability/) - 性能测试

## 📂 详细规范文档

### 后端模块规范
- [用户认证系统](./kiro/specs/user-auth-system/) - 认证、授权、JWT
- [消息系统](./kiro/specs/message-system/) - WebSocket、消息路由
- [群聊管理](./kiro/specs/group-chat-system/) - 群组、成员、权限
- [AI Agent 系统](./kiro/specs/ai-agent-system/) - AI 集成、RAG
- [媒体处理](./kiro/specs/media-system/) - 上传、压缩、存储

### 客户端规范
- [UI 系统](./kiro/specs/client-ui-system/) - 界面设计、组件

### 测试规范
- [E2E 测试](./kiro/specs/e2e-integration-tests/) - 端到端测试
- [性能测试](./kiro/specs/performance-reliability/) - 负载、压力测试

### 部署规范
- [部署运维](./kiro/specs/deployment-ops/) - Docker、K8s、监控

## 🔧 配置文件

### 环境配置
- [.env.example](.env.example) - 环境变量示例
- [server/.env.example](server/.env.example) - 后端环境变量

### Docker 配置
- [docker-compose.yml](docker-compose.yml) - 开发环境
- [docker-compose.prod.yml](docker-compose.prod.yml) - 生产环境
- [server/Dockerfile](server/Dockerfile) - 后端镜像

### Kubernetes 配置
- [k8s/namespace.yaml](k8s/namespace.yaml) - 命名空间
- [k8s/deployment.yaml](k8s/deployment.yaml) - 部署配置
- [k8s/service.yaml](k8s/service.yaml) - 服务配置
- [k8s/ingress.yaml](k8s/ingress.yaml) - 入口配置
- [k8s/hpa.yaml](k8s/hpa.yaml) - 自动扩缩容

### 监控配置
- [monitoring/prometheus.yml](monitoring/prometheus.yml) - Prometheus
- [monitoring/grafana/](monitoring/grafana/) - Grafana
- [nginx/nginx.conf](nginx/nginx.conf) - Nginx

## 🛠️ 脚本工具

### 部署脚本
- [scripts/deploy.sh](scripts/deploy.sh) - 自动化部署
- [scripts/backup.sh](scripts/backup.sh) - 数据备份
- [start-dev.sh](start-dev.sh) - 启动开发环境
- [stop-dev.sh](stop-dev.sh) - 停止开发环境

### 测试脚本
- [server/scripts/test-fast.sh](server/scripts/test-fast.sh) - 快速测试
- [server/scripts/test-parallel.sh](server/scripts/test-parallel.sh) - 并行测试

## 📊 项目统计

- **总代码行数**: ~36,000 行
- **测试数量**: 504 个
- **API 端点**: 55+ 个
- **数据库表**: 15 个（后端）+ 5 个（客户端）
- **支持平台**: 6 个（iOS/Android/macOS/Linux/Windows/Web）

## 🔗 外部资源

### 技术栈文档
- [Rust 官方文档](https://doc.rust-lang.org/)
- [Axum 框架](https://docs.rs/axum/)
- [Flutter 文档](https://flutter.dev/docs)
- [PostgreSQL 文档](https://www.postgresql.org/docs/)
- [Redis 文档](https://redis.io/documentation)

### 部署相关
- [Docker 文档](https://docs.docker.com/)
- [Kubernetes 文档](https://kubernetes.io/docs/)
- [Prometheus 文档](https://prometheus.io/docs/)
- [Grafana 文档](https://grafana.com/docs/)

## 📝 文档维护

### 文档更新原则
1. 保持文档与代码同步
2. 使用清晰的标题和结构
3. 提供实际可运行的示例
4. 包含故障排查信息

### 贡献文档
如需更新文档，请：
1. 确保内容准确
2. 遵循现有格式
3. 更新相关索引
4. 提交 Pull Request

---

**最后更新**: 2025-12-04  
**维护者**: AI-Native IM Team
