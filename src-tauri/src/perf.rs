//! Opt-in timing output for performance work.
//!
//! `SN_PERF=1` turns on `[perf]` lines on stderr (the same place the rest of
//! the crate logs). `SN_PERF_SELFTEST=1` is only reported to the frontend,
//! which runs its own measurement pass when it sees the flag.

use serde::Serialize;
use std::sync::OnceLock;
use std::time::Instant;

fn env_flag(name: &str) -> bool {
    std::env::var(name).is_ok_and(|v| v == "1")
}

/// Whether `SN_PERF=1` was set when the process started. Read once: the hot
/// paths that ask must not pay for an environment lookup on every call.
pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| env_flag("SN_PERF"))
}

fn selftest() -> bool {
    static SELFTEST: OnceLock<bool> = OnceLock::new();
    *SELFTEST.get_or_init(|| env_flag("SN_PERF_SELFTEST"))
}

/// Write one `[perf]` line when perf logging is on. The message is built
/// lazily so disabled builds do not format anything.
pub fn log(message: impl FnOnce() -> String) {
    if enabled() {
        eprintln!("[perf] {}", message());
    }
}

/// Milliseconds since `since`, with sub-millisecond resolution for short spans.
pub fn ms_since(since: Instant) -> f64 {
    since.elapsed().as_secs_f64() * 1000.0
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct PerfFlags {
    pub perf: bool,
    pub selftest: bool,
}

#[tauri::command(async)]
pub fn perf_flags() -> PerfFlags {
    PerfFlags {
        perf: enabled(),
        selftest: selftest(),
    }
}

/// Lets the frontend put its own measurements next to the Rust ones.
#[tauri::command(async)]
pub fn perf_log(line: String) {
    eprintln!("[perf] {line}");
}
