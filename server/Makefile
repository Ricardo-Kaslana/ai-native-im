# Makefile for common development tasks

.PHONY: help test test-fast test-quick test-parallel test-watch install-tools clean

help:
	@echo "Available commands:"
	@echo "  make test          - Run all tests (standard)"
	@echo "  make test-fast     - Run tests with nextest (fastest)"
	@echo "  make test-quick    - Run only unit tests (quick check)"
	@echo "  make test-parallel - Run tests in parallel"
	@echo "  make test-watch    - Watch mode for tests"
	@echo "  make install-tools - Install testing tools"
	@echo "  make clean         - Clean build artifacts"

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

# Clean build artifacts
clean:
	cargo clean
	@echo "✅ Cleaned build artifacts"

# Check if lld linker is available
check-linker:
	@if command -v lld >/dev/null 2>&1; then \
		echo "✅ lld linker is installed"; \
	else \
		echo "⚠️  lld linker not found. Install with:"; \
		echo "   Ubuntu/Debian: sudo apt install lld"; \
		echo "   macOS: brew install llvm"; \
	fi
