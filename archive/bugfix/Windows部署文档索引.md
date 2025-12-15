# Windows 部署文档索引 📚

## 🚀 从这里开始

**新手必读**：
1. 📄 [开始部署.txt](开始部署.txt) - **从这里开始！**
2. 📄 [快速开始.txt](快速开始.txt) - 图形化说明
3. 📄 [部署指南.md](部署指南.md) - 详细步骤

---

## 📖 详细文档

### 中文文档
- 📄 [部署指南.md](部署指南.md) - 完整的中文部署指南
- 📄 [WINDOWS_部署总结.txt](WINDOWS_部署总结.txt) - 快速参考卡片
- 📄 [快速开始.txt](快速开始.txt) - 图形化部署流程
- 📄 [开始部署.txt](开始部署.txt) - 入门指南

### 英文文档
- 📄 [WINDOWS_QUICK_START.md](WINDOWS_QUICK_START.md) - 快速开始（英文）
- 📄 [WINDOWS_SETUP.md](WINDOWS_SETUP.md) - 完整设置指南（英文）
- 📄 [README_WINDOWS.md](README_WINDOWS.md) - Windows README

---

## 🔧 脚本工具

### 批处理脚本 (.bat)
- 🔧 [check-env.bat](check-env.bat) - 环境检查脚本
- 🔧 [start-windows.bat](start-windows.bat) - 启动应用脚本

### PowerShell 脚本 (.ps1)
- 🔧 [check-env.ps1](check-env.ps1) - 环境检查（PowerShell）
- 🔧 [start-windows.ps1](start-windows.ps1) - 启动应用（PowerShell）

**使用方法**：
- 双击 `.bat` 文件直接运行
- PowerShell 脚本需要在 PowerShell 中运行：`.\check-env.ps1`

---

## 📚 项目文档

### 核心文档
- 📖 [README.md](README.md) - 项目总览
- 📖 [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md) - 项目架构
- 📖 [PROJECT_STATUS.md](PROJECT_STATUS.md) - 项目状态
- 📖 [PROJECT_SPEC.md](PROJECT_SPEC.md) - 项目规范

### 开发文档
- 📖 [DEV_SETUP.md](DEV_SETUP.md) - 开发环境配置
- 📖 [API_REFERENCE.md](API_REFERENCE.md) - API 文档
- 📖 [FAQ.md](FAQ.md) - 常见问题
- 📖 [CONTRIBUTING.md](CONTRIBUTING.md) - 贡献指南

### 部署文档
- 📖 [DEPLOYMENT.md](DEPLOYMENT.md) - 完整部署指南
- 📖 [QUICK_DEPLOY.md](QUICK_DEPLOY.md) - 快速部署
- 📖 [PRODUCTION_DEPLOYMENT.md](PRODUCTION_DEPLOYMENT.md) - 生产部署

### 客户端文档
- 📖 [CLIENT_QUICK_START.md](CLIENT_QUICK_START.md) - 客户端快速开始

---

## 🎯 按场景查找

### 场景 1: 我是新手，第一次部署
1. 阅读 [开始部署.txt](开始部署.txt)
2. 如果有 Docker，运行 `docker compose up -d`
3. 如果没有 Docker，阅读 [部署指南.md](部署指南.md)
4. 运行 [check-env.bat](check-env.bat) 检查环境
5. 运行 [start-windows.bat](start-windows.bat) 启动应用

### 场景 2: 我遇到了问题
1. 运行 [check-env.bat](check-env.bat) 查看哪个服务有问题
2. 查看 [部署指南.md](部署指南.md) 的"常见问题"部分
3. 查看 [FAQ.md](FAQ.md)
4. 查看 [WINDOWS_SETUP.md](WINDOWS_SETUP.md) 的故障排除部分

