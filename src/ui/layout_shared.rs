use super::*;

#[derive(Clone, Copy)]
pub(crate) struct LayoutGeometry {
    pub(crate) offset_x: f32,
    pub(crate) offset_y: f32,
    pub(crate) unit: f32,
    pub(crate) padding: f32,
    pub(crate) layout_h: f32,
}

pub(crate) fn responsive_layout_max_scale(ctx: &egui::Context, viewport: egui::Rect) -> f32 {
    let native_scale = ctx
        .native_pixels_per_point()
        .unwrap_or_else(|| ctx.pixels_per_point() / ctx.zoom_factor().max(0.1))
        .max(1.0);
    let physical_short_side = (viewport.width().min(viewport.height()) * native_scale).max(0.0);
    let t = ((physical_short_side - 1_080.0) / (2_160.0 - 1_080.0)).clamp(0.0, 1.0);
    1.0 + 0.35 * t
}

pub(crate) fn layout_geometry(
    ctx: &egui::Context,
    layout: &KeyboardLayout,
    viewport: egui::Rect,
    ui_scale: f32,
) -> LayoutGeometry {
    layout_geometry_with_reserved(
        ctx,
        layout,
        viewport,
        ui_scale,
        LAYOUT_TOP_RESERVED_H,
        LAYOUT_BOTTOM_RESERVED_H,
        LAYOUT_FIT_MARGIN,
        None,
    )
}

pub(crate) fn layout_geometry_with_reserved(
    ctx: &egui::Context,
    layout: &KeyboardLayout,
    viewport: egui::Rect,
    ui_scale: f32,
    top_reserved: f32,
    bottom_reserved: f32,
    fit_margin: f32,
    max_scale_override: Option<f32>,
) -> LayoutGeometry {
    let mut min_x: f32 = f32::MAX;
    let mut min_y: f32 = f32::MAX;
    let mut max_x: f32 = f32::MIN;
    let mut max_y: f32 = f32::MIN;
    for key in &layout.keys {
        let (x1, y1, x2, y2) = rotated_item_aabb(
            key.x,
            key.y,
            key.w,
            key.h,
            key.rotation,
            key.rotation_x,
            key.rotation_y,
        );
        min_x = min_x.min(x1);
        min_y = min_y.min(y1);
        max_x = max_x.max(x2);
        max_y = max_y.max(y2);
    }
    for encoder in &layout.encoders {
        let (x1, y1, x2, y2) = rotated_item_aabb(
            encoder.x,
            encoder.y,
            encoder.w,
            encoder.h,
            encoder.rotation,
            encoder.rotation_x,
            encoder.rotation_y,
        );
        min_x = min_x.min(x1);
        min_y = min_y.min(y1);
        max_x = max_x.max(x2);
        max_y = max_y.max(y2);
    }
    if min_x == f32::MAX {
        min_x = 0.0;
        min_y = 0.0;
        max_x = 1.0;
        max_y = 1.0;
    }

    let span_x = max_x - min_x;
    let span_y = max_y - min_y;
    let fit_width = viewport.width() * ui_scale;
    let fit_height = viewport.height() * ui_scale;
    let scale_x = (fit_width - fit_margin) / (span_x * LAYOUT_BASE_UNIT).max(1.0);
    let scale_y = (fit_height - fit_margin) / (span_y * LAYOUT_BASE_UNIT).max(1.0);
    let max_scale =
        max_scale_override.unwrap_or_else(|| responsive_layout_max_scale(ctx, viewport));
    let scale = scale_x.min(scale_y).min(max_scale);
    let unit = LAYOUT_BASE_UNIT * scale;
    let layout_w = span_x * unit;
    let layout_h = span_y * unit;
    let content_top = viewport.top() + top_reserved;
    let content_bottom = viewport.bottom() - bottom_reserved;

    LayoutGeometry {
        offset_x: viewport.center().x - layout_w / 2.0 - min_x * unit,
        offset_y: ((content_top + content_bottom) - layout_h) / 2.0 - min_y * unit,
        unit,
        padding: LAYOUT_KEY_PADDING,
        layout_h,
    }
}

pub(crate) fn layout_keycap_rect(
    offset_x: f32,
    offset_y: f32,
    unit: f32,
    padding: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(offset_x + x * unit + padding, offset_y + y * unit + padding),
        Vec2::new(w * unit - padding * 2.0, h * unit - padding * 2.0),
    )
}

fn rotate_layout_point(
    x: f32,
    y: f32,
    origin_x: f32,
    origin_y: f32,
    rotation_deg: f32,
) -> (f32, f32) {
    if rotation_deg == 0.0 {
        return (x, y);
    }
    let angle = rotation_deg.to_radians();
    let dx = x - origin_x;
    let dy = y - origin_y;
    (
        origin_x + dx * angle.cos() - dy * angle.sin(),
        origin_y + dx * angle.sin() + dy * angle.cos(),
    )
}

fn rotated_item_center(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rotation: f32,
    rotation_x: f32,
    rotation_y: f32,
) -> (f32, f32) {
    rotate_layout_point(x + w * 0.5, y + h * 0.5, rotation_x, rotation_y, rotation)
}

