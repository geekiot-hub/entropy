use super::*;

impl KeycodePicker {
    fn basic_key_button_at(
        &mut self,
        ui: &mut egui::Ui,
        origin: egui::Pos2,
        cell_w: f32,
        cell_h: f32,
        gap: f32,
        row: usize,
        col: usize,
        span: usize,
        label: &str,
        value: u16,
    ) {
        let x = origin.x + col as f32 * (cell_w + gap);
        let right_nav_extra_gap = if col >= 16 && matches!(row, 1 | 2) {
            14.0
        } else {
            0.0
        };
        let y = origin.y + row as f32 * (cell_h + gap) + right_nav_extra_gap;
        let width = span as f32 * cell_w + span.saturating_sub(1) as f32 * gap;
        let rect = egui::Rect::from_min_size(egui::pos2(x, y), Vec2::new(width, cell_h));
        let resp = picker_keycap_button_in_rect(ui, rect, label, true, false);
        if resp.clicked() {
            self.assign_keycode_value(value);
        }
        if resp.hovered() {
            resp.on_hover_text(crate::i18n::tr_text(
                self.language,
                &keycode_tooltip(value, &[], &self.layer_names),
            ));
        }
    }

    pub(super) fn show_vial_basic(&mut self, ui: &mut egui::Ui) {
        const COLS: usize = 16;
        const ROWS: usize = 6;

        let scale = responsive_picker_element_scale(ui.ctx());
        let cell_w = 54.0 * scale;
        let cell_h = 54.0 * scale;
        let gap = 3.0 * scale;
        let width = COLS as f32 * cell_w + (COLS.saturating_sub(1)) as f32 * gap;
        let height = ROWS as f32 * cell_h + (ROWS.saturating_sub(1)) as f32 * gap;
        let available_width = ui.available_width();
        let x_offset = ((available_width - width).max(0.0) * 0.5).floor();

        ui.horizontal(|ui| {
            if x_offset > 0.0 {
                ui.add_space(x_offset);
            }
            ui.allocate_ui_with_layout(
                Vec2::new(width, 32.0 * scale),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.label(
                        RichText::new(tr_picker(self.language, "key_picker.section_basic"))
                            .size(11.0 * scale)
                            .color(Color32::from_gray(150)),
                    );
                    let dropdown_width = 126.0 * scale;
                    let spacer = (ui.available_width() - dropdown_width).max(0.0);
                    if spacer > 0.0 {
                        ui.add_space(spacer);
                    }
                    let dropdown_id = ui.make_persistent_id("basic_layout_dropdown");
                    let dropdown_resp = crate::ui_style::modern_dropdown_button(
                        ui,
                        dropdown_id,
                        self.basic_layout.label(),
                        ui.visuals().text_color(),
                        dropdown_width,
                    );
                    egui::popup_below_widget(
                        ui,
                        dropdown_id,
                        &dropdown_resp,
                        egui::PopupCloseBehavior::CloseOnClickOutside,
                        |ui| {
                            ui.set_min_width(dropdown_width);
                            ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
                            for layout in BasicPickerLayout::ALL {
                                let selected = self.basic_layout == layout;
                                let (option_rect, option_resp) = ui.allocate_exact_size(
                                    Vec2::new(dropdown_width, 28.0 * scale),
                                    egui::Sense::click(),
                                );
                                if option_resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                let fill = if selected {
                                    if ui.visuals().dark_mode {
                                        Color32::from_rgb(58, 58, 61)
                                    } else {
                                        Color32::from_rgb(236, 236, 238)
                                    }
                                } else if option_resp.hovered() {
                                    crate::ui_style::hover_fill(ui.visuals().dark_mode)
                                } else {
                                    Color32::TRANSPARENT
                                };
                                ui.painter().rect_filled(option_rect, 7.0, fill);
                                ui.painter().text(
                                    option_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    layout.label(),
                                    egui::FontId::proportional(12.0 * scale),
                                    if selected {
                                        ui.visuals().text_color()
                                    } else {
                                        Color32::from_gray(150)
                                    },
                                );
                                if option_resp.clicked() {
                                    self.basic_layout = layout;
                                    ui.memory_mut(|m| m.close_popup());
                                }
                            }
                        },
                    );
                },
            );
        });
        ui.add_space(4.0);

