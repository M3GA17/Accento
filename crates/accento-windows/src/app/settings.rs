use super::*;
use crate::{
    glass::{self, Painter, Palette},
    render::{rect, rgba},
};
use windows::Win32::Graphics::DirectWrite::*;

pub const WIDTH: f32 = 900.;
pub const HEIGHT: f32 = 936.;
pub const THEME_FIRST: u16 = 230;
const MINUS: u16 = 240;
const PLUS: u16 = 241;
const FOCUS_SCROLL: u32 = WM_APP + 25;

#[derive(Clone)]
struct State {
    prefs: Preferences,
    paused: bool,
    hover: u16,
    notice: String,
    dirty: bool,
    error: Option<String>,
    focus: u16,
}
fn state() -> Option<State> {
    APP.with(|a| {
        a.try_borrow().ok().and_then(|a| {
            a.as_ref().map(|a| State {
                prefs: a.draft.clone(),
                paused: a.paused,
                hover: a.hover,
                notice: unsafe { control_text(a.main, STATUS) },
                dirty: a.dirty,
                error: a.validation_error.clone(),
                focus: unsafe { GetDlgCtrlID(GetFocus()) as u16 },
            })
        })
    })
}
pub fn scale(hwnd: HWND) -> f32 {
    unsafe {
        let mut r = RECT::default();
        let _ = GetClientRect(hwnd, &mut r);
        (r.right as f32 / WIDTH)
            .min(GetDpiForWindow(hwnd) as f32 / 96.)
            .max(0.4)
    }
}

