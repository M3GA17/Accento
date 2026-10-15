use crate::{
    input::{self, UiEvent, Worker},
    preferences::{Preferences, Theme},
    render::{self, Renderer},
};
use accento_core::{Effect, View};
mod settings;
use std::{
    cell::RefCell,
    sync::mpsc::{self, Receiver},
    time::Duration,
};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        Storage::Xps::{PRINT_WINDOW_FLAGS, PrintWindow},
        System::{Com::*, LibraryLoader::GetModuleHandleW, Threading::*},
        UI::{
            Accessibility::*, Controls::*, HiDpi::*, Input::KeyboardAndMouse::*, Shell::*,
            WindowsAndMessaging::*,
        },
    },
    core::{PCWSTR, w},
};

const TRAY: u32 = WM_APP + 10;
const TEST_DONE: u32 = WM_APP + 11;
const TEST_CONFIG: u32 = WM_APP + 12;
const TEST_FOCUS: u32 = WM_APP + 13;
const TEST_UI: u32 = WM_APP + 14;
const TEST_RENDER: u32 = WM_APP + 15;
const SETTINGS: u16 = 100;
const PAUSE: u16 = 101;
const EXIT: u16 = 102;
const SAVE: u16 = 103;
const DELAY: u16 = 200;
const TRANSPARENCY: u16 = 202;
const EXCLUDED: u16 = 203;
const TEST_FIELD: u16 = 204;
const STATUS: u16 = 205;
const LANG_FIRST: u16 = 210;

struct App {
    main: HWND,
    overlay: HWND,
    prefs: Preferences,
    draft: Preferences,
    hover: u16,
    dirty: bool,
    validation_error: Option<String>,
    scroll: f32,
    section: settings::Section,
    worker: Option<Worker>,
    events: Receiver<UiEvent>,
    view: Option<View>,
    renderer: Option<Renderer>,
    material: bool,
    paused: bool,
    font: HFONT,
    title_font: HFONT,
    section_font: HFONT,
    controls: RefCell<Vec<(usize, [i32; 4], u8)>>,
    icon: HICON,
    tray: bool,
    test_result: Option<bool>,
}
thread_local! { static APP: RefCell<Option<App>> = const { RefCell::new(None) }; }
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn id(value: u16) -> HMENU {
    HMENU(value as usize as _)
}
unsafe fn font(dpi: u32, size: f32, weight: i32) -> HFONT {
    unsafe {
        CreateFontW(
            -((size * dpi as f32 / 96.) as i32),
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH.0 as u32,
            w!("Segoe UI"),
        )
    }
}
unsafe fn font_for_title(dpi: u32) -> HFONT {
    unsafe { font(dpi, 28., FW_SEMIBOLD.0 as i32) }
}
unsafe fn font_for_section(dpi: u32) -> HFONT {
    unsafe { font(dpi, 11., FW_SEMIBOLD.0 as i32) }
}
unsafe fn send_message(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe { windows::Win32::UI::WindowsAndMessaging::SendMessageW(hwnd, msg, Some(wp), Some(lp)) }
}

