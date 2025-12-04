#!/bin/bash
# Analyze which tests are taking the most time

set -e

echo "🔍 Analyzing test execution time..."
echo ""

# Run tests with timing information
cargo test --lib -- --nocapture --test-threads=1 2>&1 | \
    grep -E "test .* \.\.\. ok|test .* \.\.\. FAILED" | \
    head -50

echo ""
echo "💡 Tips to speed up tests:"
echo "  1. Use: cargo test --lib --release"
echo "  2. Use: cargo nextest run (install with: cargo install cargo-nextest)"
echo "  3. Use: make test-quick (unit tests only)"
echo "  4. Reduce sleep() calls in tests"
echo "  5. Mock external services (Redis, databases)"