fn rotated_item_aabb(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rotation: f32,
    rotation_x: f32,
    rotation_y: f32,
) -> (f32, f32, f32, f32) {
    let corners = [(x, y), (x + w, y), (x + w, y + h), (x, y + h)];
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for (cx, cy) in corners {
        let (rx, ry) = rotate_layout_point(cx, cy, rotation_x, rotation_y, rotation);
        min_x = min_x.min(rx);
        min_y = min_y.min(ry);
        max_x = max_x.max(rx);
        max_y = max_y.max(ry);
    }
    (min_x, min_y, max_x, max_y)
}

pub(crate) fn layout_physical_key_rect(key: &PhysicalKey, geometry: LayoutGeometry) -> egui::Rect {
    let (center_x, center_y) = rotated_item_center(
        key.x,
        key.y,
        key.w,
        key.h,
        key.rotation,
        key.rotation_x,
        key.rotation_y,
    );
    egui::Rect::from_center_size(
        egui::pos2(
            geometry.offset_x + center_x * geometry.unit,
            geometry.offset_y + center_y * geometry.unit,
        ),
        Vec2::new(
            key.w * geometry.unit - geometry.padding * 2.0,
            key.h * geometry.unit - geometry.padding * 2.0,
        ),
    )
}

pub(crate) fn layout_physical_encoder_rect(
    encoder: &PhysicalEncoder,
    geometry: LayoutGeometry,
) -> egui::Rect {
    let (center_x, center_y) = rotated_item_center(
        encoder.x,
        encoder.y,
        encoder.w,
        encoder.h,
        encoder.rotation,
        encoder.rotation_x,
        encoder.rotation_y,
    );
    egui::Rect::from_center_size(
        egui::pos2(
            geometry.offset_x + center_x * geometry.unit,
            geometry.offset_y + center_y * geometry.unit,
        ),
        Vec2::new(
            encoder.w * geometry.unit - geometry.padding * 2.0,
            encoder.h * geometry.unit - geometry.padding * 2.0,
        ),
    )
}

pub(crate) fn paint_layout_keycap(
    painter: &egui::Painter,
    rect: egui::Rect,
    rotation: f32,
    fill: Color32,
    stroke: Stroke,
) {
    if rotation == 0.0 {
        painter.rect(rect, 6.0, fill, stroke, egui::StrokeKind::Inside);
        return;
    }

    let angle = rotation.to_radians();
    let center = rect.center();
    let rotate = |pos: egui::Pos2| {
        let dx = pos.x - center.x;
        let dy = pos.y - center.y;
        egui::pos2(
            center.x + dx * angle.cos() - dy * angle.sin(),
            center.y + dx * angle.sin() + dy * angle.cos(),
        )
    };

    let radius = 6.0_f32.min(rect.width() * 0.5).min(rect.height() * 0.5);
    let corner_segments = 5;
    let mut points = Vec::with_capacity((corner_segments + 1) * 4);
    let corners = [
        (
            egui::pos2(rect.right() - radius, rect.top() + radius),
            -std::f32::consts::FRAC_PI_2,
            0.0,
        ),
        (
            egui::pos2(rect.right() - radius, rect.bottom() - radius),
            0.0,
            std::f32::consts::FRAC_PI_2,
        ),
        (
            egui::pos2(rect.left() + radius, rect.bottom() - radius),
            std::f32::consts::FRAC_PI_2,
            std::f32::consts::PI,
        ),
        (
            egui::pos2(rect.left() + radius, rect.top() + radius),
            std::f32::consts::PI,
            std::f32::consts::PI * 1.5,
        ),
    ];
    for (corner_center, start, end) in corners {
        for step in 0..=corner_segments {
            let t = step as f32 / corner_segments as f32;
            let theta = start + (end - start) * t;
            let point = egui::pos2(
                corner_center.x + radius * theta.cos(),
                corner_center.y + radius * theta.sin(),
            );
            points.push(rotate(point));
        }
    }

    painter.add(egui::Shape::convex_polygon(points, fill, stroke));
}

pub(crate) fn layout_matrix_key_pressed(
    layout: &KeyboardLayout,
    matrix_pressed: &[bool],
    row: u8,
    col: u8,
) -> bool {
    matrix_pressed
        .get(row as usize * layout.cols + col as usize)
        .copied()
        .unwrap_or(false)
}

pub(crate) fn rotated_offset(dx: f32, dy: f32, angle: f32) -> egui::Vec2 {
    egui::vec2(
        dx * angle.cos() - dy * angle.sin(),
        dx * angle.sin() + dy * angle.cos(),
    )
}

pub(crate) fn paint_centered_text_rotated(
    painter: &egui::Painter,
    center: egui::Pos2,
    text: &str,
    font_id: FontId,
    color: Color32,
    rotation: f32,
) {
    if rotation == 0.0 {
        painter.text(center, egui::Align2::CENTER_CENTER, text, font_id, color);
        return;
    }

    let galley = painter.layout_no_wrap(text.to_string(), font_id, color);
    let half = galley.size() * 0.5;
    let pos = center - rotated_offset(half.x, half.y, rotation);
    painter.add(egui::Shape::Text(
        egui::epaint::TextShape::new(pos, galley, color).with_angle(rotation),
    ));
}

