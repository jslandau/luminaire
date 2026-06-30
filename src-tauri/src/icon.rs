/// Tray icon renderer — ports createLightbulbIcon from MainWindow.cpp.
///
/// The C++ version uses QPainter on a 32x32 pixmap with antialiasing.
/// We hand-compute the pixel buffer to reproduce the same shapes:
///   - Glow (radial gradient) when on
///   - Bulb (ellipse at 6,2 size 20x18)
///   - Base trapezoid (10,19)-(22,19)-(20,24)-(12,24)
///   - Screw threads (3 horizontal rects)

pub const ICON_SIZE: u32 = 32;

/// RGBA pixel buffer for the tray icon.
pub fn render_lightbulb_icon(on: bool, dark_mode: bool) -> Vec<u8> {
    let size = ICON_SIZE as usize;
    let mut rgba = vec![0u8; size * size * 4];

    // Colors (matching C++ exactly)
    let (bulb_r, bulb_g, bulb_b) = if on {
        (255u8, 220u8, 80u8)
    } else if dark_mode {
        (200u8, 200u8, 200u8)
    } else {
        (128u8, 128u8, 128u8)
    };

    let (outline_r, outline_g, outline_b) = if on {
        (200u8, 160u8, 40u8)
    } else if dark_mode {
        (150u8, 150u8, 150u8)
    } else {
        (80u8, 80u8, 80u8)
    };

    let (base_r, base_g, base_b) = if on {
        (180u8, 180u8, 180u8)
    } else {
        (100u8, 100u8, 100u8)
    };

    let (base_dark_r, base_dark_g, base_dark_b) = darken(base_r, base_g, base_b, 120);

    // Draw glow if on
    if on {
        // QRadialGradient glow(16, 12, 14)
        // glow.setColorAt(0, QColor(255, 240, 150, 180))
        // glow.setColorAt(1, QColor(255, 240, 150, 0))
        // painter.drawEllipse(2, 0, 28, 24)
        // The ellipse covers x:[2,30), y:[0,24)
        // Center of radial gradient: (16, 12), radius 14
        let cx = 16.0f64;
        let cy = 12.0f64;
        let radius = 14.0f64;
        for y in 0..24 {
            for x in 2..30 {
                let dx = (x as f64) - cx;
                let dy = (y as f64) - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                let t = (dist / radius).clamp(0.0, 1.0);
                // Linear interpolation between center and edge
                let alpha = (180.0 * (1.0 - t)).round() as u8;
                blend_pixel(&mut rgba, x, y, 255, 240, 150, alpha);
            }
        }
    }

    // Draw bulb (ellipse at 6,2 size 20x18)
    // QPainter::drawEllipse(x, y, w, h) draws an ellipse inscribed in the rect [x, x+w) x [y, y+h)
    // With a pen width of 1.5, the stroke is centered on the boundary.
    // We approximate: fill the ellipse interior with bulb color, draw outline on boundary.
    {
        let ex = 6.0f64;
        let ey = 2.0f64;
        let ew = 20.0f64;
        let eh = 18.0f64;
        let cx = ex + ew / 2.0;
        let cy = ey + eh / 2.0;
        let rx = ew / 2.0;
        let ry = eh / 2.0;
        let pen_half = 0.75f64; // half of 1.5 pen width

        for y in 0..size {
            for x in 0..size {
                let px = x as f64;
                let py = y as f64;
                let nx = (px - cx) / rx;
                let ny = (py - cy) / ry;
                let d = (nx * nx + ny * ny).sqrt();

                if d <= 1.0 - pen_half / ((rx + ry) / 2.0) {
                    // Interior
                    set_pixel(&mut rgba, x, y, bulb_r, bulb_g, bulb_b, 255);
                } else if d <= 1.0 + pen_half / ((rx + ry) / 2.0) {
                    // Outline (boundary)
                    set_pixel(&mut rgba, x, y, outline_r, outline_g, outline_b, 255);
                }
            }
        }
    }

    // Draw base/screw part
    // Trapezoid base: (10,19) -> (22,19) -> (20,24) -> (12,24)
    // We approximate as filled polygon
    {
        // Vertices: top-left (10,19), top-right (22,19), bottom-right (20,24), bottom-left (12,24)
        // For each y from 19 to 24, compute left and right x bounds
        for y in 19..=24 {
            let t = (y - 19) as f64 / 5.0; // 0 at top, 1 at bottom
            // Left edge: from (10,19) to (12,24)
            let left = 10.0 + t * 2.0;
            // Right edge: from (22,19) to (20,24)
            let right = 22.0 - t * 2.0;
            for x in 0..size {
                let px = x as f64;
                if px >= left - 0.5 && px <= right + 0.5 {
                    set_pixel(&mut rgba, x, y, base_r, base_g, base_b, 255);
                }
            }
        }
    }

    // Screw threads (rects drawn with pen of baseColor.darker(120))
    // drawRect(12, 24, 8, 2)  -> y:[24,26)
    // drawRect(13, 26, 6, 2)  -> y:[26,28)
    // drawRect(14, 28, 4, 2)  -> y:[28,30)
    // These are drawn as outlined rects, but we'll fill them since they're small
    draw_rect_outline(&mut rgba, 12, 24, 8, 2, base_dark_r, base_dark_g, base_dark_b);
    draw_rect_outline(&mut rgba, 13, 26, 6, 2, base_dark_r, base_dark_g, base_dark_b);
    draw_rect_outline(&mut rgba, 14, 28, 4, 2, base_dark_r, base_dark_g, base_dark_b);

    rgba
}

