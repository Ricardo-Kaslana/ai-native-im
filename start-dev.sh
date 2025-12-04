#!/bin/bash

echo "🚀 Starting AI Native IM Development Environment..."
echo ""

# Check if Docker is running
if ! docker.exe ps > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker Desktop first."
    exit 1
fi

# Start Docker services
echo "📦 Starting Docker services..."
docker.exe compose up -d

# Wait for services to be ready
echo "⏳ Waiting for services to be ready..."
sleep 5

# Check service status
echo ""
echo "📊 Service Status:"
docker.exe compose ps

echo ""
echo "✅ Development environment is ready!"
echo ""
echo "📝 Next steps:"
echo "   1. cd server"
echo "   2. cargo run          # Start backend server"
echo "   3. cargo test         # Run tests"
echo ""
echo "🌐 Services:"
echo "   - Backend:    http://localhost:8080"
echo "   - PostgreSQL: localhost:5432"
echo "   - Redis:      localhost:6379"
echo "   - Qdrant:     localhost:6333"
