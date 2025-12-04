#!/bin/bash

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Configuration
BACKUP_DIR=${BACKUP_DIR:-./backups}
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
POSTGRES_CONTAINER=${POSTGRES_CONTAINER:-ai-im-postgres}
REDIS_CONTAINER=${REDIS_CONTAINER:-ai-im-redis}

echo -e "${GREEN}📦 Starting backup process...${NC}"
echo ""

# Create backup directory
mkdir -p $BACKUP_DIR

# Backup PostgreSQL
echo "💾 Backing up PostgreSQL..."
docker exec $POSTGRES_CONTAINER pg_dump -U prod_user ai_im_prod | gzip > $BACKUP_DIR/postgres_${TIMESTAMP}.sql.gz

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ PostgreSQL backup completed${NC}"
    echo "   File: $BACKUP_DIR/postgres_${TIMESTAMP}.sql.gz"
else
    echo -e "${YELLOW}⚠️  PostgreSQL backup failed${NC}"
fi

echo ""

# Backup Redis
echo "💾 Backing up Redis..."
docker exec $REDIS_CONTAINER redis-cli SAVE
docker cp $REDIS_CONTAINER:/data/dump.rdb $BACKUP_DIR/redis_${TIMESTAMP}.rdb

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Redis backup completed${NC}"
    echo "   File: $BACKUP_DIR/redis_${TIMESTAMP}.rdb"
else
    echo -e "${YELLOW}⚠️  Redis backup failed${NC}"
fi

echo ""

# Cleanup old backups (keep last 7 days)
echo "🧹 Cleaning up old backups..."
find $BACKUP_DIR -name "postgres_*.sql.gz" -mtime +7 -delete
find $BACKUP_DIR -name "redis_*.rdb" -mtime +7 -delete

echo -e "${GREEN}✅ Backup process completed${NC}"
echo ""
echo "Backup files:"
ls -lh $BACKUP_DIR/*_${TIMESTAMP}*