pub fn run() -> Result<(), String> {
    unsafe { run_windows() }.map_err(|e| e.to_string())
}
unsafe fn run_windows() -> windows::core::Result<()> {
    unsafe {
        let self_test = std::env::args().any(|a| a == "--self-test");
        let background = std::env::args().any(|a| a == "--background");
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        let mutex = CreateMutexW(
            None,
            true,
            if self_test {
                w!("Local\\Accento.Test.Instance")
            } else {
                w!("Local\\Accento.Instance")
            },
        )?;
        if GetLastError() == ERROR_ALREADY_EXISTS && !self_test {
            if let Ok(existing) = FindWindowW(w!("Accento.Settings"), PCWSTR::null()) {
                let _ = ShowWindow(existing, SW_RESTORE);
                let _ = SetForegroundWindow(existing);
            }
            let _ = CloseHandle(mutex);
            CoUninitialize();
            return Ok(());
        }
        let module = HINSTANCE(GetModuleHandleW(None)?.0);
        let cursor = LoadCursorW(None, IDC_ARROW)?;
        for (name, procedure, background) in [
            (
                w!("Accento.Content"),
                Some(
                    settings::content_proc
                        as unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
                ),
                HBRUSH::default(),
            ),
            (
                w!("Accento.Settings"),
                Some(
                    settings_proc
                        as unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
                ),
                HBRUSH((COLOR_WINDOW.0 + 1) as usize as _),
            ),
            (
                w!("Accento.Picker"),
                Some(
                    picker_proc as unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
                ),
                HBRUSH::default(),
            ),
            (
                w!("Accento.Test"),
                Some(test_proc as unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT),
                HBRUSH((COLOR_WINDOW.0 + 1) as usize as _),
            ),
        ] {
            let class = WNDCLASSW {
                hInstance: module,
                lpfnWndProc: procedure,
                lpszClassName: name,
                hCursor: cursor,
                hbrBackground: background,
                ..Default::default()
            };
            if RegisterClassW(&class) == 0 {
                return Err(windows::core::Error::from_thread());
            }
        }
        let main = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("Accento.Settings"),
            w!("Accento · Impostazioni"),
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_VSCROLL,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            720,
            780,
            None,
            None,
            Some(module),
            None,
        )?;
        let overlay = CreateWindowExW(
            WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
            w!("Accento.Picker"),
            w!("Accento · Varianti"),
            WS_POPUP,
            0,
            0,
            400,
            100,
            None,
            None,
            Some(module),
            None,
        )?;
        let (prefs, warning) = if self_test {
            (Preferences::default(), None)
        } else {
            Preferences::load()
        };
        let (sender, receiver) = mpsc::channel();
        let worker = Worker::start(main, prefs.clone(), sender)
            .map_err(|message| windows::core::Error::new(E_FAIL, message))?;
        let dpi = GetDpiForWindow(main);
        let font = CreateFontW(
            -((13. * dpi as f32 / 96. * render::text_scale()) as i32),
            0,
            0,
            0,
            FW_NORMAL.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            DEFAULT_PITCH.0 as u32,
            w!("Segoe UI"),
        );
        let icon = create_icon()?;
        send_message(
            main,
            WM_SETICON,
            WPARAM(ICON_BIG as usize),
            LPARAM(icon.0 as isize),
        );
        send_message(
            main,
            WM_SETICON,
            WPARAM(ICON_SMALL as usize),
            LPARAM(icon.0 as isize),
        );
        APP.with(|a| {
            *a.borrow_mut() = Some(App {
                main,
                overlay,
                draft: prefs.clone(),
                prefs,
                hover: 0,
                dirty: false,
                validation_error: None,
                scroll: 0.,
                section: settings::Section::Writing,
                worker: Some(worker),
                events: receiver,
                view: None,
                renderer: None,
                material: false,
                paused: false,
                font,
                title_font: font_for_title(dpi),
                section_font: font_for_section(dpi),
                controls: RefCell::new(Vec::new()),
                icon,
                tray: false,
                test_result: None,
            })
        });
        APP.with(|a| {
            let mut a = a.borrow_mut();
            let a = a.as_mut().unwrap();
            build_settings(a);
            if !self_test {
                a.tray = tray_icon(a, NIM_ADD);
            }
        });
        settings::layout(main);
        if let Some(warning) = warning {
            alert(main, &warning);
        }
        if self_test {
            let _ = ShowWindow(main, SW_SHOWNORMAL);
            let _ = ShowWindow(main, SW_SHOWNORMAL);
            let _ = RedrawWindow(
                Some(main),
                None,
                None,
                RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN,
            );
            let _ = UpdateWindow(main);
            let folder = std::env::current_exe()?.parent().unwrap().to_path_buf();
            let _ = capture_window(main, &folder.join("accento-settings.bmp"));
            let _ = ShowWindow(main, SW_HIDE);
            start_test(main)?;
        } else if !background {
            let _ = ShowWindow(main, SW_SHOW);
            let _ = SetForegroundWindow(main);
        }
        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).0 > 0 {
            // Native controls retain keyboard traversal; the picker never receives focus.
            if !IsDialogMessageW(main, &message).as_bool() {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        let (result, worker) = APP.with(|a| {
            let mut app = a.borrow_mut().take().unwrap();
            if app.tray {
                tray_icon(&app, NIM_DELETE);
            }
            let _ = DestroyWindow(app.overlay);
            let _ = DestroyWindow(app.main);
            let _ = DeleteObject(HGDIOBJ(app.font.0));
            let _ = DeleteObject(HGDIOBJ(app.title_font.0));
            let _ = DeleteObject(HGDIOBJ(app.section_font.0));
            let _ = DestroyIcon(app.icon);
            (app.test_result, app.worker.take())
        });
        drop(worker);
        crate::glass::release_dc();
        let _ = CloseHandle(mutex);
        CoUninitialize();
        if self_test && result != Some(true) {
            return Err(windows::core::Error::new(
                E_FAIL,
                "Collaudo Windows non riuscito; consultare il rapporto nella cartella target.",
            ));
        }
        Ok(())
    }
}