/// Convert an RGBA buffer to ARGB32 byte order (A,R,G,B per pixel) for ksni.
pub fn to_argb32(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let pixel_count = (width * height) as usize;
    let mut argb = vec![0u8; pixel_count * 4];
    for i in 0..pixel_count {
        let r = rgba[i * 4];
        let g = rgba[i * 4 + 1];
        let b = rgba[i * 4 + 2];
        let a = rgba[i * 4 + 3];
        // ARGB32: A, R, G, B per pixel
        argb[i * 4] = a;
        argb[i * 4 + 1] = r;
        argb[i * 4 + 2] = g;
        argb[i * 4 + 3] = b;
    }
    argb
}

// --- Helper functions ---

fn set_pixel(rgba: &mut [u8], x: usize, y: usize, r: u8, g: u8, b: u8, a: u8) {
    let idx = (y * ICON_SIZE as usize + x) * 4;
    if idx + 3 < rgba.len() {
        rgba[idx] = r;
        rgba[idx + 1] = g;
        rgba[idx + 2] = b;
        rgba[idx + 3] = a;
    }
}

fn blend_pixel(rgba: &mut [u8], x: usize, y: usize, r: u8, g: u8, b: u8, a: u8) {
    let idx = (y * ICON_SIZE as usize + x) * 4;
    if idx + 3 < rgba.len() {
        if a == 0 {
            return;
        }
        let old_r = rgba[idx] as u16;
        let old_g = rgba[idx + 1] as u16;
        let old_b = rgba[idx + 2] as u16;
        let old_a = rgba[idx + 3] as u16;

        let alpha = a as u16;
        let inv_alpha = 255 - alpha;

        let new_a = alpha + (old_a * inv_alpha) / 255;
        if new_a == 0 {
            return;
        }
        rgba[idx] = ((r as u16 * alpha + old_r * inv_alpha * old_a / 255) / new_a) as u8;
        rgba[idx + 1] = ((g as u16 * alpha + old_g * inv_alpha * old_a / 255) / new_a) as u8;
        rgba[idx + 2] = ((b as u16 * alpha + old_b * inv_alpha * old_a / 255) / new_a) as u8;
        rgba[idx + 3] = new_a as u8;
    }
}

/// Draw a rectangle outline (1px border) — matches QPainter::drawRect with a pen.
fn draw_rect_outline(rgba: &mut [u8], x: usize, y: usize, w: usize, h: usize, r: u8, g: u8, b: u8) {
    let size = ICON_SIZE as usize;
    // Top and bottom edges
    for i in x..(x + w).min(size) {
        if y < size {
            set_pixel(rgba, i, y, r, g, b, 255);
        }
        if y + h - 1 < size && h > 0 {
            set_pixel(rgba, i, y + h - 1, r, g, b, 255);
        }
    }
    // Left and right edges
    for j in y..(y + h).min(size) {
        if x < size {
            set_pixel(rgba, x, j, r, g, b, 255);
        }
        if x + w - 1 < size && w > 0 {
            set_pixel(rgba, x + w - 1, j, r, g, b, 255);
        }
    }
}

