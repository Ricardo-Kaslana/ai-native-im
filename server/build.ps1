# Build script for AI-Native IM server
# This script ensures PROTOC is set correctly before building

# Set PROTOC environment variable
$env:PROTOC = ".\protoc.exe"

# Build the project
cargo build $args