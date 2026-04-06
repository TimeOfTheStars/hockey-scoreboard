use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tauri::path::BaseDirectory;
use tauri::AppHandle;
use tauri::Manager;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Json;
use axum::Router;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::{broadcast, RwLock};
use tokio_util::sync::CancellationToken;
use tower_http::services::{ServeDir, ServeFile};

/// Состояние по умолчанию совпадает с `packages/shared/types/gameState.ts` (shallow merge с ответом API).
fn default_state_value() -> Value {
    serde_json::json!({
        "TournamentTitle": "Регулярный турнир по хоккею с шайбой",
        "SeriesInfo": "",
        "BrandingImage": "",
        "TeamA": "A",
        "TeamAFull": "Team A",
        "TeamB": "B",
        "TeamBFull": "Team B",
        "penalty_a": "None",
        "penalty_b": "None",
        "ScoreA": 0,
        "ScoreB": 0,
        "ShotsA": 0,
        "ShotsB": 0,
        "logo_a": "team-a.png",
        "logo_b": "team-b.png",
        "Timer": "20:00",
        "PowerPlayTimer": "02:00",
        "PowerPlayActive": false,
        "Period": 1,
        "Running": false,
        "Visible": true
    })
}

fn shallow_merge(base: Value, patch: &Value) -> Value {
    let mut b = base
        .as_object()
        .cloned()
        .unwrap_or_default();
    if let Some(p) = patch.as_object() {
        for (k, v) in p {
            b.insert(k.clone(), v.clone());
        }
    }
    Value::Object(b)
}

pub fn merge_external_payload(raw: Value) -> Value {
    let patch = if raw.is_array() {
        raw.get(0).cloned().unwrap_or(Value::Null)
    } else {
        raw
    };
    shallow_merge(default_state_value(), &patch)
}

fn overlay_dist_path(app: &AppHandle) -> Result<PathBuf, String> {
    let bundled = app
        .path()
        .resolve("obs-overlay-dist", BaseDirectory::Resource)
        .map_err(|e| format!("путь к ресурсам Tauri: {e}"))?;
    if bundled.join("index.html").is_file() {
        return Ok(bundled);
    }

    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../obs-overlay/dist");
    let index = dev.join("index.html");
    if index.is_file() {
        return Ok(dev);
    }

    Err(format!(
        "Не найден OBS-оверлей (index.html). Ожидалось: {} или dev {}. Собери: npm --prefix apps/obs-overlay run build",
        bundled.display(),
        index.display()
    ))
}

#[derive(Clone)]
struct GatewayInner {
    state: Arc<RwLock<Value>>,
    tx: broadcast::Sender<String>,
}

async fn get_state_json(State(inner): State<GatewayInner>) -> Result<Json<Value>, StatusCode> {
    let st = inner.state.read().await;
    Ok(Json(st.clone()))
}

async fn ws_upgrade(ws: WebSocketUpgrade, State(inner): State<GatewayInner>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws_connected(socket, inner))
}

async fn ws_connected(mut socket: WebSocket, inner: GatewayInner) {
    let mut rx = inner.tx.subscribe();
    let initial = inner.state.read().await.clone();
    let envelope = serde_json::json!({ "type": "state", "payload": initial }).to_string();
    if socket.send(Message::Text(envelope.into())).await.is_err() {
        return;
    }
    let (mut sink, mut stream) = socket.split();
    let mut read = tokio::spawn(async move {
        while let Some(msg) = stream.next().await {
            if msg.is_err() {
                break;
            }
        }
    });
    loop {
        tokio::select! {
            _ = &mut read => break,
            recv = rx.recv() => {
                match recv {
                    Ok(msg) => {
                        if sink.send(Message::Text(msg.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => { /* пропуск устаревших */ }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
}

fn router(inner: GatewayInner, dist: PathBuf) -> Router {
    let index_path = dist.join("index.html");
    let static_service = ServeDir::new(&dist).not_found_service(ServeFile::new(index_path));

    Router::new()
        .route("/api/state", get(get_state_json))
        .route("/ws", get(ws_upgrade))
        .fallback_service(static_service)
        .with_state(inner)
}

async fn poll_loop(
    api_url: String,
    state_holder: Arc<RwLock<Value>>,
    tx: broadcast::Sender<String>,
    cancel: CancellationToken,
) {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => return,
    };
    let mut interval = tokio::time::interval(Duration::from_millis(800));
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = interval.tick() => {
                match client.get(&api_url).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        if let Ok(json) = resp.json::<Value>().await {
                            let merged = merge_external_payload(json);
                            {
                                let mut w = state_holder.write().await;
                                *w = merged.clone();
                            }
                            let envelope = serde_json::json!({ "type": "state", "payload": merged }).to_string();
                            let _ = tx.send(envelope);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

pub struct GatewayController {
    cancel: Option<CancellationToken>,
    server_task: Option<tokio::task::JoinHandle<std::io::Result<()>>>,
    poller_task: Option<tokio::task::JoinHandle<()>>,
}

impl GatewayController {
    pub fn new() -> Self {
        Self {
            cancel: None,
            server_task: None,
            poller_task: None,
        }
    }

    pub async fn stop(&mut self) -> Result<(), String> {
        if let Some(c) = self.cancel.take() {
            c.cancel();
        }
        if let Some(h) = self.server_task.take() {
            h.await.map_err(|e| format!("server join: {e}"))?
                .map_err(|e| format!("server: {e}"))?;
        }
        if let Some(h) = self.poller_task.take() {
            h.await.map_err(|e| format!("poller join: {e}"))?;
        }
        Ok(())
    }

    pub async fn start(
        &mut self,
        app_handle: &AppHandle,
        api_url: String,
        port: u16,
        test_mode: bool,
    ) -> Result<String, String> {
        if !test_mode {
            if !api_url.starts_with("http://") && !api_url.starts_with("https://") {
                return Err("URL должен начинаться с http:// или https://".to_string());
            }
        }
        if self.cancel.is_some() {
            self.stop().await?;
        }

        let dist = overlay_dist_path(app_handle)?;
        let dist = std::fs::canonicalize(&dist).map_err(|e| format!("canonicalize dist: {e}"))?;
        if !dist.as_path().join("index.html").is_file() {
            return Err("В obs-overlay/dist нет index.html".to_string());
        }

        let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
            .await
            .map_err(|e| format!("порт {port} недоступен: {e}"))?;
        let bound = listener
            .local_addr()
            .map_err(|e| format!("local_addr: {e}"))?;

        let state = Arc::new(RwLock::new(default_state_value()));
        let (tx, _rx) = broadcast::channel::<String>(32);
        let inner = GatewayInner {
            state: state.clone(),
            tx: tx.clone(),
        };
        let axum_app = router(inner, dist);

        let token = CancellationToken::new();
        let token_serve = token.clone();
        let server_task = tokio::spawn(async move {
            axum::serve(listener, axum_app)
                .with_graceful_shutdown(async move {
                    token_serve.cancelled().await;
                })
                .await
        });

        let poller_task = if test_mode {
            None
        } else {
            let poll_cancel = token.clone();
            let api_for_poll = api_url;
            Some(tokio::spawn(async move {
                poll_loop(api_for_poll, state, tx, poll_cancel).await;
            }))
        };

        self.cancel = Some(token);
        self.server_task = Some(server_task);
        self.poller_task = poller_task;

        Ok(format!("http://127.0.0.1:{}/", bound.port()))
    }
}
