use super::*;
use crate::{
    glass::{self, Painter, Palette},
    i18n::{CODES, NAMES, Text, tr},
    preferences::normalize_executable,
    render::{rect, rgba},
};
use windows::Win32::Graphics::{
    Direct2D::{Common::*, *},
    DirectWrite::*,
};

pub const WIDTH: f32 = 1000.;
pub const HEIGHT: f32 = 760.;
pub const THEME_FIRST: u16 = 230;
const MINUS: u16 = 240;
const PLUS: u16 = 241;
const NAV_FIRST: u16 = 250;
pub const CONTENT: u16 = 260;
const ACTIVE_FIRST: u16 = 270;
const UI_FIRST: u16 = 280;
const ADD: u16 = 290;
const DETAIL: u16 = 291;
const REMOVE_FIRST: u16 = 1000;
const FOCUS_SCROLL: u32 = WM_APP + 25;
const NAV: [Text; 3] = [Text::General, Text::Sets, Text::Blacklist];
thread_local! {
    static REPAINTS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    static POSITIONS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    static SCROLL_DRAG: std::cell::Cell<Option<f32>> = const {std::cell::Cell::new(None)};
    static LAYOUT_BUSY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    General,
    CharacterSets,
    Blacklist,
}
impl Section {
    fn index(self) -> usize {
        match self {
            Self::General => 0,
            Self::CharacterSets => 1,
            Self::Blacklist => 2,
        }
    }
    fn from_index(i: usize) -> Self {
        [Self::General, Self::CharacterSets, Self::Blacklist][i]
    }
    fn content_height(self, count: usize) -> f32 {
        match self {
            Self::General => 528.,
            Self::CharacterSets => 428.,
            Self::Blacklist => 168. + count.max(2) as f32 * 60.,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Issue {
    field: u16,
    message: String,
}
impl Issue {
    pub fn picker_error(language: &str) -> Self {
        Self {
            field: 0,
            message: tr(language, Text::PickerError).into(),
        }
    }
}
#[derive(Clone)]
struct State {
    prefs: Preferences,
    section: Section,
    hover: u16,
    dirty: bool,
    error: Option<Issue>,
    focus: u16,
    paused: bool,
    scroll: f32,
}
impl State {
    fn t(&self, key: Text) -> &'static str {
        tr(&self.prefs.ui_language, key)
    }
}
fn state() -> Option<State> {
    APP.with(|a| {
        a.try_borrow().ok().and_then(|a| {
            a.as_ref().map(|a| State {
                prefs: a.draft.clone(),
                section: a.section,
                hover: a.hover,
                dirty: a.dirty,
                error: a.validation_error.clone(),
                focus: unsafe { GetDlgCtrlID(GetFocus()) as u16 },
                paused: a.paused,
                scroll: a.scroll,
            })
        })
    })
}
#[derive(Clone, Copy, Debug)]
struct Layout {
    width: f32,
    height: f32,
    compact: bool,
    left: f32,
    content: f32,
    body: f32,
    footer: f32,
}
impl Layout {
    fn new(width: f32, height: f32) -> Self {
        let compact = width < 900.;
        let left = if compact { 32. } else { 248. };
        Self {
            width,
            height,
            compact,
            left,
            content: (width - left - 32.).max(260.),
            body: if compact { 160. } else { 132. },
            footer: height - 76.,
        }
    }
    fn scroll_max(self, section: Section, count: usize) -> f32 {
        (self.body + section.content_height(count) + 12. - self.footer).max(0.)
    }
    fn bounds(self, id: u16) -> Option<([f32; 4], bool)> {
        let l = self.left;
        let w = self.content;
        let y = self.body;
        if (NAV_FIRST..NAV_FIRST + 3).contains(&id) {
            let i = (id - NAV_FIRST) as f32;
            return Some((
                if self.compact {
                    let tw = (self.width - 184.) / 3.;
                    [24. + i * tw, 16., tw - 4., 40.]
                } else {
                    [16., 132. + i * 52., 184., 44.]
                },
                false,
            ));
        }
        let r = match id {
            SAVE => return Some(([self.width - 180., self.footer + 18., 148., 40.], false)),
            PAUSE => {
                return Some((
                    if self.compact {
                        [self.width - 148., 16., 124., 40.]
                    } else {
                        [16., self.height - 78., 184., 48.]
                    },
                    false,
                ));
            }
            DELAY => [l + w - 128., y + 27., 60., 24.],
            MINUS => [l + w - 176., y + 18., 40., 42.],
            PLUS => [l + w - 60., y + 18., 40., 42.],
            ACTIVE_FIRST..=273 => [
                l + (id - ACTIVE_FIRST) as f32 * (w + 8.) / 4.,
                y + 132.,
                (w - 24.) / 4.,
                42.,
            ],
            DETAIL => [l, y + 181., w, 28.],
            THEME_FIRST..=232 => [
                l + (id - THEME_FIRST) as f32 * (w + 8.) / 3.,
                y + 254.,
                (w - 16.) / 3.,
                44.,
            ],
            UI_FIRST..=283 => [
                l + (id - UI_FIRST) as f32 * (w + 8.) / 4.,
                y + 354.,
                (w - 24.) / 4.,
                44.,
            ],
            LANG_FIRST..=213 => [l, y + 12. + (id - LANG_FIRST) as f32 * 82., w, 72.],
            EXCLUDED => [l + 14., y + 78., w - 156., 24.],
            ADD => [l + w - 128., y + 66., 128., 44.],
            REMOVE_FIRST..=65534 => [
                l + w - 48.,
                y + 170. + (id - REMOVE_FIRST) as f32 * 60.,
                40.,
                40.,
            ],
            _ => return None,
        };
        Some((r, true))
    }
}
fn belongs(id: u16, section: Section, count: usize) -> bool {
    match id {
        DELAY | MINUS | PLUS | DETAIL | ACTIVE_FIRST..=273 | THEME_FIRST..=232 | UI_FIRST..=283 => {
            section == Section::General
        }
        LANG_FIRST..=213 => section == Section::CharacterSets,
        EXCLUDED | ADD => section == Section::Blacklist,
        REMOVE_FIRST..=65534 => {
            section == Section::Blacklist && ((id - REMOVE_FIRST) as usize) < count
        }
        _ => true,
    }
}
pub fn scale(hwnd: HWND) -> f32 {
    unsafe { GetDpiForWindow(hwnd) as f32 / 96. * render::text_scale() }
}
unsafe fn viewport(hwnd: HWND) -> Layout {
    unsafe {
        let mut r = RECT::default();
        let _ = GetClientRect(hwnd, &mut r);
        let s = scale(hwnd);
        Layout::new(r.right as f32 / s, r.bottom as f32 / s)
    }
}
unsafe fn text_if_changed(hwnd: HWND, value: &str) {
    unsafe {
        let mut buffer = vec![0u16; GetWindowTextLengthW(hwnd).max(0) as usize + 1];
        let len = GetWindowTextW(hwnd, &mut buffer);
        if String::from_utf16_lossy(&buffer[..len as usize]) != value {
            let _ = SetWindowTextW(hwnd, PCWSTR(wide(value).as_ptr()));
        }
    }
}
pub unsafe fn build(app: &mut App) {
    unsafe {
        let s = GetDpiForWindow(app.main) as f32 / 96.;
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(
            MonitorFromWindow(app.main, MONITOR_DEFAULTTONEAREST),
            &mut info,
        );
        let mut r = RECT {
            left: 0,
            top: 0,
            right: (WIDTH * s) as i32,
            bottom: (HEIGHT * s) as i32,
        };
        let _ = AdjustWindowRectExForDpi(
            &mut r,
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
            false,
            WINDOW_EX_STYLE(0),
            GetDpiForWindow(app.main),
        );
        let _ = SetWindowPos(
            app.main,
            None,
            0,
            0,
            (r.right - r.left).min(info.rcWork.right - info.rcWork.left - 48),
            (r.bottom - r.top).min(info.rcWork.bottom - info.rcWork.top - 48),
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
        for (i, key) in NAV.iter().enumerate() {
            tile(app, NAV_FIRST + i as u16, tr(&app.draft.ui_language, *key));
        }
        tile(app, PAUSE, tr(&app.draft.ui_language, Text::TurnOff));
        let _ = CreateWindowExW(
            WS_EX_CONTROLPARENT,
            w!("Accento.Content"),
            w!(""),
            WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN,
            0,
            0,
            10,
            10,
            Some(app.main),
            Some(super::id(CONTENT)),
            None,
            None,
        );
        tile(app, MINUS, tr(&app.draft.ui_language, Text::Less));
        field(
            app,
            DELAY,
            &app.draft.delay_ms.to_string(),
            ES_NUMBER as u32,
        );
        tile(app, PLUS, tr(&app.draft.ui_language, Text::More));
        for (i, name) in NAMES.iter().enumerate() {
            tile(app, ACTIVE_FIRST + i as u16, name);
        }
        tile(app, DETAIL, tr(&app.draft.ui_language, Text::SetDetail));
        for (i, key) in [Text::System, Text::Light, Text::Dark].iter().enumerate() {
            tile(
                app,
                THEME_FIRST + i as u16,
                tr(&app.draft.ui_language, *key),
            );
        }
        for (i, name) in NAMES.iter().enumerate() {
            tile(app, UI_FIRST + i as u16, name);
        }
        for (i, name) in NAMES.iter().enumerate() {
            tile(app, LANG_FIRST + i as u16, name);
        }
        field(app, EXCLUDED, "", ES_AUTOHSCROLL as u32);
        tile(app, ADD, tr(&app.draft.ui_language, Text::Add));
        tile(app, SAVE, tr(&app.draft.ui_language, Text::Save));
        send_message(
            control(app.main, DELAY),
            EM_SETLIMITTEXT,
            WPARAM(4),
            LPARAM(0),
        );
        send_message(
            control(app.main, EXCLUDED),
            EM_SETLIMITTEXT,
            WPARAM(260),
            LPARAM(0),
        );
        sync_rows(app);
        sync_names(app);
        render::settings_frame(app.main, &app.draft);
    }
}
unsafe fn tile(app: &App, id: u16, name: &str) {
    unsafe {
        let w = child(
            app,
            w!("BUTTON"),
            name,
            WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW as u32),
            id,
            0,
            0,
            20,
            20,
        );
        if !matches!(id, NAV_FIRST..=252 | SAVE | PAUSE) {
            let _ = SetParent(w, Some(control(app.main, CONTENT)));
        }
        let _ = SetWindowSubclass(w, Some(control_proc), 1, 0);
    }
}
unsafe fn field(app: &App, id: u16, value: &str, style: u32) {
    unsafe {
        let w = child(
            app,
            w!("EDIT"),
            value,
            WS_TABSTOP | WINDOW_STYLE(style),
            id,
            0,
            0,
            20,
            20,
        );
        let _ = SetParent(w, Some(control(app.main, CONTENT)));
        send_message(
            w,
            EM_SETMARGINS,
            WPARAM((EC_LEFTMARGIN | EC_RIGHTMARGIN) as usize),
            LPARAM(0),
        );
        let _ = SetWindowSubclass(w, Some(control_proc), 1, 0);
    }
}
unsafe fn sync_rows(app: &App) {
    unsafe {
        for (i, name) in app.draft.excluded_apps.iter().enumerate() {
            let id = REMOVE_FIRST + i as u16;
            if control(app.main, id).is_invalid() {
                tile(app, id, "");
            }
            text_if_changed(
                control(app.main, id),
                &format!("{} {name}", tr(&app.draft.ui_language, Text::Remove)),
            );
        }
    }
}
unsafe fn sync_names(app: &App) {
    unsafe {
        let t = |key| tr(&app.draft.ui_language, key);
        text_if_changed(app.main, &format!("Accento · {}", t(Text::Settings)));
        for (i, key) in NAV.iter().enumerate() {
            text_if_changed(
                control(app.main, NAV_FIRST + i as u16),
                &format!(
                    "{}, {}",
                    t(*key),
                    t(if app.section.index() == i {
                        Text::Selected
                    } else {
                        Text::NotSelected
                    })
                ),
            );
        }
        for (i, name) in NAMES.iter().enumerate() {
            for first in [LANG_FIRST, ACTIVE_FIRST, UI_FIRST] {
                let on = if first == UI_FIRST {
                    app.draft.ui_language == CODES[i]
                } else {
                    app.draft.languages.iter().any(|l| l == CODES[i])
                };
                text_if_changed(
                    control(app.main, first + i as u16),
                    &format!(
                        "{name}, {}",
                        t(if on {
                            Text::Selected
                        } else {
                            Text::NotSelected
                        })
                    ),
                );
            }
        }
        for (i, key) in [Text::System, Text::Light, Text::Dark].iter().enumerate() {
            let on = theme_index(&app.draft.theme) == i;
            text_if_changed(
                control(app.main, THEME_FIRST + i as u16),
                &format!(
                    "{}, {}",
                    t(*key),
                    t(if on {
                        Text::Selected
                    } else {
                        Text::NotSelected
                    })
                ),
            );
        }
        for (id, key) in [
            (MINUS, Text::Less),
            (PLUS, Text::More),
            (DETAIL, Text::SetDetail),
            (SAVE, Text::Save),
            (ADD, Text::Add),
            (
                PAUSE,
                if app.paused {
                    Text::TurnOn
                } else {
                    Text::TurnOff
                },
            ),
        ] {
            text_if_changed(control(app.main, id), t(key));
        }
        send_message(
            control(app.main, EXCLUDED),
            EM_SETCUEBANNER,
            WPARAM(0),
            LPARAM(wide(t(Text::AppPlaceholder)).as_ptr() as isize),
        );
        sync_rows(app);
    }
}
fn theme_index(theme: &Theme) -> usize {
    match theme {
        Theme::System => 0,
        Theme::Light => 1,
        Theme::Dark => 2,
    }
}
unsafe fn position(window: HWND, parent: HWND, bounds: [i32; 4]) {
    unsafe {
        let [x, y, w, h] = bounds;
        let mut r = RECT::default();
        let _ = GetWindowRect(window, &mut r);
        let mut origin = POINT {
            x: r.left,
            y: r.top,
        };
        let _ = ScreenToClient(parent, &mut origin);
        if (origin.x, origin.y, r.right - r.left, r.bottom - r.top) != (x, y, w, h) {
            let _ = SetWindowPos(
                window,
                None,
                x,
                y,
                w,
                h,
                SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOCOPYBITS,
            );
        }
    }
}
pub unsafe fn layout(hwnd: HWND) {
    unsafe {
        if LAYOUT_BUSY.with(|b| b.replace(true)) {
            return;
        }
        layout_inner(hwnd);
        LAYOUT_BUSY.with(|b| b.set(false));
    }
}
unsafe fn layout_inner(hwnd: HWND) {
    unsafe {
        let Some(st) = state() else {
            return;
        };
        let v = viewport(hwnd);
        let s = scale(hwnd);
        let count = st.prefs.excluded_apps.len();
        let scroll = st.scroll.clamp(0., v.scroll_max(st.section, count));
        APP.with(|a| {
            if let Ok(mut a) = a.try_borrow_mut()
                && let Some(a) = a.as_mut()
            {
                a.scroll = scroll;
            }
        });
        let content = control(hwnd, CONTENT);
        position(
            content,
            hwnd,
            [
                ((v.left - 4.) * s).round() as i32,
                (v.body * s).round() as i32,
                ((v.content + 8.) * s).round() as i32,
                ((v.footer - v.body - 8.).max(0.) * s).round() as i32,
            ],
        );
        let controls = APP.with(|a| a.borrow().as_ref().unwrap().controls.borrow().clone());
        for (window, _, _) in controls {
            let window = HWND(window as _);
            let id = GetDlgCtrlID(window) as u16;
            let Some(([x, y, w, h], body)) = v.bounds(id) else {
                continue;
            };
            let show = belongs(id, st.section, count);
            if IsWindowVisible(window).as_bool() != show {
                let _ = ShowWindow(window, if show { SW_SHOWNA } else { SW_HIDE });
            }
            if !show {
                continue;
            }
            let px = ((x - if body { v.left - 4. } else { 0. }) * s).round() as i32;
            let py = ((y - if body { v.body + scroll } else { 0. }) * s).round() as i32;
            position(
                window,
                if body { content } else { hwnd },
                [px, py, (w * s).round() as i32, (h * s).round() as i32],
            );
        }
        refresh(hwnd, true);
    }
}
unsafe fn invalidate(hwnd: HWND) {
    unsafe {
        let _ = InvalidateRect(Some(hwnd), None, false);
    }
}
unsafe fn refresh(hwnd: HWND, all: bool) {
    unsafe {
        let dirty = APP.with(|a| {
            let mut a = a.borrow_mut();
            let a = a.as_mut().unwrap();
            a.dirty = !draft_from_controls(hwnd, a).is_ok_and(|p| p == a.prefs);
            sync_names(a);
            a.dirty
        });
        if IsWindowEnabled(control(hwnd, SAVE)).as_bool() != dirty {
            let _ = EnableWindow(control(hwnd, SAVE), dirty);
        }
        if all {
            let _ = RedrawWindow(Some(hwnd), None, None, RDW_INVALIDATE | RDW_ALLCHILDREN);
        } else {
            let v = viewport(hwnd);
            let s = scale(hwnd);
            let footer = RECT {
                left: (v.left * s) as i32,
                top: (v.footer * s) as i32,
                right: (v.width * s) as i32,
                bottom: (v.height * s) as i32,
            };
            let _ = InvalidateRect(Some(hwnd), Some(&footer), false);
            invalidate(control(hwnd, SAVE));
        }
    }
}
unsafe fn draft_from_controls(hwnd: HWND, app: &App) -> Result<Preferences, Issue> {
    unsafe {
        let mut p = app.draft.clone();
        p.delay_ms = control_text(hwnd, DELAY)
            .trim()
            .parse()
            .map_err(|_| Issue {
                field: DELAY,
                message: tr(&p.ui_language, Text::DelayError).into(),
            })?;
        p.validate().map_err(|message| Issue {
            field: if !(100..=1500).contains(&p.delay_ms) {
                DELAY
            } else {
                0
            },
            message,
        })?;
        Ok(p)
    }
}
unsafe fn save(hwnd: HWND) {
    unsafe {
        let result = APP.with(|a| {
            let mut a = a.borrow_mut();
            let a = a.as_mut().unwrap();
            let p = draft_from_controls(hwnd, a)?;
            p.save().map_err(|message| Issue { field: 0, message })?;
            if let Some(worker) = &a.worker {
                worker.configure(p.clone());
            }
            a.prefs = p.clone();
            a.draft = p;
            a.validation_error = None;
            if a.tray {
                tray_icon(a, NIM_MODIFY);
            }
            Ok::<_, Issue>(())
        });
        if let Err(issue) = result {
            let field = issue.field;
            APP.with(|a| {
                let mut a = a.borrow_mut();
                let a = a.as_mut().unwrap();
                if field == DELAY {
                    a.section = Section::General;
                }
                a.validation_error = Some(issue);
                a.scroll = 0.;
            });
            layout(hwnd);
            if field != 0 {
                let _ = SetFocus(Some(control(hwnd, field)));
            }
        }
        refresh(hwnd, true);
    }
}
unsafe fn navigate(hwnd: HWND, section: Section) {
    unsafe {
        let changed = APP.with(|a| {
            let mut a = a.borrow_mut();
            let a = a.as_mut().unwrap();
            if a.section == section {
                return false;
            }
            a.section = section;
            a.scroll = 0.;
            a.validation_error = None;
            true
        });
        if changed {
            let _ = SetFocus(Some(control(hwnd, NAV_FIRST + section.index() as u16)));
            layout(hwnd);
        }
    }
}
pub unsafe fn command(hwnd: HWND, id: u16, notification: u16) -> bool {
    unsafe {
        if id == DELAY && notification == EN_CHANGE as u16 {
            let ready = APP.with(|a| {
                if let Ok(mut a) = a.try_borrow_mut()
                    && let Some(a) = a.as_mut()
                {
                    a.validation_error = None;
                    true
                } else {
                    false
                }
            });
            if ready {
                refresh(hwnd, false);
            }
            return true;
        }
        if id == EXCLUDED && notification == EN_CHANGE as u16 {
            let ready = APP.with(|a| {
                if let Ok(mut a) = a.try_borrow_mut()
                    && let Some(a) = a.as_mut()
                {
                    if a.validation_error
                        .as_ref()
                        .is_some_and(|e| e.field == EXCLUDED)
                    {
                        a.validation_error = None;
                    }
                    true
                } else {
                    false
                }
            });
            if ready {
                refresh(hwnd, false);
            }
            return true;
        }
        if notification != BN_CLICKED as u16 {
            return false;
        }
        if id == SAVE {
            save(hwnd);
            return true;
        }
        if (NAV_FIRST..NAV_FIRST + 3).contains(&id) {
            navigate(hwnd, Section::from_index((id - NAV_FIRST) as usize));
            return true;
        }
        if id == DETAIL {
            navigate(hwnd, Section::CharacterSets);
            return true;
        }
        if id == PAUSE {
            APP.with(|a| {
                let mut a = a.borrow_mut();
                let a = a.as_mut().unwrap();
                a.paused = !a.paused;
                if let Some(w) = &a.worker {
                    w.pause(a.paused);
                }
                sync_names(a);
                if a.tray {
                    tray_icon(a, NIM_MODIFY);
                }
            });
            invalidate(control(hwnd, PAUSE));
            return true;
        }
        if id == ADD {
            let input = control_text(hwnd, EXCLUDED);
            let added = APP.with(|a| {
                let mut a = a.borrow_mut();
                let a = a.as_mut().unwrap();
                let entry = normalize_executable(&input);
                let error = if entry.is_none() {
                    Some(Text::AppError)
                } else if a
                    .draft
                    .excluded_apps
                    .iter()
                    .any(|s| s.eq_ignore_ascii_case(entry.as_ref().unwrap()))
                {
                    Some(Text::Duplicate)
                } else {
                    None
                };
                if let Some(key) = error {
                    a.validation_error = Some(Issue {
                        field: EXCLUDED,
                        message: tr(&a.draft.ui_language, key).into(),
                    });
                    false
                } else {
                    a.draft.excluded_apps.push(entry.unwrap());
                    a.validation_error = None;
                    sync_rows(a);
                    true
                }
            });
            if added {
                text_if_changed(control(hwnd, EXCLUDED), "");
                layout(hwnd);
            } else {
                refresh(hwnd, true);
            }
            let _ = SetFocus(Some(control(hwnd, EXCLUDED)));
            return true;
        }
        if id >= REMOVE_FIRST {
            let removed = APP.with(|a| {
                let mut a = a.borrow_mut();
                let a = a.as_mut().unwrap();
                let i = (id - REMOVE_FIRST) as usize;
                if i < a.draft.excluded_apps.len() {
                    a.draft.excluded_apps.remove(i);
                    a.validation_error = None;
                    sync_rows(a);
                    true
                } else {
                    false
                }
            });
            if removed {
                let _ = SetFocus(Some(control(hwnd, EXCLUDED)));
                layout(hwnd);
            }
            return true;
        }
        if !matches!(id,LANG_FIRST..=213|ACTIVE_FIRST..=273|THEME_FIRST..=232|UI_FIRST..=283|MINUS|PLUS)
        {
            return false;
        }
        let old_dark = state().map(|s| render::dark(&s.prefs));
        APP.with(|a| {
            let mut a = a.borrow_mut();
            let a = a.as_mut().unwrap();
            a.validation_error = None;
            match id {
                LANG_FIRST..=213 | ACTIVE_FIRST..=273 => {
                    let i = if id >= ACTIVE_FIRST {
                        id - ACTIVE_FIRST
                    } else {
                        id - LANG_FIRST
                    } as usize;
                    let code = CODES[i];
                    if let Some(i) = a.draft.languages.iter().position(|l| l == code) {
                        if a.draft.languages.len() > 1 {
                            a.draft.languages.remove(i);
                        }
                    } else {
                        a.draft.languages.push(code.into());
                    }
                }
                THEME_FIRST..=232 => {
                    a.draft.theme = [Theme::System, Theme::Light, Theme::Dark]
                        [(id - THEME_FIRST) as usize]
                        .clone()
                }
                UI_FIRST..=283 => a.draft.ui_language = CODES[(id - UI_FIRST) as usize].into(),
                MINUS | PLUS => {
                    let current = control_text(hwnd, DELAY).parse::<u64>().unwrap_or(300);
                    let value = if id == MINUS {
                        current.saturating_sub(50)
                    } else {
                        current.saturating_add(50)
                    }
                    .clamp(100, 1500);
                    text_if_changed(control(hwnd, DELAY), &value.to_string());
                    a.draft.delay_ms = value;
                }
                _ => {}
            }
        });
        let theme_changed = old_dark != state().map(|s| render::dark(&s.prefs));
        if theme_changed && let Some(st) = state() {
            render::settings_frame(hwnd, &st.prefs);
        }
        let all = theme_changed || (UI_FIRST..=283).contains(&id);
        refresh(hwnd, all);
        if !all {
            for first in [LANG_FIRST, ACTIVE_FIRST, THEME_FIRST] {
                for i in 0..if first == THEME_FIRST { 3 } else { 4 } {
                    invalidate(control(hwnd, first + i));
                }
            }
        }
        true
    }
}

fn label(
    g: &Painter,
    text: &str,
    area: D2D_RECT_F,
    size: f32,
    color: D2D1_COLOR_F,
    bold: bool,
) -> windows::core::Result<()> {
    g.text(
        text,
        area,
        size,
        if bold {
            DWRITE_FONT_WEIGHT_SEMI_BOLD
        } else {
            DWRITE_FONT_WEIGHT_NORMAL
        },
        color,
        false,
    )
}
fn panel(g: &Painter, area: D2D_RECT_F, p: Palette) -> windows::core::Result<()> {
    g.fill(area, 16., p.surface)?;
    g.stroke(area, 16., p.separator, 1.)
}

fn palette(prefs: &Preferences) -> Palette {
    let mut p = Palette::new(render::dark(prefs));
    if render::high_contrast() {
        p.canvas = render::system_color(COLOR_WINDOW);
        p.surface = p.canvas;
        p.inset = p.canvas;
        p.ink = render::system_color(COLOR_WINDOWTEXT);
        p.secondary = p.ink;
        p.tertiary = p.ink;
        p.accent = render::system_color(COLOR_HIGHLIGHT);
        p.separator = p.ink;
        p.error = p.ink;
        p.selection = rgba(p.accent.r, p.accent.g, p.accent.b, 0.3);
    }
    p
}

fn nav_icon(
    g: &Painter,
    i: usize,
    x: f32,
    y: f32,
    color: D2D1_COLOR_F,
) -> windows::core::Result<()> {
    match i {
        0 => {
            g.line((x + 3., y + 5.), (x + 17., y + 5.), color, 1.5)?;
            g.line((x + 10., y + 5.), (x + 10., y + 18.), color, 1.5)?;
            g.line((x + 6., y + 18.), (x + 14., y + 18.), color, 1.5)?;
        }
        1 => {
            g.stroke(rect(x + 1., y + 1., 18., 18.), 9., color, 1.5)?;
            g.stroke(rect(x + 6., y + 1., 8., 18.), 4., color, 1.)?;
            g.line((x + 1., y + 10.), (x + 19., y + 10.), color, 1.)?;
        }
        _ => {
            g.stroke(rect(x + 1., y + 2., 18., 16.), 1., color, 1.5)?;
            g.line((x + 1., y + 7.), (x + 19., y + 7.), color, 1.5)?;
        }
    }
    Ok(())
}

pub unsafe fn paint(hwnd: HWND, dc: HDC) {
    unsafe {
        let Some(st) = state() else {
            return;
        };
        let mut r = RECT::default();
        let _ = GetClientRect(hwnd, &mut r);
        let _ = glass::draw_dc(dc, r, 96. * scale(hwnd), (0., 0.), |g| {
            canvas(g, &st, viewport(hwnd))
        });
    }
}
fn canvas(g: &Painter, st: &State, v: Layout) -> windows::core::Result<()> {
    let p = palette(&st.prefs);
    unsafe {
        g.target.Clear(Some(&p.canvas));
    }
    let l = v.left;
    let w = v.content;
    let y = v.body - st.scroll;
    if !v.compact {
        g.fill(rect(0., 0., 216., v.height), 0., sidebar(p))?;
        g.gradient(
            rect(24., 32., 32., 32.),
            8.5,
            &[
                (0., rgba(0.66, 0.75, 0.86, 0.38)),
                (0.46, rgba(0.25, 0.33, 0.43, 0.26)),
                (1., rgba(0.11, 0.16, 0.26, 0.64)),
            ],
        )?;
        g.stroke(
            rect(24.5, 32.5, 31., 31.),
            8.,
            rgba(0.84, 0.91, 1., 0.42),
            1.,
        )?;
        g.text(
            "á",
            rect(24., 30., 32., 34.),
            22.,
            DWRITE_FONT_WEIGHT_SEMI_BOLD,
            rgba(1., 1., 1., 1.),
            true,
        )?;
        label(g, "Accento", rect(68., 34., 124., 28.), 20., p.ink, true)?;
        label(
            g,
            st.t(Text::Settings),
            rect(24., 96., 168., 20.),
            11.,
            p.secondary,
            true,
        )?;
        g.line(
            (24., v.height - 102.),
            (192., v.height - 102.),
            p.separator,
            1.,
        )?;
    }
    label(
        g,
        st.t(NAV[st.section.index()]),
        rect(l, v.body - 82., w, 38.),
        28.,
        p.ink,
        true,
    )?;
    label(
        g,
        st.t([Text::GeneralHint, Text::SetsHint, Text::BlacklistHint][st.section.index()]),
        rect(l, v.body - 38., w, 24.),
        13.,
        p.secondary,
        false,
    )?;
    unsafe {
        g.target.PushAxisAlignedClip(
            &rect(l - 4., v.body, w + 8., (v.footer - v.body - 8.).max(0.)),
            D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
        );
    }
    let result: windows::core::Result<()> = (|| {
        match st.section {
            Section::General => {
                panel(g, rect(l, y, w, 80.), p)?;
                label(
                    g,
                    st.t(Text::Delay),
                    rect(l + 18., y + 12., w - 214., 26.),
                    14.,
                    p.ink,
                    true,
                )?;
                label(
                    g,
                    st.t(Text::DelayHint),
                    rect(l + 18., y + 42., w - 214., 22.),
                    12.,
                    p.secondary,
                    false,
                )?;
                g.fill(rect(l + w - 176., y + 18., 156., 42.), 10., p.inset)?;
                label(
                    g,
                    "ms",
                    rect(l + w - 96., y + 58., 48., 18.),
                    10.,
                    p.tertiary,
                    false,
                )?;
                label(
                    g,
                    st.t(Text::ActiveSets),
                    rect(l, y + 102., w, 24.),
                    14.,
                    p.ink,
                    true,
                )?;
                label(
                    g,
                    st.t(Text::Appearance),
                    rect(l, y + 224., w, 24.),
                    14.,
                    p.ink,
                    true,
                )?;
                label(
                    g,
                    st.t(Text::UiLanguage),
                    rect(l, y + 324., w, 24.),
                    14.,
                    p.ink,
                    true,
                )?;
                label(
                    g,
                    st.t(Text::Independent),
                    rect(l, y + 406., w, 24.),
                    12.,
                    p.secondary,
                    false,
                )?;
                g.line((l, y + 446.), (l + w, y + 446.), p.separator, 1.)?;
                label(
                    g,
                    st.t(Text::Gesture),
                    rect(l, y + 458., w, 24.),
                    12.,
                    p.secondary,
                    false,
                )?;
                label(
                    g,
                    st.t(Text::Release),
                    rect(l, y + 486., w, 30.),
                    11.,
                    p.tertiary,
                    false,
                )?;
                if let Some(e) = st.error.as_ref().filter(|e| e.field == DELAY) {
                    g.stroke(rect(l + w - 176., y + 18., 156., 42.), 10., p.error, 1.5)?;
                    label(g, &e.message, rect(l, y + 80., w, 20.), 11., p.error, false)?;
                }
            }
            Section::CharacterSets => {
                label(
                    g,
                    st.t(Text::SetsNote),
                    rect(l, y + 350., w, 28.),
                    12.,
                    p.secondary,
                    false,
                )?;
                label(
                    g,
                    st.t(Text::AtLeastOne),
                    rect(l, y + 390., w, 24.),
                    12.,
                    p.tertiary,
                    false,
                )?;
            }
            Section::Blacklist => {
                label(g, st.t(Text::AddApp), rect(l, y, w, 24.), 14., p.ink, true)?;
                label(
                    g,
                    st.t(Text::AppHint),
                    rect(l, y + 30., w, 24.),
                    12.,
                    p.secondary,
                    false,
                )?;
                g.fill(rect(l, y + 66., w - 140., 44.), 10., p.inset)?;
                g.stroke(
                    rect(l, y + 66., w - 140., 44.),
                    10.,
                    if st.error.as_ref().is_some_and(|e| e.field == EXCLUDED) {
                        p.error
                    } else if st.focus == EXCLUDED {
                        p.accent
                    } else {
                        p.separator
                    },
                    1.,
                )?;
                if let Some(e) = st.error.as_ref().filter(|e| e.field == EXCLUDED) {
                    label(
                        g,
                        &e.message,
                        rect(l, y + 118., w, 28.),
                        12.,
                        p.error,
                        false,
                    )?;
                }
                if st.prefs.excluded_apps.is_empty() {
                    panel(g, rect(l, y + 166., w, 120.), p)?;
                    label(
                        g,
                        st.t(Text::Empty),
                        rect(l + 24., y + 184., w - 48., 30.),
                        14.,
                        p.ink,
                        true,
                    )?;
                    label(
                        g,
                        st.t(Text::EmptyHint),
                        rect(l + 24., y + 224., w - 48., 30.),
                        12.,
                        p.secondary,
                        false,
                    )?;
                } else {
                    for (i, name) in st.prefs.excluded_apps.iter().enumerate() {
                        let row = y + 164. + i as f32 * 60.;
                        panel(g, rect(l, row, w, 52.), p)?;
                        g.fill(rect(l + 14., row + 14., 24., 24.), 6., p.inset)?;
                        label(
                            g,
                            "a",
                            rect(l + 22., row + 14., 16., 24.),
                            12.,
                            p.secondary,
                            true,
                        )?;
                        label(
                            g,
                            name,
                            rect(l + 54., row + 8., w - 114., 36.),
                            14.,
                            p.ink,
                            false,
                        )?;
                    }
                }
            }
        }
        Ok(())
    })();
    unsafe {
        g.target.PopAxisAlignedClip();
    }
    result?;
    if let Some((top, height)) = scroll_thumb(v, st) {
        g.fill(rect(v.width - 14., top, 5., height), 2.5, p.tertiary)?;
    }
    g.line(
        (if v.compact { 0. } else { 216. }, v.footer),
        (v.width, v.footer),
        p.separator,
        1.,
    )?;
    let (notice, color) = if let Some(e) = &st.error {
        (
            if e.field == 0 {
                e.message.as_str()
            } else {
                st.t(Text::CheckField)
            },
            p.error,
        )
    } else if st.dirty {
        (st.t(Text::Pending), p.secondary)
    } else {
        (st.t(Text::Saved), p.secondary)
    };
    label(
        g,
        notice,
        rect(l, v.footer + 12., w - 164., 52.),
        12.,
        color,
        false,
    )?;
    Ok(())
}
fn sidebar(p: Palette) -> D2D1_COLOR_F {
    if p.dark {
        rgba(0.11, 0.13, 0.17, 1.)
    } else {
        rgba(0.948, 0.957, 0.969, 1.)
    }
}
fn selected(id: u16, st: &State) -> bool {
    match id {
        NAV_FIRST..=252 => id - NAV_FIRST == st.section.index() as u16,
        LANG_FIRST..=213 | ACTIVE_FIRST..=273 => {
            let i = if id >= ACTIVE_FIRST {
                id - ACTIVE_FIRST
            } else {
                id - LANG_FIRST
            } as usize;
            st.prefs.languages.iter().any(|l| l == CODES[i])
        }
        UI_FIRST..=283 => st.prefs.ui_language == CODES[(id - UI_FIRST) as usize],
        THEME_FIRST..=232 => id - THEME_FIRST == theme_index(&st.prefs.theme) as u16,
        PAUSE => !st.paused,
        _ => false,
    }
}
pub unsafe fn draw_item(item: &DRAWITEMSTRUCT) -> bool {
    unsafe {
        let Some(st) = state() else {
            return false;
        };
        let p = palette(&st.prefs);
        let id = item.CtlID as u16;
        let s = scale(item.hwndItem);
        let width = (item.rcItem.right - item.rcItem.left) as f32 / s;
        let height = (item.rcItem.bottom - item.rcItem.top) as f32 / s;
        let on = selected(id, &st);
        let hover = st.hover == id;
        let pressed = item.itemState.0 & ODS_SELECTED.0 != 0;
        let focus =
            item.itemState.0 & ODS_FOCUS.0 != 0 && item.itemState.0 & ODS_NOFOCUSRECT.0 == 0;
        let _ = glass::draw_dc(item.hDC, item.rcItem, 96. * s, (0., 0.), |g| {
            let v = viewport(GetAncestor(item.hwndItem, GA_ROOT));
            let nav = (NAV_FIRST..NAV_FIRST + 3).contains(&id);
            let background = if (nav || id == PAUSE) && !v.compact {
                sidebar(p)
            } else if id >= REMOVE_FIRST {
                p.surface
            } else {
                p.canvas
            };
            g.fill(rect(0., 0., width, height), 0., background)?;
            let area = rect(1., 1., width - 2., height - 2.);
            let mut title = String::new();
            let mut center = true;
            if id == PAUSE {
                label(
                    g,
                    st.t(if st.paused {
                        Text::Disabled
                    } else {
                        Text::Enabled
                    }),
                    rect(8., 0., width - 62., height),
                    if v.compact { 10. } else { 12. },
                    p.ink,
                    true,
                )?;
                let x = width - 50.;
                let y = (height - 24.) / 2.;
                g.fill(
                    rect(x, y, 42., 24.),
                    12.,
                    if on { p.accent } else { p.tertiary },
                )?;
                g.circle(
                    x + if on { 30. } else { 12. },
                    y + 12.,
                    9.,
                    rgba(1., 1., 1., 1.),
                )?;
            } else if nav {
                if on || hover || pressed {
                    g.fill(area, 10., p.selection)?;
                }
                let i = (id - NAV_FIRST) as usize;
                if !v.compact {
                    nav_icon(g, i, 12., 12., if on { p.accent } else { p.secondary })?;
                }
                g.text(
                    st.t(NAV[i]),
                    if v.compact {
                        rect(0., 0., width, height)
                    } else {
                        rect(44., 0., width - 48., height)
                    },
                    if v.compact { 12. } else { 14. },
                    if on {
                        DWRITE_FONT_WEIGHT_SEMI_BOLD
                    } else {
                        DWRITE_FONT_WEIGHT_NORMAL
                    },
                    if on { p.accent } else { p.ink },
                    v.compact,
                )?;
            } else if (LANG_FIRST..=213).contains(&id) {
                g.fill(area, 12., p.surface)?;
                if on || hover {
                    g.fill(area, 12., p.selection)?;
                }
                g.stroke(
                    area,
                    12.,
                    if on {
                        rgba(p.accent.r, p.accent.g, p.accent.b, 0.4)
                    } else {
                        p.separator
                    },
                    1.,
                )?;
                let i = (id - LANG_FIRST) as usize;
                g.fill(rect(16., 16., 40., 40.), 10., p.inset)?;
                g.text(
                    ["à", "ç", "ñ", "ü"][i],
                    rect(16., 16., 40., 40.),
                    24.,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    p.ink,
                    true,
                )?;
                label(
                    g,
                    NAMES[i],
                    rect(72., 10., width - 130., 26.),
                    14.,
                    p.ink,
                    true,
                )?;
                label(
                    g,
                    [
                        "à  è  é  ì  ò  ó  ù",
                        "à  â  æ  ç  é  è  ê  ë  î  ï  ô  œ  ù  û  ü  ÿ",
                        "á  é  í  ñ  ó  ú  ü",
                        "ä  ö  ü  ß",
                    ][i],
                    rect(72., 38., width - 130., 22.),
                    12.,
                    p.secondary,
                    false,
                )?;
                let x = width - 40.;
                g.fill(
                    rect(x, 26., 20., 20.),
                    6.,
                    if on { p.accent } else { p.inset },
                )?;
                if on {
                    g.line((x + 4., 36.), (x + 8., 40.), rgba(1., 1., 1., 1.), 1.6)?;
                    g.line((x + 8., 40.), (x + 16., 31.), rgba(1., 1., 1., 1.), 1.6)?;
                } else {
                    g.stroke(rect(x, 26., 20., 20.), 6., p.separator, 1.)?;
                }
            } else if id == DETAIL {
                title = st.t(Text::SetDetail).into();
                center = false;
            } else if matches!(id, MINUS | PLUS) {
                g.fill(area, 10., p.inset)?;
                if hover || pressed {
                    g.fill(area, 10., p.selection)?;
                }
                title = if id == MINUS { "−" } else { "+" }.into();
            } else if id >= REMOVE_FIRST {
                if hover || pressed {
                    g.fill(area, 8., p.selection)?;
                }
                // One explicit remove button per row; the filename itself is not clickable.
                let x = width / 2.;
                let y = height / 2.;
                g.line((x - 5., y - 5.), (x + 5., y + 5.), p.secondary, 1.6)?;
                g.line((x + 5., y - 5.), (x - 5., y + 5.), p.secondary, 1.6)?;
            } else {
                let primary = matches!(id, SAVE | ADD);
                let enabled = item.itemState.0 & ODS_DISABLED.0 == 0;
                g.fill(
                    area,
                    10.,
                    if primary && enabled {
                        rgba(0.184, 0.392, 0.859, 1.)
                    } else if on || pressed || hover {
                        p.selection
                    } else {
                        p.surface
                    },
                )?;
                g.stroke(
                    area,
                    10.,
                    if on {
                        rgba(p.accent.r, p.accent.g, p.accent.b, 0.5)
                    } else {
                        p.separator
                    },
                    1.,
                )?;
                title = match id {
                    SAVE => st.t(Text::Save).into(),
                    ADD => st.t(Text::Add).into(),
                    THEME_FIRST..=232 => st
                        .t([Text::System, Text::Light, Text::Dark][(id - THEME_FIRST) as usize])
                        .into(),
                    ACTIVE_FIRST..=273 => format!(
                        "{}{}",
                        if on { "✓ " } else { "" },
                        NAMES[(id - ACTIVE_FIRST) as usize]
                    ),
                    UI_FIRST..=283 => NAMES[(id - UI_FIRST) as usize].into(),
                    _ => String::new(),
                };
                if primary {
                    g.text(
                        &title,
                        rect(0., 0., width, height),
                        13.,
                        DWRITE_FONT_WEIGHT_SEMI_BOLD,
                        if enabled {
                            rgba(1., 1., 1., 1.)
                        } else {
                            p.tertiary
                        },
                        true,
                    )?;
                    title.clear();
                }
            }
            if !title.is_empty() {
                g.text(
                    &title,
                    rect(if center { 0. } else { 2. }, 0., width - 4., height),
                    if id == DETAIL { 12. } else { 13. },
                    if on {
                        DWRITE_FONT_WEIGHT_SEMI_BOLD
                    } else {
                        DWRITE_FONT_WEIGHT_NORMAL
                    },
                    if on || id == DETAIL { p.accent } else { p.ink },
                    center,
                )?;
            }
            if focus {
                g.stroke(rect(2., 2., width - 4., height - 4.), 9., p.accent, 2.)?;
            }
            Ok(())
        });
        true
    }
}

pub unsafe extern "system" fn content_proc(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
) -> LRESULT {
    unsafe {
        let root = GetParent(hwnd).unwrap_or_default();
        match msg {
            WM_COMMAND | WM_DRAWITEM | WM_CTLCOLOREDIT | WM_CTLCOLORBTN | WM_CTLCOLORSTATIC
            | WM_MOUSEWHEEL => {
                return send_message(root, msg, wp, lp);
            }
            WM_ERASEBKGND => return LRESULT(1),
            WM_PAINT | WM_PRINTCLIENT => {
                let mut ps = PAINTSTRUCT::default();
                let dc = if msg == WM_PAINT {
                    BeginPaint(hwnd, &mut ps)
                } else {
                    HDC(wp.0 as _)
                };
                if let Some(st) = state() {
                    let v = viewport(root);
                    let mut bounds = RECT::default();
                    let _ = GetClientRect(hwnd, &mut bounds);
                    let _ = glass::draw_dc(
                        dc,
                        bounds,
                        96. * scale(root),
                        (-(v.left - 4.), -v.body),
                        |g| canvas(g, &st, v),
                    );
                }
                if msg == WM_PAINT {
                    let _ = EndPaint(hwnd, &ps);
                }
                return LRESULT(0);
            }
            _ => {}
        }
        DefWindowProcW(hwnd, msg, wp, lp)
    }
}

pub unsafe fn message(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> Option<LRESULT> {
    unsafe {
        match msg {
            WM_PAINT => {
                REPAINTS.with(|n| n.set(n.get() + 1));
                let mut ps = PAINTSTRUCT::default();
                let dc = BeginPaint(hwnd, &mut ps);
                paint(hwnd, dc);
                let _ = EndPaint(hwnd, &ps);
                Some(LRESULT(0))
            }
            WM_PRINTCLIENT => {
                paint(hwnd, HDC(wp.0 as _));
                Some(LRESULT(0))
            }
            WM_ERASEBKGND => Some(LRESULT(1)),
            WM_DRAWITEM => Some(LRESULT(
                draw_item(&*(lp.0 as *const DRAWITEMSTRUCT)) as isize
            )),
            WM_SIZE => {
                layout(hwnd);
                Some(LRESULT(0))
            }
            WM_GETMINMAXINFO => {
                let info = &mut *(lp.0 as *mut MINMAXINFO);
                let s = GetDpiForWindow(hwnd) as f32 / 96.;
                info.ptMinTrackSize = POINT {
                    x: (640. * s) as i32,
                    y: (480. * s) as i32,
                };
                Some(LRESULT(0))
            }
            WM_SETTINGCHANGE | WM_THEMECHANGED => {
                layout(hwnd);
                let prefs = state()?.prefs;
                render::settings_frame(hwnd, &prefs);
                None
            }
            WM_MOUSEWHEEL => {
                let delta = -((wp.0 >> 16) as i16 as f32) / 120. * 64.;
                let old = state()?.scroll;
                scroll_to(hwnd, old + delta);
                Some(LRESULT(0))
            }
            WM_LBUTTONDOWN => {
                let st = state()?;
                let v = viewport(hwnd);
                let s = scale(hwnd);
                let x = (lp.0 as i16) as f32 / s;
                let y = ((lp.0 >> 16) as i16) as f32 / s;
                if let Some((top, height)) = scroll_thumb(v, &st)
                    && x >= v.width - 24.
                    && y >= v.body
                    && y < v.footer - 8.
                {
                    let offset = if y >= top && y <= top + height {
                        y - top
                    } else {
                        height / 2.
                    };
                    SCROLL_DRAG.with(|drag| drag.set(Some(offset)));
                    SetCapture(hwnd);
                    drag_scroll(hwnd, y, offset);
                    return Some(LRESULT(0));
                }
                None
            }
            WM_MOUSEMOVE => {
                if let Some(offset) = SCROLL_DRAG.with(|drag| drag.get()) {
                    drag_scroll(hwnd, ((lp.0 >> 16) as i16) as f32 / scale(hwnd), offset);
                    return Some(LRESULT(0));
                }
                None
            }
            WM_LBUTTONUP => {
                if SCROLL_DRAG.with(|drag| drag.take()).is_some() {
                    let _ = ReleaseCapture();
                    return Some(LRESULT(0));
                }
                None
            }
            WM_CAPTURECHANGED => {
                SCROLL_DRAG.with(|drag| drag.set(None));
                None
            }
            FOCUS_SCROLL => {
                let id = GetDlgCtrlID(HWND(wp.0 as _)) as u16;
                let v = viewport(hwnd);
                if let Some(([_, y, _, h], true)) = v.bounds(id) {
                    let changed = APP.with(|a| {
                        let mut a = a.borrow_mut();
                        let Some(a) = a.as_mut() else {
                            return false;
                        };
                        let old = a.scroll;
                        let next = if y - old < v.body {
                            (y - v.body - 8.).max(0.)
                        } else if y + h - old > v.footer - 8. {
                            y + h - v.footer + 16.
                        } else {
                            old
                        };
                        a.scroll =
                            next.clamp(0., v.scroll_max(a.section, a.draft.excluded_apps.len()));
                        a.scroll != old
                    });
                    if changed {
                        layout(hwnd);
                    }
                }
                Some(LRESULT(0))
            }
            WM_CTLCOLOREDIT => {
                let st = state()?;
                let p = palette(&st.prefs);
                let dc = HDC(wp.0 as _);
                let color = |c: D2D1_COLOR_F| {
                    COLORREF(
                        (c.r * 255.) as u32
                            | ((c.g * 255.) as u32) << 8
                            | ((c.b * 255.) as u32) << 16,
                    )
                };
                let bg = p.inset;
                SetTextColor(dc, color(p.ink));
                SetBkColor(dc, color(bg));
                SetDCBrushColor(dc, color(bg));
                Some(LRESULT(GetStockObject(DC_BRUSH).0 as isize))
            }
            _ => None,
        }
    }
}
unsafe extern "system" fn control_proc(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
    _: usize,
    _: usize,
) -> LRESULT {
    unsafe {
        let parent = GetAncestor(hwnd, GA_ROOT);
        let id = GetDlgCtrlID(hwnd) as u16;
        if msg == WM_WINDOWPOSCHANGED {
            POSITIONS.with(|n| n.set(n.get() + 1));
        }
        if id == EXCLUDED && msg == WM_GETDLGCODE && wp.0 == VK_RETURN.0 as usize {
            return LRESULT(DefSubclassProc(hwnd, msg, wp, lp).0 | DLGC_WANTMESSAGE as isize);
        }
        if id == EXCLUDED && msg == WM_KEYDOWN && wp.0 == VK_RETURN.0 as usize {
            let _ = PostMessageW(Some(parent), WM_COMMAND, WPARAM(ADD as usize), LPARAM(0));
            return LRESULT(0);
        }
        if matches!(msg, WM_MOUSEMOVE | WM_MOUSELEAVE) {
            let hovered = if msg == WM_MOUSEMOVE { id } else { 0 };
            let changed = APP.with(|a| {
                if let Ok(mut a) = a.try_borrow_mut()
                    && let Some(a) = a.as_mut()
                {
                    let changed = a.hover != hovered;
                    a.hover = hovered;
                    changed
                } else {
                    false
                }
            });
            if changed {
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
            if msg == WM_MOUSEMOVE {
                let _ = TrackMouseEvent(&mut TRACKMOUSEEVENT {
                    cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    ..Default::default()
                });
            }
        }
        if matches!(msg, WM_SETFOCUS | WM_KILLFOCUS) {
            if matches!(id, DELAY | EXCLUDED) {
                invalidate(control(parent, CONTENT));
            }
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
        if msg == WM_SETFOCUS {
            let _ = PostMessageW(
                Some(parent),
                FOCUS_SCROLL,
                WPARAM(hwnd.0 as usize),
                LPARAM(0),
            );
        }
        if msg == WM_MOUSEWHEEL {
            let _ = PostMessageW(Some(parent), msg, wp, lp);
            return LRESULT(0);
        }
        if msg == WM_KEYDOWN
            && (NAV_FIRST..NAV_FIRST + 3).contains(&id)
            && matches!(wp.0 as u16, 0x25..=0x28)
        {
            let backwards = matches!(wp.0 as u16, 0x25 | 0x26);
            let i = (id - NAV_FIRST + if backwards { 2 } else { 1 }) % 3;
            let _ = PostMessageW(
                Some(parent),
                WM_COMMAND,
                WPARAM((NAV_FIRST + i) as usize),
                LPARAM(0),
            );
            return LRESULT(0);
        }
        DefSubclassProc(hwnd, msg, wp, lp)
    }
}

pub const CHECK_NAMES: [&str; 14] = [
    "Set attivabile da Generale e Set caratteri",
    "Ultimo set conservato",
    "Aspetto esclusivo",
    "Lingua interfaccia indipendente dai set",
    "Stepper ritardo",
    "Validazione ritardo",
    "Scorrimento solo con contenuto eccedente",
    "Salva sempre raggiungibile",
    "BlackList un elemento per applicazione",
    "BlackList rifiuta duplicati e separatori",
    "Focus porta il campo nel viewport",
    "Navigazione da tastiera",
    "Interruttore spegne e riaccende senza perdere la bozza",
    "Salva disattivato senza modifiche",
];
pub unsafe fn verify_controls(hwnd: HWND) -> u32 {
    unsafe {
        let was_visible = IsWindowVisible(hwnd).as_bool();
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        let original = APP.with(|a| a.borrow().as_ref().unwrap().prefs.clone());
        let mut bits = 0u32;
        let mut check = |i: usize, ok: bool| {
            if ok {
                bits |= 1 << i;
            }
        };
        navigate(hwnd, Section::General);
        command(hwnd, ACTIVE_FIRST + 1, 0);
        let on = state().unwrap().prefs.languages.contains(&"fr".into());
        navigate(hwnd, Section::CharacterSets);
        check(0, on && selected(LANG_FIRST + 1, &state().unwrap()));
        APP.with(|a| a.borrow_mut().as_mut().unwrap().draft.languages = vec!["it".into()]);
        command(hwnd, LANG_FIRST, 0);
        check(1, state().unwrap().prefs.languages == ["it"]);
        command(hwnd, THEME_FIRST + 2, 0);
        check(
            2,
            state().unwrap().prefs.theme == Theme::Dark
                && !selected(THEME_FIRST, &state().unwrap()),
        );
        let sets = state().unwrap().prefs.languages;
        command(hwnd, UI_FIRST + 1, 0);
        check(
            3,
            state().unwrap().prefs.ui_language == "fr"
                && state().unwrap().prefs.languages == sets
                && control_text(hwnd, SAVE) == tr("fr", Text::Save),
        );
        text_if_changed(control(hwnd, DELAY), "300");
        command(hwnd, PLUS, 0);
        check(4, control_text(hwnd, DELAY) == "350");
        text_if_changed(control(hwnd, DELAY), "20");
        let invalid =
            APP.with(|a| draft_from_controls(hwnd, a.borrow().as_ref().unwrap()).is_err());
        check(5, invalid);
        text_if_changed(control(hwnd, DELAY), "350");
        let s = GetDpiForWindow(hwnd) as f32 / 96.;
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            (720. * s) as i32,
            (510. * s) as i32,
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
        navigate(hwnd, Section::General);
        layout(hwnd);
        send_message(
            hwnd,
            WM_MOUSEWHEEL,
            WPARAM(((-120i16) as u16 as usize) << 16),
            LPARAM(0),
        );
        check(6, state().unwrap().scroll > 0.);
        let mut save_bounds = RECT::default();
        let _ = GetWindowRect(control(hwnd, SAVE), &mut save_bounds);
        let mut root_bounds = RECT::default();
        let _ = GetWindowRect(hwnd, &mut root_bounds);
        check(
            7,
            IsWindowVisible(control(hwnd, SAVE)).as_bool()
                && save_bounds.bottom < root_bounds.bottom
                && save_bounds.top > root_bounds.top,
        );
        navigate(hwnd, Section::Blacklist);
        text_if_changed(control(hwnd, EXCLUDED), " Notepad.EXE ");
        command(hwnd, ADD, 0);
        text_if_changed(control(hwnd, EXCLUDED), "code.exe");
        command(hwnd, ADD, 0);
        check(
            8,
            state().unwrap().prefs.excluded_apps == ["notepad.exe", "code.exe"]
                && control_text(hwnd, EXCLUDED).is_empty()
                && IsWindowVisible(control(hwnd, REMOVE_FIRST + 1)).as_bool(),
        );
        text_if_changed(control(hwnd, EXCLUDED), "NOTEPAD.EXE");
        command(hwnd, ADD, 0);
        let dup = state().unwrap().error.is_some();
        text_if_changed(control(hwnd, EXCLUDED), "a.exe,b.exe");
        command(hwnd, ADD, 0);
        check(
            9,
            dup && state().unwrap().error.is_some()
                && state().unwrap().prefs.excluded_apps.len() == 2,
        );
        text_if_changed(control(hwnd, EXCLUDED), "");
        command(hwnd, REMOVE_FIRST, 0);
        navigate(hwnd, Section::General);
        send_message(
            hwnd,
            FOCUS_SCROLL,
            WPARAM(control(hwnd, UI_FIRST).0 as usize),
            LPARAM(0),
        );
        let v = viewport(hwnd);
        let ([_, y, _, h], _) = v.bounds(UI_FIRST).unwrap();
        check(
            10,
            y - state().unwrap().scroll >= v.body && y + h - state().unwrap().scroll <= v.footer,
        );
        let next =
            GetNextDlgTabItem(hwnd, Some(control(hwnd, NAV_FIRST + 2)), false).unwrap_or_default();
        check(
            11,
            !next.is_invalid()
                && IsWindowVisible(next).as_bool()
                && IsWindowEnabled(next).as_bool(),
        );
        let draft = state().unwrap().prefs;
        command(hwnd, PAUSE, 0);
        let paused = state().unwrap().paused;
        command(hwnd, PAUSE, 0);
        check(
            12,
            paused && !state().unwrap().paused && state().unwrap().prefs == draft,
        );
        APP.with(|a| {
            let mut a = a.borrow_mut();
            let a = a.as_mut().unwrap();
            a.draft = original.clone();
            a.prefs = original.clone();
            a.validation_error = None;
            a.scroll = 0.;
        });
        text_if_changed(
            control(hwnd, DELAY),
            &state().unwrap().prefs.delay_ms.to_string(),
        );
        refresh(hwnd, true);
        check(
            13,
            !state().unwrap().dirty && !IsWindowEnabled(control(hwnd, SAVE)).as_bool(),
        );
        let folder = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let report = CHECK_NAMES
            .iter()
            .enumerate()
            .map(|(i, name)| {
                format!(
                    "{}: {name}\n",
                    if bits & (1 << i) != 0 { "PASS" } else { "FAIL" }
                )
            })
            .collect::<String>();
        let _ = std::fs::write(folder.join("accento-ui-controls.txt"), report);
        // Real product windows, rendered by the same controls as the shipped application.
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            (1016. * s) as i32,
            (799. * s) as i32,
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
        for (theme, theme_name) in [(Theme::Light, "light"), (Theme::Dark, "dark")] {
            for section in [Section::General, Section::CharacterSets, Section::Blacklist] {
                APP.with(|a| {
                    let mut a = a.borrow_mut();
                    let a = a.as_mut().unwrap();
                    a.draft.theme = theme.clone();
                    a.section = section;
                    a.scroll = 0.;
                    a.draft.excluded_apps = vec!["notepad.exe".into(), "code.exe".into()];
                });
                if let Some(st) = state() {
                    render::settings_frame(hwnd, &st.prefs);
                }
                APP.with(|a| sync_rows(a.borrow().as_ref().unwrap()));
                layout(hwnd);
                let _ = RedrawWindow(
                    Some(hwnd),
                    None,
                    None,
                    RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN,
                );
                let _ = capture_window(
                    hwnd,
                    &folder.join(format!(
                        "accento-settings-{}-{theme_name}.bmp",
                        section.index()
                    )),
                );
            }
        }
        for (language, section, width, height, slug, paused, error, empty) in [
            (
                "it",
                Section::General,
                720.,
                700.,
                "compact-general",
                false,
                false,
                false,
            ),
            (
                "it",
                Section::CharacterSets,
                720.,
                700.,
                "compact-sets",
                false,
                false,
                false,
            ),
            (
                "it",
                Section::Blacklist,
                720.,
                700.,
                "compact-blacklist",
                false,
                false,
                false,
            ),
            (
                "fr",
                Section::General,
                1000.,
                760.,
                "general-fr",
                false,
                false,
                false,
            ),
            (
                "es",
                Section::General,
                1000.,
                760.,
                "general-es",
                false,
                false,
                false,
            ),
            (
                "de",
                Section::General,
                1000.,
                760.,
                "general-de",
                false,
                false,
                false,
            ),
            (
                "it",
                Section::General,
                1000.,
                760.,
                "general-off",
                true,
                false,
                false,
            ),
            (
                "it",
                Section::Blacklist,
                1000.,
                760.,
                "blacklist-error",
                false,
                true,
                false,
            ),
            (
                "it",
                Section::Blacklist,
                1000.,
                760.,
                "blacklist-empty",
                false,
                false,
                true,
            ),
        ] {
            APP.with(|a| {
                let mut a = a.borrow_mut();
                let a = a.as_mut().unwrap();
                a.draft.ui_language = language.into();
                a.section = section;
                a.scroll = 0.;
                a.paused = paused;
                a.draft.excluded_apps = if empty {
                    vec![]
                } else {
                    vec!["notepad.exe".into(), "code.exe".into()]
                };
                a.validation_error = error.then(|| Issue {
                    field: EXCLUDED,
                    message: tr(language, Text::AppError).into(),
                });
            });
            let _ = SetWindowPos(
                hwnd,
                None,
                0,
                0,
                ((width + 16.) * s) as i32,
                ((height + 39.) * s) as i32,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
            layout(hwnd);
            let _ = RedrawWindow(
                Some(hwnd),
                None,
                None,
                RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN,
            );
            let _ = capture_window(hwnd, &folder.join(format!("accento-settings-{slug}.bmp")));
        }
        APP.with(|a| {
            let mut a = a.borrow_mut();
            let a = a.as_mut().unwrap();
            a.draft = original;
            a.validation_error = None;
            a.paused = false;
            a.scroll = 0.;
            a.section = Section::General;
        });
        text_if_changed(
            control(hwnd, DELAY),
            &state().unwrap().prefs.delay_ms.to_string(),
        );
        layout(hwnd);
        if !was_visible {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
        bits
    }
}
/// Wheel events at a clamped boundary must neither mutate preferences nor repaint/reposition the UI.
pub unsafe fn verify_repaint(hwnd: HWND) -> bool {
    unsafe {
        navigate(hwnd, Section::CharacterSets);
        let _ = RedrawWindow(
            Some(hwnd),
            None,
            None,
            RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN,
        );
        let before = state().unwrap().prefs;
        REPAINTS.with(|n| n.set(0));
        POSITIONS.with(|n| n.set(0));
        for _ in 0..10 {
            send_message(hwnd, WM_MOUSEWHEEL, WPARAM(120 << 16), LPARAM(0));
            let _ = RedrawWindow(Some(hwnd), None, None, RDW_UPDATENOW | RDW_ALLCHILDREN);
        }
        let count = REPAINTS.with(|n| n.get());
        let positions = POSITIONS.with(|n| n.get());
        let unchanged = state().unwrap().prefs == before;
        command(hwnd, LANG_FIRST + 1, 0);
        let click_positions = POSITIONS.with(|n| n.get());
        let passed = count == 0 && positions == 0 && click_positions == 0 && unchanged;
        let report = format!(
            "{}: wheel at top: {count} unnecessary root paints / 10 events; {positions} repositioned controls; preferences unchanged: {unchanged}; click repositioned controls: {click_positions}\n",
            if passed { "PASS" } else { "FAIL" }
        );
        let folder = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let _ = std::fs::write(folder.join("accento-ui-test.txt"), report);
        APP.with(|a| {
            let mut a = a.borrow_mut();
            let a = a.as_mut().unwrap();
            a.draft = Preferences::default();
        });
        passed
    }
}

fn scroll_thumb(v: Layout, st: &State) -> Option<(f32, f32)> {
    let max = v.scroll_max(st.section, st.prefs.excluded_apps.len());
    if max <= 0. {
        return None;
    }
    let track = (v.footer - v.body - 8.).max(1.);
    let height = (track * track / (track + max)).max(28.).min(track);
    Some((v.body + (track - height) * st.scroll / max, height))
}
unsafe fn scroll_to(hwnd: HWND, value: f32) {
    unsafe {
        let v = viewport(hwnd);
        let changed = APP.with(|a| {
            let mut a = a.borrow_mut();
            let Some(a) = a.as_mut() else {
                return false;
            };
            let next = value.clamp(0., v.scroll_max(a.section, a.draft.excluded_apps.len()));
            let changed = next != a.scroll;
            a.scroll = next;
            changed
        });
        if changed {
            layout(hwnd);
        }
    }
}
unsafe fn drag_scroll(hwnd: HWND, y: f32, offset: f32) {
    unsafe {
        let Some(st) = state() else {
            return;
        };
        let v = viewport(hwnd);
        let Some((_, height)) = scroll_thumb(v, &st) else {
            return;
        };
        let track = (v.footer - v.body - 8. - height).max(1.);
        scroll_to(
            hwnd,
            (y - v.body - offset) / track * v.scroll_max(st.section, st.prefs.excluded_apps.len()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_navigation_and_footer_stay_outside_scrolling_body() {
        for (w, h) in [(640., 480.), (720., 700.), (1000., 760.)] {
            let v = Layout::new(w, h);
            for id in [SAVE, PAUSE, NAV_FIRST, NAV_FIRST + 1, NAV_FIRST + 2] {
                let ([x, y, width, height], body) = v.bounds(id).unwrap();
                assert!(!body);
                assert!(x >= 0. && x + width <= w);
                assert!(y >= 0. && y + height <= h);
            }
            assert!(v.scroll_max(Section::General, 0) >= 0.);
        }
    }
    #[test]
    fn blacklist_scroll_extent_tracks_every_item() {
        let v = Layout::new(1000., 760.);
        assert_eq!(v.scroll_max(Section::Blacklist, 0), 0.);
        assert!(v.scroll_max(Section::Blacklist, 20) > 600.);
        assert!(!belongs(REMOVE_FIRST + 2, Section::Blacklist, 2));
        assert!(belongs(REMOVE_FIRST + 1, Section::Blacklist, 2));
        assert!(!belongs(REMOVE_FIRST, Section::General, 2));
    }
}
