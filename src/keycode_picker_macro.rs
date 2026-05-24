use super::*;

impl KeycodePicker {
    fn show_macro_editor_contents(
        &mut self,
        ui: &mut egui::Ui,
        raw_n: u8,
        grid_id: &'static str,
        _add_action_id: &'static str,
        _footer_text: &'static str,
    ) -> u8 {
        let mut selected_macro = raw_n;
        ui.label(
            RichText::new(crate::i18n::tr_catalog(
                self.language,
                "macro_editor.choose_macro",
            ))
            .size(11.0)
            .color(Color32::from_gray(150)),
        );
        ui.add_space(4.0);
        egui::Frame::NONE.show(ui, |ui| {
            let slot_scroll_height = 86.0 * responsive_picker_element_scale(ui.ctx());
            ui.set_max_height(slot_scroll_height);
            egui::ScrollArea::vertical()
                .max_height(slot_scroll_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Grid::new(grid_id)
                        .num_columns(16)
                        .spacing([4.0, 4.0])
                        .show(ui, |ui| {
                            for i in 0..128u8 {
                                let is_active = i == selected_macro;
                                let has_content = self.macro_has_content(i as usize);
                                let display_name = self.macro_display_name(i as usize);
                                let id_text = format!("M{}", i);
                                let mut resp = picker_slot_button(
                                    ui,
                                    &id_text,
                                    &display_name,
                                    is_active,
                                    has_content,
                                );
                                if display_name != id_text {
                                    resp = resp.on_hover_text(display_name.clone());
                                }
                                if resp.clicked() {
                                    self.ensure_macro_meta_len(i as usize);
                                    selected_macro = i;
                                }
                                if (i + 1) % 16 == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                });
        });
        ui.add_space(crate::ui_style::modal_space_sm());

        if selected_macro == 254 {
            ui.label(
                RichText::new(crate::i18n::tr_catalog(
                    self.language,
                    "macro_editor.select_a_macro_above_to_edit",
                ))
                .size(16.0)
                .color(Color32::from_gray(140)),
            );
            return selected_macro;
        }

        let n = selected_macro as usize;
        self.ensure_macro_meta_len(n);

        let scale = responsive_picker_element_scale(ui.ctx());
        let macro_font_size = 14.0 * scale;
        ui.add_space(4.0 * scale);
        if let Some(name) = self.macro_names.get_mut(n) {
            let resp = crate::ui_style::modern_text_field_sized(
                ui,
                ui.make_persistent_id(("macro_name", grid_id, n)),
                name,
                124.0 * scale,
                32.0 * scale,
                crate::i18n::tr_catalog(self.language, "macro_editor.macro_name"),
                7,
                egui::Align::Center,
            );
            if resp.changed() {
                let trimmed: String = name.chars().take(7).collect();
                *name = trimmed;
            }
        }
        ui.add_space(6.0);

        let mut remove_idx = None;
        let mut move_up: Option<usize> = None;
        let mut move_down: Option<usize> = None;
        let avail_w = ui.available_width();
        {
            let action_count = self.macro_actions[n].len();
            for (i, action) in self.macro_actions[n].iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    let arrow_size = picker_scaled_size(ui.ctx(), 28.0, 28.0);
                    let up_resp = picker_button(ui, "↑", arrow_size, i > 0, false).on_hover_text(
                        crate::i18n::tr_catalog(self.language, "macro_editor.move_up"),
                    );
                    let down_resp = picker_button(ui, "↓", arrow_size, i + 1 < action_count, false)
                        .on_hover_text(crate::i18n::tr_catalog(
                            self.language,
                            "macro_editor.move_down",
                        ));
                    if up_resp.clicked() && i > 0 {
                        move_up = Some(i);
                    }
                    if down_resp.clicked() && i + 1 < action_count {
                        move_down = Some(i);
                    }

                    let (type_label, type_color, tooltip) = match action {
                        MacroAction::Text(_) => (
                            crate::i18n::tr_catalog(self.language, "macro_editor.text"),
                            crate::ui_style::accent(),
                            crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.types_text_characters_one_by_one",
                            ),
                        ),
                        MacroAction::Tap(_) => (
                            crate::i18n::tr_catalog(self.language, "macro_editor.tap"),
                            crate::ui_style::accent(),
                            crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.press_and_release_a_key",
                            ),
                        ),
                        MacroAction::Down(_) => (
                            crate::i18n::tr_catalog(self.language, "macro_editor.down"),
                            Color32::from_rgb(200, 150, 50),
                            crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.press_a_key_hold_until_up",
                            ),
                        ),
                        MacroAction::Up(_) => (
                            crate::i18n::tr_catalog(self.language, "macro_editor.up"),
                            Color32::from_rgb(132, 150, 178),
                            crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.release_a_previously_pressed_key",
                            ),
                        ),
                        MacroAction::Delay(_) => (
                            crate::i18n::tr_catalog(self.language, "macro_editor.delay"),
                            Color32::from_gray(150),
                            crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.wait_before_next_action",
                            ),
                        ),
                    };
                    ui.allocate_ui(picker_scaled_size(ui.ctx(), 55.0, 30.0), |ui| {
                        ui.add(
                            egui::Label::new(
                                RichText::new(type_label)
                                    .size(macro_font_size)
                                    .color(type_color)
                                    .strong(),
                            )
                            .sense(egui::Sense::hover()),
                        )
                        .on_hover_text(tooltip);
                    });

                    match action {
                        MacroAction::Text(text) => {
                            let text_w = (avail_w - 220.0 * scale).max(150.0 * scale);
                            crate::ui_style::modern_text_field_sized(
                                ui,
                                ui.make_persistent_id(("macro_text_action", grid_id, n, i)),
                                text,
                                text_w,
                                32.0 * scale,
                                crate::i18n::tr_catalog(
                                    self.language,
                                    "macro_editor.type_text_here",
                                ),
                                256,
                                egui::Align::Min,
                            )
                            .on_hover_text(crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.characters_to_type_when_this_macro_runs",
                            ));
                        }
                        MacroAction::Tap(kc) => {
                            let label = keycode_label_with_names_and_layout(
                                *kc as u16,
                                &[],
                                &self.layer_names,
                                self.key_legend_layout,
                            );
                            if picker_button(
                                ui,
                                &label,
                                picker_scaled_size(ui.ctx(), 100.0, 30.0),
                                true,
                                false,
                            )
                            .on_hover_text(crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.click_to_change_key_press_and_release_this_key",
                            ))
                            .clicked()
                            {
                                self.macro_key_pick = Some((n, i));
                            }
                        }
                        MacroAction::Down(kc) => {
                            let label = keycode_label_with_names_and_layout(
                                *kc as u16,
                                &[],
                                &self.layer_names,
                                self.key_legend_layout,
                            );
                            if picker_button(
                                ui,
                                &label,
                                picker_scaled_size(ui.ctx(), 100.0, 30.0),
                                true,
                                false,
                            )
                            .on_hover_text(crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.click_to_change_key_holds_down_until_up",
                            ))
                            .clicked()
                            {
                                self.macro_key_pick = Some((n, i));
                            }
                        }
                        MacroAction::Up(kc) => {
                            let label = keycode_label_with_names_and_layout(
                                *kc as u16,
                                &[],
                                &self.layer_names,
                                self.key_legend_layout,
                            );
                            if picker_button(
                                ui,
                                &label,
                                picker_scaled_size(ui.ctx(), 100.0, 30.0),
                                true,
                                false,
                            )
                            .on_hover_text(crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.click_to_change_key_releases_this_key",
                            ))
                            .clicked()
                            {
                                self.macro_key_pick = Some((n, i));
                            }
                        }
                        MacroAction::Delay(ms) => {
                            let mut ms_str = ms.to_string();
                            if crate::ui_style::modern_text_field_sized(
                                ui,
                                ui.make_persistent_id(("macro_delay", grid_id, n, i)),
                                &mut ms_str,
                                80.0 * scale,
                                32.0 * scale,
                                "",
                                5,
                                egui::Align::Center,
                            )
                            .on_hover_text(crate::i18n::tr_catalog(
                                self.language,
                                "macro_editor.delay_is_in_milliseconds",
                            ))
                            .changed()
                            {
                                if let Ok(v) = ms_str.parse::<u16>() {
                                    *ms = v;
                                }
                            }
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if picker_button(
                            ui,
                            "✕",
                            picker_scaled_size(ui.ctx(), 30.0, 30.0),
                            true,
                            false,
                        )
                        .on_hover_text(crate::i18n::tr_catalog(
                            self.language,
                            "macro_editor.remove_this_action",
                        ))
                        .clicked()
                        {
                            remove_idx = Some(i);
                        }
                    });
                });
                ui.add_space(2.0);
            }
        }
        if let Some(idx) = remove_idx {
            if idx < self.macro_actions[n].len() {
                self.macro_undo_stack
                    .push((n, self.macro_actions[n].clone()));
                self.macro_actions[n].remove(idx);
                if let Some((mn, ai)) = self.macro_key_pick {
                    if mn == n && ai >= idx {
                        self.macro_key_pick = None;
                    }
                }
            }
        }
        if let Some(idx) = move_up {
            if idx > 0 {
                self.macro_actions[n].swap(idx, idx - 1);
            }
        }
        if let Some(idx) = move_down {
            if idx + 1 < self.macro_actions[n].len() {
                self.macro_actions[n].swap(idx, idx + 1);
            }
        }

        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
            if picker_button(
                ui,
                crate::i18n::tr_catalog(self.language, "macro_editor.plus_text"),
                picker_scaled_size(ui.ctx(), 72.0, 30.0),
                true,
                false,
            )
            .on_hover_text(crate::i18n::tr_catalog(
                self.language,
                "macro_editor.type_characters",
            ))
            .clicked()
            {
                self.macro_actions[n].push(MacroAction::Text(String::new()));
            }
            if picker_button(
                ui,
                crate::i18n::tr_catalog(self.language, "macro_editor.plus_tap"),
                picker_scaled_size(ui.ctx(), 66.0, 30.0),
                true,
                false,
            )
            .on_hover_text(crate::i18n::tr_catalog(
                self.language,
                "macro_editor.press_and_release_a_key",
            ))
            .clicked()
            {
                self.macro_actions[n].push(MacroAction::Tap(0x04));
                self.macro_key_pick = Some((n, self.macro_actions[n].len() - 1));
            }
            if picker_button(
                ui,
                crate::i18n::tr_catalog(self.language, "macro_editor.plus_down"),
                picker_scaled_size(ui.ctx(), 80.0, 30.0),
                true,
                false,
            )
            .on_hover_text(crate::i18n::tr_catalog(
                self.language,
                "macro_editor.hold_a_key",
            ))
            .clicked()
            {
                self.macro_actions[n].push(MacroAction::Down(0x04));
                self.macro_key_pick = Some((n, self.macro_actions[n].len() - 1));
            }
            if picker_button(
                ui,
                crate::i18n::tr_catalog(self.language, "macro_editor.plus_up"),
                picker_scaled_size(ui.ctx(), 64.0, 30.0),
                true,
                false,
            )
            .on_hover_text(crate::i18n::tr_catalog(
                self.language,
                "macro_editor.release_a_key",
            ))
            .clicked()
            {
                self.macro_actions[n].push(MacroAction::Up(0x04));
                self.macro_key_pick = Some((n, self.macro_actions[n].len() - 1));
            }
            if picker_button(
                ui,
                crate::i18n::tr_catalog(self.language, "macro_editor.plus_delay"),
                picker_scaled_size(ui.ctx(), 82.0, 30.0),
                true,
                false,
            )
            .on_hover_text(crate::i18n::tr_catalog(
                self.language,
                "macro_editor.pause_in_milliseconds",
            ))
            .clicked()
            {
                self.macro_actions[n].push(MacroAction::Delay(100));
            }
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let can_clear_macro = self.macro_has_content(n)
                || self
                    .macro_names
                    .get(n)
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false);
            if picker_button(
                ui,
                crate::i18n::tr_catalog(self.language, "key_picker_text.clear_all"),
                picker_scaled_size(ui.ctx(), 86.0, 30.0),
                can_clear_macro,
                false,
            )
            .on_hover_text(crate::i18n::tr_catalog(
                self.language,
                "key_picker_text.remove_all_actions_from_this_macro",
            ))
            .clicked()
            {
                self.macro_undo_stack
                    .push((n, self.macro_actions[n].clone()));
                self.macro_actions[n].clear();
                if n < self.macro_texts.len() {
                    self.macro_texts[n].clear();
                }
                if n < self.macro_names.len() {
                    self.macro_names[n].clear();
                }
            }
            if picker_button(
                ui,
                crate::i18n::tr_catalog(self.language, "key_picker_text.undo_undo"),
                picker_scaled_size(ui.ctx(), 78.0, 30.0),
                !self.macro_undo_stack.is_empty(),
                false,
            )
            .on_hover_text(crate::i18n::tr_catalog(
                self.language,
                "key_picker_text.undo_last_change",
            ))
            .clicked()
            {
                if let Some((idx, prev)) = self.macro_undo_stack.pop() {
                    if idx < self.macro_actions.len() {
                        self.macro_actions[idx] = prev;
                    }
                }
            }
        });

        selected_macro
    }

    fn macro_has_content(&self, n: usize) -> bool {
        self.macro_actions
            .get(n)
            .map(|a| !a.is_empty())
            .unwrap_or(false)
            || self
                .macro_texts
                .get(n)
                .map(|s| !s.is_empty())
                .unwrap_or(false)
    }

    fn ensure_macro_meta_len(&mut self, n: usize) {
        while self.macro_texts.len() <= n {
            self.macro_texts.push(String::new());
        }
        while self.macro_names.len() <= n {
            self.macro_names.push(String::new());
        }
        while self.macro_actions.len() <= n {
            self.macro_actions.push(vec![]);
        }
    }

    fn macro_display_name(&self, n: usize) -> String {
        match self.macro_names.get(n) {
            Some(name) if !name.trim().is_empty() => name.clone(),
            _ => format!("M{}", n),
        }
    }

    fn macro_custom_name(&self, n: usize) -> Option<String> {
        self.macro_names
            .get(n)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    pub(super) fn encode_macro(&mut self, n: usize) {
        while self.macro_texts.len() <= n {
            self.macro_texts.push(String::new());
        }
        while self.macro_actions.len() <= n {
            self.macro_actions.push(vec![]);
        }
        let mut encoded = Vec::new();
        for action in &self.macro_actions[n] {
            match action {
                MacroAction::Text(s) => encoded.extend_from_slice(s.as_bytes()),
                MacroAction::Tap(kc) => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(*kc);
                }
                MacroAction::Down(kc) => {
                    encoded.push(1);
                    encoded.push(2);
                    encoded.push(*kc);
                }
                MacroAction::Up(kc) => {
                    encoded.push(1);
                    encoded.push(3);
                    encoded.push(*kc);
                }
                MacroAction::Delay(ms) => {
                    let hi = (*ms / 255 + 1) as u8;
                    let lo = (*ms % 255 + 1) as u8;
                    encoded.push(1);
                    encoded.push(4);
                    encoded.push(lo);
                    encoded.push(hi);
                }
            }
        }
        self.macro_texts[n] = String::from_utf8_lossy(&encoded).to_string();
    }

    pub(super) fn show_vial_macros(&mut self, ui: &mut egui::Ui) {
        let previous = self.macro_inline_selected.unwrap_or(0);
        let selected = self.show_macro_editor_contents(
            ui,
            previous,
            "macro_grid_inline",
            "add_action_inline",
            "Saved to device when you close the keycode picker",
        );
        if selected != previous && (previous as usize) < self.macro_count {
            self.encode_macro(previous as usize);
            self.macros_dirty = true;
        }
        self.macro_inline_selected = Some(selected);
    }
}
