use super::*;

impl EntropyApp {
    pub(super) fn draw_layout_layer_switcher_and_hints(
        &mut self,
        ui: &mut egui::Ui,
        top_base_y: f32,
        main_tabs_h: f32,
        layer_bar_h: f32,
    ) {
        // ── Layer switcher ─────────────────────────────────────────────────
        {
            let layer_count = self.layer_count;
            let selected = self.selected_layer;
            // raw_name — чистое имя без префикса, хранится в layer_names
            let raw_name = self
                .layer_names
                .get(selected)
                .cloned()
                .unwrap_or_else(|| selected.to_string());
            let visible_raw_name: String = raw_name.chars().take(12).collect();
            // display_name — с префиксом для отображения
            let display_name = if !raw_name.is_empty() && raw_name != selected.to_string() {
                format!("{}. {}", selected, visible_raw_name)
            } else {
                visible_raw_name.clone()
            };
            let name = display_name;
            let center_x = ui.max_rect().center().x;
            let bar_y = top_base_y + main_tabs_h + 24.0;
            let any_top_dropdown_open = ui.memory(|m| {
                m.data
                    .get_temp::<bool>(ui.make_persistent_id("device_dropdown_open"))
                    .unwrap_or(false)
                    || m.data
                        .get_temp::<bool>(ui.make_persistent_id("advanced_dropdown_open"))
                        .unwrap_or(false)
                    || m.data
                        .get_temp::<bool>(ui.make_persistent_id("settings_dropdown_open"))
                        .unwrap_or(false)
            });

            // Layer name / edit field
            let name_rect = egui::Rect::from_min_size(
                egui::pos2(center_x - 85.0, bar_y),
                Vec2::new(170.0, 52.0),
            );
            self.register_tour_target(
                TourTarget::LayerSwitcher,
                name_rect.expand2(Vec2::new(72.0, 8.0)),
            );

            let display_name_len = visible_raw_name.chars().count();
            let display_label_size = if display_name_len > 10 {
                26.0
            } else if display_name_len > 7 {
                31.0
            } else {
                39.0
            };
            let label_font = egui::FontId {
                size: display_label_size,
                family: egui::FontFamily::Proportional,
            };
            let text_color = if self.dark_mode {
                Color32::from_gray(245)
            } else {
                Color32::from_gray(60)
            };

            if self.editing_layer == Some(selected) {
                // Limit input to 12 chars
                if self.editing_layer_text.chars().count() > 12 {
                    let s: String = self.editing_layer_text.chars().take(12).collect();
                    self.editing_layer_text = s;
                }
                let editing_font = egui::FontId {
                    size: 39.0,
                    family: egui::FontFamily::Proportional,
                };
                let resp = ui.put(
                    name_rect,
                    egui::TextEdit::singleline(&mut self.editing_layer_text)
                        .font(editing_font)
                        .horizontal_align(egui::Align::Center)
                        .char_limit(12)
                        .frame(false),
                );
                // Request focus only on the first frame so lost_focus() works correctly.
                if !self.editing_layer_focus_requested {
                    resp.request_focus();
                    self.editing_layer_focus_requested = true;
                }
                // Commit on Enter or lost focus (click outside); cancel on Escape.
                let commit = resp.lost_focus() || ui.input(|inp| inp.key_pressed(egui::Key::Enter));
                let cancel = ui.input(|inp| inp.key_pressed(egui::Key::Escape));
                if commit || cancel {
                    if commit {
                        let proposed_name = self.editing_layer_text.trim().to_string();
                        if proposed_name.is_empty() {
                            self.editing_layer_text = raw_name.clone();
                        } else {
                            let new_name = proposed_name;
                            while self.layer_names.len() <= selected {
                                self.layer_names.push(self.layer_names.len().to_string());
                            }
                            self.layer_names[selected] = new_name.clone();
                            #[cfg(not(target_arch = "wasm32"))]
                            save_layer_names(&self.layer_names, &self.current_device_name);
                            #[cfg(target_arch = "wasm32")]
                            save_layer_names(&self.layer_names, "default");
                            // Also write name back to the connected device
                            #[cfg(not(target_arch = "wasm32"))]
                            if self.firmware == FirmwareProtocol::Vial {
                                if let Some(dev) = &self.hid_device {
                                    if let Err(e) =
                                        dev.set_qmk_setting_string(200 + selected as u16, &new_name)
                                    {
                                        log::warn!(
                                            "Vial set_qmk_setting_string failed for layer {}: {}",
                                            selected,
                                            e
                                        );
                                    }
                                }
                            }
                        }
                    }
                    self.editing_layer = None;
                    self.editing_layer_focus_requested = false;
                }
            } else {
                let mid_y = bar_y + layer_bar_h / 2.0;

                // Fixed arrow positions based on max 7-char name width so
                // arrows never jump around as the layer name changes.
                // name_rect is 170px wide → half = 85px; gap keeps arrows clear.
                let fixed_half = 85.0_f32;
                let gap = 16.0_f32;
                let arrow_y = mid_y - 2.0;
                let left_center = egui::pos2(center_x - fixed_half - gap - 24.0, arrow_y);
                let right_center = egui::pos2(center_x + fixed_half + gap + 24.0, arrow_y);

                // Still measure actual text width for painting the name and edit icon.
                let text_w = ui.fonts(|f| {
                    f.layout_no_wrap(name.clone(), label_font.clone(), text_color)
                        .size()
                        .x
                });

                // Allocate name FIRST — arrows are allocated last and win in egui's
                // hit-test order (last allocation = highest priority).
                let name_hit = egui::Rect::from_center_size(
                    egui::pos2(center_x, mid_y),
                    Vec2::new(text_w + 12.0, 52.0),
                );
                let name_r = ui.allocate_rect(name_hit, Sense::click());

                // Full layer switch zone from arrow to arrow for mouse wheel switching.
                // Keep click/hover hitboxes close to the actual arrow glyph size.
                let left_hit = egui::Rect::from_center_size(left_center, Vec2::new(28.0, 44.0));
                let right_hit = egui::Rect::from_center_size(right_center, Vec2::new(28.0, 44.0));
                let wheel_hit = egui::Rect::from_min_max(
                    egui::pos2(left_hit.left(), mid_y - 26.0),
                    egui::pos2(right_hit.right(), mid_y + 26.0),
                );
                let wheel_r = ui.allocate_rect(wheel_hit, Sense::hover());

                // Scroll wheel over the whole layer bar switches layers (down = next, up = prev)
                if wheel_r.hovered() {
                    let scroll = ui.input(|i| i.raw_scroll_delta.y);
                    if scroll < 0.0 && selected > 0 {
                        self.selected_layer = selected - 1;
                    } else if scroll > 0.0 && selected + 1 < layer_count {
                        self.selected_layer = selected + 1;
                    }
                }

                // Allocate arrows LAST so they have click priority over the name rect.
                let left_r = ui.allocate_rect(left_hit, Sense::click());
                let right_r = ui.allocate_rect(right_hit, Sense::click());
                if left_r.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if right_r.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if left_r.clicked() && selected > 0 {
                    self.selected_layer = selected - 1;
                    self.jump_back_stack.clear();
                }
                if right_r.clicked() && selected + 1 < layer_count {
                    self.selected_layer = selected + 1;
                    self.jump_back_stack.clear();
                }
                if name_r.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if name_r.clicked() {
                    self.editing_layer = Some(selected);
                    self.editing_layer_text = raw_name.clone();
                }

                // Paint
                let dis = if self.dark_mode {
                    Color32::from_gray(60)
                } else {
                    Color32::from_gray(200)
                };
                let ac_l = if left_r.hovered() {
                    app_accent()
                } else if self.dark_mode {
                    Color32::from_gray(140)
                } else {
                    Color32::from_gray(120)
                };
                let ac_r = if right_r.hovered() {
                    app_accent()
                } else if self.dark_mode {
                    Color32::from_gray(140)
                } else {
                    Color32::from_gray(120)
                };
                ui.painter().text(
                    left_center,
                    egui::Align2::CENTER_CENTER,
                    "‹",
                    FontId::proportional(52.0),
                    if selected == 0 { dis } else { ac_l },
                );
                ui.painter().text(
                    right_center,
                    egui::Align2::CENTER_CENTER,
                    "›",
                    FontId::proportional(52.0),
                    if selected + 1 >= layer_count {
                        dis
                    } else {
                        ac_r
                    },
                );
                ui.painter().text(
                    egui::pos2(center_x, mid_y),
                    egui::Align2::CENTER_CENTER,
                    &name,
                    label_font,
                    text_color,
                );

                // Hint text below layer name
                let hint_color = if self.dark_mode {
                    Color32::from_gray(100)
                } else {
                    Color32::from_gray(160)
                };
                let hint_font = FontId::proportional(11.0);
                let secondary_hint_font = hint_font.clone();
                let hint_y = ui.max_rect().bottom() - 36.0;
                let any_hovered = self.prev_hovered_key.is_some() || self.prev_hovered_encoder;
                let hint_language = self.app_settings.language;
                let tr_hint = |key: &'static str| crate::i18n::tr_catalog(hint_language, key);
                if let Some(hl) = self.hover_layer {
                    let hl_name = self
                        .layer_names
                        .get(hl)
                        .cloned()
                        .unwrap_or_else(|| hl.to_string());
                    let mut line = 0i32;
                    let line_h = 13.0f32;
                    let base_y = hint_y - 15.0;
                    // Line 1: always
                    ui.painter().text(
                        egui::pos2(center_x, base_y + line as f32 * line_h),
                        egui::Align2::CENTER_CENTER,
                        tr_hint("key_hints.change_key"),
                        hint_font.clone(),
                        hint_color,
                    );
                    line += 1;
                    // Line 2: go to layer (if not current)
                    if hl != self.selected_layer {
                        let layer_index = hl.to_string();
                        let go_to_layer_hint = crate::i18n::tr_catalog_format(
                            hint_language,
                            "key_hints.go_to_layer",
                            &[("layer", layer_index.as_str()), ("name", hl_name.as_str())],
                        );
                        ui.painter().text(
                            egui::pos2(center_x, base_y + line as f32 * line_h),
                            egui::Align2::CENTER_CENTER,
                            go_to_layer_hint,
                            hint_font.clone(),
                            hint_color,
                        );
                        line += 1;
                    }
                    // Line 3: change layer number
                    ui.painter().text(
                        egui::pos2(center_x, base_y + line as f32 * line_h),
                        egui::Align2::CENTER_CENTER,
                        tr_hint("key_hints.change_layer_number"),
                        hint_font.clone(),
                        hint_color,
                    );
                    line += 1;
                    // Line 4: go back (if in jump mode)
                    if !self.jump_back_stack.is_empty() {
                        ui.painter().text(
                            egui::pos2(center_x, base_y + line as f32 * line_h),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.esc_back"),
                            hint_font.clone(),
                            hint_color,
                        );
                    }
                    let _ = hint_font;
                } else if !self.jump_back_stack.is_empty() {
                    if any_hovered {
                        ui.painter().text(
                            egui::pos2(center_x, hint_y - 9.0),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.change_key"),
                            hint_font.clone(),
                            hint_color,
                        );
                    }
                    ui.painter().text(
                        egui::pos2(center_x, if any_hovered { hint_y + 5.0 } else { hint_y }),
                        egui::Align2::CENTER_CENTER,
                        tr_hint("key_hints.right_click_or_esc_back"),
                        hint_font,
                        hint_color,
                    );
                } else if any_hovered {
                    // Check if hovered key is a mod key
                    let (
                        hovered_is_mod,
                        hovered_can_swap_side,
                        hovered_can_retarget_mod_key,
                        hovered_is_macro,
                        hovered_is_tap_dance,
                        hovered_is_mouse,
                        hovered_is_alt_repeat,
                        hovered_is_grave_escape,
                        hovered_is_layer,
                    ) = {
                        let hint_kc = self
                            .prev_hovered_key
                            .and_then(|ki| {
                                self.layout
                                    .as_ref()
                                    .map(|l| l.get_keycode(self.selected_layer, ki))
                            })
                            .or(self.prev_hovered_encoder_keycode)
                            .or_else(|| {
                                self.selected_key.and_then(|(selected_layer, selected_ki)| {
                                    (selected_layer == self.selected_layer)
                                        .then(|| {
                                            self.layout.as_ref().map(|l| {
                                                l.get_keycode(self.selected_layer, selected_ki)
                                            })
                                        })
                                        .flatten()
                                })
                            });
                        hint_kc
                            .map(|kc| {
                                let is_plain_mod = (0x00E0..=0x00E7).contains(&kc)
                                    || matches!(
                                        kc,
                                        0x52A1
                                            | 0x52A2
                                            | 0x52A4
                                            | 0x52A7
                                            | 0x52A8
                                            | 0x52AF
                                            | 0x52B1
                                            | 0x52B2
                                            | 0x52B4
                                            | 0x52B8
                                    );
                                let is_mod = is_plain_mod
                                    || (kc >= 0x2000 && kc < 0x4000)
                                    || (kc >= 0x0100 && kc < 0x2000 && (kc & 0xFF) != 0);
                                let can_swap_side = toggle_handed_modifier(kc).is_some();
                                let is_macro = kc >= 0x7700 && kc <= 0x77FF;
                                let is_tap_dance = kc >= 0x5700 && kc <= 0x57FF;
                                let is_mouse = is_mouse_keycode(kc);
                                let is_alt_repeat = is_alt_repeat_keycode(kc);
                                let is_grave_escape = kc == 0x7C16;
                                let is_layer = vial_layer_target(kc).is_some();
                                let can_retarget_mod_key = !is_layer
                                    && ((kc >= 0x2000 && kc < 0x4000)
                                        || (kc >= 0x0100 && kc < 0x2000 && (kc & 0xFF) != 0));
                                (
                                    is_mod,
                                    can_swap_side,
                                    can_retarget_mod_key,
                                    is_macro,
                                    is_tap_dance,
                                    is_mouse,
                                    is_alt_repeat,
                                    is_grave_escape,
                                    is_layer,
                                )
                            })
                            .unwrap_or((
                                false, false, false, false, false, false, false, false, false,
                            ))
                    };
                    if hovered_is_mod {
                        if hovered_can_swap_side {
                            let show_retarget = hovered_can_retarget_mod_key;
                            ui.painter().text(
                                egui::pos2(
                                    center_x,
                                    if show_retarget {
                                        hint_y - 22.0
                                    } else {
                                        hint_y - 10.0
                                    },
                                ),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.change_key"),
                                hint_font.clone(),
                                hint_color,
                            );
                            if show_retarget {
                                ui.painter().text(
                                    egui::pos2(center_x, hint_y - 4.0),
                                    egui::Align2::CENTER_CENTER,
                                    tr_hint("key_hints.change_modifier_key"),
                                    secondary_hint_font.clone(),
                                    hint_color,
                                );
                            }
                            ui.painter().text(
                                egui::pos2(
                                    center_x,
                                    if show_retarget {
                                        hint_y + 12.0
                                    } else {
                                        hint_y + 8.0
                                    },
                                ),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.switch_modifier_side"),
                                secondary_hint_font,
                                hint_color,
                            );
                        } else {
                            ui.painter().text(
                                egui::pos2(center_x, hint_y - 14.0),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.change_key"),
                                hint_font.clone(),
                                hint_color,
                            );
                            ui.painter().text(
                                egui::pos2(center_x, hint_y + 4.0),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.change_modifier_key"),
                                secondary_hint_font,
                                hint_color,
                            );
                        }
                    } else if hovered_is_macro {
                        ui.painter().text(
                            egui::pos2(center_x, hint_y - 14.0),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.change_key"),
                            hint_font.clone(),
                            hint_color,
                        );
                        ui.painter().text(
                            egui::pos2(center_x, hint_y + 4.0),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.edit_macro"),
                            secondary_hint_font.clone(),
                            hint_color,
                        );
                    } else if hovered_is_tap_dance {
                        ui.painter().text(
                            egui::pos2(center_x, hint_y - 14.0),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.change_key"),
                            hint_font.clone(),
                            hint_color,
                        );
                        ui.painter().text(
                            egui::pos2(center_x, hint_y + 4.0),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.edit_tap_dance"),
                            secondary_hint_font,
                            hint_color,
                        );
                    } else if hovered_is_mouse {
                        ui.painter().text(
                            egui::pos2(center_x, hint_y - 14.0),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.change_key"),
                            hint_font.clone(),
                            hint_color,
                        );
                        ui.painter().text(
                            egui::pos2(center_x, hint_y + 4.0),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.open_mouse_keys"),
                            secondary_hint_font.clone(),
                            hint_color,
                        );
                    } else if hovered_is_alt_repeat {
                        ui.painter().text(
                            egui::pos2(center_x, hint_y - 14.0),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.change_key"),
                            hint_font.clone(),
                            hint_color,
                        );
                        ui.painter().text(
                            egui::pos2(center_x, hint_y + 4.0),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.open_alt_repeat"),
                            secondary_hint_font,
                            hint_color,
                        );
                    } else if hovered_is_grave_escape {
                        ui.painter().text(
                            egui::pos2(center_x, hint_y - 14.0),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.change_key"),
                            hint_font.clone(),
                            hint_color,
                        );
                        ui.painter().text(
                            egui::pos2(center_x, hint_y + 4.0),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.open_grave_escape"),
                            secondary_hint_font,
                            hint_color,
                        );
                    } else if hovered_is_layer {
                        ui.painter().text(
                            egui::pos2(center_x, hint_y - 22.0),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.change_key"),
                            hint_font.clone(),
                            hint_color,
                        );
                        ui.painter().text(
                            egui::pos2(center_x, hint_y - 4.0),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.go_to_that_layer"),
                            secondary_hint_font.clone(),
                            hint_color,
                        );
                        ui.painter().text(
                            egui::pos2(center_x, hint_y + 12.0),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.change_layer_target"),
                            secondary_hint_font,
                            hint_color,
                        );
                    } else {
                        ui.painter().text(
                            egui::pos2(center_x, hint_y),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.change_key"),
                            hint_font,
                            hint_color,
                        );
                    }
                } else if name_r.hovered() {
                    ui.painter().text(
                        egui::pos2(center_x, hint_y),
                        egui::Align2::CENTER_CENTER,
                        tr_hint("key_hints.rename_layer"),
                        hint_font,
                        hint_color,
                    );
                }
            }
        }
    }
}
