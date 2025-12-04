#!/bin/bash
# Quick test script - runs only unit tests, skips integration tests

set -e

echo "⚡ Running quick unit tests only..."

# Run only library tests (skip integration tests)
cargo test \
    --lib \
    --release \
    -- \
    --test-threads=$(nproc) \
    --quiet

echo "✅ Quick tests completed!"
