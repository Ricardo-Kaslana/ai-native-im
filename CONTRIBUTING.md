# 🤝 贡献指南

感谢你对 AI-Native IM 项目的关注！我们欢迎各种形式的贡献。

## 📋 目录

- [行为准则](#行为准则)
- [如何贡献](#如何贡献)
- [开发流程](#开发流程)
- [代码规范](#代码规范)
- [提交规范](#提交规范)
- [测试要求](#测试要求)

---

## 行为准则

### 我们的承诺

为了营造开放和友好的环境，我们承诺：

- 使用友好和包容的语言
- 尊重不同的观点和经验
- 优雅地接受建设性批评
- 关注对社区最有利的事情
- 对其他社区成员表示同理心

### 不可接受的行为

- 使用性化的语言或图像
- 人身攻击或侮辱性评论
- 公开或私下骚扰
- 未经许可发布他人的私人信息
- 其他不道德或不专业的行为

---

## 如何贡献

### 报告 Bug

发现 Bug？请创建 Issue 并包含：

1. **清晰的标题** - 简洁描述问题
2. **复现步骤** - 详细的操作步骤
3. **预期行为** - 应该发生什么
4. **实际行为** - 实际发生了什么
5. **环境信息** - 操作系统、版本等
6. **截图/日志** - 如果适用

**Bug 报告模板**:
```markdown
## 问题描述
简短描述问题

## 复现步骤
1. 执行 '...'
2. 点击 '...'
3. 看到错误

## 预期行为
应该显示...

## 实际行为
实际显示...

## 环境
- OS: Ubuntu 22.04
- Rust: 1.75
- Flutter: 3.16

## 日志
```
错误日志
```
```

### 提出新功能

有好的想法？创建 Feature Request：

1. **功能描述** - 清晰描述功能
2. **使用场景** - 为什么需要这个功能
3. **建议实现** - 如何实现（可选）
4. **替代方案** - 其他可能的方案

### 提交代码

1. **Fork 项目**
2. **创建分支** - `git checkout -b feature/amazing-feature`
3. **编写代码** - 遵循代码规范
4. **添加测试** - 确保测试通过
5. **提交更改** - 遵循提交规范
6. **推送分支** - `git push origin feature/amazing-feature`
7. **创建 PR** - 详细描述更改

---

## 开发流程

### 1. 设置开发环境

```bash
# 克隆项目
git clone <repository-url>
cd ai-native-im

# 安装依赖
cd server && cargo build
cd ../client && flutter pub get

# 启动开发环境
./start-dev.sh
```

详细步骤见 [DEV_SETUP.md](DEV_SETUP.md)

### 2. 创建分支

使用描述性的分支名：

```bash
# 新功能
git checkout -b feature/user-profile-edit

# Bug 修复
git checkout -b fix/message-send-error

# 文档更新
git checkout -b docs/api-reference

# 性能优化
git checkout -b perf/database-query
```

### 3. 开发

- 保持提交小而专注
- 频繁提交
- 编写清晰的提交信息
- 添加必要的测试
- 更新相关文档

### 4. 测试

```bash
# 后端测试
cd server
make test-fast

# 客户端测试
cd client
flutter test

# 运行所有测试
make test-all
```

### 5. 提交 Pull Request

**PR 标题格式**:
```
<type>: <description>

例如:
feat: add user profile editing
fix: resolve message send error
docs: update API reference
```

**PR 描述模板**:
```markdown
## 更改类型
- [ ] Bug 修复
- [ ] 新功能
- [ ] 文档更新
- [ ] 性能优化
- [ ] 代码重构

## 更改描述
清晰描述做了什么更改

## 相关 Issue
Closes #123

## 测试
- [ ] 添加了新测试
- [ ] 所有测试通过
- [ ] 手动测试通过

## 截图
如果适用，添加截图

## 检查清单
- [ ] 代码遵循项目规范
- [ ] 添加了必要的注释
- [ ] 更新了相关文档
- [ ] 没有引入新的警告
- [ ] 测试覆盖率没有降低
```

---

## 代码规范

### Rust 代码规范

#### 格式化

```bash
# 格式化代码
cargo fmt

# 检查格式
cargo fmt -- --check
```

#### Linting

```bash
# 运行 clippy
cargo clippy -- -D warnings
```

#### 命名规范

```rust
// 类型使用 PascalCase
struct UserProfile { }
enum MessageType { }

// 函数和变量使用 snake_case
fn send_message() { }
let user_id = "...";

// 常量使用 SCREAMING_SNAKE_CASE
const MAX_MESSAGE_LENGTH: usize = 1000;

// 生命周期使用短小的名称
fn process<'a>(data: &'a str) { }
```

#### 错误处理

```rust
// 使用 Result 类型
fn process_data() -> Result<Data, Error> {
    // ...
}

// 使用 ? 操作符
let data = fetch_data()?;

// 自定义错误类型
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
```

#### 文档注释

```rust
/// 发送消息到指定用户
///
/// # Arguments
///
/// * `user_id` - 目标用户 ID
/// * `content` - 消息内容
///
/// # Returns
///
/// 返回消息 ID
///
/// # Errors
///
/// 如果用户不存在或消息发送失败，返回错误
pub async fn send_message(
    user_id: Uuid,
    content: String,
) -> Result<Uuid, Error> {
    // ...
}
```

### Flutter/Dart 代码规范

#### 格式化

```bash
# 格式化代码
flutter format .

# 分析代码
flutter analyze
```

#### 命名规范

```dart
// 类使用 PascalCase
class UserProfile { }
class MessageWidget extends StatelessWidget { }

// 函数和变量使用 camelCase
void sendMessage() { }
final userId = '...';

// 常量使用 lowerCamelCase
const maxMessageLength = 1000;

// 私有成员使用下划线前缀
String _privateField;
void _privateMethod() { }
```

#### Widget 结构

```dart
class MyWidget extends StatelessWidget {
  const MyWidget({
    Key? key,
    required this.title,
  }) : super(key: key);

  final String title;

  @override
  Widget build(BuildContext context) {
    return Container(
      // ...
    );
  }
}
```

#### 状态管理

```dart
// 使用 Riverpod
final userProvider = StateNotifierProvider<UserNotifier, User>((ref) {
  return UserNotifier();
});

// 在 Widget 中使用
class MyWidget extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final user = ref.watch(userProvider);
    return Text(user.name);
  }
}
```

---

## 提交规范

### Commit Message 格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Type 类型

- `feat` - 新功能
- `fix` - Bug 修复
- `docs` - 文档更新
- `style` - 代码格式（不影响功能）
- `refactor` - 代码重构
- `perf` - 性能优化
- `test` - 测试相关
- `chore` - 构建/工具相关

### Scope 范围

- `auth` - 认证系统
- `message` - 消息系统
- `group` - 群组系统
- `ai` - AI Agent
- `media` - 媒体处理
- `client` - 客户端
- `server` - 服务端

### 示例

```bash
# 新功能
git commit -m "feat(auth): add password reset functionality"

# Bug 修复
git commit -m "fix(message): resolve WebSocket reconnection issue"

# 文档
git commit -m "docs(api): update authentication endpoints"

# 性能优化
git commit -m "perf(database): optimize message query with indexes"
```

---

## 测试要求

### 后端测试

#### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_send_message() {
        // Arrange
        let service = MessageService::new();
        
        // Act
        let result = service.send_message(...).await;
        
        // Assert
        assert!(result.is_ok());
    }
}
```

#### 集成测试

```rust
// tests/integration_test.rs
#[tokio::test]
async fn test_complete_auth_flow() {
    let pool = setup_test_db().await;
    let service = AuthService::new(pool);
    
    // 注册
    let user = service.register(...).await.unwrap();
    
    // 登录
    let tokens = service.login(...).await.unwrap();
    
    // 验证
    assert!(service.verify_token(&tokens.access_token).is_ok());
}
```

### 客户端测试

#### Widget 测试

```dart
testWidgets('MessageBubble displays content', (tester) async {
  await tester.pumpWidget(
    MaterialApp(
      home: MessageBubble(
        message: Message(content: 'Hello'),
      ),
    ),
  );

  expect(find.text('Hello'), findsOneWidget);
});
```

#### 集成测试

```dart
testWidgets('Complete login flow', (tester) async {
  await tester.pumpWidget(MyApp());
  
  // 输入用户名
  await tester.enterText(find.byKey(Key('username')), 'john');
  
  // 输入密码
  await tester.enterText(find.byKey(Key('password')), 'pass');
  
  // 点击登录
  await tester.tap(find.byKey(Key('login_button')));
  await tester.pumpAndSettle();
  
  // 验证跳转到主页
  expect(find.byType(HomePage), findsOneWidget);
});
```

### 测试覆盖率

- 新功能必须包含测试
- 目标覆盖率: 80%+
- 关键路径必须 100% 覆盖

---

## 代码审查

### 审查清单

**功能性**
- [ ] 代码实现了预期功能
- [ ] 边界条件处理正确
- [ ] 错误处理完善

**代码质量**
- [ ] 代码清晰易读
- [ ] 命名有意义
- [ ] 没有重复代码
- [ ] 遵循项目规范

**测试**
- [ ] 包含必要的测试
- [ ] 测试覆盖关键路径
- [ ] 所有测试通过

**文档**
- [ ] 更新了相关文档
- [ ] 添加了必要的注释
- [ ] API 变更有文档说明

**性能**
- [ ] 没有明显的性能问题
- [ ] 数据库查询优化
- [ ] 资源使用合理

---

## 发布流程

### 版本号规范

遵循 [语义化版本](https://semver.org/lang/zh-CN/)：

- `MAJOR.MINOR.PATCH`
- `1.0.0` - 主版本（不兼容的 API 变更）
- `1.1.0` - 次版本（向后兼容的功能）
- `1.1.1` - 修订版本（向后兼容的 Bug 修复）

### 发布步骤

1. 更新版本号
2. 更新 CHANGELOG
3. 运行完整测试
4. 创建 Git tag
5. 构建发布包
6. 发布到仓库

---

## 获取帮助

- 📖 查看 [文档](DOCS_INDEX.md)
- 💬 加入讨论区
- 📧 联系维护者
- 🐛 报告 Issue

---

## 许可证

通过贡献代码，你同意你的贡献将在与项目相同的许可证下发布。

---

**感谢你的贡献！** 🎉
