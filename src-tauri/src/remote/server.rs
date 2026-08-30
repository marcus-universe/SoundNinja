use super::{
    bump_clients, emit_command, snapshot_state, state_json, RemoteCtl, RemoteSound,
};
use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, Request, State,
    },
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::oneshot;

#[derive(Clone)]
struct AppState {
    ctl: Arc<RemoteCtl>,
}

#[derive(Deserialize, Default)]
struct AuthQuery {
    token: Option<String>,
}

#[derive(Deserialize)]
struct TriggerBody {
    id: String,
}

#[derive(Deserialize, Default)]
struct StopBody {
    id: Option<String>,
}

#[derive(Deserialize)]
struct WsCmd {
    cmd: String,
    id: Option<String>,
}

fn token_ok(headers: &HeaderMap, query: &AuthQuery, expected: &str) -> bool {
    if let Some(raw) = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()) {
        if let Some(got) = raw.strip_prefix("Bearer ").or_else(|| raw.strip_prefix("bearer ")) {
            if got.trim() == expected {
                return true;
            }
        }
    }
    query
        .token
        .as_deref()
        .map(|t| t.trim() == expected)
        .unwrap_or(false)
}

fn expected_token(ctl: &RemoteCtl) -> Option<String> {
    ctl.inner
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .token
        .clone()
}

fn require_auth(headers: &HeaderMap, query: &AuthQuery, ctl: &RemoteCtl) -> Result<(), StatusCode> {
    match expected_token(ctl) {
        None => Ok(()),
        Some(expected) if token_ok(headers, query, &expected) => Ok(()),
        Some(_) => Err(StatusCode::FORBIDDEN),
    }
}

async fn with_cors(req: Request, next: Next) -> Response {
    let star = HeaderValue::from_static("*");
    if req.method() == Method::OPTIONS {
        let mut res = Response::new(Body::empty());
        *res.status_mut() = StatusCode::NO_CONTENT;
        let h = res.headers_mut();
        h.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, star);
        h.insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET, POST, OPTIONS"),
        );
        h.insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("Authorization, Content-Type"),
        );
        return res;
    }
    let mut res = next.run(req).await;
    res.headers_mut()
        .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, star);
    res
}

async fn info(State(state): State<AppState>) -> impl IntoResponse {
    let requires_token = expected_token(&state.ctl).is_some();
    Json(serde_json::json!({
        "app": "SoundNinja",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": 1,
        "requiresToken": requires_token,
    }))
}

async fn sounds(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
) -> Result<Json<Vec<RemoteSound>>, StatusCode> {
    require_auth(&headers, &query, &state.ctl)?;
    let (sounds, _) = snapshot_state(&state.ctl);
    Ok(Json(sounds))
}

async fn handle_state(
    State(st): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    require_auth(&headers, &query, &st.ctl)?;
    let (sounds, playing) = snapshot_state(&st.ctl);
    Ok(Json(serde_json::json!({
        "sounds": sounds,
        "playing": playing,
    })))
}

async fn trigger_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
    Json(body): Json<TriggerBody>,
) -> Result<StatusCode, StatusCode> {
    require_auth(&headers, &query, &state.ctl)?;
    let id = body.id.trim().to_string();
    if id.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    emit_command(&state.ctl, "trigger", Some(id));
    Ok(StatusCode::NO_CONTENT)
}

async fn trigger_get(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    require_auth(&headers, &query, &state.ctl)?;
    let id = id.trim().to_string();
    if id.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    emit_command(&state.ctl, "trigger", Some(id));
    Ok(StatusCode::NO_CONTENT)
}

async fn stop_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
    body: Option<Json<StopBody>>,
) -> Result<StatusCode, StatusCode> {
    require_auth(&headers, &query, &state.ctl)?;
    let id = body.and_then(|Json(b)| b.id).and_then(|s| {
        let t = s.trim().to_string();
        if t.is_empty() {
            None
        } else {
            Some(t)
        }
    });
    emit_command(&state.ctl, "stop", id);
    Ok(StatusCode::NO_CONTENT)
}

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    require_auth(&headers, &query, &state.ctl)?;
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state)))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    bump_clients(&state.ctl, 1);
    let (sounds, playing) = snapshot_state(&state.ctl);
    let _ = socket
        .send(Message::Text(state_json(&sounds, &playing).into()))
        .await;

    let mut rx = state.ctl.bus.subscribe();
    loop {
        tokio::select! {
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(cmd) = serde_json::from_str::<WsCmd>(&text) {
                            dispatch_ws_cmd(&state.ctl, cmd);
                        }
                    }
                    Some(Ok(Message::Ping(p))) => {
                        let _ = socket.send(Message::Pong(p)).await;
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
            event = rx.recv() => {
                match event {
                    Ok(payload) => {
                        if socket.send(Message::Text(payload.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                }
            }
        }
    }
    bump_clients(&state.ctl, -1);
}

fn dispatch_ws_cmd(ctl: &RemoteCtl, cmd: WsCmd) {
    match cmd.cmd.as_str() {
        "trigger" => {
            let id = cmd.id.and_then(|s| {
                let t = s.trim().to_string();
                if t.is_empty() {
                    None
                } else {
                    Some(t)
                }
            });
            if let Some(id) = id {
                emit_command(ctl, "trigger", Some(id));
            }
        }
        "stop" => {
            let id = cmd.id.and_then(|s| {
                let t = s.trim().to_string();
                if t.is_empty() {
                    None
                } else {
                    Some(t)
                }
            });
            emit_command(ctl, "stop", id);
        }
        _ => {}
    }
}

fn router(ctl: Arc<RemoteCtl>) -> Router {
    let state = AppState { ctl };
    Router::new()
        .route("/api/v1/info", get(info))
        .route("/api/v1/sounds", get(sounds))
        .route("/api/v1/state", get(handle_state))
        .route("/api/v1/trigger", post(trigger_post))
        .route("/api/v1/trigger/{id}", get(trigger_get))
        .route("/api/v1/stop", post(stop_post))
        .route("/api/v1/ws", get(ws_upgrade))
        .layer(middleware::from_fn(with_cors))
        .with_state(state)
}

/// Bind then spawn the axum server. Returns a readable error when the port is taken.
pub async fn spawn(
    ctl: Arc<RemoteCtl>,
    port: u16,
    shutdown_rx: oneshot::Receiver<()>,
) -> Result<(), String> {
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                format!("Port {port} is already in use")
            } else {
                e.to_string()
            }
        })?;

    let app = router(ctl);
    tauri::async_runtime::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await;
    });
    Ok(())
}
