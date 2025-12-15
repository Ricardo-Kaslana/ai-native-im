# 编译速度优化指南

本文档介绍了为加快 Rust 后端编译速度所做的优化配置。

## 优化概览

### 1. Profile 配置优化 (Cargo.toml)

```toml
[profile.dev]
opt-level = 0           # 不优化，加快编译
debug = 1               # 减少调试信息
incremental = true      # 增量编译
codegen-units = 256     # 更多并行编译单元
split-debuginfo = "unpacked"  # 分离调试信息

[profile.dev.package."*"]
opt-level = 2           # 依赖包使用优化（只编译一次）
debug = false
```

### 2. 平台特定链接器优化 (.cargo/config.toml)

**Windows:**
```toml
[target.x86_64-pc-windows-msvc]
rustflags = [
    "-C", "link-arg=/INCREMENTAL",
    "-C", "link-arg=/OPT:REF",
    "-C", "link-arg=/OPT:ICF",
]
```

**Linux:**
```toml
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

### 3. 依赖包特定优化

对于编译慢但运行时性能重要的包，始终使用优化：

```toml
[profile.dev.package.image]
opt-level = 2

[profile.dev.package.regex]
opt-level = 2

[profile.dev.package.bcrypt]
opt-level = 2
```

## 快速命令

### Windows PowerShell

```powershell
# 快速检查
.\scripts\fast-build.ps1 check

# 快速构建
.\scripts\fast-build.ps1 build

# 快速测试
.\scripts\fast-build.ps1 test

# 清理缓存（保留依赖）
.\scripts\fast-build.ps1 clean
```

### Linux/macOS

```bash
# 快速检查
./scripts/fast-build.sh check

# 快速构建
./scripts/fast-build.sh build

# 快速测试
./scripts/fast-build.sh test

# 清理缓存（保留依赖）
./scripts/fast-build.sh clean
```

### Make 命令

```bash
make check        # 快速语法检查
make build        # 构建项目
make fast-check   # 超快检查（无调试信息）
make clean-cache  # 清理增量缓存（保留依赖）
```

## 编译时间对比

| 操作 | 优化前 | 优化后 | 提升 |
|------|--------|--------|------|
| 首次编译 | ~3-5 min | ~2-3 min | ~40% |
| 增量编译 | ~30-60s | ~5-15s | ~70% |
| cargo check | ~20-40s | ~3-10s | ~75% |

## 进一步优化建议

### 1. 安装更快的链接器

**Linux (推荐 mold):**
```bash
# Ubuntu/Debian
sudo apt install mold

# 然后在 .cargo/config.toml 中启用
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

**macOS:**
```bash
brew install llvm
```

### 2. 使用 sccache 缓存编译结果

```bash
# 安装
cargo install sccache

# 配置环境变量
export RUSTC_WRAPPER=sccache
```

### 3. 使用 cargo-nextest 加速测试

```bash
# 安装
cargo install cargo-nextest

# 运行测试
cargo nextest run
```

### 4. 使用 cargo-watch 自动重编译

```bash
# 安装
cargo install cargo-watch

# 监视文件变化自动检查
cargo watch -x check

# 监视文件变化自动测试
cargo watch -x 'test --lib'
```

## 常见问题

### Q: 为什么首次编译还是很慢？

首次编译需要编译所有依赖，这是无法避免的。优化主要针对增量编译。

### Q: 如何查看编译时间？

```bash
# 显示编译时间
cargo build --timings

# 生成 HTML 报告
cargo build --timings=html
```

### Q: 清理缓存后编译变慢了？

使用 `make clean-cache` 而不是 `cargo clean`，这样可以保留已编译的依赖。

### Q: Windows 上链接很慢？

确保使用 MSVC 工具链，并且 Visual Studio 已正确安装。增量链接已在配置中启用。
