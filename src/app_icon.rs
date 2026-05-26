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

fn sd_capsule(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32, radius: f32) -> f32 {
    let pax = px - ax;
    let pay = py - ay;
    let bax = bx - ax;
    let bay = by - ay;
    let h = ((pax * bax + pay * bay) / (bax * bax + bay * bay)).clamp(0.0, 1.0);
    let dx = pax - bax * h;
    let dy = pay - bay * h;
    (dx * dx + dy * dy).sqrt() - radius
}

fn draw_capsule(
    pixel: &mut [f32; 4],
    x: f32,
    y: f32,
    a: (f32, f32),
    b: (f32, f32),
    radius: f32,
    color: [f32; 3],
) {
    let distance = sd_capsule(x, y, a.0, a.1, b.0, b.1, radius);
    let alpha = smooth_alpha(distance, 0.020);
    if alpha > 0.0 {
        blend(pixel, [color[0], color[1], color[2], alpha]);
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
                blend(&mut pixel, [0.36, 0.33, 0.38, border_alpha * 0.70]);
            }

            draw_capsule(
                &mut pixel,
                nx,
                ny,
                (-0.45, -0.22),
                (0.42, -0.46),
                0.085,
                [0.77, 0.52, 0.57],
            );
            draw_capsule(
                &mut pixel,
                nx,
                ny,
                (-0.48, 0.05),
                (0.47, -0.08),
                0.085,
                [0.57, 0.50, 0.72],
            );
            draw_capsule(
                &mut pixel,
                nx,
                ny,
                (-0.42, 0.33),
                (0.38, 0.34),
                0.085,
                [0.45, 0.60, 0.78],
            );

            let dot = (nx * nx + ny * ny).sqrt() - 0.105;
            let dot_alpha = smooth_alpha(dot, softness * 1.5);
            if dot_alpha > 0.0 {
                blend(&mut pixel, [0.92, 0.90, 0.86, dot_alpha]);
            }

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
