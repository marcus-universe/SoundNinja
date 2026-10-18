//! LAN web UI. Serves the SoundNinja SPA and mirrors the desktop board.
//! Audio stays on the host machine. The Companion HTTP API in `remote` is untouched.

mod patch;
mod server;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{AppHandle, Emitter};
use tokio::sync::{broadcast, oneshot};

const BUS_CAP: usize = 256;
pub const DEFAULT_PORT: u16 = 7332;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebuiStatus {
    pub running: bool,
    pub port: u16,
    pub clients: usize,
    pub token: String,
}

struct RpcWait {
    tx: oneshot::Sender<Result<Value, String>>,
}

struct WebuiInner {
    port: u16,
    token: String,
    running: bool,
    clients: usize,
    seq: u64,
    /// Full board snapshot. `Null` until the desktop host publishes one.
    state: Value,
    theme: Value,
    /// Canonical directory prefixes the file route may read (images, fonts).
    allow: Vec<PathBuf>,
    dev_origin: Option<String>,
}

pub(crate) struct WebuiCtl {
    inner: Mutex<WebuiInner>,
    bus: broadcast::Sender<Arc<str>>,
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
    app: Mutex<Option<AppHandle>>,
    rpc: Mutex<std::collections::HashMap<u64, RpcWait>>,
}

static WEBUI: OnceLock<Arc<WebuiCtl>> = OnceLock::new();

fn ctl() -> Arc<WebuiCtl> {
    WEBUI
        .get_or_init(|| {
            let (bus, _) = broadcast::channel(BUS_CAP);
            Arc::new(WebuiCtl {
                inner: Mutex::new(WebuiInner {
                    port: DEFAULT_PORT,
                    token: String::new(),
                    running: false,
                    clients: 0,
                    seq: 0,
                    state: Value::Null,
                    theme: Value::Null,
                    allow: Vec::new(),
                    dev_origin: None,
                }),
                bus,
                shutdown: Mutex::new(None),
                app: Mutex::new(None),
                rpc: Mutex::new(std::collections::HashMap::new()),
            })
        })
        .clone()
}

fn lock(inner: &Mutex<WebuiInner>) -> std::sync::MutexGuard<'_, WebuiInner> {
    inner.lock().unwrap_or_else(|e| e.into_inner())
}

pub(crate) fn token_matches(got: &str, expected: &str) -> bool {
    let a = got.trim().as_bytes();
    let b = expected.as_bytes();
    if a.len() != b.len() || a.is_empty() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn random_token() -> String {
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0xA5A5);
    seed ^= (std::process::id() as u128) << 17;
    const ALPHA: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";
    let mut out = String::with_capacity(8);
    for _ in 0..8 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        let idx = (seed >> 33) as usize % ALPHA.len();
        out.push(ALPHA[idx] as char);
    }
    out
}

fn dev_origin(app: &AppHandle) -> Option<String> {
    #[cfg(debug_assertions)]
    {
        app.config().build.dev_url.as_ref().map(|u| {
            u.as_str().trim_end_matches('/').to_string()
        })
    }
    #[cfg(not(debug_assertions))]
    {
        let _ = app;
        None
    }
}

fn bump_clients(ctl: &WebuiCtl, delta: isize) {
    let mut inner = lock(&ctl.inner);
    if delta < 0 {
        inner.clients = inner.clients.saturating_sub((-delta) as usize);
    } else {
        inner.clients = inner.clients.saturating_add(delta as usize);
    }
}

fn broadcast(ctl: &WebuiCtl, payload: String) {
    let _ = ctl.bus.send(Arc::from(payload));
}

fn snapshot_message(inner: &WebuiInner) -> String {
    serde_json::json!({
        "type": "snapshot",
        "seq": inner.seq,
        "state": inner.state,
        "theme": inner.theme,
    })
    .to_string()
}

pub(crate) fn current_token(ctl: &WebuiCtl) -> String {
    lock(&ctl.inner).token.clone()
}

pub(crate) fn take_snapshot(ctl: &WebuiCtl) -> String {
    snapshot_message(&lock(&ctl.inner))
}

