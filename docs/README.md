# 📚 AI Native IM 文档中心

本目录包含项目的所有技术文档。

## 📋 快速导航

### 入门指南
- [开发环境配置](../DEV_SETUP.md) - 环境安装和配置
- [项目总览](../archive/docs/PROJECT_OVERVIEW.md) - 项目架构和模块说明

### 客户端开发
- [客户端开发计划](../CLIENT_DEVELOPMENT_PLAN.md) - 详细开发规划和进度
- [客户端架构](../CLIENT_ARCHITECTURE.md) - Flutter客户端技术架构
- [客户端集成清单](../CLIENT_INTEGRATION_CHECKLIST.md) - 后端API对接清单

### API文档
- [API参考](../archive/docs/API_REFERENCE.md) - 完整API接口文档
- [API集成指南](../API_INTEGRATION_GUIDE.md) - 客户端API调用指南
- [错误码规范](../archive/docs/ERROR_CODES.md) - 统一错误码定义

### AI Agent系统
- [AI Agent指南](../archive/docs/AI_AGENT_GUIDE.md) - AI Agent功能说明
- [AI Agent工作流](../archive/docs/AI_AGENT_WORKFLOW.md) - AI Agent处理流程

### 测试与部署
- [测试指南](../TESTING_GUIDE.md) - 测试策略和执行方法

### 更新日志
- [更新日志](../changelogs/CHANGELOG.md) - 版本更新记录

## 📁 目录结构

```
docs/                           # 文档中心
├── README.md                   # 本文件

archive/docs/                   # 核心规范文档
├── PROJECT_OVERVIEW.md         # 项目总览
├── PROJECT_SPEC.md             # 项目规格
├── API_REFERENCE.md            # API参考
├── ERROR_CODES.md              # 错误码规范
├── AI_AGENT_GUIDE.md           # AI Agent指南
└── AI_AGENT_WORKFLOW.md        # AI Agent工作流

根目录文档/                      # 开发指南
├── DEV_SETUP.md                # 开发环境配置
├── CLIENT_DEVELOPMENT_PLAN.md  # 客户端开发计划
├── CLIENT_ARCHITECTURE.md      # 客户端架构
├── CLIENT_INTEGRATION_CHECKLIST.md  # 集成清单
├── API_INTEGRATION_GUIDE.md    # API集成指南
└── TESTING_GUIDE.md            # 测试指南

changelogs/                     # 更新日志
└── CHANGELOG.md                # 版本变更记录

archive/bugfix/                 # Bug修复记录
└── *.md                        # 各类问题修复文档
```

## 🔍 按角色查找

### 后端开发者
1. [开发环境配置](../DEV_SETUP.md)
2. [API参考](../archive/docs/API_REFERENCE.md)
3. [错误码规范](../archive/docs/ERROR_CODES.md)
4. [AI Agent工作流](../archive/docs/AI_AGENT_WORKFLOW.md)

### 客户端开发者
1. [客户端开发计划](../CLIENT_DEVELOPMENT_PLAN.md)
2. [客户端架构](../CLIENT_ARCHITECTURE.md)
3. [API集成指南](../API_INTEGRATION_GUIDE.md)
4. [错误码规范](../archive/docs/ERROR_CODES.md)

### 新成员入门
1. [项目总览](../archive/docs/PROJECT_OVERVIEW.md) - 了解项目
2. [开发环境配置](../DEV_SETUP.md) - 配置环境
3. [更新日志](../changelogs/CHANGELOG.md) - 了解最新进展

## 📊 项目状态

| 模块 | 状态 | 完成度 |
|------|------|--------|
| 用户认证系统 | ✅ 完成 | 100% |
| 消息系统 | ✅ 完成 | 100% |
| 群聊管理 | ✅ 完成 | 100% |
| AI Agent | ✅ 完成 | 100% |
| 媒体处理 | ✅ 完成 | 100% |
| 客户端基础 | ✅ 完成 | 100% |
| 客户端认证 | ✅ 完成 | 100% |
| 客户端聊天 | ✅ 完成 | 90% |
| 客户端联系人 | ⏳ 待开始 | 0% |
| 客户端群组 | ⏳ 待开始 | 0% |

## 🔗 相关链接

- **后端服务**: `http://localhost:8080`
- **WebSocket**: `ws://localhost:8080/ws`
- **API基础路径**: `/api/v1`

---

**最后更新**: 2025-12-13
