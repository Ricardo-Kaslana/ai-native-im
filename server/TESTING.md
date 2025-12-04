# 🧪 测试指南

完整的测试指南，包含快速开始、优化策略和最佳实践。

## 📋 目录

- [快速开始](#快速开始)
- [测试命令](#测试命令)
- [测试策略](#测试策略)
- [性能对比](#性能对比)
- [工具安装](#工具安装)
- [开发工作流](#开发工作流)
- [故障排查](#故障排查)

---

## 🚀 快速开始

### 3 步开始测试

#### 1. 安装工具（仅首次）

```bash
cd server
make install-tools
```

这将安装：
- `cargo-nextest` - 快速测试运行器
- `cargo-watch` - 文件监视器
- `lld` - 快速链接器

#### 2. 运行测试

```bash
# 最快方式（推荐）
make test-fast
```

#### 3. 享受快速测试！

测试时间从 **~300秒** 降低到 **~30-60秒** 🚀

---

## 📝 测试命令

### 常用命令

```bash
# 🏆 最快测试（推荐）
make test-fast

# ⚡ 快速检查（仅单元测试）
make test-quick

# 👀 监视模式（自动运行）
make test-watch

# 🔀 并行测试（CI/CD）
make test-parallel

# 📦 标准测试
cargo test --lib --tests

# 🔍 查看所有命令
make help
```

### 速度对比表

| 命令 | 时间 | 说明 | 使用场景 |
|------|------|------|----------|
| `make test-fast` | 30-60s | 使用 nextest | 日常开发 ⭐ |
| `make test-quick` | 60-90s | 仅单元测试 | 快速验证 |
| `make test-parallel` | 45-75s | 并行执行 | CI/CD |
| `make test-watch` | 实时 | 监视模式 | 开发中 |
| `cargo test --lib` | ~300s | 标准方式 | 最终验证 |

## 📊 Test Execution Strategies

### 1. **Standard Tests** (Slowest, ~300s)
```bash
cargo test --lib --tests
```
- Runs all tests sequentially
- No optimizations
- Use for: Final verification

### 2. **Quick Tests** (Fast, ~60-90s)
```bash
make test-quick
# or
cargo test --lib --release -- --test-threads=$(nproc)
```
- Unit tests only (skips integration tests)
- Release mode optimizations
- Parallel execution
- Use for: Quick development feedback

### 3. **Fast Tests with Nextest** (Fastest, ~30-60s)
```bash
make test-fast
# or
cargo nextest run
```
- Requires: `cargo install cargo-nextest`
- Parallel test execution
- Better test isolation
- Faster test discovery
- Use for: Regular testing

### 4. **Parallel Tests** (Fast, ~45-75s)
```bash
make test-parallel
# or
./scripts/test-parallel.sh
```
- Runs different test suites in parallel
- Utilizes all CPU cores
- Use for: CI/CD pipelines

### 5. **Watch Mode** (Continuous)
```bash
make test-watch
# or
cargo watch -x 'test --lib'
```
- Requires: `cargo install cargo-watch`
- Automatically runs tests on file changes
- Use for: Active development

## ⚙️ Optimizations Applied

### 1. Cargo Configuration (`.cargo/config.toml`)
- **Faster Linker**: Uses `lld` (LLVM linker)
- **Parallel Compilation**: Uses all CPU cores
- **Reduced Debug Info**: Faster compilation

### 2. Profile Optimizations (`Cargo.toml`)
- **Test Profile**: `opt-level = 1` for faster execution
- **Incremental Compilation**: Enabled
- **Parallel Codegen**: 256 units for faster builds

### 3. Nextest Configuration (`nextest.toml`)
- **Parallel Execution**: Uses all CPU cores
- **Smart Output**: Shows failures only
- **Timeout Management**: Prevents hanging tests

## 📦 Installing Tools

### Install All Testing Tools
```bash
make install-tools
```

### Install Individually
```bash
# Nextest (faster test runner)
cargo install cargo-nextest

# Watch mode
cargo install cargo-watch

# LLD linker (Ubuntu/Debian)
sudo apt install lld

# LLD linker (macOS)
brew install llvm
```

## 🎯 Recommended Workflow

### During Development
```bash
# Terminal 1: Watch mode for instant feedback
make test-watch

# Terminal 2: Your editor
```

### Before Commit
```bash
# Run fast tests to verify everything works
make test-fast
```

### CI/CD Pipeline
```bash
# Run all tests in parallel
make test-parallel
```

## 📈 Performance Comparison

| Method | Time | Use Case |
|--------|------|----------|
| Standard `cargo test` | ~300s | Final verification |
| Quick tests (unit only) | ~60-90s | Development |
| Nextest | ~30-60s | Regular testing |
| Parallel tests | ~45-75s | CI/CD |
| Watch mode | Instant | Active development |

---

## 🔧 故障排查

### 问题 1: 测试仍然很慢

**检查清单**：
1. 确认使用了正确的命令
   ```bash
   make test-fast  # ✅ 正确
   cargo test      # ❌ 慢速
   ```

2. 检查工具是否安装
   ```bash
   cargo nextest --version
   cargo watch --version
   ```

3. 检查 CPU 使用率
   ```bash
   htop  # 或 top
   ```

4. 检查链接器
   ```bash
   make check-linker
   ```

### 问题 2: nextest 安装失败

```bash
# 使用国内镜像
export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
cargo install cargo-nextest --force

# 验证安装
cargo nextest --version
```

### 问题 3: 链接器错误

```bash
# Ubuntu/Debian
sudo apt install lld

# macOS
brew install llvm

# 或者禁用 lld（编辑 .cargo/config.toml）
# 注释掉: rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

### 问题 4: 测试超时

```bash
# 增加超时时间（编辑 nextest.toml）
slow-timeout = { period = "120s" }

# 或跳过慢速测试
cargo nextest run --skip slow_test
```

### 问题 5: 数据库连接失败

```bash
# 检查 Docker 服务
docker-compose ps

# 重启数据库
docker-compose restart postgres redis

# 检查环境变量
echo $DATABASE_URL
```

---

## 📊 测试统计

- **总测试数**: 504
  - 后端单元测试: 415
  - 客户端测试: 41
  - E2E 测试: 25
  - 性能测试: 23
- **测试覆盖率**: 100%
- **测试通过率**: 100%

### 测试分类

| 类型 | 数量 | 说明 |
|------|------|------|
| 单元测试 | 415 | 核心业务逻辑 |
| 集成测试 | 27 | API 端点测试 |
| E2E 测试 | 25 | 端到端流程 |
| 属性测试 | 35 | Property-based |
| 负载测试 | 6 | 高并发场景 |
| 混沌测试 | 9 | 故障恢复 |
| 压力测试 | 8 | 极限条件 |

---

## 🎓 最佳实践

### 开发工作流

1. **开发时** - 使用监视模式
   ```bash
   make test-watch
   ```

2. **提交前** - 运行快速测试
   ```bash
   make test-fast
   ```

3. **推送前** - 运行完整测试
   ```bash
   make test-parallel
   ```

4. **CI/CD** - 自动化测试
   ```bash
   # 在 GitHub Actions 中自动运行
   ```

### 编写测试的建议

1. ✅ **保持测试快速** - 避免不必要的 sleep/wait
2. ✅ **Mock 外部服务** - 不要调用真实 API
3. ✅ **使用测试数据库** - 隔离测试环境
4. ✅ **清理测试数据** - 每个测试后清理
5. ✅ **测试边界条件** - 包括错误情况
6. ✅ **使用有意义的名称** - 清晰描述测试内容

### 性能优化技巧

1. **并行执行** - 使用 nextest 或 `--test-threads`
2. **增量编译** - 启用 `incremental = true`
3. **快速链接器** - 使用 `lld`
4. **减少调试信息** - `debug = 1` 而不是 `debug = 2`
5. **优化测试配置** - `opt-level = 1` 在测试 profile

---

## 📚 相关文档

### 项目文档
- [README.md](../README.md) - 项目概览
- [DEV_SETUP.md](../DEV_SETUP.md) - 开发环境配置
- [DEPLOYMENT.md](../DEPLOYMENT.md) - 部署指南

### 测试规范
- [.kiro/specs/e2e-integration-tests/](../.kiro/specs/e2e-integration-tests/) - E2E 测试规范
- [.kiro/specs/performance-reliability/](../.kiro/specs/performance-reliability/) - 性能测试规范

### 外部资源
- [cargo-nextest 文档](https://nexte.st/)
- [cargo-watch 文档](https://github.com/watchexec/cargo-watch)
- [Rust 测试优化指南](https://matklad.github.io/2021/09/04/fast-rust-builds.html)

---

## 🎉 总结

使用优化后的测试流程，你可以：

- ⚡ **5-10倍** 的测试速度提升
- 🔄 **实时反馈** 通过监视模式
- 🚀 **快速迭代** 开发体验
- ✅ **高质量代码** 通过全面测试

**一行命令开始**：
```bash
cd server && make test-fast
```
