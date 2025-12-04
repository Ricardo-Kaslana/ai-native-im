#!/bin/bash
# Parallel test execution - runs different test suites in parallel

set -e

echo "🔀 Running tests in parallel..."

# Get number of CPU cores
CORES=$(nproc)
echo "📊 Using $CORES CPU cores"

# Function to run a test suite
run_test_suite() {
    local suite=$1
    echo "▶️  Running $suite tests..."
    cargo test --test "$suite" --release -- --test-threads=2 --quiet
    echo "✅ $suite tests completed"
}

export -f run_test_suite

# List of test suites (integration tests)
TEST_SUITES=(
    "auth_service_tests"
    "user_service_tests"
    "admin_service_tests"
    "message_integration_tests"
    "group_service_tests"
    "ai_agent_tests"
)

# Run unit tests first (fastest)
echo "📦 Running unit tests..."
cargo test --lib --release -- --test-threads=$CORES --quiet &
UNIT_PID=$!

# Run integration tests in parallel (limit to avoid resource exhaustion)
echo "🔧 Running integration tests in parallel..."
printf '%s\n' "${TEST_SUITES[@]}" | xargs -P 4 -I {} bash -c 'run_test_suite "$@"' _ {}

# Wait for unit tests to complete
wait $UNIT_PID

echo "✅ All tests completed!"
