use super::*;

impl EntropyApp {
    pub(super) fn draw_layer_led_settings_page(
        &mut self,
        ui: &mut egui::Ui,
        content_rect: egui::Rect,
    ) {
        let lang = self.app_settings.language;
        let dark = ui.visuals().dark_mode;
        let hid_ready = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.hid_device.is_some()
            }
            #[cfg(target_arch = "wasm32")]
            {
                false
            }
        };

        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(18.0);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::LayerLedsTitle))
                        .size(18.0)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(crate::i18n::tr(
                        lang,
                        crate::i18n::Key::LayerLedsDescription,
                    ))
                    .size(13.0)
                    .color(app_muted_text(dark)),
                );
                ui.add_space(24.0);

                if !self.layer_led_settings.supported {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::LayerLedsUnavailable),
                        Some(crate::i18n::tr(lang, crate::i18n::Key::LayerLedsEnableHint)),
                    );
                    return;
                }

                if !hid_ready {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::LayerLedsConnect),
                        None,
                    );
                    return;
                }

                const TOTAL_ROWS: usize = 18;
                let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
                let list = allocate_adaptive_settings_list_viewport(
                    ui,
                    "layer_led_settings",
                    metrics,
                    TOTAL_ROWS,
                    0.0,
                );
                ui.allocate_ui_at_rect(list.content_rect, |ui| {
                    ui.set_clip_rect(list.viewport);
                    ui.set_min_size(list.content_rect.size());
                    ui.spacing_mut().item_spacing.y = 0.0;
                    self.draw_layer_led_editor_content(
                        ui,
                        list.first_visible_row..list.last_visible_row,
                        list.row_content_width,
                        list.row_height,
                        list.suppress_tooltips,
                    );
                });

                if list.has_scrollbar {
                    crate::ui_style::paint_floating_scrollbar_handle(
                        ui,
                        list.track_rect,
                        list.handle_height,
                        list.scroll_ratio,
                        list.track_hovered,
                    );
                }
            });
        });
    }

    fn draw_layer_led_editor_content(
        &mut self,
        ui: &mut egui::Ui,
        row_range: std::ops::Range<usize>,
        content_width: f32,
        row_height: f32,
        suppress_tooltips: bool,
    ) {
        let scale = (row_height / 54.0).clamp(1.0, 1.12);
        let slider_width = 168.0 * scale;
        let value_width = 36.0 * scale;
        let slider_size = [slider_width, 18.0 * scale];
        let slider_control_width = slider_width + value_width;
        let swatch_width = 64.0 * scale;
        let swatch_size = Vec2::new(64.0 * scale, 34.0 * scale);

        for row_idx in row_range {
            match row_idx {
                0 => {
                    let brightness_max = 255.0_f32;
                    let mut value = (self.layer_led_settings.brightness as f32 / brightness_max
                        * 100.0)
                        .round()
                        .clamp(0.0, 100.0);
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        crate::i18n::tr_catalog(
                            self.app_settings.language,
                            "advanced_settings.led_brightness",
                        ),
                        true,
                        if suppress_tooltips {
                            None
                        } else {
                            Some(crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "advanced_settings.global_led_brightness_for_layer_color_lighting",
                            ))
                        },
                        slider_control_width,
                        |ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            let dark = ui.visuals().dark_mode;
                            let slider_fill = app_accent().gamma_multiply(0.5);
                            ui.visuals_mut().selection.bg_fill = slider_fill;
                            ui.visuals_mut().widgets.active.bg_fill = slider_fill;
                            ui.visuals_mut().widgets.active.weak_bg_fill = slider_fill;
                            ui.visuals_mut().widgets.hovered.bg_stroke =
                                Stroke::new(1.0, slider_fill);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.add_sized(
                                        [value_width, row_height],
                                        egui::Label::new(
                                            RichText::new(format!("{}%", value.round() as u8))
                                                .size(12.0 * scale)
                                                .color(if dark {
                                                    Color32::from_gray(230)
                                                } else {
                                                    Color32::from_gray(55)
                                                }),
                                        )
                                        .halign(egui::Align::RIGHT),
                                    );
                                    ui.spacing_mut().slider_width = slider_width;
                                    let slider = egui::Slider::new(&mut value, 0.0..=100.0)
                                        .step_by(1.0)
                                        .show_value(false)
                                        .trailing_fill(true);
                                    let resp = ui.add_sized(slider_size, slider);
                                    if resp.changed() {
                                        let new_value = ((value / 100.0) * brightness_max)
                                            .round()
                                            .clamp(0.0, brightness_max)
                                            as u16;
                                        if new_value != self.layer_led_settings.brightness {
                                            self.layer_led_settings.brightness = new_value;
                                            self.write_layer_led_brightness(new_value);
                                        }
                                    }
                                },
                            );
                        },
                    );
                }
                1 => {
                    let mut value = self.layer_led_settings.timeout_mins as f32;
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        crate::i18n::tr_catalog(
                            self.app_settings.language,
                            "advanced_settings.led_timeout",
                        ),
                        true,
                        if suppress_tooltips {
                            None
                        } else {
                            Some(crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "advanced_settings.minutes_before_leds_turn_off_automatically_0_disables_timeout",
                            ))
                        },
                        slider_control_width,
                        |ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            let dark = ui.visuals().dark_mode;
                            let slider_fill = if dark {
                                Color32::from_rgb(92, 92, 96)
                            } else {
                                Color32::from_rgb(190, 184, 182)
                            };
                            ui.visuals_mut().selection.bg_fill = slider_fill;
                            ui.visuals_mut().widgets.active.bg_fill = slider_fill;
                            ui.visuals_mut().widgets.active.weak_bg_fill = slider_fill;
                            ui.visuals_mut().widgets.hovered.bg_stroke =
                                Stroke::new(1.0, slider_fill);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let value_text = if value.round() as u8 == 0 {
                                        crate::i18n::tr_catalog(
                                            self.app_settings.language,
                                            "advanced_settings.off",
                                        )
                                        .to_string()
                                    } else {
                                        format!("{}m", value.round() as u8)
                                    };
                                    ui.add_sized(
                                        [value_width, row_height],
                                        egui::Label::new(
                                            RichText::new(value_text).size(12.0 * scale).color(
                                                if dark {
                                                    Color32::from_gray(230)
                                                } else {
                                                    Color32::from_gray(55)
                                                },
                                            ),
                                        )
                                        .halign(egui::Align::RIGHT),
                                    );
                                    ui.spacing_mut().slider_width = slider_width;
                                    let slider = egui::Slider::new(&mut value, 0.0..=255.0)
                                        .step_by(1.0)
                                        .show_value(false)
                                        .trailing_fill(true);
                                    let resp = ui.add_sized(slider_size, slider);
                                    let resp = settings_field_unit_tooltip(
                                        resp,
                                        self.app_settings.language,
                                        suppress_tooltips,
                                        SettingsFieldUnit::Minutes,
                                    );
                                    if resp.changed() {
                                        let new_value = value.round().clamp(0.0, 255.0) as u8;
                                        if new_value != self.layer_led_settings.timeout_mins {
                                            self.layer_led_settings.timeout_mins = new_value;
                                            self.write_layer_led_timeout(new_value);
                                        }
                                    }
                                },
                            );
                        },
                    );
                }
                2..=17 => {
                    let layer = row_idx - 2;
                    let current = self.layer_led_settings.layer_colors[layer];
                    let layer_name = self
                        .layer_names
                        .get(layer)
                        .map(|name| name.trim())
                        .filter(|name| !name.is_empty() && *name != layer.to_string())
                        .map(|name| {
                            let visible: String = name.chars().take(22).collect();
                            if matches!(self.app_settings.language, crate::i18n::Language::Russian)
                            {
                                format!("Слой {layer}: {visible}")
                            } else {
                                format!("Layer {layer}: {visible}")
                            }
                        })
                        .unwrap_or_else(|| {
                            if matches!(self.app_settings.language, crate::i18n::Language::Russian)
                            {
                                format!("Цвет слоя {layer}")
                            } else {
                                format!("Layer {layer} color")
                            }
                        });
                    let label = layer_name;
                    let tooltip =
                        if matches!(self.app_settings.language, crate::i18n::Language::Russian) {
                            format!("Цвет подсветки, когда активен слой {layer}")
                        } else {
                            format!("LED palette color used when layer {layer} is active")
                        };
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        &label,
                        true,
                        if suppress_tooltips {
                            None
                        } else {
                            Some(tooltip.as_str())
                        },
                        swatch_width,
                        |ui| {
                            let dark = ui.visuals().dark_mode;
                            let popup_id = ui.make_persistent_id(("layer_led_color_picker", layer));
                            let popup_open = ui.memory(|m| m.is_popup_open(popup_id));
                            let swatch_color = layer_led_palette_color(current);
                            let swatch_border = if popup_open {
                                app_accent()
                            } else if dark {
                                Color32::from_gray(95)
                            } else {
                                Color32::from_gray(185)
                            };
                            let (swatch_rect, swatch_resp) =
                                ui.allocate_exact_size(swatch_size, Sense::click());
                            if swatch_resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if swatch_resp.clicked() {
                                ui.memory_mut(|m| m.toggle_popup(popup_id));
                            }
                            ui.painter().rect(
                                swatch_rect,
                                9.0,
                                app_surface_fill(dark),
                                Stroke::new(1.0, swatch_border),
                                egui::StrokeKind::Inside,
                            );
                            ui.painter().rect(
                                swatch_rect.shrink(5.0 * scale),
                                6.0,
                                swatch_color,
                                Stroke::new(1.0, swatch_border.gamma_multiply(0.85)),
                                egui::StrokeKind::Inside,
                            );
                            if current == 0 {
                                ui.painter().line_segment(
                                    [
                                        swatch_rect.left_top()
                                            + egui::vec2(10.0 * scale, 10.0 * scale),
                                        swatch_rect.right_bottom()
                                            - egui::vec2(10.0 * scale, 10.0 * scale),
                                    ],
                                    Stroke::new(1.2, app_muted_text(dark)),
                                );
                            }
                            swatch_resp.clone().on_hover_text(crate::i18n::tr_catalog(
                                self.app_settings.language,
                                layer_led_palette_name(current),
                            ));

                            ui.style_mut().visuals.window_stroke =
                                crate::ui_style::modal_outline_stroke(dark);
                            ui.style_mut().visuals.window_fill = app_surface_fill(dark);
                            egui::popup_below_widget(
                                ui,
                                popup_id,
                                &swatch_resp,
                                egui::PopupCloseBehavior::CloseOnClickOutside,
                                |ui| {
                                    let cell = 28.0 * scale;
                                    let gap = 6.0 * scale;
                                    const COLS: usize = 5;
                                    let picker_width = cell * COLS as f32 + gap * (COLS - 1) as f32;
                                    ui.set_min_width(picker_width);
                                    ui.spacing_mut().item_spacing = Vec2::new(gap, gap);
                                    for row in 0..5 {
                                        ui.horizontal(|ui| {
                                            for col in 0..COLS {
                                                let color_idx = row * COLS + col;
                                                let Some(option_label) =
                                                    LAYER_LED_PALETTE.get(color_idx)
                                                else {
                                                    continue;
                                                };
                                                let color_idx_u8 = color_idx as u8;
                                                let selected = color_idx_u8 == current;
                                                let (cell_rect, cell_resp) = ui
                                                    .allocate_exact_size(
                                                        Vec2::splat(cell),
                                                        Sense::click(),
                                                    );
                                                if cell_resp.hovered() {
                                                    ui.ctx().set_cursor_icon(
                                                        egui::CursorIcon::PointingHand,
                                                    );
                                                }
                                                let outline = if selected {
                                                    app_accent()
                                                } else if dark {
                                                    Color32::from_rgb(72, 72, 76)
                                                } else {
                                                    Color32::from_rgb(210, 210, 214)
                                                };
                                                ui.painter().rect(
                                                    cell_rect,
                                                    7.0,
                                                    app_surface_fill(dark),
                                                    Stroke::new(
                                                        if selected { 1.6 } else { 1.0 },
                                                        outline,
                                                    ),
                                                    egui::StrokeKind::Inside,
                                                );
                                                ui.painter().rect(
                                                    cell_rect.shrink(4.5 * scale),
                                                    5.0,
                                                    layer_led_palette_color(color_idx_u8),
                                                    Stroke::NONE,
                                                    egui::StrokeKind::Inside,
                                                );
                                                if color_idx == 0 {
                                                    ui.painter().line_segment(
                                                        [
                                                            cell_rect.left_top()
                                                                + egui::vec2(
                                                                    8.0 * scale,
                                                                    8.0 * scale,
                                                                ),
                                                            cell_rect.right_bottom()
                                                                - egui::vec2(
                                                                    8.0 * scale,
                                                                    8.0 * scale,
                                                                ),
                                                        ],
                                                        Stroke::new(1.1, app_muted_text(dark)),
                                                    );
                                                }
                                                cell_resp.clone().on_hover_text(
                                                    crate::i18n::tr_text(
                                                        self.app_settings.language,
                                                        option_label,
                                                    ),
                                                );
                                                if cell_resp.clicked() {
                                                    self.layer_led_settings.layer_colors[layer] =
                                                        color_idx_u8;
                                                    self.write_layer_led_color(layer, color_idx_u8);
                                                    ui.memory_mut(|m| m.close_popup());
                                                }
                                            }
                                        });
                                    }
                                },
                            );
                        },
                    );
                }
                _ => {}
            }
        }
    }

    fn write_layer_led_color(&mut self, layer: usize, value: u8) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let qsid = 300 + layer as u16;
        if let Err(e) = hid.set_qmk_setting_u8(qsid, value) {
            self.status_msg = format!("Failed to save Layer LED color (layer {layer}): {}", e);
            log::warn!("set_qmk_setting_u8(layer_led qsid {qsid}) failed: {e}");
        }
    }

    fn write_layer_led_brightness(&mut self, value: u16) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let value = value.min(255);
        if let Err(e) = hid.set_qmk_setting_u16(316, value) {
            self.status_msg = format!("Failed to save Layer LED brightness: {}", e);
            log::warn!("set_qmk_setting_u16(layer_led brightness) failed: {e}");
        }
    }

    fn write_layer_led_timeout(&mut self, value: u8) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        if let Err(e) = hid.set_qmk_setting_u8(317, value) {
            self.status_msg = format!("Failed to save Layer LED timeout: {}", e);
            log::warn!("set_qmk_setting_u8(layer_led timeout) failed: {e}");
        }
    }
}
