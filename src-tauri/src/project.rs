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
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

use crate::gifcache;

/// Serialises project access inside this process. Reads from `gifcache` use
/// their own read-only connection and rely on `busy_timeout` instead.
static PROJECT_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Per-session `.bak` copies are skipped above this size (embedded audio can
/// make a project several GB; the in-file snapshots still cover the layout).
const MAX_BACKUP_BYTES: u64 = 512 * 1024 * 1024;
/// `PRAGMA quick_check` reads every page. Skip it for very large files.
const MAX_CHECK_BYTES: u64 = 256 * 1024 * 1024;

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
    let _guard = PROJECT_LOCK.lock().await;
    commit(&path, &statements, &options.unwrap_or_default(), &dirs).await
}

pub(crate) async fn commit(
    path: &str,
    statements: &[Statement],
    options: &CommitOptions,
    dirs: &GifDirs,
) -> Result<CommitReport, String> {
    let mut report = CommitReport::default();
    if options.backup {
        if let Ok(meta) = std::fs::metadata(path) {
            if meta.len() > 0 && meta.len() <= MAX_BACKUP_BYTES {
                report.backup_written = std::fs::copy(path, format!("{path}.bak")).is_ok();
            }
        }
    }

    let mut conn = connect_write(path).await?;
    let mut shrink = false;
    let result: Result<(), String> = async {
        let mut tx = conn.begin().await.map_err(err)?;
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
        }
        if options.embed_audio == Some(true) {
            embed_audio(&mut tx, &mut report).await?;
        } else if options.embed_audio == Some(false) && table_exists(&mut tx, "media").await {
            let gone = sqlx::query("DELETE FROM media")
                .execute(&mut *tx)
                .await
                .map_err(err)?;
            shrink = gone.rows_affected() > 0;
        }
        if options.embed_gifs == Some(true) {
            import_gifs(&mut tx, dirs).await?;
        } else if options.embed_gifs == Some(false) {
            shrink |= externalize_gifs(&mut tx, dirs).await?;
        }
        tx.commit().await.map_err(err)
    }
    .await;

    if result.is_ok() && shrink {
        // Reclaim the space the removed blobs held. Best effort.
        let _ = sqlx::query("VACUUM").execute(&mut conn).await;
    }
    let _ = conn.close().await;
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

