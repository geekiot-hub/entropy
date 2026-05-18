use super::*;

impl KeycodePicker {
    pub(super) fn show_td_key_picker(&mut self, ctx: &egui::Context, td_idx: usize, field: u8) {
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.td_key_pick = None;
            return;
        }

        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } = event
                {
                    if let Some(qmk) = egui_key_to_qmk(*key, *modifiers) {
                        if qmk > 0 && qmk < 0x0100 {
                            self.set_tap_dance_field(td_idx, field, qmk);
                            self.td_key_pick = None;
                        }
                    }
                }
            }
        });

        let field_name = match field {
            0 => crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_tap"),
            1 => crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_hold"),
            2 => crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_double_tap"),
            3 => crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_tap_plus_hold"),
            _ => "?",
        };
        let helper_text = match field {
            0 => crate::i18n::tr_catalog(
                self.language,
                "key_picker_text.best_for_normal_keys_navigation_media_and_special_actions",
            ),
            1 => crate::i18n::tr_catalog(self.language, "key_picker_text.hold_actions_are_limited_to_left_right_modifiers_and_layers"),
            2 => crate::i18n::tr_catalog(self.language, "key_picker_text.best_for_a_second_tap_action_usually_another_normal_key_or_command"),
            3 => crate::i18n::tr_catalog(self.language, "key_picker_text.tap_then_hold_actions_are_limited_to_left_right_modifiers_and_layers"),
            _ => tr_picker(self.language, "key_picker.press_key_or_click_cancel"),
        };
        let td_choices: Vec<(u16, String, String)> = if matches!(field, 1 | 3) {
            let gui = gui_label(false).to_string();
            let mut out: Vec<(u16, String, String)> = vec![
                (
                    0x00E0,
                    "Left\nCtrl".into(),
                    crate::i18n::tr_catalog(self.language, "key_picker_text.left_control").into(),
                ),
                (
                    0x00E4,
                    "Right\nCtrl".into(),
                    crate::i18n::tr_catalog(self.language, "key_picker_text.right_control").into(),
                ),
                (
                    0x00E1,
                    "Left\nShift".into(),
                    crate::i18n::tr_catalog(self.language, "key_picker_text.left_shift").into(),
                ),
                (
                    0x00E5,
                    "Right\nShift".into(),
                    crate::i18n::tr_catalog(self.language, "key_picker_text.right_shift").into(),
                ),
                (
                    0x00E2,
                    "Left\nAlt".into(),
                    crate::i18n::tr_catalog(self.language, "key_picker_text.left_alt").into(),
                ),
                (
                    0x00E6,
                    "Right\nAlt".into(),
                    crate::i18n::tr_catalog(self.language, "key_picker_text.right_alt").into(),
                ),
                (
                    0x00E3,
                    format!("Left\n{}", gui),
                    crate::i18n::tr_text(self.language, &format!("Left {}", gui)),
                ),
                (
                    0x00E7,
                    format!("Right\n{}", gui),
                    crate::i18n::tr_text(self.language, &format!("Right {}", gui)),
                ),
            ];
            out.extend(
                self.tap_dance_layer_choices()
                    .into_iter()
                    .map(|(value, _label)| {
                        let layer = (value & 0x1F) as usize;
                        let layer_name = self
                            .layer_names
                            .get(layer)
                            .cloned()
                            .unwrap_or_else(|| layer.to_string());
                        (
                            value,
                            format!("MO({})\n{}", layer, layer_name),
                            crate::i18n::tr_text(
                                self.language,
                                &format!("Momentarily activate layer {} while held", layer_name),
                            ),
                        )
                    }),
            );
            out
        } else {
            KEYCODES
                .iter()
                .filter(|kc| {
                    kc.value != 0
                        && kc.value != 0x0001
                        && !kc.name.starts_with("RGB_")
                        && matches!(
                            kc.category,
                            KeycodeCategory::Basic
                                | KeycodeCategory::Function
                                | KeycodeCategory::Navigation
                                | KeycodeCategory::Media
                                | KeycodeCategory::Special
                        )
                        && !is_f13_to_f24(kc.value)
                })
                .map(|kc| {
                    (
                        kc.value,
                        keycode_label_with_names_and_layout(
                            kc.value,
                            &[],
                            &self.layer_names,
                            self.key_legend_layout,
                        ),
                        keycode_tooltip(kc.value, &[], &self.layer_names),
                    )
                })
                .collect()
        };
        let mut still_open = true;
        let popup_size = key_picker_popup_size(ctx);
        let window_title = crate::i18n::tr_catalog_format(
            self.language,
            "key_picker.pick_key_for",
            &[("field", field_name)],
        );
        crate::ui_style::centered_modal_window(
            ctx,
            &window_title,
            self.popup_state.id(PopupKey::TdKeyPickWindow),
            &mut still_open,
            popup_size,
        )
        .show(ctx, |ui| {
            apply_picker_button_visuals(ui);
            crate::ui_style::modal_intro(
                ui,
                tr_picker(self.language, "key_picker.press_key_or_click_cancel"),
            );
            crate::ui_style::modal_hint(ui, helper_text);
            ui.add_space(crate::ui_style::modal_space_xs());
            if picker_button(
                ui,
                tr_picker(self.language, "key_picker.none_clear"),
                crate::ui_style::modal_action_button_size(),
                true,
                false,
            )
            .clicked()
            {
                self.set_tap_dance_field(td_idx, field, 0);
                self.td_key_pick = None;
            }
            ui.add_space(4.0);
            egui::ScrollArea::vertical()
                .max_height(key_picker_popup_scroll_height(popup_size))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if matches!(field, 1 | 3) {
                        let modifier_choices: Vec<(u16, String, String)> =
                            td_choices.iter().take(8).cloned().collect();
                        let layer_choices: Vec<(u16, String, String)> =
                            td_choices.iter().skip(8).cloned().collect();
                        let groups =
                            vec![("Modifiers", modifier_choices), ("Layers", layer_choices)];
                        if let Some(value) =
                            show_grouped_popup_choice_buttons(ui, groups, self.language)
                        {
                            self.set_tap_dance_field(td_idx, field, value);
                            self.td_key_pick = None;
                        }
                    } else {
                        let key_choices: Vec<&'static crate::keycode::Keycode> = KEYCODES
                            .iter()
                            .filter(|kc| {
                                kc.value != 0
                                    && kc.value != 0x0001
                                    && !kc.name.starts_with("RGB_")
                                    && matches!(
                                        kc.category,
                                        KeycodeCategory::Basic
                                            | KeycodeCategory::Function
                                            | KeycodeCategory::Navigation
                                            | KeycodeCategory::Media
                                            | KeycodeCategory::Special
                                    )
                                    && !is_f13_to_f24(kc.value)
                            })
                            .collect();
                        if let Some(value) = show_grouped_popup_key_buttons(
                            ui,
                            key_choices,
                            &self.layer_names,
                            false,
                            self.language,
                            self.key_legend_layout,
                        ) {
                            self.set_tap_dance_field(td_idx, field, value);
                            self.td_key_pick = None;
                        }
                    }
                });
        });
        if !still_open {
            self.td_key_pick = None;
        }
    }

    fn tap_dance_layer_choices(&self) -> Vec<(u16, String)> {
        let count = self.layer_count.max(1).min(self.layer_names.len().max(1));
        (0..count)
            .map(|layer| {
                let name = self
                    .layer_names
                    .get(layer)
                    .cloned()
                    .unwrap_or_else(|| layer.to_string());
                (0x5220 | layer as u16, format!("MO({})", name))
            })
            .collect()
    }

    fn set_tap_dance_field(&mut self, n: usize, field: u8, value: u16) {
        if n >= self.tap_dance_entries.len() {
            return;
        }
        let current = match self.tap_dance_entries.get(n) {
            Some(td) => match field {
                0 => td.on_tap,
                1 => td.on_hold,
                2 => td.on_double_tap,
                3 => td.on_tap_hold,
                _ => return,
            },
            None => return,
        };
        if current == value {
            return;
        }
        self.push_tap_dance_undo(n);
        if let Some(td) = self.tap_dance_entries.get_mut(n) {
            match field {
                0 => td.on_tap = value,
                1 => td.on_hold = value,
                2 => td.on_double_tap = value,
                3 => td.on_tap_hold = value,
                _ => {}
            }
        }
        self.tap_dance_dirty = true;
    }
}
