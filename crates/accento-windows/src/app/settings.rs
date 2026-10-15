use super::*;
use crate::{
    glass::{self, Painter, Palette},
    render::{rect, rgba},
};
use windows::Win32::Graphics::{
    Direct2D::{Common::*, *},
    DirectWrite::*,
};

pub const WIDTH: f32 = 1000.;
pub const HEIGHT: f32 = 700.; // The native title bar replaces FigCraft's 40-DIP title bar.
pub const THEME_FIRST: u16 = 230;
const MINUS: u16 = 240;
const PLUS: u16 = 241;
const NAV_FIRST: u16 = 250;
pub const CONTENT: u16 = 260;
const FOCUS_SCROLL: u32 = WM_APP + 25;
const NAMES: [&str; 4] = ["Scrittura", "Lingue", "Aspetto", "Applicazioni"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Writing,
    Languages,
    Appearance,
    Applications,
}
impl Section {
    fn index(self) -> usize {
        match self {
            Self::Writing => 0,
            Self::Languages => 1,
            Self::Appearance => 2,
            Self::Applications => 3,
        }
    }
    fn from_index(i: usize) -> Self {
        [
            Self::Writing,
            Self::Languages,
            Self::Appearance,
            Self::Applications,
        ][i]
    }
    fn content_height(self) -> f32 {
        match self {
            Self::Writing => 446.,
            Self::Languages => 450.,
            Self::Appearance => 456.,
            Self::Applications => 310.,
        }
    }
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
        let left = if compact { 32. } else { 256. };
        Self {
            width,
            height,
            compact,
            left,
            content: (width - left - if compact { 32. } else { 40. }).max(260.),
            body: if compact { 190. } else { 140. },
            footer: height - 80.,
        }
    }
    fn scroll_max(self, section: Section) -> f32 {
        (self.body + section.content_height() + 16. - self.footer).max(0.)
    }
    fn bounds(self, id: u16) -> Option<([f32; 4], bool)> {
        let l = self.left;
        let w = self.content;
        let y = self.body;
        if (NAV_FIRST..NAV_FIRST + 4).contains(&id) {
            let i = (id - NAV_FIRST) as f32;
            return Some((
                if self.compact {
                    [32. + i * w / 4., 16., w / 4. - 8., 40.]
                } else {
                    [16., 134. + i * 52., 184., 44.]
                },
                false,
            ));
        }
        let r = match id {
            SAVE => return Some(([self.width - 184., self.footer + 20., 144., 40.], false)),
            DELAY => [l + w - 128., y + 238., 60., 24.],
            MINUS => [l + w - 174., y + 232., 40., 40.],
            PLUS => [l + w - 60., y + 232., 40., 40.],
            TEST_FIELD => [l + 20., y + 392., w - 40., 28.],
            LANG_FIRST..=213 => [l, y + 32. + (id - LANG_FIRST) as f32 * 82., w, 72.],
            THEME_FIRST..=232 => {
                let tw = (w - 48.) / 3.;
                [
                    l + 12. + (id - THEME_FIRST) as f32 * (tw + 12.),
                    y + 44.,
                    tw,
                    122.,
                ]
            }
            TRANSPARENCY => [l + w - 70., y + 292., 52., 40.],
            EXCLUDED => [l + 32., y + 136., w - 64., 24.],
            _ => return None,
        };
        Some((r, true))
    }
}
fn belongs(id: u16, section: Section) -> bool {
    match id {
        DELAY | MINUS | PLUS | TEST_FIELD => section == Section::Writing,
        LANG_FIRST..=213 => section == Section::Languages,
        THEME_FIRST..=232 | TRANSPARENCY => section == Section::Appearance,
        EXCLUDED => section == Section::Applications,
        _ => true,
    }
}
fn field_error(error: &str) -> u16 {
    let value = error.to_lowercase();
    if value.contains("ritardo") {
        DELAY
    } else if value.contains("esclusione") {
        EXCLUDED
    } else {
        0
    }
}
#[derive(Clone)]
struct State {
    prefs: Preferences,
    section: Section,
    hover: u16,
    dirty: bool,
    error: Option<String>,
    focus: u16,
    notice: String,
    paused: bool,
    scroll: f32,
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
                notice: unsafe { control_text(a.main, STATUS) },
                paused: a.paused,
                scroll: a.scroll,
            })
        })
    })
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

