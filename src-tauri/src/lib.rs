pub mod log_watcher;
pub mod market_api;
pub mod ocr;
pub mod screenshot;

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use log::{error, info, warn};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::menu::{Menu, MenuItem};

use log_watcher::RewardScreenEvent;

static SETTINGS_OPEN: std::sync::OnceLock<std::sync::Arc<std::sync::atomic::AtomicBool>> =
    std::sync::OnceLock::new();

// Stored so the settings window can request it via get_log_path command.
static LOG_PATH_CACHE: std::sync::OnceLock<std::sync::Mutex<String>> =
    std::sync::OnceLock::new();

// Screenshot delay in milliseconds (default 1500).
static SCREENSHOT_DELAY_MS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(1500);

// Overlay vertical position as percent of screen height (default 50).
static OVERLAY_Y_PERCENT: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(50);

// ── Detection pipeline ────────────────────────────────────────────────────────

/// One item's resolved data — sent to the frontend via `reward-items-ready`.
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RewardItemPayload {
    name: String,
    plat_value: u32,
    ducat_value: u32,
}

/// screenshot → OCR → prices.  Runs fully async; screenshot and OCR steps
/// are offloaded to a blocking thread via `spawn_blocking`.
async fn run_detection_pipeline() -> anyhow::Result<Vec<RewardItemPayload>> {
    // Wait for the reward screen UI to fully render before screenshotting.
    let delay = SCREENSHOT_DELAY_MS.load(std::sync::atomic::Ordering::Relaxed);
    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;

    // 1. Screenshot (blocking subprocess).
    let png_bytes = tokio::task::spawn_blocking(screenshot::capture_reward_region)
        .await
        .map_err(|e| anyhow::anyhow!("screenshot task panicked: {e}"))?
        .map_err(|e| anyhow::anyhow!("screenshot failed: {e:#}"))?;

    // 2. OCR (blocking subprocess).
    let item_names = tokio::task::spawn_blocking(move || {
        // Save debug screenshot so we can inspect what was captured.
        let debug_path = std::env::temp_dir().join("tennohelios_debug.png");
        if let Err(e) = std::fs::write(&debug_path, &png_bytes) {
            warn!("Could not save debug screenshot: {e}");
        } else {
            info!("Debug screenshot saved to {}", debug_path.display());
        }
        ocr::extract_item_names(&png_bytes)
    })
        .await
        .map_err(|e| anyhow::anyhow!("OCR task panicked: {e}"))?
        .map_err(|e| anyhow::anyhow!("OCR failed: {e:#}"))?;

    // OCR returns exactly 4 slots; all empty = complete failure.
    if item_names.iter().all(|n| n.is_empty()) {
        return Err(anyhow::anyhow!("OCR returned no recognisable item names"));
    }

    // 3. Fetch prices from warframe.market.
    let prices = market_api::fetch_prices(&item_names)
        .await
        .map_err(|e| anyhow::anyhow!("price fetch failed: {e:#}"))?;

    let items = item_names
        .into_iter()
        .zip(prices)
        .map(|(name, price)| RewardItemPayload {
            name,
            plat_value: price.plat_avg_48h as u32,
            ducat_value: price.ducat_value,
        })
        .collect();

    Ok(items)
}

/// Try to find the Warframe window position via xdotool (X11/XWayland only).
/// Returns (x, y) of the window's top-left corner, or None if not found.
fn find_warframe_window_pos() -> Option<(i32, i32)> {
    // Try by Steam App ID class first (most reliable), then fall back to window name.
    let search_args: &[&[&str]] = &[
        &["search", "--classname", "steam_app_230410"],
        &["search", "--name", "Warframe"],
    ];

    let window_id = search_args.iter().find_map(|args| {
        let out = std::process::Command::new("xdotool")
            .args(*args)
            .output()
            .ok()?;
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .next()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
    })?;

    let geom_out = std::process::Command::new("xdotool")
        .args(["getwindowgeometry", &window_id])
        .output()
        .ok()?;

    // Output contains a line like: "  Position: 2560,0 (screen: 0)"
    for line in String::from_utf8_lossy(&geom_out.stdout).lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Position:") {
            if let Some(coords) = trimmed
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.split_once(','))
            {
                if let (Ok(x), Ok(y)) = (coords.0.parse::<i32>(), coords.1.parse::<i32>()) {
                    info!("Found Warframe window at ({x}, {y}) id={window_id}");
                    return Some((x, y));
                }
            }
        }
    }
    None
}

