pub mod log_watcher;

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use log::error;
use tauri::{AppHandle, Emitter};

use log_watcher::RewardScreenEvent;

/// Tauri command: override the EE.log path at runtime (for testing / custom installs).
#[tauri::command]
fn set_log_path(app: AppHandle, path: String) {
    let path = PathBuf::from(path);
    spawn_log_watcher(app, path);
}

/// Spawn the log watcher in a background thread and forward events to the frontend.
pub fn spawn_log_watcher(app: AppHandle, log_path: PathBuf) {
    let (tx, rx) = mpsc::channel::<RewardScreenEvent>();

    // Watcher thread — blocks on the notify event loop.
    thread::spawn(move || {
        if let Err(e) = log_watcher::watch(log_path, tx) {
            error!("Log watcher exited with error: {e}");
        }
    });

    // Forwarder thread — relays channel events to Tauri frontend.
    thread::spawn(move || {
        for event in rx {
            if let Err(e) = app.emit("reward-screen-detected", &event.trigger_line) {
                error!("Failed to emit event to frontend: {e}");
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![set_log_path])
        .setup(|app| {
            let log_path = log_watcher::default_log_path();
            spawn_log_watcher(app.handle().clone(), log_path);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running TennoHelios");
}
