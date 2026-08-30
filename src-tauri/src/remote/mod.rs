//! HTTP + WebSocket remote-control server for Companion / HTTP clients.

mod netif;
mod server;

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{AppHandle, Emitter};
use tokio::sync::{broadcast, oneshot};

#[tauri::command]
pub fn get_local_ips() -> Vec<netif::LocalIpInfo> {
    netif::list_local_ips()
}

const BUS_CAP: usize = 64;
pub const DEFAULT_PORT: u16 = 7331;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSound {
    pub id: String,
    pub name: String,
    pub tabs: Vec<String>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteStatus {
    pub running: bool,
    pub port: u16,
    pub clients: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteCommand {
    pub action: String,
    pub id: Option<String>,
}

#[derive(Debug, Default)]
struct RemoteInner {
    sounds: Vec<RemoteSound>,
    playing: Vec<String>,
    token: Option<String>,
    port: u16,
    running: bool,
    clients: usize,
}

struct RemoteCtl {
    inner: Mutex<RemoteInner>,
    bus: broadcast::Sender<String>,
    shutdown: Mutex<Option<oneshot::Sender<()>>>,
    app: Mutex<Option<AppHandle>>,
}

static REMOTE: OnceLock<Arc<RemoteCtl>> = OnceLock::new();

fn ctl() -> Arc<RemoteCtl> {
    REMOTE
        .get_or_init(|| {
            let (bus, _) = broadcast::channel(BUS_CAP);
            Arc::new(RemoteCtl {
                inner: Mutex::new(RemoteInner {
                    port: DEFAULT_PORT,
                    ..RemoteInner::default()
                }),
                bus,
                shutdown: Mutex::new(None),
                app: Mutex::new(None),
            })
        })
        .clone()
}

fn snapshot_state(ctl: &RemoteCtl) -> (Vec<RemoteSound>, Vec<String>) {
    let inner = ctl.inner.lock().unwrap_or_else(|e| e.into_inner());
    (inner.sounds.clone(), inner.playing.clone())
}

fn state_json(sounds: &[RemoteSound], playing: &[String]) -> String {
    serde_json::json!({
        "type": "state",
        "sounds": sounds,
        "playing": playing,
    })
    .to_string()
}

fn emit_command(ctl: &RemoteCtl, action: &str, id: Option<String>) {
    let app = ctl.app.lock().unwrap_or_else(|e| e.into_inner()).clone();
    if let Some(app) = app {
        let _ = app.emit(
            "remote_command",
            RemoteCommand {
                action: action.to_string(),
                id,
            },
        );
    }
}

fn bump_clients(ctl: &RemoteCtl, delta: isize) {
    let mut inner = ctl.inner.lock().unwrap_or_else(|e| e.into_inner());
    if delta < 0 {
        inner.clients = inner.clients.saturating_sub((-delta) as usize);
    } else {
        inner.clients = inner.clients.saturating_add(delta as usize);
    }
}

async fn stop_inner(handle: &RemoteCtl) {
    let tx = handle.shutdown.lock().unwrap_or_else(|e| e.into_inner()).take();
    if let Some(tx) = tx {
        let _ = tx.send(());
    }
    let mut inner = handle.inner.lock().unwrap_or_else(|e| e.into_inner());
    inner.running = false;
    inner.clients = 0;
}

#[tauri::command]
pub async fn remote_start(
    app: AppHandle,
    port: u16,
    token: Option<String>,
) -> Result<RemoteStatus, String> {
    if port == 0 {
        return Err("Port must be between 1 and 65535".into());
    }
    let handle = ctl();
    stop_inner(&handle).await;
    // Let the previous listener release the socket.
    tokio::time::sleep(std::time::Duration::from_millis(40)).await;

    {
        let mut app_slot = handle.app.lock().unwrap_or_else(|e| e.into_inner());
        *app_slot = Some(app);
    }
    {
        let mut inner = handle.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.port = port;
        inner.token = token
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty());
        inner.running = false;
        inner.clients = 0;
    }

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    {
        let mut slot = handle.shutdown.lock().unwrap_or_else(|e| e.into_inner());
        *slot = Some(shutdown_tx);
    }

    server::spawn(handle.clone(), port, shutdown_rx).await?;

    {
        let mut inner = handle.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.running = true;
        inner.port = port;
    }
    Ok(status_now(&handle))
}

#[tauri::command]
pub async fn remote_stop() -> Result<RemoteStatus, String> {
    let handle = ctl();
    stop_inner(&handle).await;
    Ok(status_now(&handle))
}

#[tauri::command]
pub fn remote_status() -> RemoteStatus {
    status_now(&ctl())
}

fn status_now(handle: &RemoteCtl) -> RemoteStatus {
    let inner = handle.inner.lock().unwrap_or_else(|e| e.into_inner());
    RemoteStatus {
        running: inner.running,
        port: inner.port,
        clients: inner.clients,
    }
}

#[tauri::command]
pub fn remote_publish_state(sounds: Vec<RemoteSound>, playing: Vec<String>) -> Result<(), String> {
    let handle = ctl();
    {
        let mut inner = handle.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.sounds = sounds.clone();
        inner.playing = playing.clone();
    }
    let _ = handle.bus.send(state_json(&sounds, &playing));
    Ok(())
}
