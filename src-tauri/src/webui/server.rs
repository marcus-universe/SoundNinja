use super::{
    action_allowed, app_handle, current_token, dev_origin_now, emit_in, file_allowed, note_client,
    rpc_allowed, subscribe, take_snapshot, token_matches, wait_rpc, WebuiCtl,
};
use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, Request, State,
    },
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use tower_http::compression::CompressionLayer;

#[derive(Deserialize, Default)]
struct TokenQuery {
    t: Option<String>,
}

struct CachedAsset {
    body: Vec<u8>,
    mime: String,
    etag: String,
    immutable: bool,
    /// Vite dev output. Never stored, and never marked immutable.
    dev: bool,
}

struct UrlTarget {
    /// Path with the query removed. Embedded assets are looked up by this.
    path: String,
    /// Path plus query. Vite virtual ids (`?%2Fdesigns%2F…`) need the query.
    fetch: String,
}

fn etag_for(bytes: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    format!("\"{:x}\"", hasher.finish())
}

fn inject_html(html: &str) -> String {
    const TAGS: &str = concat!(
        r#"<script>!function(){var i=console.info.bind(console);console.info=function(){var a=arguments[0];if(typeof a==="string"&&a.indexOf("<Suspense> is an experimental")===0)return;return i.apply(console,arguments)}}()</script>"#,
        r#"<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">"#,
        r#"<meta name="mobile-web-app-capable" content="yes">"#,
        r#"<meta name="apple-mobile-web-app-capable" content="yes">"#,
        r#"<link rel="icon" type="image/png" href="/__sn/icon.png">"#,
        r#"<link rel="apple-touch-icon" href="/__sn/apple-touch-icon.png">"#,
        r#"<script src="/__sn/bridge.js"></script>"#,
    );
    let mut stripped = html.to_string();
    while let Some(start) = stripped.find("<meta name=\"viewport\"") {
        let Some(end) = stripped[start..].find('>') else { break };
        stripped.replace_range(start..start + end + 1, "");
    }
    if let Some(idx) = stripped.find("<head>") {
        let at = idx + "<head>".len();
        let mut out = String::with_capacity(stripped.len() + TAGS.len());
        out.push_str(&stripped[..at]);
        out.push_str(TAGS);
        out.push_str(&stripped[at..]);
        out
    } else {
        format!("{TAGS}{stripped}")
    }
}

fn parse_url_target(raw: &str) -> Option<UrlTarget> {
    let (path, query) = match raw.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (raw, None),
    };
    if path.is_empty()
        || !path.starts_with('/')
        || path.contains("..")
        || path.contains('\\')
        || path.contains('\0')
    {
        return None;
    }
    let fetch = match query {
        Some(query) if !query.is_empty() => format!("{path}?{query}"),
        _ => path.to_string(),
    };
    Some(UrlTarget {
        path: path.to_string(),
        fetch,
    })
}

fn mime_of(path: &str, fallback: &str) -> String {
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext {
        "html" => "text/html; charset=utf-8".into(),
        "js" | "mjs" => "text/javascript; charset=utf-8".into(),
        "css" => "text/css; charset=utf-8".into(),
        "json" => "application/json".into(),
        "svg" => "image/svg+xml".into(),
        "png" => "image/png".into(),
        "jpg" | "jpeg" => "image/jpeg".into(),
        "gif" => "image/gif".into(),
        "webp" => "image/webp".into(),
        "woff2" => "font/woff2".into(),
        "woff" => "font/woff".into(),
        "ttf" => "font/ttf".into(),
        "otf" => "font/otf".into(),
        "ico" => "image/x-icon".into(),
        "map" => "application/json".into(),
        _ => {
            if fallback.is_empty() {
                "application/octet-stream".into()
            } else {
                fallback.to_string()
            }
        }
    }
}

fn is_immutable(path: &str) -> bool {
    path.starts_with("/_nuxt/")
}