/// Darken a color by a factor (e.g., 120 means darker(120) in Qt — multiply by 100/120).
fn darken(r: u8, g: u8, b: u8, factor: u32) -> (u8, u8, u8) {
    let f = 100.0f64 / factor as f64;
    (
        (r as f64 * f).round() as u8,
        (g as f64 * f).round() as u8,
        (b as f64 * f).round() as u8,
    )
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_size() {
        let icon = render_lightbulb_icon(true, false);
        assert_eq!(icon.len(), (ICON_SIZE * ICON_SIZE * 4) as usize);
    }

    #[test]
    fn test_on_icon_has_yellow_pixels() {
        let icon = render_lightbulb_icon(true, false);
        // The bulb center (~16, 11) should be yellow-ish (255, 220, 80)
        let idx = (11 * ICON_SIZE as usize + 16) * 4;
        assert_eq!(icon[idx], 255);     // R
        assert_eq!(icon[idx + 1], 220); // G
        assert_eq!(icon[idx + 2], 80);  // B
    }

    #[test]
    fn test_off_icon_has_gray_pixels() {
        let icon = render_lightbulb_icon(false, false);
        // The bulb center should be gray (128, 128, 128)
        let idx = (11 * ICON_SIZE as usize + 16) * 4;
        assert_eq!(icon[idx], 128);
        assert_eq!(icon[idx + 1], 128);
        assert_eq!(icon[idx + 2], 128);
    }

    #[test]
    fn test_off_icon_dark_mode_lighter_gray() {
        let icon = render_lightbulb_icon(false, true);
        // Dark mode: lighter gray (200, 200, 200)
        let idx = (11 * ICON_SIZE as usize + 16) * 4;
        assert_eq!(icon[idx], 200);
        assert_eq!(icon[idx + 1], 200);
        assert_eq!(icon[idx + 2], 200);
    }

    #[test]
    fn test_on_icon_has_glow() {
        let icon = render_lightbulb_icon(true, false);
        // Glow extends above the bulb — check a pixel at (16, 1) which is in the glow ellipse
        // but outside the bulb ellipse
        let idx = (1 * ICON_SIZE as usize + 16) * 4;
        let a = icon[idx + 3];
        // Should have some alpha from the glow
        assert!(a > 0, "glow pixel should have alpha > 0, got {}", a);
        // And it should be yellowish
        assert!(icon[idx] > 200, "glow should be reddish-yellow, R={}", icon[idx]);
    }

    #[test]
    fn test_off_icon_no_glow() {
        let icon = render_lightbulb_icon(false, false);
        // No glow when off — pixel at (16, 1) should be transparent
        let idx = (1 * ICON_SIZE as usize + 16) * 4;
        assert_eq!(icon[idx + 3], 0, "off icon should have no glow");
    }

    #[test]
    fn test_to_argb32_conversion() {
        // 2x2 image with known values
        let rgba = vec![
            255, 0, 0, 255,    // pixel 0: red, opaque
            0, 255, 0, 128,    // pixel 1: green, semi-transparent
            0, 0, 255, 0,      // pixel 2: blue, transparent
            255, 255, 0, 255,  // pixel 3: yellow, opaque
        ];
        let argb = to_argb32(&rgba, 2, 2);
        // Pixel 0: A=255, R=255, G=0, B=0
        assert_eq!(&argb[0..4], &[255, 255, 0, 0]);
        // Pixel 1: A=128, R=0, G=255, B=0
        assert_eq!(&argb[4..8], &[128, 0, 255, 0]);
        // Pixel 2: A=0, R=0, G=0, B=255
        assert_eq!(&argb[8..12], &[0, 0, 0, 255]);
        // Pixel 3: A=255, R=255, G=255, B=0
        assert_eq!(&argb[12..16], &[255, 255, 255, 0]);
    }
}
