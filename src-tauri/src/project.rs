//! Project file (`.sninja`) access.
//!
//! A project is one SQLite file. Every read and write goes through this
//! module on a short-lived, single connection:
//!
//! * A save is ONE transaction. Either the whole board lands or nothing does —
//!   a crash mid-save can no longer leave tabs without sounds or groups
//!   without members. (tauri-plugin-sql hands out pooled connections, so a
//!   `BEGIN` there could not cover the following statements.)
//! * The journal mode is `DELETE`, not WAL. The project is a single file again:
//!   copying `board.sninja` to another machine carries every committed change,
//!   with no `-wal` sidecar left behind.
//! * No connection stays open between calls, so the file can be copied,
//!   renamed or replaced at any time on every platform.

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sqlx::query::Query;
use sqlx::sqlite::{
    SqliteArguments, SqliteConnectOptions, SqliteJournalMode, SqliteRow, SqliteSynchronous,
};
use sqlx::{Column, ConnectOptions, Connection, Row, Sqlite, SqliteConnection, TypeInfo, ValueRef};
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

use crate::gifcache;

/// Serialises project access inside this process. Reads from `gifcache` use
/// their own read-only connection and rely on `busy_timeout` instead.
static PROJECT_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Per-session `.bak` copies are skipped above this size (embedded audio can
/// make a project several GB; the in-file snapshots still cover the layout).
const MAX_BACKUP_BYTES: u64 = 512 * 1024 * 1024;
/// `PRAGMA quick_check` reads every page. Skip it for very large files.
const MAX_CHECK_BYTES: u64 = 256 * 1024 * 1024;
/// Backups at least this large are copied in chunks so the save can report
/// progress; smaller ones take a single `std::fs::copy`.
const BACKUP_CHUNKED_MIN_BYTES: u64 = 8 * 1024 * 1024;
const BACKUP_CHUNK_BYTES: usize = 1024 * 1024;

fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

fn write_options(path: &str) -> SqliteConnectOptions {
    SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Delete)
        .synchronous(SqliteSynchronous::Full)
        .busy_timeout(Duration::from_secs(5))
}

fn read_options(path: &str) -> SqliteConnectOptions {
    SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(false)
        .read_only(true)
        .busy_timeout(Duration::from_secs(5))
}

async fn connect_write(path: &str) -> Result<SqliteConnection, String> {
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(err)?;
        }
    }
    write_options(path)
        .connect()
        .await
        .map_err(|e| format!("Cannot open '{path}': {e}"))
}

fn bind_value<'q>(
    query: Query<'q, Sqlite, SqliteArguments<'q>>,
    value: &'q Value,
) -> Query<'q, Sqlite, SqliteArguments<'q>> {
    match value {
        Value::Null => query.bind(None::<String>),
        Value::Bool(b) => query.bind(i64::from(*b)),
        Value::Number(n) => match n.as_i64() {
            Some(i) => query.bind(i),
            None => query.bind(n.as_f64().unwrap_or(0.0)),
        },
        Value::String(s) => query.bind(s.as_str()),
        other => query.bind(other.to_string()),
    }
}

fn row_to_json(row: &SqliteRow) -> Map<String, Value> {
    let mut out = Map::new();
    for (i, col) in row.columns().iter().enumerate() {
        let value = match row.try_get_raw(i) {
            Ok(raw) if !raw.is_null() => match raw.type_info().name() {
                "INTEGER" | "BOOLEAN" => row.try_get::<i64, _>(i).map(Value::from).ok(),
                "REAL" => row.try_get::<f64, _>(i).map(Value::from).ok(),
                "TEXT" => row.try_get::<String, _>(i).map(Value::from).ok(),
                // Blobs never travel to the webview.
                "BLOB" => None,
                _ => row
                    .try_get::<String, _>(i)
                    .map(Value::from)
                    .or_else(|_| row.try_get::<i64, _>(i).map(Value::from))
                    .or_else(|_| row.try_get::<f64, _>(i).map(Value::from))
                    .ok(),
            },
            _ => None,
        };
        out.insert(col.name().to_string(), value.unwrap_or(Value::Null));
    }
    out
}

async fn table_exists(conn: &mut SqliteConnection, table: &str) -> bool {
    sqlx::query("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?")
        .bind(table)
        .fetch_optional(conn)
        .await
        .map(|r| r.is_some())
        .unwrap_or(false)
}

fn file_stat(path: &Path) -> Option<(i64, i64)> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() {
        return None;
    }
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    Some((meta.len() as i64, mtime))
}

/// Last path segment, splitting on both separators — a Windows path must still
/// yield its file name when the project is opened on Linux.
pub(crate) fn base_name(path: &str) -> String {
    path.rsplit(['/', '\\'])
        .find(|s| !s.is_empty())
        .unwrap_or("sound")
        .to_string()
}

pub(crate) fn safe_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    let trimmed = cleaned.trim().trim_end_matches('.').to_string();
    if trimmed.is_empty() {
        "sound".into()
    } else {
        trimmed
    }
}

fn remove_with_sidecars(path: &str) {
    for suffix in ["", "-wal", "-shm", "-journal"] {
        let _ = std::fs::remove_file(format!("{path}{suffix}"));
    }
}

// ── Inspect ───────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInspect {
    pub exists: bool,
    pub size_bytes: u64,
    /// File opens and passes the integrity check (or is empty / missing).
    pub ok: bool,
    pub error: Option<String>,
    /// No tables yet: a brand-new project file.
    pub is_new: bool,
    /// `settings.formatVersion`. `None` on files last written by a build that
    /// predates format versioning.
    pub format_version: Option<i64>,
    pub app_version: Option<String>,
    pub sound_count: i64,
    pub has_backup: bool,
}

/// Look at a project file without modifying it.
#[tauri::command]
pub async fn project_inspect(path: String) -> Result<ProjectInspect, String> {
    let _guard = PROJECT_LOCK.lock().await;
    Ok(inspect(&path).await)
}

async fn inspect(path: &str) -> ProjectInspect {
    let mut out = ProjectInspect {
        has_backup: Path::new(&format!("{path}.bak")).is_file(),
        ..Default::default()
    };
    let Ok(meta) = std::fs::metadata(path) else {
        out.ok = true;
        out.is_new = true;
        return out;
    };
    out.exists = true;
    out.size_bytes = meta.len();
    if meta.len() == 0 {
        out.ok = true;
        out.is_new = true;
        return out;
    }
    let mut conn = match read_options(path).connect().await {
        Ok(c) => c,
        Err(e) => {
            out.error = Some(e.to_string());
            return out;
        }
    };
    let result: Result<(), sqlx::Error> = async {
        let tables: Vec<String> =
            sqlx::query("SELECT name FROM sqlite_master WHERE type = 'table'")
                .fetch_all(&mut conn)
                .await?
                .iter()
                .filter_map(|r| r.try_get::<String, _>(0).ok())
                .collect();
        let has = |t: &str| tables.iter().any(|n| n == t);
        out.is_new = !has("sounds") && !has("settings") && !has("tabs");
        if meta.len() <= MAX_CHECK_BYTES {
            let check: String = sqlx::query("PRAGMA quick_check(1)")
                .fetch_one(&mut conn)
                .await?
                .try_get(0)?;
            if !check.eq_ignore_ascii_case("ok") {
                return Err(sqlx::Error::Protocol(format!("integrity check: {check}")));
            }
        }
        if has("settings") {
            let rows = sqlx::query(
                "SELECT key, value FROM settings WHERE key IN ('formatVersion', 'appVersion')",
            )
            .fetch_all(&mut conn)
            .await?;
            for row in rows {
                let key: String = row.try_get(0)?;
                let value: Option<String> = row.try_get(1)?;
                match key.as_str() {
                    "formatVersion" => {
                        out.format_version = value.and_then(|v| v.trim().parse().ok())
                    }
                    _ => out.app_version = value,
                }
            }
        }
        if has("sounds") {
            out.sound_count = sqlx::query("SELECT COUNT(*) FROM sounds")
                .fetch_one(&mut conn)
                .await?
                .try_get(0)?;
        }
        Ok(())
    }
    .await;
    let _ = conn.close().await;
    match result {
        Ok(()) => out.ok = true,
        Err(e) => out.error = Some(e.to_string()),
    }
    out
}