pub unsafe fn build(app: &mut App) {
    unsafe {
        let dpi = GetDpiForWindow(app.main);
        let s = dpi as f32 / 96.;
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
        let style = WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_VSCROLL;
        let _ = AdjustWindowRectExForDpi(&mut r, style, false, WINDOW_EX_STYLE(0), dpi);
        let _ = SetWindowPos(
            app.main,
            None,
            0,
            0,
            (r.right - r.left).min(info.rcWork.right - info.rcWork.left - 48),
            (r.bottom - r.top).min(info.rcWork.bottom - info.rcWork.top - 48),
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
        for (i, name) in ["Italiano", "Français", "Español", "Deutsch"]
            .iter()
            .enumerate()
        {
            tile(
                app,
                LANG_FIRST + i as u16,
                name,
                [28 + i as i32 * 166, 300, 160, 160],
            );
        }
        for (i, name) in ["Automatico", "Chiaro", "Scuro"].iter().enumerate() {
            tile(
                app,
                THEME_FIRST + i as u16,
                name,
                [28 + i as i32 * 166, 514, 160, 160],
            );
        }
        tile(app, TRANSPARENCY, "Liquid Glass", [526, 514, 160, 160]);
        tile(app, MINUS, "Riduci ritardo", [206, 172, 36, 36]);
        tile(app, PLUS, "Aumenta ritardo", [336, 172, 36, 36]);
        field(
            app,
            DELAY,
            &app.draft.delay_ms.to_string(),
            [251, 180, 72, 22],
            ES_NUMBER as u32,
        );
        field(
            app,
            EXCLUDED,
            &app.draft.excluded_apps.join(", "),
            [54, 776, 358, 24],
            ES_AUTOHSCROLL as u32,
        );
        field(
            app,
            TEST_FIELD,
            "",
            [480, 776, 360, 24],
            ES_AUTOHSCROLL as u32,
        );
        tile(app, SAVE, "Salva modifiche", [652, 870, 216, 48]);
        let hidden = child(
            app,
            w!("STATIC"),
            "Tutto pronto",
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
unsafe fn tile(app: &App, id: u16, name: &str, bounds: [i32; 4]) {
    unsafe {
        let [x, y, w, h] = bounds;
        let hwnd = child(
            app,
            w!("BUTTON"),
            name,
            WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW as u32),
            id,
            x,
            y,
            w,
            h,
        );
        let _ = SetWindowSubclass(hwnd, Some(control_proc), 1, 0);
    }
}
unsafe fn field(app: &App, id: u16, value: &str, bounds: [i32; 4], style: u32) {
    unsafe {
        let [x, y, w, h] = bounds;
        let hwnd = child(
            app,
            w!("EDIT"),
            value,
            WS_TABSTOP | WINDOW_STYLE(style),
            id,
            x,
            y,
            w,
            h,
        );
        send_message(
            hwnd,
            EM_SETMARGINS,
            WPARAM((EC_LEFTMARGIN | EC_RIGHTMARGIN) as usize),
            LPARAM(0),
        );
        let _ = SetWindowSubclass(hwnd, Some(control_proc), 1, 0);
    }
}

pub unsafe fn layout(hwnd: HWND) {
    unsafe {
        let Some((controls, scroll)) = APP.with(|a| {
            a.try_borrow()
                .ok()
                .and_then(|a| a.as_ref().map(|a| (a.controls.borrow().clone(), a.scroll)))
        }) else {
            return;
        };
        let s = scale(hwnd);
        let mut client = RECT::default();
        let _ = GetClientRect(hwnd, &mut client);
        let page = client.bottom as f32 / s;
        let scroll = scroll.min((HEIGHT - page).max(0.));
        APP.with(|a| {
            if let Ok(mut a) = a.try_borrow_mut()
                && let Some(a) = a.as_mut()
            {
                a.scroll = scroll;
            }
        });
        SetScrollInfo(
            hwnd,
            SB_VERT,
            &SCROLLINFO {
                cbSize: size_of::<SCROLLINFO>() as u32,
                fMask: SIF_RANGE | SIF_PAGE | SIF_POS,
                nMin: 0,
                nMax: HEIGHT as i32 - 1,
                nPage: page as u32,
                nPos: scroll as i32,
                ..Default::default()
            },
            true,
        );
        for (window, [x, y, w, h], _) in controls {
            let window = HWND(window as _);
            if GetDlgCtrlID(window) == STATUS as i32 {
                continue;
            }
            let _ = SetWindowPos(
                window,
                None,
                (x as f32 * s).round() as i32,
                ((y as f32 - scroll) * s).round() as i32,
                (w as f32 * s).round() as i32,
                (h as f32 * s).round() as i32,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
        let _ = RedrawWindow(Some(hwnd), None, None, RDW_INVALIDATE | RDW_ALLCHILDREN);
    }
}
pub unsafe fn paint(hwnd: HWND, dc: HDC) {
    unsafe {
        let Some(st) = state() else { return };
        let scroll = APP.with(|a| a.borrow().as_ref().unwrap().scroll);
        let mut r = RECT::default();
        let _ = GetClientRect(hwnd, &mut r);
        let s = scale(hwnd);
        let _ = glass::draw_dc(dc, r, 96. * s, (0., -scroll), |g| {
            canvas(g, &st, scroll, r.bottom as f32 / s)
        });
    }
}
fn canvas(g: &Painter, st: &State, scroll: f32, viewport: f32) -> windows::core::Result<()> {
    let p = Palette::new(render::dark(&st.prefs));
    unsafe {
        g.target.Clear(Some(&p.canvas));
    }
    g.fill(rect(0., scroll, WIDTH, viewport), 0., p.canvas)?;
    // Ambient light stays in the header, leaving the square option tiles visually quiet.
    g.gradient(
        rect(0., 0., WIDTH, 116.),
        0.,
        &[
            (
                0.,
                if p.dark {
                    rgba(0.16, 0.19, 0.29, 1.)
                } else {
                    rgba(0.84, 0.90, 1., 1.)
                },
            ),
            (0.65, p.canvas),
            (1., p.canvas),
        ],
    )?;
    g.card(rect(32., 27., 54., 54.), 16., p, false, false, false, false)?;
    g.text(
        "á",
        rect(32., 24., 54., 54.),
        30.,
        DWRITE_FONT_WEIGHT_SEMI_BOLD,
        p.ink,
        true,
    )?;
    g.text(
        "Accento",
        rect(104., 22., 410., 43.),
        34.,
        DWRITE_FONT_WEIGHT_SEMI_BOLD,
        p.ink,
        false,
    )?;
    g.text(
        "Un piccolo gesto. La lettera giusta.",
        rect(106., 66., 470., 24.),
        14.,
        DWRITE_FONT_WEIGHT_NORMAL,
        p.secondary,
        false,
    )?;
    g.fill(
        rect(714., 40., 150., 32.),
        16.,
        if p.dark {
            rgba(0.31, 0.69, 0.48, 0.1)
        } else {
            rgba(1., 1., 1., 0.45)
        },
    )?;
    g.circle(
        732.,
        56.,
        3.5,
        if st.paused {
            p.tertiary
        } else {
            rgba(0.19, 0.64, 0.40, 1.)
        },
    )?;
    g.text(
        if st.paused {
            "In pausa"
        } else {
            "Pronto a scrivere"
        },
        rect(744., 41., 116., 30.),
        12.,
        DWRITE_FONT_WEIGHT_MEDIUM,
        p.secondary,
        false,
    )?;
    g.line(
        (32., 110.),
        (868., 110.),
        if p.dark {
            rgba(1., 1., 1., 0.06)
        } else {
            rgba(0.40, 0.50, 0.65, 0.10)
        },
        1.,
    )?;
    for x in [32., 458.] {
        g.card(
            rect(x, 132., 410., 112.),
            22.,
            p,
            false,
            false,
            false,
            false,
        )?;
    }
    g.text(
        "Il tuo ritmo",
        rect(52., 146., 160., 23.),
        17.,
        DWRITE_FONT_WEIGHT_SEMI_BOLD,
        p.ink,
        false,
    )?;
    g.text(
        "Ritardo di apertura",
        rect(52., 178., 150., 23.),
        13.,
        DWRITE_FONT_WEIGHT_NORMAL,
        p.secondary,
        false,
    )?;
    g.text(
        "Da 100 a 1500 ms",
        rect(52., 206., 220., 19.),
        11.,
        DWRITE_FONT_WEIGHT_NORMAL,
        p.tertiary,
        false,
    )?;
    g.fill(rect(246., 170., 82., 40.), 12., p.inset)?;
    g.stroke(
        rect(246.5, 170.5, 81., 39.),
        11.5,
        if p.dark {
            rgba(1., 1., 1., 0.10)
        } else {
            rgba(0.40, 0.51, 0.69, 0.14)
        },
        1.,
    )?;
    g.text(
        "ms",
        rect(380., 175., 38., 27.),
        12.,
        DWRITE_FONT_WEIGHT_NORMAL,
        p.tertiary,
        false,
    )?;
    g.text(
        "Un gesto naturale",
        rect(480., 146., 350., 25.),
        17.,
        DWRITE_FONT_WEIGHT_SEMI_BOLD,
        p.ink,
        false,
    )?;
    keycap(g, rect(480., 181., 36., 36.), "e", p)?;
    g.text(
        "+",
        rect(523., 184., 20., 26.),
        16.,
        DWRITE_FONT_WEIGHT_NORMAL,
        p.tertiary,
        true,
    )?;
    keycap(g, rect(550., 181., 102., 36.), "Spazio", p)?;
    g.text(
        "Scegli, poi rilascia.",
        rect(674., 184., 178., 28.),
        12.,
        DWRITE_FONT_WEIGHT_NORMAL,
        p.secondary,
        false,
    )?;
    g.text(
        "Le tue lingue",
        rect(36., 262., 400., 28.),
        20.,
        DWRITE_FONT_WEIGHT_SEMI_BOLD,
        p.ink,
        false,
    )?;
    g.text(
        "Aspetto",
        rect(36., 476., 400., 28.),
        20.,
        DWRITE_FONT_WEIGHT_SEMI_BOLD,
        p.ink,
        false,
    )?;
    g.text(
        "Sempre nell’ordine giusto",
        rect(714., 330., 156., 24.),
        12.,
        DWRITE_FONT_WEIGHT_SEMI_BOLD,
        p.secondary,
        false,
    )?;
    g.text(
        "Le varianti si combinano.",
        rect(714., 360., 160., 22.),
        11.5,
        DWRITE_FONT_WEIGHT_NORMAL,
        p.tertiary,
        false,
    )?;
    g.text(
        "Nessun duplicato.",
        rect(714., 381., 156., 22.),
        11.5,
        DWRITE_FONT_WEIGHT_NORMAL,
        p.tertiary,
        false,
    )?;
    g.text(
        "A modo tuo",
        rect(714., 544., 156., 24.),
        12.,
        DWRITE_FONT_WEIGHT_SEMI_BOLD,
        p.secondary,
        false,
    )?;
    g.text(
        "Luce, profondità, contrasto.",
        rect(714., 574., 160., 22.),
        11.,
        DWRITE_FONT_WEIGHT_NORMAL,
        p.tertiary,
        false,
    )?;
    g.text(
        "Anche senza trasparenza.",
        rect(714., 595., 160., 22.),
        11.,
        DWRITE_FONT_WEIGHT_NORMAL,
        p.tertiary,
        false,
    )?;
    for x in [32., 458.] {
        g.card(
            rect(x, 704., 410., 128.),
            22.,
            p,
            false,
            false,
            false,
            false,
        )?;
        g.fill(rect(x + 16., 764., 378., 43.), 13., p.inset)?;
        g.stroke(
            rect(x + 16.5, 764.5, 377., 42.),
            12.5,
            if p.dark {
                rgba(1., 1., 1., 0.11)
            } else {
                rgba(0.38, 0.48, 0.64, 0.15)
            },
            1.,
        )?;
    }
    g.text(
        "App escluse",
        rect(54., 716., 350., 27.),
        17.,
        DWRITE_FONT_WEIGHT_SEMI_BOLD,
        p.ink,
        false,
    )?;
    g.text(
        "Nomi completi, separati da virgola.",
        rect(54., 744., 360., 21.),
        12.,
        DWRITE_FONT_WEIGHT_NORMAL,
        p.secondary,
        false,
    )?;
    g.text(
        "Prova il tuo prossimo accento",
        rect(480., 716., 360., 27.),
        17.,
        DWRITE_FONT_WEIGHT_SEMI_BOLD,
        p.ink,
        false,
    )?;
    g.text(
        "Tieni e · Spazio · scegli con le frecce · rilascia",
        rect(480., 744., 368., 21.),
        11.,
        DWRITE_FONT_WEIGHT_NORMAL,
        p.secondary,
        false,
    )?;
    for (id, area) in [
        (DELAY, rect(246., 170., 82., 40.)),
        (EXCLUDED, rect(48., 764., 378., 43.)),
        (TEST_FIELD, rect(474., 764., 378., 43.)),
    ] {
        let invalid = st.error.as_ref().is_some_and(|e| {
            (id == DELAY && e.contains("ritardo")) || (id == EXCLUDED && e.contains("esclusione"))
        });
        if st.focus == id || invalid {
            g.stroke(
                area,
                12.,
                if invalid {
                    rgba(0.90, 0.28, 0.32, 1.)
                } else {
                    p.accent
                },
                1.6,
            )?;
        }
    }
    g.line(
        (32., 854.),
        (868., 854.),
        if p.dark {
            rgba(1., 1., 1., 0.07)
        } else {
            rgba(0.40, 0.50, 0.65, 0.12)
        },
        1.,
    )?;
    let error = st.error.as_ref().map(|e| {
        if e.contains("ritardo") {
            "Scegli un ritardo fra 100 e 1500 ms."
        } else if e.contains("esclusione") {
            "Usa un nome completo, per esempio notepad.exe."
        } else {
            "Non è stato possibile salvare le impostazioni."
        }
    });
    g.text(
        if let Some(error) = error {
            error
        } else if st.dirty {
            "Modifiche da salvare"
        } else {
            &st.notice
        },
        rect(36., 870., 560., 23.),
        13.,
        DWRITE_FONT_WEIGHT_MEDIUM,
        if st.error.is_some() {
            rgba(0.90, 0.28, 0.32, 1.)
        } else {
            p.secondary
        },
        false,
    )?;
    g.text(
        "Solo sul tuo dispositivo. Nessun account.",
        rect(36., 895., 560., 20.),
        11.,
        DWRITE_FONT_WEIGHT_NORMAL,
        p.tertiary,
        false,
    )?;
    Ok(())
}
fn keycap(
    g: &Painter,
    r: windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F,
    text: &str,
    p: Palette,
) -> windows::core::Result<()> {
    g.card(r, 9., p, false, false, false, false)?;
    g.text(text, r, 14., DWRITE_FONT_WEIGHT_MEDIUM, p.ink, true)
}

pub unsafe fn draw_item(item: &DRAWITEMSTRUCT) -> bool {
    unsafe {
        let Some(st) = state() else { return true };
        let control = item.CtlID as u16;
        let p = Palette::new(render::dark(&st.prefs));
        let dpi = 96. * scale(GetParent(item.hwndItem).unwrap_or_default());
        let width = (item.rcItem.right - item.rcItem.left) as f32 * 96. / dpi;
        let height = (item.rcItem.bottom - item.rcItem.top) as f32 * 96. / dpi;
        let selected = match control {
            LANG_FIRST..=213 => st
                .prefs
                .languages
                .iter()
                .any(|l| l == ["it", "fr", "es", "de"][(control - LANG_FIRST) as usize]),
            THEME_FIRST..=232 => match st.prefs.theme {
                Theme::System => control == 230,
                Theme::Light => control == 231,
                Theme::Dark => control == 232,
            },
            TRANSPARENCY => st.prefs.transparency,
            _ => false,
        };
        let hovered = st.hover == control;
        let pressed = item.itemState.0 & ODS_SELECTED.0 != 0;
        let focused =
            item.itemState.0 & ODS_FOCUS.0 != 0 && item.itemState.0 & ODS_NOFOCUSRECT.0 == 0;
        let _ = glass::draw_dc(item.hDC, item.rcItem, dpi, (0., 0.), |g| {
            g.fill(rect(0., 0., width, height), 0., p.canvas)?;
            if control == SAVE {
                g.card(
                    rect(3., 3., width - 6., height - 6.),
                    14.,
                    p,
                    true,
                    hovered,
                    pressed,
                    focused,
                )?;
                let shift = if pressed {
                    -0.07
                } else if hovered {
                    0.055
                } else {
                    0.
                };
                g.gradient(
                    rect(4., 4., width - 8., height - 8.),
                    13.,
                    &[
                        (
                            0.,
                            rgba(0.38 + shift, 0.57 + shift, (0.98 + shift).min(1.), 1.),
                        ),
                        (0.5, rgba(0.24 + shift, 0.43 + shift, 0.84 + shift, 1.)),
                        (1., rgba(0.16 + shift, 0.32 + shift, 0.72 + shift, 1.)),
                    ],
                )?;
                return g.text(
                    "Salva modifiche",
                    rect(6., 4., width - 12., height - 8.),
                    14.,
                    DWRITE_FONT_WEIGHT_SEMI_BOLD,
                    rgba(1., 1., 1., 1.),
                    true,
                );
            }
            let small = matches!(control, MINUS | PLUS);
            if small {
                g.fill(
                    rect(0., 0., width, height),
                    0.,
                    if p.dark {
                        rgba(0.177, 0.198, 0.251, 1.)
                    } else {
                        rgba(0.977, 0.984, 0.998, 1.)
                    },
                )?;
            }
            let inset = if small { 3. } else { 8. };
            let area = rect(inset, inset, width - 2. * inset, height - 2. * inset);
            g.card(
                area,
                if small { 9. } else { 22. },
                p,
                selected,
                hovered,
                pressed,
                focused,
            )?;
            if small {
                return g.text(
                    if control == MINUS { "−" } else { "+" },
                    area,
                    19.,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    p.ink,
                    true,
                );
            }
            g.check(width - 30., 30., selected, p)?;
            match control {
                LANG_FIRST..=213 => {
                    let i = (control - LANG_FIRST) as usize;
                    g.text(
                        ["à", "ç", "ñ", "ü"][i],
                        rect(24., 32., width - 48., 58.),
                        40.,
                        DWRITE_FONT_WEIGHT_NORMAL,
                        p.ink,
                        true,
                    )?;
                    g.text(
                        ["Italiano", "Français", "Español", "Deutsch"][i],
                        rect(16., 101., width - 32., 25.),
                        14.,
                        DWRITE_FONT_WEIGHT_SEMI_BOLD,
                        p.ink,
                        true,
                    )?;
                    g.text(
                        ["IT", "FR", "ES", "DE"][i],
                        rect(16., 126., width - 32., 18.),
                        10.,
                        DWRITE_FONT_WEIGHT_MEDIUM,
                        p.tertiary,
                        true,
                    )?;
                }
                THEME_FIRST..=232 => {
                    let i = (control - THEME_FIRST) as usize;
                    let x = (width - 62.) / 2.;
                    g.fill(rect(x, 42., 62., 43.), 9., rgba(0.64, 0.70, 0.80, 0.18))?;
                    g.fill(
                        rect(x + 1., 43., 60., 41.),
                        8.,
                        if i == 2 {
                            rgba(0.12, 0.15, 0.22, 1.)
                        } else {
                            rgba(0.98, 0.99, 1., 1.)
                        },
                    )?;
                    if i == 0 {
                        g.fill(rect(x + 31., 43., 30., 41.), 7., rgba(0.14, 0.18, 0.26, 1.))?;
                    }
                    g.fill(
                        rect(x + 6., 49., 13., 29.),
                        4.,
                        rgba(0.52, 0.64, 0.83, 0.24),
                    )?;
                    for row in 0..3 {
                        g.fill(
                            rect(x + 24., 50. + row as f32 * 9., 29., 4.),
                            2.,
                            rgba(0.55, 0.63, 0.76, 0.3),
                        )?;
                    }
                    g.text(
                        ["Automatico", "Chiaro", "Scuro"][i],
                        rect(16., 103., width - 32., 25.),
                        14.,
                        DWRITE_FONT_WEIGHT_SEMI_BOLD,
                        p.ink,
                        true,
                    )?;
                    g.text(
                        ["Segue Windows", "Luce naturale", "Luce soffusa"][i],
                        rect(16., 128., width - 32., 18.),
                        10.,
                        DWRITE_FONT_WEIGHT_NORMAL,
                        p.tertiary,
                        true,
                    )?;
                }
                TRANSPARENCY => {
                    let x = (width - 54.) / 2.;
                    g.fill(
                        rect(x, 52., 54., 30.),
                        15.,
                        if selected { p.accent } else { p.tertiary },
                    )?;
                    g.circle(
                        x + if selected { 39. } else { 15. },
                        67.,
                        12.,
                        rgba(1., 1., 1., 1.),
                    )?;
                    g.text(
                        "Liquid Glass",
                        rect(16., 103., width - 32., 25.),
                        14.,
                        DWRITE_FONT_WEIGHT_SEMI_BOLD,
                        p.ink,
                        true,
                    )?;
                    g.text(
                        if selected {
                            "Trasparenza attiva"
                        } else {
                            "Superficie opaca"
                        },
                        rect(16., 128., width - 32., 18.),
                        10.,
                        DWRITE_FONT_WEIGHT_NORMAL,
                        p.tertiary,
                        true,
                    )?;
                }
                _ => {}
            }
            Ok(())
        });
        true
    }
}

pub unsafe fn command(hwnd: HWND, id: u16, notification: u16) -> bool {
    unsafe {
        if matches!(id, DELAY | EXCLUDED) && notification == EN_CHANGE as u16 {
            APP.with(|a| {
                if let Ok(mut a) = a.try_borrow_mut()
                    && let Some(a) = a.as_mut()
                {
                    a.dirty = true;
                    a.validation_error = None;
                }
            });
            let _ = InvalidateRect(Some(hwnd), None, false);
            return false;
        }
        if !matches!(id,LANG_FIRST..=213|THEME_FIRST..=232|TRANSPARENCY|MINUS|PLUS) {
            return false;
        }
        let changed = APP.with(|a| {
            let mut a = a.borrow_mut();
            let Some(a) = a.as_mut() else { return false };
            match id {
                LANG_FIRST..=213 => {
                    let code = ["it", "fr", "es", "de"][(id - LANG_FIRST) as usize];
                    if let Some(i) = a.draft.languages.iter().position(|l| l == code) {
                        if a.draft.languages.len() == 1 {
                            return false;
                        }
                        a.draft.languages.remove(i);
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
                        current.saturating_sub(50).max(100)
                    } else {
                        (current + 50).min(1500)
                    };
                    let text = wide(&value.to_string());
                    let _ = SetWindowTextW(control(hwnd, DELAY), PCWSTR(text.as_ptr()));
                    a.draft.delay_ms = value;
                }
                _ => {}
            }
            a.dirty = true;
            true
        });
        if changed {
            let prefs = APP.with(|a| {
                let a = a.borrow();
                let a = a.as_ref().unwrap();
                sync_names(a);
                a.draft.clone()
            });
            render::settings_frame(hwnd, &prefs);
            let _ = RedrawWindow(Some(hwnd), None, None, RDW_INVALIDATE | RDW_ALLCHILDREN);
        }
        true
    }
}

unsafe fn sync_names(app: &App) {
    unsafe {
        for (i, (name, code)) in [
            ("Italiano", "it"),
            ("Français", "fr"),
            ("Español", "es"),
            ("Deutsch", "de"),
        ]
        .iter()
        .enumerate()
        {
            let selected = app.draft.languages.iter().any(|l| l == code);
            let value = wide(&format!(
                "{name}, {}",
                if selected {
                    "selezionato"
                } else {
                    "non selezionato"
                }
            ));
            let _ = SetWindowTextW(
                control(app.main, LANG_FIRST + i as u16),
                PCWSTR(value.as_ptr()),
            );
        }
        for (i, name) in ["Automatico", "Chiaro", "Scuro"].iter().enumerate() {
            let selected = match app.draft.theme {
                Theme::System => i == 0,
                Theme::Light => i == 1,
                Theme::Dark => i == 2,
            };
            let value = wide(&format!(
                "{name}, {}",
                if selected {
                    "selezionato"
                } else {
                    "non selezionato"
                }
            ));
            let _ = SetWindowTextW(
                control(app.main, THEME_FIRST + i as u16),
                PCWSTR(value.as_ptr()),
            );
        }
        let value = wide(if app.draft.transparency {
            "Liquid Glass, attivo"
        } else {
            "Liquid Glass, disattivato"
        });
        let _ = SetWindowTextW(control(app.main, TRANSPARENCY), PCWSTR(value.as_ptr()));
    }
}

pub unsafe fn verify_controls(hwnd: HWND) -> u32 {
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOWNORMAL);
        let _ = ShowWindow(hwnd, SW_SHOWNORMAL);
        let original = APP.with(|a| a.borrow().as_ref().unwrap().draft.clone());
        let delay = control_text(hwnd, DELAY);
        let mut mask = 0;
        let click = |id| {
            send_message(control(hwnd, id), BM_CLICK, WPARAM(0), LPARAM(0));
        };
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
        let folder = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        click(THEME_FIRST + 1);
        let light = APP.with(|a| a.borrow().as_ref().unwrap().draft.theme == Theme::Light);
        let _ = capture_window(hwnd, &folder.join("accento-settings-light.bmp"));
        click(THEME_FIRST + 2);
        if light && APP.with(|a| a.borrow().as_ref().unwrap().draft.theme == Theme::Dark) {
            mask |= 4;
        }
        let _ = capture_window(hwnd, &folder.join("accento-settings-dark.bmp"));
        click(TRANSPARENCY);
        if APP.with(|a| !a.borrow().as_ref().unwrap().draft.transparency) {
            mask |= 8;
        }
        click(PLUS);
        if control_text(hwnd, DELAY) == "350" {
            mask |= 16;
        }
        click(MINUS);
        let _ = SetWindowTextW(control(hwnd, DELAY), w!("20"));
        send_message(hwnd, WM_COMMAND, WPARAM(SAVE as usize), LPARAM(0));
        if APP.with(|a| {
            let a = a.borrow();
            let a = a.as_ref().unwrap();
            a.validation_error.is_some() && a.prefs.delay_ms == 300
        }) {
            mask |= 32;
        }
        let mut bounds = RECT::default();
        let _ = GetWindowRect(hwnd, &mut bounds);
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            660,
            600,
            SWP_NOMOVE | SWP_NOACTIVATE | SWP_NOZORDER,
        );
        layout(hwnd);
        let mut info = SCROLLINFO {
            cbSize: size_of::<SCROLLINFO>() as u32,
            fMask: SIF_ALL,
            ..Default::default()
        };
        let _ = GetScrollInfo(hwnd, SB_VERT, &mut info);
        if info.nMax > info.nPage as i32 {
            mask |= 64;
        }
        send_message(
            hwnd,
            FOCUS_SCROLL,
            WPARAM(control(hwnd, SAVE).0 as usize),
            LPARAM(0),
        );
        let mut save = RECT::default();
        let _ = GetWindowRect(control(hwnd, SAVE), &mut save);
        let mut client = RECT::default();
        let _ = GetClientRect(hwnd, &mut client);
        let mut origin = POINT::default();
        let _ = ClientToScreen(hwnd, &mut origin);
        if save.top >= origin.y && save.bottom <= origin.y + client.bottom {
            mask |= 128;
        }
        let _ = capture_window(hwnd, &folder.join("accento-settings-compact.bmp"));
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            bounds.right - bounds.left,
            bounds.bottom - bounds.top,
            SWP_NOMOVE | SWP_NOACTIVATE | SWP_NOZORDER,
        );
        let value = wide(&delay);
        let _ = SetWindowTextW(control(hwnd, DELAY), PCWSTR(value.as_ptr()));
        APP.with(|a| {
            let mut a = a.borrow_mut();
            let a = a.as_mut().unwrap();
            a.draft = original.clone();
            a.scroll = 0.;
            a.dirty = false;
            a.validation_error = None;
            sync_names(a);
        });
        render::settings_frame(hwnd, &original);
        layout(hwnd);
        let _ = ShowWindow(hwnd, SW_HIDE);
        mask
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
            WM_DRAWITEM => {
                let item = &*(lp.0 as *const DRAWITEMSTRUCT);
                Some(LRESULT(draw_item(item) as isize))
            }
            WM_SIZE => {
                layout(hwnd);
                Some(LRESULT(0))
            }
            WM_MOUSEWHEEL | WM_VSCROLL => {
                let s = scale(hwnd);
                let mut r = RECT::default();
                let _ = GetClientRect(hwnd, &mut r);
                let page = r.bottom as f32 / s;
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
                                2 => -page * 0.8,
                                3 => page * 0.8,
                                5 | 4 => {
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
                        a.scroll = (a.scroll + delta).clamp(0., (HEIGHT - page).max(0.));
                    }
                });
                layout(hwnd);
                Some(LRESULT(0))
            }
            FOCUS_SCROLL => {
                let bounds = APP.with(|a| {
                    let a = a.borrow();
                    let a = a.as_ref()?;
                    let controls = a.controls.borrow();
                    controls
                        .iter()
                        .find(|(w, _, _)| *w == wp.0)
                        .map(|(_, r, _)| (*r, a.scroll))
                });
                if let Some(([_, y, _, h], scroll)) = bounds {
                    let mut r = RECT::default();
                    let _ = GetClientRect(hwnd, &mut r);
                    let page = r.bottom as f32 / scale(hwnd);
                    let wanted = if (y as f32) < scroll {
                        y as f32 - 12.
                    } else if (y + h) as f32 > scroll + page {
                        (y + h) as f32 - page + 12.
                    } else {
                        scroll
                    };
                    APP.with(|a| {
                        if let Some(a) = a.borrow_mut().as_mut() {
                            a.scroll = wanted.max(0.);
                        }
                    });
                    layout(hwnd);
                }
                Some(LRESULT(0))
            }
            WM_CTLCOLOREDIT => {
                let p = state().map(|s| Palette::new(render::dark(&s.prefs)))?;
                let dc = HDC(wp.0 as _);
                let color = |c: windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F| {
                    COLORREF(
                        (c.r * 255.) as u32
                            | ((c.g * 255.) as u32) << 8
                            | ((c.b * 255.) as u32) << 16,
                    )
                };
                SetTextColor(dc, color(p.ink));
                SetBkColor(dc, color(p.inset));
                SetDCBrushColor(dc, color(p.inset));
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
        let parent = GetParent(hwnd).unwrap_or_default();
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
        DefSubclassProc(hwnd, msg, wp, lp)
    }
}
