#!/bin/bash
# Fast test execution script

set -e

echo "🚀 Running tests with optimizations..."

# Check if nextest is installed
if command -v cargo-nextest &> /dev/null; then
    echo "✅ Using cargo-nextest (faster test runner)"
    cargo nextest run --all-features
else
    echo "⚠️  cargo-nextest not found. Install with: cargo install cargo-nextest"
    echo "📦 Using standard cargo test with optimizations..."
    
    # Run with optimizations
    RUST_TEST_THREADS=$(nproc) cargo test \
        --lib \
        --tests \
        --release \
        -- \
        --test-threads=$(nproc) \
        --nocapture
fi

echo "✅ Tests completed!"