pub(crate) fn subscribe(ctl: &WebuiCtl) -> broadcast::Receiver<Arc<str>> {
    ctl.bus.subscribe()
}

pub(crate) fn app_handle(ctl: &WebuiCtl) -> Option<AppHandle> {
    ctl.app.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

pub(crate) fn dev_origin_now(ctl: &WebuiCtl) -> Option<String> {
    lock(&ctl.inner).dev_origin.clone()
}

pub(crate) fn note_client(ctl: &WebuiCtl, delta: isize) {
    bump_clients(ctl, delta);
}

/// Canonical path is allowed when it sits inside a published prefix and the
/// extension is an image or font. Audio and databases never match.
pub(crate) fn file_allowed(ctl: &WebuiCtl, raw: &Path) -> Option<PathBuf> {
    let canon = std::fs::canonicalize(raw).ok()?;
    let ext = canon
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    const OK: &[&str] = &[
        "png", "jpg", "jpeg", "gif", "webp", "bmp", "ico", "svg", "woff", "woff2", "ttf", "otf",
    ];
    if !OK.contains(&ext.as_str()) {
        return None;
    }
    let allow = lock(&ctl.inner).allow.clone();
    let ok = allow.iter().any(|prefix| canon.starts_with(prefix));
    if ok {
        Some(canon)
    } else {
        None
    }
}

fn merge_prefixes(inner: &mut WebuiInner, prefixes: &[Value]) {
    for value in prefixes {
        let Some(text) = value.as_str() else { continue };
        let path = PathBuf::from(text);
        let Ok(canon) = std::fs::canonicalize(&path) else { continue };
        if !inner.allow.iter().any(|p| p == &canon) {
            inner.allow.push(canon);
        }
    }
}

fn merge_prefixes_from_paths(inner: &mut WebuiInner, paths: impl Iterator<Item = PathBuf>) {
    let mut seen: HashSet<PathBuf> = inner.allow.iter().cloned().collect();
    for path in paths {
        let parent = path.parent().map(|p| p.to_path_buf()).unwrap_or(path);
        let Ok(canon) = std::fs::canonicalize(&parent) else { continue };
        if seen.insert(canon.clone()) {
            inner.allow.push(canon);
        }
    }
}

pub(crate) fn emit_in(ctl: &WebuiCtl, msg: Value) {
    let app = app_handle(ctl);
    if let Some(app) = app {
        let _ = app.emit("webui_in", msg);
    }
}

pub(crate) async fn wait_rpc(ctl: &WebuiCtl, id: u64) -> Result<Value, String> {
    let (tx, rx) = oneshot::channel();
    {
        let mut map = ctl.rpc.lock().unwrap_or_else(|e| e.into_inner());
        map.insert(id, RpcWait { tx });
    }
    match tokio::time::timeout(std::time::Duration::from_secs(20), rx).await {
        Ok(Ok(result)) => result,
        Ok(Err(_)) => Err("Desktop closed the request".into()),
        Err(_) => {
            let mut map = ctl.rpc.lock().unwrap_or_else(|e| e.into_inner());
            map.remove(&id);
            Err("Desktop did not answer".into())
        }
    }
}

fn status_now(handle: &WebuiCtl) -> WebuiStatus {
    let inner = lock(&handle.inner);
    WebuiStatus {
        running: inner.running,
        port: inner.port,
        clients: inner.clients,
        token: inner.token.clone(),
    }
}

async fn stop_inner(handle: &WebuiCtl) {
    let tx = handle
        .shutdown
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take();
    if let Some(tx) = tx {
        let _ = tx.send(());
    }
    let mut inner = lock(&handle.inner);
    inner.running = false;
    inner.clients = 0;
}

#[tauri::command]
pub async fn webui_start(
    app: AppHandle,
    port: u16,
    token: Option<String>,
) -> Result<WebuiStatus, String> {
    if port == 0 {
        return Err("Port must be between 1 and 65535".into());
    }
    let handle = ctl();
    stop_inner(&handle).await;
    tokio::time::sleep(std::time::Duration::from_millis(40)).await;

    let token = token
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(random_token);
    let origin = dev_origin(&app);
    {
        let mut slot = handle.app.lock().unwrap_or_else(|e| e.into_inner());
        *slot = Some(app);
    }
    {
        let mut inner = lock(&handle.inner);
        inner.port = port;
        inner.token = token;
        inner.running = false;
        inner.clients = 0;
        inner.dev_origin = origin;
    }

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    {
        let mut slot = handle.shutdown.lock().unwrap_or_else(|e| e.into_inner());
        *slot = Some(shutdown_tx);
    }

    server::spawn(handle.clone(), port, shutdown_rx).await?;
    {
        let mut inner = lock(&handle.inner);
        inner.running = true;
        inner.port = port;
    }
    Ok(status_now(&handle))
}

#[tauri::command]
pub async fn webui_stop() -> Result<WebuiStatus, String> {
    let handle = ctl();
    stop_inner(&handle).await;
    Ok(status_now(&handle))
}

#[tauri::command]
pub fn webui_status() -> WebuiStatus {
    status_now(&ctl())
}

#[tauri::command]
pub fn webui_qr_svg(url: String) -> Result<String, String> {
    let trimmed = url.trim();
    if trimmed.is_empty() || trimmed.len() > 2048 {
        return Err("URL is empty".into());
    }
    let code = qrcode::QrCode::new(trimmed.as_bytes()).map_err(|e| e.to_string())?;
    let svg = code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(240, 240)
        .quiet_zone(true)
        .build();
    Ok(svg)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebuiReply {
    pub id: u64,
    pub result: Option<Value>,
    pub error: Option<String>,
}

#[tauri::command]
pub fn webui_reply(id: u64, result: Option<Value>, error: Option<String>) {
    let handle = ctl();
    let wait = {
        let mut map = handle.rpc.lock().unwrap_or_else(|e| e.into_inner());
        map.remove(&id)
    };
    if let Some(wait) = wait {
        let out = match error {
            Some(err) if !err.is_empty() => Err(err),
            _ => Ok(result.unwrap_or(Value::Null)),
        };
        let _ = wait.tx.send(out);
    }
}

/// Host → browsers. `kind` is snapshot | patch | theme | event | allow | host.
#[tauri::command]
pub fn webui_publish(kind: String, payload: Value) -> Result<(), String> {
    let handle = ctl();
    let kind = kind.as_str();
    let mut message = None;
    {
        let mut inner = lock(&handle.inner);
        match kind {
            "snapshot" => {
                inner.state = payload;
                inner.seq = inner.seq.saturating_add(1);
                message = Some(snapshot_message(&inner));
            }
            "patch" => {
                let ops = payload
                    .get("ops")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                if inner.state.is_null() {
                    inner.state = Value::Object(serde_json::Map::new());
                }
                patch::apply_ops(&mut inner.state, &ops);
                inner.seq = inner.seq.saturating_add(1);
                message = Some(
                    serde_json::json!({
                        "type": "patch",
                        "seq": inner.seq,
                        "ops": ops,
                    })
                    .to_string(),
                );
            }
            "theme" => {
                inner.theme = payload.clone();
                message = Some(
                    serde_json::json!({
                        "type": "theme",
                        "theme": payload,
                    })
                    .to_string(),
                );
            }
            "event" => {
                let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let body = payload.get("payload").cloned().unwrap_or(Value::Null);
                message = Some(
                    serde_json::json!({
                        "type": "event",
                        "name": name,
                        "payload": body,
                    })
                    .to_string(),
                );
            }
            "allow" => {
                if let Some(list) = payload.get("prefixes").and_then(|v| v.as_array()) {
                    merge_prefixes(&mut inner, list);
                }
                if let Some(list) = payload.get("files").and_then(|v| v.as_array()) {
                    let paths = list.iter().filter_map(|v| v.as_str().map(PathBuf::from));
                    merge_prefixes_from_paths(&mut inner, paths);
                }
            }
            "host" => {
                let up = payload.get("up").and_then(|v| v.as_bool()).unwrap_or(false);
                let ty = if up { "hostUp" } else { "hostDown" };
                message = Some(serde_json::json!({ "type": ty }).to_string());
            }
            _ => return Err(format!("Unknown publish kind {kind}")),
        }
    }
    if let Some(message) = message {
        broadcast(&handle, message);
    }
    Ok(())
}

/// Commands a browser may ask the desktop to run. Everything else is rejected
/// in the server before it reaches the host webview.
pub(crate) fn rpc_allowed(cmd: &str) -> bool {
    matches!(
        cmd,
        "play_sound"
            | "pause_sound"
            | "resume_sound"
            | "pause_all"
            | "resume_all"
            | "stop_all"
            | "set_sound_volume"
            | "set_output_volume"
            | "set_input_volume"
            | "get_input_volume"
            | "seek_playing"
            | "set_playing_loop"
            | "get_playing_sounds"
            | "get_sound_duration"
            | "get_file_waveform_peaks"
            | "is_sound_pumping"
            | "get_audio_hosts"
            | "get_out_devices"
            | "get_out_devices_host"
            | "get_in_devices"
            | "get_in_devices_host"
            | "get_loopback_devices"
            | "get_asio_device_channels"
            | "set_asio_channel_map"
            | "set_cache_config"
            | "get_cache_stats"
            | "clear_sound_cache"
            | "gif_cache_paths"
            | "get_system_fonts"
            | "read_text_file_abs"
            | "read_file_base64_abs"
            | "list_dir_files_abs"
            | "plugin:os|platform"
            | "plugin:os|version"
            | "plugin:os|arch"
            | "plugin:os|locale"
    )
}

pub(crate) fn action_allowed(store: &str, name: &str) -> bool {
    match store {
        "jsonHandeling" => matches!(
            name,
            "renameSound"
                | "removeSound"
                | "setSoundVolume"
                | "setSoundLoop"
                | "setSoundLoopByPath"
                | "setSoundColor"
                | "setSoundGif"
                | "setSoundTabs"
                | "setSoundColorMany"
                | "setSoundTabsMany"
                | "removeSoundsMany"
                | "reorderSounds"
                | "addTab"
                | "removeTab"
                | "renameTab"
                | "setTabColor"
                | "reorderTabs"
                | "addSeparator"
                | "removeSeparator"
                | "setSeparatorPosition"
                | "updateSeparator"
                | "setTabButtonAlign"
                | "applyBoardLayout"
                | "setTheme"
                | "setCustomCss"
                | "setOutSource"
                | "setStopOnRetrigger"
                | "setOverlapSounds"
                | "setCacheConfig"
                | "setOutputVolume"
                | "setOutputHost"
                | "setAsioChannels"
                | "setUniformButtonHeight"
                | "setAllowReorder"
                | "setGifPlayOnHover"
                | "setPreloadGifs"
                | "setSetting"
                | "setThemeColors"
                | "addTag"
                | "removeTag"
                | "updateTag"
                | "setSoundTags"
                | "toggleSoundTag"
                | "persistSelectedTagIds"
                | "setSortMode"
                | "setActiveSound"
                | "ReturnStatusAll"
                | "undo"
                | "redo"
        ),
        "appSettings" => matches!(
            name,
            "setNavbarSide"
                | "setNavbarTooltips"
                | "setShowTagBadges"
                | "setGifTitleChip"
                | "setBasicButtonColors"
                | "setOutputSource"
                | "setOutputHost"
                | "setOutputVolume"
                | "setInputSource"
                | "setInputHost"
                | "setInputVolume"
                | "setAsioOutputChannels"
                | "setAsioInputChannels"
                | "setAsioChannels"
                | "setHotkeys"
                | "setAppHotkey"
                | "setSoundTriggersGlobal"
                | "setTitlebarMode"
                | "setHideTitlebar"
                | "setCheckUpdatesOnStart"
        ),
        _ => false,
    }
}