fn asset_response(cached: &CachedAsset, headers: &axum::http::HeaderMap) -> Response {
    if !cached.dev {
        if let Some(inm) = headers.get(header::IF_NONE_MATCH).and_then(|v| v.to_str().ok()) {
            if inm == cached.etag {
                return StatusCode::NOT_MODIFIED.into_response();
            }
        }
    }
    // Hashed /_nuxt files can be cached forever. Dev output and HTML revalidate.
    let cache = if cached.dev {
        "no-store"
    } else if cached.immutable {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, &cached.mime)
        .header(header::CACHE_CONTROL, cache)
        .header("X-Content-Type-Options", "nosniff");
    if !cached.dev {
        builder = builder.header(header::ETAG, &cached.etag);
    }
    builder
        .body(Body::from(cached.body.clone()))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn resolve_embedded(app: &tauri::AppHandle, path: &str) -> Option<(Vec<u8>, String)> {
    let resolver = app.asset_resolver();
    let mut candidates = vec![path.to_string()];
    let trimmed = path.trim_start_matches('/');
    if trimmed != path {
        candidates.push(trimmed.to_string());
    }
    if trimmed.is_empty() {
        candidates.push("index.html".into());
    }
    for candidate in candidates {
        if let Some(asset) = resolver.get(candidate) {
            return Some((asset.bytes, asset.mime_type));
        }
    }
    None
}

fn header_str(headers: &axum::http::HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string()
}

/// `Accept: text/css` is how Vite decides to answer a stylesheet with CSS.
/// Without it, `.scss` comes back as a JavaScript module and the browser blocks it.
fn fetch_dev(
    origin: &str,
    target: &str,
    accept: &str,
    sec_fetch_dest: &str,
) -> Result<(u16, String, Vec<u8>), String> {
    let url = format!("{origin}{target}");
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::limited(4))
        .build()
        .map_err(|e| e.to_string())?;
    let mut req = client.get(&url);
    if !accept.is_empty() {
        req = req.header(header::ACCEPT, accept);
    }
    if !sec_fetch_dest.is_empty() {
        req = req.header("sec-fetch-dest", sec_fetch_dest);
    }
    let resp = req.send().map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    let ctype = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let bytes = resp.bytes().map_err(|e| e.to_string())?.to_vec();
    Ok((status, ctype, bytes))
}

fn is_vite_client(path: &str) -> bool {
    path.contains("@vite/client") || path.contains("%40vite/client")
}