// Keep the native class, style and geometry together at each control declaration.
#[allow(clippy::too_many_arguments)]
unsafe fn child(
    app: &App,
    class: PCWSTR,
    text: &str,
    style: WINDOW_STYLE,
    control: u16,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> HWND {
    unsafe {
        let scale = GetDpiForWindow(app.main) as f32 / 96.;
        let encoded = wide(text);
        let window = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class,
            PCWSTR(encoded.as_ptr()),
            WS_CHILD | WS_VISIBLE | style,
            (x as f32 * scale) as i32,
            (y as f32 * scale) as i32,
            (width as f32 * scale) as i32,
            (height as f32 * scale) as i32,
            Some(app.main),
            Some(id(control)),
            None,
            None,
        )
        .unwrap_or_default();
        let role = if control == 0 && text == "Accento" {
            1
        } else if control == 0
            && text.chars().any(char::is_alphabetic)
            && text == text.to_uppercase()
            && text.len() < 40
        {
            2
        } else {
            0
        };
        let font = match role {
            1 => app.title_font,
            2 => app.section_font,
            _ => app.font,
        };
        send_message(window, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
        app.controls
            .borrow_mut()
            .push((window.0 as usize, [x, y, width, height], role));
        window
    }
}
unsafe fn build_settings(app: &mut App) {
    unsafe {
        settings::build(app);
    }
}
unsafe fn control(main: HWND, identifier: u16) -> HWND {
    unsafe {
        GetDlgItem(Some(main), identifier as i32)
            .or_else(|_| {
                let content = GetDlgItem(Some(main), settings::CONTENT as i32)?;
                GetDlgItem(Some(content), identifier as i32)
            })
            .unwrap_or_default()
    }
}
unsafe fn control_text(main: HWND, identifier: u16) -> String {
    unsafe {
        let window = control(main, identifier);
        let n = GetWindowTextLengthW(window);
        let mut text = vec![0u16; n as usize + 1];
        let n = GetWindowTextW(window, &mut text);
        String::from_utf16_lossy(&text[..n as usize])
    }
}
unsafe fn status(main: HWND, text: &str) {
    unsafe {
        let text = wide(text);
        let _ = SetWindowTextW(control(main, STATUS), PCWSTR(text.as_ptr()));
        let _ = InvalidateRect(Some(main), None, false);
    }
}
unsafe fn alert(main: HWND, text: &str) {
    unsafe {
        let text = wide(text);
        MessageBoxW(
            Some(main),
            PCWSTR(text.as_ptr()),
            w!("Accento"),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

unsafe extern "system" fn settings_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe {
        if let Some(result) = settings::message(hwnd, msg, wp, lp) {
            return result;
        }
        match msg {
            UI_EVENT_ALIAS => {
                drain_events();
                return LRESULT(0);
            }
            WM_CTLCOLORSTATIC | WM_CTLCOLORBTN => {
                let dc = HDC(wp.0 as _);
                SetBkMode(dc, TRANSPARENT);
                SetTextColor(dc, COLORREF(GetSysColor(COLOR_WINDOWTEXT)));
                return LRESULT(GetSysColorBrush(COLOR_WINDOW).0 as isize);
            }
            WM_COMMAND => {
                let command = (wp.0 & 0xffff) as u16;
                if settings::command(hwnd, command, (wp.0 >> 16) as u16) {
                    return LRESULT(0);
                }
                if command == SETTINGS {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                    let _ = SetForegroundWindow(hwnd);
                    return LRESULT(0);
                }
                if command == PAUSE {
                    APP.with(|a| {
                        if let Some(a) = a.borrow_mut().as_mut() {
                            a.paused = !a.paused;
                            if let Some(w) = &a.worker {
                                w.pause(a.paused);
                            }
                            status(
                                hwnd,
                                if a.paused {
                                    "In pausa"
                                } else {
                                    "Attivo · Solo sul tuo dispositivo"
                                },
                            );
                            tray_icon(a, NIM_MODIFY);
                        }
                    });
                    return LRESULT(0);
                }
                if command == EXIT {
                    PostQuitMessage(0);
                    return LRESULT(0);
                }
            }
            TRAY => {
                if lp.0 as u32 == WM_LBUTTONDBLCLK {
                    let _ =
                        PostMessageW(Some(hwnd), WM_COMMAND, WPARAM(SETTINGS as usize), LPARAM(0));
                }
                if matches!(lp.0 as u32, WM_RBUTTONUP | WM_CONTEXTMENU) {
                    let paused = APP.with(|a| a.borrow().as_ref().is_some_and(|a| a.paused));
                    if let Ok(menu) = CreatePopupMenu() {
                        let _ = AppendMenuW(menu, MF_STRING, SETTINGS as usize, w!("Impostazioni"));
                        let _ = AppendMenuW(
                            menu,
                            MF_STRING,
                            PAUSE as usize,
                            if paused { w!("Riprendi") } else { w!("Pausa") },
                        );
                        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null());
                        let _ = AppendMenuW(menu, MF_STRING, EXIT as usize, w!("Esci"));
                        let mut point = POINT::default();
                        let _ = GetCursorPos(&mut point);
                        let _ = SetForegroundWindow(hwnd);
                        let _ = TrackPopupMenu(
                            menu,
                            TPM_RIGHTBUTTON,
                            point.x,
                            point.y,
                            Some(0),
                            hwnd,
                            None,
                        );
                        let _ = DestroyMenu(menu);
                        let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));
                    }
                }
                return LRESULT(0);
            }
            WM_CLOSE => {
                let has_tray = APP.with(|a| a.borrow().as_ref().is_some_and(|a| a.tray));
                if has_tray {
                    let _ = ShowWindow(hwnd, SW_HIDE);
                } else {
                    PostQuitMessage(0);
                }
                return LRESULT(0);
            }
            TEST_DONE => {
                APP.with(|a| {
                    if let Some(a) = a.borrow_mut().as_mut() {
                        a.test_result = Some(wp.0 == 1);
                    }
                });
                PostQuitMessage(0);
                return LRESULT(0);
            }
            TEST_CONFIG => {
                if std::env::args().any(|a| a == "--self-test") {
                    APP.with(|a| {
                        if let Some(a) = a.borrow_mut().as_mut() {
                            let mut p = Preferences::default();
                            match wp.0 {
                                1 => p.theme = Theme::Dark,
                                2 => p.transparency = false,
                                3 => p.excluded_apps.push("accento.exe".into()),
                                _ => p.theme = Theme::Light,
                            }
                            a.draft = p.clone();
                            a.prefs = p.clone();
                            if let Some(worker) = &a.worker {
                                worker.configure(p);
                            }
                        }
                    });
                }
                return LRESULT(0);
            }
            TEST_UI if std::env::args().any(|a| a == "--self-test") => {
                return LRESULT(settings::verify_controls(hwnd) as isize);
            }
            TEST_RENDER if std::env::args().any(|a| a == "--self-test") => {
                let folder = std::env::current_exe()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .to_path_buf();
                let valid = APP.with(|a| {
                    a.borrow()
                        .as_ref()
                        .and_then(|a| a.renderer.as_ref())
                        .is_some_and(|r| r.verify_test_surface(&folder, wp.0))
                });
                return LRESULT(valid as isize);
            }
            WM_DPICHANGED => {
                APP.with(|a| {
                    if let Some(a) = a.borrow_mut().as_mut() {
                        let dpi = (wp.0 & 0xffff) as u32;
                        let font = CreateFontW(
                            -((13. * dpi as f32 / 96. * render::text_scale()) as i32),
                            0,
                            0,
                            0,
                            FW_NORMAL.0 as i32,
                            0,
                            0,
                            0,
                            DEFAULT_CHARSET,
                            OUT_DEFAULT_PRECIS,
                            CLIP_DEFAULT_PRECIS,
                            CLEARTYPE_QUALITY,
                            DEFAULT_PITCH.0 as u32,
                            w!("Segoe UI"),
                        );
                        let title_font = font_for_title(dpi);
                        let section_font = font_for_section(dpi);
                        for (window, [x, y, width, height], role) in a.controls.borrow().iter() {
                            let window = HWND(*window as _);
                            let scale = dpi as f32 / 96.;
                            let _ = SetWindowPos(
                                window,
                                None,
                                (*x as f32 * scale) as i32,
                                (*y as f32 * scale) as i32,
                                (*width as f32 * scale) as i32,
                                (*height as f32 * scale) as i32,
                                SWP_NOZORDER | SWP_NOACTIVATE,
                            );
                            let chosen = match role {
                                1 => title_font,
                                2 => section_font,
                                _ => font,
                            };
                            send_message(window, WM_SETFONT, WPARAM(chosen.0 as usize), LPARAM(1));
                        }
                        let _ = DeleteObject(HGDIOBJ(a.font.0));
                        a.font = font;
                        let _ = DeleteObject(HGDIOBJ(a.title_font.0));
                        let _ = DeleteObject(HGDIOBJ(a.section_font.0));
                        a.title_font = title_font;
                        a.section_font = section_font;
                    }
                });
                let bounds = &*(lp.0 as *const RECT);
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    bounds.left,
                    bounds.top,
                    bounds.right - bounds.left,
                    bounds.bottom - bounds.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            _ => {}
        }
        DefWindowProcW(hwnd, msg, wp, lp)
    }
}
const UI_EVENT_ALIAS: u32 = input::UI_EVENT;

unsafe fn drain_events() {
    unsafe {
        let mut errors = Vec::new();
        // Drain to the latest state before touching the native window; avoids stale flashes and reentrancy.
        let changed = APP.with(|a| {
            let mut a = a.borrow_mut();
            let Some(a) = a.as_mut() else { return false };
            let mut changed = false;
            while let Ok(event) = a.events.try_recv() {
                match event {
                    UiEvent::Effect(Effect::Show(view)) => {
                        a.view = Some(view);
                        changed = true;
                    }
                    UiEvent::Effect(Effect::Hide) => {
                        a.view = None;
                        changed = true;
                    }
                    UiEvent::Error(error) => errors.push(error),
                    _ => {}
                }
            }
            changed
        });
        if changed {
            update_picker();
        }
        for error in errors {
            let main = APP.with(|a| a.borrow().as_ref().unwrap().main);
            alert(main, &error);
        }
    }
}

unsafe fn update_picker() {
    unsafe {
        // Window messages may be dispatched synchronously by SetWindowPos / ShowWindow.
        let state = APP.with(|a| {
            let a = a.borrow();
            let a = a.as_ref().unwrap();
            (a.overlay, a.view.clone(), a.prefs.clone())
        });
        let (hwnd, view, prefs) = state;
        let Some(view) = view else {
            APP.with(|a| {
                if let Some(a) = a.borrow_mut().as_mut() {
                    a.renderer = None;
                }
            });
            let _ = ShowWindow(hwnd, SW_HIDE);
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_NOTOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
            return;
        };
        let choices = view
            .candidates
            .iter()
            .map(char::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        let accessible = match view.selected {
            Some(index) => format!(
                "Accento. Varianti: {choices}. Selezionato {}. Rilascia la lettera per inserire.",
                view.candidates[index]
            ),
            None => format!("Accento. Varianti: {choices}. Usa le frecce per scegliere."),
        };
        let title = wide(&accessible);
        let _ = SetWindowTextW(hwnd, PCWSTR(title.as_ptr()));
        NotifyWinEvent(
            EVENT_OBJECT_NAMECHANGE,
            hwnd,
            OBJID_WINDOW.0,
            CHILDID_SELF as i32,
        );
        if view.selected.is_some()
            && UiaClientsAreListening().as_bool()
            && let Ok(provider) = UiaHostProviderFromHwnd(hwnd)
        {
            let _ = UiaRaiseNotificationEvent(
                &provider,
                NotificationKind_ActionCompleted,
                NotificationProcessing_MostRecent,
                &windows::core::BSTR::from(accessible),
                &windows::core::BSTR::from("Accento.Selection"),
            );
        }
        let monitor = MonitorFromWindow(HWND(view.context.window as _), MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            let _ = ShowWindow(hwnd, SW_HIDE);
            return;
        }
        let dpi = GetDpiForWindow(HWND(view.context.window as _)).max(96);
        let scale = dpi as f32 / 96.;
        let factor = render::text_scale();
        let width =
            (crate::picker_material::width(view.candidates.len()) * factor * scale).round() as i32;
        let width = width.min(info.rcWork.right - info.rcWork.left - 32);
        let height = (crate::picker_material::HEIGHT * factor * scale).round() as i32;
        let x = info.rcWork.left + (info.rcWork.right - info.rcWork.left - width) / 2;
        let y = info.rcWork.bottom - height - (32. * scale) as i32;
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            x,
            y,
            width,
            height,
            SWP_NOACTIVATE,
        );
        let material = render::glass_enabled(&prefs);
        let result = APP.with(|a| {
            let mut a = a.borrow_mut();
            let a = a.as_mut().unwrap();
            a.material = material;
            if let Some(renderer) = a.renderer.as_mut() {
                renderer.resize(width as u32, height as u32, dpi as f32)?;
            } else {
                a.renderer = Some(Renderer::new(
                    hwnd,
                    width as u32,
                    height as u32,
                    dpi as f32,
                    material,
                )?);
            }
            a.renderer
                .as_ref()
                .unwrap()
                .picker(&view, &prefs, material, dpi as f32)
        });
        match result {
            Ok(()) => {
                let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
            Err(error) => {
                let _ = ShowWindow(hwnd, SW_HIDE);
                APP.with(|a| {
                    if let Some(a) = a.borrow_mut().as_mut() {
                        a.renderer = None;
                        if let Some(w) = &a.worker {
                            w.pause(true);
                        }
                        a.paused = true;
                        status(a.main, "In pausa · Errore del selettore");
                    }
                });
                eprintln!("Rendering del selettore: {error}");
            }
        }
    }
}
unsafe extern "system" fn picker_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_MOUSEACTIVATE => return LRESULT(MA_NOACTIVATEANDEAT as isize),
            WM_ERASEBKGND => return LRESULT(1),
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let _ = BeginPaint(hwnd, &mut ps);
                APP.with(|a| {
                    if let Ok(a) = a.try_borrow()
                        && let Some(a) = a.as_ref()
                        && let (Some(view), Some(renderer)) = (&a.view, &a.renderer)
                    {
                        let _ = renderer.picker(
                            view,
                            &a.prefs,
                            a.material,
                            GetDpiForWindow(hwnd) as f32,
                        );
                    }
                });
                let _ = EndPaint(hwnd, &ps);
                return LRESULT(0);
            }
            _ => {}
        }
        DefWindowProcW(hwnd, msg, wp, lp)
    }
}
unsafe fn tray_icon(app: &App, action: NOTIFY_ICON_MESSAGE) -> bool {
    unsafe {
        let mut data = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: app.main,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: TRAY,
            hIcon: app.icon,
            ..Default::default()
        };
        let text = wide(if app.paused {
            "Accento · In pausa"
        } else {
            "Accento · Accenti da tastiera"
        });
        data.szTip[..text.len()].copy_from_slice(&text);
        Shell_NotifyIconW(action, &data).as_bool()
    }
}
unsafe fn create_icon() -> windows::core::Result<HICON> {
    unsafe {
        let mut pixels = vec![0u8; 32 * 32 * 4];
        for y in 0..32 {
            for x in 0..32 {
                let i = (y * 32 + x) * 4;
                let rounded = (x as f32 - 15.5).abs().max((y as f32 - 15.5).abs()) < 14.
                    || ((x as f32 - 15.5).abs() - 10.)
                        .max(0.)
                        .hypot(((y as f32 - 15.5).abs() - 10.).max(0.))
                        < 5.;
                if rounded {
                    pixels[i] = 224;
                    pixels[i + 1] = 100;
                    pixels[i + 2] = 42;
                    pixels[i + 3] = 255;
                }
                let fx = x as f32;
                let fy = y as f32;
                let a = (9..=23).contains(&y)
                    && ((fx - (16. - (fy - 9.) * 0.43)).abs() < 1.4
                        || (fx - (16. + (fy - 9.) * 0.43)).abs() < 1.4
                        || (17..=19).contains(&y) && (12..=20).contains(&x));
                let accent = (5..=7).contains(&y) && (18..=22).contains(&x);
                if a || accent {
                    pixels[i] = 255;
                    pixels[i + 1] = 255;
                    pixels[i + 2] = 255;
                    pixels[i + 3] = 255;
                }
            }
        }
        CreateIcon(None, 32, 32, 1, 32, [0u8; 128].as_ptr(), pixels.as_ptr())
    }
}

