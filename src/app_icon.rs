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

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn mark_color(x: f32, y: f32) -> [f32; 3] {
    let rose = [0.82, 0.46, 0.55];
    let violet = [0.58, 0.46, 0.78];
    let blue = [0.42, 0.58, 0.78];
    let t = ((x + 0.55) * 0.72 + (0.42 - y) * 0.28).clamp(0.0, 1.0);
    if t < 0.58 {
        mix(rose, violet, t / 0.58)
    } else {
        mix(violet, blue, (t - 0.58) / 0.42)
    }
}

fn draw_gradient_box(
    pixel: &mut [f32; 4],
    x: f32,
    y: f32,
    cx: f32,
    cy: f32,
    hx: f32,
    hy: f32,
    radius: f32,
) {
    let distance = sd_round_box(x - cx, y - cy, hx, hy, radius);
    let alpha = smooth_alpha(distance, 0.020);
    if alpha > 0.0 {
        let c = mark_color(x, y);
        blend(pixel, [c[0], c[1], c[2], alpha]);
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

            let shadow = sd_round_box(nx - 0.035, ny - 0.045, 0.80, 0.80, 0.24);
            blend(
                &mut pixel,
                [0.0, 0.0, 0.0, smooth_alpha(shadow, softness * 2.0) * 0.22],
            );

            let bg = sd_round_box(nx, ny, 0.80, 0.80, 0.24);
            let bg_alpha = smooth_alpha(bg, softness * 1.6);
            if bg_alpha > 0.0 {
                blend(&mut pixel, [0.135, 0.135, 0.155, bg_alpha]);
                let border_alpha = smooth_alpha(bg.abs() - 0.018, softness * 1.8) * bg_alpha;
                blend(&mut pixel, [0.37, 0.33, 0.39, border_alpha * 0.68]);
            }

            let mark_shadow = [0.0, 0.0, 0.0, 0.17];
            let shadow_shift_x = -0.035;
            let shadow_shift_y = 0.020;
            for (cx, cy, hx, hy, radius) in [
                (
                    -0.310 + shadow_shift_x,
                    -0.020 + shadow_shift_y,
                    0.075,
                    0.455,
                    0.026,
                ),
                (
                    -0.040 + shadow_shift_x,
                    -0.380 + shadow_shift_y,
                    0.340,
                    0.072,
                    0.032,
                ),
                (
                    -0.075 + shadow_shift_x,
                    -0.020 + shadow_shift_y,
                    0.300,
                    0.068,
                    0.030,
                ),
                (
                    -0.040 + shadow_shift_x,
                    0.340 + shadow_shift_y,
                    0.340,
                    0.072,
                    0.032,
                ),
            ] {
                let d = sd_round_box(nx - cx, ny - cy, hx, hy, radius);
                let a = smooth_alpha(d, 0.020) * mark_shadow[3];
                if a > 0.0 {
                    blend(
                        &mut pixel,
                        [mark_shadow[0], mark_shadow[1], mark_shadow[2], a],
                    );
                }
            }

            draw_gradient_box(&mut pixel, nx, ny, -0.310, -0.020, 0.075, 0.455, 0.026);
            draw_gradient_box(&mut pixel, nx, ny, -0.040, -0.380, 0.340, 0.072, 0.032);
            draw_gradient_box(&mut pixel, nx, ny, -0.075, -0.020, 0.300, 0.068, 0.030);
            draw_gradient_box(&mut pixel, nx, ny, -0.040, 0.340, 0.340, 0.072, 0.032);

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
