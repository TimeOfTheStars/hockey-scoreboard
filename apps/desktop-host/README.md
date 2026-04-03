# Desktop Host (Tauri)

Нативное приложение (Windows / macOS): ввод URL внешнего API, проверка ответа, предпросмотр табло и **локальный gateway** для OBS.

## Gateway (локально)

После «Запуск сервера» поднимается `http://127.0.0.1:<порт>/`:

- раздаётся собранный **OBS overlay** (`apps/obs-overlay/dist`);
- `GET /api/state` — JSON состояния;
- `GET /ws` — websocket с сообщениями `{"type":"state","payload":{...}}`;
- фоновый **poll** внешнего API раз в ~800 ms, merge с дефолтом как на фронте.

**OBS:** Browser Source → URL из зелёного блока в UI (например `http://127.0.0.1:8787/`).

## Разработка

Из каталога `apps/desktop-host`:

```bash
npm install
npm run tauri:dev
```

`tauri.conf.json` перед dev собирает оверлей (`npm run build:overlay`), затем Vite на `http://localhost:5174`.

## Сборка установщика

```bash
npm run tauri:build
```

Требуется установленный Rust (`cargo`) и платформенные зависимости Tauri (Xcode на macOS и т.д.).
