use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Maximum total RAM used by the cache (64 MiB).
const MAX_CACHE_SIZE: usize = 64 * 1024 * 1024;
/// Maximum size of a single cached entry (16 MiB).
/// Files larger than this are streamed from disk and never cached.
const MAX_FILE_ENTRY: usize = 16 * 1024 * 1024;
/// Drop unused entries after this idle time (unless currently playing).
const ENTRY_TTL: Duration = Duration::from_secs(90);

// ---------------------------------------------------------------------------
// Streaming cursor (cache hit = memory, miss = disk)
// ---------------------------------------------------------------------------

pub enum AudioCursor {
    Mem(Cursor<Arc<[u8]>>),
    Disk(BufReader<File>),
}

impl Read for AudioCursor {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Mem(c) => c.read(buf),
            Self::Disk(c) => c.read(buf),
        }
    }
}

impl Seek for AudioCursor {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match self {
            Self::Mem(c) => c.seek(pos),
            Self::Disk(c) => c.seek(pos),
        }
    }
}

// ---------------------------------------------------------------------------
// LRU cache
// ---------------------------------------------------------------------------

struct CacheEntry {
    data: Arc<[u8]>,
    last_used: Instant,
}

pub struct SoundCache {
    entries: HashMap<String, CacheEntry>,
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

    pub fn peek(&self, path: &str) -> Option<Arc<[u8]>> {
        self.entries.get(path).map(|e| e.data.clone())
    }

    /// Return cached bytes and promote entry to MRU position.
    pub fn get(&mut self, path: &str) -> Option<Arc<[u8]>> {
        if self.entries.contains_key(path) {
            self.order.retain(|p| p != path);
            self.order.push_front(path.to_string());
            if let Some(e) = self.entries.get_mut(path) {
                e.last_used = Instant::now();
                return Some(e.data.clone());
            }
        }
        None
    }

    /// Store bytes in the cache (LRU eviction when full).
    /// Files exceeding `max_entry` are returned as an `Arc` but never stored.
    pub fn insert(&mut self, path: String, data: Vec<u8>) -> Arc<[u8]> {
        if let Some(existing) = self.entries.get_mut(&path) {
            self.order.retain(|p| p != &path);
            self.order.push_front(path);
            existing.last_used = Instant::now();
            return existing.data.clone();
        }

        let arc: Arc<[u8]> = data.into();
        let size = arc.len();

        if size > self.max_entry {
            return arc;
        }

        while self.total_size + size > self.max_size {
            match self.order.pop_back() {
                Some(lru) => {
                    if let Some(evicted) = self.entries.remove(&lru) {
                        self.total_size = self.total_size.saturating_sub(evicted.data.len());
                    }
                }
                None => break,
            }
        }

        self.order.push_front(path.clone());
        self.total_size += size;
        self.entries.insert(
            path,
            CacheEntry {
                data: arc.clone(),
                last_used: Instant::now(),
            },
        );
        arc
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.order.clear();
        self.total_size = 0;
    }

    /// Drop one path so the next `load_bytes` re-reads from disk.
    pub fn invalidate(&mut self, path: &str) {
        if let Some(evicted) = self.entries.remove(path) {
            self.total_size = self.total_size.saturating_sub(evicted.data.len());
            self.order.retain(|p| p != path);
        }
    }

    /// Drop entries idle longer than TTL and not in `keep`.
    pub fn evict_idle(&mut self, keep: &HashSet<String>) {
        let now = Instant::now();
        let stale: Vec<String> = self
            .entries
            .iter()
            .filter(|(p, e)| !keep.contains(*p) && now.duration_since(e.last_used) > ENTRY_TTL)
            .map(|(p, _)| p.clone())
            .collect();
        for path in stale {
            self.invalidate(&path);
        }
    }

    pub fn set_limits(&mut self, max_size: usize, max_entry: usize) {
        self.max_size = max_size;
        self.max_entry = max_entry;
        while self.total_size > self.max_size {
            match self.order.pop_back() {
                Some(lru) => {
                    if let Some(evicted) = self.entries.remove(&lru) {
                        self.total_size = self.total_size.saturating_sub(evicted.data.len());
                    }
                }
                None => break,
            }
        }
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

/// Evict a single cached path (e.g. rewritten preview WAV).
pub fn invalidate_path(path: &str) {
    if let Ok(mut cache) = sound_cache().lock() {
        cache.invalidate(path);
    }
}

pub fn peek_bytes(path: &str) -> Option<Arc<[u8]>> {
    sound_cache().lock().ok()?.peek(path)
}

/// Open a decoder cursor: memory if cached, otherwise stream the file.
/// Cache miss also kicks a background warm so the next play is a hit.
pub fn open_cursor(path: &str) -> Result<AudioCursor, String> {
    if let Some(data) = peek_bytes(path) {
        if let Ok(mut cache) = sound_cache().lock() {
            let _ = cache.get(path);
        }
        return Ok(AudioCursor::Mem(Cursor::new(data)));
    }
    let file = File::open(path).map_err(|e| format!("Failed to open '{}': {}", path, e))?;
    warm_in_background(path.to_string());
    Ok(AudioCursor::Disk(BufReader::with_capacity(64 * 1024, file)))
}

pub fn evict_idle(keep: &[String]) {
    let set: HashSet<String> = keep.iter().cloned().collect();
    if let Ok(mut cache) = sound_cache().lock() {
        cache.evict_idle(&set);
    }
}

/// Return the raw bytes of a sound file (used by duration fallback / warm).
pub fn load_bytes(path: &str) -> Result<Arc<[u8]>, String> {
    {
        let mut cache = sound_cache().lock().map_err(|e| e.to_string())?;
        if let Some(data) = cache.get(path) {
            return Ok(data);
        }
    }

    let data =
        std::fs::read(path).map_err(|e| format!("Failed to read '{}': {}", path, e))?;

    let mut cache = sound_cache().lock().map_err(|e| e.to_string())?;
    Ok(cache.insert(path.to_string(), data))
}

fn warm_in_background(path: String) {
    std::thread::spawn(move || {
        if let Err(e) = load_bytes(&path) {
            eprintln!("[cache] warm failed '{}': {}", path, e);
        }
    });
}

// ---------------------------------------------------------------------------
// Duration cache (path + mtime + size)
// ---------------------------------------------------------------------------

static DURATION_CACHE: OnceLock<Mutex<HashMap<String, (u64, u64, f64)>>> = OnceLock::new();

fn duration_cache() -> &'static Mutex<HashMap<String, (u64, u64, f64)>> {
    DURATION_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn file_stamp(path: &str) -> Option<(u64, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    let size = meta.len();
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())?;
    Some((mtime, size))
}

pub fn cached_duration(path: &str) -> Option<f64> {
    let (mtime, size) = file_stamp(path)?;
    let guard = duration_cache().lock().ok()?;
    let (m, s, d) = guard.get(path)?;
    if *m == mtime && *s == size {
        Some(*d)
    } else {
        None
    }
}

pub fn store_duration(path: &str, secs: f64) {
    if let (Some((mtime, size)), Ok(mut guard)) = (file_stamp(path), duration_cache().lock()) {
        if guard.len() > 256 {
            guard.clear();
        }
        guard.insert(path.to_string(), (mtime, size, secs));
    }
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
