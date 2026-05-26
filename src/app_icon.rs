pub(crate) fn rgba_icon(size: u32) -> Vec<u8> {
    let size = size.max(1);
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    let max = (size - 1).max(1) as f32;

    for y in 0..size {
        for x in 0..size {
            let nx = ((x as f32 + 0.5) / size as f32) * 2.0 - 1.0;
            let ny = ((y as f32 + 0.5) / size as f32) * 2.0 - 1.0;
            let edge = nx.abs().max(ny.abs());
            let alpha = if edge < 0.88 {
                255
            } else {
                (((0.98 - edge) / 0.10) * 255.0).clamp(0.0, 255.0) as u8
            };

            let t = (x as f32 + y as f32) / (2.0 * max);
            let ((r0, g0, b0), (r1, g1, b1), u) = if t < 0.5 {
                ((196.0, 132.0, 144.0), (146.0, 128.0, 184.0), t / 0.5)
            } else {
                (
                    (146.0, 128.0, 184.0),
                    (116.0, 154.0, 212.0),
                    (t - 0.5) / 0.5,
                )
            };
            let mut r = (r0 + (r1 - r0) * u).round() as u8;
            let mut g = (g0 + (g1 - g0) * u).round() as u8;
            let mut b = (b0 + (b1 - b0) * u).round() as u8;

            if size >= 32 {
                let sx = x as f32 / size as f32;
                let sy = y as f32 / size as f32;
                let mark = (0.26 < sx && sx < 0.38 && 0.25 < sy && sy < 0.75)
                    || (0.32 < sx && sx < 0.72 && 0.25 < sy && sy < 0.36)
                    || (0.32 < sx && sx < 0.64 && 0.44 < sy && sy < 0.55)
                    || (0.32 < sx && sx < 0.72 && 0.64 < sy && sy < 0.75);
                if mark {
                    r = 255;
                    g = 255;
                    b = 255;
                }
            }

            rgba.extend_from_slice(&[r, g, b, alpha]);
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
