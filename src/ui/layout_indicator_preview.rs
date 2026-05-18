use super::*;

const STICKY_LAYOUT_KEYBOARD_MARGIN: f32 = 1.0_f32;

fn draw_sticky_encoder_arrow(
    painter: &egui::Painter,
    center: egui::Pos2,
    encoder_radius: f32,
    top: bool,
    color: Color32,
) {
    let (start_deg, end_deg) = if top {
        (240.0_f32, 300.0_f32)
    } else {
        (120.0_f32, 60.0_f32)
    };
    let r = encoder_radius * 1.22;
    let mut points = Vec::new();
    for step in 0..=12 {
        let t = step as f32 / 12.0;
        let deg = start_deg + (end_deg - start_deg) * t;
        let rad = deg.to_radians();
        points.push(egui::pos2(
            center.x + rad.cos() * r,
            center.y + rad.sin() * r,
        ));
    }
    painter.add(egui::Shape::line(points.clone(), Stroke::new(1.7, color)));
    if points.len() >= 2 {
        let end = points[points.len() - 1];
        let prev = points[points.len() - 2];
        let dir = egui::vec2(end.x - prev.x, end.y - prev.y).normalized();
        let left = egui::vec2(-dir.y, dir.x);
        painter.add(egui::Shape::convex_polygon(
            vec![
                end,
                egui::pos2(
                    end.x - dir.x * 3.6 + left.x * 2.4,
                    end.y - dir.y * 3.6 + left.y * 2.4,
                ),
                egui::pos2(
                    end.x - dir.x * 3.6 - left.x * 2.4,
                    end.y - dir.y * 3.6 - left.y * 2.4,
                ),
            ],
            color,
            Stroke::NONE,
        ));
    }
}

fn sticky_compact_label(label: &str, max_chars: usize) -> String {
    let label = label.trim();
    let count = label.chars().count();
    if count <= max_chars {
        return label.to_string();
    }
    let keep = max_chars.saturating_sub(1).max(1);
    format!("{}…", label.chars().take(keep).collect::<String>())
}

fn sticky_key_label_sizes(label: &str, rect: egui::Rect) -> (Option<f32>, f32) {
    let (top, bottom) = key_label_font_sizes(label);
    let longest = label
        .split(['\n', '/'])
        .map(|part| part.trim().chars().count())
        .max()
        .unwrap_or(1)
        .max(1) as f32;
    let available = (rect.width() - 8.0).max(8.0);
    let base = bottom.max(top.unwrap_or(bottom));
    let fit_scale = (available / (longest * base * 0.58)).clamp(0.58, 1.0);
    (top.map(|size| size * fit_scale), bottom * fit_scale)
}

fn draw_sticky_key_label(
    painter: &egui::Painter,
    rect: egui::Rect,
    label: &str,
    dark: bool,
    rotation: f32,
    dimmed: bool,
) {
    let clip_rect = rect.shrink2(egui::vec2(4.0, 3.0));
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
    let (top_size, bottom_size) = sticky_key_label_sizes(label, rect);
    let (top_color, bottom_color) = if dimmed {
        if dark {
            (Color32::from_rgb(45, 45, 50), Color32::from_rgb(62, 56, 56))
        } else {
            (
                Color32::from_rgb(215, 215, 220),
                Color32::from_rgb(200, 200, 208),
            )
        }
    } else if dark {
        (
            Color32::from_rgb(130, 130, 145),
            Color32::from_rgb(239, 233, 232),
        )
    } else {
        (
            Color32::from_rgb(130, 130, 150),
            Color32::from_rgb(26, 26, 30),
        )
    };

    let clipped = painter.with_clip_rect(clip_rect);
    if let Some(top_str) = top {
        let center = rect.center();
        paint_centered_text_rotated(
            &clipped,
            center + rotated_offset(0.0, -7.0, rotation),
            top_str,
            FontId::proportional(top_size.unwrap_or(9.0)),
            top_color,
            rotation,
        );
        paint_centered_text_rotated(
            &clipped,
            center + rotated_offset(0.0, 6.0, rotation),
            bottom,
            FontId::proportional(bottom_size),
            bottom_color,
            rotation,
        );
    } else {
        let font_size = if bottom == "↵" {
            16.0_f32.min(bottom_size + 4.0)
        } else {
            bottom_size
        };
        paint_centered_text_rotated(
            &clipped,
            rect.center(),
            bottom,
            FontId::proportional(font_size),
            bottom_color,
            rotation,
        );
    }
}

fn preview_layout_geometry(
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
        2.0,
        2.0,
        6.0,
        Some(f32::INFINITY),
    )
}