unsafe extern "system" fn test_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    unsafe {
        if msg == TEST_FOCUS {
            let _ = SetForegroundWindow(hwnd);
            if let Ok(edit) = GetDlgItem(Some(hwnd), wp.0 as i32) {
                let _ = SetFocus(Some(edit));
            }
            return LRESULT(0);
        }
        DefWindowProcW(hwnd, msg, wp, lp)
    }
}
unsafe fn start_test(main: HWND) -> windows::core::Result<()> {
    unsafe {
        let monitor = MonitorFromWindow(main, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(monitor, &mut info);
        let target = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("Accento.Test"),
            w!("Accento · Collaudo input"),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            info.rcWork.left + (info.rcWork.right - info.rcWork.left - 660) / 2,
            info.rcWork.bottom - 310,
            660,
            300,
            None,
            None,
            None,
            None,
        )?;
        let edit = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("EDIT"),
            w!(""),
            WS_CHILD | WS_VISIBLE | WS_BORDER | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
            25,
            35,
            580,
            60,
            Some(target),
            Some(id(1)),
            None,
            None,
        )?;
        let second = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("EDIT"),
            w!(""),
            WS_CHILD | WS_VISIBLE | WS_BORDER,
            25,
            110,
            580,
            40,
            Some(target),
            Some(id(2)),
            None,
            None,
        )?;
        let foreground_thread = GetWindowThreadProcessId(GetForegroundWindow(), None);
        let test_thread = GetCurrentThreadId();
        let attached = foreground_thread != test_thread
            && AttachThreadInput(test_thread, foreground_thread, true).as_bool();
        let _ = ShowWindow(target, SW_RESTORE);
        let _ = SetForegroundWindow(target);
        let _ = SetFocus(Some(edit));
        if attached {
            let _ = AttachThreadInput(test_thread, foreground_thread, false);
        }
        let target = target.0 as usize;
        let edit = edit.0 as usize;
        let main = main.0 as usize;
        let second = second.0 as usize;
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(500));
            let mut results = Vec::new();
            let checks = send_message(HWND(main as _), TEST_UI, WPARAM(0), LPARAM(0)).0 as u32;
            for (i, label) in [
                "Tile lingua attivabile",
                "Ultima lingua conservata",
                "Temi esclusivi e anteprima",
                "Controllo Glass attivabile",
                "Stepper ritardo",
                "Errore di validazione inline",
                "Scorrimento finestra compatta",
                "Comando Salva raggiungibile",
                "Bozza conservata fra sezioni",
                "Errore riporta al campo nella sezione corretta",
                "Focus scorre il campo nel viewport",
                "Tab entra nei controlli della sezione",
                "Visibilità controlli nelle quattro sezioni compatte",
                "Salva disattivato senza modifiche",
            ]
            .iter()
            .enumerate()
            {
                results.push((*label, checks & (1 << i) != 0));
            }
            send_message(HWND(target as _), TEST_FOCUS, WPARAM(1), LPARAM(0));
            std::thread::sleep(Duration::from_millis(200));
            let folder = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(std::env::temp_dir);
            let send = |vk: VIRTUAL_KEY, up: bool| {
                if !up && GetForegroundWindow().0 as usize != target {
                    return;
                }
                let event = INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: vk,
                            dwFlags: if up {
                                KEYEVENTF_KEYUP
                            } else {
                                KEYBD_EVENT_FLAGS(0)
                            },
                            dwExtraInfo: 0xACCE7E57,
                            ..Default::default()
                        },
                    },
                };
                let _ = SendInput(&[event], size_of::<INPUT>() as i32);
                std::thread::sleep(Duration::from_millis(25));
            };
            let read = || {
                let mut text = [0u16; 256];
                let n = GetWindowTextW(HWND(edit as _), &mut text);
                String::from_utf16_lossy(&text[..n as usize])
            };
            let clear = || {
                let _ = SetWindowTextW(HWND(edit as _), w!(""));
                std::thread::sleep(Duration::from_millis(60));
            };
            let gesture = || {
                send(VK_E, false);
                send(VK_SPACE, false);
                send(VK_SPACE, true);
                std::thread::sleep(Duration::from_millis(340));
            };
            let mut focused = GetForegroundWindow().0 as usize == target;
            if focused {
                gesture();
                send(VK_RIGHT, false);
                send(VK_RIGHT, true);
                send(VK_E, true);
                std::thread::sleep(Duration::from_millis(150));
                results.push(("e + selezione → è", read() == "è"));
                results.push((
                    "Focus conservato",
                    GetForegroundWindow().0 as usize == target,
                ));
                clear();
                send(VK_SHIFT, false);
                gesture();
                send(VK_RIGHT, false);
                send(VK_RIGHT, true);
                send(VK_E, true);
                send(VK_SHIFT, true);
                std::thread::sleep(Duration::from_millis(100));
                results.push(("Maiuscola → È", read() == "È"));
                clear();
                gesture();
                send(VK_ESCAPE, false);
                send(VK_ESCAPE, true);
                send(VK_E, true);
                std::thread::sleep(Duration::from_millis(100));
                results.push(("Esc conserva e", read() == "e"));
                clear();
                send(VK_E, false);
                send(VK_SPACE, false);
                send(VK_E, true);
                send(VK_SPACE, true);
                std::thread::sleep(Duration::from_millis(150));
                results.push(("Falsa partenza conserva e spazio", read() == "e "));
                clear();
                gesture();
                send(VK_E, true);
                std::thread::sleep(Duration::from_millis(100));
                results.push(("Nessuna selezione conserva e", read() == "e"));
                clear();
                gesture();
                send(VK_SPACE, false);
                send(VK_SPACE, true);
                send(VK_E, true);
                std::thread::sleep(Duration::from_millis(100));
                results.push(("Seconda pressione Spazio seleziona", read() == "è"));
                clear();
                gesture();
                send(VK_LEFT, false);
                send(VK_LEFT, true);
                send(VK_E, true);
                std::thread::sleep(Duration::from_millis(100));
                results.push(("Sinistra seleziona ultimo candidato", read() == "é"));
                clear();
                gesture();
                send(VK_RIGHT, false);
                send(VK_RIGHT, true);
                send_message(HWND(target as _), TEST_FOCUS, WPARAM(2), LPARAM(0));
                std::thread::sleep(Duration::from_millis(60));
                send(VK_E, true);
                std::thread::sleep(Duration::from_millis(100));
                results.push((
                    "Cambio campo annulla senza invio",
                    read() == "e" && GetWindowTextLengthW(HWND(second as _)) == 0,
                ));
                send_message(HWND(target as _), TEST_FOCUS, WPARAM(1), LPARAM(0));
                clear();
                send_message(
                    HWND(main as _),
                    WM_COMMAND,
                    WPARAM(PAUSE as usize),
                    LPARAM(0),
                );
                std::thread::sleep(Duration::from_millis(100));
                send(VK_E, false);
                send(VK_SPACE, false);
                send(VK_E, true);
                send(VK_SPACE, true);
                std::thread::sleep(Duration::from_millis(100));
                results.push(("Pausa lascia passare la digitazione", read() == "e "));
                send_message(
                    HWND(main as _),
                    WM_COMMAND,
                    WPARAM(PAUSE as usize),
                    LPARAM(0),
                );
                send_message(HWND(main as _), TEST_CONFIG, WPARAM(3), LPARAM(0));
                std::thread::sleep(Duration::from_millis(100));
                clear();
                send(VK_E, false);
                send(VK_SPACE, false);
                send(VK_E, true);
                send(VK_SPACE, true);
                std::thread::sleep(Duration::from_millis(100));
                results.push(("Esclusione esatta del processo", read() == "e "));
                for (mode, label) in [
                    (0, "Glass chiaro"),
                    (1, "Glass scuro"),
                    (2, "Modalità opaca"),
                ] {
                    send_message(HWND(main as _), TEST_CONFIG, WPARAM(mode), LPARAM(0));
                    std::thread::sleep(Duration::from_millis(100));
                    clear();
                    gesture();
                    send(VK_RIGHT, false);
                    send(VK_RIGHT, true);
                    std::thread::sleep(Duration::from_millis(80));
                    let visible = FindWindowW(w!("Accento.Picker"), PCWSTR::null())
                        .ok()
                        .filter(|w| IsWindowVisible(*w).as_bool());
                    results.push((label, visible.is_some()));
                    results.push((
                        "Alpha del popup e spazi trasparenti",
                        send_message(HWND(main as _), TEST_RENDER, WPARAM(mode), LPARAM(0)).0 == 1,
                    ));
                    if let Some(window) = visible {
                        let _ = capture_window(
                            window,
                            &folder.join(format!("accento-picker-{mode}.bmp")),
                        );
                    }
                    send(VK_E, true);
                    std::thread::sleep(Duration::from_millis(100));
                    results.push(("Inserimento dopo cambio aspetto", read() == "è"));
                }
                focused = GetForegroundWindow().0 as usize == target;
            }
            results.push(("Finestra di collaudo disponibile", focused));
            let passed = results.iter().all(|(_, passed)| *passed);
            let report = results
                .iter()
                .map(|(name, passed)| format!("{} {name}", if *passed { "PASS" } else { "FAIL" }))
                .collect::<Vec<_>>()
                .join("\n");
            let _ = std::fs::write(folder.join("accento-self-test.txt"), &report);
            println!("{report}");
            let _ = PostMessageW(Some(HWND(target as _)), WM_CLOSE, WPARAM(0), LPARAM(0));
            let _ = PostMessageW(
                Some(HWND(main as _)),
                TEST_DONE,
                WPARAM(passed as usize),
                LPARAM(0),
            );
        });
        Ok(())
    }
}

