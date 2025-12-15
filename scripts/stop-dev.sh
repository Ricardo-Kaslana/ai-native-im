#!/bin/bash

echo "🛑 Stopping AI Native IM Development Environment..."
echo ""

# Stop Docker services
docker.exe compose down

echo ""
echo "✅ All services stopped."
echo ""
echo "💡 To start again, run: ./start-dev.sh"
echo ""
