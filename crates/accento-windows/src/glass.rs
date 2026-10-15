//! Shared optical treatment for the native picker and settings controls.
use crate::render::{rect, rgba};
use std::cell::RefCell;
use windows::{
    Win32::{
        Foundation::*,
        Graphics::{
            Direct2D::{Common::*, *},
            DirectWrite::*,
            Dxgi::Common::*,
            Gdi::*,
        },
    },
    core::{Interface, Result, w},
};
use windows_numerics::{Matrix3x2, Vector2};

#[derive(Clone, Copy)]
pub struct Palette {
    pub dark: bool,
    pub canvas: D2D1_COLOR_F,
    pub ink: D2D1_COLOR_F,
    pub secondary: D2D1_COLOR_F,
    pub tertiary: D2D1_COLOR_F,
    pub accent: D2D1_COLOR_F,
    pub inset: D2D1_COLOR_F,
}
impl Palette {
    pub fn new(dark: bool) -> Self {
        if dark {
            Self {
                dark,
                canvas: rgba(0.075, 0.085, 0.115, 1.),
                ink: rgba(0.96, 0.97, 0.99, 1.),
                secondary: rgba(0.64, 0.69, 0.77, 1.),
                tertiary: rgba(0.45, 0.51, 0.60, 1.),
                accent: rgba(0.43, 0.66, 1., 1.),
                inset: rgba(0.09, 0.11, 0.16, 1.),
            }
        } else {
            Self {
                dark,
                canvas: rgba(0.925, 0.94, 0.965, 1.),
                ink: rgba(0.12, 0.15, 0.20, 1.),
                secondary: rgba(0.39, 0.44, 0.51, 1.),
                tertiary: rgba(0.55, 0.59, 0.65, 1.),
                accent: rgba(0.16, 0.37, 0.82, 1.),
                inset: rgba(0.955, 0.966, 0.985, 1.),
            }
        }
    }
}