// ── Query ─────────────────────────────────────────────────────────────────────

/// Run one SELECT and return the rows as JSON objects.
#[tauri::command]
pub async fn project_query(
    path: String,
    sql: String,
    params: Option<Vec<Value>>,
) -> Result<Vec<Map<String, Value>>, String> {
    let _guard = PROJECT_LOCK.lock().await;
    query(&path, &sql, &params.unwrap_or_default()).await
}

pub(crate) async fn query(
    path: &str,
    sql: &str,
    params: &[Value],
) -> Result<Vec<Map<String, Value>>, String> {
    let mut conn = connect_write(path).await?;
    let mut query = sqlx::query(sql);
    for p in params {
        query = bind_value(query, p);
    }
    let rows = query.fetch_all(&mut conn).await.map_err(err);
    let _ = conn.close().await;
    Ok(rows?.iter().map(row_to_json).collect())
}

// ── Commit ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct Statement {
    pub sql: String,
    #[serde(default)]
    pub params: Vec<Value>,
    /// Ignore a failure of this statement (schema upgrades: `ADD COLUMN` on a
    /// column that already exists). The transaction carries on.
    #[serde(default)]
    pub tolerant: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitOptions {
    /// `Some(true)`: store every sound file inside the project.
    /// `Some(false)`: drop stored audio. `None`: leave as is.
    pub embed_audio: Option<bool>,
    /// `Some(true)`: keep GIF/image bytes inside the project.
    /// `Some(false)`: move them to the app's image library. `None`: leave as is.
    pub embed_gifs: Option<bool>,
    /// Copy the file to `<path>.bak` before writing.
    #[serde(default)]
    pub backup: bool,
    /// Emit `project_save_progress` events while the save runs.
    #[serde(default)]
    pub progress: bool,
}

// ── Save progress ─────────────────────────────────────────────────────────────

/// Event the frontend listens to while a save with `progress: true` runs.
const SAVE_PROGRESS_EVENT: &str = "project_save_progress";
/// Minimum gap between two progress events inside one phase.
const PROGRESS_INTERVAL_MS: u64 = 80;
/// Work units are bytes. A statement counts as a small fixed amount, an image
/// as a typical image size, so the phases weigh in roughly by their real cost.
const STATEMENT_UNITS: u64 = 4096;
const IMAGE_UNITS: u64 = 256 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Backup,
    Board,
    Audio,
    Images,
    Finalizing,
}

impl Phase {
    /// In discriminant order, so `Phase::ALL[phase as usize] == phase`.
    const ALL: [Phase; 5] = [
        Phase::Backup,
        Phase::Board,
        Phase::Audio,
        Phase::Images,
        Phase::Finalizing,
    ];

    fn as_str(self) -> &'static str {
        match self {
            Phase::Backup => "backup",
            Phase::Board => "board",
            Phase::Audio => "audio",
            Phase::Images => "images",
            Phase::Finalizing => "finalizing",
        }
    }
}

/// Payload of [`SAVE_PROGRESS_EVENT`]. `done` / `total` / `percent` describe
/// the whole save, not the current phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProgress {
    pub path: String,
    pub phase: &'static str,
    pub done: u64,
    pub total: u64,
    pub percent: u32,
}

struct ProgressState {
    sink: Box<dyn Fn(SaveProgress) + Send + Sync>,
    path: String,
    started: Instant,
    /// Milliseconds after `started` at which the last throttled event went out.
    last_emit_ms: AtomicU64,
    done: AtomicU64,
    /// Expected work per phase, indexed like [`Phase::ALL`]. Their sum is the
    /// total; a phase replaces its entry once it knows its real workload.
    planned: [AtomicU64; 5],
    /// Highest percent reported so far; reports never go below it.
    percent: AtomicU32,
    /// Index into [`Phase::ALL`].
    phase: AtomicUsize,
}

impl ProgressState {
    /// Report the current state. Stays at 99 or below: 100 is reserved for
    /// [`Progress::finish`], after the transaction is on disk.
    fn total(&self) -> u64 {
        self.planned.iter().map(|units| units.load(Ordering::Relaxed)).sum()
    }

    fn emit(&self) {
        let total = self.total();
        let done = self.done.load(Ordering::Relaxed).min(total);
        // Nothing planned yet reads as 0 %, not as a division by zero.
        let ratio = done.saturating_mul(99).checked_div(total).unwrap_or(0);
        self.send(done, total, ratio.min(99) as u32);
    }

    fn send(&self, done: u64, total: u64, percent: u32) {
        // `fetch_max` keeps the reported percent monotonic even when the total
        // is corrected upwards in the middle of a save.
        let percent = self.percent.fetch_max(percent, Ordering::Relaxed).max(percent);
        let phase = Phase::ALL[self.phase.load(Ordering::Relaxed).min(Phase::ALL.len() - 1)];
        (self.sink)(SaveProgress {
            path: self.path.clone(),
            phase: phase.as_str(),
            done,
            total,
            percent,
        });
    }
}

/// Optional progress reporting for one save. Cheap to clone, so the backup
/// copy on the blocking pool can report bytes too. Every method is a no-op
/// when progress was not requested.
#[derive(Clone, Default)]
pub(crate) struct Progress {
    state: Option<Arc<ProgressState>>,
}

impl Progress {
    /// No reporting: nothing is emitted and no pre-pass runs.
    pub(crate) fn none() -> Self {
        Self::default()
    }

    pub(crate) fn new(path: &str, sink: impl Fn(SaveProgress) + Send + Sync + 'static) -> Self {
        Self {
            state: Some(Arc::new(ProgressState {
                sink: Box::new(sink),
                path: path.to_owned(),
                started: Instant::now(),
                last_emit_ms: AtomicU64::new(0),
                done: AtomicU64::new(0),
                planned: Default::default(),
                percent: AtomicU32::new(0),
                phase: AtomicUsize::new(0),
            })),
        }
    }

    fn enabled(&self) -> bool {
        self.state.is_some()
    }

    /// Set the work a phase is expected to do: the estimate before the save
    /// starts, then the real figure once the phase has looked at its input.
    fn plan(&self, phase: Phase, units: u64) {
        if let Some(state) = &self.state {
            state.planned[phase as usize].store(units, Ordering::Relaxed);
        }
    }

    /// Enter a phase. Always emits, so the frontend can label what is running
    /// even when the phase is over before the throttle would let an event out.
    fn begin(&self, phase: Phase) {
        if let Some(state) = &self.state {
            state.phase.store(phase as usize, Ordering::Relaxed);
            state.emit();
        }
    }

    /// Record `units` of finished work; emits at most every
    /// [`PROGRESS_INTERVAL_MS`].
    fn advance(&self, units: u64) {
        let Some(state) = &self.state else { return };
        state.done.fetch_add(units, Ordering::Relaxed);
        let now_ms = state.started.elapsed().as_millis() as u64;
        let last = state.last_emit_ms.load(Ordering::Relaxed);
        if now_ms.saturating_sub(last) < PROGRESS_INTERVAL_MS {
            return;
        }
        // Only the caller that wins the exchange emits for this interval.
        if state
            .last_emit_ms
            .compare_exchange(last, now_ms, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            state.emit();
        }
    }

