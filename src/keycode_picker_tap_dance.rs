use super::*;

impl KeycodePicker {
    pub(super) fn show_vial_tap_dance(&mut self, ui: &mut egui::Ui) {
        if self.tap_dance_entries.is_empty() {
            self.tap_dance_editor_open = None;
            ui.label(
                RichText::new(crate::i18n::tr_catalog(
                    self.language,
                    "tap_dance_editor.no_tap_dance_slots_available_on_this_keyboard",
                ))
                .size(16.0)
                .color(Color32::from_gray(140)),
            );
            return;
        }

        let selected = match self.tap_dance_editor_open {
            Some(n) if (n as usize) < self.tap_dance_entries.len() => n,
            _ => 0,
        };
        self.tap_dance_editor_open = Some(selected);
        self.ensure_tap_dance_name_len(selected as usize);

        ui.label(
            RichText::new(crate::i18n::tr_catalog(
                self.language,
                "tap_dance_editor.choose_tap_dance",
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
                    egui::Grid::new("tap_dance_grid_inline")
                        .num_columns(16)
                        .spacing([4.0, 4.0])
                        .show(ui, |ui| {
                            for n in 0..self.tap_dance_entries.len() as u8 {
                                self.ensure_tap_dance_name_len(n as usize);
                                let is_active = n == selected;
                                let display_name = self.tap_dance_display_name(n as usize);
                                let id_text = format!("TD{}", n);
                                let has_content = {
                                    let td = &self.tap_dance_entries[n as usize];
                                    td.on_tap != 0
                                        || td.on_hold != 0
                                        || td.on_double_tap != 0
                                        || td.on_tap_hold != 0
                                        || td.tapping_term != 200
                                };
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
                                    self.tap_dance_editor_open = Some(n);
                                }
                                if (n + 1) % 16 == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                });
        });
        ui.add_space(crate::ui_style::modal_space_sm());

        let n = self.tap_dance_editor_open.unwrap_or(0) as usize;
        self.ensure_tap_dance_name_len(n);
        let scale = responsive_picker_element_scale(ui.ctx());
        let td_font_size = 14.0 * scale;
        ui.add_space(4.0 * scale);
        let prev_name = self.tap_dance_names.get(n).cloned().unwrap_or_default();
        let mut edited_name = prev_name.clone();
        let resp = crate::ui_style::modern_text_field_sized(
            ui,
            ui.make_persistent_id(("tap_dance_name", n)),
            &mut edited_name,
            124.0 * scale,
            32.0 * scale,
            crate::i18n::tr_catalog(self.language, "tap_dance_editor.td_name"),
            7,
            egui::Align::Center,
        );
        if resp.changed() {
            let trimmed: String = edited_name.chars().take(7).collect();
            if trimmed != prev_name {
                self.push_tap_dance_undo(n);
                self.ensure_tap_dance_name_len(n);
                self.tap_dance_names[n] = trimmed;
                self.tap_dance_dirty = true;
            }
        }
        ui.add_space(8.0);

        let fields = [
            (
                crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_tap"),
                crate::i18n::tr_catalog(self.language, "key_picker_text.key_sent_on_single_tap"),
                0u8,
            ),
            (
                crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_hold"),
                crate::i18n::tr_catalog(self.language, "key_picker_text.key_sent_when_held"),
                1,
            ),
            (
                crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_double_tap"),
                crate::i18n::tr_catalog(self.language, "key_picker_text.key_sent_on_double_tap"),
                2,
            ),
            (
                crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_tap_plus_hold"),
                crate::i18n::tr_catalog(self.language, "key_picker_text.key_sent_on_tap_then_hold"),
                3,
            ),
        ];

        egui::Grid::new("td_fields_inline")
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                for (label, tooltip, field_id) in &fields {
                    ui.add(
                        egui::Label::new(RichText::new(*label).size(td_font_size).strong())
                            .sense(egui::Sense::hover()),
                    )
                    .on_hover_text(*tooltip);

                    let kc = match field_id {
                        0 => self.tap_dance_entries[n].on_tap,
                        1 => self.tap_dance_entries[n].on_hold,
                        2 => self.tap_dance_entries[n].on_double_tap,
                        3 => self.tap_dance_entries[n].on_tap_hold,
                        _ => 0,
                    };
                    let kc_label = if kc == 0 {
                        "None".to_string()
                    } else {
                        crate::keycode::keycode_label_with_names_and_layout(kc, &[], &self.layer_names, self.key_legend_layout)
                    };
                    if picker_button(ui, &kc_label, Vec2::new(120.0, 30.0), true, false)
                        .on_hover_text(if kc == 0 {
                            crate::i18n::tr_catalog(
                                self.language,
                                "tap_dance_editor.click_to_assign_a_key",
                            )
                            .to_string()
                        } else {
                            crate::i18n::tr_text(
                                self.language,
                                &keycode_tooltip(kc, &[], &self.layer_names),
                            )
                        })
                        .clicked()
                    {
                        self.td_key_pick = Some((n, *field_id));
                    }
                    ui.end_row();
                }

                ui.add(
                    egui::Label::new(
                        RichText::new(crate::i18n::tr_catalog(
                            self.language,
                            "tap_dance_editor.tapping_term",
                        ))
                        .size(td_font_size)
                        .strong(),
                    )
                    .sense(egui::Sense::hover()),
                )
                .on_hover_text(crate::i18n::tr_catalog(
                    self.language,
                    "tap_dance_editor.time_in_milliseconds_to_distinguish_tap_from_hold_default_200",
                ));
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    let prev_term = self.tap_dance_entries[n].tapping_term;
                    let mut term_str = prev_term.to_string();
                    if crate::ui_style::modern_text_field_sized(
                        ui,
                        ui.make_persistent_id(("tap_dance_term", n)),
                        &mut term_str,
                        76.0 * scale,
                        32.0 * scale,
                        "",
                        5,
                        egui::Align::Center,
                    )
                    .on_hover_text(crate::i18n::tr_catalog(
                        self.language,
                        "tap_dance_editor.tapping_term_is_in_milliseconds",
                    ))
                    .changed()
                    {
                        if let Ok(v) = term_str.parse::<u16>() {
                            let v = v.clamp(10, 3000);
                            if v != prev_term {
                                self.push_tap_dance_undo(n);
                                self.tap_dance_entries[n].tapping_term = v;
                                self.tap_dance_dirty = true;
                            }
                        }
                    }
                });
                ui.end_row();
            });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let can_clear_tap_dance = self
                .tap_dance_entries
                .get(n)
                .map(|td| {
                    td.on_tap != 0
                        || td.on_hold != 0
                        || td.on_double_tap != 0
                        || td.on_tap_hold != 0
                        || td.tapping_term != 200
                })
                .unwrap_or(false)
                || self
                    .tap_dance_names
                    .get(n)
                    .map(|s| !s.trim().is_empty())
                    .unwrap_or(false);
            if picker_button(
                ui,
                crate::i18n::tr_catalog(self.language, "key_picker_text.clear_all"),
                picker_scaled_size(ui.ctx(), 86.0, 30.0),
                can_clear_tap_dance,
                false,
            )
            .on_hover_text(crate::i18n::tr_catalog(
                self.language,
                "tap_dance_editor.clear_all_actions_for_this_tap_dance",
            ))
            .clicked()
            {
                self.push_tap_dance_undo(n);
                if let Some(td) = self.tap_dance_entries.get_mut(n) {
                    td.on_tap = 0;
                    td.on_hold = 0;
                    td.on_double_tap = 0;
                    td.on_tap_hold = 0;
                    td.tapping_term = 200;
                }
                if n < self.tap_dance_names.len() {
                    self.tap_dance_names[n].clear();
                }
                self.tap_dance_dirty = true;
            }
            let can_undo_current = self
                .tap_dance_undo_stack
                .iter()
                .any(|(idx, _, _)| *idx == n);
            if picker_button(
                ui,
                crate::i18n::tr_catalog(self.language, "key_picker_text.undo_undo"),
                picker_scaled_size(ui.ctx(), 78.0, 30.0),
                can_undo_current,
                false,
            )
            .on_hover_text(crate::i18n::tr_catalog(
                self.language,
                "tap_dance_editor.undo_last_tap_dance_change",
            ))
            .clicked()
            {
                if let Some(pos) = self
                    .tap_dance_undo_stack
                    .iter()
                    .rposition(|(idx, _, _)| *idx == n)
                {
                    let (idx, prev, prev_name) = self.tap_dance_undo_stack.remove(pos);
                    if idx < self.tap_dance_entries.len() {
                        self.tap_dance_entries[idx] = prev;
                    }
                    self.ensure_tap_dance_name_len(idx);
                    if idx < self.tap_dance_names.len() {
                        self.tap_dance_names[idx] = prev_name;
                    }
                    self.tap_dance_editor_open = Some(idx as u8);
                    self.tap_dance_dirty = true;
                }
            }
        });
    }

    fn show_tap_dance_editor(&mut self, ctx: &egui::Context, active_td: u8) {
        if ctx.input(|i| i.key_pressed(Key::Escape)) && self.td_key_pick.is_none() {
            self.tap_dance_editor_open = None;
            self.tap_dance_dirty = true;
            if active_td < self.tap_dance_entries.len() as u8 {
                self.result = Some(0x5700 + active_td as u16); // TD keycode
            }
            self.open = false;
            return;
        }

        let mut still_open = true;
        crate::ui_style::centered_modal_window(
            ctx,
            crate::i18n::tr_catalog(self.language, "tap_dance_editor.tap_dance_editor"),
            self.popup_state.id(PopupKey::TapDanceEditorWindow),
            &mut still_open,
            responsive_window_size(ctx, Vec2::new(680.0, 480.0), Vec2::new(980.0, 720.0)),
        )
        .show(ctx, |ui| {
            // Tabs
            ui.horizontal_wrapped(|ui| {
                for n in 0..self.tap_dance_entries.len() as u8 {
                    let is_active = n == active_td;
                    let label = format!("TD{}", n);
                    let btn =
                        egui::Button::new(RichText::new(&label).size(14.0).color(if is_active {
                            Color32::WHITE
                        } else {
                            Color32::from_gray(100)
                        }))
                        .fill(if is_active {
                            crate::ui_style::accent()
                        } else {
                            Color32::TRANSPARENT
                        })
                        .min_size(crate::ui_style::modal_tab_button_size());
                    if ui.add(btn).clicked() {
                        self.tap_dance_editor_open = Some(n);
                    }
                }
            });
            ui.separator();

            if self.tap_dance_entries.is_empty() {
                ui.label(
                    RichText::new(crate::i18n::tr_catalog(
                        self.language,
                        "tap_dance_editor.no_tap_dance_slots_available_on_this_keyboard",
                    ))
                    .size(16.0)
                    .color(Color32::from_gray(140)),
                );
                return;
            }

            if active_td == 255 || active_td as usize >= self.tap_dance_entries.len() {
                ui.label(
                    RichText::new(crate::i18n::tr_catalog(
                        self.language,
                        "key_picker_text.select_a_tap_dance_tab_above_to_edit",
                    ))
                    .size(16.0)
                    .color(Color32::from_gray(140)),
                );
                return;
            }

            let scale = responsive_picker_element_scale(ui.ctx());
            let n = active_td as usize;
            ui.label(
                RichText::new(format!("TD{}", n))
                    .size(18.0 * scale)
                    .strong(),
            );
            ui.add_space(8.0 * scale);

            let fields = [
                (
                    crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_tap"),
                    crate::i18n::tr_catalog(
                        self.language,
                        "key_picker_text.key_sent_on_single_tap",
                    ),
                    0u8,
                ),
                (
                    crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_hold"),
                    crate::i18n::tr_catalog(self.language, "key_picker_text.key_sent_when_held"),
                    1,
                ),
                (
                    crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_double_tap"),
                    crate::i18n::tr_catalog(
                        self.language,
                        "key_picker_text.key_sent_on_double_tap",
                    ),
                    2,
                ),
                (
                    crate::i18n::tr_catalog(self.language, "tap_dance_editor.on_tap_plus_hold"),
                    crate::i18n::tr_catalog(
                        self.language,
                        "key_picker_text.key_sent_on_tap_then_hold",
                    ),
                    3,
                ),
            ];

            egui::Grid::new("td_fields")
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    for (label, tooltip, field_id) in &fields {
                        ui.add(
                            egui::Label::new(RichText::new(*label).size(15.0 * scale).strong())
                                .sense(egui::Sense::hover()),
                        )
                        .on_hover_text(*tooltip);

                        let kc = match field_id {
                            0 => self.tap_dance_entries[n].on_tap,
                            1 => self.tap_dance_entries[n].on_hold,
                            2 => self.tap_dance_entries[n].on_double_tap,
                            3 => self.tap_dance_entries[n].on_tap_hold,
                            _ => 0,
                        };
                        let kc_label = if kc == 0 {
                            "None".to_string()
                        } else {
                            crate::keycode::keycode_label_with_names_and_layout(kc, &[], &self.layer_names, self.key_legend_layout)
                        };
                        if ui
                            .add(
                                egui::Button::new(RichText::new(&kc_label).size(16.0))
                                    .min_size(picker_scaled_size(ui.ctx(), 132.0, 30.0)),
                            )
                            .on_hover_text(if kc == 0 {
                                crate::i18n::tr_catalog(
                                    self.language,
                                    "tap_dance_editor.click_to_assign_a_key",
                                )
                                .to_string()
                            } else {
                                crate::i18n::tr_text(
                                    self.language,
                                    &keycode_tooltip(kc, &[], &self.layer_names),
                                )
                            })
                            .clicked()
                        {
                            self.td_key_pick = Some((n, *field_id));
                        }
                        ui.end_row();
                    }

                    // Tapping term
                    ui.add(
                        egui::Label::new(
                            RichText::new(crate::i18n::tr_catalog(
                                self.language,
                                "tap_dance_editor.tapping_term",
                            ))
                            .size(15.0 * scale)
                            .strong(),
                        )
                        .sense(egui::Sense::hover()),
                    )
                    .on_hover_text(crate::i18n::tr_catalog(
                        self.language,
                        "tap_dance_editor.time_in_milliseconds_to_distinguish_tap_from_hold_default_200",
                    ));
                    let mut term_str = self.tap_dance_entries[n].tapping_term.to_string();
                    ui.horizontal(|ui| {
                        if crate::ui_style::modern_text_field_sized(
                            ui,
                            ui.make_persistent_id(("tap_dance_legacy_term", n)),
                            &mut term_str,
                            80.0 * scale,
                            32.0 * scale,
                            "",
                            5,
                            egui::Align::Center,
                        )
                        .on_hover_text(crate::i18n::tr_catalog(
                            self.language,
                            "tap_dance_editor.tapping_term_is_in_milliseconds",
                        ))
                        .changed()
                        {
                            if let Ok(v) = term_str.parse::<u16>() {
                                self.tap_dance_entries[n].tapping_term = v;
                            }
                        }
                    });
                    ui.end_row();
                });
        });

        if !still_open {
            if active_td < self.tap_dance_entries.len() as u8 {
                self.result = Some(0x5700 + active_td as u16);
            }
            self.tap_dance_editor_open = None;
            self.tap_dance_dirty = true;
            self.open = false;
        }
    }

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

    fn ensure_tap_dance_name_len(&mut self, n: usize) {
        while self.tap_dance_names.len() <= n {
            self.tap_dance_names.push(String::new());
        }
    }

    fn tap_dance_display_name(&self, n: usize) -> String {
        match self.tap_dance_names.get(n) {
            Some(name) if !name.trim().is_empty() => name.clone(),
            _ => format!("TD{}", n),
        }
    }

    fn push_tap_dance_undo(&mut self, n: usize) {
        self.ensure_tap_dance_name_len(n);
        if let Some(td) = self.tap_dance_entries.get(n).cloned() {
            let name = self.tap_dance_names.get(n).cloned().unwrap_or_default();
            self.tap_dance_undo_stack.push((n, td, name));
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
