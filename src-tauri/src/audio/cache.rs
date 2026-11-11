//! Cache control: warming, duration lookups and the cache commands.
//!
//! Decoded audio lives in [`super::pcm`]. This module decides what gets
//! decoded ahead of time and remembers track lengths.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use rodio::{ChannelCount, SampleRate};

use super::pcm::Prefetch;

/// Evict a single path from the cache (e.g. a rewritten preview WAV).
pub fn invalidate_path(path: &str) {
    super::pcm::invalidate(path);
}

// ---------------------------------------------------------------------------
// Duration cache (path + mtime + size)
// ---------------------------------------------------------------------------

/// Entries kept before the map is reset. Sized for several large boards: a
/// reset below the board size would throw away durations the warmer just
/// stored, and every lookup after that reopens its file.
const MAX_DURATION_ENTRIES: usize = 4096;

/// Path → `(mtime, size, duration in seconds)`.
type DurationMap = HashMap<String, (u64, u64, f64)>;

static DURATION_CACHE: OnceLock<Mutex<DurationMap>> = OnceLock::new();

fn duration_cache() -> &'static Mutex<DurationMap> {
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
        if guard.len() > MAX_DURATION_ENTRIES {
            guard.clear();
        }
        guard.insert(path.to_string(), (mtime, size, secs));
    }
}

// ---------------------------------------------------------------------------
// Warming
// ---------------------------------------------------------------------------

/// Bumped by every warm request. Workers of an older request stop before
/// their next file, so switching tabs quickly does not queue up whole boards.
static WARM_GENERATION: AtomicU64 = AtomicU64::new(0);

/// How long the warmer waits for the prewarmed stream to publish its format.
const PREWARM_WAIT: Duration = Duration::from_millis(1500);

/// Decode threads for warming. Leaves one core for playback and the UI, and
/// stays low on ARM boards where every core is busy and thermals are tight.
fn warm_threads() -> usize {
    let cap = if cfg!(any(target_arch = "aarch64", target_arch = "arm")) {
        3
    } else if cfg!(target_arch = "x86_64") {
        4
    } else {
        2
    };
    let cores = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(1);
    cores.saturating_sub(1).clamp(1, cap)
}

/// Decode target for warming. Prefers the format of the open (or just
/// prewarmed) stream, because that is the key a later play looks up; falls
/// back to asking the device.
fn resolve_warm_format(
    device_name: Option<&str>,
    host_name: Option<&str>,
) -> Option<(SampleRate, ChannelCount)> {
    if let Some(name) = device_name {
        let deadline = Instant::now() + PREWARM_WAIT;
        while super::playback::stream_format_for(name, host_name).is_none()
            && super::playback::prewarm_expected(host_name)
            && Instant::now() < deadline
        {
            std::thread::sleep(Duration::from_millis(25));
        }
    }
    super::playback::warm_format(device_name, host_name)
}

