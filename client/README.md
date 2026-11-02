# AIIM Flutter 客户端

AI-Native IM 的 Flutter 客户端，包含登录、会话、聊天、联系人、群组、个人资料、设置和 AI Agent 等页面。

> 项目已归档，不再维护。客户端只完成了部分联调，接口与交互可能不完整。

## 平台

- Android
- iOS
- Windows
- Web

当前仓库没有 macOS / Linux 客户端工程。

## 运行

```bash
flutter pub get

# 默认连接 http://localhost:8080
flutter run -d windows

# 覆盖 API 地址
flutter run -d windows --dart-define=API_BASE_URL=http://localhost:8080
```

Android 模拟器访问宿主机时使用 `http://10.0.2.2:8080`。

## 相关文档

- [项目根 README](../README.md)
- [文档索引](../docs/README.md)
- [开发期 AI 笔记（可能过时）](../docs/archive/CLAUDE.md)