/// Vite's dev client opens `ws://127.0.0.1:<hmr-port>`. A phone cannot reach that.
/// This module keeps `updateStyle` so dev CSS still applies, and skips the socket.
fn vite_client_stub() -> CachedAsset {
    const JS: &str = r#"const sheets = new Map()
export function updateStyle(id, content) {
  let style = sheets.get(id)
  if (!style) {
    style = document.createElement("style")
    style.setAttribute("type", "text/css")
    style.setAttribute("data-vite-dev-id", id)
    document.head.appendChild(style)
    sheets.set(id, style)
  }
  style.textContent = content
}
export function removeStyle(id) {
  const style = sheets.get(id)
  if (!style) return
  style.remove()
  sheets.delete(id)
}
const hot = {
  data: {},
  accept() {},
  acceptDeps() {},
  acceptExports() {},
  dispose() {},
  prune() {},
  decline() {},
  invalidate() {},
  on() {},
  off() {},
  send() {},
}
export function createHotContext() { return hot }
export function injectQuery(url, query) {
  if (!url || (url[0] !== "." && url[0] !== "/")) return url
  const pathname = url.replace(/[?#].*$/, "")
  const parsed = new URL(url, "http://vite.dev")
  const extra = parsed.search ? "&" + parsed.search.slice(1) : ""
  return pathname + "?" + query + extra + (parsed.hash || "")
}
export class ErrorOverlay extends HTMLElement {}
"#;
    let body = JS.as_bytes().to_vec();
    CachedAsset {
        etag: etag_for(&body),
        immutable: false,
        dev: true,
        mime: "text/javascript; charset=utf-8".into(),
        body,
    }
}

async fn load_asset(
    state: &AppStateFull,
    target: &UrlTarget,
    headers: &axum::http::HeaderMap,
) -> Result<CachedAsset, StatusCode> {
    let path = if target.path == "/" {
        "/index.html"
    } else {
        target.path.as_str()
    };
    let dev = dev_origin_now(&state.ctl);
    // Remote browsers must not open Vite's HMR socket on 127.0.0.1.
    // The desktop webview talks to Vite directly and still gets hot reload.
    if dev.is_some() && is_vite_client(path) {
        return Ok(vite_client_stub());
    }
    if dev.is_none() {
        let cache = state.cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(hit) = cache.get(path) {
            return Ok(CachedAsset {
                body: hit.body.clone(),
                mime: hit.mime.clone(),
                etag: hit.etag.clone(),
                immutable: hit.immutable,
                dev: false,
            });
        }
    }

    let loaded = if let Some(origin) = dev.clone() {
        let fetch = target.fetch.clone();
        let accept = header_str(headers, "accept");
        let dest = header_str(headers, "sec-fetch-dest");
        let fetched = tokio::task::spawn_blocking(move || fetch_dev(&origin, &fetch, &accept, &dest))
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)?
            .map_err(|_| StatusCode::BAD_GATEWAY)?;
        if fetched.0 >= 400 {
            return Err(StatusCode::NOT_FOUND);
        }
        (fetched.2, fetched.1)
    } else {
        let app = app_handle(&state.ctl).ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
        let path_owned = path.to_string();
        tokio::task::spawn_blocking(move || resolve_embedded(&app, &path_owned))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?
    };

    let mut body = loaded.0;
    // Dev: keep Vite's type. A stylesheet is text/css; a CSS module is text/javascript.
    let mut mime = if dev.is_some() && !loaded.1.is_empty() {
        loaded.1
    } else {
        mime_of(path, &loaded.1)
    };
    let html = path == "/" || path.ends_with(".html") || mime.contains("text/html");
    if html {
        let text = String::from_utf8_lossy(&body);
        body = inject_html(&text).into_bytes();
        mime = "text/html; charset=utf-8".into();
    }
    let is_dev = dev.is_some();
    let cached = CachedAsset {
        etag: etag_for(&body),
        immutable: !is_dev && is_immutable(path) && !html,
        dev: is_dev,
        body,
        mime,
    };
    if !is_dev {
        let mut cache = state.cache.lock().unwrap_or_else(|e| e.into_inner());
        cache.insert(
            path.to_string(),
            CachedAsset {
                body: cached.body.clone(),
                mime: cached.mime.clone(),
                etag: cached.etag.clone(),
                immutable: cached.immutable,
                dev: false,
            },
        );
    }
    Ok(cached)
}

#[derive(Clone)]
struct AppStateFull {
    ctl: Arc<WebuiCtl>,
    cache: Arc<Mutex<HashMap<String, CachedAsset>>>,
}

async fn session(State(state): State<AppStateFull>, Query(q): Query<TokenQuery>) -> StatusCode {
    let expected = current_token(&state.ctl);
    match q.t.as_deref() {
        Some(got) if token_matches(got, &expected) => StatusCode::NO_CONTENT,
        _ => StatusCode::FORBIDDEN,
    }
}

fn png_icon(bytes: &'static [u8]) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/png")
        .header(header::CACHE_CONTROL, "public, max-age=86400")
        .body(Body::from(bytes))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

async fn icon_png() -> Response {
    png_icon(include_bytes!("../../icons/128x128.png"))
}

async fn apple_icon() -> Response {
    png_icon(include_bytes!("../../icons/128x128@2x.png"))
}

async fn favicon() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/vnd.microsoft.icon")
        .header(header::CACHE_CONTROL, "public, max-age=86400")
        .body(Body::from(include_bytes!("../../icons/icon.ico").as_slice()))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

async fn bridge_js() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(include_str!("bridge.js")))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

async fn index(State(state): State<AppStateFull>, req: Request) -> Response {
    serve_path(&state, "/", req.headers()).await
}

async fn asset(State(state): State<AppStateFull>, req: Request) -> Response {
    let raw = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");
    serve_path(&state, raw, req.headers()).await
}

async fn serve_path(state: &AppStateFull, raw: &str, headers: &axum::http::HeaderMap) -> Response {
    let Some(target) = parse_url_target(raw) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    match load_asset(state, &target, headers).await {
        Ok(cached) => asset_response(&cached, headers),
        Err(code) => code.into_response(),
    }
}

async fn user_file(
    State(state): State<AppStateFull>,
    Query(q): Query<FileQuery>,
) -> Response {
    let expected = current_token(&state.ctl);
    if !q.t.as_deref().is_some_and(|got| token_matches(got, &expected)) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Some(raw) = q.p.as_deref().filter(|p| !p.is_empty() && !p.contains('\0')) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(canon) = file_allowed(&state.ctl, PathBuf::from(raw).as_path()) else {
        return StatusCode::FORBIDDEN.into_response();
    };
    let bytes = match tokio::task::spawn_blocking(move || std::fs::read(canon)).await {
        Ok(Ok(bytes)) => bytes,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    let mime = mime_of(raw, "");
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .header(header::CACHE_CONTROL, "private, max-age=3600")
        .header("X-Content-Type-Options", "nosniff")
        .body(Body::from(bytes))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

#[derive(Deserialize, Default)]
struct FileQuery {
    p: Option<String>,
    t: Option<String>,
}

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<AppStateFull>,
    Query(q): Query<TokenQuery>,
) -> Result<Response, StatusCode> {
    let expected = current_token(&state.ctl);
    match q.t.as_deref() {
        Some(got) if token_matches(got, &expected) => {}
        _ => return Err(StatusCode::FORBIDDEN),
    }
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state.ctl)))
}

