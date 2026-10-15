use crate::preferences::{Preferences, Theme};
use accento_core::View;
use windows::{
    Win32::{
        Foundation::*,
        Graphics::{
            Direct2D::{Common::*, *},
            DirectWrite::*,
            Dwm::*,
            Dxgi::Common::*,
            Gdi::*,
        },
        System::Registry::*,
        UI::{Accessibility::*, Controls::MARGINS, WindowsAndMessaging::*},
    },
    core::{Interface, Result},
};

pub fn registry_number(path: &str, name: &str, fallback: u32) -> u32 {
    let path: Vec<_> = path.encode_utf16().chain(Some(0)).collect();
    let name: Vec<_> = name.encode_utf16().chain(Some(0)).collect();
    let mut value = fallback;
    let mut size = 4;
    unsafe {
        let _ = RegGetValueW(
            HKEY_CURRENT_USER,
            windows::core::PCWSTR(path.as_ptr()),
            windows::core::PCWSTR(name.as_ptr()),
            RRF_RT_REG_DWORD,
            None,
            Some((&mut value as *mut u32).cast()),
            Some(&mut size),
        );
    }
    value
}
pub fn dark(p: &Preferences) -> bool {
    match p.theme {
        Theme::Dark => true,
        Theme::Light => false,
        Theme::System => {
            registry_number(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                "AppsUseLightTheme",
                1,
            ) == 0
        }
    }
}
pub fn high_contrast() -> bool {
    unsafe {
        let mut hc = HIGHCONTRASTW {
            cbSize: size_of::<HIGHCONTRASTW>() as u32,
            ..Default::default()
        };
        let _ = SystemParametersInfoW(
            SPI_GETHIGHCONTRAST,
            hc.cbSize,
            Some((&mut hc as *mut HIGHCONTRASTW).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
        hc.dwFlags.0 & HCF_HIGHCONTRASTON.0 != 0
    }
}
pub fn text_scale() -> f32 {
    registry_number("Software\\Microsoft\\Accessibility", "TextScaleFactor", 100).clamp(100, 225)
        as f32
        / 100.0
}
pub fn glass(hwnd: HWND, p: &Preferences) -> bool {
    unsafe {
        let enabled = p.transparency
            && !high_contrast()
            && registry_number(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                "EnableTransparency",
                1,
            ) != 0;
        let backdrop = if enabled {
            DWMSBT_TRANSIENTWINDOW
        } else {
            DWMSBT_NONE
        };
        let result = DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            (&backdrop as *const DWM_SYSTEMBACKDROP_TYPE).cast(),
            4,
        );
        let corner = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            (&corner as *const DWM_WINDOW_CORNER_PREFERENCE).cast(),
            4,
        );
        let dark = i32::from(dark(p));
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&dark as *const i32).cast(),
            4,
        );
        let margins = if enabled && result.is_ok() {
            MARGINS {
                cxLeftWidth: -1,
                ..Default::default()
            }
        } else {
            MARGINS::default()
        };
        let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);
        enabled && result.is_ok()
    }
}

pub fn settings_frame(hwnd: HWND, p: &Preferences) {
    unsafe {
        // GDI text fields and owner-drawn controls require an opaque composition target.
        let dark = i32::from(dark(p));
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            (&dark as *const i32).cast(),
            4,
        );
        let corner = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            (&corner as *const DWM_WINDOW_CORNER_PREFERENCE).cast(),
            4,
        );
    }
}

pub struct Renderer {
    target: ID2D1HwndRenderTarget,
    write: IDWriteFactory,
    pub size: (u32, u32),
}

pub(crate) fn rgba(r: f32, g: f32, b: f32, a: f32) -> D2D1_COLOR_F {
    D2D1_COLOR_F { r, g, b, a }
}
pub(crate) fn rect(x: f32, y: f32, w: f32, h: f32) -> D2D_RECT_F {
    D2D_RECT_F {
        left: x,
        top: y,
        right: x + w,
        bottom: y + h,
    }
}
fn system_color(index: SYS_COLOR_INDEX) -> D2D1_COLOR_F {
    let c = unsafe { GetSysColor(index) };
    rgba(
        (c & 255) as f32 / 255.,
        ((c >> 8) & 255) as f32 / 255.,
        ((c >> 16) & 255) as f32 / 255.,
        1.,
    )
}

