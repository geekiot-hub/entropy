fn blend(dst: &mut [f32; 4], src: [f32; 4]) {
    let a = src[3].clamp(0.0, 1.0);
    let inv = 1.0 - a;
    dst[0] = src[0] * a + dst[0] * inv;
    dst[1] = src[1] * a + dst[1] * inv;
    dst[2] = src[2] * a + dst[2] * inv;
    dst[3] = a + dst[3] * inv;
}

fn smooth_alpha(distance: f32, softness: f32) -> f32 {
    (0.5 - distance / softness).clamp(0.0, 1.0)
}

fn sd_round_box(px: f32, py: f32, hx: f32, hy: f32, radius: f32) -> f32 {
    let qx = px.abs() - hx + radius;
    let qy = py.abs() - hy + radius;
    let ox = qx.max(0.0);
    let oy = qy.max(0.0);
    (ox * ox + oy * oy).sqrt() + qx.max(qy).min(0.0) - radius
}

fn rotate(px: f32, py: f32, angle: f32) -> (f32, f32) {
    let (s, c) = angle.sin_cos();
    (px * c - py * s, px * s + py * c)
}

fn draw_half(pixel: &mut [f32; 4], x: f32, y: f32, cx: f32, cy: f32, angle: f32) {
    let (lx, ly) = rotate(x - cx, y - cy, -angle);
    let body = sd_round_box(lx, ly, 0.255, 0.335, 0.070);
    let shadow = sd_round_box(lx - 0.018, ly - 0.022, 0.255, 0.335, 0.070);
    blend(pixel, [0.0, 0.0, 0.0, smooth_alpha(shadow, 0.030) * 0.20]);

    let body_alpha = smooth_alpha(body, 0.024);
    if body_alpha > 0.0 {
        blend(pixel, [0.89, 0.87, 0.83, body_alpha]);
    }

    let border_alpha = smooth_alpha(body.abs() - 0.018, 0.018) * body_alpha.max(0.65);
    if border_alpha > 0.0 {
        blend(pixel, [0.70, 0.56, 0.62, border_alpha * 0.75]);
    }

    let key_color = [0.16, 0.16, 0.18, 1.0];
    for row in 0..4 {
        for col in 0..3 {
            let kx = -0.135 + col as f32 * 0.135;
            let ky = -0.185 + row as f32 * 0.115;
            let key = sd_round_box(lx - kx, ly - ky, 0.034, 0.028, 0.010);
            let key_alpha = smooth_alpha(key, 0.016) * body_alpha;
            if key_alpha > 0.0 {
                blend(
                    pixel,
                    [key_color[0], key_color[1], key_color[2], key_alpha * 0.82],
                );
            }
        }
    }

    let thumb = sd_round_box(lx, ly - 0.285, 0.085, 0.026, 0.012);
    let thumb_alpha = smooth_alpha(thumb, 0.016) * body_alpha;
    if thumb_alpha > 0.0 {
        blend(pixel, [0.16, 0.16, 0.18, thumb_alpha * 0.82]);
    }
}

pub(crate) fn rgba_icon(size: u32) -> Vec<u8> {
    let size = size.max(1);
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    let softness = 2.0 / size as f32;

    for y in 0..size {
        for x in 0..size {
            let nx = ((x as f32 + 0.5) / size as f32) * 2.0 - 1.0;
            let ny = ((y as f32 + 0.5) / size as f32) * 2.0 - 1.0;
            let mut pixel = [0.0, 0.0, 0.0, 0.0];

            let bg = sd_round_box(nx, ny, 0.88, 0.88, 0.22);
            let bg_alpha = smooth_alpha(bg, softness * 1.4);
            if bg_alpha > 0.0 {
                blend(&mut pixel, [0.145, 0.145, 0.165, bg_alpha]);
                let border_alpha = smooth_alpha(bg.abs() - 0.020, softness * 1.8) * bg_alpha;
                blend(&mut pixel, [0.35, 0.32, 0.36, border_alpha * 0.72]);
            }

            draw_half(&mut pixel, nx, ny, -0.285, 0.045, -0.22);
            draw_half(&mut pixel, nx, ny, 0.285, 0.045, 0.22);

            rgba.push((pixel[0].clamp(0.0, 1.0) * 255.0).round() as u8);
            rgba.push((pixel[1].clamp(0.0, 1.0) * 255.0).round() as u8);
            rgba.push((pixel[2].clamp(0.0, 1.0) * 255.0).round() as u8);
            rgba.push((pixel[3].clamp(0.0, 1.0) * 255.0).round() as u8);
        }
    }

    rgba
}

pub(crate) fn egui_icon(size: u32) -> egui::IconData {
    egui::IconData {
        rgba: rgba_icon(size),
        width: size,
        height: size,
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn tray_icon(size: u32) -> Option<tray_icon::Icon> {
    tray_icon::Icon::from_rgba(rgba_icon(size), size, size).ok()
}