/// Decode `paths` with `threads` workers until done or until `keep_going`
/// turns false. Workers take the next index from a shared counter, so files
/// are started strictly in input order (the visible tab comes first).
/// Returns how many files were handed to `decode`.
fn warm_paths<D, K>(paths: &[String], threads: usize, decode: D, keep_going: K) -> usize
where
    D: Fn(&str) + Sync,
    K: Fn() -> bool + Sync,
{
    let next = AtomicUsize::new(0);
    let started = AtomicUsize::new(0);
    let worker = || {
        while keep_going() {
            let Some(path) = paths.get(next.fetch_add(1, Ordering::Relaxed)) else {
                break;
            };
            started.fetch_add(1, Ordering::Relaxed);
            decode(path);
        }
    };
    std::thread::scope(|scope| {
        for _ in 1..threads.clamp(1, paths.len().max(1)) {
            scope.spawn(worker);
        }
        // The calling thread is the first worker.
        worker();
    });
    started.into_inner()
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

/// Pre-decode a list of sound files so the first press is instant and the audio
/// callback never has to decode. Returns immediately; the work runs on its own
/// threads. A newer call replaces an older one that is still running.
///
/// `device_name` / `host_name` decide the target format. When that cannot be
/// resolved nothing is decoded; the first play of each sound then decodes it.
#[tauri::command(async)]
pub fn warm_sound_cache(paths: Vec<String>, device_name: Option<String>, host_name: Option<String>) {
    let generation = WARM_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    // These are the pads on screen: keep their buffers past the idle TTL.
    super::pcm::set_pinned(&paths);
    // Open the output stream now: the first play then skips the device open,
    // and the warmer below can decode to the format that stream really runs at.
    if let Some(name) = device_name.as_deref() {
        super::playback::request_prewarm(name, host_name.as_deref());
    }

    let spawned = std::thread::Builder::new()
        .name("sn-warm".into())
        .spawn(move || {
            let started = Instant::now();
            let current = || WARM_GENERATION.load(Ordering::SeqCst) == generation;
            // Resolved here rather than in the command: the fallback enumerates
            // devices, which is too slow for a tokio worker.
            let Some((rate, channels)) =
                resolve_warm_format(device_name.as_deref(), host_name.as_deref())
            else {
                crate::perf::log(|| "warm skipped: output format unknown".to_string());
                return;
            };
            let decoded = AtomicUsize::new(0);
            let threads = warm_threads();
            let visited = warm_paths(
                &paths,
                threads,
                |path| {
                    if super::pcm::prefetch_one(path, rate, channels) == Prefetch::Decoded {
                        decoded.fetch_add(1, Ordering::Relaxed);
                    }
                },
                current,
            );
            crate::perf::log(|| {
                format!(
                    "warm files={} visited={visited} decoded={} threads={threads} \
                     format={}Hz/{}ch superseded={} total={:.1}ms",
                    paths.len(),
                    decoded.load(Ordering::Relaxed),
                    rate.get(),
                    channels.get(),
                    !current(),
                    crate::perf::ms_since(started)
                )
            });
        });
    if let Err(e) = spawned {
        eprintln!("[cache] warm thread failed to start: {e}");
    }
}

/// Evict every cached sound, freeing RAM immediately.
#[tauri::command(async)]
pub fn clear_sound_cache() -> Result<(), String> {
    super::pcm::clear();
    Ok(())
}

/// Return current cache usage statistics (decoded PCM).
#[tauri::command(async)]
pub fn get_cache_stats() -> Result<CacheStats, String> {
    let (cached_count, total_size_bytes, max_size_bytes) = super::pcm::stats();
    Ok(CacheStats {
        cached_count,
        total_size_bytes,
        max_size_bytes,
    })
}

/// Update the cache size limits at runtime.
/// `max_size_mib`  – total RAM limit in MiB (min 32, max 4096).
/// `max_entry_mib` – per-file limit in MiB; files above this are never kept.
#[tauri::command(async)]
pub fn set_cache_config(max_size_mib: u64, max_entry_mib: u64) -> Result<(), String> {
    let max_size = (max_size_mib as usize).saturating_mul(1024 * 1024);
    let max_entry = (max_entry_mib as usize).saturating_mul(1024 * 1024);
    super::pcm::set_limits(max_size, max_entry);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{warm_paths, warm_threads};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;

    fn paths(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("sound-{i:03}")).collect()
    }

    #[test]
    fn warm_threads_stays_within_the_per_architecture_cap() {
        let threads = warm_threads();
        assert!(threads >= 1);
        assert!(threads <= 4);
    }

    #[test]
    fn warm_paths_visits_every_path_exactly_once() {
        let input = paths(50);
        let seen = Mutex::new(Vec::new());
        let visited = warm_paths(
            &input,
            4,
            |path| seen.lock().unwrap().push(path.to_owned()),
            || true,
        );
        assert_eq!(visited, 50);
        let mut seen = seen.into_inner().unwrap();
        seen.sort();
        assert_eq!(seen, input);
    }

    #[test]
    fn warm_paths_starts_files_in_input_order_on_one_thread() {
        let input = paths(10);
        let seen = Mutex::new(Vec::new());
        warm_paths(&input, 1, |path| seen.lock().unwrap().push(path.to_owned()), || true);
        assert_eq!(seen.into_inner().unwrap(), input);
    }

    #[test]
    fn warm_paths_stops_between_files_when_superseded() {
        let input = paths(100);
        let keep_going = AtomicBool::new(true);
        let visited = warm_paths(
            &input,
            1,
            // A newer warm request arrives while the third file decodes.
            |path| {
                if path == "sound-002" {
                    keep_going.store(false, Ordering::SeqCst);
                }
            },
            || keep_going.load(Ordering::SeqCst),
        );
        assert_eq!(visited, 3);
    }

    #[test]
    fn warm_paths_handles_an_empty_list() {
        assert_eq!(warm_paths(&[], 4, |_| {}, || true), 0);
    }
}