/// Captures only the owned picker window for the opt-in integration test.
unsafe fn capture_window(hwnd: HWND, path: &std::path::Path) -> Result<(), String> {
    unsafe {
        let mut bounds = RECT::default();
        GetWindowRect(hwnd, &mut bounds).map_err(|e| e.to_string())?;
        let width = bounds.right - bounds.left;
        let height = bounds.bottom - bounds.top;
        let source = GetWindowDC(Some(hwnd));
        let dc = CreateCompatibleDC(Some(source));
        let bitmap = CreateCompatibleBitmap(source, width, height);
        let previous = SelectObject(dc, HGDIOBJ(bitmap.0));
        let mut class = [0u16; 64];
        let length = GetClassNameW(hwnd, &mut class);
        let settings = String::from_utf16_lossy(&class[..length as usize]) == "Accento.Settings";
        let copied = if settings {
            if PrintWindow(hwnd, dc, PRINT_WINDOW_FLAGS(2)).as_bool() {
                Ok(())
            } else {
                Err(windows::core::Error::from_thread())
            }
        } else {
            let desktop = GetDC(None);
            let result = BitBlt(
                dc,
                0,
                0,
                width,
                height,
                Some(desktop),
                bounds.left,
                bounds.top,
                SRCCOPY | CAPTUREBLT,
            );
            ReleaseDC(None, desktop);
            result
        };
        SelectObject(dc, previous);
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        let rows = GetDIBits(
            dc,
            bitmap,
            0,
            height as u32,
            Some(pixels.as_mut_ptr().cast()),
            &mut info,
            DIB_RGB_COLORS,
        );
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        let _ = DeleteDC(dc);
        ReleaseDC(Some(hwnd), source);
        copied.map_err(|e| e.to_string())?;
        if rows == 0 {
            return Err("Cattura del selettore non disponibile".into());
        }
        let mut bmp = Vec::new();
        bmp.extend_from_slice(b"BM");
        bmp.extend_from_slice(&(54u32 + pixels.len() as u32).to_le_bytes());
        bmp.extend_from_slice(&[0u8; 4]);
        bmp.extend_from_slice(&54u32.to_le_bytes());
        bmp.extend_from_slice(&40u32.to_le_bytes());
        bmp.extend_from_slice(&width.to_le_bytes());
        bmp.extend_from_slice(&(-height).to_le_bytes());
        bmp.extend_from_slice(&1u16.to_le_bytes());
        bmp.extend_from_slice(&32u16.to_le_bytes());
        bmp.extend_from_slice(&[0u8; 24]);
        bmp.extend(pixels);
        std::fs::write(path, bmp).map_err(|e| e.to_string())
    }
}
