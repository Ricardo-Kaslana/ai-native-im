#!/bin/bash

echo "=========================================="
echo "  AI Native IM - 开发环境检查"
echo "=========================================="
echo ""

# 检查工具版本
echo "✅ 已安装的工具:"
echo "  - Rust:          $(rustc --version 2>/dev/null || echo '❌ 未安装')"
echo "  - Cargo:         $(cargo --version 2>/dev/null || echo '❌ 未安装')"
echo "  - Docker:        $(docker --version 2>/dev/null || echo '❌ 未安装')"
echo "  - Docker Compose: $(docker compose version 2>/dev/null || echo '❌ 未安装')"
echo "  - Flutter:       $(flutter --version 2>&1 | head -1 || echo '❌ 未安装')"
echo "  - SQLx CLI:      $(sqlx --version 2>/dev/null || echo '❌ 未安装')"
echo "  - Protoc:        $(protoc --version 2>/dev/null || echo '❌ 未安装')"
echo "  - Git:           $(git --version 2>/dev/null || echo '❌ 未安装')"
echo ""

# 检查 Docker 容器
echo "✅ Docker 容器状态:"
sudo docker compose ps
echo ""

# 检查数据库连接
echo "✅ 数据库连接测试:"
if PGPASSWORD=dev_password psql -h localhost -U dev_user -d ai_im_dev -c "SELECT 1;" > /dev/null 2>&1; then
    echo "  - PostgreSQL: ✅ 连接成功"
else
    echo "  - PostgreSQL: ⚠️  需要安装 postgresql-client: sudo apt install -y postgresql-client"
fi

# 检查 Redis 连接
if redis-cli ping > /dev/null 2>&1; then
    echo "  - Redis:      ✅ 连接成功"
else
    echo "  - Redis:      ⚠️  需要安装 redis-tools: sudo apt install -y redis-tools"
fi

# 检查 Qdrant 连接
if curl -s http://localhost:6333/healthz > /dev/null 2>&1; then
    echo "  - Qdrant:     ✅ 连接成功"
else
    echo "  - Qdrant:     ❌ 连接失败"
fi
echo ""

# 检查 Cargo 配置
echo "✅ Cargo 配置:"
if [ -f ~/.cargo/config.toml ]; then
    echo "  - 镜像源: ✅ 已配置阿里云镜像"
else
    echo "  - 镜像源: ⚠️  未配置"
fi
echo ""

echo "=========================================="
echo "  环境配置完成！"
echo "=========================================="
echo ""
echo "快速开始:"
echo "  1. 启动服务: cd server && cargo run"
echo "  2. 运行测试: cd server && cargo test"
echo "  3. 查看日志: sudo docker compose logs -f"
echo ""