/// Store each sound's audio file in `media`, keyed by the sound's stable id.
/// Files whose size and mtime match the stored copy are skipped, so a normal
/// save costs one `stat` per sound.
async fn embed_audio(conn: &mut SqliteConnection, report: &mut CommitReport) -> Result<(), String> {
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

    for row in sounds {
        let path: String = row.try_get(0).map_err(err)?;
        let id: String = row.try_get(1).map_err(err)?;
        let Some((size, mtime)) = file_stat(Path::new(&path)) else {
            if !stored.contains_key(&id) {
                report.missing_audio.push(path);
            }
            continue;
        };
        if stored.get(&id) == Some(&(Some(size), Some(mtime))) {
            continue;
        }
        let read_path = path.clone();
        let bytes = tauri::async_runtime::spawn_blocking(move || std::fs::read(read_path))
            .await
            .map_err(err)?;
        let bytes = match bytes {
            Ok(b) => b,
            Err(_) => {
                if !stored.contains_key(&id) {
                    report.missing_audio.push(path);
                }
                continue;
            }
        };
        report.embedded_audio += 1;
        report.embedded_bytes += bytes.len() as u64;
        sqlx::query(
            "INSERT OR REPLACE INTO media (sound_id, file_name, byte_len, src_size, src_mtime, data)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(base_name(&path))
        .bind(bytes.len() as i64)
        .bind(size)
        .bind(mtime)
        .bind(bytes)
        .execute(&mut *conn)
        .await
        .map_err(err)?;
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
async fn import_gifs(conn: &mut SqliteConnection, dirs: &GifDirs) -> Result<(), String> {
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
    .collect();

    for id in missing {
        if !gifcache::is_content_hash(&id) {
            continue;
        }
        let hit = [dirs.library.as_deref(), dirs.cache.as_deref()]
            .into_iter()
            .flatten()
            .find_map(|dir| gifcache::hit_on_disk(dir, &id));
        let Some(entry) = hit else { continue };
        let Ok(bytes) = std::fs::read(&entry.path) else { continue };
        let poster = if entry.poster_path != entry.path {
            std::fs::read(&entry.poster_path).ok().map(|b| BASE64.encode(b))
        } else {
            None
        };
        sqlx::query(
            "INSERT OR IGNORE INTO gif_blobs (id, mime, data, poster, byte_len)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&entry.mime)
        .bind(BASE64.encode(&bytes))
        .bind(poster)
        .bind(bytes.len() as i64)
        .execute(&mut *conn)
        .await
        .map_err(err)?;
    }
    Ok(())
}

/// Move every stored image out of the project into the app image library.
/// A blob is only deleted once its file is safely on disk.
async fn externalize_gifs(conn: &mut SqliteConnection, dirs: &GifDirs) -> Result<bool, String> {
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
        .collect();
    let mut removed = false;
    for id in ids {
        if !gifcache::is_content_hash(&id) {
            continue;
        }
        let row = sqlx::query("SELECT mime, data, poster FROM gif_blobs WHERE id = ?")
            .bind(&id)
            .fetch_optional(&mut *conn)
            .await
            .map_err(err)?;
        let Some(row) = row else { continue };
        let mime: String = row.try_get(0).map_err(err)?;
        let data: String = row.try_get(1).map_err(err)?;
        let poster: Option<String> = row.try_get(2).map_err(err)?;
        if gifcache::store_files(library, &id, &mime, &data, poster.as_deref()).is_err() {
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
    let mut conn = connect_write(path).await?;
    let result: Result<Vec<ExtractedSound>, String> = async {
        if !table_exists(&mut conn, "media").await || !table_exists(&mut conn, "sounds").await {
            return Ok(Vec::new());
        }
        let rows = sqlx::query(
            "SELECT s.path, m.sound_id, m.file_name, m.byte_len
             FROM media m JOIN sounds s ON s.sound_id = m.sound_id",
        )
        .fetch_all(&mut conn)
        .await
        .map_err(err)?;

        let mut out = Vec::new();
        for row in rows {
            let sound_path: String = row.try_get(0).map_err(err)?;
            if Path::new(&sound_path).is_file() {
                continue;
            }
            let id: String = row.try_get(1).map_err(err)?;
            let name: String = row.try_get(2).map_err(err)?;
            let len: i64 = row.try_get(3).map_err(err)?;
            let safe_id: String = id.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
            let target = dir.join(format!("{safe_id}-{len}-{}", safe_file_name(&name)));

            let fresh = std::fs::metadata(&target).map(|m| m.len() as i64 == len).unwrap_or(false);
            if !fresh {
                let data: Vec<u8> = sqlx::query("SELECT data FROM media WHERE sound_id = ?")
                    .bind(&id)
                    .fetch_one(&mut conn)
                    .await
                    .map_err(err)?
                    .try_get(0)
                    .map_err(err)?;
                std::fs::create_dir_all(dir).map_err(err)?;
                let tmp = target.with_extension("partial");
                std::fs::write(&tmp, &data).map_err(err)?;
                std::fs::rename(&tmp, &target).map_err(err)?;
            }
            // Record the extracted file's stat so the next save sees the stored
            // copy as current and does not read every file back in.
            if let Some((size, mtime)) = file_stat(&target) {
                let _ = sqlx::query("UPDATE media SET src_size = ?, src_mtime = ? WHERE sound_id = ?")
                    .bind(size)
                    .bind(mtime)
                    .bind(&id)
                    .execute(&mut conn)
                    .await;
            }
            out.push(ExtractedSound {
                sound_id: id,
                path: target.to_string_lossy().into_owned(),
            });
        }
        Ok(out)
    }
    .await;
    let _ = conn.close().await;
    result
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

    #[test]
    fn base_name_handles_both_separators() {
        assert_eq!(base_name("C:\\board\\Lolli\\kick.wav"), "kick.wav");
        assert_eq!(base_name("/home/deck/board/kick.wav"), "kick.wav");
        assert_eq!(safe_file_name("a:b*c?.wav"), "a_b_c_.wav");
    }
}