    /// The save is on disk: report 100 %.
    fn finish(&self) {
        if let Some(state) = &self.state {
            state.phase.store(Phase::Finalizing as usize, Ordering::Relaxed);
            let total = state.total();
            state.done.store(total, Ordering::Relaxed);
            state.send(total, total, 100);
        }
    }
}

/// Wall-clock time per phase for `SN_PERF` output.
struct PhaseTimer {
    started: Instant,
    lap: Instant,
    laps: Vec<(&'static str, f64)>,
}

impl PhaseTimer {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            started: now,
            lap: now,
            laps: Vec::new(),
        }
    }

    /// Close the span since the previous lap under `name`.
    fn lap(&mut self, name: &'static str) {
        self.laps.push((name, crate::perf::ms_since(self.lap)));
        self.lap = Instant::now();
    }

    fn log(&self, path: &str) {
        crate::perf::log(|| {
            let phases: Vec<String> = self
                .laps
                .iter()
                .map(|(name, ms)| format!("{name}={ms:.1}ms"))
                .collect();
            format!(
                "commit {} total={:.1}ms '{path}'",
                phases.join(" "),
                crate::perf::ms_since(self.started)
            )
        });
    }
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitReport {
    pub embedded_audio: u32,
    pub embedded_bytes: u64,
    /// Sounds that could not be embedded: file missing and no stored copy.
    pub missing_audio: Vec<String>,
    pub backup_written: bool,
}

/// Directories used when moving GIFs in and out of a project.
#[derive(Debug, Clone, Default)]
pub(crate) struct GifDirs {
    pub library: Option<PathBuf>,
    pub cache: Option<PathBuf>,
}

/// Apply `statements` as one atomic transaction, then sync embedded media.
#[tauri::command]
pub async fn project_commit(
    app: AppHandle,
    path: String,
    statements: Vec<Statement>,
    options: Option<CommitOptions>,
) -> Result<CommitReport, String> {
    let dirs = GifDirs {
        library: gifcache::library_dir(&app).ok(),
        cache: gifcache::cache_dir(&app).ok(),
    };
    let options = options.unwrap_or_default();
    let _guard = PROJECT_LOCK.lock().await;
    if !options.progress {
        return commit(&path, &statements, &options, &dirs).await;
    }
    let progress = Progress::new(&path, move |update| {
        let _ = app.emit(SAVE_PROGRESS_EVENT, update);
    });
    commit_with(&path, &statements, &options, &dirs, &progress).await
}

/// A save without progress events.
pub(crate) async fn commit(
    path: &str,
    statements: &[Statement],
    options: &CommitOptions,
    dirs: &GifDirs,
) -> Result<CommitReport, String> {
    commit_with(path, statements, options, dirs, &Progress::none()).await
}

/// Size of the file a backup would copy, or `None` when no backup is due
/// (not requested, new/empty file, or too large to duplicate).
fn backup_len(path: &str, options: &CommitOptions) -> Option<u64> {
    if !options.backup {
        return None;
    }
    let len = std::fs::metadata(path).ok()?.len();
    (len > 0 && len <= MAX_BACKUP_BYTES).then_some(len)
}

/// Copy `src` to `dst` through `<dst>.partial` and a rename, so a copy that
/// fails half way leaves the previous backup intact. Files of at least
/// `chunked_min_bytes` are copied in chunks and report each chunk to
/// `on_bytes`; smaller ones report once.
fn backup_file(
    src: &Path,
    dst: &Path,
    chunked_min_bytes: u64,
    mut on_bytes: impl FnMut(u64),
) -> std::io::Result<()> {
    let mut partial = dst.as_os_str().to_owned();
    partial.push(".partial");
    let partial = PathBuf::from(partial);

    let copy = |on_bytes: &mut dyn FnMut(u64)| -> std::io::Result<()> {
        if std::fs::metadata(src)?.len() < chunked_min_bytes {
            on_bytes(std::fs::copy(src, &partial)?);
        } else {
            let mut reader = std::fs::File::open(src)?;
            let mut writer = std::fs::File::create(&partial)?;
            let mut chunk = vec![0u8; BACKUP_CHUNK_BYTES];
            loop {
                let read = reader.read(&mut chunk)?;
                if read == 0 {
                    break;
                }
                writer.write_all(&chunk[..read])?;
                on_bytes(read as u64);
            }
        }
        std::fs::rename(&partial, dst)
    };
    let result = copy(&mut on_bytes);
    if result.is_err() {
        let _ = std::fs::remove_file(&partial);
    }
    result
}

/// Expected audio bytes and image units of a save, from the project as it is
/// on disk before the save. Only an estimate: the statements of this save may
/// add or remove sounds. The phases replace it with their real workload.
async fn estimate_media_work(
    path: &str,
    statements: &[Statement],
    options: &CommitOptions,
) -> (u64, u64) {
    let wants_audio = options.embed_audio == Some(true);
    if !wants_audio && options.embed_gifs.is_none() {
        return (0, 0);
    }

    // Sounds already in the project, with the stamp of their stored copy.
    let mut known: Vec<(String, Option<(i64, i64)>)> = Vec::new();
    let mut images = 0u64;
    let has_file = std::fs::metadata(path).map(|m| m.len() > 0).unwrap_or(false);
    if has_file {
        if let Ok(mut conn) = read_options(path).connect().await {
            if wants_audio && table_exists(&mut conn, "sounds").await {
                let sql = if table_exists(&mut conn, "media").await {
                    "SELECT s.path, m.src_size, m.src_mtime FROM sounds s
                     LEFT JOIN media m ON m.sound_id = s.sound_id
                     WHERE s.sound_id IS NOT NULL AND s.sound_id != ''"
                } else {
                    "SELECT path, NULL, NULL FROM sounds
                     WHERE sound_id IS NOT NULL AND sound_id != ''"
                };
                if let Ok(rows) = sqlx::query(sql).fetch_all(&mut conn).await {
                    known = rows
                        .iter()
                        .filter_map(|r| {
                            let path: String = r.try_get(0).ok()?;
                            let size: Option<i64> = r.try_get(1).ok()?;
                            let mtime: Option<i64> = r.try_get(2).ok()?;
                            Some((path, size.zip(mtime)))
                        })
                        .collect();
                }
            }
            if table_exists(&mut conn, "gif_blobs").await {
                let sql = match options.embed_gifs {
                    Some(true) if table_exists(&mut conn, "sounds").await => Some(
                        "SELECT COUNT(DISTINCT gif_id) FROM sounds
                         WHERE gif_id IS NOT NULL AND gif_id != ''
                           AND gif_id NOT IN (SELECT id FROM gif_blobs)",
                    ),
                    Some(false) => Some("SELECT COUNT(*) FROM gif_blobs"),
                    _ => None,
                };
                if let Some(sql) = sql {
                    if let Ok(row) = sqlx::query(sql).fetch_one(&mut conn).await {
                        images = row.try_get::<i64, _>(0).unwrap_or(0).max(0) as u64;
                    }
                }
            }
            let _ = conn.close().await;
        }
    }
    let mut audio = 0;
    if wants_audio {
        // Sounds this save adds are not in the file yet. Their paths travel as
        // parameters of the statements that write the `sounds` table, so any
        // such parameter that names an existing file is counted as well.
        let mut candidates: Vec<String> = statements
            .iter()
            .filter(|s| s.sql.to_ascii_lowercase().contains("sounds"))
            .flat_map(|s| s.params.iter())
            .filter_map(Value::as_str)
            .filter(|p| p.contains(['/', '\\']))
            .map(str::to_owned)
            .collect();
        candidates.sort_unstable();
        candidates.dedup();
        audio = tauri::async_runtime::spawn_blocking(move || {
            let mut seen: HashSet<&str> = HashSet::new();
            let mut bytes = 0u64;
            for (path, stored) in &known {
                seen.insert(path.as_str());
                if let Some(stat) = file_stat(Path::new(path)) {
                    if *stored != Some(stat) {
                        bytes += stat.0 as u64;
                    }
                }
            }
            for path in &candidates {
                if seen.insert(path.as_str()) {
                    if let Some((size, _)) = file_stat(Path::new(path)) {
                        bytes += size as u64;
                    }
                }
            }
            bytes
        })
        .await
        .unwrap_or(0);
    }
    (audio, images * IMAGE_UNITS)
}