### 场景 3: 我想了解项目
1. 阅读 [README.md](README.md) - 项目概览
2. 阅读 [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md) - 架构设计
3. 阅读 [PROJECT_STATUS.md](PROJECT_STATUS.md) - 当前状态
4. 查看 [API_REFERENCE.md](API_REFERENCE.md) - API 文档

### 场景 4: 我想开发客户端
1. 阅读 [CLIENT_QUICK_START.md](CLIENT_QUICK_START.md)
2. 确保后端已启动
3. 进入 `client` 目录
4. 运行 `flutter run`

### 场景 5: 我想部署到生产环境
1. 阅读 [DEPLOYMENT.md](DEPLOYMENT.md)
2. 阅读 [PRODUCTION_DEPLOYMENT.md](PRODUCTION_DEPLOYMENT.md)
3. 查看 [docker-compose.prod.yml](docker-compose.prod.yml)
4. 查看 [k8s/](k8s/) 目录的 Kubernetes 配置

---

## 🔍 快速查找

### 安装相关
- PostgreSQL 安装 → [部署指南.md](部署指南.md) 步骤 1
- Redis 安装 → [部署指南.md](部署指南.md) 步骤 2
- Qdrant 安装 → [部署指南.md](部署指南.md) 步骤 3

### 配置相关
- 环境变量配置 → [server/.env](server/.env)
- 数据库配置 → [部署指南.md](部署指南.md)
- AI 模型配置 → [server/.env](server/.env) AI 部分

### 测试相关
- 运行测试 → [server/TESTING.md](server/TESTING.md)
- API 测试 → [API_REFERENCE.md](API_REFERENCE.md)
- 性能测试 → [PROJECT_STATUS.md](PROJECT_STATUS.md)

### 故障排除
- PostgreSQL 问题 → [部署指南.md](部署指南.md) 常见问题
- Redis 问题 → [部署指南.md](部署指南.md) 常见问题
- Qdrant 问题 → [部署指南.md](部署指南.md) 常见问题
- 编译问题 → [WINDOWS_SETUP.md](WINDOWS_SETUP.md) 故障排除

---

## 📊 文档统计

| 类型 | 数量 | 说明 |
|------|------|------|
| 部署文档 | 7 个 | Windows 专用部署指南 |
| 脚本工具 | 4 个 | 自动化脚本 |
| 项目文档 | 10+ 个 | 项目说明和 API 文档 |
| 配置文件 | 3 个 | Docker 和环境配置 |

---

## 💡 推荐阅读顺序

### 对于新手
1. [开始部署.txt](开始部署.txt) ⭐
2. [快速开始.txt](快速开始.txt)
3. [部署指南.md](部署指南.md)
4. [README.md](README.md)

### 对于开发者
1. [README.md](README.md)
2. [PROJECT_OVERVIEW.md](PROJECT_OVERVIEW.md)
3. [DEV_SETUP.md](DEV_SETUP.md)
4. [API_REFERENCE.md](API_REFERENCE.md)

### 对于运维人员
1. [DEPLOYMENT.md](DEPLOYMENT.md)
2. [PRODUCTION_DEPLOYMENT.md](PRODUCTION_DEPLOYMENT.md)
3. [docker-compose.prod.yml](docker-compose.prod.yml)
4. [k8s/](k8s/) 配置

---

## 🆘 获取帮助

1. **运行检查脚本**：`check-env.bat`
2. **查看常见问题**：[FAQ.md](FAQ.md)
3. **查看部署指南**：[部署指南.md](部署指南.md)
4. **查看项目状态**：[PROJECT_STATUS.md](PROJECT_STATUS.md)

---

## 🎉 快速链接

- 🚀 [开始部署](开始部署.txt)
- 📖 [项目总览](README.md)
- 🔧 [环境检查](check-env.bat)
- 🎯 [启动应用](start-windows.bat)
- 📚 [API 文档](API_REFERENCE.md)
- ❓ [常见问题](FAQ.md)

---

**祝你部署顺利！** 🎉

如果有任何问题，请先运行 `check-env.bat` 检查环境状态。
