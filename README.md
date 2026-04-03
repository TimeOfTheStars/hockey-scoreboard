# hockey-scoreboard

Монорепозиторий:

- [`apps/obs-overlay`](apps/obs-overlay) — OBS-оверлей (Vite + React).
- [`apps/desktop-host`](apps/desktop-host) — Tauri: проверка внешнего API, предпросмотр, **локальный gateway** для OBS (`/api/state`, `/ws`, статика оверлея).
- [`packages/shared/types/gameState.ts`](packages/shared/types/gameState.ts) — тип `GameState`.

Нужны **Node.js** и **Rust** (`cargo`) для desktop-host. Если при `npm run dev` видите `failed to run cargo metadata`: установите Rust с [rustup.rs](https://rustup.rs), откройте **новый** терминал или выполните `source ~/.cargo/env`. Скрипт `apps/desktop-host/scripts/tauri-run.sh` подхватывает `~/.cargo/env` автоматически.

## OBS

1. В корне: `npm install` и `npm --prefix apps/desktop-host install`.
2. `npm run dev` — откроется Tauri-приложение.
3. «Проверить данные» → «Запуск сервера».
4. В OBS Browser Source — URL из зелёного блока (часто `http://127.0.0.1:8787/`).

Подробнее: [apps/desktop-host/README.md](apps/desktop-host/README.md).

## Только оверлей (dev)

```bash
npm --prefix apps/obs-overlay install
npm --prefix apps/obs-overlay run dev
```

Прод-сборка оверлея настроена на работу с **локальным** gateway (same-origin `/api/state` и `/ws`): см. `apps/obs-overlay/.env.production`.

## Если нет стилей Tailwind

1. Перезапустите dev-сервер и откройте тот порт, который показал Vite.
2. Жёсткое обновление страницы: `Cmd+Shift+R` / `Ctrl+Shift+R`.
3. Tailwind подключён в [`apps/obs-overlay/vite.config.ts`](apps/obs-overlay/vite.config.ts) и [`apps/desktop-host/vite.config.ts`](apps/desktop-host/vite.config.ts).

## Сборка

- Оверлей: `npm run build:obs-overlay` (из корня) или `npm --prefix apps/obs-overlay run build`.
- Установщик desktop: `npm run build:desktop`.
