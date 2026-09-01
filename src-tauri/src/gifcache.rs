//! On-disk cache for GIF/image button backgrounds.
//!
//! The project database stays the single source of truth so projects remain
//! self-contained and export/import is unchanged. What changed is delivery: a
//! blob used to travel to the webview as base64 through `plugin:sql|select`
//! (over 10 MB for one animation) and was then decoded in JavaScript. Now Rust
//! decodes it once into a content-addressed file and the webview loads that
//! file natively through `convertFileSrc`.

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::Serialize;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::Row;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

use crate::task::run_blocking;

/// File paths the webview can hand straight to `convertFileSrc`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GifCacheEntry {
    pub id: String,
    pub mime: String,
    /// Animated original.
    pub path: String,
    /// First-frame still shown while the button is idle. Equals `path` when the
    /// project has no separate poster.
    pub poster_path: String,
}

fn cache_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?
        .join("gif-cache");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn extension_for(mime: &str) -> &'static str {
    match mime {
        "image/webp" => "webp",
        "image/png" => "png",
        "image/jpeg" => "jpg",
        _ => "gif",
    }
}

/// Reject anything that is not a plain content hash before it reaches the
/// filesystem — ids come from the renderer.
fn is_content_hash(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn write_if_missing(path: &Path, base64_data: &str) -> Result<(), String> {
    if path.exists() {
        return Ok(());
    }
    let bytes = BASE64
        .decode(base64_data.as_bytes())
        .map_err(|e| format!("Corrupt image data: {e}"))?;
    // Write to a sibling first so a crash mid-write cannot leave a half file
    // that later looks like a valid cache hit.
    let tmp = path.with_extension("partial");
    std::fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

/// Materialise the requested blobs as files and return their paths.
///
/// Ids already present on disk skip the database entirely, so repeated board
/// renders cost one `stat` per image instead of a multi-megabyte IPC round trip.
#[tauri::command]
pub async fn gif_cache_paths(
    app: AppHandle,
    project_db: String,
    ids: Vec<String>,
) -> Result<Vec<GifCacheEntry>, String> {
    run_blocking(move || resolve_paths(&app, &project_db, &ids)).await
}

fn resolve_paths(
    app: &AppHandle,
    project_db: &str,
    ids: &[String],
) -> Result<Vec<GifCacheEntry>, String> {
    let dir = cache_dir(app)?;
    let wanted: Vec<&String> = ids
        .iter()
        .filter(|id| is_content_hash(id))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    if wanted.is_empty() {
        return Ok(Vec::new());
    }

    let mut out = Vec::with_capacity(wanted.len());
    let mut missing: Vec<&str> = Vec::new();
    for id in &wanted {
        match hit_on_disk(&dir, id) {
            Some(entry) => out.push(entry),
            None => missing.push(id.as_str()),
        }
    }
    if missing.is_empty() {
        return Ok(out);
    }

    for row in read_blobs(project_db, &missing)? {
        let ext = extension_for(&row.mime);
        let anim = dir.join(format!("{}.{ext}", row.id));
        write_if_missing(&anim, &row.data)?;

        let poster = match row.poster.as_deref() {
            Some(data) if !data.is_empty() => {
                let path = dir.join(format!("{}.poster.png", row.id));
                write_if_missing(&path, data)?;
                path
            }
            _ => anim.clone(),
        };
        // Marker so later lookups know the mime without touching the database.
        let _ = std::fs::write(dir.join(format!("{}.mime", row.id)), &row.mime);

        out.push(GifCacheEntry {
            id: row.id,
            mime: row.mime,
            path: anim.to_string_lossy().into_owned(),
            poster_path: poster.to_string_lossy().into_owned(),
        });
    }
    Ok(out)
}

fn hit_on_disk(dir: &Path, id: &str) -> Option<GifCacheEntry> {
    let mime = std::fs::read_to_string(dir.join(format!("{id}.mime"))).ok()?;
    let anim = dir.join(format!("{id}.{}", extension_for(&mime)));
    if !anim.exists() {
        return None;
    }
    let poster = dir.join(format!("{id}.poster.png"));
    let poster_path = if poster.exists() { poster } else { anim.clone() };
    Some(GifCacheEntry {
        id: id.to_owned(),
        mime,
        path: anim.to_string_lossy().into_owned(),
        poster_path: poster_path.to_string_lossy().into_owned(),
    })
}

struct BlobRow {
    id: String,
    mime: String,
    data: String,
    poster: Option<String>,
}

/// Read the blobs directly from the project SQLite file. Read-only, and on its
/// own short-lived connection so it never contends with the plugin's pool.
fn read_blobs(project_db: &str, ids: &[&str]) -> Result<Vec<BlobRow>, String> {
    let options = SqliteConnectOptions::new()
        .filename(project_db)
        .read_only(true)
        .create_if_missing(false);

    tauri::async_runtime::block_on(async move {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(|e| format!("Cannot open '{project_db}': {e}"))?;

        let placeholders = vec!["?"; ids.len()].join(", ");
        let sql =
            format!("SELECT id, mime, data, poster FROM gif_blobs WHERE id IN ({placeholders})");
        let mut query = sqlx::query(&sql);
        for id in ids {
            query = query.bind(*id);
        }
        let rows = query
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("GIF lookup failed: {e}"))?;
        pool.close().await;

        Ok(rows
            .into_iter()
            .map(|r| BlobRow {
                id: r.get::<String, _>("id"),
                mime: r.get::<String, _>("mime"),
                data: r.get::<String, _>("data"),
                poster: r.get::<Option<String>, _>("poster"),
            })
            .collect())
    })
}

/// Seed the cache straight after a new image is stored, so the button shows it
/// without a database round trip.
#[tauri::command]
pub async fn gif_cache_put(
    app: AppHandle,
    id: String,
    mime: String,
    data: String,
    poster: Option<String>,
) -> Result<GifCacheEntry, String> {
    run_blocking(move || {
        if !is_content_hash(&id) {
            return Err("Invalid image id".into());
        }
        let dir = cache_dir(&app)?;
        let anim = dir.join(format!("{id}.{}", extension_for(&mime)));
        write_if_missing(&anim, &data)?;

        let poster_path = match poster.as_deref() {
            Some(p) if !p.is_empty() => {
                let path = dir.join(format!("{id}.poster.png"));
                write_if_missing(&path, p)?;
                path
            }
            _ => anim.clone(),
        };
        std::fs::write(dir.join(format!("{id}.mime")), &mime).map_err(|e| e.to_string())?;

        Ok(GifCacheEntry {
            id,
            mime,
            path: anim.to_string_lossy().into_owned(),
            poster_path: poster_path.to_string_lossy().into_owned(),
        })
    })
    .await
}

/// Drop every cached file. The project database keeps the originals, so this is
/// always safe — the next render simply re-extracts what it needs.
#[tauri::command(async)]
pub fn gif_cache_clear(app: AppHandle) -> Result<(), String> {
    let dir = cache_dir(&app)?;
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
        let _ = std::fs::remove_file(entry.path());
    }
    Ok(())
}
