//! System tray + close-to-tray lifetime for tray-resident Desktop.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};
use tauri_plugin_notification::NotificationExt;

static TRAY_HIDE_NOTIFIED: AtomicBool = AtomicBool::new(false);

const CLOSE_TO_TRAY_KEY: &str = "close_to_tray";
const PREFS_FILE: &str = "desktop-prefs.json";

pub struct TrayPrefs {
    /// When true (default), window close hides to tray instead of exiting.
    pub close_to_tray: AtomicBool,
}

impl TrayPrefs {
    pub fn load() -> Self {
        let close = read_pref_bool(CLOSE_TO_TRAY_KEY).unwrap_or(true);
        Self {
            close_to_tray: AtomicBool::new(close),
        }
    }

    pub fn close_to_tray(&self) -> bool {
        self.close_to_tray.load(Ordering::Relaxed)
    }

    pub fn set_close_to_tray(&self, value: bool) {
        self.close_to_tray.store(value, Ordering::Relaxed);
        let _ = write_pref_bool(CLOSE_TO_TRAY_KEY, value);
    }
}

fn prefs_path() -> Option<std::path::PathBuf> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(std::path::PathBuf::from)?;
    Some(home.join(".pytxo").join(PREFS_FILE))
}

fn read_pref_bool(key: &str) -> Option<bool> {
    let path = prefs_path()?;
    let raw = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    v.get(key)?.as_bool()
}

fn write_pref_bool(key: &str, value: bool) -> std::io::Result<()> {
    let path = prefs_path().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "home directory missing")
    })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut map: serde_json::Map<String, serde_json::Value> = if path.exists() {
        match serde_json::from_str(&std::fs::read_to_string(&path)?) {
            Ok(serde_json::Value::Object(m)) => m,
            _ => serde_json::Map::new(),
        }
    } else {
        serde_json::Map::new()
    };
    map.insert(key.to_string(), serde_json::Value::Bool(value));
    std::fs::write(
        path,
        serde_json::to_string_pretty(&serde_json::Value::Object(map))?,
    )
}

pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn hide_main_window_to_tray<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    if !TRAY_HIDE_NOTIFIED.swap(true, Ordering::Relaxed) {
        let _ = app
            .notification()
            .builder()
            .title("Pytxo Desktop")
            .body("Still running in the tray. Open from the tray icon or Quit to exit.")
            .show();
    }
}

pub fn install_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let show_i = MenuItem::with_id(app, "show", "Show Pytxo Desktop", true, None::<&str>)?;
    let hide_i = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &hide_i, &quit_i])?;

    let mut builder = TrayIconBuilder::with_id("pytxo-main-tray")
        .tooltip("Pytxo Desktop")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "hide" => hide_main_window_to_tray(app),
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    let _ = builder.build(app)?;
    Ok(())
}

#[tauri::command]
pub fn set_tray_needs_you(app: AppHandle, count: u32) {
    let Some(tray) = app.tray_by_id("pytxo-main-tray") else {
        return;
    };
    let tip = if count == 0 {
        "Pytxo Desktop".to_string()
    } else if count == 1 {
        "Pytxo Desktop — 1 needs you".to_string()
    } else {
        format!("Pytxo Desktop — {count} need you")
    };
    let _ = tray.set_tooltip(Some(&tip));
}

#[tauri::command]
pub fn get_close_to_tray(prefs: tauri::State<'_, TrayPrefs>) -> bool {
    prefs.close_to_tray()
}

#[tauri::command]
pub fn set_close_to_tray(prefs: tauri::State<'_, TrayPrefs>, enabled: bool) {
    prefs.set_close_to_tray(enabled);
}

/// Shared flag so window close can decide hide vs exit without looking up State
/// from every event (State may not be available on all event paths).
pub fn close_to_tray_enabled(app: &AppHandle) -> bool {
    app.try_state::<TrayPrefs>()
        .map(|p| p.close_to_tray())
        .unwrap_or(true)
}
