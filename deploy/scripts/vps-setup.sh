#!/usr/bin/env bash
# =============================================================
# Первичная настройка VPS (Ubuntu 22.04/24.04 или Debian 12).
# Запускать от root:  bash vps-setup.sh
# Скрипт безопасно перезапускаем: повторный запуск ничего не ломает.
# =============================================================
set -euo pipefail

echo "==> 1/5 Обновление системы"
apt-get update
DEBIAN_FRONTEND=noninteractive apt-get upgrade -y

echo "==> 2/5 Установка базовых пакетов"
DEBIAN_FRONTEND=noninteractive apt-get install -y \
    ca-certificates curl gnupg git ufw

echo "==> 3/5 Swap-файл 4 ГБ (важно для моделей на VPS с малым ОЗУ)"
if swapon --show | grep -q .; then
    echo "    Swap уже есть — пропускаем"
else
    fallocate -l 4G /swapfile
    chmod 600 /swapfile
    mkswap /swapfile
    swapon /swapfile
    echo '/swapfile none swap sw 0 0' >> /etc/fstab
    # vm.swappiness=10: swap используется только при реальной нехватке
    sysctl -w vm.swappiness=10
    echo 'vm.swappiness=10' > /etc/sysctl.d/99-swap.conf
    echo "    Swap создан"
fi

echo "==> 4/5 Установка Docker (официальный скрипт)"
if command -v docker >/dev/null 2>&1; then
    echo "    Docker уже установлен — пропускаем"
else
    curl -fsSL https://get.docker.com | sh
fi
systemctl enable --now docker

echo "==> 5/5 Фаервол: открываем только SSH, HTTP, HTTPS"
ufw allow OpenSSH
ufw allow 80/tcp
ufw allow 443/tcp
ufw --force enable
ufw status verbose

echo
echo "============================================================"
echo "Готово! Дальше — по инструкции docs/deploy-vps.md:"
echo "  1) клонируйте проект:  git clone https://github.com/naaacleee/proj.git /opt/mentor"
echo "  2) настройте DNS-запись домена на IP этого сервера"
echo "  3) заполните /opt/mentor/deploy/.env"
echo "  4) docker compose -f docker-compose.prod.yml --profile ai up -d --build"
echo "============================================================"
