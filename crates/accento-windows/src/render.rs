use crate::{
    picker_material as material,
    preferences::{Preferences, Theme},
};
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
        UI::{Accessibility::*, WindowsAndMessaging::*},
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
        / 100.
}
pub fn glass_enabled(p: &Preferences) -> bool {
    p.transparency
        && !high_contrast()
        && registry_number(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
            "EnableTransparency",
            1,
        ) != 0
}
pub fn settings_frame(hwnd: HWND, p: &Preferences) {
    unsafe {
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
pub(crate) fn system_color(index: SYS_COLOR_INDEX) -> D2D1_COLOR_F {
    let c = unsafe { GetSysColor(index) };
    rgba(
        (c & 255) as f32 / 255.,
        ((c >> 8) & 255) as f32 / 255.,
        ((c >> 16) & 255) as f32 / 255.,
        1.,
    )
}

/// A premultiplied BGRA surface presented with per-pixel alpha. No DWM backdrop
/// is applied to the enclosing rectangle. Backdrop is sampled once per opening;
/// navigating never captures our own keys or writes screen pixels to disk.
pub struct Renderer {
    hwnd: HWND,
    dc: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
    pixels: *mut u8,
    target: ID2D1DCRenderTarget,
    write: IDWriteFactory,
    backdrop: Vec<u8>,
    pub size: (u32, u32),
    dpi: f32,
    sampled: bool,
}
impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.previous);
            let _ = DeleteObject(HGDIOBJ(self.bitmap.0));
            let _ = DeleteDC(self.dc);
        }
    }
}
impl Renderer {
    pub fn new(hwnd: HWND, width: u32, height: u32, dpi: f32, sampled: bool) -> Result<Self> {
        unsafe {
            let factory: ID2D1Factory = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
            let target = factory.CreateDCRenderTarget(&D2D1_RENDER_TARGET_PROPERTIES {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: dpi,
                dpiY: dpi,
                ..Default::default()
            })?;
            let write = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
            let screen = GetDC(None);
            let dc = CreateCompatibleDC(Some(screen));
            if dc.0.is_null() {
                ReleaseDC(None, screen);
                return Err(windows::core::Error::from_thread());
            }
            let info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width as i32,
                    biHeight: -(height as i32),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut pixels = std::ptr::null_mut();
            let bitmap =
                match CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut pixels, None, 0) {
                    Ok(bitmap) => bitmap,
                    Err(e) => {
                        let _ = DeleteDC(dc);
                        ReleaseDC(None, screen);
                        return Err(e);
                    }
                };
            let previous = SelectObject(dc, HGDIOBJ(bitmap.0));
            let mut area = RECT::default();
            let _ = GetWindowRect(hwnd, &mut area);
            let copied = if sampled {
                BitBlt(
                    dc,
                    0,
                    0,
                    width as i32,
                    height as i32,
                    Some(screen),
                    area.left,
                    area.top,
                    SRCCOPY | CAPTUREBLT,
                )
            } else {
                Err(windows::core::Error::empty())
            };
            let _ = GdiFlush();
            ReleaseDC(None, screen);
            let mut backdrop = vec![0; (width * height * 4) as usize];
            let length = backdrop.len();
            if copied.is_ok() {
                backdrop.copy_from_slice(std::slice::from_raw_parts(pixels.cast::<u8>(), length));
            } else {
                backdrop.fill(210);
            }
            let backdrop = material::blur(
                &backdrop,
                width as usize,
                height as usize,
                (3. * dpi / 96.).round().max(1.) as usize,
            );
            let renderer = Self {
                hwnd,
                dc,
                bitmap,
                previous,
                pixels: pixels.cast(),
                target,
                write,
                backdrop,
                size: (width, height),
                dpi,
                sampled,
            };
            renderer.target.BindDC(
                dc,
                &RECT {
                    left: 0,
                    top: 0,
                    right: width as i32,
                    bottom: height as i32,
                },
            )?;
            renderer
                .target
                .SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
            Ok(renderer)
        }
    }
    pub fn resize(&mut self, width: u32, height: u32, dpi: f32) -> Result<()> {
        if self.size != (width, height) || self.dpi != dpi {
            unsafe {
                let _ = ShowWindow(self.hwnd, SW_HIDE);
            }
            *self = Self::new(self.hwnd, width, height, dpi, self.sampled)?;
        }
        Ok(())
    }
    /// Called only by --self-test, after EndDraw. Checks the actual D2D surface,
    /// not just the input to the compositor, and exports its alpha for review.
    pub fn verify_test_surface(&self, folder: &std::path::Path, mode: usize) -> bool {
        let bytes = unsafe {
            std::slice::from_raw_parts(self.pixels, (self.size.0 * self.size.1 * 4) as usize)
        };
        let w = self.size.0 as usize;
        let h = self.size.1 as usize;
        let scale = self.dpi / 96. * text_scale();
        let alpha = |x: usize, y: usize| bytes[(y * w + x) * 4 + 3];
        let corners = [
            alpha(0, 0),
            alpha(w - 1, 0),
            alpha(0, h - 1),
            alpha(w - 1, h - 1),
        ];
        let count = material::visible_count(w as f32, scale);
        let gaps = (0..count.saturating_sub(1)).all(|i| {
            alpha(
                ((76. + i as f32 * 72.) * scale).round() as usize,
                (40. * scale).round() as usize,
            ) <= 16
        });
        let _ = std::fs::write(folder.join(format!("accento-picker-{mode}.bgra")), bytes);
        let _ = std::fs::write(
            folder.join(format!("accento-picker-{mode}.json")),
            format!(
                "{{\"width\":{w},\"height\":{h},\"cornerAlpha\":{corners:?},\"gapsTransparent\":{gaps}}}"
            ),
        );
        corners == [0; 4] && gaps && alpha((40. * scale) as usize, (40. * scale) as usize) > 230
    }
    pub fn picker(&self, view: &View, p: &Preferences, glass: bool, dpi: f32) -> Result<()> {
        use crate::glass::{Painter, Palette};
        let factor = text_scale();
        let glass = glass && glass_enabled(p);
        let scale = dpi / 96.;
        let width = self.size.0 as f32 / scale;
        let visible = material::visible_count(width, factor).min(view.candidates.len());
        let first = view
            .selected
            .map(|s| s.saturating_sub(visible.saturating_sub(1)))
            .unwrap_or(0);
        let contrast = high_contrast();
        let colors = Palette::new(dark(p));
        let pixels = material::pixels(
            self.size.0 as usize,
            self.size.1 as usize,
            scale * factor,
            visible,
            dark(p),
            glass,
            &self.backdrop,
        );
        let g = Painter::new(self.target.cast()?, self.write.clone());
        unsafe {
            self.target.BeginDraw();
            self.target.Clear(Some(&rgba(0., 0., 0., 0.)));
        }
        let result = (|| unsafe {
            let bitmap = self.target.CreateBitmap(
                D2D_SIZE_U {
                    width: self.size.0,
                    height: self.size.1,
                },
                Some(pixels.as_ptr().cast()),
                self.size.0 * 4,
                &D2D1_BITMAP_PROPERTIES {
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                    },
                    dpiX: dpi,
                    dpiY: dpi,
                },
            )?;
            self.target.DrawBitmap(
                &bitmap,
                None,
                1.,
                D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                None,
            );
            for (local, c) in view.candidates.iter().skip(first).take(visible).enumerate() {
                let selected = view.selected == Some(first + local);
                let x = (8. + local as f32 * 72.) * factor;
                let area = rect(x, 8. * factor, 64. * factor, 64. * factor);
                if contrast {
                    g.fill(
                        area,
                        17. * factor,
                        system_color(if selected {
                            COLOR_HIGHLIGHT
                        } else {
                            COLOR_WINDOW
                        }),
                    )?;
                    g.stroke(area, 17. * factor, system_color(COLOR_WINDOWTEXT), 1.)?;
                } else {
                    if selected {
                        g.fill(
                            area,
                            17. * factor,
                            if colors.dark {
                                rgba(0.48, 0.66, 1., 0.16)
                            } else {
                                rgba(0.2, 0.42, 0.92, 0.10)
                            },
                        )?;
                    }
                    g.stroke(
                        rect(x + 0.5 * factor, 8.5 * factor, 63. * factor, 63. * factor),
                        16.5 * factor,
                        if colors.dark {
                            rgba(1., 1., 1., 0.42)
                        } else {
                            rgba(1., 1., 1., 0.91)
                        },
                        factor,
                    )?;
                    g.stroke(
                        rect(x + 1.5 * factor, 9.5 * factor, 61. * factor, 61. * factor),
                        15.5 * factor,
                        if selected {
                            if colors.dark {
                                rgba(0.55, 0.72, 1., 0.65)
                            } else {
                                rgba(0.24, 0.45, 0.93, 0.50)
                            }
                        } else if colors.dark {
                            rgba(0., 0., 0., 0.30)
                        } else {
                            rgba(0.32, 0.39, 0.49, 0.20)
                        },
                        0.7 * factor,
                    )?;
                }
                g.text(
                    &c.to_string(),
                    area,
                    32. * factor,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    if contrast {
                        system_color(if selected {
                            COLOR_HIGHLIGHTTEXT
                        } else {
                            COLOR_WINDOWTEXT
                        })
                    } else if selected {
                        colors.accent
                    } else {
                        colors.ink
                    },
                    true,
                )?;
            }
            Ok(())
        })();
        let end = unsafe { self.target.EndDraw(None, None) };
        result.and(end)?;
        unsafe {
            let _ = GdiFlush();
            let mut area = RECT::default();
            GetWindowRect(self.hwnd, &mut area)?;
            UpdateLayeredWindow(
                self.hwnd,
                None,
                Some(&POINT {
                    x: area.left,
                    y: area.top,
                }),
                Some(&SIZE {
                    cx: self.size.0 as i32,
                    cy: self.size.1 as i32,
                }),
                Some(self.dc),
                Some(&POINT::default()),
                COLORREF(0),
                Some(&BLENDFUNCTION {
                    BlendOp: AC_SRC_OVER as u8,
                    BlendFlags: 0,
                    SourceConstantAlpha: 255,
                    AlphaFormat: AC_SRC_ALPHA as u8,
                }),
                ULW_ALPHA,
            )?;
        }
        Ok(())
    }
}