pub struct Painter {
    pub target: ID2D1RenderTarget,
    write: IDWriteFactory,
}
impl Painter {
    pub fn new(target: ID2D1RenderTarget, write: IDWriteFactory) -> Self {
        Self { target, write }
    }
    pub fn fill(&self, area: D2D_RECT_F, radius: f32, color: D2D1_COLOR_F) -> Result<()> {
        unsafe {
            let brush = self.target.CreateSolidColorBrush(&color, None)?;
            self.target.FillRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: area,
                    radiusX: radius,
                    radiusY: radius,
                },
                &brush,
            );
        }
        Ok(())
    }
    pub fn stroke(
        &self,
        area: D2D_RECT_F,
        radius: f32,
        color: D2D1_COLOR_F,
        width: f32,
    ) -> Result<()> {
        unsafe {
            let brush = self.target.CreateSolidColorBrush(&color, None)?;
            self.target.DrawRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: area,
                    radiusX: radius,
                    radiusY: radius,
                },
                &brush,
                width,
                None,
            );
        }
        Ok(())
    }
    pub fn gradient(
        &self,
        area: D2D_RECT_F,
        radius: f32,
        colors: &[(f32, D2D1_COLOR_F)],
    ) -> Result<()> {
        unsafe {
            let stops: Vec<_> = colors
                .iter()
                .map(|(position, color)| D2D1_GRADIENT_STOP {
                    position: *position,
                    color: *color,
                })
                .collect();
            let collection = self.target.CreateGradientStopCollection(
                &stops,
                D2D1_GAMMA_2_2,
                D2D1_EXTEND_MODE_CLAMP,
            )?;
            let brush = self.target.CreateLinearGradientBrush(
                &D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                    startPoint: Vector2 {
                        X: area.left,
                        Y: area.top,
                    },
                    endPoint: Vector2 {
                        X: area.right,
                        Y: area.bottom,
                    },
                },
                None,
                &collection,
            )?;
            self.target.FillRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: area,
                    radiusX: radius,
                    radiusY: radius,
                },
                &brush,
            );
        }
        Ok(())
    }
    pub fn text(
        &self,
        value: &str,
        area: D2D_RECT_F,
        size: f32,
        weight: DWRITE_FONT_WEIGHT,
        color: D2D1_COLOR_F,
        center: bool,
    ) -> Result<()> {
        unsafe {
            let format = self.write.CreateTextFormat(
                w!("Segoe UI Variable"),
                None,
                weight,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                size,
                w!("it-IT"),
            )?;
            format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;
            format.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP)?;
            if center {
                format.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER)?;
            }
            let brush = self.target.CreateSolidColorBrush(&color, None)?;
            self.target.DrawText(
                &value.encode_utf16().collect::<Vec<_>>(),
                &format,
                &area,
                &brush,
                D2D1_DRAW_TEXT_OPTIONS_CLIP,
                DWRITE_MEASURING_MODE_NATURAL,
            );
        }
        Ok(())
    }
    pub fn line(
        &self,
        from: (f32, f32),
        to: (f32, f32),
        color: D2D1_COLOR_F,
        width: f32,
    ) -> Result<()> {
        unsafe {
            let brush = self.target.CreateSolidColorBrush(&color, None)?;
            self.target.DrawLine(
                Vector2 {
                    X: from.0,
                    Y: from.1,
                },
                Vector2 { X: to.0, Y: to.1 },
                &brush,
                width,
                None,
            );
        }
        Ok(())
    }
    pub fn circle(&self, x: f32, y: f32, radius: f32, color: D2D1_COLOR_F) -> Result<()> {
        unsafe {
            let brush = self.target.CreateSolidColorBrush(&color, None)?;
            self.target.FillEllipse(
                &D2D1_ELLIPSE {
                    point: Vector2 { X: x, Y: y },
                    radiusX: radius,
                    radiusY: radius,
                },
                &brush,
            );
        }
        Ok(())
    }
    /// The rims stay neutral. Tint communicates selection rather than decorating every surface.
    #[allow(clippy::too_many_arguments)]
    pub fn card(
        &self,
        area: D2D_RECT_F,
        radius: f32,
        p: Palette,
        selected: bool,
        hovered: bool,
        pressed: bool,
        focused: bool,
    ) -> Result<()> {
        let dy = if pressed { 1.0 } else { 0.0 };
        let a = rect(
            area.left,
            area.top + dy,
            area.right - area.left,
            area.bottom - area.top,
        );
        for (spread, alpha) in [(5., 0.012), (3., 0.018), (1., 0.035)] {
            self.fill(
                rect(
                    a.left - spread,
                    a.top + 3.,
                    a.right - a.left + 2. * spread,
                    a.bottom - a.top + spread,
                ),
                radius + spread,
                rgba(0.02, 0.04, 0.09, if p.dark { alpha * 1.5 } else { alpha }),
            )?;
        }
        let colors = if p.dark {
            [
                (0., rgba(0.25, 0.28, 0.34, 1.)),
                (0.44, rgba(0.17, 0.19, 0.245, 1.)),
                (1., rgba(0.13, 0.15, 0.20, 1.)),
            ]
        } else {
            [
                (0., rgba(1., 1., 1., 1.)),
                (0.45, rgba(0.975, 0.982, 0.998, 1.)),
                (1., rgba(0.875, 0.905, 0.95, 1.)),
            ]
        };
        self.gradient(a, radius, &colors)?;
        if selected {
            self.fill(
                a,
                radius,
                if p.dark {
                    rgba(0.25, 0.47, 0.90, 0.22)
                } else {
                    rgba(0.38, 0.61, 1., 0.15)
                },
            )?;
        }
        if hovered {
            self.fill(
                a,
                radius,
                rgba(1., 1., 1., if p.dark { 0.075 } else { 0.30 }),
            )?;
        }
        if pressed {
            self.fill(a, radius, rgba(0.12, 0.22, 0.38, 0.07))?;
        }
        self.stroke(
            rect(
                a.left + 0.6,
                a.top + 0.6,
                a.right - a.left - 1.2,
                a.bottom - a.top - 1.2,
            ),
            radius - 0.6,
            if p.dark {
                rgba(1., 1., 1., 0.22)
            } else {
                rgba(1., 1., 1., 0.96)
            },
            1.,
        )?;
        self.stroke(
            rect(
                a.left + 1.7,
                a.top + 1.7,
                a.right - a.left - 3.4,
                a.bottom - a.top - 3.4,
            ),
            radius - 1.7,
            if p.dark {
                rgba(0., 0., 0., 0.28)
            } else {
                rgba(0.52, 0.61, 0.74, 0.22)
            },
            0.6,
        )?;
        if selected {
            self.stroke(
                rect(
                    a.left + 0.7,
                    a.top + 0.7,
                    a.right - a.left - 1.4,
                    a.bottom - a.top - 1.4,
                ),
                radius - 0.7,
                if p.dark {
                    rgba(0.48, 0.69, 1., 0.85)
                } else {
                    rgba(0.28, 0.49, 0.89, 0.65)
                },
                1.4,
            )?;
        }
        if focused {
            self.stroke(
                rect(
                    a.left - 2.5,
                    a.top - 2.5,
                    a.right - a.left + 5.,
                    a.bottom - a.top + 5.,
                ),
                radius + 2.5,
                p.accent,
                2.,
            )?;
        }
        Ok(())
    }
    pub fn check(&self, x: f32, y: f32, on: bool, p: Palette) -> Result<()> {
        self.circle(
            x,
            y,
            9.,
            if on {
                p.accent
            } else {
                if p.dark {
                    rgba(1., 1., 1., 0.08)
                } else {
                    rgba(0.34, 0.42, 0.55, 0.07)
                }
            },
        )?;
        if on {
            self.line((x - 3.5, y), (x - 0.5, y + 3.), rgba(1., 1., 1., 1.), 1.6)?;
            self.line(
                (x - 0.5, y + 3.),
                (x + 4., y - 3.),
                rgba(1., 1., 1., 1.),
                1.6,
            )?;
        }
        Ok(())
    }
}

