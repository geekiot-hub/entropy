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

    fn open_combo_key_picker(&mut self, combo_idx: usize, field: ComboPickField) {
        self.combo_pick_target = Some((combo_idx, field));
        self.keycode_picker.layer_names = self.layer_names.clone();
        self.keycode_picker
            .open_full_key_picker(crate::keycode_picker::KeycodeTab::Symbols);
    }

    fn handle_combo_editor_input(&mut self, ctx: &egui::Context, allow_close: bool) -> bool {
        if !self.keycode_picker.open && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if allow_close {
                return true;
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
        let input_keys_control_width = metrics.value(228.0);
        let input_keys_row_height = row_height.max(metrics.value(62.0));
        let input_key_size = metrics.size(54.0, 54.0);
        let timeout_control_width = metrics.value(118.0);
        let custom_pairs = self
            .layout
            .as_ref()
            .map(|l| l.custom_keycodes.clone())
            .unwrap_or_default();
        let custom = custom_pairs.as_slice();
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
                    input_keys_row_height,
                    crate::i18n::tr_catalog(self.app_settings.language, "combo_editor.input_keys"),
                    true,
                    Some(crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "combo_editor.keys_that_must_be_pressed_together",
                    )),
                    input_keys_control_width,
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 4.0 * scale;
                        for key_idx in 0..4 {
                            let value = self.combo_entries[combo_idx].keys[key_idx];
                            let button_label = if value == 0 {
                                String::new()
                            } else {
                                keycode_label_with_macro_names(
                                    value,
                                    custom,
                                    &self.layer_names,
                                    &self.keycode_picker.macro_names,
                                    &self.keycode_picker.tap_dance_names,
                                    self.app_settings.key_legend_layout,
                                )
                            };
                            let hover_label = button_label.replace('\n', " ");
                            let resp = crate::ui_style::modern_keycap_button(
                                ui,
                                button_label.as_str(),
                                input_key_size,
                                true,
                            );
                            if !hover_label.is_empty() {
                                resp.clone().on_hover_text(hover_label.as_str());
                            }
                            if resp.clicked_by(egui::PointerButton::Primary) {
                                self.open_combo_key_picker(
                                    combo_idx,
                                    ComboPickField::Trigger(key_idx),
                                );
                            }
                            if value != 0 && resp.clicked_by(egui::PointerButton::Secondary) {
                                self.push_combo_undo();
                                self.combo_entries[combo_idx].keys[key_idx] = 0;
                                self.combo_dirty = true;
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
                            self.open_combo_key_picker(combo_idx, ComboPickField::Output);
                        }
                    },
                );

                if let Some(current_combo_term) = self.combo_term {
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
                                    let edit_id = egui::Id::new("combo_term");
                                    let mut combo_term_text = ui.ctx().data_mut(|d| {
                                        d.get_temp::<String>(edit_id)
                                            .unwrap_or_else(|| current_combo_term.to_string())
                                    });
                                    if combo_term_text.parse::<u16>().ok()
                                        != Some(current_combo_term)
                                        && !ui.memory(|m| m.has_focus(edit_id))
                                    {
                                        combo_term_text = current_combo_term.to_string();
                                    }
                                    let resp = crate::ui_style::modern_text_field_sized(
                                        ui,
                                        edit_id,
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
                                        if filtered != combo_term_text {
                                            combo_term_text = filtered;
                                        }
                                    }
                                    let commit = resp.lost_focus()
                                        || (resp.has_focus()
                                            && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                                    if commit {
                                        match combo_term_text.trim().parse::<u16>() {
                                            Ok(parsed) => {
                                                let next_combo_term = parsed.max(1);
                                                if next_combo_term != current_combo_term {
                                                    self.combo_undo_stack
                                                        .push(combo_undo_snapshot.clone());
                                                    self.combo_term = Some(next_combo_term);
                                                    self.combo_term_dirty = true;
                                                }
                                                combo_term_text = next_combo_term.to_string();
                                            }
                                            Err(_) => {
                                                combo_term_text = current_combo_term.to_string();
                                            }
                                        }
                                    }
                                    ui.ctx().data_mut(|d| {
                                        d.insert_temp(edit_id, combo_term_text);
                                    });
                                    if self.combo_undo_stack.len() > 64 {
                                        self.combo_undo_stack.remove(0);
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
