#!/bin/bash
# Fast Build Script for Linux/macOS
# 快速编译脚本
#
# 使用方法:
#   ./scripts/fast-build.sh           # 快速检查
#   ./scripts/fast-build.sh build     # 快速构建
#   ./scripts/fast-build.sh test      # 快速测试
#   ./scripts/fast-build.sh clean     # 清理缓存

set -e

COMMAND=${1:-check}

# 设置环境变量优化编译
export CARGO_INCREMENTAL=1
export RUSTFLAGS="-C debuginfo=1"

# 获取 CPU 核心数
if [[ "$OSTYPE" == "darwin"* ]]; then
    CORES=$(sysctl -n hw.ncpu)
else
    CORES=$(nproc)
fi

echo "🚀 Fast Build - Using $CORES CPU cores"
echo "Command: $COMMAND"

START_TIME=$(date +%s)

case $COMMAND in
    check)
        echo "Running cargo check..."
        cargo check --jobs $CORES
        ;;
    build)
        echo "Running cargo build..."
        cargo build --jobs $CORES
        ;;
    test)
        echo "Running cargo test (lib only)..."
        cargo test --lib --jobs $CORES -- --test-threads=$CORES
        ;;
    clean)
        echo "Cleaning incremental cache..."
        rm -rf target/debug/incremental
        rm -rf target/debug/.fingerprint/ai-im-server*
        echo "Cache cleaned. Dependencies preserved."
        ;;
    release)
        echo "Running release build..."
        cargo build --release --jobs $CORES
        ;;
    *)
        echo "Unknown command: $COMMAND"
        echo "Usage: $0 [check|build|test|clean|release]"
        exit 1
        ;;
esac

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))
echo ""
echo "✅ Completed in ${DURATION} seconds"
