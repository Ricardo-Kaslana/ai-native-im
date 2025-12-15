# AI Agent 数据格式不匹配修复

## 问题描述

创建 AI Agent 后，重新加载时失败：
```
❌ Failed to load AI Agent: type 'Null' is not a subtype of type 'Map<String, dynamic>' in type cast
Stack trace: #0 new AIAgent.fromJson (package:client/data/models/ai_agent.dart:32:29)
```

## 根本原因

### 后端数据结构（平铺）
```rust
pub struct AIAgent {
    pub id: Uuid,
    pub group_id: Uuid,
    pub name: String,
    pub toxicity_level: i16,        // ✅ 平铺字段
    pub enthusiasm_level: i16,      // ✅ 平铺字段
    pub expertise_domains: Vec<String>,
    pub speech_suffix: Option<String>,
    pub trigger_keywords: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**API 返回**：
```json
{
  "id": "...",
  "group_id": "...",
  "name": "科莱塔",
  "toxicity_level": 5,
  "enthusiasm_level": 5,
  "expertise_domains": [],
  "speech_suffix": "喵~",
  "trigger_keywords": [],
  "is_active": true,
  "created_at": "...",
  "updated_at": "..."
}
```

### 客户端期望（嵌套）

```dart
// 修复前
factory AIAgent.fromJson(Map<String, dynamic> json) {
  return AIAgent(
    personality: AIAgentPersonality.fromJson(
      json['personality'] as Map<String, dynamic>,  // ❌ 期望嵌套对象
    ),
    ...
  );
}
```

**期望的格式**：
```json
{
  "id": "...",
  "personality": {              // ❌ 后端没有这个嵌套对象
    "toxicity_level": 5,
    "enthusiasm_level": 5,
    ...
  },
  ...
}
```

### 不匹配

- 后端：平铺的字段（`toxicity_level`, `enthusiasm_level`）
- 客户端：期望嵌套的 `personality` 对象
- 结果：`json['personality']` 返回 `null`，类型转换失败

## 解决方案

修改客户端的 `fromJson` 方法，从平铺的字段构造 `personality` 对象：

### 修复前
```dart
factory AIAgent.fromJson(Map<String, dynamic> json) {
  return AIAgent(
    personality: AIAgentPersonality.fromJson(
      json['personality'] as Map<String, dynamic>,  // ❌ 失败
    ),
    ...
  );
}
```

### 修复后
```dart
factory AIAgent.fromJson(Map<String, dynamic> json) {
  return AIAgent(
    // Backend returns flat structure, construct personality from individual fields
    personality: AIAgentPersonality(
      toxicityLevel: json['toxicity_level'] as int? ?? 5,
      enthusiasmLevel: json['enthusiasm_level'] as int? ?? 5,
      expertiseDomains: (json['expertise_domains'] as List<dynamic>?)
              ?.map((e) => e as String)
              .toList() ??
          [],
      speechSuffix: json['speech_suffix'] as String?,
    ),
    ...
  );
}
```

## 关键改进

### 1. 直接构造对象
```dart
// 不再调用 fromJson
personality: AIAgentPersonality(...)
```

### 2. 从平铺字段读取
```dart
toxicityLevel: json['toxicity_level'] as int? ?? 5,
enthusiasmLevel: json['enthusiasm_level'] as int? ?? 5,
```

### 3. 处理列表字段
```dart
expertiseDomains: (json['expertise_domains'] as List<dynamic>?)
        ?.map((e) => e as String)
        .toList() ??
    [],
```

### 4. 可选字段
```dart
speechSuffix: json['speech_suffix'] as String?,
```

## 数据流

### 创建 AI Agent

**客户端 → 后端**：
```dart
// CreateAgentRequest.toJson()
{
  "name": "科莱塔",
  "toxicity_level": 5,        // ✅ 平铺格式
  "enthusiasm_level": 5,
  "speech_suffix": "喵~",
  ...
}
```

**后端处理**：
```rust
// CreateAIAgent 直接匹配
pub struct CreateAIAgent {
    pub name: String,
    pub toxicity_level: Option<i16>,
    pub enthusiasm_level: Option<i16>,
    ...
}
```

### 获取 AI Agent

**后端 → 客户端**：
```json
{
  "id": "...",
  "toxicity_level": 5,        // ✅ 平铺格式
  "enthusiasm_level": 5,
  ...
}
```

**客户端处理**：
```dart
// AIAgent.fromJson() 从平铺字段构造 personality
personality: AIAgentPersonality(
  toxicityLevel: json['toxicity_level'],
  ...
)
```

## 为什么不修改后端？

### 选项 1：修改后端（不推荐）
```rust
// 需要添加嵌套结构
pub struct AIAgent {
    pub id: Uuid,
    pub personality: Personality,  // 嵌套
    ...
}