/// [`commit`] with progress reporting. With [`Progress::none`] this does
/// exactly what a plain commit does.
pub(crate) async fn commit_with(
    path: &str,
    statements: &[Statement],
    options: &CommitOptions,
    dirs: &GifDirs,
    progress: &Progress,
) -> Result<CommitReport, String> {
    let mut timer = PhaseTimer::new();
    let mut report = CommitReport::default();
    let backup_bytes = backup_len(path, options);
    if progress.enabled() {
        // The whole workload is laid out before the first event, so the
        // percentage describes the save as a whole from the start.
        let (audio, images) = estimate_media_work(path, statements, options).await;
        progress.plan(Phase::Backup, backup_bytes.unwrap_or(0));
        progress.plan(Phase::Board, statements.len() as u64 * STATEMENT_UNITS);
        progress.plan(Phase::Audio, audio);
        progress.plan(Phase::Images, images);
        timer.lap("estimate");
    }

    if backup_bytes.is_some() {
        progress.begin(Phase::Backup);
        let src = PathBuf::from(path);
        let dst = PathBuf::from(format!("{path}.bak"));
        let reporter = progress.clone();
        // A large project takes seconds to copy; keep that off the async thread.
        report.backup_written = tauri::async_runtime::spawn_blocking(move || {
            backup_file(&src, &dst, BACKUP_CHUNKED_MIN_BYTES, |bytes| reporter.advance(bytes))
        })
        .await
        .is_ok_and(|copied| copied.is_ok());
        timer.lap("backup");
    }

    let mut conn = connect_write(path).await?;
    let mut shrink = false;
    let result: Result<(), String> = async {
        let mut tx = conn.begin().await.map_err(err)?;
        progress.begin(Phase::Board);
        for stmt in statements {
            let mut query = sqlx::query(&stmt.sql);
            for p in &stmt.params {
                query = bind_value(query, p);
            }
            if let Err(e) = query.execute(&mut *tx).await {
                if !stmt.tolerant {
                    return Err(format!("{e} — in: {}", truncate(&stmt.sql, 120)));
                }
            }
            progress.advance(STATEMENT_UNITS);
        }
        timer.lap("board");
        if options.embed_audio == Some(true) {
            progress.begin(Phase::Audio);
            embed_audio(&mut tx, &mut report, progress).await?;
            timer.lap("audio");
        } else if options.embed_audio == Some(false) && table_exists(&mut tx, "media").await {
            let gone = sqlx::query("DELETE FROM media")
                .execute(&mut *tx)
                .await
                .map_err(err)?;
            shrink = gone.rows_affected() > 0;
            timer.lap("audio");
        }
        if options.embed_gifs == Some(true) {
            progress.begin(Phase::Images);
            import_gifs(&mut tx, dirs, progress).await?;
            timer.lap("images");
        } else if options.embed_gifs == Some(false) {
            progress.begin(Phase::Images);
            shrink |= externalize_gifs(&mut tx, dirs, progress).await?;
            timer.lap("images");
        }
        progress.begin(Phase::Finalizing);
        tx.commit().await.map_err(err)
    }
    .await;

    if result.is_ok() && shrink {
        // Reclaim the space the removed blobs held. Best effort.
        let _ = sqlx::query("VACUUM").execute(&mut conn).await;
    }
    let _ = conn.close().await;
    timer.lap("finalize");
    timer.log(path);
    if result.is_ok() {
        progress.finish();
    }
    result.map(|()| report)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}

async fn ensure_media_table(conn: &mut SqliteConnection) -> Result<(), String> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS media (
            sound_id TEXT PRIMARY KEY,
            file_name TEXT NOT NULL,
            byte_len INTEGER NOT NULL,
            src_size INTEGER,
            src_mtime INTEGER,
            data BLOB NOT NULL
        )",
    )
    .execute(conn)
    .await
    .map(|_| ())
    .map_err(err)
}

/// One sound whose file has to be (re)stored in `media`.
struct EmbedJob {
    path: String,
    id: String,
    size: i64,
    mtime: i64,
}

/// Decide per sound whether its file must be stored: `Ok` with the job when
/// the file changed since the stored copy, `Err(path)` when the file is gone
/// and nothing is stored for it. Unchanged files yield nothing.
fn plan_embed(
    sounds: Vec<(String, String)>,
    stored: &HashMap<String, (Option<i64>, Option<i64>)>,
) -> (Vec<EmbedJob>, Vec<String>) {
    let mut jobs = Vec::new();
    let mut missing = Vec::new();
    for (path, id) in sounds {
        match file_stat(Path::new(&path)) {
            None => {
                if !stored.contains_key(&id) {
                    missing.push(path);
                }
            }
            Some((size, mtime)) => {
                if stored.get(&id) != Some(&(Some(size), Some(mtime))) {
                    jobs.push(EmbedJob { path, id, size, mtime });
                }
            }
        }
    }
    (jobs, missing)
}

