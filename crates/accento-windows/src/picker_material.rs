//! Pixel material for the floating keys. No paint ever fills the popup rectangle.
//! Backdrop pixels are sampled only in the popup bounds and never persisted.

pub const CELL: f32 = 64.;
pub const GAP: f32 = 8.;
pub const PADDING: f32 = 8.;
pub const HEIGHT: f32 = 80.;

pub fn width(count: usize) -> f32 {
    count as f32 * (CELL + GAP) + PADDING
}

pub fn visible_count(width: f32, scale: f32) -> usize {
    ((((width + 0.51) / scale - PADDING) / (CELL + GAP)).floor() as usize).max(1)
}

fn distance(x: f32, y: f32, cell: f32, radius: f32) -> f32 {
    let qx = (x - cell / 2.).abs() - (cell / 2. - radius);
    let qy = (y - cell / 2.).abs() - (cell / 2. - radius);
    qx.max(0.).hypot(qy.max(0.)) + qx.max(qy).min(0.) - radius
}

pub fn blur(input: &[u8], width: usize, height: usize, radius: usize) -> Vec<u8> {
    let mut horizontal = vec![0; input.len()];
    let mut output = vec![0; input.len()];
    for y in 0..height {
        for x in 0..width {
            let left = x.saturating_sub(radius);
            let right = (x + radius).min(width - 1);
            for c in 0..3 {
                let sum: u32 = (left..=right)
                    .map(|xx| input[(y * width + xx) * 4 + c] as u32)
                    .sum();
                horizontal[(y * width + x) * 4 + c] = (sum / (right - left + 1) as u32) as u8;
            }
        }
    }
    for y in 0..height {
        let top = y.saturating_sub(radius);
        let bottom = (y + radius).min(height - 1);
        for x in 0..width {
            for c in 0..3 {
                let sum: u32 = (top..=bottom)
                    .map(|yy| horizontal[(yy * width + x) * 4 + c] as u32)
                    .sum();
                output[(y * width + x) * 4 + c] = (sum / (bottom - top + 1) as u32) as u8;
            }
            output[(y * width + x) * 4 + 3] = 255;
        }
    }
    output
}

/// BGRA, premultiplied. Alpha is zero between keys except their short shadows.
pub fn pixels(
    width: usize,
    height: usize,
    scale: f32,
    count: usize,
    dark: bool,
    glass: bool,
    backdrop: &[u8],
) -> Vec<u8> {
    let mut out = vec![0; width * height * 4];
    let cell = CELL * scale;
    let pad = PADDING * scale;
    for y in 0..height {
        for x in 0..width {
            let i = (y * width + x) * 4;
            for key in 0..count {
                let left = pad + key as f32 * (CELL + GAP) * scale;
                let xx = x as f32 + 0.5 - left;
                let yy = y as f32 + 0.5 - pad;
                let d = distance(xx, yy, cell, 17. * scale);
                let coverage = (0.5 - d).clamp(0., 1.);
                if coverage == 0. {
                    if glass {
                        let shadow = distance(xx, yy - 3. * scale, cell, 17. * scale).max(0.);
                        let alpha = (12. * (-shadow / (2. * scale)).exp()) as u8;
                        out[i + 3] = out[i + 3].max(alpha);
                    }
                    continue;
                }
                let (mut color, alpha) = if glass {
                    let nx = (xx / cell * 2. - 1.).clamp(-1., 1.);
                    let ny = (yy / cell * 2. - 1.).clamp(-1., 1.);
                    let sx = (x as f32 + nx * nx.abs().powi(5) * 4. * scale)
                        .round()
                        .clamp(0., (width - 1) as f32) as usize;
                    let sy = (y as f32 + ny * ny.abs().powi(5) * 4. * scale)
                        .round()
                        .clamp(0., (height - 1) as f32) as usize;
                    let source = (sy * width + sx) * 4;
                    let tint = if dark {
                        [54., 43., 33.]
                    } else {
                        [255., 250., 245.]
                    };
                    let light = (1. - (xx + yy) / (2. * cell)).clamp(0., 1.);
                    let mut rgb = [0.; 3];
                    for c in 0..3 {
                        rgb[c] = backdrop[source + c] as f32 * if dark { 0.28 } else { 0.67 }
                            + tint[c] * if dark { 0.72 } else { 0.33 }
                            + light * if dark { 8. } else { 10. };
                    }
                    (rgb, 242.)
                } else {
                    (
                        if dark {
                            [48., 38., 32.]
                        } else {
                            [255., 255., 255.]
                        },
                        255.,
                    )
                };
                for c in &mut color {
                    *c = c.clamp(0., 255.);
                }
                let a = (alpha * coverage).round();
                for c in 0..3 {
                    out[i + c] = (color[c] * a / 255.).round() as u8;
                }
                out[i + 3] = a as u8;
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transparent_gaps_survive_every_material_and_dpi() {
        for scale in [1., 1.25, 1.5, 2., 2.25] {
            for count in [1, 2, 4] {
                let w = (width(count) * scale).round() as usize;
                let h = (HEIGHT * scale).round() as usize;
                for glass in [false, true] {
                    let image = pixels(w, h, scale, count, false, glass, &vec![200; w * h * 4]);
                    assert_eq!(image[3], 0);
                    assert_eq!(image[((h - 1) * w + w - 1) * 4 + 3], 0);
                    for gap in 0..count - 1 {
                        let x = ((76. + gap as f32 * 72.) * scale).round() as usize;
                        assert!(image[((40. * scale) as usize * w + x) * 4 + 3] <= 12);
                    }
                    assert!(
                        image[((40. * scale) as usize * w + (40. * scale) as usize) * 4 + 3] > 230
                    );
                    assert!(
                        image
                            .as_chunks::<4>()
                            .0
                            .iter()
                            .all(|p| p[..3].iter().all(|c| *c <= p[3]))
                    );
                }
            }
        }
    }
    #[test]
    fn glass_reacts_to_backdrop_but_solid_keys_do_not() {
        let w = 152;
        let h = 80;
        let a = vec![32; w * h * 4];
        let b = vec![224; w * h * 4];
        assert_ne!(
            pixels(w, h, 1., 2, true, true, &a),
            pixels(w, h, 1., 2, true, true, &b)
        );
        assert_eq!(
            pixels(w, h, 1., 2, true, false, &a),
            pixels(w, h, 1., 2, true, false, &b)
        );
    }
    #[test]
    fn fractional_text_scale_does_not_drop_the_last_candidate() {
        for scale in [1.1, 1.13, 1.25, 1.3, 1.75, 2.25] {
            for count in 1..=8 {
                assert_eq!(visible_count((width(count) * scale).round(), scale), count);
            }
        }
    }
}
