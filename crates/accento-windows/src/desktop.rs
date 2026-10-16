//! Tauri owns the main thread. The existing native input/picker loop owns its STA thread.
use crate::preferences::Preferences;
use serde::Serialize;
use std::sync::{
    OnceLock,
    atomic::{AtomicIsize, Ordering},
    mpsc,
};
use tauri::{Emitter, Manager};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW, PostMessageW, WM_APP, WM_COMMAND},
};

pub const REQUEST: u32 = WM_APP + 30;
static HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();
static NATIVE: AtomicIsize = AtomicIsize::new(0);

#[derive(Clone, Serialize)]
pub struct Snapshot {
    pub preferences: Preferences,
    pub paused: bool,
    pub warning: Option<String>,
}
pub enum Action {
    Read,
    Save(Preferences),
    Pause(bool),
}
pub struct Request {
    pub action: Action,
    pub reply: mpsc::Sender<Result<Snapshot, String>>,
}
struct Bridge(mpsc::Sender<Request>);

pub fn active() -> bool {
    HANDLE.get().is_some()
}
pub fn ready(hwnd: HWND) {
    NATIVE.store(hwnd.0 as isize, Ordering::Release);
}
pub fn publish(snapshot: &Snapshot) {
    if let Some(app) = HANDLE.get() {
        let _ = app.emit("accento-state", snapshot);
    }
}
pub fn show_settings() {
    if let Some(app) = HANDLE.get() {
        let handle = app.clone();
        // Always create WebView2 on the Tauri event loop, outside native window callbacks.
        let _ = app.run_on_main_thread(move || {
            if let Some(window) = handle.get_webview_window("settings") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            } else if let Err(error) = tauri::WebviewWindowBuilder::new(
                &handle,
                "settings",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("Accento · Impostazioni")
            .inner_size(1040., 780.)
            .min_inner_size(640., 540.)
            .build()
            {
                eprintln!("Impossibile aprire le impostazioni: {error}");
                let text: Vec<u16> = format!("Impossibile aprire le impostazioni.\nVerifica che Microsoft Edge WebView2 Runtime sia installato.\n\n{error}")
                    .encode_utf16().chain(Some(0)).collect();
                unsafe { MessageBoxW(None, windows::core::PCWSTR(text.as_ptr()), windows::core::w!("Accento"), MB_OK | MB_ICONERROR); }
            }
        });
    }
}
#[tauri::command]
fn quit_app() -> Result<(), String> {
    let hwnd = NATIVE.load(Ordering::Acquire);
    if hwnd == 0 {
        return Err("Il motore di Accento non è disponibile.".into());
    }
    unsafe { PostMessageW(Some(HWND(hwnd as _)), WM_COMMAND, WPARAM(102), LPARAM(0)) }
        .map_err(|error| error.to_string())
}
fn request(bridge: &Bridge, action: Action) -> Result<Snapshot, String> {
    let hwnd = NATIVE.load(Ordering::Acquire);
    if hwnd == 0 {
        return Err("Il motore di Accento non è disponibile.".into());
    }
    let (reply, result) = mpsc::channel();
    bridge
        .0
        .send(Request { action, reply })
        .map_err(|e| e.to_string())?;
    unsafe { PostMessageW(Some(HWND(hwnd as _)), REQUEST, WPARAM(0), LPARAM(0)) }
        .map_err(|e| e.to_string())?;
    result
        .recv_timeout(std::time::Duration::from_secs(5))
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn get_settings(bridge: tauri::State<'_, Bridge>) -> Result<Snapshot, String> {
    let bridge = Bridge(bridge.0.clone());
    tauri::async_runtime::spawn_blocking(move || request(&bridge, Action::Read))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn save_settings(
    bridge: tauri::State<'_, Bridge>,
    preferences: Preferences,
) -> Result<Snapshot, String> {
    let bridge = Bridge(bridge.0.clone());
    tauri::async_runtime::spawn_blocking(move || request(&bridge, Action::Save(preferences)))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
async fn set_paused(bridge: tauri::State<'_, Bridge>, paused: bool) -> Result<Snapshot, String> {
    let bridge = Bridge(bridge.0.clone());
    tauri::async_runtime::spawn_blocking(move || request(&bridge, Action::Pause(paused)))
        .await
        .map_err(|e| e.to_string())?
}

pub fn run() -> Result<(), String> {
    let (sender, receiver) = mpsc::channel();
    tauri::Builder::default()
        .manage(Bridge(sender))
        .invoke_handler(tauri::generate_handler![
            crate::platform_info::get_platform,
            quit_app,
            get_settings,
            save_settings,
            set_paused
        ])
        .setup(move |app| {
            HANDLE
                .set(app.handle().clone())
                .map_err(|_| "Desktop già inizializzato")?;
            let (ready_tx, ready_rx) = mpsc::channel();
            let handle = app.handle().clone();
            std::thread::Builder::new()
                .name("accento-native".into())
                .spawn(move || {
                    let result = crate::app::run_web(receiver, ready_tx.clone());
                    if let Err(error) = &result {
                        let _ = ready_tx.send(Err(error.clone()));
                    }
                    NATIVE.store(0, Ordering::Release);
                    handle.exit(if result.is_ok() { 0 } else { 1 });
                })?;
            ready_rx.recv_timeout(std::time::Duration::from_secs(15))??;
            if NATIVE.load(Ordering::Acquire) == 0 {
                return Ok(());
            }
            if !std::env::args().any(|a| a == "--background") {
                show_settings();
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .map_err(|e| e.to_string())?
        .run(|_, event| {
            if let tauri::RunEvent::ExitRequested {
                code: None,
                ref api,
                ..
            } = event
            {
                // Closing the last settings window frees its webview; the tray and input remain active.
                api.prevent_exit();
            }
            if let tauri::RunEvent::Exit = event {
                let hwnd = NATIVE.load(Ordering::Acquire);
                if hwnd != 0 {
                    unsafe {
                        let _ =
                            PostMessageW(Some(HWND(hwnd as _)), WM_COMMAND, WPARAM(102), LPARAM(0));
                    }
                }
            }
        });
    Ok(())
}