pub struct Personality {
    pub toxicity_level: i16,
    pub enthusiasm_level: i16,
    ...
}
```

**缺点**：
- 需要修改数据库表结构
- 需要修改所有相关代码
- 更复杂的序列化/反序列化
- 影响其他功能

### 选项 2：修改客户端（推荐）✅
```dart
// 只需修改 fromJson 方法
personality: AIAgentPersonality(
  toxicityLevel: json['toxicity_level'],
  ...
)
```

**优点**：
- 只需修改一个方法
- 不影响数据库
- 不影响其他代码
- 更简单、更快

## 其他字段处理

### customPrompt 字段

客户端的 `AIAgentPersonality` 有 `customPrompt` 字段，但后端没有：

```dart
class AIAgentPersonality {
  final String? customPrompt;  // 客户端独有
  ...
}
```

**处理方式**：
- `fromJson` 时不设置（默认为 `null`）
- `toJson` 时包含（后端可以忽略）
- 未来后端可以添加支持

### avatarUrl 字段

后端没有 `avatar_url` 字段，但客户端有：

```dart
class AIAgent {
  final String? avatarUrl;  // 客户端有
  ...
}
```

**处理方式**：
- `fromJson` 时尝试读取（如果后端添加了）
- 如果不存在，设置为 `null`
- 不影响功能

## 测试

### 热重载客户端
```bash
r
```

### 测试流程

1. **创建 AI Agent**：
   - 填写表单
   - 点击创建
   - ✅ 应该成功

2. **查看 AI Agent**：
   - 返回群组信息
   - 再次进入 AI Agent 界面
   - ✅ 应该显示详情，不再崩溃

3. **验证数据**：
   - 名称：科莱塔
   - 毒舌程度：5
   - 热情度：5
   - 口癖：喵~

### 验证日志

**创建时**：
```
🔵 REQUEST[POST] => /groups/.../agent
✅ RESPONSE[201] => /groups/.../agent
Data: {success: true, data: {...}}
```

**加载时**：
```
🔵 Loading AI Agent for group: ...
🔵 REQUEST[GET] => /groups/.../agent
✅ RESPONSE[200] => /groups/.../agent
Data: {success: true, data: {id: ..., toxicity_level: 5, ...}}
✅ AI Agent loaded: ...
```

不应该看到：
```
❌ Failed to load AI Agent: type 'Null' is not a subtype of type 'Map<String, dynamic>'
```

## 最佳实践

### 1. 文档化数据格式

在代码中添加注释：
```dart
/// Backend returns flat structure:
/// {
///   "toxicity_level": 5,
///   "enthusiasm_level": 5,
///   ...
/// }
factory AIAgent.fromJson(Map<String, dynamic> json) {
  ...
}
```

### 2. 处理可选字段

总是使用 `??` 提供默认值：
```dart
toxicityLevel: json['toxicity_level'] as int? ?? 5,
```

### 3. 类型安全

使用显式类型转换：
```dart
json['field'] as int?  // 而不是 json['field']
```

### 4. 列表处理

安全地处理列表：
```dart
(json['list'] as List<dynamic>?)
    ?.map((e) => e as String)
    .toList() ??
[]
```

## 状态

✅ 已修复
✅ 可以测试

## 相关问题

- `probs/AI_Agent_Null处理修复.md` - 处理 null 值
- `probs/AI_Agent_Extension类型修复.md` - 修复认证问题

## 下一步

1. 热重载客户端
2. 创建 AI Agent
3. 验证可以正常加载
4. 测试更新和删除功能
