use serde::Deserialize;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

/// WebView2 keeps one browser environment per process and refuses to create a
/// second one whose command line differs from the first. The main window sets
/// `additionalBrowserArgs`, so a tool window built with wry's defaults gets a
/// dead webview: the window is registered but never reaches the event loop and
/// every call on it fails with `FailedToReceiveMessage`. Reusing the exact args
/// from the main window config is what keeps tool windows alive.
fn main_browser_args(app: &AppHandle) -> String {
    app.config()
        .app
        .windows
        .first()
        .and_then(|w| w.additional_browser_args.clone())
        .unwrap_or_default()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolWindowSpec {
    label: String,
    url: String,
    title: String,
    width: f64,
    height: f64,
    min_width: Option<f64>,
    min_height: Option<f64>,
    decorations: bool,
}

/// Creates (or re-focuses) a tool window such as the Record Editor.
///
/// Must stay `async`: a sync command runs on the main thread and building a
/// window from there deadlocks WebView2 on Windows.
#[tauri::command]
pub async fn open_tool_window(app: AppHandle, spec: ToolWindowSpec) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(&spec.label) {
        if win.is_minimized().unwrap_or(false) {
            let _ = win.unminimize();
        }
        win.show().map_err(|e| e.to_string())?;
        let _ = win.set_focus();
        return Ok(());
    }

    let mut builder =
        WebviewWindowBuilder::new(&app, &spec.label, WebviewUrl::App(spec.url.into()))
            .title(&spec.title)
            .inner_size(spec.width, spec.height)
            .resizable(true)
            .visible(true)
            .focused(true)
            .decorations(spec.decorations)
            .additional_browser_args(&main_browser_args(&app));

    if let (Some(w), Some(h)) = (spec.min_width, spec.min_height) {
        builder = builder.min_inner_size(w, h);
    }

    let win = builder.build().map_err(|e| e.to_string())?;
    crate::menu::strip_window_menu_for(app, spec.label).await?;
    let _ = win.set_focus();
    Ok(())
}