/// Store each sound's audio file in `media`, keyed by the sound's stable id.
/// Files whose size and mtime match the stored copy are skipped, so a normal
/// save costs one `stat` per sound.
async fn embed_audio(
    conn: &mut SqliteConnection,
    report: &mut CommitReport,
    progress: &Progress,
) -> Result<(), String> {
    ensure_media_table(conn).await?;
    let sounds = sqlx::query(
        "SELECT path, sound_id FROM sounds WHERE sound_id IS NOT NULL AND sound_id != ''",
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(err)?;
    let stored: HashMap<String, (Option<i64>, Option<i64>)> =
        sqlx::query("SELECT sound_id, src_size, src_mtime FROM media")
            .fetch_all(&mut *conn)
            .await
            .map_err(err)?
            .iter()
            .filter_map(|r| {
                Some((
                    r.try_get::<String, _>(0).ok()?,
                    (r.try_get(1).ok()?, r.try_get(2).ok()?),
                ))
            })
            .collect();

    let mut pairs = Vec::with_capacity(sounds.len());
    for row in &sounds {
        let path: String = row.try_get(0).map_err(err)?;
        let id: String = row.try_get(1).map_err(err)?;
        pairs.push((path, id));
    }
    // One hop to the blocking pool for every stat, instead of stalling the
    // async thread once per sound.
    let (jobs, missing, stored) =
        tauri::async_runtime::spawn_blocking(move || {
            let (jobs, missing) = plan_embed(pairs, &stored);
            (jobs, missing, stored)
        })
        .await
        .map_err(err)?;
    report.missing_audio.extend(missing);
    // Now the real workload is known: exactly the files that will be read.
    progress.plan(Phase::Audio, jobs.iter().map(|j| j.size as u64).sum());

    for job in jobs {
        let read_path = job.path.clone();
        let bytes = tauri::async_runtime::spawn_blocking(move || std::fs::read(read_path))
            .await
            .map_err(err)?;
        let bytes = match bytes {
            Ok(b) => b,
            Err(_) => {
                if !stored.contains_key(&job.id) {
                    report.missing_audio.push(job.path);
                }
                progress.advance(job.size as u64);
                continue;
            }
        };
        report.embedded_audio += 1;
        report.embedded_bytes += bytes.len() as u64;
        sqlx::query(
            "INSERT OR REPLACE INTO media (sound_id, file_name, byte_len, src_size, src_mtime, data)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&job.id)
        .bind(base_name(&job.path))
        .bind(bytes.len() as i64)
        .bind(job.size)
        .bind(job.mtime)
        .bind(bytes)
        .execute(&mut *conn)
        .await
        .map_err(err)?;
        progress.advance(job.size as u64);
    }

    sqlx::query(
        "DELETE FROM media WHERE sound_id NOT IN
           (SELECT sound_id FROM sounds WHERE sound_id IS NOT NULL)",
    )
    .execute(&mut *conn)
    .await
    .map_err(err)?;
    Ok(())
}

/// Pull image bytes a sound references back into `gif_blobs` when they only
/// exist in the app library / cache (project saved without embedded images, or
/// an undo restored a button background whose blob was already collected).
async fn import_gifs(
    conn: &mut SqliteConnection,
    dirs: &GifDirs,
    progress: &Progress,
) -> Result<(), String> {
    if !table_exists(conn, "gif_blobs").await || !table_exists(conn, "sounds").await {
        return Ok(());
    }
    let missing: Vec<String> = sqlx::query(
        "SELECT DISTINCT gif_id FROM sounds
         WHERE gif_id IS NOT NULL AND gif_id != ''
           AND gif_id NOT IN (SELECT id FROM gif_blobs)",
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(err)?
    .iter()
    .filter_map(|r| r.try_get::<String, _>(0).ok())
    .filter(|id| gifcache::is_content_hash(id))
    .collect();
    progress.plan(Phase::Images, missing.len() as u64 * IMAGE_UNITS);

    for id in missing {
        // Lookup, read and base64 all touch the disk or burn CPU: do them on
        // the blocking pool and come back with the row to insert.
        let dirs = dirs.clone();
        let lookup_id = id.clone();
        let image = tauri::async_runtime::spawn_blocking(move || read_library_image(&dirs, &lookup_id))
            .await
            .map_err(err)?;
        progress.advance(IMAGE_UNITS);
        let Some(image) = image else { continue };
        sqlx::query(
            "INSERT OR IGNORE INTO gif_blobs (id, mime, data, poster, byte_len)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(image.mime)
        .bind(image.data)
        .bind(image.poster)
        .bind(image.byte_len)
        .execute(&mut *conn)
        .await
        .map_err(err)?;
    }
    Ok(())
}

/// An image from the app library / cache, encoded the way `gif_blobs` stores it.
struct LibraryImage {
    mime: String,
    /// Base64 of the image file.
    data: String,
    /// Base64 of the separate poster frame, if the image has one.
    poster: Option<String>,
    byte_len: i64,
}

fn read_library_image(dirs: &GifDirs, id: &str) -> Option<LibraryImage> {
    let entry = [dirs.library.as_deref(), dirs.cache.as_deref()]
        .into_iter()
        .flatten()
        .find_map(|dir| gifcache::hit_on_disk(dir, id))?;
    let bytes = std::fs::read(&entry.path).ok()?;
    let poster = if entry.poster_path != entry.path {
        std::fs::read(&entry.poster_path).ok().map(|b| BASE64.encode(b))
    } else {
        None
    };
    Some(LibraryImage {
        mime: entry.mime,
        data: BASE64.encode(&bytes),
        poster,
        byte_len: bytes.len() as i64,
    })
}

/// Move every stored image out of the project into the app image library.
/// A blob is only deleted once its file is safely on disk.
async fn externalize_gifs(
    conn: &mut SqliteConnection,
    dirs: &GifDirs,
    progress: &Progress,
) -> Result<bool, String> {
    let Some(library) = dirs.library.as_deref() else {
        return Ok(false);
    };
    if !table_exists(conn, "gif_blobs").await {
        return Ok(false);
    }
    let ids: Vec<String> = sqlx::query("SELECT id FROM gif_blobs")
        .fetch_all(&mut *conn)
        .await
        .map_err(err)?
        .iter()
        .filter_map(|r| r.try_get::<String, _>(0).ok())
        .filter(|id| gifcache::is_content_hash(id))
        .collect();
    progress.plan(Phase::Images, ids.len() as u64 * IMAGE_UNITS);
    let mut removed = false;
    for id in ids {
        progress.advance(IMAGE_UNITS);
        let row = sqlx::query("SELECT mime, data, poster FROM gif_blobs WHERE id = ?")
            .bind(&id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(err)?;
        let Some(row) = row else { continue };
        let mime: String = row.try_get(0).map_err(err)?;
        let data: String = row.try_get(1).map_err(err)?;
        let poster: Option<String> = row.try_get(2).map_err(err)?;
        // Base64 decode plus two file writes per image: blocking pool.
        let (dir, file_id) = (library.to_path_buf(), id.clone());
        let stored = tauri::async_runtime::spawn_blocking(move || {
            gifcache::store_files(&dir, &file_id, &mime, &data, poster.as_deref()).is_ok()
        })
        .await
        .map_err(err)?;
        if !stored {
            continue;
        }
        sqlx::query("DELETE FROM gif_blobs WHERE id = ?")
            .bind(&id)
            .execute(&mut *conn)
            .await
            .map_err(err)?;
        removed = true;
    }
    Ok(removed)
}

// ── Copy ──────────────────────────────────────────────────────────────────────

/// Copy a project to `dst` as a clean, compact, single-file database.
///
/// `VACUUM INTO` reads through SQLite, so changes still sitting in a `-wal`
/// file of a project written by an older build are included — a plain file
/// copy would silently drop them. Falls back to a byte copy when the source
/// cannot be read as a database (kept so a damaged file can be set aside).
#[tauri::command]
pub async fn project_copy(src: String, dst: String) -> Result<(), String> {
    let _guard = PROJECT_LOCK.lock().await;
    copy_project(&src, &dst).await
}

pub(crate) async fn copy_project(src: &str, dst: &str) -> Result<(), String> {
    if !Path::new(src).is_file() {
        return Err(format!("Project not found: {src}"));
    }
    if let Some(parent) = Path::new(dst).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(err)?;
        }
    }
    remove_with_sidecars(dst);
    let target = dst.replace('\'', "''");
    let sql = format!("VACUUM INTO '{target}'");

    // Read-only first: the source stays byte-for-byte untouched.
    let mut done = false;
    if let Ok(mut conn) = read_options(src).connect().await {
        done = sqlx::query(&sql).execute(&mut conn).await.is_ok();
        let _ = conn.close().await;
    }
    if !done {
        remove_with_sidecars(dst);
        if let Ok(mut conn) = write_options(src).create_if_missing(false).connect().await {
            done = sqlx::query(&sql).execute(&mut conn).await.is_ok();
            let _ = conn.close().await;
        }
    }
    if !done {
        remove_with_sidecars(dst);
        std::fs::copy(src, dst).map_err(err)?;
    }
    Ok(())
}

// ── Embedded audio → files ────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedSound {
    pub sound_id: String,
    pub path: String,
}

/// Write embedded audio back to disk for every sound whose file is missing on
/// this machine. Returns the new path per sound id.
#[tauri::command]
pub async fn project_extract_media(
    app: AppHandle,
    path: String,
    project_id: String,
) -> Result<Vec<ExtractedSound>, String> {
    let key: String = project_id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(64)
        .collect();
    let key = if key.is_empty() { "project".to_string() } else { key };
    let dir = app
        .path()
        .app_data_dir()
        .map_err(err)?
        .join("project-media")
        .join(key);
    let _guard = PROJECT_LOCK.lock().await;
    extract_media(&path, &dir).await
}

pub(crate) async fn extract_media(path: &str, dir: &Path) -> Result<Vec<ExtractedSound>, String> {
    extract_media_counted(path, dir).await.map(|(sounds, _)| sounds)
}

/// [`extract_media`] plus the number of `media` rows whose source stamp had
/// to be rewritten. A project that was already extracted reports `0`: opening
/// it again writes nothing to the file.
async fn extract_media_counted(
    path: &str,
    dir: &Path,
) -> Result<(Vec<ExtractedSound>, usize), String> {
    let mut conn = connect_write(path).await?;
    let result = extract_with(&mut conn, dir).await;
    let _ = conn.close().await;
    result
}

/// One embedded sound whose original file is missing on this machine.
struct ExtractJob {
    id: String,
    /// Where the embedded copy lives (or will be written).
    target: PathBuf,
    /// Stamp of `target` when it is already on disk with the right length.
    on_disk: Option<(i64, i64)>,
    /// `src_size` / `src_mtime` recorded in the `media` row.
    stored: (Option<i64>, Option<i64>),
}

/// Write an embedded file next to its final name and rename it into place, so
/// an interrupted write never leaves a truncated sound behind.
fn write_extracted(dir: &Path, target: &Path, data: &[u8]) -> Result<Option<(i64, i64)>, String> {
    std::fs::create_dir_all(dir).map_err(err)?;
    let tmp = target.with_extension("partial");
    std::fs::write(&tmp, data).map_err(err)?;
    std::fs::rename(&tmp, target).map_err(err)?;
    Ok(file_stat(target))
}

async fn extract_with(
    conn: &mut SqliteConnection,
    dir: &Path,
) -> Result<(Vec<ExtractedSound>, usize), String> {
    if !table_exists(conn, "media").await || !table_exists(conn, "sounds").await {
        return Ok((Vec::new(), 0));
    }
    let rows = sqlx::query(
        "SELECT s.path, m.sound_id, m.file_name, m.byte_len, m.src_size, m.src_mtime
         FROM media m JOIN sounds s ON s.sound_id = m.sound_id",
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(err)?;

    let mut candidates = Vec::with_capacity(rows.len());
    for row in &rows {
        let sound_path: String = row.try_get(0).map_err(err)?;
        let id: String = row.try_get(1).map_err(err)?;
        let name: String = row.try_get(2).map_err(err)?;
        let len: i64 = row.try_get(3).map_err(err)?;
        let stored: (Option<i64>, Option<i64>) =
            (row.try_get(4).map_err(err)?, row.try_get(5).map_err(err)?);
        let safe_id: String = id.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
        let target = dir.join(format!("{safe_id}-{len}-{}", safe_file_name(&name)));
        candidates.push((sound_path, len, ExtractJob { id, target, on_disk: None, stored }));
    }
    drop(rows);

    // Every stat in one hop to the blocking pool: which sounds are missing
    // here, and which of those already have their extracted copy.
    let jobs: Vec<ExtractJob> = tauri::async_runtime::spawn_blocking(move || {
        candidates
            .into_iter()
            .filter(|(sound_path, _, _)| !Path::new(sound_path).is_file())
            .map(|(_, len, mut job)| {
                job.on_disk = file_stat(&job.target).filter(|(size, _)| *size == len);
                job
            })
            .collect()
    })
    .await
    .map_err(err)?;

    let mut out = Vec::with_capacity(jobs.len());
    let mut stamps: Vec<(String, i64, i64)> = Vec::new();
    for job in jobs {
        let stat = match job.on_disk {
            Some(stat) => Some(stat),
            None => {
                let data: Vec<u8> = sqlx::query("SELECT data FROM media WHERE sound_id = ?")
                    .bind(&job.id)
                    .fetch_one(&mut *conn)
                    .await
                    .map_err(err)?
                    .try_get(0)
                    .map_err(err)?;
                let (dir, target) = (dir.to_path_buf(), job.target.clone());
                tauri::async_runtime::spawn_blocking(move || write_extracted(&dir, &target, &data))
                    .await
                    .map_err(err)??
            }
        };
        // Record the extracted file's stat so the next save sees the stored
        // copy as current and does not read every file back in. Skipped when
        // the row already says so: that is every sound on a repeated open.
        if let Some((size, mtime)) = stat {
            if job.stored != (Some(size), Some(mtime)) {
                stamps.push((job.id.clone(), size, mtime));
            }
        }
        out.push(ExtractedSound {
            sound_id: job.id,
            path: job.target.to_string_lossy().into_owned(),
        });
    }

    // One transaction for all stamps: one fsync instead of one per sound.
    // Best effort, as before — a failed stamp only costs a re-read on save.
    let updates = stamps.len();
    if !stamps.is_empty() {
        if let Ok(mut tx) = conn.begin().await {
            for (id, size, mtime) in &stamps {
                let _ = sqlx::query(
                    "UPDATE media SET src_size = ?, src_mtime = ? WHERE sound_id = ?",
                )
                .bind(size)
                .bind(mtime)
                .bind(id)
                .execute(&mut *tx)
                .await;
            }
            let _ = tx.commit().await;
        }
    }
    Ok((out, updates))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stmt(sql: &str, params: Vec<Value>) -> Statement {
        Statement { sql: sql.into(), params, tolerant: false }
    }

    fn tolerant(sql: &str) -> Statement {
        Statement { sql: sql.into(), params: vec![], tolerant: true }
    }

    fn schema() -> Vec<Statement> {
        vec![
            stmt("CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT)", vec![]),
            stmt(
                "CREATE TABLE IF NOT EXISTS sounds (path TEXT PRIMARY KEY, name TEXT, sound_id TEXT, gif_id TEXT)",
                vec![],
            ),
            stmt("CREATE TABLE IF NOT EXISTS tabs (name TEXT PRIMARY KEY)", vec![]),
        ]
    }

    async fn count(path: &str, table: &str) -> i64 {
        let mut conn = connect_write(path).await.unwrap();
        let n = sqlx::query(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&mut conn)
            .await
            .unwrap()
            .try_get(0)
            .unwrap();
        let _ = conn.close().await;
        n
    }

    #[tokio::test]
    async fn commit_rolls_back_everything_when_one_statement_fails() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p.sninja").to_string_lossy().into_owned();
        let none = GifDirs::default();
        commit(&path, &schema(), &CommitOptions::default(), &none).await.unwrap();
        commit(
            &path,
            &[stmt("INSERT INTO sounds (path, name) VALUES ($1, $2)", vec!["/a.wav".into(), "A".into()])],
            &CommitOptions::default(),
            &none,
        )
        .await
        .unwrap();

        let failed = commit(
            &path,
            &[
                stmt("DELETE FROM sounds", vec![]),
                stmt("INSERT INTO no_such_table (x) VALUES (1)", vec![]),
            ],
            &CommitOptions::default(),
            &none,
        )
        .await;
        assert!(failed.is_err());
        assert_eq!(count(&path, "sounds").await, 1);
    }

    #[tokio::test]
    async fn tolerant_statement_does_not_abort_the_transaction() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p.sninja").to_string_lossy().into_owned();
        let none = GifDirs::default();
        commit(&path, &schema(), &CommitOptions::default(), &none).await.unwrap();
        commit(
            &path,
            &[
                tolerant("ALTER TABLE sounds ADD COLUMN name TEXT"),
                stmt("INSERT INTO tabs (name) VALUES ($1)", vec!["Lolli".into()]),
            ],
            &CommitOptions::default(),
            &none,
        )
        .await
        .unwrap();
        assert_eq!(count(&path, "tabs").await, 1);
    }

    #[tokio::test]
    async fn inspect_reports_new_legacy_and_versioned_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p.sninja").to_string_lossy().into_owned();
        let fresh = inspect(&path).await;
        assert!(fresh.ok && fresh.is_new && !fresh.exists);

        let none = GifDirs::default();
        commit(&path, &schema(), &CommitOptions::default(), &none).await.unwrap();
        let legacy = inspect(&path).await;
        assert!(legacy.ok && !legacy.is_new);
        assert_eq!(legacy.format_version, None);

        commit(
            &path,
            &[stmt("INSERT INTO settings (key, value) VALUES ('formatVersion', $1)", vec!["2".into()])],
            &CommitOptions::default(),
            &none,
        )
        .await
        .unwrap();
        assert_eq!(inspect(&path).await.format_version, Some(2));
    }

    #[tokio::test]
    async fn inspect_flags_a_file_that_is_not_a_database() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("broken.sninja");
        std::fs::write(&path, vec![7u8; 4096]).unwrap();
        let report = inspect(&path.to_string_lossy()).await;
        assert!(!report.ok);
        assert!(report.error.is_some());
    }

    #[tokio::test]
    async fn embedded_audio_round_trips_to_another_location() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p.sninja").to_string_lossy().into_owned();
        let audio = dir.path().join("kick.wav");
        std::fs::write(&audio, b"RIFFdata").unwrap();
        let none = GifDirs::default();
        commit(&path, &schema(), &CommitOptions::default(), &none).await.unwrap();

        let embed = CommitOptions { embed_audio: Some(true), ..Default::default() };
        let report = commit(
            &path,
            &[stmt(
                "INSERT INTO sounds (path, name, sound_id) VALUES ($1, $2, $3)",
                vec![audio.to_string_lossy().into_owned().into(), "Kick".into(), "abcd1234".into()],
            )],
            &embed,
            &none,
        )
        .await
        .unwrap();
        assert_eq!(report.embedded_audio, 1);

        // Unchanged file: the second save only stats it.
        let again = commit(&path, &[], &embed, &none).await.unwrap();
        assert_eq!(again.embedded_audio, 0);

        // "Other machine": the original file is gone.
        std::fs::remove_file(&audio).unwrap();
        let out = dir.path().join("extracted");
        let extracted = extract_media(&path, &out).await.unwrap();
        assert_eq!(extracted.len(), 1);
        assert_eq!(extracted[0].sound_id, "abcd1234");
        assert_eq!(std::fs::read(&extracted[0].path).unwrap(), b"RIFFdata");

        // Missing file keeps its stored copy and is not reported missing.
        let keep = commit(&path, &[], &embed, &none).await.unwrap();
        assert!(keep.missing_audio.is_empty());
        assert_eq!(count(&path, "media").await, 1);

        let drop = CommitOptions { embed_audio: Some(false), ..Default::default() };
        commit(&path, &[], &drop, &none).await.unwrap();
        assert_eq!(count(&path, "media").await, 0);
    }

    #[tokio::test]
    async fn gifs_move_to_the_library_and_back() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p.sninja").to_string_lossy().into_owned();
        let dirs = GifDirs { library: Some(dir.path().join("lib")), cache: None };
        let mut setup = schema();
        setup.push(stmt(
            "CREATE TABLE gif_blobs (id TEXT PRIMARY KEY, mime TEXT NOT NULL, data TEXT NOT NULL, poster TEXT, byte_len INTEGER)",
            vec![],
        ));
        setup.push(stmt(
            "INSERT INTO gif_blobs (id, mime, data, poster, byte_len) VALUES ('abc123', 'image/gif', $1, NULL, 6)",
            vec![BASE64.encode(b"GIF89a").into()],
        ));
        setup.push(stmt(
            "INSERT INTO sounds (path, name, sound_id, gif_id) VALUES ('/a.wav', 'A', 'aaaaaaaa', 'abc123')",
            vec![],
        ));
        commit(&path, &setup, &CommitOptions::default(), &dirs).await.unwrap();

        let out = CommitOptions { embed_gifs: Some(false), ..Default::default() };
        commit(&path, &[], &out, &dirs).await.unwrap();
        assert_eq!(count(&path, "gif_blobs").await, 0);
        assert!(dir.path().join("lib").join("abc123.gif").is_file());

        let back = CommitOptions { embed_gifs: Some(true), ..Default::default() };
        commit(&path, &[], &back, &dirs).await.unwrap();
        assert_eq!(count(&path, "gif_blobs").await, 1);
    }

    #[tokio::test]
    async fn copy_project_produces_an_independent_file() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("a.sninja").to_string_lossy().into_owned();
        let dst = dir.path().join("sub").join("b's copy.sninja").to_string_lossy().into_owned();
        let none = GifDirs::default();
        let mut setup = schema();
        setup.push(stmt("INSERT INTO tabs (name) VALUES ('Lolli')", vec![]));
        commit(&src, &setup, &CommitOptions::default(), &none).await.unwrap();
        copy_project(&src, &dst).await.unwrap();
        assert_eq!(count(&dst, "tabs").await, 1);
        // Overwriting an existing destination works too.
        copy_project(&src, &dst).await.unwrap();
        assert_eq!(count(&dst, "tabs").await, 1);
    }

    #[tokio::test]
    async fn query_returns_rows_as_typed_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p.sninja").to_string_lossy().into_owned();
        let none = GifDirs::default();
        commit(
            &path,
            &[
                stmt("CREATE TABLE t (i INTEGER, r REAL, s TEXT, n TEXT, b INTEGER)", vec![]),
                stmt(
                    "INSERT INTO t (i, r, s, n, b) VALUES ($1, $2, $3, $4, $5)",
                    vec![7.into(), 2.5.into(), "x".into(), Value::Null, true.into()],
                ),
            ],
            &CommitOptions::default(),
            &none,
        )
        .await
        .unwrap();
        let rows = query(&path, "SELECT i, r, s, n, b, COUNT(*) AS c FROM t WHERE s = $1", &["x".into()])
            .await
            .unwrap();
        assert_eq!(
            Value::Object(rows[0].clone()),
            serde_json::json!({ "i": 7, "r": 2.5, "s": "x", "n": null, "b": 1, "c": 1 })
        );
        let info = query(&path, "PRAGMA table_info(t)", &[]).await.unwrap();
        assert_eq!(info.len(), 5);
        assert_eq!(info[0]["name"], "i");
    }

    #[tokio::test]
    async fn opening_a_wal_project_folds_it_back_into_one_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p.sninja").to_string_lossy().into_owned();
        // A project as an older build left it: WAL journal, changes in the sidecar.
        let mut old = SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .connect()
            .await
            .unwrap();
        sqlx::query("CREATE TABLE tabs (name TEXT PRIMARY KEY)").execute(&mut old).await.unwrap();
        sqlx::query("INSERT INTO tabs (name) VALUES ('Lolli')").execute(&mut old).await.unwrap();
        let _ = old.close().await;

        let rows = query(&path, "PRAGMA journal_mode", &[]).await.unwrap();
        assert_eq!(rows[0]["journal_mode"], "delete");
        assert_eq!(count(&path, "tabs").await, 1);
        assert!(!Path::new(&format!("{path}-wal")).exists());
    }

    /// Collects every progress event of a save.
    fn recording_progress(path: &str) -> (Progress, Arc<std::sync::Mutex<Vec<SaveProgress>>>) {
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let sink = Arc::clone(&events);
        let progress = Progress::new(path, move |update| sink.lock().unwrap().push(update));
        (progress, events)
    }

    #[tokio::test]
    async fn commit_with_reports_monotonic_percent_ending_at_100() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p.sninja").to_string_lossy().into_owned();
        let none = GifDirs::default();
        commit(&path, &schema(), &CommitOptions::default(), &none).await.unwrap();

        let mut statements = Vec::new();
        for i in 0..40 {
            let audio = dir.path().join(format!("s{i}.wav"));
            std::fs::write(&audio, vec![i as u8; 20_000]).unwrap();
            statements.push(stmt(
                "INSERT INTO sounds (path, name, sound_id) VALUES ($1, $2, $3)",
                vec![
                    audio.to_string_lossy().into_owned().into(),
                    format!("S{i}").into(),
                    format!("id{i:04}").into(),
                ],
            ));
        }
        let options = CommitOptions {
            embed_audio: Some(true),
            embed_gifs: Some(true),
            backup: true,
            progress: true,
        };
        let (progress, events) = recording_progress(&path);
        let report = commit_with(&path, &statements, &options, &none, &progress).await.unwrap();
        assert_eq!(report.embedded_audio, 40);
        assert!(report.backup_written);

        let events = events.lock().unwrap();
        let phases: Vec<&str> = events.iter().map(|e| e.phase).collect();
        // Each phase announces itself, in order, however fast it is.
        let mut order = phases.clone();
        order.dedup();
        assert_eq!(order, ["backup", "board", "audio", "images", "finalizing"]);
        for pair in events.windows(2) {
            assert!(pair[1].percent >= pair[0].percent, "{events:?}");
        }
        for event in events.iter() {
            assert_eq!(event.path, path);
            assert!(event.done <= event.total, "{event:?}");
            assert!(event.percent <= 100);
        }
        let last = events.last().unwrap();
        assert_eq!((last.phase, last.percent), ("finalizing", 100));
        assert_eq!(last.done, last.total);
        // 100 is reported exactly once, by the final event.
        assert_eq!(events.iter().filter(|e| e.percent == 100).count(), 1);
        // The new sounds were counted before the first event went out.
        assert!(events[0].total >= 40 * 20_000, "{:?}", events[0]);
    }

    #[tokio::test]
    async fn commit_with_does_not_report_100_when_the_save_fails() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p.sninja").to_string_lossy().into_owned();
        let (progress, events) = recording_progress(&path);
        let options = CommitOptions { progress: true, ..Default::default() };
        let failed = commit_with(
            &path,
            &[stmt("INSERT INTO no_such_table (x) VALUES (1)", vec![])],
            &options,
            &GifDirs::default(),
            &progress,
        )
        .await;
        assert!(failed.is_err());
        assert!(events.lock().unwrap().iter().all(|e| e.percent < 100));
    }

    #[tokio::test]
    async fn commit_without_progress_emits_no_events() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p.sninja").to_string_lossy().into_owned();
        let none = Progress::none();
        assert!(!none.enabled());
        commit_with(&path, &schema(), &CommitOptions::default(), &GifDirs::default(), &none)
            .await
            .unwrap();
        // The options default keeps reporting off for existing callers.
        let parsed: CommitOptions = serde_json::from_str(r#"{"backup":true}"#).unwrap();
        assert!(!parsed.progress);
    }

    #[test]
    fn save_progress_serialises_with_the_contract_field_names() {
        let (progress, events) = recording_progress("/boards/a.sninja");
        progress.plan(Phase::Audio, 200);
        progress.begin(Phase::Audio);
        let json = serde_json::to_value(&events.lock().unwrap()[0]).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "path": "/boards/a.sninja", "phase": "audio", "done": 0, "total": 200, "percent": 0
            })
        );
    }

    #[test]
    fn progress_never_reports_a_lower_percent_when_the_total_grows() {
        let (progress, events) = recording_progress("p");
        progress.plan(Phase::Board, 100);
        progress.begin(Phase::Board);
        progress.state.as_ref().unwrap().done.store(90, Ordering::Relaxed);
        progress.begin(Phase::Audio);
        // The audio phase finds far more work than the estimate assumed.
        progress.plan(Phase::Audio, 1_000);
        progress.begin(Phase::Audio);
        progress.finish();
        let percents: Vec<u32> = events.lock().unwrap().iter().map(|e| e.percent).collect();
        assert_eq!(percents, [0, 89, 89, 100]);
    }

    #[test]
    fn backup_file_copies_large_files_in_chunks_byte_for_byte() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("p.sninja");
        let dst = dir.path().join("p.sninja.bak");
        // Larger than two chunks and not a multiple of the chunk size.
        let data: Vec<u8> = (0..2 * BACKUP_CHUNK_BYTES + 12_345).map(|i| (i % 251) as u8).collect();
        std::fs::write(&src, &data).unwrap();
        std::fs::write(&dst, b"previous backup").unwrap();

        let mut reported = Vec::new();
        // Threshold lowered so this file takes the chunked path.
        backup_file(&src, &dst, 1024, |bytes| reported.push(bytes)).unwrap();
        assert_eq!(std::fs::read(&dst).unwrap(), data);
        assert_eq!(reported.len(), 3);
        assert_eq!(reported.iter().sum::<u64>(), data.len() as u64);
        assert!(!dir.path().join("p.sninja.bak.partial").exists());
    }

    #[test]
    fn backup_file_copies_small_files_in_one_step() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("p.sninja");
        let dst = dir.path().join("p.sninja.bak");
        std::fs::write(&src, b"small project").unwrap();
        let mut reported = Vec::new();
        backup_file(&src, &dst, BACKUP_CHUNKED_MIN_BYTES, |bytes| reported.push(bytes)).unwrap();
        assert_eq!(std::fs::read(&dst).unwrap(), b"small project");
        assert_eq!(reported, [13]);
    }

    #[test]
    fn backup_file_keeps_the_previous_backup_when_the_copy_fails() {
        let dir = tempfile::tempdir().unwrap();
        let dst = dir.path().join("p.sninja.bak");
        std::fs::write(&dst, b"previous backup").unwrap();
        let missing = dir.path().join("gone.sninja");
        assert!(backup_file(&missing, &dst, 1024, |_| {}).is_err());
        assert_eq!(std::fs::read(&dst).unwrap(), b"previous backup");
        assert!(!dir.path().join("p.sninja.bak.partial").exists());
    }

    #[tokio::test]
    async fn extract_media_writes_no_stamp_on_a_second_run() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p.sninja").to_string_lossy().into_owned();
        let none = GifDirs::default();
        commit(&path, &schema(), &CommitOptions::default(), &none).await.unwrap();
        let mut statements = Vec::new();
        for name in ["kick", "snare"] {
            let audio = dir.path().join(format!("{name}.wav"));
            std::fs::write(&audio, format!("RIFF{name}")).unwrap();
            statements.push(stmt(
                "INSERT INTO sounds (path, name, sound_id) VALUES ($1, $2, $3)",
                vec![audio.to_string_lossy().into_owned().into(), name.into(), format!("id{name}").into()],
            ));
        }
        let embed = CommitOptions { embed_audio: Some(true), ..Default::default() };
        commit(&path, &statements, &embed, &none).await.unwrap();
        // "Other machine": the original files are gone.
        for name in ["kick", "snare"] {
            std::fs::remove_file(dir.path().join(format!("{name}.wav"))).unwrap();
        }

        let out = dir.path().join("extracted");
        let (first, first_updates) = extract_media_counted(&path, &out).await.unwrap();
        assert_eq!(first.len(), 2);
        assert_eq!(first_updates, 2);

        let (second, second_updates) = extract_media_counted(&path, &out).await.unwrap();
        assert_eq!(second.len(), 2);
        assert_eq!(second_updates, 0);
        assert!(second.iter().all(|s| Path::new(&s.path).is_file()));

        // A deleted extracted file is written again and stamped again.
        std::fs::remove_file(&second[0].path).unwrap();
        let (third, third_updates) = extract_media_counted(&path, &out).await.unwrap();
        assert_eq!(third.len(), 2);
        assert!(third_updates <= 1);
        assert!(Path::new(&third[0].path).is_file());
    }

    #[test]
    fn base_name_handles_both_separators() {
        assert_eq!(base_name("C:\\board\\Lolli\\kick.wav"), "kick.wav");
        assert_eq!(base_name("/home/deck/board/kick.wav"), "kick.wav");
        assert_eq!(safe_file_name("a:b*c?.wav"), "a_b_c_.wav");
    }
}