        let keys: &[(usize, usize, usize, &str, u16)] = &[
            (0, 0, 1, "Esc", 0x0029),
            (0, 1, 1, "F1", 0x003A),
            (0, 2, 1, "F2", 0x003B),
            (0, 3, 1, "F3", 0x003C),
            (0, 4, 1, "F4", 0x003D),
            (0, 5, 1, "F5", 0x003E),
            (0, 6, 1, "F6", 0x003F),
            (0, 7, 1, "F7", 0x0040),
            (0, 8, 1, "F8", 0x0041),
            (0, 9, 1, "F9", 0x0042),
            (0, 10, 1, "F10", 0x0043),
            (0, 11, 1, "F11", 0x0044),
            (0, 12, 1, "F12", 0x0045),
            (1, 0, 1, "`", 0x0035),
            (1, 1, 1, "1", 0x001E),
            (1, 2, 1, "2", 0x001F),
            (1, 3, 1, "3", 0x0020),
            (1, 4, 1, "4", 0x0021),
            (1, 5, 1, "5", 0x0022),
            (1, 6, 1, "6", 0x0023),
            (1, 7, 1, "7", 0x0024),
            (1, 8, 1, "8", 0x0025),
            (1, 9, 1, "9", 0x0026),
            (1, 10, 1, "0", 0x0027),
            (1, 11, 1, "-", 0x002D),
            (1, 12, 1, "=", 0x002E),
            (1, 13, 1, "Backspace", 0x002A),
            (1, 14, 1, "Insert", 0x0049),
            (1, 15, 1, "Delete", 0x004C),
            (2, 0, 2, "Tab", 0x002B),
            (2, 2, 1, "Q", 0x0014),
            (2, 3, 1, "W", 0x001A),
            (2, 4, 1, "E", 0x0008),
            (2, 5, 1, "R", 0x0015),
            (2, 6, 1, "T", 0x0017),
            (2, 7, 1, "Y", 0x001C),
            (2, 8, 1, "U", 0x0018),
            (2, 9, 1, "I", 0x000C),
            (2, 10, 1, "O", 0x0012),
            (2, 11, 1, "P", 0x0013),
            (2, 12, 1, "[", 0x002F),
            (2, 13, 1, "]", 0x0030),
            (2, 14, 1, "\\", 0x0031),
            (3, 0, 2, "Caps\nLock", 0x0039),
            (3, 2, 1, "A", 0x0004),
            (3, 3, 1, "S", 0x0016),
            (3, 4, 1, "D", 0x0007),
            (3, 5, 1, "F", 0x0009),
            (3, 6, 1, "G", 0x000A),
            (3, 7, 1, "H", 0x000B),
            (3, 8, 1, "J", 0x000D),
            (3, 9, 1, "K", 0x000E),
            (3, 10, 1, "L", 0x000F),
            (3, 11, 1, ";", 0x0033),
            (3, 12, 1, "'", 0x0034),
            (3, 13, 2, "Enter", 0x0028),
            (4, 0, 3, "Shift", 0x00E1),
            (4, 3, 1, "Z", 0x001D),
            (4, 4, 1, "X", 0x001B),
            (4, 5, 1, "C", 0x0006),
            (4, 6, 1, "V", 0x0019),
            (4, 7, 1, "B", 0x0005),
            (4, 8, 1, "N", 0x0011),
            (4, 9, 1, "M", 0x0010),
            (4, 10, 1, ",", 0x0036),
            (4, 11, 1, ".", 0x0037),
            (4, 12, 1, "/", 0x0038),
            (4, 13, 2, "Shift", 0x00E5),
            (5, 0, 2, "Ctrl", 0x00E0),
            (5, 2, 1, gui_label(false), 0x00E3),
            (5, 3, 1, "Alt", 0x00E2),
            (5, 4, 4, "Space", 0x002C),
            (5, 8, 1, "Alt", 0x00E6),
            (5, 9, 1, "Menu", 0x0065),
            (5, 10, 1, "Ctrl", 0x00E4),
            (0, 13, 1, "Print\nScreen", 0x0046),
            (0, 14, 1, "Scroll\nLock", 0x0047),
            (0, 15, 1, "Pause", 0x0048),
            (2, 15, 1, "Home", 0x004A),
            (3, 15, 1, "End", 0x004D),
            (4, 15, 1, "Page\nUp", 0x004B),
            (5, 15, 1, "Page\nDown", 0x004E),
            (5, 11, 1, "←", 0x0050),
            (5, 12, 1, "↑", 0x0052),
            (5, 13, 1, "↓", 0x0051),
            (5, 14, 1, "→", 0x004F),
        ];

        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(available_width, height), egui::Sense::hover());
        let origin = egui::pos2(rect.min.x + x_offset, rect.min.y);
        for &(row, col, span, fallback_label, value) in keys {
            let assigned_value = self.basic_layout.map_value(value);
            let display_label = if self.key_legend_layout != KeyLegendLayout::English {
                if self.show_shifted_number_symbols {
                    if let Some(label) =
                        picker_shifted_number_label(assigned_value, self.key_legend_layout)
                    {
                        label
                    } else {
                        crate::keycode::find_keycode(assigned_value)
                            .map(|_| {
                                keycode_label_with_names_and_layout(
                                    assigned_value,
                                    &[],
                                    &self.layer_names,
                                    self.key_legend_layout,
                                )
                            })
                            .unwrap_or_else(|| fallback_label.to_string())
                    }
                } else {
                    crate::keycode::find_keycode(assigned_value)
                        .map(|_| {
                            keycode_label_with_names_and_layout(
                                assigned_value,
                                &[],
                                &self.layer_names,
                                self.key_legend_layout,
                            )
                        })
                        .unwrap_or_else(|| fallback_label.to_string())
                }
            } else {
                if self.show_shifted_number_symbols {
                    match assigned_value {
                        0x0035 => "~\n`".to_string(),
                        0x001E => "!\n1".to_string(),
                        0x001F => "@\n2".to_string(),
                        0x0020 => "#\n3".to_string(),
                        0x0021 => "$\n4".to_string(),
                        0x0022 => "%\n5".to_string(),
                        0x0023 => "^\n6".to_string(),
                        0x0024 => "&\n7".to_string(),
                        0x0025 => "*\n8".to_string(),
                        0x0026 => "(\n9".to_string(),
                        0x0027 => ")\n0".to_string(),
                        0x002D => "_\n-".to_string(),
                        0x002E => "+\n=".to_string(),
                        _ => crate::keycode::find_keycode(assigned_value)
                            .map(|_| {
                                keycode_label_with_names_and_layout(
                                    assigned_value,
                                    &[],
                                    &self.layer_names,
                                    self.key_legend_layout,
                                )
                            })
                            .unwrap_or_else(|| fallback_label.to_string()),
                    }
                } else {
                    crate::keycode::find_keycode(assigned_value)
                        .map(|_| {
                            keycode_label_with_names_and_layout(
                                assigned_value,
                                &[],
                                &self.layer_names,
                                self.key_legend_layout,
                            )
                        })
                        .unwrap_or_else(|| fallback_label.to_string())
                }
            };
            self.basic_key_button_at(
                ui,
                origin,
                cell_w,
                cell_h,
                gap,
                row,
                col,
                span,
                &display_label,
                assigned_value,
            );
        }
    }
}