async fn handle_socket(mut socket: WebSocket, ctl: Arc<WebuiCtl>) {
    note_client(&ctl, 1);
    let mut rx = subscribe(&ctl);
    let (reply_tx, mut reply_rx) = mpsc::unbounded_channel::<String>();
    let hello = take_snapshot(&ctl);
    if socket.send(Message::Text(hello.into())).await.is_err() {
        note_client(&ctl, -1);
        return;
    }
    loop {
        tokio::select! {
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(frame) = on_client_text(&ctl, &text, &reply_tx).await {
                            let _ = socket.send(Message::Text(frame.into())).await;
                        }
                    }
                    Some(Ok(Message::Ping(p))) => {
                        if socket.send(Message::Pong(p)).await.is_err() { break; }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
            Some(frame) = reply_rx.recv() => {
                if socket.send(Message::Text(frame.into())).await.is_err() {
                    break;
                }
            }
            outbound = rx.recv() => {
                match outbound {
                    Ok(msg) => {
                        if socket.send(Message::Text((*msg).to_string().into())).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        let snap = take_snapshot(&ctl);
                        if socket.send(Message::Text(snap.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    }
    note_client(&ctl, -1);
}

async fn on_client_text(
    ctl: &Arc<WebuiCtl>,
    text: &str,
    reply_tx: &mpsc::UnboundedSender<String>,
) -> Result<(), String> {
    let msg: Value = serde_json::from_str(text).map_err(|_| "bad message".to_string())?;
    let kind = msg.get("type").and_then(|v| v.as_str()).unwrap_or("");
    match kind {
        "hello" | "ping" => {
            if kind == "ping" {
                return Err(serde_json::json!({"type":"pong"}).to_string());
            }
            Ok(())
        }
        "action" => {
            let store = msg.get("store").and_then(|v| v.as_str()).unwrap_or("");
            let name = msg.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if !action_allowed(store, name) {
                return Ok(());
            }
            emit_in(
                ctl,
                serde_json::json!({
                    "type": "action",
                    "store": store,
                    "name": name,
                    "args": msg.get("args").cloned().unwrap_or(Value::Array(Vec::new())),
                }),
            );
            Ok(())
        }
        "rpc" => {
            let id = msg.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
            let cmd = msg.get("cmd").and_then(|v| v.as_str()).unwrap_or("");
            let args = msg.get("args").cloned().unwrap_or(Value::Object(serde_json::Map::new()));
            if id == 0 || !rpc_allowed(cmd) {
                return Err(rpc_frame(id, Err("Not available in remote".into())));
            }
            emit_in(
                ctl,
                serde_json::json!({
                    "type": "rpc",
                    "id": id,
                    "cmd": cmd,
                    "args": args,
                }),
            );
            let ctl = Arc::clone(ctl);
            let tx = reply_tx.clone();
            tokio::spawn(async move {
                let result = wait_rpc(&ctl, id).await;
                let _ = tx.send(rpc_frame(id, result));
            });
            Ok(())
        }
        "play" => {
            let id = msg.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            if !id.is_empty() {
                emit_in(ctl, serde_json::json!({ "type": "play", "id": id }));
            }
            Ok(())
        }
        "stop" => {
            let id = msg.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
            emit_in(ctl, serde_json::json!({ "type": "stop", "id": id }));
            Ok(())
        }
        _ => Ok(()),
    }
}

fn rpc_frame(id: u64, result: Result<Value, String>) -> String {
    match result {
        Ok(value) => {
            serde_json::json!({ "type": "rpcResult", "id": id, "result": value }).to_string()
        }
        Err(err) => serde_json::json!({ "type": "rpcResult", "id": id, "error": err }).to_string(),
    }
}

pub async fn spawn(
    ctl: Arc<WebuiCtl>,
    port: u16,
    shutdown_rx: oneshot::Receiver<()>,
) -> Result<(), String> {
    let state = AppStateFull {
        ctl: ctl.clone(),
        cache: Arc::new(Mutex::new(HashMap::new())),
    };
    let pages = Router::new()
        .route("/__sn/bridge.js", get(bridge_js))
        .route("/__sn/icon.png", get(icon_png))
        .route("/__sn/apple-touch-icon.png", get(apple_icon))
        .route("/favicon.ico", get(favicon))
        .route("/__sn/session", get(session))
        .route("/__sn/file", get(user_file))
        .route("/", get(index))
        .route("/index.html", get(index))
        .fallback(get(asset))
        .layer(CompressionLayer::new())
        .with_state(state.clone());

    let ws = Router::new()
        .route("/__sn/ws", get(ws_upgrade))
        .with_state(state);

    let app = ws.merge(pages);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                format!("Port {port} is already in use")
            } else {
                e.to_string()
            }
        })?;

    tauri::async_runtime::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await;
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{is_vite_client, parse_url_target};

    #[test]
    fn vite_client_path_is_stubbed() {
        assert!(is_vite_client("/_nuxt/@vite/client"));
        assert!(is_vite_client("/_nuxt/%40vite/client"));
        assert!(!is_vite_client("/_nuxt/src/app.vue"));
    }

    #[test]
    fn keeps_vite_virtual_query() {
        let raw = "/_nuxt/@id/__x00__virtual:public?%2Fdesigns%2FLogo_Animated.webp";
        let target = parse_url_target(raw).expect("target");
        assert_eq!(target.path, "/_nuxt/@id/__x00__virtual:public");
        assert_eq!(target.fetch, raw);
    }

    #[test]
    fn keeps_stylesheet_query() {
        let raw = "/_nuxt/src/assets/scss/main.scss?direct";
        let target = parse_url_target(raw).expect("target");
        assert_eq!(target.fetch, raw);
    }

    #[test]
    fn rejects_parent_segments() {
        assert!(parse_url_target("/_nuxt/../secret").is_none());
    }
}
