# OBS Overlay (web)

Собирается Vite-ом в `apps/obs-overlay/dist`.

Оверлей должен:
- брать состояние только с локального сервера: `GET /api/state` и `ws://<localhost>/ws`;
- использовать общие типы `GameState` из `packages/shared/types`.

`apps/desktop-host` (Tauri) в дальнейшем будет:
- встраивать/раздавать `apps/obs-overlay/dist`;
- поднимать websocket `/ws` и `GET /api/state`.

