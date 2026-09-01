//! Global OS shortcuts for sound triggers (optional).

use serde::Deserialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSoundBinding {
    pub combo: String,
    pub sound_id: String,
}

static LAST_ACCELS: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn to_accelerator(combo: &str) -> String {
    if combo.starts_with("Ctrl+") {
        format!("CommandOrControl+{}", &combo["Ctrl+".len()..])
    } else {
        combo.to_string()
    }
}

#[tauri::command]
pub fn set_global_sound_hotkeys(
    app: AppHandle,
    enabled: bool,
    bindings: Vec<GlobalSoundBinding>,
) -> Result<(), String> {
    let gs = app.global_shortcut();
    {
        let mut last = LAST_ACCELS.lock().map_err(|e| e.to_string())?;
        last.clear();
    }
    let _ = gs.unregister_all();
    if !enabled {
        return Ok(());
    }

    let mut accels = Vec::new();
    for b in bindings {
        if b.combo.trim().is_empty() || b.sound_id.trim().is_empty() {
            continue;
        }
        let accel = to_accelerator(b.combo.trim());
        let sound_id = b.sound_id.clone();
        gs.on_shortcut(accel.as_str(), move |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let _ = app.emit("global_sound_hotkey", sound_id.clone());
            }
        })
        .map_err(|e| format!("register {accel}: {e}"))?;
        accels.push(accel);
    }
    if let Ok(mut last) = LAST_ACCELS.lock() {
        *last = accels;
    }
    Ok(())
}
