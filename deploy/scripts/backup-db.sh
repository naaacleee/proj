#!/usr/bin/env bash
# =============================================================
# Резервное копирование базы данных (Postgres).
# Запуск из каталога deploy/:
#   bash scripts/backup-db.sh
# Хранит дампы в /var/backups/mentor/ 14 дней.
#
# Добавить в cron (ежедневно в 03:30):
#   crontab -e
#   30 3 * * * cd /opt/mentor/deploy && bash scripts/backup-db.sh >> /var/log/mentor-backup.log 2>&1
# =============================================================
set -euo pipefail

BACKUP_DIR=/var/backups/mentor
RETENTION_DAYS=14
mkdir -p "$BACKUP_DIR"

# Пароль Postgres берётся из .env рядом с docker-compose.prod.yml
if [ -f .env ]; then
    # shellcheck disable=SC1091
    set -a; . ./.env; set +a
fi

TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)
FILE="$BACKUP_DIR/mentor_$TIMESTAMP.sql.gz"

echo "==> Дамп базы в $FILE"
docker compose -f docker-compose.prod.yml exec -T db \
    pg_dump -U mentor -d mentor | gzip > "$FILE"

echo "==> Чистка дампов старше $RETENTION_DAYS дней"
find "$BACKUP_DIR" -name 'mentor_*.sql.gz' -mtime "+$RETENTION_DAYS" -delete

echo "==> Готово: $(du -h "$FILE" | cut -f1)"
