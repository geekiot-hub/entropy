use super::*;

impl EntropyApp {
    pub(super) fn draw_combo_settings_page(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        content_rect: egui::Rect,
    ) {
        self.handle_combo_editor_input(ctx, false);
        let lang = self.app_settings.language;
        let dark = ui.visuals().dark_mode;

        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                let scale = responsive_settings_editor_scale(ui.ctx());
                ui.add_space(18.0 * scale);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::ComboTitle))
                        .size(18.0 * scale)
                        .strong(),
                );
                ui.add_space(6.0 * scale);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::ComboDescription))
                        .size(13.0 * scale)
                        .color(app_muted_text(dark)),
                );
                ui.add_space(18.0 * scale);
                self.draw_combo_editor_content(ui, false);
            });
        });
    }

    pub(super) fn push_combo_undo(&mut self) {
        self.combo_undo_stack.push((
            self.combo_entries.clone(),
            self.combo_names.clone(),
            self.combo_term,
            self.selected_combo,
            self.combo_visible_count,
        ));
        if self.combo_undo_stack.len() > 64 {
            self.combo_undo_stack.remove(0);
        }
    }

    fn apply_combo_capture(&mut self) {
        if !(2..=4).contains(&self.combo_capture_keys.len()) {
            return;
        }
        self.push_combo_undo();
        if let Some(combo) = self.combo_entries.get_mut(self.selected_combo) {
            combo.keys = [0; 4];
            for (idx, kc) in self.combo_capture_keys.iter().copied().enumerate().take(4) {
                combo.keys[idx] = kc;
            }
            self.combo_dirty = true;
        }
        self.combo_capture_open = false;
        self.combo_capture_keys.clear();
    }

    fn cancel_combo_capture(&mut self) {
        self.combo_capture_open = false;
        self.combo_capture_keys.clear();
    }

    fn handle_combo_editor_input(&mut self, ctx: &egui::Context, allow_close: bool) -> bool {
        if !self.keycode_picker.open && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.combo_capture_open {
                self.cancel_combo_capture();
            } else if allow_close {
                self.combo_capture_open = false;
                self.combo_capture_keys.clear();
                return true;
            }
        }

        if self.combo_capture_open {
            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.apply_combo_capture();
            } else {
                for event in ctx.input(|i| i.events.clone()) {
                    if let egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } = event
                    {
                        if matches!(key, egui::Key::Enter | egui::Key::Escape) {
                            continue;
                        }
                        if let Some(kc) = egui_key_to_qmk(key, modifiers) {
                            if !self.combo_capture_keys.contains(&kc)
                                && self.combo_capture_keys.len() < 4
                            {
                                self.combo_capture_keys.push(kc);
                            }
                        }
                    }
                }
            }
        }

        false
    }

    fn draw_combo_editor_content(&mut self, ui: &mut egui::Ui, show_intro: bool) {
        let dark = ui.visuals().dark_mode;
        if show_intro {
            crate::ui_style::modal_hint(
                ui,
                crate::i18n::tr(
                    self.app_settings.language,
                    crate::i18n::Key::ComboDescription,
                ),
            );
        }

        if self.firmware != FirmwareProtocol::Vial {
            crate::ui_style::modal_empty_state(
                ui,
                "Dynamic combos are not supported for this firmware",
                None,
            );
            return;
        }

        if self.combo_entries.is_empty() {
            crate::ui_style::modal_empty_state(
                ui,
                "This device does not report any dynamic combo slots",
                None,
            );
            return;
        }

        self.selected_combo = self
            .selected_combo
            .min(self.combo_entries.len().saturating_sub(1));
        self.combo_names
            .resize(self.combo_entries.len(), String::new());
        self.combo_visible_count = self.combo_entries.len().max(1);

        let combo_idx = self.selected_combo;
        let page_center_x = ui.max_rect().center().x;
        let combo_undo_snapshot = (
            self.combo_entries.clone(),
            self.combo_names.clone(),
            self.combo_term,
            self.selected_combo,
            self.combo_visible_count,
        );
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        let scale = metrics.scale;
        let content_width = metrics.settings_content_width();
        let row_content_width = metrics.settings_row_content_width();
        let row_height = metrics.settings_row_height();
        let control_width = metrics.settings_control_width();
        let control_height = metrics.settings_control_height();
        let control_font_size = metrics.settings_control_font_size();
        let timeout_control_width = metrics.value(118.0);
        let custom = self
            .layout
            .as_ref()
            .map(|l| l.custom_keycodes.as_slice())
            .unwrap_or(&[]);
        let selected_combo_empty = self
            .combo_entries
            .get(combo_idx)
            .map(|entry| entry.keys.iter().all(|&k| k == 0) && entry.output == 0)
            .unwrap_or(true)
            && self
                .combo_names
                .get(combo_idx)
                .map(|name| name.trim().is_empty())
                .unwrap_or(true);
        let selected_text = match self.combo_names.get(combo_idx) {
            Some(name) if !name.trim().is_empty() => format!("C{}: {}", combo_idx, name.trim()),
            _ => format!("C{}", combo_idx),
        };
        let selected_text_color = if selected_combo_empty {
            app_inactive_entry_text(dark)
        } else {
            ui.visuals().text_color()
        };
        let input_summary = {
            let keys: Vec<String> = if self.combo_capture_open {
                self.combo_capture_keys
                    .iter()
                    .copied()
                    .map(|kc| {
                        keycode_label_with_macro_names(
                            kc,
                            custom,
                            &self.layer_names,
                            &self.keycode_picker.macro_names,
                            &self.keycode_picker.tap_dance_names,
                            self.app_settings.key_legend_layout,
                        )
                        .replace('\n', " ")
                    })
                    .collect()
            } else {
                self.combo_entries[combo_idx]
                    .keys
                    .iter()
                    .copied()
                    .filter(|&kc| kc != 0)
                    .map(|kc| {
                        keycode_label_with_macro_names(
                            kc,
                            custom,
                            &self.layer_names,
                            &self.keycode_picker.macro_names,
                            &self.keycode_picker.tap_dance_names,
                            self.app_settings.key_legend_layout,
                        )
                        .replace('\n', " ")
                    })
                    .collect()
            };
            if keys.is_empty() {
                if self.combo_capture_open {
                    crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "combo_editor.press_2_4_keys",
                    )
                    .to_string()
                } else {
                    crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "combo_editor.record_2_4_keys",
                    )
                    .to_string()
                }
            } else {
                keys.join(" + ")
            }
        };
        let output_label = if self.combo_entries[combo_idx].output == 0 {
            crate::i18n::tr_catalog(self.app_settings.language, "combo_editor.pick_output")
                .to_string()
        } else {
            keycode_label_with_macro_names(
                self.combo_entries[combo_idx].output,
                custom,
                &self.layer_names,
                &self.keycode_picker.macro_names,
                &self.keycode_picker.tap_dance_names,
                self.app_settings.key_legend_layout,
            )
            .replace('\n', " ")
        };

        crate::ui_style::modal_content(
            ui,
            crate::ui_style::ModalLayout::new(content_width).with_top_padding(4.0 * scale),
            |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    row_content_width,
                    row_height,
                    crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.entry"),
                    true,
                    Some(crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "combo_editor.select_combo_slot",
                    )),
                    control_width,
                    |ui| {
                        let dropdown_id = ui.make_persistent_id("combo_entry_dropdown");
                        let dropdown_resp = crate::ui_style::modern_dropdown_button_sized(
                            ui,
                            dropdown_id,
                            selected_text.as_str(),
                            selected_text_color,
                            control_width,
                            control_height,
                            control_font_size,
                        );
                        ui.style_mut().visuals.window_stroke =
                            crate::ui_style::modal_outline_stroke(dark);
                        ui.style_mut().visuals.window_fill = app_surface_fill(dark);
                        egui::popup_below_widget(
                            ui,
                            dropdown_id,
                            &dropdown_resp,
                            egui::PopupCloseBehavior::CloseOnClickOutside,
                            |ui| {
                                ui.set_min_width(control_width);
                                ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
                                egui::ScrollArea::vertical()
                                    .id_salt("combo_entry_dropdown_scroll")
                                    .max_height(142.0 * scale)
                                    .auto_shrink([false, true])
                                    .show(ui, |ui| {
                                        for entry_idx in 0..self.combo_entries.len() {
                                            let empty = self
                                                .combo_entries
                                                .get(entry_idx)
                                                .map(|entry| {
                                                    entry.keys.iter().all(|&k| k == 0)
                                                        && entry.output == 0
                                                })
                                                .unwrap_or(true)
                                                && self
                                                    .combo_names
                                                    .get(entry_idx)
                                                    .map(|name| name.trim().is_empty())
                                                    .unwrap_or(true);
                                            let option_text = match self.combo_names.get(entry_idx)
                                            {
                                                Some(name) if !name.trim().is_empty() => {
                                                    format!("C{}: {}", entry_idx, name.trim())
                                                }
                                                _ => format!("C{}", entry_idx),
                                            };
                                            let selected = entry_idx == self.selected_combo;
                                            let (option_rect, option_resp) = ui
                                                .allocate_exact_size(
                                                    Vec2::new(control_width, 28.0 * scale),
                                                    Sense::click(),
                                                );
                                            if option_resp.hovered() {
                                                ui.ctx().set_cursor_icon(
                                                    egui::CursorIcon::PointingHand,
                                                );
                                            }
                                            let option_fill = if selected {
                                                if dark {
                                                    Color32::from_rgb(58, 58, 61)
                                                } else {
                                                    Color32::from_rgb(236, 236, 238)
                                                }
                                            } else if option_resp.hovered() {
                                                crate::ui_style::hover_fill(dark)
                                            } else {
                                                Color32::TRANSPARENT
                                            };
                                            ui.painter().rect_filled(option_rect, 7.0, option_fill);
                                            ui.painter().text(
                                                egui::pos2(
                                                    option_rect.left() + 10.0,
                                                    option_rect.center().y,
                                                ),
                                                egui::Align2::LEFT_CENTER,
                                                option_text,
                                                FontId::proportional(12.0 * scale),
                                                if selected {
                                                    ui.visuals().text_color()
                                                } else if empty {
                                                    app_inactive_entry_text(dark)
                                                } else {
                                                    app_muted_text(dark)
                                                },
                                            );
                                            if option_resp.clicked() {
                                                self.selected_combo = entry_idx;
                                                ui.memory_mut(|m| m.close_popup());
                                            }
                                        }
                                    });
                            },
                        );
                    },
                );

                let mut combo_name_changed = false;
                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    row_content_width,
                    row_height,
                    crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.name"),
                    true,
                    Some(crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "combo_editor.local_name_for_this_combo_slot",
                    )),
                    control_width,
                    |ui| {
                        if let Some(name) = self.combo_names.get_mut(combo_idx) {
                            let resp = crate::ui_style::modern_text_field_sized(
                                ui,
                                egui::Id::new(("combo_name", combo_idx)),
                                name,
                                control_width,
                                control_height,
                                crate::i18n::tr_catalog(
                                    self.app_settings.language,
                                    "alt_repeat_editor.name",
                                ),
                                12,
                                egui::Align::Center,
                            );
                            combo_name_changed = resp.changed();
                            resp.clone().on_hover_text(crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "alt_repeat_editor.stored_locally_in_entropy",
                            ));
                        }
                    },
                );
                if combo_name_changed {
                    self.combo_undo_stack.push(combo_undo_snapshot.clone());
                    self.combo_names_dirty = true;
                }

                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    row_content_width,
                    row_height,
                    crate::i18n::tr_catalog(self.app_settings.language, "combo_editor.input_keys"),
                    true,
                    Some(crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "combo_editor.keys_that_must_be_pressed_together",
                    )),
                    control_width,
                    |ui| {
                        let field_resp = crate::ui_style::modern_button_with_font(
                            ui,
                            input_summary.as_str(),
                            Vec2::new(control_width, control_height),
                            control_font_size,
                            true,
                        );
                        if field_resp.clicked() {
                            self.combo_capture_keys.clear();
                            self.combo_capture_open = true;
                        }
                        if self.combo_capture_open {
                            let clicked_outside_input = ui.ctx().input(|i| {
                                i.pointer.any_pressed()
                                    && i.pointer
                                        .interact_pos()
                                        .map(|pos| !field_resp.rect.contains(pos))
                                        .unwrap_or(false)
                            });
                            if clicked_outside_input {
                                self.apply_combo_capture();
                            }
                        }
                    },
                );

                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    row_content_width,
                    row_height,
                    crate::i18n::tr_catalog(self.app_settings.language, "combo_editor.output_key"),
                    true,
                    Some(crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "combo_editor.keycode_sent_when_the_combo_activates",
                    )),
                    control_width,
                    |ui| {
                        let resp = crate::ui_style::modern_button_with_font(
                            ui,
                            output_label.as_str(),
                            Vec2::new(control_width, control_height),
                            control_font_size,
                            true,
                        );
                        if resp.clicked() {
                            self.combo_pick_target = Some((combo_idx, ComboPickField::Output));
                            self.keycode_picker.result = None;
                            self.keycode_picker.selected_tab = KeycodeTab::Basic;
                            self.keycode_picker.open = true;
                        }
                    },
                );

                if let Some(current_combo_term) = self.combo_term {
                    let mut combo_term_text = current_combo_term.to_string();
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        row_content_width,
                        row_height,
                        crate::i18n::tr_catalog(self.app_settings.language, "common.timeout"),
                        true,
                        Some(crate::i18n::tr_catalog(
                            self.app_settings.language,
                            "combo_editor.maximum_time_between_combo_key_presses",
                        )),
                        timeout_control_width,
                        |ui| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let resp = crate::ui_style::modern_text_field_sized(
                                        ui,
                                        egui::Id::new("combo_term"),
                                        &mut combo_term_text,
                                        70.0 * scale,
                                        control_height,
                                        "",
                                        4,
                                        egui::Align::RIGHT,
                                    );
                                    let resp = settings_field_unit_tooltip(
                                        resp,
                                        self.app_settings.language,
                                        false,
                                        SettingsFieldUnit::Milliseconds,
                                    );
                                    if resp.changed() {
                                        let filtered: String = combo_term_text
                                            .chars()
                                            .filter(|c| c.is_ascii_digit())
                                            .take(4)
                                            .collect();
                                        if let Ok(parsed) = filtered.parse::<u16>() {
                                            self.combo_undo_stack.push(combo_undo_snapshot.clone());
                                            self.combo_term = Some(parsed.max(1));
                                            self.combo_term_dirty = true;
                                        }
                                    }
                                },
                            );
                        },
                    );
                }
            },
        );

        ui.add_space(14.0 * scale);
        let action_size = crate::ui_style::modal_action_button_size() * scale;
        let action_width = action_size.x * 2.0 + 8.0 * scale;
        let action_rect = egui::Rect::from_min_size(
            egui::pos2(page_center_x - action_width / 2.0, ui.cursor().min.y),
            Vec2::new(action_width, action_size.y),
        );
        ui.allocate_ui_at_rect(action_rect, |ui| {
            ui.set_min_size(action_rect.size());
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 8.0 * scale;
                let clear_enabled = combo_idx < self.combo_entries.len()
                    && (self.combo_entries[combo_idx].keys.iter().any(|&k| k != 0)
                        || self.combo_entries[combo_idx].output != 0
                        || self
                            .combo_names
                            .get(combo_idx)
                            .map(|s| !s.trim().is_empty())
                            .unwrap_or(false));
                let clear_resp = crate::ui_style::modern_button_with_font(
                    ui,
                    crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.clear"),
                    action_size,
                    control_font_size,
                    clear_enabled,
                );
                if clear_resp.clicked() && clear_enabled {
                    self.push_combo_undo();
                    self.combo_entries[combo_idx] = ComboEntry::default();
                    if let Some(name) = self.combo_names.get_mut(combo_idx) {
                        name.clear();
                    }
                    self.combo_dirty = true;
                    self.combo_names_dirty = true;
                }
                let undo_enabled = !self.combo_undo_stack.is_empty();
                let undo_resp = crate::ui_style::modern_button_with_font(
                    ui,
                    crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.undo"),
                    action_size,
                    control_font_size,
                    undo_enabled,
                );
                if undo_resp.clicked() && undo_enabled {
                    if let Some((entries, names, term, selected, visible_count)) =
                        self.combo_undo_stack.pop()
                    {
                        self.combo_entries = entries;
                        self.combo_names = names;
                        self.combo_term = term;
                        self.combo_visible_count =
                            visible_count.clamp(1, self.combo_entries.len().max(1));
                        self.selected_combo =
                            selected.min(self.combo_visible_count.saturating_sub(1));
                        self.combo_dirty = true;
                        self.combo_names_dirty = true;
                        self.combo_term_dirty = true;
                    }
                }
            });
        });
        ui.allocate_space(Vec2::new(1.0, action_size.y));
    }
}
