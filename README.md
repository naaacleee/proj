# Ментор — персональный ИИ-наставник

Приложение персонального ИИ-наставника для саморазвития, целей, дисципины и полезных привычек.
**Клиент — Kotlin (Android), бэкенд — Rust, ИИ — локальные самохостируемые модели (без внешних AI-сервисов).**

Документация проекта:
- [docs/architecture.md](docs/architecture.md) — концепция, подсистемы, архитектура, этапы, риски;
- [docs/ai-local-stack.md](docs/ai-local-stack.md) — локальный ИИ-стек: модели, скорости, ограничения, путь к «своей» нейросети.

**Текущий статус: Фаза 0 (каркас).** MVP-цикл (знакомство → цель → план → задачи → проверка → отчёт) — в Фазе 1.

---

## Структура репозитория

```
backend/            Rust-бэкенд (Cargo workspace, модульный монолит)
  crates/api        HTTP-сервер (axum): auth, диалог с наставником, healthchecks
  crates/core       домен: пароли (argon2id), JWT, refresh-токены с ротацией
  crates/db         PostgreSQL (sqlx), миграции, репозитории
  crates/llm        LlmProvider: OpenAI-совместимый клиент локальных моделей
android/            Android-клиент (Kotlin, Jetpack Compose, feature-модули)
  app               приложение, навигация, сервис-локатор
  core/ui           тема, общие компоненты
  core/network      Retrofit + SSE-клиент, токены, настройки сервера
  feature/auth      регистрация/вход
  feature/main      главный экран: Главная, Чат, Цели, Расписание, Настройки
deploy/             docker-compose (Postgres+pgvector, Redis, llama.cpp, backend)
docs/               документация
```

---

## Быстрый старт (Windows + WSL2)

### 1. Бэкенд

```bash
cd deploy
docker compose up -d db redis          # база и кэш
cd ../backend
cp .env.example .env                   # при необходимости поправьте
cargo run -p mentor-api                # миграции применяются автоматически
```

Проверка:

```bash
curl http://127.0.0.1:8080/healthz
curl -X POST http://127.0.0.1:8080/api/v1/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"alice","password":"supersecret123"}'
```

### 2. Локальная нейросеть (наставник)

Под 16 ГБ ОЗУ рекомендуется **Qwen3-4B (Q4_K_M)** — быстрая и компактная; при закрытых приложениях можно поставить Qwen3-8B (качество диалога выше). Подробности — в [docs/ai-local-stack.md](docs/ai-local-stack.md).

**Вариант A — llama.cpp (в compose):**
1. Скачайте GGUF-файл модели Q4_K_M (например, `Qwen3-4B-Instruct-GGUF` на Hugging Face) и сохраните как `deploy/models/model.gguf` (файлы моделей в git не попадают).
2. `docker compose --profile ai up llama` — OpenAI-совместимый API на `:8081`.

**Вариант B — Ollama (проще на Windows):**
1. Установите [Ollama](https://ollama.com), выполните `ollama pull qwen3:4b`.
2. В `.env` укажите `LLM_BASE_URL=http://127.0.0.1:11434` и `LLM_CHAT_MODEL=qwen3:4b`.

Диалог с проверкой:

```bash
curl -X POST http://127.0.0.1:8080/api/v1/mentor/chat \
  -H "Authorization: Bearer <access_token>" \
  -H 'Content-Type: application/json' \
  -d '{"message":"Привет! Помоги мне составить план на неделю."}'
```

### 3. Android-приложение

1. Откройте папку `android/` в **Android Studio** (JDK 17).
2. Если Gradle-wrapper отсутствует — выполните в `android/`: `gradle wrapper --gradle-version 8.11.1`.
3. Запустите приложение на эмуляторе (бэкенд доступен как `http://10.0.2.2:8080`) или на телефоне (в настройках приложения укажите LAN-IP ноутбука, например `http://192.168.x.x:8080`).

Что уже работает в приложении: регистрация/вход, чат с наставником (стриминг ответов локальной модели), настройки адреса сервера. Остальные разделы — честные заглушки до Фазы 1.

---

## Развёртывание на VPS (продакшен)

Полная пошаговая инструкция установки на VPS с HTTPS, нейросетью и бэкапами: **[docs/deploy-vps.md](docs/deploy-vps.md)**.
Продакшен-конфигурация: `deploy/docker-compose.prod.yml` (наружу — только Caddy с HTTPS), секреты — в `deploy/.env` (шаблон: `deploy/.env.example`).

---

## API (Фаза 0)

| Метод | Путь | Описание |
|-------|------|----------|
| GET | `/healthz` | живость процесса |
| GET | `/readyz` | готовность (проверка БД) |
| POST | `/api/v1/auth/register` | регистрация (username, password) |
| POST | `/api/v1/auth/login` | вход → access + refresh токены |
| POST | `/api/v1/auth/refresh` | ротация refresh-токена |
| POST | `/api/v1/auth/logout` | отзыв refresh-токена |
| GET | `/api/v1/me` | текущий пользователь (Bearer) |
| POST | `/api/v1/mentor/chat` | диалог с наставником (JSON) |
| POST | `/api/v1/mentor/chat/stream` | диалог со стримингом (SSE) |

---

## Тесты и CI

```bash
cd backend && cargo test --workspace
```

CI (GitHub Actions): сборка и тесты бэкенда, пробная сборка Android. Android-job пока не блокирующий.

## Честные ограничения текущего этапа

- Код бэкенда и клиента написан как каркас Фазы 0 и не проверен компилятором в среде разработки этого чата — собирайте по инструкции выше; ошибки сборки присылайте, чтобы поправить.
- Локальные модели отвечают медленнее облачных (см. docs/ai-local-stack.md) — стриминг уже реализован.
- TLS отключён для локальной разработки; перед публичным запуском — HTTPS через reverse-proxy и `usesCleartextTraffic=false` в Android.