impl Renderer {
    pub fn new(hwnd: HWND, width: u32, height: u32, dpi: f32) -> Result<Self> {
        unsafe {
            let factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
            let target = factory.CreateHwndRenderTarget(
                &D2D1_RENDER_TARGET_PROPERTIES {
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                    },
                    dpiX: dpi,
                    dpiY: dpi,
                    ..Default::default()
                },
                &D2D1_HWND_RENDER_TARGET_PROPERTIES {
                    hwnd,
                    pixelSize: D2D_SIZE_U { width, height },
                    presentOptions: D2D1_PRESENT_OPTIONS_NONE,
                },
            )?;
            target.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
            let write = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
            Ok(Self {
                target,
                write,
                size: (width, height),
            })
        }
    }
    pub fn resize(&mut self, width: u32, height: u32, dpi: f32) -> Result<()> {
        unsafe {
            self.target.SetDpi(dpi, dpi);
            self.target.Resize(&D2D_SIZE_U { width, height })?;
        }
        self.size = (width, height);
        Ok(())
    }
    pub fn picker(&self, view: &View, p: &Preferences, material: bool, dpi: f32) -> Result<()> {
        use crate::glass::{Painter, Palette};
        let factor = text_scale();
        let width = self.size.0 as f32 * 96. / dpi;
        let height = self.size.1 as f32 * 96. / dpi;
        let dark = dark(p);
        let contrast = high_contrast();
        let colors = Palette::new(dark);
        let g = Painter::new(self.target.cast()?, self.write.clone());
        unsafe {
            self.target.BeginDraw();
            self.target.Clear(Some(&rgba(0., 0., 0., 0.)));
        }
        let result = (|| {
            let root = rect(0., 0., width, height);
            if contrast {
                g.fill(root, 24., system_color(COLOR_WINDOW))?;
            } else {
                let opacity = if material { 0.86 } else { 1. };
                g.gradient(
                    root,
                    25.,
                    &if dark {
                        [
                            (0., rgba(0.19, 0.22, 0.28, opacity)),
                            (0.5, rgba(0.105, 0.13, 0.18, opacity)),
                            (1., rgba(0.07, 0.09, 0.14, opacity)),
                        ]
                    } else {
                        [
                            (0., rgba(0.99, 1., 1., opacity)),
                            (0.45, rgba(0.92, 0.95, 1., opacity)),
                            (1., rgba(0.83, 0.88, 0.95, opacity)),
                        ]
                    },
                )?;
                g.stroke(
                    rect(0.7, 0.7, width - 1.4, height - 1.4),
                    24.3,
                    if dark {
                        rgba(1., 1., 1., 0.26)
                    } else {
                        rgba(1., 1., 1., 0.94)
                    },
                    1.,
                )?;
                g.stroke(
                    rect(1.8, 1.8, width - 3.6, height - 3.6),
                    23.2,
                    if dark {
                        rgba(0., 0., 0., 0.20)
                    } else {
                        rgba(0.40, 0.49, 0.65, 0.12)
                    },
                    0.7,
                )?;
            }
            let ink = if contrast {
                system_color(COLOR_WINDOWTEXT)
            } else {
                colors.ink
            };
            let muted = if contrast { ink } else { colors.secondary };
            g.text(
                &format!(
                    "Varianti di {}",
                    if view.candidates.first().is_some_and(|c| c.is_uppercase()) {
                        view.base.to_uppercase().to_string()
                    } else {
                        view.base.to_string()
                    }
                ),
                rect(18., 7., width - 36., 22. * factor),
                11. * factor,
                DWRITE_FONT_WEIGHT_MEDIUM,
                muted,
                false,
            )?;
            let cell = 64. * factor;
            let gap = 12. * factor;
            let step = cell + gap;
            let visible = ((width - 24. + gap) / step).floor().max(1.) as usize;
            let first = view
                .selected
                .map(|s| s.saturating_sub(visible - 1))
                .unwrap_or(0);
            let count = view.candidates.len().min(visible);
            let start = (width - (count as f32 * step - gap)) / 2.;
            for (index, c) in view.candidates.iter().enumerate().skip(first).take(visible) {
                let x = start + (index - first) as f32 * step;
                let box_rect = rect(x, 34. * factor, cell, cell);
                let selected = view.selected == Some(index);
                if contrast {
                    g.fill(
                        box_rect,
                        14.,
                        if selected {
                            system_color(COLOR_HIGHLIGHT)
                        } else {
                            system_color(COLOR_WINDOW)
                        },
                    )?;
                    g.stroke(box_rect, 14., ink, 1.)?;
                } else {
                    g.card(
                        box_rect,
                        16. * factor,
                        colors,
                        selected,
                        false,
                        false,
                        false,
                    )?;
                }
                g.text(
                    &c.to_string(),
                    rect(x, 34. * factor, cell, cell - 4. * factor),
                    32. * factor,
                    DWRITE_FONT_WEIGHT_MEDIUM,
                    if selected {
                        if contrast {
                            system_color(COLOR_HIGHLIGHTTEXT)
                        } else {
                            colors.accent
                        }
                    } else {
                        ink
                    },
                    true,
                )?;
                if selected && !contrast {
                    g.circle(x + cell / 2., 90. * factor, 1.8 * factor, colors.accent)?;
                }
            }
            g.text(
                "← → scegli",
                rect(18., 110. * factor, width / 2., 22. * factor),
                10.5 * factor,
                DWRITE_FONT_WEIGHT_NORMAL,
                muted,
                false,
            )?;
            g.text(
                "Esc annulla",
                rect(
                    width - 100. * factor,
                    110. * factor,
                    84. * factor,
                    22. * factor,
                ),
                10.5 * factor,
                DWRITE_FONT_WEIGHT_NORMAL,
                muted,
                true,
            )?;
            Ok(())
        })();
        let end = unsafe { self.target.EndDraw(None, None) };
        result.and(end)
    }
}
