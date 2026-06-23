use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};

/// Maximum total RAM used by the cache (256 MiB).
const MAX_CACHE_SIZE: usize = 256 * 1024 * 1024;
/// Maximum size of a single cached entry (50 MiB).
/// Files larger than this are read from disk every time and never cached.
const MAX_FILE_ENTRY: usize = 50 * 1024 * 1024;

// ---------------------------------------------------------------------------
// LRU cache
// ---------------------------------------------------------------------------

pub struct SoundCache {
    entries: HashMap<String, Arc<[u8]>>,
    /// Access order — front = most recently used, back = least recently used.
    order: VecDeque<String>,
    total_size: usize,
    pub max_size: usize,
    max_entry: usize,
}

impl SoundCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            total_size: 0,
            max_size: MAX_CACHE_SIZE,
            max_entry: MAX_FILE_ENTRY,
        }
    }

    /// Return cached bytes and promote entry to MRU position.
    pub fn get(&mut self, path: &str) -> Option<Arc<[u8]>> {
        if self.entries.contains_key(path) {
            self.order.retain(|p| p != path);
            self.order.push_front(path.to_string());
            self.entries.get(path).cloned()
        } else {
            None
        }
    }

    /// Store bytes in the cache (LRU eviction when full).
    /// Files exceeding `max_entry` are returned as an `Arc` but never stored.
    pub fn insert(&mut self, path: String, data: Vec<u8>) -> Arc<[u8]> {
        // Already cached — just promote and return.
        if let Some(existing) = self.entries.get(&path) {
            self.order.retain(|p| p != &path);
            self.order.push_front(path);
            return existing.clone();
        }

        let arc: Arc<[u8]> = data.into();
        let size = arc.len();

        if size > self.max_entry {
            return arc; // Too large to cache; serve from the temporary Arc.
        }

        // Evict LRU entries until there is room.
        while self.total_size + size > self.max_size {
            match self.order.pop_back() {
                Some(lru) => {
                    if let Some(evicted) = self.entries.remove(&lru) {
                        self.total_size = self.total_size.saturating_sub(evicted.len());
                    }
                }
                None => break,
            }
        }

        self.order.push_front(path.clone());
        self.total_size += size;
        self.entries.insert(path, arc.clone());
        arc
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.order.clear();
        self.total_size = 0;
    }

    pub fn set_limits(&mut self, max_size: usize, max_entry: usize) {
        self.max_size = max_size;
        self.max_entry = max_entry;
    }

    pub fn stats(&self) -> CacheStats {
        CacheStats {
            cached_count: self.entries.len(),
            total_size_bytes: self.total_size,
            max_size_bytes: self.max_size,
        }
    }
}

// ---------------------------------------------------------------------------
// Global instance
// ---------------------------------------------------------------------------

static SOUND_CACHE: OnceLock<Mutex<SoundCache>> = OnceLock::new();

pub fn sound_cache() -> &'static Mutex<SoundCache> {
    SOUND_CACHE.get_or_init(|| Mutex::new(SoundCache::new()))
}

// ---------------------------------------------------------------------------
// Core helper used by playback and duration queries
// ---------------------------------------------------------------------------

/// Return the raw bytes of a sound file.
///
/// On the first access the file is read from disk and stored in the LRU cache.
/// Subsequent accesses return a cheap `Arc` clone — zero disk I/O.
pub fn load_bytes(path: &str) -> Result<Arc<[u8]>, String> {
    // Fast path: cache hit.
    {
        let mut cache = sound_cache().lock().map_err(|e| e.to_string())?;
        if let Some(data) = cache.get(path) {
            return Ok(data);
        }
    }

    // Cache miss: read from disk.
    let data =
        std::fs::read(path).map_err(|e| format!("Failed to read '{}': {}", path, e))?;

    // Re-lock and insert (a concurrent warm might have beaten us; insert handles that).
    let mut cache = sound_cache().lock().map_err(|e| e.to_string())?;
    Ok(cache.insert(path.to_string(), data))
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[derive(serde::Serialize)]
pub struct CacheStats {
    pub cached_count: usize,
    pub total_size_bytes: usize,
    pub max_size_bytes: usize,
}

/// Pre-load a list of sound files into RAM in the background.
/// Call this from the frontend on startup (e.g. with all sound paths in the
/// current project) to eliminate HDD spin-up latency on first playback.
#[tauri::command]
pub fn warm_sound_cache(paths: Vec<String>) {
    std::thread::spawn(move || {
        for path in paths {
            if let Err(e) = load_bytes(&path) {
                eprintln!("[cache] warm failed '{}': {}", path, e);
            }
        }
    });
}

/// Evict all cached sounds, freeing RAM immediately.
#[tauri::command]
pub fn clear_sound_cache() -> Result<(), String> {
    sound_cache().lock().map_err(|e| e.to_string())?.clear();
    Ok(())
}

/// Return current cache usage statistics.
#[tauri::command]
pub fn get_cache_stats() -> Result<CacheStats, String> {
    let cache = sound_cache().lock().map_err(|e| e.to_string())?;
    Ok(cache.stats())
}

/// Update the cache size limits at runtime.
/// `max_size_mib`  – total RAM limit in MiB (min 32, max 4096).
/// `max_entry_mib` – per-file limit in MiB; files above this are never cached.
#[tauri::command]
pub fn set_cache_config(max_size_mib: u64, max_entry_mib: u64) -> Result<(), String> {
    let max_size = (max_size_mib as usize).saturating_mul(1024 * 1024);
    let max_entry = (max_entry_mib as usize).saturating_mul(1024 * 1024);
    sound_cache()
        .lock()
        .map_err(|e| e.to_string())?
        .set_limits(max_size, max_entry);
    Ok(())
}
