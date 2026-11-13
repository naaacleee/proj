# План работ и шпаргалка команд

> Составлено 13.11.2026 по итогам запуска. Сервер и приложение работают.
> Ветка: `arena/5e984311-proj`.

---

## План работ (от владельца продукта, 13.11.2026)

Всё ниже — пожелания к следующим этапам. Порядок обсуждаем отдельно.

1. **Персонализация нейросети.** Настраиваться под конкретного пользователя: запоминать
   стиль общения, интересы, особенности; подстраивать тон и сложность объяснений.
2. **Выбор характера нейросети.** Пользователь выбирает характер/персонажа наставника
   (несколько вариантов). Арта пока нет — заглушки, архитектура под подключаемые ассеты.
3. **Менюшка с вариантами в чате.** Иногда ИИ отвечает не только текстом, но и показывает
   меню с вариантами на выбор (кнопки-ответы/действия) — для быстрых решений.
4. **ИИ записывает цели и расписание.** Из чата по командам пользователя («запиши цель…»,
   «поставь задачу на завтра…») ИИ сам наполняет разделы «Цели» и «Расписание».
5. **Дизайн — улучшить «в сотню раз».** Сильно переработать интерфейс приложения.
6. **Доделать всё, что уже есть.** Довести существующие функции до ума (в т.ч. бэкапы,
   экраны «Цели»/«Расписание» и пр.).

---

## Шпаргалка команд

### Сервер (VPS, папка `/root/mentor/deploy`)

```bash
# Статус — что запущено и здорово ли
docker compose -f docker-compose.prod.yml ps

# Запуск всего сервера (с нейросетью)
docker compose -f docker-compose.prod.yml --profile ai up -d

# Выключить сервер (сайт и приложение перестанут работать)
docker compose -f docker-compose.prod.yml --profile ai down

# Перезапуск без пересборки
docker compose -f docker-compose.prod.yml --profile ai restart

# Логи (Ctrl+C — выйти из просмотра, сервер продолжит работать)
docker compose -f docker-compose.prod.yml logs -f --tail 50
docker compose -f docker-compose.prod.yml logs -f llama    # только нейросеть
docker compose -f docker-compose.prod.yml logs -f backend  # только бэкенд

# После обновления кода (git pull) — пересобрать и запустить
cd /root/mentor && git pull && cd deploy
docker compose -f docker-compose.prod.yml --profile ai up -d --build

# Проверка, что всё живо
curl https://isotopia.duckdns.org/healthz     # ответ: ok

# Бэкап базы вручную
bash scripts/backup-db.sh
```

Сервер сам поднимается после перезагрузки VPS (`restart: unless-stopped`).

### Ноутбук (сборка APK, папка `...\Desktop\bs\proj-arena-5e984311-proj\android`)

```powershell
cd C:\Users\Lenovo\Desktop\bs\proj-arena-5e984311-proj\android
$env:Path += ";C:\Users\Lenovo\Desktop\bs\gradle-8.11.1\bin"
gradle assembleDebug
```
Готовый APK: `app\build\outputs\apk\debug\app-debug.apk`

### Проверка чата с ИИ из консоли (на VPS)

```bash
curl -s -X POST https://isotopia.duckdns.org/api/v1/auth/login -H "Content-Type: application/json" -d '{"username":"НИК","password":"ПАРОЛЬ"}' -o /root/mentor/deploy/login.json
TOKEN=$(python3 -c "import json;print(json.load(open('/root/mentor/deploy/login.json'))['access_token'])")
curl -N -X POST https://isotopia.duckdns.org/api/v1/mentor/chat/stream -H "Content-Type: application/json" -H "Authorization: Bearer $TOKEN" -d '{"message":"Привет!"}'
```

---

## Полезные факты

- Сервер: `https://isotopia.duckdns.org` (HTTPS, сертификат автоматический).
- Модель: Qwen3-1.7B Q4_K_M (быстрая). Замена на Qwen3-4B — один файл `model.gguf`.
- Учётки в приложении: ник 3–32 символа (латиница, цифры, `_` `-` `.`), пароль от 8 символов.
- Бэкапы: `/var/backups/mentor/`, cron 03:30, хранение 14 дней.
- API регистрация/вход: `username` + `password` (не email!).
