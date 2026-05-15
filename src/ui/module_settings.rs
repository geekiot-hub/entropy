use super::*;

impl EntropyApp {
    fn module_setting_label(&self, title: &str) -> String {
        crate::i18n::tr_text(self.app_settings.language, title)
    }

    fn module_setting_tooltip(&self, field: &ModuleSettingField) -> String {
        let lang = self.app_settings.language;
        let key = match field.title.as_str() {
            "Left mode" | "Right mode" => "modules_settings.mode_tooltip",
            "Left ball axis" | "Right ball axis" => "modules_settings.ball_axis_tooltip",
            "Left touch axis" | "Right touch axis" => "modules_settings.touch_axis_tooltip",
            "Left ball DPI" | "Right ball DPI" => "modules_settings.ball_dpi_tooltip",
            "Left touch DPI" | "Right touch DPI" => "modules_settings.touch_dpi_tooltip",
            "Left scroll sens" | "Right scroll sens" => "modules_settings.scroll_sens_tooltip",
            "Left sniper sens" | "Right sniper sens" => "modules_settings.sniper_sens_tooltip",
            "Left text sens" | "Right text sens" => "modules_settings.text_sens_tooltip",
            "Left invert scroll" | "Right invert scroll" => {
                "modules_settings.invert_scroll_tooltip"
            }
            "Left acceleration" | "Right acceleration" => "modules_settings.acceleration_tooltip",
            "Sticky mode" => "modules_settings.sticky_mode_tooltip",
            "LED blinks" => "modules_settings.led_blinks_tooltip",
            "Auto layer in Normal" => "modules_settings.auto_layer_normal_tooltip",
            "Auto layer" => "modules_settings.auto_layer_tooltip",
            "Auto layer in Sniper" => "modules_settings.auto_layer_sniper_tooltip",
            "Auto layer in Scroll" => "modules_settings.auto_layer_scroll_tooltip",
            "Auto layer in Text" => "modules_settings.auto_layer_text_tooltip",
            _ => "modules_settings.generic_tooltip",
        };
        let field_label = self.module_setting_label(&field.title);
        crate::i18n::tr_catalog_format(lang, key, &[("field", field_label.as_str())])
    }

    fn write_module_setting_value(&mut self, field: &ModuleSettingField, value: u16) {
        self.module_settings.set_value(field.qsid, value);
        let Some(hid) = &self.hid_device else {
            return;
        };
        let result = if field.width > 1 {
            hid.set_qmk_setting_u16(field.qsid, value)
        } else {
            hid.set_qmk_setting_u8(field.qsid, value.min(u8::MAX as u16) as u8)
        };
        if let Err(e) = result {
            self.status_msg = format!("Failed to save module setting (qsid {}): {}", field.qsid, e);
            log::warn!("set_qmk_setting(module qsid {}) failed: {e}", field.qsid);
        }
    }

