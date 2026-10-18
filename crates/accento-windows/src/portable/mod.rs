//! Tauri shell shared by Linux and macOS; Windows keeps its established native host.
use crate::{
    i18n::{Text, tr},
    preferences::{Preferences, Theme},
};
use accento_core::View;
use serde::Serialize;
use std::sync::{Arc, Mutex, mpsc};
use tauri::{
    Emitter, Manager,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

#[derive(Clone, Serialize)]
pub struct Snapshot {
    pub preferences: Preferences,
    pub paused: bool,
    pub warning: Option<String>,
}
#[derive(Clone, Serialize)]
pub struct PickerView {
    candidates: Vec<String>,
    selected: Option<usize>,
    generation: u64,
}
impl From<View> for PickerView {
    fn from(view: View) -> Self {
        Self {
            candidates: view.candidates,
            selected: view.selected,
            generation: view.generation,
        }
    }
}
use accento_platform::Command;
fn desktop_test() -> bool {
    std::env::args().any(|arg| arg == "--desktop-test")
}
pub struct State {
    snapshot: Mutex<Snapshot>,
    control: Mutex<Option<Worker>>,
    picker: Mutex<Option<PickerView>>,
}
struct Worker {
    sender: mpsc::Sender<Command>,
    join: Option<std::thread::JoinHandle<()>>,
}
impl Worker {
    fn send(&self, command: Command) -> Result<(), mpsc::SendError<Command>> {
        self.sender.send(command)
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.sender.send(Command::Stop);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}
#[derive(Clone)]
pub struct Ui {
    pub app: tauri::AppHandle,
    state: Arc<State>,
}
impl Ui {
    pub fn picker(&self, view: Option<View>) {
        let view = view.map(PickerView::from);
        *self.state.picker.lock().unwrap() = view.clone();
        let app = self.app.clone();
        let _ = self.app.run_on_main_thread(move || {
            if let Some(window) = app.get_webview_window("picker") {
                if let Some(view) = view {
                    let monitor = app
                        .cursor_position()
                        .ok()
                        .and_then(|p| app.monitor_from_point(p.x, p.y).ok().flatten())
                        .or_else(|| app.primary_monitor().ok().flatten());
                    if let Some(monitor) = monitor {
                        let scale = monitor.scale_factor();
                        let width = ((view.candidates.len().min(12) as f64) * 72. + 8.)
                            .min(monitor.size().width as f64 / scale - 32.);
                        let x = monitor.position().x as f64
                            + (monitor.size().width as f64 - width * scale) / 2.;
                        let y = monitor.position().y as f64 + monitor.size().height as f64
                            - 160. * scale;
                        let _ = window.set_size(tauri::LogicalSize::new(width, 88.));
                        let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
                    }
                    let _ = window.emit("accento-picker", Some(view));
                    let _ = window.show();
                } else {
                    let _ = window.emit("accento-picker", Option::<PickerView>::None);
                    let _ = window.hide();
                }
            }
        });
    }
    pub fn failed(&self, message: String) {
        eprintln!("Accento input unavailable: {message}");
        let mut snapshot = self.state.snapshot.lock().unwrap();
        snapshot.paused = true;
        snapshot.warning = Some(message);
        let _ = self.app.emit("accento-state", snapshot.clone());
        drop(snapshot);
        self.picker(None);
    }
}
impl accento_platform::DesktopUi for Ui {
    fn picker(&self, view: Option<View>) {
        self.picker(view);
    }
    fn failed(&self, message: String) {
        self.failed(message);
    }
}
fn publish(app: &tauri::AppHandle, state: &State) -> Snapshot {
    let snapshot = state.snapshot.lock().unwrap().clone();
    let _ = app.emit("accento-state", &snapshot);
    if let Some(tray) = app.tray_by_id("accento") {
        let _ = tray.set_tooltip(Some(if snapshot.paused {
            "Accento · Paused"
        } else {
            "Accento"
        }));
    }
    snapshot
}
fn start(app: &tauri::AppHandle, state: &Arc<State>) -> Result<(), String> {
    let preferences = state.snapshot.lock().unwrap().preferences.clone();
    let (tx, rx) = mpsc::channel();
    let (ready_tx, ready_rx) = mpsc::channel();
    let ui = Ui {
        app: app.clone(),
        state: state.clone(),
    };
    let join = std::thread::Builder::new()
        .name("accento-input".into())
        .spawn(move || {
            let result =
                accento_platform::run(preferences, rx, Arc::new(ui.clone()), ready_tx.clone());
            if let Err(error) = result {
                let _ = ready_tx.send(Err(error.clone()));
                ui.failed(error);
            }
        })
        .map_err(|e| e.to_string())?;
    ready_rx
        .recv_timeout(std::time::Duration::from_secs(10))
        .map_err(|e| e.to_string())??;
    *state.control.lock().unwrap() = Some(Worker {
        sender: tx,
        join: Some(join),
    });
    Ok(())
}
#[tauri::command]
fn get_settings(state: tauri::State<'_, Arc<State>>) -> Snapshot {
    state.snapshot.lock().unwrap().clone()
}
#[tauri::command]
fn get_picker(state: tauri::State<'_, Arc<State>>) -> Option<PickerView> {
    state.picker.lock().unwrap().clone()
}
#[tauri::command]
fn finish_desktop_test(app: tauri::AppHandle, report: serde_json::Value) -> Result<(), String> {
    if !desktop_test() {
        return Err("Desktop verification is not enabled.".into());
    }
    let path = std::env::var_os("ACCENTO_DESKTOP_REPORT")
        .ok_or("Desktop verification report path is missing.")?;
    std::fs::write(
        path,
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    app.exit(if report["passed"] == true { 0 } else { 1 });
    Ok(())
}
#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}
#[tauri::command]
fn save_settings(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<State>>,
    mut preferences: Preferences,
) -> Result<Snapshot, String> {
    preferences.theme = Theme::Dark;
    preferences.validate()?;
    preferences.save()?;
    if let Some(control) = state.control.lock().unwrap().as_ref() {
        let _ = control.send(Command::Configure(preferences.clone()));
    }
    {
        let mut snapshot = state.snapshot.lock().unwrap();
        snapshot.preferences = preferences;
        if !snapshot.paused {
            snapshot.warning = None;
        }
    }
    Ok(publish(&app, &state))
}
#[tauri::command]
fn set_paused(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<State>>,
    paused: bool,
) -> Result<Snapshot, String> {
    set_pause(&app, &state, paused)
}
fn set_pause(app: &tauri::AppHandle, state: &Arc<State>, paused: bool) -> Result<Snapshot, String> {
    let alive = state
        .control
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|tx| tx.send(Command::Pause(paused)).is_ok());
    if !paused && !alive && !desktop_test() {
        start(app, state)?;
    }
    {
        let mut snapshot = state.snapshot.lock().unwrap();
        snapshot.paused = paused;
        if !paused {
            snapshot.warning = None;
        }
    }
    Ok(publish(app, state))
}
fn show_settings(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
    } else {
        tauri::WebviewWindowBuilder::new(
            app,
            "settings",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .title("Accento · Settings")
        .inner_size(1040., 780.)
        .min_inner_size(640., 540.)
        .build()?;
    }
    Ok(())
}
pub fn run() -> Result<(), String> {
    #[cfg(target_os = "linux")]
    accento_platform::initialize();
    let (mut preferences, warning) = Preferences::load();
    preferences.theme = Theme::Dark;
    let state = Arc::new(State {
        snapshot: Mutex::new(Snapshot {
            preferences,
            warning,
            paused: false,
        }),
        control: Mutex::new(None),
        picker: Mutex::new(None),
    });
    let setup_state = state.clone();
    let builder = tauri::Builder::default()
        .manage(state.clone())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            set_paused,
            get_picker,
            finish_desktop_test,
            quit_app,
            crate::platform_info::get_platform
        ])
        .on_page_load(|webview, payload| {
            if desktop_test()
                && std::env::var_os("ACCENTO_DESKTOP_REPORT").is_some()
                && webview.label() == "settings"
                && payload.event() == tauri::webview::PageLoadEvent::Finished
            {
                let _ = webview.eval(include_str!("smoke.js"));
            }
        });
    let builder =
        if std::env::args().any(|arg| matches!(arg.as_str(), "--desktop-test" | "--input-test")) {
            builder
        } else {
            builder.plugin(tauri_plugin_single_instance::init(|app, _, _| {
                let _ = show_settings(app);
            }))
        };
    builder
        .setup(move |app| {
            let lang = setup_state
                .snapshot
                .lock()
                .unwrap()
                .preferences
                .ui_language
                .clone();
            let settings = MenuItem::with_id(
                app,
                "settings",
                tr(&lang, Text::Settings),
                true,
                None::<&str>,
            )?;
            let pause = MenuItem::with_id(
                app,
                "pause",
                format!("{} / {}", tr(&lang, Text::Pause), tr(&lang, Text::Resume)),
                true,
                None::<&str>,
            )?;
            let exit = MenuItem::with_id(app, "exit", tr(&lang, Text::Exit), true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings, &pause, &exit])?;
            let menu_state = setup_state.clone();
            TrayIconBuilder::with_id("accento")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Accento")
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "settings" => {
                        let _ = show_settings(app);
                    }
                    "pause" => {
                        let paused = !menu_state.snapshot.lock().unwrap().paused;
                        if let Err(error) = set_pause(app, &menu_state, paused) {
                            Ui {
                                app: app.clone(),
                                state: menu_state.clone(),
                            }
                            .failed(error);
                        }
                    }
                    "exit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            tauri::WebviewWindowBuilder::new(
                app,
                "picker",
                tauri::WebviewUrl::App("index.html?surface=picker".into()),
            )
            .title("Accento · Variants")
            .inner_size(152., 88.)
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .focusable(false)
            .focused(false)
            .visible(false)
            .resizable(false)
            .build()?;
            if !desktop_test()
                && let Err(error) = start(app.handle(), &setup_state)
            {
                Ui {
                    app: app.handle().clone(),
                    state: setup_state.clone(),
                }
                .failed(error);
            }
            if !std::env::args().any(|arg| arg == "--background") {
                show_settings(app.handle())?;
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .map_err(|e| e.to_string())?
        .run(move |_, event| {
            if let tauri::RunEvent::ExitRequested {
                code: None,
                ref api,
                ..
            } = event
            {
                api.prevent_exit();
            }
            if let tauri::RunEvent::Exit = event
                && let Some(control) = state.control.lock().unwrap().take()
            {
                drop(control);
            }
        });
    Ok(())
}
