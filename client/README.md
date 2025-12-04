# AI Native IM Client

跨平台即时通讯客户端，使用 Flutter 构建。

## 功能特性

- 🎨 Neo-Minimalism 设计风格
- 🌙 深色/浅色主题支持 (OLED 优化)
- 💬 实时消息收发
- 👥 群组聊天
- 🤖 AI Agent 交互
- 📱 跨平台支持 (iOS, Android, macOS, Linux, Windows, Web)

## 快速开始

### 环境要求

- Flutter 3.16+
- Dart 3.2+

### 安装依赖

```bash
cd client
flutter pub get
```

### 运行应用

```bash
# 查看可用设备
flutter devices

# 运行 (自动选择设备)
flutter run

# 指定平台运行
flutter run -d chrome    # Web
flutter run -d macos     # macOS
flutter run -d linux     # Linux
```

### 代码检查

```bash
flutter analyze
flutter test
```

## 项目结构

```
lib/
├── main.dart                    # 应用入口
├── app.dart                     # 应用配置
├── core/                        # 核心功能
│   ├── constants/               # 常量 (颜色、字体、间距)
│   ├── theme/                   # 主题配置
│   ├── utils/                   # 工具函数
│   └── network/                 # 网络层
├── data/                        # 数据层
│   ├── models/                  # 数据模型
│   ├── repositories/            # 数据仓库
│   └── local/                   # 本地数据库
├── domain/                      # 业务逻辑层
│   ├── usecases/                # 用例
│   └── entities/                # 业务实体
└── presentation/                # 表现层
    ├── providers/               # Riverpod Providers
    ├── screens/                 # 页面
    └── widgets/                 # 组件
```

## 技术栈

- **框架**: Flutter 3.24+
- **状态管理**: Riverpod
- **本地存储**: Drift
- **网络**: Dio, WebSocket
- **UI**: Material 3, Cupertino

## 开发进度

- [x] 项目搭建
- [x] 设计系统
- [x] 状态管理基础
- [x] 认证界面
- [x] 会话列表
- [x] 聊天界面
- [x] 深色模式
- [x] 本地数据库
- [x] WebSocket 集成
- [x] 媒体处理
- [ ] 更多功能...

## 许可证

MIT License