impl EntropyApp {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn paint_sticky_layout_preview(
        ui: &mut egui::Ui,
        layout: &KeyboardLayout,
        layer: usize,
        layer_names: &[String],
        macro_names: &[String],
        tap_dance_names: &[String],
        key_legend_layout: KeyLegendLayout,
        show_shifted_number_symbols: bool,
        encoder_visibility: &[bool],
        matrix_pressed: &[bool],
        pressed_key_layers: &[Option<usize>],
        ui_scale: f32,
        dark: bool,
        rect: egui::Rect,
    ) {
        let painter = ui.painter_at(rect);
        let keyboard_rect = rect.shrink(STICKY_LAYOUT_KEYBOARD_MARGIN);
        let geometry = preview_layout_geometry(ui.ctx(), layout, keyboard_rect, ui_scale);
        let outline = if dark {
            Color32::from_rgb(58, 58, 62)
        } else {
            Color32::from_rgb(225, 225, 229)
        };
        let key_fill = if dark {
            Color32::from_rgb(48, 48, 52)
        } else {
            Color32::from_rgb(255, 255, 255)
        };
        let empty_fill = if dark {
            Color32::from_rgb(28, 28, 31)
        } else {
            Color32::from_rgb(248, 248, 250)
        };

        let key_rects: Vec<(usize, egui::Rect)> = layout
            .keys
            .iter()
            .enumerate()
            .map(|(ki, key)| (ki, layout_physical_key_rect(key, geometry)))
            .collect();

        let mut encoder_groups: Vec<(u8, egui::Rect, Option<(usize, u16)>, Option<(usize, u16)>)> =
            Vec::new();
        for (encoder_idx, encoder) in layout.encoders.iter().enumerate() {
            if !encoder_visibility
                .get(encoder.encoder_idx as usize)
                .copied()
                .unwrap_or(true)
            {
                continue;
            }

            let encoder_rect = layout_physical_encoder_rect(encoder, geometry);
            let kc = layout.get_encoder_keycode(layer, encoder_idx);
            if let Some((_, group_rect, ccw, cw)) = encoder_groups
                .iter_mut()
                .find(|(idx, _, _, _)| *idx == encoder.encoder_idx)
            {
                *group_rect = group_rect.union(encoder_rect);
                if encoder.direction == 0 {
                    *ccw = Some((encoder_idx, kc));
                } else {
                    *cw = Some((encoder_idx, kc));
                }
            } else {
                encoder_groups.push((
                    encoder.encoder_idx,
                    encoder_rect,
                    (encoder.direction == 0).then_some((encoder_idx, kc)),
                    (encoder.direction != 0).then_some((encoder_idx, kc)),
                ));
            }
        }

        let mut encoder_press_rects: Vec<(usize, egui::Rect)> = Vec::new();
        for (_, group_rect, _, _) in &encoder_groups {
            let center = group_rect.center();
            let radius = group_rect.width().min(group_rect.height()) * 0.5;
            let mut best_key: Option<(usize, f32)> = None;
            for (ki, key_rect) in &key_rects {
                if encoder_press_rects
                    .iter()
                    .any(|(assigned_ki, _)| assigned_ki == ki)
                {
                    continue;
                }
                let dist = key_rect.center().distance(center);
                if dist > radius * 0.38 {
                    continue;
                }
                match best_key {
                    Some((_, best_dist)) if dist >= best_dist => {}
                    _ => best_key = Some((*ki, dist)),
                }
            }
            if let Some((ki, _)) = best_key {
                let press_rect = egui::Rect::from_center_size(
                    center,
                    Vec2::new(
                        (radius * 0.88).min(group_rect.width() * 0.44),
                        (radius * 0.48).min(group_rect.height() * 0.22),
                    ),
                );
                encoder_press_rects.push((ki, press_rect));
            }
        }

        for (ki, key_rect) in &key_rects {
            if encoder_press_rects
                .iter()
                .any(|(press_ki, _)| press_ki == ki)
            {
                continue;
            }

            let key = &layout.keys[*ki];
            let matrix_idx = key.row as usize * layout.cols + key.col as usize;
            let is_pressed = layout_matrix_key_pressed(layout, matrix_pressed, key.row, key.col);
            let key_layer = if is_pressed {
                pressed_key_layers
                    .get(matrix_idx)
                    .and_then(|source_layer| *source_layer)
                    .filter(|source_layer| *source_layer < layout.layers.len())
                    .unwrap_or(layer)
            } else {
                layer
            };
            let kc = layout.get_keycode(key_layer, *ki);
            let is_transparent = kc == 0x0001;
            let fill = if is_pressed {
                app_hover_fill(dark)
            } else if kc == 0x0000 {
                empty_fill
            } else {
                key_fill
            };
            let stroke = if is_pressed { app_accent() } else { outline };
            paint_layout_keycap(
                &painter,
                *key_rect,
                key.rotation,
                fill,
                Stroke::new(1.0, stroke),
            );

            if kc == 0x0000 {
                continue;
            }

            let label_kc = if is_transparent {
                (0..key_layer)
                    .rev()
                    .map(|fallback_layer| layout.get_keycode(fallback_layer, *ki))
                    .find(|fallback| !matches!(*fallback, 0x0000 | 0x0001))
                    .unwrap_or(0x0000)
            } else {
                kc
            };
            if label_kc == 0x0000 {
                continue;
            }
            let label = number_row_shifted_label(
                keycode_label_with_macro_names(
                    label_kc,
                    &layout.custom_keycodes,
                    layer_names,
                    macro_names,
                    tap_dance_names,
                    key_legend_layout,
                ),
                show_shifted_number_symbols,
                key_legend_layout,
            );
            draw_sticky_key_label(
                &painter,
                *key_rect,
                &label,
                dark,
                key.rotation.to_radians(),
                is_transparent,
            );
        }

        let label_for = |kc: Option<u16>| -> String {
            let label = match kc.unwrap_or(0) {
                0x0000 => "✕".to_string(),
                0x0001 => "▽".to_string(),
                value => keycode_label_with_macro_names(
                    value,
                    &layout.custom_keycodes,
                    layer_names,
                    macro_names,
                    tap_dance_names,
                    key_legend_layout,
                )
                .replace('\n', " "),
            };
            sticky_compact_label(&label, 9)
        };
        let text_color = if dark {
            Color32::from_gray(232)
        } else {
            Color32::from_gray(32)
        };

        for (_, group_rect, ccw, cw) in encoder_groups {
            let center = group_rect.center();
            let radius = (group_rect.width().min(group_rect.height())
                * LAYOUT_ENCODER_RADIUS_FACTOR)
                .max(14.0);
            let fill_radius = radius + LAYOUT_ENCODER_FILL_EXTRA;
            let press_slot = encoder_press_rects
                .iter()
                .find(|(_, press_rect)| press_rect.center().distance(center) < 1.0)
                .map(|(press_ki, press_rect)| (*press_ki, *press_rect));
            let press_is_pressed = press_slot
                .map(|(press_ki, _)| {
                    let key = &layout.keys[press_ki];
                    layout_matrix_key_pressed(layout, matrix_pressed, key.row, key.col)
                })
                .unwrap_or(false);

            let (top_rect, middle_rect, bottom_rect) = if let Some((_, press_rect)) = press_slot {
                let divider_gap = radius * 0.06;
                let top_divider_y = press_rect.top() - divider_gap;
                let bottom_divider_y = press_rect.bottom() + divider_gap;
                (
                    egui::Rect::from_min_max(
                        egui::pos2(center.x - fill_radius, center.y - fill_radius),
                        egui::pos2(center.x + fill_radius, top_divider_y),
                    ),
                    Some(egui::Rect::from_min_max(
                        egui::pos2(center.x - fill_radius, top_divider_y),
                        egui::pos2(center.x + fill_radius, bottom_divider_y),
                    )),
                    egui::Rect::from_min_max(
                        egui::pos2(center.x - fill_radius, bottom_divider_y),
                        egui::pos2(center.x + fill_radius, center.y + fill_radius),
                    ),
                )
            } else {
                (
                    egui::Rect::from_min_max(
                        egui::pos2(center.x - fill_radius, center.y - fill_radius),
                        egui::pos2(center.x + fill_radius, center.y),
                    ),
                    None,
                    egui::Rect::from_min_max(
                        egui::pos2(center.x - fill_radius, center.y),
                        egui::pos2(center.x + fill_radius, center.y + fill_radius),
                    ),
                )
            };

            painter.circle_filled(center, fill_radius, key_fill);
            painter
                .with_clip_rect(top_rect)
                .circle_filled(center, fill_radius, key_fill);
            if let Some(middle_rect) = middle_rect {
                let middle_fill = if press_is_pressed {
                    app_hover_fill(dark)
                } else {
                    key_fill
                };
                painter
                    .with_clip_rect(middle_rect)
                    .circle_filled(center, fill_radius, middle_fill);
            }
            painter
                .with_clip_rect(bottom_rect)
                .circle_filled(center, fill_radius, key_fill);
            painter.circle_stroke(center, radius, Stroke::new(1.0, outline));

            let has_press_button = press_slot.is_some();
            let top_label = label_for(cw.map(|(_, kc)| kc));
            let bottom_label = label_for(ccw.map(|(_, kc)| kc));
            let top_font = if has_press_button {
                egui::FontId::proportional(if top_label.chars().count() > 9 {
                    6.6
                } else {
                    7.4
                })
            } else {
                egui::FontId::proportional(if top_label.chars().count() > 9 {
                    8.5
                } else {
                    9.5
                })
            };
            let bottom_font = if has_press_button {
                egui::FontId::proportional(if bottom_label.chars().count() > 9 {
                    6.6
                } else {
                    7.4
                })
            } else {
                egui::FontId::proportional(if bottom_label.chars().count() > 9 {
                    8.5
                } else {
                    9.5
                })
            };
            let top_label_y = center.y - radius * if has_press_button { 0.52 } else { 0.30 };
            let bottom_label_y = center.y + radius * if has_press_button { 0.52 } else { 0.30 };
            painter.text(
                egui::pos2(center.x, top_label_y),
                egui::Align2::CENTER_CENTER,
                top_label,
                top_font,
                text_color,
            );
            painter.text(
                egui::pos2(center.x, bottom_label_y),
                egui::Align2::CENTER_CENTER,
                bottom_label,
                bottom_font,
                text_color,
            );

            draw_sticky_encoder_arrow(&painter, center, radius, true, outline);
            draw_sticky_encoder_arrow(&painter, center, radius, false, outline);

            if let Some((press_ki, _)) = press_slot {
                let middle_rect = middle_rect.unwrap();
                let top_divider_y = middle_rect.top();
                let bottom_divider_y = middle_rect.bottom();
                let divider_radius = (radius - 0.5).max(0.0);
                let top_divider_half_width = (divider_radius * divider_radius
                    - (top_divider_y - center.y) * (top_divider_y - center.y))
                    .max(0.0)
                    .sqrt();
                let bottom_divider_half_width = (divider_radius * divider_radius
                    - (bottom_divider_y - center.y) * (bottom_divider_y - center.y))
                    .max(0.0)
                    .sqrt();
                painter.line_segment(
                    [
                        egui::pos2(center.x - top_divider_half_width, top_divider_y),
                        egui::pos2(center.x + top_divider_half_width, top_divider_y),
                    ],
                    Stroke::new(1.0, outline),
                );
                painter.line_segment(
                    [
                        egui::pos2(center.x - bottom_divider_half_width, bottom_divider_y),
                        egui::pos2(center.x + bottom_divider_half_width, bottom_divider_y),
                    ],
                    Stroke::new(1.0, outline),
                );

                let press_label = {
                    let key = &layout.keys[press_ki];
                    let matrix_idx = key.row as usize * layout.cols + key.col as usize;
                    let press_layer = pressed_key_layers
                        .get(matrix_idx)
                        .and_then(|source_layer| *source_layer)
                        .filter(|source_layer| *source_layer < layout.layers.len())
                        .unwrap_or(layer);
                    let kc = layout.get_keycode(press_layer, press_ki);
                    if kc == 0x0001 {
                        let fallback_kc = (0..press_layer)
                            .rev()
                            .map(|fallback_layer| layout.get_keycode(fallback_layer, press_ki))
                            .find(|fallback| !matches!(*fallback, 0x0000 | 0x0001))
                            .unwrap_or(0x0000);
                        if fallback_kc == 0x0000 {
                            "▽".to_string()
                        } else {
                            keycode_label_with_macro_names(
                                fallback_kc,
                                &layout.custom_keycodes,
                                layer_names,
                                macro_names,
                                tap_dance_names,
                                key_legend_layout,
                            )
                        }
                    } else if kc == 0x0000 {
                        "✕".to_string()
                    } else {
                        keycode_label_with_macro_names(
                            kc,
                            &layout.custom_keycodes,
                            layer_names,
                            macro_names,
                            tap_dance_names,
                            key_legend_layout,
                        )
                    }
                }
                .replace('\n', " ");
                let press_label = sticky_compact_label(&press_label, 8);
                let press_text_rect = middle_rect.shrink2(egui::vec2(4.0, 2.0));
                let press_font = FontId::proportional(if press_label.chars().count() > 8 {
                    7.2
                } else {
                    8.2
                });
                painter.with_clip_rect(press_text_rect).text(
                    press_text_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    press_label,
                    press_font,
                    text_color,
                );
            } else {
                let divider_half_width = (radius - 0.5).max(0.0);
                painter.line_segment(
                    [
                        egui::pos2(center.x - divider_half_width, center.y),
                        egui::pos2(center.x + divider_half_width, center.y),
                    ],
                    Stroke::new(1.0, outline),
                );
            }
        }
    }
}