/// Remove the GNOME compositor shadow by setting the X11 window type to NOTIFICATION.
/// Must be called after the window is visible. Uses xdotool + xprop.
fn remove_compositor_shadow() {
    let Ok(out) = std::process::Command::new("xdotool")
        .args(["search", "--name", "TennoHelios"])
        .output()
    else {
        return;
    };
    // Use the last window ID — internal helper windows appear first, main window last.
    let Some(wid) = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .last()
        .map(str::trim)
        .map(str::to_owned)
    else {
        return;
    };
    let status = std::process::Command::new("xprop")
        .args([
            "-id", &wid,
            "-f", "_NET_WM_WINDOW_TYPE", "32a",
            "-set", "_NET_WM_WINDOW_TYPE", "_NET_WM_WINDOW_TYPE_DOCK",
        ])
        .status();
    info!("xprop shadow removal for wid={wid}: {status:?}");
}

/// Position the overlay window at the top-center of whichever monitor Warframe is on.
/// Falls back to primary monitor if Warframe window cannot be found.
fn position_overlay(app: &AppHandle) {
    let window = match app.get_webview_window("main") {
        Some(w) => w,
        None => {
            warn!("Could not get main window for positioning");
            return;
        }
    };

    let monitors = match window.available_monitors() {
        Ok(m) => m,
        Err(e) => {
            warn!("Could not enumerate monitors: {e}");
            return;
        }
    };

    if monitors.is_empty() {
        warn!("No monitors found");
        return;
    }

    // Determine which monitor Warframe is on.
    let warframe_pos = find_warframe_window_pos();
    info!("Warframe window position: {warframe_pos:?}");

    let target = if let Some((wx, wy)) = warframe_pos {
        monitors
            .iter()
            .find(|m| {
                let p = m.position();
                let s = m.size();
                wx >= p.x
                    && wx < p.x + s.width as i32
                    && wy >= p.y
                    && wy < p.y + s.height as i32
            })
            .or_else(|| monitors.first())
    } else {
        // No Warframe window found — use primary or first monitor.
        window
            .primary_monitor()
            .ok()
            .flatten()
            .as_ref()
            .and_then(|pm| monitors.iter().find(|m| m.name() == pm.name()))
            .or_else(|| monitors.first())
    };

    // Window is click-through by default; disabled only when showing overlay (handled by frontend).

    if let Some(monitor) = target {
        let pos = monitor.position();
        let size = monitor.size();

        // Reward cards occupy the center 50% of the screen width.
        // Overlay matches that area exactly so cards align under game cards.
        const OVERLAY_HEIGHT: u32 = 280;
        let overlay_width = size.width / 2;
        let x = pos.x + (size.width / 4) as i32;
        let y_pct = OVERLAY_Y_PERCENT.load(std::sync::atomic::Ordering::Relaxed) as f32 / 100.0;
        let y = pos.y + (size.height as f32 * y_pct) as i32;
        info!(
            "Positioning overlay on monitor {:?} — {overlay_width}x{OVERLAY_HEIGHT} at ({x}, {y})",
            monitor.name()
        );
        let _ = window.set_size(tauri::PhysicalSize::new(overlay_width, OVERLAY_HEIGHT));
        let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
    }

    // Remove GNOME compositor shadow after a short delay (window must be visible first).
    thread::spawn(|| {
        thread::sleep(std::time::Duration::from_millis(500));
        remove_compositor_shadow();
    });
}

/// Open settings as a separate, centered, non-transparent window.
fn open_settings(app: &AppHandle) {
    // If already open, just focus it.
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    let log_path = LOG_PATH_CACHE.get()
        .and_then(|m| m.lock().ok())
        .map(|s| s.clone())
        .unwrap_or_default();

    match tauri::WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("TennoHelios — Settings")
        .inner_size(720.0, 560.0)
        .min_inner_size(560.0, 400.0)
        .center()
        .resizable(true)
        .decorations(false)
        .transparent(false)
        .always_on_top(true)
        .build()
    {
        Ok(w) => {
            // Emit current settings to the new window after a short delay (WebView needs to load).
            let app_clone = app.clone();
            thread::spawn(move || {
                thread::sleep(std::time::Duration::from_millis(300));
                let _ = app_clone.emit_to("settings", "log-path", &log_path);
                let _ = app_clone.emit_to("settings", "show-settings", ());
            });
            info!("Settings window opened");
            let _ = w.show();
        }
        Err(e) => error!("Failed to open settings window: {e}"),
    }
}

/// Tauri command: set screenshot delay in ms (clamped 500–5000).
#[tauri::command]
fn set_screenshot_delay(ms: u64) {
    let clamped = ms.clamp(500, 5000);
    SCREENSHOT_DELAY_MS.store(clamped, std::sync::atomic::Ordering::Relaxed);
    info!("Screenshot delay set to {clamped}ms");
}

