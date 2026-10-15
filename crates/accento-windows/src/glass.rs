//! Shared optical treatment for the native picker and settings controls.
use crate::render::rgba;
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
    pub surface: D2D1_COLOR_F,
    pub separator: D2D1_COLOR_F,
    pub selection: D2D1_COLOR_F,
    pub error: D2D1_COLOR_F,
}
impl Palette {
    pub fn new(dark: bool) -> Self {
        if dark {
            Self {
                dark,
                canvas: rgba(0.086, 0.102, 0.133, 1.),
                ink: rgba(0.96, 0.97, 0.99, 1.),
                secondary: rgba(0.64, 0.69, 0.77, 1.),
                tertiary: rgba(0.53, 0.58, 0.66, 1.),
                accent: rgba(0.54, 0.71, 1., 1.),
                inset: rgba(0.078, 0.102, 0.137, 1.),
                surface: rgba(0.125, 0.149, 0.188, 1.),
                separator: rgba(0.84, 0.89, 1., 0.094),
                selection: rgba(0.48, 0.66, 1., 0.11),
                error: rgba(1., 0.61, 0.67, 1.),
            }
        } else {
            Self {
                dark,
                canvas: rgba(0.969, 0.973, 0.980, 1.),
                ink: rgba(0.12, 0.15, 0.20, 1.),
                secondary: rgba(0.39, 0.44, 0.51, 1.),
                tertiary: rgba(0.486, 0.529, 0.596, 1.),
                accent: rgba(0.153, 0.361, 0.867, 1.),
                inset: rgba(0.941, 0.949, 0.965, 1.),
                surface: rgba(1., 1., 1., 1.),
                separator: rgba(0.141, 0.208, 0.314, 0.086),
                selection: rgba(0.2, 0.42, 0.91, 0.078),
                error: rgba(0.71, 0.176, 0.267, 1.),
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