pub(crate) fn draw_key_label_dimmed(
    painter: &egui::Painter,
    rect: egui::Rect,
    label: &str,
    dark: bool,
    rotation: f32,
) {
    let dim = if dark {
        Color32::from_rgb(62, 56, 56)
    } else {
        Color32::from_rgb(200, 200, 208)
    };
    let dim_top = if dark {
        Color32::from_rgb(45, 45, 50)
    } else {
        Color32::from_rgb(215, 215, 220)
    };
    let (top, bottom) = if label.contains('\n') {
        let mut parts = label.splitn(2, '\n');
        let t = parts.next().unwrap_or("");
        let b = parts.next().unwrap_or(label);
        (Some(t), b)
    } else if let Some(pos) = label.find('/') {
        (Some(&label[..pos]), &label[pos + 1..])
    } else {
        (None, label)
    };
    let (top_size, bottom_size) = key_label_font_sizes(label);

    if let Some(top_str) = top {
        let center = rect.center();
        paint_centered_text_rotated(
            painter,
            center + rotated_offset(0.0, -7.0, rotation),
            top_str,
            FontId::proportional(top_size.unwrap_or(9.0)),
            dim_top,
            rotation,
        );
        paint_centered_text_rotated(
            painter,
            center + rotated_offset(0.0, 6.0, rotation),
            bottom,
            FontId::proportional(bottom_size),
            dim,
            rotation,
        );
    } else {
        let font_size = if bottom == "↵" { 16.0 } else { bottom_size };
        paint_centered_text_rotated(
            painter,
            rect.center(),
            bottom,
            FontId::proportional(font_size),
            dim,
            rotation,
        );
    }
}

pub(crate) fn number_row_shifted_label(
    label: String,
    enabled: bool,
    key_legend_layout: KeyLegendLayout,
) -> String {
    if !enabled {
        return label;
    }

    let Some((digit, english, russian)) = (match label.as_str() {
        "1" => Some(("1", "!", "!")),
        "2" => Some(("2", "@", "\"")),
        "3" => Some(("3", "#", "№")),
        "4" => Some(("4", "$", ";")),
        "5" => Some(("5", "%", "%")),
        "6" => Some(("6", "^", ":")),
        "7" => Some(("7", "&", "?")),
        "8" => Some(("8", "*", "*")),
        "9" => Some(("9", "(", "(")),
        "0" => Some(("0", ")", ")")),
        _ => None,
    }) else {
        return label;
    };

    let shifted = match key_legend_layout {
        KeyLegendLayout::English => english.to_string(),
        KeyLegendLayout::Russian => {
            if english == russian {
                english.to_string()
            } else {
                format!("{}  {}", english, russian)
            }
        }
        KeyLegendLayout::RussianPrimary => {
            if english == russian {
                russian.to_string()
            } else {
                format!("{}  {}", russian, english)
            }
        }
    };
    format!("{}\n{}", shifted, digit)
}

pub(crate) fn draw_key_label(
    painter: &egui::Painter,
    rect: egui::Rect,
    label: &str,
    dark: bool,
    rotation: f32,
) {
    // Split on "\n" first, then on "/" — show top part small+dim, bottom part normal
    let (top, bottom) = if label.contains('\n') {
        let mut parts = label.splitn(2, '\n');
        let t = parts.next().unwrap_or("");
        let b = parts.next().unwrap_or(label);
        (Some(t), b)
    } else if let Some(pos) = label.find('/') {
        let t = &label[..pos];
        let b = &label[pos + 1..];
        (Some(t), b)
    } else {
        (None, label)
    };
    let (top_size, bottom_size) = key_label_font_sizes(label);

    if let Some(top_str) = top {
        // Two-line layout
        let top_color = if dark {
            Color32::from_rgb(130, 130, 145)
        } else {
            Color32::from_rgb(130, 130, 150)
        };
        let main_color = if dark {
            Color32::from_rgb(239, 233, 232)
        } else {
            Color32::from_rgb(26, 26, 30)
        };
        paint_centered_text_rotated(
            painter,
            rect.center() + rotated_offset(0.0, -7.0, rotation),
            top_str,
            FontId::proportional(top_size.unwrap_or(9.0)),
            top_color,
            rotation,
        );
        paint_centered_text_rotated(
            painter,
            rect.center() + rotated_offset(0.0, 6.0, rotation),
            bottom,
            FontId::proportional(bottom_size),
            main_color,
            rotation,
        );
    } else {
        let font_size = if bottom == "↵" { 16.0 } else { bottom_size };
        paint_centered_text_rotated(
            painter,
            rect.center(),
            bottom,
            FontId::proportional(font_size),
            if dark {
                Color32::from_rgb(239, 233, 232)
            } else {
                Color32::from_rgb(26, 26, 30)
            },
            rotation,
        );
    }
}