pub unsafe fn build(app: &mut App) {
    unsafe {
        let s = GetDpiForWindow(app.main) as f32 / 96.;
        let monitor = MonitorFromWindow(app.main, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(monitor, &mut info);
        let mut r = RECT {
            left: 0,
            top: 0,
            right: (WIDTH * s) as i32,
            bottom: (HEIGHT * s) as i32,
        };
        let _ = AdjustWindowRectExForDpi(
            &mut r,
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_VSCROLL,
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
        for (i, name) in NAMES.iter().enumerate() {
            tile(app, NAV_FIRST + i as u16, name);
        }
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
        for (i, name) in ["Italiano", "Français", "Español", "Deutsch"]
            .iter()
            .enumerate()
        {
            tile(app, LANG_FIRST + i as u16, name);
        }
        for (i, name) in ["Automatico", "Chiaro", "Scuro"].iter().enumerate() {
            tile(app, THEME_FIRST + i as u16, name);
        }
        for (id, name) in [
            (TRANSPARENCY, "Liquid Glass"),
            (MINUS, "Riduci ritardo"),
            (PLUS, "Aumenta ritardo"),
            (SAVE, "Salva modifiche"),
        ] {
            tile(app, id, name);
        }
        field(
            app,
            DELAY,
            &app.draft.delay_ms.to_string(),
            ES_NUMBER as u32,
        );
        field(
            app,
            EXCLUDED,
            &app.draft.excluded_apps.join(", "),
            ES_AUTOHSCROLL as u32,
        );
        field(app, TEST_FIELD, "", ES_AUTOHSCROLL as u32);
        let _ = SetWindowTextW(
            control(app.main, DELAY),
            PCWSTR(wide(&app.draft.delay_ms.to_string()).as_ptr()),
        );
        send_message(
            control(app.main, DELAY),
            EM_SETLIMITTEXT,
            WPARAM(4),
            LPARAM(0),
        );
        send_message(
            control(app.main, EXCLUDED),
            EM_SETLIMITTEXT,
            WPARAM(8192),
            LPARAM(0),
        );
        send_message(
            control(app.main, TEST_FIELD),
            EM_SETCUEBANNER,
            WPARAM(0),
            LPARAM(wide("Scrivi una parola…").as_ptr() as isize),
        );
        let hidden = child(
            app,
            w!("STATIC"),
            "Tutte le modifiche sono salvate",
            WINDOW_STYLE(0),
            STATUS,
            0,
            0,
            1,
            1,
        );
        let _ = ShowWindow(hidden, SW_HIDE);
        render::settings_frame(app.main, &app.draft);
        sync_names(app);
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
        if !matches!(id, NAV_FIRST..=253 | SAVE) {
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

pub unsafe fn layout(hwnd: HWND) {
    unsafe {
        let Some((controls, section, scroll, dirty)) = APP.with(|a| {
            a.try_borrow().ok().and_then(|a| {
                a.as_ref()
                    .map(|a| (a.controls.borrow().clone(), a.section, a.scroll, a.dirty))
            })
        }) else {
            return;
        };
        let v = viewport(hwnd);
        let s = scale(hwnd);
        let scroll = scroll.clamp(0., v.scroll_max(section));
        APP.with(|a| {
            if let Ok(mut a) = a.try_borrow_mut()
                && let Some(a) = a.as_mut()
            {
                a.scroll = scroll;
            }
        });
        // A disabled scroll bar retains its width, preventing a resize/show/hide loop.
        SetScrollInfo(
            hwnd,
            SB_VERT,
            &SCROLLINFO {
                cbSize: size_of::<SCROLLINFO>() as u32,
                fMask: SIF_RANGE | SIF_PAGE | SIF_POS,
                nMin: 0,
                nMax: section.content_height() as i32 + 15,
                nPage: (v.footer - v.body).max(1.) as u32,
                nPos: scroll as i32,
                ..Default::default()
            },
            true,
        );
        let content = control(hwnd, CONTENT);
        let _ = SetWindowPos(
            content,
            None,
            ((v.left - 4.) * s).round() as i32,
            (v.body * s).round() as i32,
            ((v.content + 8.) * s).round() as i32,
            ((v.footer - v.body - 12.).max(0.) * s).round() as i32,
            SWP_NOZORDER | SWP_NOACTIVATE,
        );
        let _ = EnableWindow(control(hwnd, SAVE), dirty);
        for (window, _, _) in controls {
            let window = HWND(window as _);
            let id = GetDlgCtrlID(window) as u16;
            let Some(([x, y, w, h], body)) = v.bounds(id) else {
                continue;
            };
            let show = belongs(id, section);
            let _ = ShowWindow(window, if show { SW_SHOWNA } else { SW_HIDE });
            if !show {
                continue;
            }
            let y = y - if body { scroll } else { 0. };
            let px = ((x - if body { v.left - 4. } else { 0. }) * s).round() as i32;
            let py = ((y - if body { v.body } else { 0. }) * s).round() as i32;
            let pw = (w * s).round() as i32;
            let ph = (h * s).round() as i32;
            let _ = SetWindowPos(window, None, px, py, pw, ph, SWP_NOZORDER | SWP_NOACTIVATE);
        }
        let _ = RedrawWindow(Some(hwnd), None, None, RDW_INVALIDATE | RDW_ALLCHILDREN);
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
fn keycap(g: &Painter, area: D2D_RECT_F, text: &str, p: Palette) -> windows::core::Result<()> {
    g.fill(area, 9., p.inset)?;
    g.stroke(area, 9., p.separator, 1.)?;
    g.text(text, area, 13., DWRITE_FONT_WEIGHT_SEMI_BOLD, p.ink, true)
}
fn preview(g: &Painter, x: f32, y: f32, p: Palette, glass: bool) -> windows::core::Result<()> {
    for (i, c) in ["è", "é"].iter().enumerate() {
        let area = rect(x + i as f32 * 72., y, 64., 64.);
        if glass {
            g.gradient(
                area,
                17.,
                &[
                    (
                        0.,
                        if p.dark {
                            rgba(0.3, 0.35, 0.43, 1.)
                        } else {
                            rgba(1., 1., 1., 1.)
                        },
                    ),
                    (
                        1.,
                        if p.dark {
                            p.surface
                        } else {
                            rgba(0.90, 0.93, 0.97, 1.)
                        },
                    ),
                ],
            )?;
        } else {
            g.fill(area, 17., p.surface)?;
        }
        if i == 0 {
            g.fill(area, 17., p.selection)?;
        }
        g.stroke(
            area,
            17.,
            if i == 0 { p.accent } else { p.separator },
            if i == 0 { 1.2 } else { 1. },
        )?;
        g.text(
            c,
            area,
            32.,
            DWRITE_FONT_WEIGHT_NORMAL,
            if i == 0 { p.accent } else { p.ink },
            true,
        )?;
    }
    Ok(())
}
pub unsafe fn paint(hwnd: HWND, dc: HDC) {
    unsafe {
        let Some(st) = state() else { return };
        let mut r = RECT::default();
        let _ = GetClientRect(hwnd, &mut r);
        let v = viewport(hwnd);
        let s = scale(hwnd);
        let _ = glass::draw_dc(dc, r, 96. * s, (0., 0.), |g| canvas(g, &st, v));
    }
}
fn canvas(g: &Painter, st: &State, v: Layout) -> windows::core::Result<()> {
    let p = palette(&st.prefs);
    unsafe {
        g.target.Clear(Some(&p.canvas));
    }
    let l = v.left;
    let w = v.content;
    if !v.compact {
        g.fill(
            rect(0., 0., 216., v.height),
            0.,
            if p.dark {
                rgba(0.11, 0.13, 0.17, 1.)
            } else {
                rgba(0.948, 0.957, 0.969, 1.)
            },
        )?;
        g.fill(rect(24., 32., 32., 32.), 10., rgba(0.184, 0.392, 0.859, 1.))?;
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
            "IMPOSTAZIONI",
            rect(24., 98., 168., 20.),
            10.,
            p.secondary,
            true,
        )?;
        let sy = v.height - 126.;
        g.line((24., sy), (192., sy), p.separator, 1.)?;
        g.circle(
            27.,
            v.height - 93.,
            3.,
            if st.paused {
                p.tertiary
            } else {
                rgba(0.13, 0.52, 0.31, 1.)
            },
        )?;
        label(
            g,
            if st.paused {
                "Accento è in pausa"
            } else {
                "Accento è attivo"
            },
            rect(38., v.height - 107., 158., 28.),
            12.,
            p.secondary,
            false,
        )?;
        label(
            g,
            "Resta nell’area di notifica",
            rect(24., v.height - 73., 176., 24.),
            11.,
            p.tertiary,
            false,
        )?;
    }
    let title_y = v.body - 98.;
    label(
        g,
        NAMES[st.section.index()],
        rect(l, title_y, w, 42.),
        28.,
        p.ink,
        true,
    )?;
    label(
        g,
        [
            "Il tuo gesto, al ritmo giusto.",
            "Solo i caratteri che ti servono.",
            "La luce cambia. La scrittura resta al centro.",
            "Scegli dove lasciare la tastiera così com’è.",
        ][st.section.index()],
        rect(l, title_y + 46., w, 24.),
        13.,
        p.secondary,
        false,
    )?;
    unsafe {
        g.target.PushAxisAlignedClip(
            &rect(l - 4., v.body, w + 8., (v.footer - v.body - 12.).max(0.)),
            D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
        );
    }
    let result: windows::core::Result<()> = (|| {
        let y = v.body - st.scroll;
        match st.section {
            Section::Writing => {
                label(g, "ATTIVAZIONE", rect(l, y, w, 20.), 11., p.secondary, true)?;
                panel(g, rect(l, y + 32., w, 164.), p)?;
                label(
                    g,
                    "Gesto di apertura",
                    rect(l + 20., y + 50., w - 40., 24.),
                    14.,
                    p.ink,
                    true,
                )?;
                keycap(g, rect(l + 20., y + 86., 72., 36.), "Lettera", p)?;
                label(
                    g,
                    "+",
                    rect(l + 102., y + 90., 24., 26.),
                    14.,
                    p.secondary,
                    false,
                )?;
                keycap(g, rect(l + 132., y + 86., 76., 36.), "Spazio", p)?;
                label(
                    g,
                    "Tieni la lettera, premi Spazio e scegli con ← →.",
                    rect(l + 20., y + 140., w - 40., 22.),
                    13.,
                    p.secondary,
                    false,
                )?;
                label(
                    g,
                    "Rilascia la lettera per inserire. Esc per annullare.",
                    rect(l + 20., y + 164., w - 40., 20.),
                    12.,
                    p.secondary,
                    false,
                )?;
                panel(g, rect(l, y + 212., w, 100.), p)?;
                label(
                    g,
                    "Ritardo di apertura",
                    rect(l + 20., y + 228., w - 224., 24.),
                    14.,
                    p.ink,
                    true,
                )?;
                label(
                    g,
                    "Da 100 a 1500 ms",
                    rect(l + 20., y + 261., w - 224., 24.),
                    12.,
                    p.secondary,
                    false,
                )?;
                g.fill(rect(l + w - 174., y + 232., 154., 40.), 10., p.inset)?;
                label(
                    g,
                    "ms",
                    rect(l + w - 96., y + 272., 56., 20.),
                    10.,
                    p.tertiary,
                    false,
                )?;
                if st.error.as_ref().is_some_and(|e| field_error(e) == DELAY) {
                    g.stroke(rect(l + w - 174., y + 232., 154., 40.), 10., p.error, 1.5)?;
                    label(
                        g,
                        "Usa un valore da 100 a 1500 ms.",
                        rect(l + 20., y + 290., w - 210., 20.),
                        11.,
                        p.error,
                        false,
                    )?;
                }
                label(
                    g,
                    "PROVALO QUI",
                    rect(l, y + 336., w, 20.),
                    11.,
                    p.secondary,
                    true,
                )?;
                panel(g, rect(l, y + 366., w, 80.), p)?;
            }
            Section::Languages => {
                label(
                    g,
                    "LINGUE DISPONIBILI",
                    rect(l, y, w, 20.),
                    11.,
                    p.secondary,
                    true,
                )?;
                g.text(
                    &format!(
                        "{} attiv{}",
                        st.prefs.languages.len(),
                        if st.prefs.languages.len() == 1 {
                            "a"
                        } else {
                            "e"
                        }
                    ),
                    rect(l + w - 100., y, 100., 20.),
                    12.,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    p.secondary,
                    true,
                )?;
                label(
                    g,
                    "Puoi attivare più lingue. I caratteri comuni compaiono una sola volta.",
                    rect(l, y + 380., w, 24.),
                    12.,
                    p.secondary,
                    false,
                )?;
                label(
                    g,
                    "Mantieni almeno una lingua attiva.",
                    rect(l, y + 426., w, 22.),
                    12.,
                    p.secondary,
                    false,
                )?;
            }
            Section::Appearance => {
                label(g, "TEMA", rect(l, y, w, 20.), 11., p.secondary, true)?;
                panel(g, rect(l, y + 32., w, 146.), p)?;
                label(
                    g,
                    if st.prefs.theme == Theme::System {
                        "Automatico segue il tema di Windows."
                    } else {
                        "Il tema cambia subito in anteprima. Salva per conservarlo."
                    },
                    rect(l, y + 188., w, 22.),
                    12.,
                    p.secondary,
                    false,
                )?;
                label(
                    g,
                    "MATERIALE DEL POPUP",
                    rect(l, y + 240., w, 20.),
                    11.,
                    p.secondary,
                    true,
                )?;
                panel(g, rect(l, y + 272., w, 90.), p)?;
                label(
                    g,
                    "Liquid Glass",
                    rect(l + 20., y + 286., w - 108., 24.),
                    14.,
                    p.ink,
                    true,
                )?;
                label(
                    g,
                    "Luce e trasparenza sui singoli caratteri.",
                    rect(l + 20., y + 314., w - 108., 22.),
                    12.,
                    p.secondary,
                    false,
                )?;
                label(
                    g,
                    "Con la trasparenza disattivata, i tasti usano",
                    rect(l, y + 378., w - 180., 20.),
                    12.,
                    p.secondary,
                    false,
                )?;
                label(
                    g,
                    "una superficie solida.",
                    rect(l, y + 400., w - 180., 20.),
                    12.,
                    p.secondary,
                    false,
                )?;
                preview(
                    g,
                    l + w - 152.,
                    y + 380.,
                    p,
                    render::glass_enabled(&st.prefs),
                )?;
            }
            Section::Applications => {
                label(g, "APP ESCLUSE", rect(l, y, w, 20.), 11., p.secondary, true)?;
                panel(g, rect(l, y + 32., w, 220.), p)?;
                label(
                    g,
                    "Dove Accento non si attiva",
                    rect(l + 20., y + 48., w - 40., 24.),
                    14.,
                    p.ink,
                    true,
                )?;
                label(
                    g,
                    "Nomi completi, separati da virgola. Senza percorsi.",
                    rect(l + 20., y + 80., w - 40., 24.),
                    12.,
                    p.secondary,
                    false,
                )?;
                g.fill(rect(l + 20., y + 126., w - 40., 40.), 10., p.inset)?;
                g.stroke(
                    rect(l + 20., y + 126., w - 40., 40.),
                    10.,
                    if st
                        .error
                        .as_ref()
                        .is_some_and(|e| field_error(e) == EXCLUDED)
                    {
                        p.error
                    } else if st.focus == EXCLUDED {
                        p.accent
                    } else {
                        p.separator
                    },
                    if st.focus == EXCLUDED { 1.5 } else { 1. },
                )?;
                if st
                    .error
                    .as_ref()
                    .is_some_and(|e| field_error(e) == EXCLUDED)
                {
                    label(
                        g,
                        "Inserisci un nome completo: app.exe, senza percorso.",
                        rect(l + 20., y + 174., w - 40., 22.),
                        12.,
                        p.error,
                        false,
                    )?;
                }
                label(
                    g,
                    "Il confronto usa il nome del processo e ignora maiuscole e minuscole.",
                    rect(l + 20., y + 210., w - 40., 24.),
                    12.,
                    p.secondary,
                    false,
                )?;
                label(
                    g,
                    "Nelle app escluse la tastiera mantiene il comportamento abituale.",
                    rect(l, y + 276., w, 30.),
                    13.,
                    p.secondary,
                    false,
                )?;
            }
        }
        Ok(())
    })();
    unsafe {
        g.target.PopAxisAlignedClip();
    }
    result?;
    g.line(
        (if v.compact { 0. } else { 216. }, v.footer),
        (v.width, v.footer),
        p.separator,
        1.,
    )?;
    let (notice, color) = if let Some(error) = &st.error {
        (
            if field_error(error) != 0 {
                "Controlla il campo evidenziato"
            } else {
                error.as_str()
            },
            p.error,
        )
    } else if st.dirty {
        ("Modifiche da salvare", p.secondary)
    } else if st.notice == "Impostazioni salvate" {
        (
            "Impostazioni salvate",
            if p.dark {
                rgba(0.47, 0.86, 0.64, 1.)
            } else {
                rgba(0.13, 0.52, 0.31, 1.)
            },
        )
    } else {
        ("Tutte le modifiche sono salvate", p.secondary)
    };
    label(
        g,
        notice,
        rect(l, v.footer + 21., w - 170., 40.),
        12.,
        color,
        false,
    )?;
    Ok(())
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
fn selected(id: u16, st: &State) -> bool {
    match id {
        NAV_FIRST..=253 => id - NAV_FIRST == st.section.index() as u16,
        LANG_FIRST..=213 => st
            .prefs
            .languages
            .iter()
            .any(|l| l == ["it", "fr", "es", "de"][(id - LANG_FIRST) as usize]),
        THEME_FIRST..=232 => {
            id - THEME_FIRST
                == match st.prefs.theme {
                    Theme::System => 0,
                    Theme::Light => 1,
                    Theme::Dark => 2,
                }
        }
        TRANSPARENCY => st.prefs.transparency,
        _ => false,
    }
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
        2 => {
            g.stroke(rect(x + 1., y + 1., 18., 18.), 9., color, 1.5)?;
            g.line((x + 10., y + 1.), (x + 10., y + 19.), color, 1.5)?;
        }
        _ => {
            g.stroke(rect(x + 1., y + 2., 18., 16.), 1., color, 1.5)?;
            g.line((x + 1., y + 7.), (x + 19., y + 7.), color, 1.5)?;
        }
    }
    Ok(())
}
pub unsafe fn draw_item(item: &DRAWITEMSTRUCT) -> bool {
    unsafe {
        let Some(st) = state() else { return false };
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
            let background = if (NAV_FIRST..NAV_FIRST + 4).contains(&id) && !v.compact {
                if p.dark {
                    rgba(0.11, 0.13, 0.17, 1.)
                } else {
                    rgba(0.948, 0.957, 0.969, 1.)
                }
            } else if matches!(id, THEME_FIRST..=232 | TRANSPARENCY | MINUS | PLUS) {
                p.surface
            } else {
                p.canvas
            };
            g.fill(rect(0., 0., width, height), 0., background)?;
            if id == SAVE {
                g.fill(
                    rect(0.5, 0.5, width - 1., height - 1.),
                    10.,
                    if st.dirty {
                        if pressed {
                            rgba(0.13, 0.29, 0.66, 1.)
                        } else if hover {
                            rgba(0.24, 0.45, 0.92, 1.)
                        } else {
                            rgba(0.184, 0.392, 0.859, 1.)
                        }
                    } else {
                        p.inset
                    },
                )?;
                if !st.dirty {
                    g.stroke(
                        rect(0.5, 0.5, width - 1., height - 1.),
                        10.,
                        p.separator,
                        1.,
                    )?;
                }
                g.text(
                    "Salva modifiche",
                    rect(0., 0., width, height),
                    13.,
                    DWRITE_FONT_WEIGHT_SEMI_BOLD,
                    if st.dirty {
                        rgba(1., 1., 1., 1.)
                    } else {
                        p.tertiary
                    },
                    true,
                )?;
            } else if (NAV_FIRST..NAV_FIRST + 4).contains(&id) {
                if on || hover || pressed {
                    g.fill(
                        rect(0., 0., width, height),
                        10.,
                        if on {
                            p.selection
                        } else {
                            rgba(p.accent.r, p.accent.g, p.accent.b, 0.05)
                        },
                    )?;
                }
                let ink = if on { p.accent } else { p.ink };
                let i = (id - NAV_FIRST) as usize;
                if !v.compact {
                    nav_icon(g, i, 12., 12., if on { p.accent } else { p.secondary })?;
                }
                g.text(
                    NAMES[i],
                    if v.compact {
                        rect(0., 0., width, height)
                    } else {
                        rect(44., 0., width - 48., height)
                    },
                    if v.compact { 13. } else { 14. },
                    if on {
                        DWRITE_FONT_WEIGHT_SEMI_BOLD
                    } else {
                        DWRITE_FONT_WEIGHT_NORMAL
                    },
                    ink,
                    v.compact,
                )?;
            } else if (LANG_FIRST..=213).contains(&id) {
                g.fill(rect(0., 0., width, height), 12., p.surface)?;
                if on || hover {
                    g.fill(
                        rect(0., 0., width, height),
                        12.,
                        if on {
                            p.selection
                        } else {
                            rgba(p.accent.r, p.accent.g, p.accent.b, 0.04)
                        },
                    )?;
                }
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
                    ["Italiano", "Français", "Español", "Deutsch"][i],
                    rect(72., 12., width - 146., 24.),
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
                    rect(72., 38., width - 146., 20.),
                    12.,
                    p.secondary,
                    false,
                )?;
                let x = width - 42.;
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
            } else if (THEME_FIRST..=232).contains(&id) {
                g.fill(rect(0., 0., width, height), 12., p.canvas)?;
                if on || hover {
                    g.fill(
                        rect(0., 0., width, height),
                        12.,
                        if on {
                            p.selection
                        } else {
                            rgba(p.accent.r, p.accent.g, p.accent.b, 0.04)
                        },
                    )?;
                }
                if on {
                    g.stroke(
                        rect(0.5, 0.5, width - 1., height - 1.),
                        11.5,
                        rgba(p.accent.r, p.accent.g, p.accent.b, 0.5),
                        1.,
                    )?;
                }
                let i = (id - THEME_FIRST) as usize;
                let sw = width - 32.;
                g.fill(
                    rect(16., 12., sw, 58.),
                    7.,
                    if i == 2 {
                        rgba(0.125, 0.149, 0.196, 1.)
                    } else {
                        rgba(0.965, 0.973, 0.988, 1.)
                    },
                )?;
                if i == 0 {
                    g.fill(
                        rect(16. + sw / 2., 12., sw / 2., 58.),
                        7.,
                        rgba(0.125, 0.149, 0.196, 1.),
                    )?;
                }
                g.fill(rect(22., 20., 18., 42.), 4., rgba(0.52, 0.58, 0.67, 0.25))?;
                for row in 0..3 {
                    g.fill(
                        rect(48., 22. + row as f32 * 12., sw - 48., 4.),
                        2.,
                        rgba(0.53, 0.58, 0.67, 0.4),
                    )?;
                }
                g.text(
                    ["Automatico", "Chiaro", "Scuro"][i],
                    rect(8., 82., width - 16., 28.),
                    13.,
                    if on {
                        DWRITE_FONT_WEIGHT_SEMI_BOLD
                    } else {
                        DWRITE_FONT_WEIGHT_NORMAL
                    },
                    if on { p.accent } else { p.ink },
                    true,
                )?;
            } else if id == TRANSPARENCY {
                g.fill(
                    rect(4., 7., 44., 26.),
                    13.,
                    if on { p.accent } else { p.tertiary },
                )?;
                g.circle(if on { 36. } else { 17. }, 20., 10., rgba(1., 1., 1., 1.))?;
            } else if matches!(id, MINUS | PLUS) {
                g.fill(
                    if id == MINUS {
                        rect(0., 0., width + 12., height)
                    } else {
                        rect(-12., 0., width + 12., height)
                    },
                    10.,
                    p.inset,
                )?;
                if hover || pressed {
                    g.fill(rect(2., 2., width - 4., height - 4.), 8., p.selection)?;
                }
                g.text(
                    if id == MINUS { "−" } else { "+" },
                    rect(0., 0., width, height),
                    19.,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    p.secondary,
                    true,
                )?;
            }
            if focus {
                g.stroke(rect(1.5, 1.5, width - 3., height - 3.), 10., p.accent, 2.)?;
            }
            Ok(())
        });
        true
    }
}

unsafe fn draft_from_controls(hwnd: HWND, a: &App) -> Result<Preferences, String> {
    unsafe {
        let mut p = a.draft.clone();
        p.delay_ms = control_text(hwnd, DELAY)
            .trim()
            .parse()
            .map_err(|_| "Inserisci un ritardo numerico tra 100 e 1500 ms.".to_string())?;
        p.excluded_apps = control_text(hwnd, EXCLUDED)
            .split([',', ';', '\n'])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_ascii_lowercase)
            .collect();
        p.validate()?;
        Ok(p)
    }
}
unsafe fn refresh(hwnd: HWND) {
    unsafe {
        let prefs = APP.with(|a| {
            let mut a = a.borrow_mut();
            let a = a.as_mut().unwrap();
            a.dirty = !draft_from_controls(hwnd, a).is_ok_and(|p| p == a.prefs);
            sync_names(a);
            a.draft.clone()
        });
        render::settings_frame(hwnd, &prefs);
        layout(hwnd);
    }
}
unsafe fn save(hwnd: HWND) {
    unsafe {
        let result = APP.with(|a| {
            let mut a = a.borrow_mut();
            let a = a.as_mut().unwrap();
            let p = draft_from_controls(hwnd, a)?;
            p.save()?;
            if let Some(worker) = &a.worker {
                worker.configure(p.clone());
            }
            a.prefs = p.clone();
            a.draft = p;
            a.dirty = false;
            a.validation_error = None;
            Ok::<(), String>(())
        });
        match result {
            Ok(()) => status(hwnd, "Impostazioni salvate"),
            Err(e) => {
                let field = field_error(&e);
                APP.with(|a| {
                    let mut a = a.borrow_mut();
                    let a = a.as_mut().unwrap();
                    a.validation_error = Some(e);
                    if field == DELAY {
                        a.section = Section::Writing;
                    } else if field == EXCLUDED {
                        a.section = Section::Applications;
                    }
                    a.scroll = 0.;
                });
                layout(hwnd);
                if field != 0 {
                    let _ = SetFocus(Some(control(hwnd, field)));
                }
            }
        }
        layout(hwnd);
    }
}
pub unsafe fn command(hwnd: HWND, id: u16, notification: u16) -> bool {
    unsafe {
        if matches!(id, DELAY | EXCLUDED) && notification == EN_CHANGE as u16 {
            let changed = APP.with(|a| {
                if let Ok(mut a) = a.try_borrow_mut()
                    && let Some(a) = a.as_mut()
                {
                    a.dirty = true;
                    a.validation_error = None;
                    true
                } else {
                    false
                }
            });
            if changed {
                refresh(hwnd);
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
        if (NAV_FIRST..NAV_FIRST + 4).contains(&id) {
            APP.with(|a| {
                let mut a = a.borrow_mut();
                let a = a.as_mut().unwrap();
                a.section = Section::from_index((id - NAV_FIRST) as usize);
                a.scroll = 0.;
            });
            // Focus must not remain in a control hidden by the new section.
            let _ = SetFocus(Some(control(hwnd, id)));
            layout(hwnd);
            return true;
        }
        if !matches!(id,LANG_FIRST..=213|THEME_FIRST..=232|TRANSPARENCY|MINUS|PLUS) {
            return false;
        }
        APP.with(|a| {
            let mut a = a.borrow_mut();
            let a = a.as_mut().unwrap();
            match id {
                LANG_FIRST..=213 => {
                    let code = ["it", "fr", "es", "de"][(id - LANG_FIRST) as usize];
                    if let Some(i) = a.draft.languages.iter().position(|l| l == code) {
                        if a.draft.languages.len() > 1 {
                            a.draft.languages.remove(i);
                        }
                    } else {
                        a.draft.languages.push(code.into());
                    }
                }
                THEME_FIRST..=232 => {
                    a.draft.theme = match id {
                        231 => Theme::Light,
                        232 => Theme::Dark,
                        _ => Theme::System,
                    }
                }
                TRANSPARENCY => a.draft.transparency = !a.draft.transparency,
                MINUS | PLUS => {
                    let current = control_text(hwnd, DELAY).parse::<u64>().unwrap_or(300);
                    let value = if id == MINUS {
                        current.saturating_sub(50).clamp(100, 1500)
                    } else {
                        current.saturating_add(50).clamp(100, 1500)
                    };
                    let _ = SetWindowTextW(
                        control(hwnd, DELAY),
                        PCWSTR(wide(&value.to_string()).as_ptr()),
                    );
                    a.draft.delay_ms = value;
                }
                _ => {}
            }
        });
        refresh(hwnd);
        true
    }
}
unsafe fn sync_names(app: &App) {
    unsafe {
        for (i, name) in NAMES.iter().enumerate() {
            let text = wide(&format!(
                "{name}{}",
                if app.section.index() == i {
                    ", selezionato"
                } else {
                    ""
                }
            ));
            let _ = SetWindowTextW(
                control(app.main, NAV_FIRST + i as u16),
                PCWSTR(text.as_ptr()),
            );
        }
        for (i, (name, code)) in [
            ("Italiano", "it"),
            ("Français", "fr"),
            ("Español", "es"),
            ("Deutsch", "de"),
        ]
        .iter()
        .enumerate()
        {
            let on = app.draft.languages.iter().any(|l| l == code);
            let text = wide(&format!(
                "{name}, {}",
                if on { "selezionato" } else { "non selezionato" }
            ));
            let _ = SetWindowTextW(
                control(app.main, LANG_FIRST + i as u16),
                PCWSTR(text.as_ptr()),
            );
        }
        for (i, name) in ["Automatico", "Chiaro", "Scuro"].iter().enumerate() {
            let on = match app.draft.theme {
                Theme::System => i == 0,
                Theme::Light => i == 1,
                Theme::Dark => i == 2,
            };
            let text = wide(&format!(
                "{name}, {}",
                if on { "selezionato" } else { "non selezionato" }
            ));
            let _ = SetWindowTextW(
                control(app.main, THEME_FIRST + i as u16),
                PCWSTR(text.as_ptr()),
            );
        }
        let _ = SetWindowTextW(
            control(app.main, TRANSPARENCY),
            PCWSTR(
                wide(if app.draft.transparency {
                    "Liquid Glass, attivo"
                } else {
                    "Liquid Glass, disattivato"
                })
                .as_ptr(),
            ),
        );
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
            WM_MOUSEWHEEL | WM_VSCROLL => {
                let v = viewport(hwnd);
                APP.with(|a| {
                    if let Ok(mut a) = a.try_borrow_mut()
                        && let Some(a) = a.as_mut()
                    {
                        let delta = if msg == WM_MOUSEWHEEL {
                            -((wp.0 >> 16) as i16 as f32) / 120. * 64.
                        } else {
                            match (wp.0 & 0xffff) as u32 {
                                0 => -40.,
                                1 => 40.,
                                2 => -(v.footer - v.body) * 0.8,
                                3 => (v.footer - v.body) * 0.8,
                                4 | 5 => {
                                    let mut info = SCROLLINFO {
                                        cbSize: size_of::<SCROLLINFO>() as u32,
                                        fMask: SIF_TRACKPOS,
                                        ..Default::default()
                                    };
                                    let _ = GetScrollInfo(hwnd, SB_VERT, &mut info);
                                    info.nTrackPos as f32 - a.scroll
                                }
                                _ => 0.,
                            }
                        };
                        a.scroll = (a.scroll + delta).clamp(0., v.scroll_max(a.section));
                    }
                });
                layout(hwnd);
                Some(LRESULT(0))
            }
            FOCUS_SCROLL => {
                let id = GetDlgCtrlID(HWND(wp.0 as _)) as u16;
                let v = viewport(hwnd);
                if let Some(([_, y, _, h], true)) = v.bounds(id) {
                    APP.with(|a| {
                        if let Some(a) = a.borrow_mut().as_mut() {
                            if y - a.scroll < v.body {
                                a.scroll = (y - v.body - 8.).max(0.);
                            } else if y + h - a.scroll > v.footer - 12. {
                                a.scroll = y + h - v.footer + 20.;
                            }
                        }
                    });
                    layout(hwnd);
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
                let id = GetDlgCtrlID(HWND(lp.0 as _)) as u16;
                let bg = if id == TEST_FIELD { p.surface } else { p.inset };
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
            let _ = InvalidateRect(Some(parent), None, false);
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
            && (NAV_FIRST..NAV_FIRST + 4).contains(&id)
            && matches!(wp.0 as u16, 0x25..=0x28)
        {
            let backwards = matches!(wp.0 as u16, 0x25 | 0x26);
            let i = (id - NAV_FIRST + if backwards { 3 } else { 1 }) % 4;
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

pub unsafe fn verify_controls(hwnd: HWND) -> u32 {
    unsafe {
        let original = APP.with(|a| a.borrow().as_ref().unwrap().draft.clone());
        let delay = control_text(hwnd, DELAY);
        let folder = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let mut mask = 0;
        let click = |id| {
            send_message(hwnd, WM_COMMAND, WPARAM(id as usize), LPARAM(0));
        };
        let capture = |name: &str| {
            let _ = RedrawWindow(
                Some(hwnd),
                None,
                None,
                RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN,
            );
            let _ = capture_window(hwnd, &folder.join(name));
        };
        let _ = ShowWindow(hwnd, SW_SHOWNORMAL);
        click(NAV_FIRST + 1);
        click(LANG_FIRST + 1);
        if APP.with(|a| {
            a.borrow()
                .as_ref()
                .unwrap()
                .draft
                .languages
                .iter()
                .any(|l| l == "fr")
        }) {
            mask |= 1;
        }
        click(LANG_FIRST + 1);
        click(LANG_FIRST);
        if APP.with(|a| a.borrow().as_ref().unwrap().draft.languages == vec!["it"]) {
            mask |= 2;
        }
        capture("accento-settings-languages.bmp");
        click(NAV_FIRST + 2);
        click(THEME_FIRST + 1);
        let light = APP.with(|a| a.borrow().as_ref().unwrap().draft.theme == Theme::Light);
        capture("accento-settings-appearance.bmp");
        click(THEME_FIRST + 2);
        if light && APP.with(|a| a.borrow().as_ref().unwrap().draft.theme == Theme::Dark) {
            mask |= 4;
        }
        capture("accento-settings-dark.bmp");
        click(TRANSPARENCY);
        if APP.with(|a| !a.borrow().as_ref().unwrap().draft.transparency) {
            mask |= 8;
        }
        click(NAV_FIRST);
        click(THEME_FIRST + 1);
        click(PLUS);
        if control_text(hwnd, DELAY) == "350" {
            mask |= 16;
        }
        click(MINUS);
        capture("accento-settings-light.bmp");
        let _ = SetWindowTextW(control(hwnd, DELAY), w!("20"));
        click(SAVE);
        if APP.with(|a| {
            let a = a.borrow();
            let a = a.as_ref().unwrap();
            a.validation_error.is_some() && a.prefs.delay_ms == 300
        }) {
            mask |= 32;
        }
        let _ = SetWindowTextW(control(hwnd, DELAY), PCWSTR(wide(&delay).as_ptr()));
        click(NAV_FIRST + 3);
        let _ = SetWindowTextW(control(hwnd, EXCLUDED), w!("notepad"));
        click(NAV_FIRST);
        click(NAV_FIRST + 3);
        if control_text(hwnd, EXCLUDED) == "notepad" {
            mask |= 1 << 8;
        }
        click(NAV_FIRST);
        click(SAVE);
        if APP.with(|a| a.borrow().as_ref().unwrap().section == Section::Applications)
            && GetFocus() == control(hwnd, EXCLUDED)
        {
            mask |= 1 << 9;
        }
        capture("accento-settings-error.bmp");
        let _ = SetWindowTextW(control(hwnd, EXCLUDED), w!(""));
        let mut bounds = RECT::default();
        let _ = GetWindowRect(hwnd, &mut bounds);
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            740,
            650,
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
        click(NAV_FIRST);
        if viewport(hwnd).compact && IsWindowVisible(control(hwnd, NAV_FIRST + 3)).as_bool() {
            mask |= 64;
        }
        let mut save = RECT::default();
        let _ = GetWindowRect(control(hwnd, SAVE), &mut save);
        let mut client = RECT::default();
        let _ = GetClientRect(hwnd, &mut client);
        let mut origin = POINT::default();
        let _ = ClientToScreen(hwnd, &mut origin);
        if save.top >= origin.y && save.bottom <= origin.y + client.bottom {
            mask |= 128;
        }
        capture("accento-settings-compact.bmp");
        send_message(
            hwnd,
            FOCUS_SCROLL,
            WPARAM(control(hwnd, TEST_FIELD).0 as usize),
            LPARAM(0),
        );
        let mut field = RECT::default();
        let mut clip = RECT::default();
        let _ = GetWindowRect(control(hwnd, TEST_FIELD), &mut field);
        let _ = GetWindowRect(control(hwnd, CONTENT), &mut clip);
        if field.top >= clip.top && field.bottom <= clip.bottom {
            mask |= 1 << 10;
        }
        capture("accento-settings-compact-scrolled.bmp");
        let next = GetNextDlgTabItem(hwnd, Some(control(hwnd, NAV_FIRST + 3)), false);
        if next
            .is_ok_and(|next| GetParent(next).is_ok_and(|parent| parent == control(hwnd, CONTENT)))
        {
            mask |= 1 << 11;
        }
        let mut sections = true;
        for i in 0..4 {
            click(NAV_FIRST + i);
            for (id, section) in [
                (DELAY, Section::Writing),
                (LANG_FIRST, Section::Languages),
                (THEME_FIRST, Section::Appearance),
                (EXCLUDED, Section::Applications),
            ] {
                sections &= IsWindowVisible(control(hwnd, id)).as_bool()
                    == (Section::from_index(i as usize) == section);
            }
            capture(&format!("accento-settings-compact-{i}.bmp"));
        }
        if sections {
            mask |= 1 << 12;
        }
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            bounds.right - bounds.left,
            bounds.bottom - bounds.top,
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
        APP.with(|a| {
            let mut a = a.borrow_mut();
            let a = a.as_mut().unwrap();
            a.draft = original.clone();
            a.section = Section::Writing;
            a.scroll = 0.;
            a.dirty = false;
            a.validation_error = None;
            sync_names(a);
        });
        render::settings_frame(hwnd, &original);
        layout(hwnd);
        if !IsWindowEnabled(control(hwnd, SAVE)).as_bool() {
            mask |= 1 << 13;
        }
        let _ = ShowWindow(hwnd, SW_HIDE);
        mask
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compact_reflows_without_scaling_and_footer_stays_visible() {
        for (width, height) in [(1000., 700.), (720., 780.), (640., 450.)] {
            let v = Layout::new(width, height);
            assert_eq!(v.compact, width < 900.);
            assert_eq!(v.bounds(SAVE).unwrap().0[3], 40.);
            let [_, y, _, h] = v.bounds(SAVE).unwrap().0;
            assert!(y + h <= height);
            for i in 0..4 {
                let [x, _, w, _] = v.bounds(NAV_FIRST + i).unwrap().0;
                assert!(x + w <= width);
            }
        }
    }
    #[test]
    fn scrolling_can_reveal_every_body_control_without_covering_footer() {
        let v = Layout::new(720., 450.);
        for section in [
            Section::Writing,
            Section::Languages,
            Section::Appearance,
            Section::Applications,
        ] {
            for id in [
                DELAY,
                MINUS,
                PLUS,
                TEST_FIELD,
                LANG_FIRST,
                213,
                THEME_FIRST,
                232,
                TRANSPARENCY,
                EXCLUDED,
            ] {
                if belongs(id, section) {
                    let ([_, y, _, h], _) = v.bounds(id).unwrap();
                    assert!(y + h - v.scroll_max(section) <= v.footer - 12.);
                }
            }
        }
    }
}
