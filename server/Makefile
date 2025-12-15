# Makefile for common development tasks
# 开发任务 Makefile

.PHONY: help test test-fast test-quick test-parallel test-watch install-tools clean
.PHONY: check build fast-check fast-build clean-cache

# 获取 CPU 核心数
CORES := $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

help:
	@echo "Available commands:"
	@echo ""
	@echo "🔨 Build Commands:"
	@echo "  make check         - Quick syntax check (fastest)"
	@echo "  make build         - Build the project"
	@echo "  make fast-check    - Ultra-fast check with optimizations"
	@echo "  make fast-build    - Optimized incremental build"
	@echo ""
	@echo "🧪 Test Commands:"
	@echo "  make test          - Run all tests (standard)"
	@echo "  make test-fast     - Run tests with nextest (fastest)"
	@echo "  make test-quick    - Run only unit tests (quick check)"
	@echo "  make test-parallel - Run tests in parallel"
	@echo "  make test-watch    - Watch mode for tests"
	@echo ""
	@echo "🧹 Maintenance:"
	@echo "  make clean         - Clean all build artifacts"
	@echo "  make clean-cache   - Clean only incremental cache (keep deps)"
	@echo "  make install-tools - Install development tools"
	@echo ""
	@echo "Using $(CORES) CPU cores for parallel builds"

# Standard test run
test:
	cargo test --lib --tests

# Fast test with nextest
test-fast:
	@if command -v cargo-nextest >/dev/null 2>&1; then \
		cargo nextest run --all-features; \
	else \
		echo "⚠️  cargo-nextest not installed. Run 'make install-tools' first."; \
		echo "📦 Falling back to optimized cargo test..."; \
		cargo test --lib --tests --release -- --test-threads=$$(nproc); \
	fi

# Quick unit tests only
test-quick:
	cargo test --lib --release -- --test-threads=$$(nproc) --quiet

# Parallel test execution
test-parallel:
	./scripts/test-parallel.sh

# Watch mode for continuous testing
test-watch:
	@if command -v cargo-watch >/dev/null 2>&1; then \
		cargo watch -x 'test --lib'; \
	else \
		echo "⚠️  cargo-watch not installed. Run 'make install-tools' first."; \
	fi

# Install testing tools
install-tools:
	@echo "📦 Installing testing tools..."
	cargo install cargo-nextest
	cargo install cargo-watch
	@echo "✅ Tools installed!"

# ============== Build Commands ==============

# Quick syntax check
check:
	CARGO_INCREMENTAL=1 cargo check --jobs $(CORES)

# Standard build
build:
	CARGO_INCREMENTAL=1 cargo build --jobs $(CORES)

# Ultra-fast check with all optimizations
fast-check:
	CARGO_INCREMENTAL=1 RUSTFLAGS="-C debuginfo=0" cargo check --jobs $(CORES)

# Optimized incremental build
fast-build:
	CARGO_INCREMENTAL=1 cargo build --jobs $(CORES)

# ============== Clean Commands ==============

# Clean all build artifacts
clean:
	cargo clean
	@echo "✅ Cleaned all build artifacts"

# Clean only incremental cache (preserves compiled dependencies)
clean-cache:
	rm -rf target/debug/incremental
	rm -rf target/debug/.fingerprint/ai-im-server*
	rm -rf target/debug/deps/ai_im_server*
	rm -rf target/debug/deps/libai_im_server*
	@echo "✅ Cleaned incremental cache (dependencies preserved)"

# ============== Utility Commands ==============

# Check if lld linker is available
check-linker:
	@if command -v lld >/dev/null 2>&1; then \
		echo "✅ lld linker is installed"; \
	else \
		echo "⚠️  lld linker not found. Install with:"; \
		echo "   Ubuntu/Debian: sudo apt install lld"; \
		echo "   macOS: brew install llvm"; \
		echo "   Windows: comes with Visual Studio"; \
	fi

# Show build stats
stats:
	@echo "Build cache size:"
	@du -sh target/debug 2>/dev/null || echo "No debug build"
	@du -sh target/release 2>/dev/null || echo "No release build"