    fn draw_module_settings_row(
        &mut self,
        ui: &mut egui::Ui,
        row_idx: usize,
        content_width: f32,
        row_height: f32,
        suppress_tooltips: bool,
    ) {
        let Some(field) = self.module_settings.fields.get(row_idx).cloned() else {
            return;
        };
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        let dark = ui.visuals().dark_mode;
        let label = self.module_setting_label(&field.title);
        let tooltip = if suppress_tooltips {
            None
        } else {
            Some(self.module_setting_tooltip(&field))
        };
        let raw_value = self.module_settings.value(field.qsid);
        match field.kind {
            ModuleSettingKind::Boolean => {
                let switch_width = metrics.value(46.0);
                let switch_size = metrics.size(46.0, 24.0);
                let mask = 1u16 << field.bit;
                let mut checked = raw_value & mask != 0;
                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    content_width,
                    row_height,
                    label.as_str(),
                    true,
                    tooltip.as_deref(),
                    switch_width,
                    |ui| {
                        let resp = crate::ui_style::settings_switch_sized_stable(
                            ui,
                            ("module_settings", field.qsid, field.bit),
                            &mut checked,
                            switch_size,
                        );
                        if resp.changed() {
                            let new_value = if checked {
                                raw_value | mask
                            } else {
                                raw_value & !mask
                            };
                            self.write_module_setting_value(&field, new_value);
                        }
                    },
                );
            }
            ModuleSettingKind::Integer => {
                let field_width = metrics.value(86.0);
                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    content_width,
                    row_height,
                    label.as_str(),
                    true,
                    tooltip.as_deref(),
                    field_width,
                    |ui| {
                        let edit_id = egui::Id::new(("module_setting_edit", field.qsid));
                        let current = raw_value.clamp(field.min, field.max);
                        let mut text = ui.ctx().data_mut(|d| {
                            d.get_temp::<String>(edit_id)
                                .unwrap_or_else(|| current.to_string())
                        });
                        if text.parse::<u16>().ok() != Some(current)
                            && !ui.memory(|m| m.has_focus(edit_id))
                        {
                            text = current.to_string();
                        }
                        let resp = crate::ui_style::modern_text_field_sized(
                            ui,
                            edit_id,
                            &mut text,
                            field_width,
                            metrics.settings_control_height(),
                            "",
                            5,
                            egui::Align::Center,
                        );
                        let commit = resp.lost_focus()
                            || (resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                        if commit {
                            match text.trim().parse::<u16>() {
                                Ok(value) => {
                                    let value = value.clamp(field.min, field.max);
                                    if value != raw_value {
                                        self.write_module_setting_value(&field, value);
                                    }
                                    text = value.to_string();
                                }
                                Err(_) => text = current.to_string(),
                            }
                        }
                        ui.ctx().data_mut(|d| d.insert_temp(edit_id, text));
                    },
                );
            }
            ModuleSettingKind::Select => {
                let dropdown_width = metrics.value(120.0);
                let selected_idx = (raw_value as usize).min(field.variants.len().saturating_sub(1));
                let variants = field
                    .variants
                    .iter()
                    .map(|variant| crate::i18n::tr_text(self.app_settings.language, variant))
                    .collect::<Vec<_>>();
                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    content_width,
                    row_height,
                    label.as_str(),
                    true,
                    tooltip.as_deref(),
                    dropdown_width,
                    |ui| {
                        let dropdown_id =
                            ui.make_persistent_id(("module_setting_dropdown", field.qsid));
                        let (_, picked) = Self::draw_touchpad_select_control(
                            ui,
                            dark,
                            dropdown_id,
                            selected_idx,
                            &variants,
                            dropdown_width,
                        );
                        if let Some(picked) = picked {
                            self.write_module_setting_value(&field, picked as u16);
                        }
                    },
                );
            }
        }
    }

    pub(super) fn draw_module_settings_page(
        &mut self,
        ui: &mut egui::Ui,
        content_rect: egui::Rect,
    ) {
        let lang = self.app_settings.language;
        let dark = ui.visuals().dark_mode;
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(18.0);
                ui.label(
                    RichText::new(crate::i18n::tr_catalog(lang, "modules_settings.title"))
                        .size(18.0)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(crate::i18n::tr_catalog(
                        lang,
                        "modules_settings.description",
                    ))
                    .size(13.0)
                    .color(app_muted_text(dark)),
                );
                ui.add_space(24.0);

                if !self.module_settings.supported {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr_catalog(lang, "modules_settings.unavailable"),
                        Some(crate::i18n::tr(
                            lang,
                            crate::i18n::Key::QmkSettingsEnableHint,
                        )),
                    );
                    return;
                }

                let list = allocate_adaptive_settings_list_viewport(
                    ui,
                    "module_settings",
                    metrics,
                    self.module_settings.row_count(),
                    0.0,
                );
                ui.allocate_ui_at_rect(list.content_rect, |ui| {
                    ui.set_clip_rect(list.viewport);
                    ui.set_min_size(list.content_rect.size());
                    ui.spacing_mut().item_spacing.y = 0.0;
                    for row_idx in list.first_visible_row..list.last_visible_row {
                        self.draw_module_settings_row(
                            ui,
                            row_idx,
                            list.row_content_width,
                            list.row_height,
                            list.suppress_tooltips,
                        );
                    }
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
}
