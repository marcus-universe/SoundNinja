//! Off-thread execution helper for Tauri commands.
//!
//! Tauri runs a `#[tauri::command]` on the main thread unless it is `async` or
//! declared `#[tauri::command(async)]`. Short blocking work is fine on an async
//! worker; anything that can run for seconds belongs on the blocking pool so it
//! neither freezes the UI nor starves IPC.

/// Run a CPU-bound job on Tauri's blocking pool and await its result.
pub async fn run_blocking<T, F>(job: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(job)
        .await
        .map_err(|e| e.to_string())?
}