/// Tauri command: set overlay Y position as percent of screen height (clamped 10–90).
#[tauri::command]
fn set_overlay_y_percent(percent: u64, app: AppHandle) {
    let clamped = percent.clamp(10, 90);
    OVERLAY_Y_PERCENT.store(clamped, std::sync::atomic::Ordering::Relaxed);
    position_overlay(&app);
    info!("Overlay Y set to {clamped}%");
}

/// Tauri command: get current screenshot delay.
#[tauri::command]
fn get_screenshot_delay() -> u64 {
    SCREENSHOT_DELAY_MS.load(std::sync::atomic::Ordering::Relaxed)
}

/// Tauri command: get current overlay Y percent.
#[tauri::command]
fn get_overlay_y_percent() -> u64 {
    OVERLAY_Y_PERCENT.load(std::sync::atomic::Ordering::Relaxed)
}

/// Tauri command: parse Warframe version from EE.log ("Build label: Warframe34.x.x").
#[tauri::command]
fn get_warframe_version() -> String {
    let log_path = LOG_PATH_CACHE.get()
        .and_then(|m| m.lock().ok())
        .map(|s| std::path::PathBuf::from(s.clone()))
        .unwrap_or_else(log_watcher::default_log_path);

    let Ok(file) = std::fs::File::open(&log_path) else {
        return "Unknown".into();
    };
    use std::io::{BufRead, BufReader};
    for line in BufReader::new(file).lines().take(100).flatten() {
        if let Some(pos) = line.find("Build label:") {
            return line[pos + "Build label:".len()..].trim().to_string();
        }
    }
    "Unknown".into()
}

/// Tauri command: close the settings window.
#[tauri::command]
fn close_settings(app: AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.close();
    }
    info!("Settings window closed");
}

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
        use log_watcher::RewardScreenEvent;
        for event in rx {
            match event {
                RewardScreenEvent::Show { trigger_line } => {
                    // If settings was open, close it and restore overlay size first.
                    if let Some(flag) = SETTINGS_OPEN.get() {
                        if flag.swap(false, std::sync::atomic::Ordering::SeqCst) {
                            position_overlay(&app);
                            let _ = app.emit("hide-settings", ());
                        }
                    }
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_always_on_top(true);
                        let _ = std::process::Command::new("xdotool")
                            .args(["search", "--name", "TennoHelios", "windowraise"])
                            .spawn();
                        info!("Overlay shown: {trigger_line}");
                    }
                    // Notify frontend immediately so it can show a loading state.
                    if let Err(e) = app.emit("reward-screen-detected", &trigger_line) {
                        error!("Failed to emit show event: {e}");
                    }
                    // Run the full detection pipeline in the async runtime.
                    let app_clone = app.clone();
                    tauri::async_runtime::spawn(async move {
                        match run_detection_pipeline().await {
                            Ok(items) => {
                                if let Err(e) = app_clone.emit("reward-items-ready", &items) {
                                    error!("Failed to emit reward-items-ready: {e}");
                                }
                            }
                            Err(e) => {
                                error!("Detection pipeline failed: {e:#}");
                                let _ = app_clone.emit("reward-items-error", e.to_string());
                            }
                        }
                    });
                }
                RewardScreenEvent::Hide => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                        info!("Overlay hidden: reward screen dismissed");
                    }
                    if let Err(e) = app.emit("reward-screen-dismissed", ()) {
                        error!("Failed to emit hide event: {e}");
                    }
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            set_log_path, close_settings,
            set_screenshot_delay, set_overlay_y_percent,
            get_screenshot_delay, get_overlay_y_percent,
            get_warframe_version,
        ])
        .setup(|app| {
            let log_path = log_watcher::default_log_path();
            let log_path_str = log_path.display().to_string();
            // Cache log path for the settings window to use later.
            LOG_PATH_CACHE.set(std::sync::Mutex::new(log_path_str.clone())).ok();
            spawn_log_watcher(app.handle().clone(), log_path.clone());
            position_overlay(app.handle());
            // Hide overlay window until a reward screen is detected.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            // ── System tray ───────────────────────────────────────────────────
            let tray_settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let tray_quit    = MenuItem::with_id(app, "quit", "Quit TennoHelios", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&tray_settings, &tray_quit])?;

            let icon = app.default_window_icon()
                .cloned()
                .expect("no default window icon — check tauri.conf.json bundle.icon");

            let tray_handle = app.handle().clone();
            TrayIconBuilder::new()
                .icon(icon)
                .tooltip("TennoHelios — Warframe overlay")
                .menu(&tray_menu)
                .on_menu_event(move |_tray, event| match event.id().as_ref() {
                    "settings" => {
                        open_settings(&tray_handle);
                        info!("Tray: settings opened");
                    }
                    "quit" => {
                        info!("Tray: quit");
                        std::process::exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|_tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        // left-click on tray icon — reserved for future use
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running TennoHelios");
}