struct DcPainter {
    target: ID2D1DCRenderTarget,
    painter: Painter,
}
pub fn release_dc() {
    DC.with(|dc| {
        dc.borrow_mut().take();
    });
}
thread_local! {static DC:RefCell<Option<DcPainter>>=const{RefCell::new(None)};}
pub fn draw_dc(
    hdc: HDC,
    bounds: RECT,
    dpi: f32,
    offset: (f32, f32),
    draw: impl FnOnce(&Painter) -> Result<()>,
) -> Result<()> {
    DC.with(|slot| {
        let mut slot = slot.borrow_mut();
        unsafe {
            if slot.is_none() {
                let factory: ID2D1Factory =
                    D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
                let target = factory.CreateDCRenderTarget(&D2D1_RENDER_TARGET_PROPERTIES {
                    pixelFormat: D2D1_PIXEL_FORMAT {
                        format: DXGI_FORMAT_B8G8R8A8_UNORM,
                        alphaMode: D2D1_ALPHA_MODE_IGNORE,
                    },
                    ..Default::default()
                })?;
                let painter = Painter::new(
                    target.cast()?,
                    DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?,
                );
                *slot = Some(DcPainter { target, painter });
            }
            let d = slot.as_ref().unwrap();
            d.target.BindDC(hdc, &bounds)?;
            d.target.SetDpi(dpi, dpi);
            d.target.SetTransform(&Matrix3x2 {
                M11: 1.,
                M12: 0.,
                M21: 0.,
                M22: 1.,
                M31: offset.0,
                M32: offset.1,
            });
            d.target
                .SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
            d.target.BeginDraw();
            let result = draw(&d.painter);
            let end = d.target.EndDraw(None, None);
            result.and(end)
        }
    })
}
