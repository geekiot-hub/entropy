use super::*;

impl EntropyApp {
    pub(super) fn draw_layout(
        &mut self,
        ui: &mut egui::Ui,
        layout: &KeyboardLayout,
        ctx: &egui::Context,
    ) {
        let avail = ui.available_size();
        let viewport = egui::Rect::from_min_max(
            ui.min_rect().min,
            egui::pos2(ui.min_rect().left() + avail.x, ui.max_rect().bottom()),
        );
        let geometry = layout_geometry(
            ui.ctx(),
            layout,
            viewport,
            clamp_ui_scale(self.app_settings.ui_scale),
        );
        let offset_x = geometry.offset_x;
        let offset_y = geometry.offset_y;
        let unit = geometry.unit;
        let padding = geometry.padding;
        let layout_h = geometry.layout_h;
        let main_tabs_h = 32.0_f32;
        let layer_bar_h = 68.0_f32;
        let top_reserved_h = LAYOUT_TOP_RESERVED_H;
        let top_base_y = ui.min_rect().top() + 6.0;
        self.last_layout_geometry = Some((offset_x, offset_y, unit, padding));

        // ── Main menu tabs ────────────────────────────────────────────────
        {
            use crate::i18n::Key as TrKey;

            let lang = self.app_settings.language;
            let center_x = ui.min_rect().center().x;
            let tabs_y = top_base_y;
            let tab_font_size = 15.0;
            let tab_height = 28.0;
            let tab_gap = 16.0;
            let tabs = [
                (
                    MainMenuTab::Keyboard,
                    crate::i18n::tr(lang, TrKey::MainTabLayout),
                    "main_menu.layout_tooltip",
                ),
                (
                    MainMenuTab::Advanced,
                    crate::i18n::tr(lang, TrKey::MainTabAdvanced),
                    "main_menu.advanced_tooltip",
                ),
                (
                    MainMenuTab::Settings,
                    crate::i18n::tr(lang, TrKey::MainTabConfig),
                    "main_menu.settings_tooltip",
                ),
            ];
            let tab_widths = tabs.map(|(_, label, _)| {
                (top_menu_text_width(ui, label, tab_font_size) + 34.0).max(96.0)
            });
            let total_w = tab_widths.iter().sum::<f32>() + tab_gap * (tabs.len() - 1) as f32;
            let start_x = center_x - total_w / 2.0;
            let mut device_tab_rect = None;
            let mut device_tab_hovered = false;
            let mut advanced_tab_rect = None;
            let mut advanced_tab_hovered = false;
            let mut settings_tab_rect = None;
            let mut settings_tab_hovered = false;

            let mut tab_x = start_x;
            for (idx, (tab, label, tooltip)) in tabs.iter().enumerate() {
                let slot_rect = egui::Rect::from_min_size(
                    egui::pos2(tab_x, tabs_y),
                    Vec2::new(tab_widths[idx], tab_height),
                );
                tab_x += tab_widths[idx] + tab_gap;
                let resp = ui.allocate_rect(slot_rect, Sense::CLICK);
                resp.clone()
                    .on_hover_text(crate::i18n::tr_catalog(lang, tooltip));
                if matches!(tab, MainMenuTab::Keyboard) {
                    device_tab_rect = Some(slot_rect);
                    device_tab_hovered = resp.hovered();
                }
                if matches!(tab, MainMenuTab::Advanced) {
                    advanced_tab_rect = Some(slot_rect);
                    advanced_tab_hovered = resp.hovered();
                }
                if matches!(tab, MainMenuTab::Settings) {
                    settings_tab_rect = Some(slot_rect);
                    settings_tab_hovered = resp.hovered();
                }
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if resp.clicked() {
                    match tab {
                        MainMenuTab::Keyboard => {
                            self.main_menu_tab = MainMenuTab::Keyboard;
                        }
                        MainMenuTab::Advanced => {}
                        MainMenuTab::Settings => {
                            if self.main_menu_tab != MainMenuTab::Settings {
                                self.reset_matrix_tester_state();
                            }
                            self.matrix_tester_unlock_prompted = false;
                            self.matrix_tester_lock_checked = false;
                            self.main_menu_tab = MainMenuTab::Settings;
                        }
                    }
                }

                let is_active = self.main_menu_tab == *tab;
                let text_color = if is_active {
                    ui.visuals().widgets.inactive.fg_stroke.color
                } else if resp.hovered() {
                    if ui.visuals().dark_mode {
                        Color32::from_gray(135)
                    } else {
                        Color32::from_gray(120)
                    }
                } else if ui.visuals().dark_mode {
                    Color32::from_gray(90)
                } else {
                    Color32::from_gray(150)
                };

                ui.painter().text(
                    slot_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    *label,
                    FontId::proportional(tab_font_size),
                    text_color,
                );
            }

            self.register_tour_target(
                TourTarget::MainNavigation,
                egui::Rect::from_min_size(
                    egui::pos2(start_x, tabs_y),
                    Vec2::new(total_w, tab_height),
                ),
            );
            if let Some(device_rect) = device_tab_rect {
                self.register_tour_target(TourTarget::DeviceSelector, device_rect);
            }
            if let Some(settings_rect) = settings_tab_rect {
                self.register_tour_target(TourTarget::SettingsMenu, settings_rect);
            }

            let zoom_width = 108.0;
            let zoom_left_top = egui::pos2(ui.min_rect().right() - 18.0 - zoom_width, tabs_y);
            self.draw_ui_scale_controls(ui, zoom_left_top);

            let undo_enabled = !self.undo_stack.is_empty();
            let undo_label = crate::i18n::tr_catalog(lang, "alt_repeat_editor.undo_curved");
            let undo_font = FontId::proportional(14.0);
            let undo_text_w = ui.fonts(|f| {
                f.layout_no_wrap(
                    undo_label.to_owned(),
                    undo_font.clone(),
                    ui.visuals().widgets.inactive.fg_stroke.color,
                )
                .size()
                .x
            });
            let undo_rect = egui::Rect::from_min_size(
                egui::pos2(ui.min_rect().left() + 24.0, tabs_y),
                Vec2::new(undo_text_w + 12.0, tab_height),
            );
            let undo_resp = ui.allocate_rect(undo_rect, Sense::CLICK);
            if undo_enabled {
                undo_resp.clone().on_hover_text(crate::i18n::tr_catalog(
                    lang,
                    "key_picker_text.undo_last_change",
                ));
            }
            if undo_resp.hovered() && undo_enabled {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if undo_resp.clicked() && undo_enabled {
                self.undo();
                ctx.request_repaint();
            }
            let undo_color = if !undo_enabled {
                if ui.visuals().dark_mode {
                    Color32::from_gray(58)
                } else {
                    Color32::from_gray(178)
                }
            } else if undo_resp.hovered() {
                app_accent()
            } else {
                ui.visuals().widgets.inactive.fg_stroke.color
            };
            let undo_text_pos = egui::pos2(undo_rect.left() + 6.0, undo_rect.center().y);
            ui.painter().text(
                undo_text_pos,
                egui::Align2::LEFT_CENTER,
                undo_label,
                undo_font,
                undo_color,
            );

            let divider_color = if ui.visuals().dark_mode {
                Color32::from_gray(105)
            } else {
                Color32::from_gray(170)
            };
            let divider_top = tabs_y + 4.0;
            let divider_bottom = tabs_y + tab_height - 4.0;
            let mut divider_x = start_x;
            for width in tab_widths.iter().take(tabs.len() - 1) {
                divider_x += *width;
                let x = divider_x + tab_gap / 2.0;
                ui.painter().line_segment(
                    [egui::pos2(x, divider_top), egui::pos2(x, divider_bottom)],
                    egui::Stroke::new(1.5, divider_color),
                );
                divider_x += tab_gap;
            }

            if let Some(device_rect) = device_tab_rect {
                let dropdown_id = ui.make_persistent_id("device_dropdown_open");
                let was_open = ui
                    .ctx()
                    .data(|d| d.get_temp::<bool>(dropdown_id))
                    .unwrap_or(false);
                let device_count = self.device_manager.devices().len();
                let device_rows = device_count.max(1) as f32;
                let devices_h = 12.0 + device_rows * 30.0;
                let sticky_layout_h = 36.0;
                let show_key_legend_switcher =
                    self.app_settings.key_legend_layout.is_multilingual();
                let key_legend_switcher_h = if show_key_legend_switcher { 36.0 } else { 0.0 };
                let mut device_menu_labels: Vec<String> =
                    if self.device_manager.devices().is_empty() {
                        vec![crate::i18n::tr(lang, TrKey::NoDevicesFound).to_owned()]
                    } else {
                        self.device_manager
                            .devices()
                            .iter()
                            .map(|dev| {
                                self.device_display_names
                                    .get(&dev.path)
                                    .cloned()
                                    .unwrap_or_else(|| dev.name.clone())
                            })
                            .collect()
                    };
                if show_key_legend_switcher {
                    if let Some(order_key) = self.app_settings.key_legend_layout.order_i18n_key() {
                        device_menu_labels
                            .push(crate::i18n::tr_catalog(lang, order_key).to_owned());
                    }
                }
                device_menu_labels.push(
                    crate::i18n::tr_catalog(lang, "ui.sticky_layout_window_label").to_owned(),
                );
                let dropdown_size = Vec2::new(
                    adaptive_top_dropdown_width(
                        ui,
                        device_menu_labels.iter().map(String::as_str),
                        152.0,
                    ),
                    devices_h + key_legend_switcher_h + sticky_layout_h + 12.0,
                );
                let dropdown_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        device_rect.center().x - dropdown_size.x / 2.0,
                        device_rect.bottom() + 6.0,
                    ),
                    dropdown_size,
                );
                let hover_bridge_rect = device_rect.union(dropdown_rect).expand(3.0);
                let pointer_over_bridge = ui
                    .ctx()
                    .input(|i| i.pointer.hover_pos())
                    .map(|pos| hover_bridge_rect.contains(pos))
                    .unwrap_or(false);
                let show_dropdown = !advanced_tab_hovered
                    && !settings_tab_hovered
                    && (device_tab_hovered || (was_open && pointer_over_bridge));

                if show_dropdown {
                    let area_id = ui.make_persistent_id("device_dropdown_area");
                    let mut device_clicked = false;
                    egui::Area::new(area_id)
                        .order(egui::Order::Foreground)
                        .fixed_pos(dropdown_rect.min)
                        .show(ctx, |ui| {
                            let dark = ui.visuals().dark_mode;
                            top_dropdown_frame(dark).show(ui, |ui| {
                                ui.set_min_width(dropdown_size.x - 16.0);

                                let prev_selected = self.selected_device;
                                if self.device_manager.devices().is_empty() {
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(dropdown_size.x - 16.0, 30.0),
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            ui.add_space(10.0);
                                            ui.label(
                                                RichText::new(crate::i18n::tr(
                                                    lang,
                                                    TrKey::NoDevicesFound,
                                                ))
                                                .size(13.0)
                                                .color(app_muted_text(ui.visuals().dark_mode)),
                                            );
                                        },
                                    );
                                } else {
                                    for (i, dev) in self.device_manager.devices().iter().enumerate()
                                    {
                                        let is_selected = self.selected_device == Some(i);
                                        let cached_display_name = self
                                            .device_display_names
                                            .get(&dev.path)
                                            .map(String::as_str);
                                        let display_name =
                                            cached_display_name.unwrap_or(dev.name.as_str());
                                        let resp = top_dropdown_item(
                                            ui,
                                            dropdown_size.x - 16.0,
                                            display_name,
                                            true,
                                            is_selected,
                                        );
                                        if resp.clicked() {
                                            self.selected_device = Some(i);
                                            self.main_menu_tab = MainMenuTab::Keyboard;
                                            device_clicked = true;
                                        }
                                    }
                                }

                                #[cfg(not(target_arch = "wasm32"))]
                                if self.selected_device != prev_selected {
                                    if let Some(idx) = self.selected_device {
                                        self.start_connect(idx);
                                    }
                                }


                                if show_key_legend_switcher {
                                    if let Some(order_key) =
                                        self.app_settings.key_legend_layout.order_i18n_key()
                                    {
                                        ui.add_space(6.0);
                                        let order_label = crate::i18n::tr_catalog(lang, order_key);
                                        if top_dropdown_item(
                                            ui,
                                            dropdown_size.x - 16.0,
                                            order_label,
                                            true,
                                            false,
                                        )
                                        .clicked()
                                        {
                                            self.app_settings.key_legend_layout =
                                                self.app_settings.key_legend_layout.toggled_order();
                                            save_app_settings(&self.app_settings);
                                            ctx.request_repaint();
                                        }
                                    }
                                }

                                ui.add_space(6.0);
                                if top_dropdown_item(
                                    ui,
                                    dropdown_size.x - 16.0,
                                    crate::i18n::tr_catalog(lang, "ui.sticky_layout_window_label"),
                                    true,
                                    self.app_settings.sticky_layout_window,
                                )
                                .clicked()
                                {
                                    if self.app_settings.sticky_layout_window {
                                        self.app_settings.sticky_layout_window = false;
                                        self.pending_layout_indicator_open_after_unlock = false;
                                        self.sticky_layout_last_size = None;
                                        save_app_settings(&self.app_settings);
                                    } else if self.is_vial_locked() {
                                        self.pending_layout_indicator_open_after_unlock = true;
                                        self.unlock_open = true;
                                        self.status_msg = crate::i18n::tr_catalog(
                                            self.app_settings.language,
                                            "matrix_tester.keyboard_is_locked_unlock_it_to_use_matrix_tester",
                                        )
                                        .into();
                                    } else {
                                        self.app_settings.sticky_layout_window = true;
                                        self.sticky_layout_last_size = None;
                                        save_app_settings(&self.app_settings);
                                    }
                                    ctx.request_repaint();
                                    device_clicked = true;
                                }
                            });
                        });

                    ui.ctx().data_mut(|d| {
                        d.insert_temp(
                            dropdown_id,
                            !device_clicked && (device_tab_hovered || pointer_over_bridge),
                        )
                    });
                } else {
                    ui.ctx().data_mut(|d| d.insert_temp(dropdown_id, false));
                }
            }

            if let Some(advanced_rect) = advanced_tab_rect {
                let dropdown_id = ui.make_persistent_id("advanced_dropdown_open");
                let was_open = ui
                    .ctx()
                    .data(|d| d.get_temp::<bool>(dropdown_id))
                    .unwrap_or(false);
                let combo_supported = !self.combo_entries.is_empty();
                let key_override_supported = !self.key_override_entries.is_empty();
                let auto_shift_supported = self.auto_shift_timeout.is_some();
                let advanced_item_count = 1
                    + combo_supported as usize
                    + auto_shift_supported as usize
                    + key_override_supported as usize;
                let mut advanced_menu_labels =
                    vec![crate::i18n::tr_catalog(lang, "text_expander.title")];
                if combo_supported {
                    advanced_menu_labels.push(crate::i18n::tr(lang, TrKey::ComboTitle));
                }
                if auto_shift_supported {
                    advanced_menu_labels.push(crate::i18n::tr(lang, TrKey::AutoShiftTitle));
                }
                if key_override_supported {
                    advanced_menu_labels.push(crate::i18n::tr(lang, TrKey::KeyOverridesTitle));
                }
                let advanced_dropdown_width =
                    adaptive_top_dropdown_width(ui, advanced_menu_labels, 152.0);
                let dropdown_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        advanced_rect.center().x - advanced_dropdown_width / 2.0,
                        advanced_rect.bottom() + 6.0,
                    ),
                    Vec2::new(
                        advanced_dropdown_width,
                        (advanced_item_count.max(1) as f32) * 28.0 + 22.0,
                    ),
                );
                let hover_bridge_rect = advanced_rect.union(dropdown_rect).expand(3.0);
                let pointer_over_bridge = ui
                    .ctx()
                    .input(|i| i.pointer.hover_pos())
                    .map(|pos| hover_bridge_rect.contains(pos))
                    .unwrap_or(false);
                let show_dropdown = advanced_item_count > 0
                    && !device_tab_hovered
                    && !settings_tab_hovered
                    && (advanced_tab_hovered || (was_open && pointer_over_bridge));

                if show_dropdown {
                    let dark = ui.visuals().dark_mode;
                    let item_width = dropdown_rect.width() - 16.0;
                    let (
                        text_expander_hovered,
                        combo_hovered,
                        auto_shift_hovered,
                        key_override_hovered,
                        advanced_clicked,
                    ) = egui::Area::new(egui::Id::new("advanced_dropdown_area"))
                        .order(egui::Order::Foreground)
                        .fixed_pos(dropdown_rect.min)
                        .show(ui.ctx(), |ui| {
                            top_dropdown_frame(dark)
                                .show(ui, |ui| {
                                    ui.set_min_width(item_width);
                                    let text_expander_resp = top_dropdown_item(
                                        ui,
                                        item_width,
                                        crate::i18n::tr_catalog(lang, "text_expander.title"),
                                        true,
                                        self.main_menu_tab == MainMenuTab::Advanced
                                            && self.settings_tab == SettingsTab::TextExpander,
                                    );
                                    let combo_resp = combo_supported.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::ComboTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Advanced
                                                && self.settings_tab == SettingsTab::Combo,
                                        )
                                    });
                                    let auto_shift_resp = auto_shift_supported.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::AutoShiftTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Advanced
                                                && self.settings_tab == SettingsTab::AutoShift,
                                        )
                                    });
                                    let key_override_resp = key_override_supported.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::KeyOverridesTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Advanced
                                                && self.settings_tab == SettingsTab::KeyOverrides,
                                        )
                                    });
                                    if text_expander_resp.clicked() {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_text_expander_settings_page();
                                    }

                                    if combo_resp.as_ref().map(|r| r.clicked()).unwrap_or(false) {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.settings_tab = SettingsTab::Combo;
                                        self.main_menu_tab = MainMenuTab::Advanced;
                                        if self.combo_visible_count == 0 {
                                            self.combo_visible_count = 1;
                                        }
                                    }
                                    if auto_shift_resp
                                        .as_ref()
                                        .map(|r| r.clicked())
                                        .unwrap_or(false)
                                    {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.settings_tab = SettingsTab::AutoShift;
                                        self.main_menu_tab = MainMenuTab::Advanced;
                                        if self.is_vial_locked() {
                                            self.unlock_open = true;
                                            self.status_msg = format!(
                                                "{} — {}",
                                                crate::i18n::tr(
                                                    self.app_settings.language,
                                                    TrKey::KeyboardLocked,
                                                ),
                                                crate::i18n::tr(
                                                    self.app_settings.language,
                                                    TrKey::AutoShiftUnlockHint,
                                                ),
                                            );
                                        }
                                    }
                                    if key_override_resp
                                        .as_ref()
                                        .map(|r| r.clicked())
                                        .unwrap_or(false)
                                    {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.settings_tab = SettingsTab::KeyOverrides;
                                        self.main_menu_tab = MainMenuTab::Advanced;
                                    }
                                    (
                                        text_expander_resp.hovered(),
                                        combo_resp.as_ref().map(|r| r.hovered()).unwrap_or(false),
                                        auto_shift_resp
                                            .as_ref()
                                            .map(|r| r.hovered())
                                            .unwrap_or(false),
                                        key_override_resp
                                            .as_ref()
                                            .map(|r| r.hovered())
                                            .unwrap_or(false),
                                        text_expander_resp.clicked()
                                            || combo_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || auto_shift_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || key_override_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false),
                                    )
                                })
                                .inner
                        })
                        .inner;
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(
                            dropdown_id,
                            !advanced_clicked
                                && (advanced_tab_hovered
                                    || text_expander_hovered
                                    || combo_hovered
                                    || auto_shift_hovered
                                    || key_override_hovered
                                    || pointer_over_bridge),
                        )
                    });
                } else {
                    ui.ctx().data_mut(|d| d.insert_temp(dropdown_id, false));
                }
            }

            if let Some(settings_rect) = settings_tab_rect {
                let dropdown_id = ui.make_persistent_id("settings_dropdown_open");
                let was_open = ui
                    .ctx()
                    .data(|d| d.get_temp::<bool>(dropdown_id))
                    .unwrap_or(false);
                let rgb_available_for_menu = self.rgb_settings.supported || layout.supports_rgb;
                let layer_leds_available_for_menu = self.layer_led_settings.supported;
                let show_rgb_item = rgb_available_for_menu;
                let show_layer_leds_item = layer_leds_available_for_menu;
                let show_encoders_item = layout.encoder_count() > 0;
                let show_layout_options_item = layout
                    .layout_options
                    .iter()
                    .any(|option| !Self::is_encoder_layout_option(option));
                let show_modules_item = self.module_settings.supported;
                let show_touchpad_item = self.touchpad_settings.supported;
                let show_live_features_item = self.live_features_available_for_selected_device();
                let show_magic_item = self.magic_settings.supported;
                let show_tap_hold_item =
                    self.tap_hold_settings.supported || self.one_shot_settings.supported;
                let show_matrix_item = self.firmware == FirmwareProtocol::Vial;
                let show_lock_item = self.firmware == FirmwareProtocol::Vial
                    && self.layout.is_some()
                    && !self.vial_unlock_polling
                    && !self.unlock_open;
                let is_unlocked = if show_lock_item {
                    self.hid_device
                        .as_ref()
                        .and_then(|hid| hid.get_unlock_status().ok())
                        .map(|(unlocked, _keys)| unlocked)
                        .unwrap_or(false)
                } else {
                    false
                };
                let lock_label = if is_unlocked {
                    crate::i18n::tr_catalog(lang, "ui.lock_keyboard_action")
                } else {
                    crate::i18n::tr_catalog(lang, "ui.unlock_keyboard_action")
                };
                let settings_item_count = 2
                    + show_matrix_item as usize
                    + show_rgb_item as usize
                    + show_layer_leds_item as usize
                    + show_encoders_item as usize
                    + show_layout_options_item as usize
                    + show_modules_item as usize
                    + show_touchpad_item as usize
                    + show_live_features_item as usize
                    + show_magic_item as usize
                    + show_tap_hold_item as usize
                    + show_lock_item as usize;
                // Keep hover bridge in sync with actual item height (30px) and frame padding.
                // Underestimating this makes lower items close the dropdown on hover.
                let dropdown_height = settings_item_count as f32 * 30.0 + 12.0;
                let mut settings_menu_labels = vec![
                    crate::i18n::tr(lang, TrKey::AppSettingsTitle),
                    crate::i18n::tr(lang, TrKey::UniversalSymbolsTitle),
                ];
                if show_matrix_item {
                    settings_menu_labels.push(crate::i18n::tr(lang, TrKey::MatrixTesterTitle));
                }
                if show_rgb_item {
                    settings_menu_labels.push(crate::i18n::tr(lang, TrKey::RgbTitle));
                }
                if show_layer_leds_item {
                    settings_menu_labels.push(crate::i18n::tr(lang, TrKey::LayerLedsTitle));
                }
                if show_encoders_item {
                    settings_menu_labels.push(crate::i18n::tr(lang, TrKey::EncodersTitle));
                }
                if show_layout_options_item {
                    settings_menu_labels.push(crate::i18n::tr(lang, TrKey::DisplayPresetsTitle));
                }
                if show_modules_item {
                    settings_menu_labels
                        .push(crate::i18n::tr_catalog(lang, "modules_settings.title"));
                }
                if show_touchpad_item {
                    settings_menu_labels.push(crate::i18n::tr(lang, TrKey::TouchpadTitle));
                }
                if show_live_features_item {
                    settings_menu_labels.push(crate::i18n::tr(lang, TrKey::LiveFeaturesTitle));
                }
                if show_magic_item {
                    settings_menu_labels.push(crate::i18n::tr(lang, TrKey::MagicTitle));
                }
                if show_tap_hold_item {
                    settings_menu_labels.push(crate::i18n::tr(lang, TrKey::TapHoldOneShotTitle));
                }
                if show_lock_item {
                    settings_menu_labels.push(lock_label);
                }
                let dropdown_width = adaptive_top_dropdown_width(ui, settings_menu_labels, 184.0);
                let dropdown_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        settings_rect.center().x - dropdown_width / 2.0,
                        settings_rect.bottom() + 6.0,
                    ),
                    Vec2::new(dropdown_width, dropdown_height),
                );
                let hover_bridge_rect = settings_rect.union(dropdown_rect).expand(3.0);
                let pointer_over_bridge = ui
                    .ctx()
                    .input(|i| i.pointer.hover_pos())
                    .map(|pos| hover_bridge_rect.contains(pos))
                    .unwrap_or(false);
                let show_dropdown = !device_tab_hovered
                    && !advanced_tab_hovered
                    && (settings_tab_hovered || (was_open && pointer_over_bridge));

                if show_dropdown {
                    let dark = ui.visuals().dark_mode;
                    let rgb_available = rgb_available_for_menu;
                    let item_width = dropdown_rect.width() - 16.0;
                    let (
                        app_hovered,
                        matrix_hovered,
                        universal_symbols_hovered,
                        rgb_hovered,
                        layer_leds_hovered,
                        encoders_hovered,
                        layout_options_hovered,
                        modules_hovered,
                        touchpad_hovered,
                        live_features_hovered,
                        magic_hovered,
                        tap_hold_hovered,
                        lock_hovered,
                        settings_clicked,
                    ) = egui::Area::new(egui::Id::new("settings_dropdown_area"))
                        .order(egui::Order::Foreground)
                        .fixed_pos(dropdown_rect.min)
                        .show(ui.ctx(), |ui| {
                            top_dropdown_frame(dark)
                                .show(ui, |ui| {
                                    ui.set_min_width(item_width);
                                    ui.spacing_mut().item_spacing.y = 0.0;
                                    let app_resp = top_dropdown_item(
                                        ui,
                                        item_width,
                                        crate::i18n::tr(lang, TrKey::AppSettingsTitle),
                                        true,
                                        self.main_menu_tab == MainMenuTab::Settings
                                            && self.settings_tab == SettingsTab::AppSettings,
                                    );
                                    let matrix_resp = show_matrix_item.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::MatrixTesterTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::MatrixTester,
                                        )
                                    });
                                    let universal_symbols_resp = top_dropdown_item(
                                        ui,
                                        item_width,
                                        crate::i18n::tr(lang, TrKey::UniversalSymbolsTitle),
                                        true,
                                        self.main_menu_tab == MainMenuTab::Settings
                                            && self.settings_tab
                                                == SettingsTab::UniversalSymbolsSetup,
                                    );
                                    let rgb_resp = if show_rgb_item {
                                        Some(top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::RgbTitle),
                                            rgb_available,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::Rgb,
                                        ))
                                    } else {
                                        None
                                    };
                                    let layer_leds_resp = show_layer_leds_item.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::LayerLedsTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::LayerLeds,
                                        )
                                    });
                                    let encoders_resp = show_encoders_item.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::EncodersTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::Encoders,
                                        )
                                    });
                                    let layout_options_resp = show_layout_options_item.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::DisplayPresetsTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::LayoutOptions,
                                        )
                                    });
                                    let modules_resp = show_modules_item.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr_catalog(lang, "modules_settings.title"),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::Modules,
                                        )
                                    });
                                    let touchpad_resp = show_touchpad_item.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::TouchpadTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::Touchpad,
                                        )
                                    });
                                    let live_features_resp = show_live_features_item.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::LiveFeaturesTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::LiveFeatures,
                                        )
                                    });
                                    let magic_resp = show_magic_item.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::MagicTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::Magic,
                                        )
                                    });
                                    let tap_hold_resp = show_tap_hold_item.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::TapHoldOneShotTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::TapHold,
                                        )
                                    });
                                    let lock_resp = show_lock_item.then(|| {
                                        top_dropdown_item(ui, item_width, lock_label, true, false)
                                    });
                                    if app_resp.clicked() {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_app_settings_page();
                                    }
                                    if matrix_resp.as_ref().map(|r| r.clicked()).unwrap_or(false) {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.settings_tab = SettingsTab::MatrixTester;
                                        if self.main_menu_tab != MainMenuTab::Settings {
                                            self.reset_matrix_tester_state();
                                        }
                                        self.matrix_tester_unlock_prompted = false;
                                        self.matrix_tester_lock_checked = false;
                                        self.main_menu_tab = MainMenuTab::Settings;
                                    }
                                    if universal_symbols_resp.clicked() {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_universal_symbols_setup_page();
                                    }
                                    if let Some(rgb_resp) = &rgb_resp {
                                        if rgb_resp.clicked() && rgb_available {
                                            self.close_top_dropdowns(ui.ctx());
                                            self.settings_tab = SettingsTab::Rgb;
                                            self.main_menu_tab = MainMenuTab::Settings;
                                        }
                                        if !rgb_available {
                                            let _ = rgb_resp.clone().on_hover_text(
                                                crate::i18n::tr(lang, TrKey::RgbUnavailableTooltip),
                                            );
                                        }
                                    }
                                    if layer_leds_resp
                                        .as_ref()
                                        .map(|r| r.clicked())
                                        .unwrap_or(false)
                                    {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_layer_led_settings_page();
                                    }
                                    if encoders_resp.as_ref().map(|r| r.clicked()).unwrap_or(false)
                                    {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.settings_tab = SettingsTab::Encoders;
                                        self.main_menu_tab = MainMenuTab::Settings;
                                    }
                                    if layout_options_resp
                                        .as_ref()
                                        .map(|r| r.clicked())
                                        .unwrap_or(false)
                                    {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_layout_options_settings_page();
                                    }
                                    if modules_resp.as_ref().map(|r| r.clicked()).unwrap_or(false) {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_modules_settings_page();
                                    }
                                    if touchpad_resp.as_ref().map(|r| r.clicked()).unwrap_or(false)
                                    {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_touchpad_settings_page();
                                    }
                                    if live_features_resp
                                        .as_ref()
                                        .map(|r| r.clicked())
                                        .unwrap_or(false)
                                    {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_live_features_settings_page();
                                    }
                                    if magic_resp.as_ref().map(|r| r.clicked()).unwrap_or(false) {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_magic_settings_page();
                                    }
                                    if tap_hold_resp.as_ref().map(|r| r.clicked()).unwrap_or(false)
                                    {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_tap_hold_settings_page();
                                    }
                                    if lock_resp.as_ref().map(|r| r.clicked()).unwrap_or(false) {
                                        self.close_top_dropdowns(ui.ctx());
                                        if is_unlocked {
                                            if let Some(hid) = &self.hid_device {
                                                let layout_indicator_was_open =
                                                    self.app_settings.sticky_layout_window;
                                                match hid.lock() {
                                                    Ok(()) => {
                                                        if layout_indicator_was_open {
                                                            self.status_msg = crate::i18n::tr_catalog(
                                                                self.app_settings.language,
                                                                "ui.sticky_layout_closed_due_to_lock",
                                                            )
                                                            .into();
                                                            self.app_settings.sticky_layout_window = false;
                                                            self.pending_layout_indicator_open_after_unlock = false;
                                                            self.sticky_layout_last_size = None;
                                                            save_app_settings(&self.app_settings);
                                                        } else {
                                                            self.status_msg = "Device locked".into();
                                                        }
                                                    }
                                                    Err(e) => {
                                                        self.status_msg = format!("Lock failed: {e}")
                                                    }
                                                }
                                            }
                                        } else {
                                            self.unlock_open = true;
                                        }
                                    }
                                    (
                                        app_resp.hovered(),
                                        matrix_resp.as_ref().map(|r| r.hovered()).unwrap_or(false),
                                        universal_symbols_resp.hovered(),
                                        rgb_resp
                                            .as_ref()
                                            .map(|resp| resp.hovered())
                                            .unwrap_or(false),
                                        layer_leds_resp
                                            .as_ref()
                                            .map(|r| r.hovered())
                                            .unwrap_or(false),
                                        encoders_resp
                                            .as_ref()
                                            .map(|r| r.hovered())
                                            .unwrap_or(false),
                                        layout_options_resp
                                            .as_ref()
                                            .map(|r| r.hovered())
                                            .unwrap_or(false),
                                        modules_resp.as_ref().map(|r| r.hovered()).unwrap_or(false),
                                        touchpad_resp
                                            .as_ref()
                                            .map(|r| r.hovered())
                                            .unwrap_or(false),
                                        live_features_resp
                                            .as_ref()
                                            .map(|r| r.hovered())
                                            .unwrap_or(false),
                                        magic_resp.as_ref().map(|r| r.hovered()).unwrap_or(false),
                                        tap_hold_resp
                                            .as_ref()
                                            .map(|r| r.hovered())
                                            .unwrap_or(false),
                                        lock_resp.as_ref().map(|r| r.hovered()).unwrap_or(false),
                                        app_resp.clicked()
                                            || matrix_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || universal_symbols_resp.clicked()
                                            || rgb_resp
                                                .as_ref()
                                                .map(|resp| resp.clicked() && rgb_available)
                                                .unwrap_or(false)
                                            || layer_leds_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || encoders_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || layout_options_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || modules_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || touchpad_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || live_features_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || magic_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || tap_hold_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || lock_resp.as_ref().map(|r| r.clicked()).unwrap_or(false),
                                    )
                                })
                                .inner
                        })
                        .inner;
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(
                            dropdown_id,
                            !settings_clicked
                                && (settings_tab_hovered
                                    || app_hovered
                                    || matrix_hovered
                                    || universal_symbols_hovered
                                    || rgb_hovered
                                    || layer_leds_hovered
                                    || encoders_hovered
                                    || layout_options_hovered
                                    || modules_hovered
                                    || touchpad_hovered
                                    || live_features_hovered
                                    || magic_hovered
                                    || tap_hold_hovered
                                    || lock_hovered
                                    || pointer_over_bridge),
                        )
                    });
                } else {
                    ui.ctx().data_mut(|d| d.insert_temp(dropdown_id, false));
                }
            }
            if matches!(
                self.main_menu_tab,
                MainMenuTab::Settings | MainMenuTab::Advanced
            ) {
                self.draw_settings_screen(ui, layout, ctx, ui.min_rect().top() + top_reserved_h);
                return;
            }

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
                    let commit =
                        resp.lost_focus() || ui.input(|inp| inp.key_pressed(egui::Key::Enter));
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
                                        if let Err(e) = dev.set_qmk_setting_string(
                                            200 + selected as u16,
                                            &new_name,
                                        ) {
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
                    let right_hit =
                        egui::Rect::from_center_size(right_center, Vec2::new(28.0, 44.0));
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

        // Pass 1: allocate
        let key_rects: Vec<(usize, egui::Rect)> = layout
            .keys
            .iter()
            .enumerate()
            .map(|(ki, key)| {
                let geometry = LayoutGeometry {
                    offset_x,
                    offset_y,
                    unit,
                    padding,
                    layout_h,
                };
                let rect = layout_physical_key_rect(key, geometry);
                (ki, rect)
            })
            .collect();
        let encoder_rects: Vec<(usize, egui::Rect)> = layout
            .encoders
            .iter()
            .enumerate()
            .map(|(ei, encoder)| {
                let geometry = LayoutGeometry {
                    offset_x,
                    offset_y,
                    unit,
                    padding,
                    layout_h,
                };
                let rect = layout_physical_encoder_rect(encoder, geometry);
                (ei, rect)
            })
            .collect();
        let keyboard_target_rect = key_rects
            .iter()
            .map(|(_, rect)| *rect)
            .chain(encoder_rects.iter().map(|(_, rect)| *rect))
            .reduce(|acc, rect| acc.union(rect));
        if let Some(rect) = keyboard_target_rect {
            self.register_tour_target(TourTarget::KeyboardArea, rect.expand(10.0));
        }
        self.register_tour_target(
            TourTarget::BottomHints,
            egui::Rect::from_center_size(
                egui::pos2(ui.max_rect().center().x, ui.max_rect().bottom() - 34.0),
                Vec2::new(ui.max_rect().width().min(560.0), 52.0),
            ),
        );
        let mut encoder_groups: Vec<(u8, egui::Rect, Option<(usize, u16)>, Option<(usize, u16)>)> =
            Vec::new();
        for (ei, rect) in &encoder_rects {
            let encoder = &layout.encoders[*ei];
            if !self
                .encoder_visibility
                .get(encoder.encoder_idx as usize)
                .copied()
                .unwrap_or(true)
            {
                continue;
            }
            let kc = layout.get_encoder_keycode(self.selected_layer, *ei);
            if let Some((_, group_rect, ccw, cw)) = encoder_groups
                .iter_mut()
                .find(|(idx, _, _, _)| *idx == encoder.encoder_idx)
            {
                *group_rect = group_rect.union(*rect);
                if encoder.direction == 0 {
                    *ccw = Some((*ei, kc));
                } else {
                    *cw = Some((*ei, kc));
                }
            } else {
                encoder_groups.push((
                    encoder.encoder_idx,
                    *rect,
                    if encoder.direction == 0 {
                        Some((*ei, kc))
                    } else {
                        None
                    },
                    if encoder.direction == 0 {
                        None
                    } else {
                        Some((*ei, kc))
                    },
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
        let mut rects: Vec<(usize, egui::Rect, egui::Response)> =
            Vec::with_capacity(layout.keys.len());
        for (ki, rect) in &key_rects {
            let response_rect = encoder_press_rects
                .iter()
                .find(|(press_ki, _)| press_ki == ki)
                .map(|(_, press_rect)| *press_rect)
                .unwrap_or(*rect);
            let response = ui.allocate_rect(response_rect, Sense::click());
            rects.push((*ki, response_rect, response));
        }

        // Reset hover_layer each frame — will be set again if a layer key is hovered
        let prev_hover = self.hover_layer;
        self.hover_layer = None;

        // Pass 2: hover + clicks + tooltips
        let mut hovered_key: Option<usize> = None;
        for (ki, _, response) in &mut rects {
            if response.hovered() {
                hovered_key = Some(*ki);
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if response.clicked() {
                self.selected_key = Some((self.selected_layer, *ki));
                self.keycode_picker.open = true;
                self.keycode_picker.result = None;
                self.keycode_picker.search_query.clear();
                self.keycode_picker.layer_names = self.layer_names.clone();
                self.keycode_picker.vial_quantum_pending_mod = None;
                self.keycode_picker.vial_quantum_pending_mt = None;
                self.keycode_picker.vial_layer_pending = None;
                // Reset all editor states so picker opens normally
                self.keycode_picker.tap_dance_editor_open = None;
                self.keycode_picker.selected_tab = crate::keycode_picker::KeycodeTab::Basic;
            }

            // Right-click actions: layer jump/retarget, modifier side swap, editors/settings.
            if response.secondary_clicked() {
                let ctrl_held = ui.input(|i| i.modifiers.ctrl);
                let kc = layout.get_keycode(self.selected_layer, *ki);
                self.handle_secondary_target(ctrl_held, kc, Some(*ki), None);
                if self.secondary_click_handled {
                    continue;
                }
            }

            // Tooltip — for layer keys show mini layout preview
            let kc = layout.get_keycode(self.selected_layer, *ki);
            // MO/TG/TO/OSL/TT/DF range and LT; OSM also lives in 0x52xx
            // but is deliberately excluded by vial_layer_target().
            let preview_layer: Option<usize> = vial_layer_target(kc);

            if let Some(preview_layer_idx) = preview_layer {
                if response.hovered() {
                    hovered_key = Some(*ki); // keep hovered_key for layer keys too
                    if self.app_settings.layer_hover_preview {
                        self.hover_layer = Some(preview_layer_idx);
                    } else {
                        let tip = keycode_tooltip_with_macro_names(
                            kc,
                            &layout.custom_keycodes,
                            &self.layer_names,
                            &self.keycode_picker.macro_names,
                            &self.keycode_picker.tap_dance_names,
                        );
                        *response = response
                            .clone()
                            .on_hover_text(crate::i18n::tr_text(self.app_settings.language, &tip));
                    }
                }
                if response.secondary_clicked() && preview_layer_idx != self.selected_layer {
                    // Right-click: jump to that layer
                    self.jump_back_stack.push(self.selected_layer);
                    self.selected_layer = preview_layer_idx;
                    self.hover_layer = None;
                    self.secondary_click_handled = true;
                }
            } else if response.hovered() {
                let tip = keycode_tooltip_with_macro_names(
                    kc,
                    &layout.custom_keycodes,
                    &self.layer_names,
                    &self.keycode_picker.macro_names,
                    &self.keycode_picker.tap_dance_names,
                );
                *response = response
                    .clone()
                    .on_hover_text(crate::i18n::tr_text(self.app_settings.language, &tip));
            }
        }

        // Animate hover_layer_progress
        let target_progress = if self.hover_layer.is_some() {
            1.0f32
        } else {
            0.0f32
        };
        let speed = 4.0f32;
        let dt = ctx.input(|i| i.stable_dt).min(0.1);
        self.hover_layer_progress +=
            (target_progress - self.hover_layer_progress) * (speed * dt).min(1.0);
        if (self.hover_layer_progress - target_progress).abs() > 0.01 {
            ctx.request_repaint();
        }

        // Pass 3: paint
        let painter = ui.painter();
        let mut hovered_encoder = false;
        let mut hovered_encoder_keycode = None;
        let hover_target = self
            .hover_layer
            .unwrap_or(prev_hover.unwrap_or(self.selected_layer));
        let hover_alpha = self.hover_layer_progress;
        let dark = self.dark_mode;
        // Use hover layer for logic (TRNS resolution etc) when mostly visible
        let layer = if hover_alpha > 0.5 {
            hover_target
        } else {
            self.selected_layer
        };
        let layer_led_color_idx = if self.layer_led_settings.supported {
            self.layer_led_settings
                .layer_colors
                .get(layer.min(15))
                .copied()
                .filter(|color_idx| !matches!(color_idx, 0 | 1))
        } else {
            None
        };
        // Off and White should keep the standard neutral outline/fill so disabled/uncolored
        // layers do not look artificially tinted.
        let layer_led_outline = layer_led_color_idx.map(layer_led_outline_color);
        let layer_led_hover_fill =
            layer_led_color_idx.map(|color_idx| layer_led_hover_fill(color_idx, dark));
        for (ki, rect, _) in &rects {
            let key = &layout.keys[*ki];
            let is_selected = self.selected_key == Some((layer, *ki));
            let is_hovered = hovered_key == Some(*ki);
            // Accent: #5B68DF indigo
            let bg = if is_selected {
                app_accent()
            } else if is_hovered {
                layer_led_hover_fill.unwrap_or_else(|| crate::ui_style::hover_fill(dark))
            } else {
                if dark {
                    Color32::from_rgb(48, 48, 52)
                } else {
                    Color32::from_rgb(255, 255, 255)
                }
            };

            let press_rect_override = encoder_press_rects
                .iter()
                .find(|(press_ki, _)| *press_ki == *ki)
                .map(|(_, press_rect)| *press_rect);
            let draw_rect = press_rect_override.unwrap_or(*rect);

            let is_hovering = hover_alpha > 0.05;

            if press_rect_override.is_some() {
                continue;
            }

            let kc = layout.get_keycode(layer, *ki);

            if kc == 0x0001 {
                paint_layout_keycap(
                    painter,
                    draw_rect,
                    key.rotation,
                    bg,
                    Stroke::new(
                        1.0,
                        layer_led_outline.unwrap_or_else(|| {
                            if dark {
                                Color32::from_rgb(54, 54, 58)
                            } else {
                                Color32::from_rgb(230, 230, 233)
                            }
                        }),
                    ),
                );
                if !is_hovering {
                    let fallback_kc = (0..layer)
                        .rev()
                        .map(|l| layout.get_keycode(l, *ki))
                        .find(|&k| k != 0x0001)
                        .unwrap_or(0x0000);
                    let label = if fallback_kc == 0x0000 || fallback_kc == 0x0001 {
                        String::new()
                    } else {
                        keycode_label_with_macro_names(
                            fallback_kc,
                            &layout.custom_keycodes,
                            &self.layer_names,
                            &self.keycode_picker.macro_names,
                            &self.keycode_picker.tap_dance_names,
                            self.app_settings.key_legend_layout,
                        )
                    };
                    let label = number_row_shifted_label(
                        label,
                        self.app_settings.show_shifted_number_symbols,
                        self.app_settings.key_legend_layout,
                    );
                    draw_key_label_dimmed(
                        &painter,
                        draw_rect,
                        &label,
                        dark,
                        key.rotation.to_radians(),
                    );
                }
            } else if kc == 0x0000 {
                let no_bg = if dark {
                    Color32::from_rgb(20, 20, 22)
                } else {
                    Color32::from_rgb(255, 255, 255)
                };
                let no_border = layer_led_outline.unwrap_or_else(|| {
                    if dark {
                        Color32::from_rgb(40, 40, 44)
                    } else {
                        Color32::from_rgb(230, 230, 233)
                    }
                });
                let fill = if is_selected || is_hovered { bg } else { no_bg };
                paint_layout_keycap(
                    painter,
                    draw_rect,
                    key.rotation,
                    fill,
                    Stroke::new(1.0, no_border),
                );
            } else {
                let border = layer_led_outline.unwrap_or_else(|| {
                    if dark {
                        Color32::from_rgb(54, 54, 58)
                    } else {
                        Color32::from_rgb(230, 230, 233)
                    }
                });
                paint_layout_keycap(
                    painter,
                    draw_rect,
                    key.rotation,
                    bg,
                    Stroke::new(1.0, border),
                );
                let label = number_row_shifted_label(
                    keycode_label_with_macro_names(
                        kc,
                        &layout.custom_keycodes,
                        &self.layer_names,
                        &self.keycode_picker.macro_names,
                        &self.keycode_picker.tap_dance_names,
                        self.app_settings.key_legend_layout,
                    ),
                    self.app_settings.show_shifted_number_symbols,
                    self.app_settings.key_legend_layout,
                );
                draw_key_label(&painter, draw_rect, &label, dark, key.rotation.to_radians());
            }
        }

        let encoder_custom_keycodes = layout.custom_keycodes.clone();
        let encoder_layer_names = self.layer_names.clone();
        let encoder_macro_names = self.keycode_picker.macro_names.clone();
        let encoder_tap_dance_names = self.keycode_picker.tap_dance_names.clone();
        let encoder_key_legend_layout = self.app_settings.key_legend_layout;
        let encoder_label = |kc: u16| -> String {
            match kc {
                0x0000 => "✕".to_string(),
                0x0001 => "▽".to_string(),
                _ => keycode_label_with_macro_names(
                    kc,
                    &encoder_custom_keycodes,
                    &encoder_layer_names,
                    &encoder_macro_names,
                    &encoder_tap_dance_names,
                    encoder_key_legend_layout,
                )
                .replace('\n', " "),
            }
        };

        let draw_encoder_arrow = |painter: &egui::Painter,
                                  center: egui::Pos2,
                                  encoder_radius: f32,
                                  top: bool,
                                  color: Color32| {
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
        };

        const ENCODER_HOVER_SCALE: f32 = 1.5;
        let encoder_hover_enlarge = self.app_settings.encoder_hover_enlarge;
        for (_encoder_idx, rect, ccw, cw) in &encoder_groups {
            let center = rect.center();
            let base_radius = rect.width().min(rect.height()) * LAYOUT_ENCODER_RADIUS_FACTOR;
            let hover_radius = base_radius * ENCODER_HOVER_SCALE;
            let interactive_radius = if encoder_hover_enlarge {
                hover_radius
            } else {
                base_radius
            };
            let circle_bounds = egui::Rect::from_center_size(
                center,
                egui::vec2(interactive_radius * 2.0, interactive_radius * 2.0),
            );
            let press_slot = encoder_press_rects
                .iter()
                .find(|(_, press_rect)| press_rect.center().distance(center) < 1.0)
                .map(|(press_ki, press_rect)| (*press_ki, *press_rect));
            let (top_rect, middle_rect, bottom_rect) = if let Some((_, press_rect)) = press_slot {
                let divider_gap = base_radius * 0.06;
                let top_divider_y = press_rect.top() - divider_gap;
                let bottom_divider_y = press_rect.bottom() + divider_gap;
                (
                    egui::Rect::from_min_max(
                        circle_bounds.min,
                        egui::pos2(circle_bounds.max.x, top_divider_y),
                    ),
                    Some(egui::Rect::from_min_max(
                        egui::pos2(circle_bounds.min.x, top_divider_y),
                        egui::pos2(circle_bounds.max.x, bottom_divider_y),
                    )),
                    egui::Rect::from_min_max(
                        egui::pos2(circle_bounds.min.x, bottom_divider_y),
                        circle_bounds.max,
                    ),
                )
            } else {
                (
                    egui::Rect::from_min_max(
                        circle_bounds.min,
                        egui::pos2(circle_bounds.max.x, center.y),
                    ),
                    None,
                    egui::Rect::from_min_max(
                        egui::pos2(circle_bounds.min.x, center.y),
                        circle_bounds.max,
                    ),
                )
            };
            let top_resp = ui.allocate_rect(top_rect, Sense::click());
            let middle_resp =
                middle_rect.map(|middle_rect| ui.allocate_rect(middle_rect, Sense::click()));
            let bottom_resp = ui.allocate_rect(bottom_rect, Sense::click());
            let encoder_hovered = top_resp.hovered()
                || middle_resp.as_ref().map(|r| r.hovered()).unwrap_or(false)
                || bottom_resp.hovered();
            let radius = if encoder_hovered && encoder_hover_enlarge {
                hover_radius
            } else {
                base_radius
            };
            let font_scale = if encoder_hovered && encoder_hover_enlarge {
                ENCODER_HOVER_SCALE
            } else {
                1.0
            };
            if encoder_hovered {
                hovered_encoder = true;
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            let ctrl_held = ui.input(|i| i.modifiers.ctrl);
            if top_resp.hovered() {
                if let Some((_, kc)) = cw {
                    hovered_encoder_keycode = Some(*kc);
                    let tip = keycode_tooltip_with_macro_names(
                        *kc,
                        &layout.custom_keycodes,
                        &self.layer_names,
                        &self.keycode_picker.macro_names,
                        &self.keycode_picker.tap_dance_names,
                    );
                    let _ = top_resp
                        .clone()
                        .on_hover_text(crate::i18n::tr_text(self.app_settings.language, &tip));
                }
            }
            if top_resp.secondary_clicked() {
                if let Some((visual_idx, kc)) = cw {
                    self.handle_secondary_target(ctrl_held, *kc, None, Some(*visual_idx));
                }
            }
            if top_resp.clicked() {
                if let Some((visual_idx, _)) = cw {
                    self.selected_key = None;
                    self.selected_encoder = Some((layer, *visual_idx));
                    self.keycode_picker.open = true;
                    self.keycode_picker.result = None;
                }
            }
            if let (Some((press_ki, _)), Some(middle_resp)) = (press_slot, middle_resp.as_ref()) {
                if middle_resp.hovered() {
                    hovered_key = Some(press_ki);
                    let kc = layout.get_keycode(self.selected_layer, press_ki);
                    hovered_encoder_keycode = Some(kc);
                    let tip = keycode_tooltip_with_macro_names(
                        kc,
                        &layout.custom_keycodes,
                        &self.layer_names,
                        &self.keycode_picker.macro_names,
                        &self.keycode_picker.tap_dance_names,
                    );
                    let _ = middle_resp
                        .clone()
                        .on_hover_text(crate::i18n::tr_text(self.app_settings.language, &tip));
                }
                if middle_resp.secondary_clicked() {
                    let kc = layout.get_keycode(self.selected_layer, press_ki);
                    self.handle_secondary_target(ctrl_held, kc, Some(press_ki), None);
                }
                if middle_resp.clicked() {
                    self.open_picker_for_target(Some(press_ki), None);
                    self.selected_encoder = None;
                }
            }
            if bottom_resp.hovered() {
                if let Some((_, kc)) = ccw {
                    hovered_encoder_keycode = Some(*kc);
                    let tip = keycode_tooltip_with_macro_names(
                        *kc,
                        &layout.custom_keycodes,
                        &self.layer_names,
                        &self.keycode_picker.macro_names,
                        &self.keycode_picker.tap_dance_names,
                    );
                    let _ = bottom_resp
                        .clone()
                        .on_hover_text(crate::i18n::tr_text(self.app_settings.language, &tip));
                }
            }
            if bottom_resp.secondary_clicked() {
                if let Some((visual_idx, kc)) = ccw {
                    self.handle_secondary_target(ctrl_held, *kc, None, Some(*visual_idx));
                }
            }
            if bottom_resp.clicked() {
                if let Some((visual_idx, _)) = ccw {
                    self.selected_key = None;
                    self.selected_encoder = Some((self.selected_layer, *visual_idx));
                    self.keycode_picker.open = true;
                    self.keycode_picker.result = None;
                }
            }

            let top_selected = cw
                .map(|(visual_idx, _)| Some((layer, visual_idx)) == self.selected_encoder)
                .unwrap_or(false);
            let bottom_selected = ccw
                .map(|(visual_idx, _)| Some((layer, visual_idx)) == self.selected_encoder)
                .unwrap_or(false);
            let middle_selected = press_slot
                .map(|(press_ki, _)| self.selected_key == Some((layer, press_ki)))
                .unwrap_or(false);
            let middle_hovered = middle_resp.as_ref().map(|r| r.hovered()).unwrap_or(false);
            let visuals = &ui.visuals().widgets;
            let fill_radius = radius + LAYOUT_ENCODER_FILL_EXTRA;
            let top_fill = if top_selected {
                visuals.active.bg_fill
            } else if top_resp.hovered() {
                visuals.hovered.bg_fill
            } else {
                visuals.inactive.bg_fill
            };
            let bottom_fill = if bottom_selected {
                visuals.active.bg_fill
            } else if bottom_resp.hovered() {
                visuals.hovered.bg_fill
            } else {
                visuals.inactive.bg_fill
            };
            let middle_fill = if middle_selected {
                visuals.active.bg_fill
            } else if middle_hovered {
                visuals.hovered.bg_fill
            } else {
                visuals.inactive.bg_fill
            };
            let outline = if top_selected || bottom_selected || middle_selected {
                visuals.active.bg_stroke
            } else if top_resp.hovered() || middle_hovered || bottom_resp.hovered() {
                visuals.hovered.bg_stroke
            } else {
                visuals.inactive.bg_stroke
            };

            let painter = ui.painter();
            painter.circle_filled(center, fill_radius, visuals.inactive.bg_fill);
            painter
                .with_clip_rect(top_rect)
                .circle_filled(center, fill_radius, top_fill);
            if let Some(middle_rect) = middle_rect {
                painter
                    .with_clip_rect(middle_rect)
                    .circle_filled(center, fill_radius, middle_fill);
            }
            painter
                .with_clip_rect(bottom_rect)
                .circle_filled(center, fill_radius, bottom_fill);
            painter.circle_stroke(center, radius, outline);

            let has_press_button = encoder_press_rects
                .iter()
                .any(|(_, press_rect)| press_rect.center().distance(center) < 1.0);
            let top_label = encoder_label(cw.map(|(_, kc)| kc).unwrap_or(0x0000));
            let bottom_label = encoder_label(ccw.map(|(_, kc)| kc).unwrap_or(0x0000));
            let top_font = if has_press_button {
                egui::FontId::proportional(
                    if top_label.chars().count() > 9 {
                        6.6
                    } else {
                        7.4
                    } * font_scale,
                )
            } else {
                egui::FontId::proportional(
                    if top_label.chars().count() > 9 {
                        8.5
                    } else {
                        9.5
                    } * font_scale,
                )
            };
            let bottom_font = if has_press_button {
                egui::FontId::proportional(
                    if bottom_label.chars().count() > 9 {
                        6.6
                    } else {
                        7.4
                    } * font_scale,
                )
            } else {
                egui::FontId::proportional(
                    if bottom_label.chars().count() > 9 {
                        8.5
                    } else {
                        9.5
                    } * font_scale,
                )
            };
            let top_label_y = center.y - radius * if has_press_button { 0.52 } else { 0.30 };
            let bottom_label_y = center.y + radius * if has_press_button { 0.52 } else { 0.30 };
            let top_text_color = if top_selected {
                visuals.active.fg_stroke.color
            } else if top_resp.hovered() {
                visuals.hovered.fg_stroke.color
            } else {
                visuals.inactive.fg_stroke.color
            };
            let bottom_text_color = if bottom_selected {
                visuals.active.fg_stroke.color
            } else if bottom_resp.hovered() {
                visuals.hovered.fg_stroke.color
            } else {
                visuals.inactive.fg_stroke.color
            };
            painter.text(
                egui::pos2(center.x, top_label_y),
                egui::Align2::CENTER_CENTER,
                top_label,
                top_font,
                top_text_color,
            );
            painter.text(
                egui::pos2(center.x, bottom_label_y),
                egui::Align2::CENTER_CENTER,
                bottom_label,
                bottom_font,
                bottom_text_color,
            );

            let arrow_color_top = outline.color;
            let arrow_color_bottom = outline.color;
            draw_encoder_arrow(painter, center, radius, true, arrow_color_top);
            draw_encoder_arrow(painter, center, radius, false, arrow_color_bottom);

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
                    Stroke::new(1.0, outline.color),
                );
                painter.line_segment(
                    [
                        egui::pos2(center.x - bottom_divider_half_width, bottom_divider_y),
                        egui::pos2(center.x + bottom_divider_half_width, bottom_divider_y),
                    ],
                    Stroke::new(1.0, outline.color),
                );
                let is_hovering = hover_alpha > 0.05;
                let text_color = if middle_selected {
                    visuals.active.fg_stroke.color
                } else if middle_hovered {
                    visuals.hovered.fg_stroke.color
                } else {
                    visuals.inactive.fg_stroke.color
                };
                let press_text_rect = middle_rect.shrink2(egui::vec2(4.0, 2.0));

                let press_label = {
                    let kc = layout.get_keycode(layer, press_ki);
                    if kc == 0x0001 && !is_hovering {
                        let fallback_kc = (0..layer)
                            .rev()
                            .map(|l| layout.get_keycode(l, press_ki))
                            .find(|&k| k != 0x0001)
                            .unwrap_or(0x0000);
                        if fallback_kc == 0x0000 || fallback_kc == 0x0001 {
                            "▽".to_string()
                        } else {
                            keycode_label_with_macro_names(
                                fallback_kc,
                                &layout.custom_keycodes,
                                &self.layer_names,
                                &self.keycode_picker.macro_names,
                                &self.keycode_picker.tap_dance_names,
                                self.app_settings.key_legend_layout,
                            )
                        }
                    } else if kc == 0x0001 {
                        "▽".to_string()
                    } else if kc == 0x0000 {
                        "✕".to_string()
                    } else {
                        keycode_label_with_macro_names(
                            kc,
                            &layout.custom_keycodes,
                            &self.layer_names,
                            &self.keycode_picker.macro_names,
                            &self.keycode_picker.tap_dance_names,
                            self.app_settings.key_legend_layout,
                        )
                    }
                }
                .replace('\n', " ");
                let press_font = FontId::proportional(
                    if press_label.chars().count() > 8 {
                        7.2
                    } else {
                        8.2
                    } * font_scale,
                );
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
                    Stroke::new(1.0, outline.color),
                );
            }
        }

        self.prev_hovered_key = hovered_key;
        self.prev_hovered_encoder = hovered_encoder;
        self.prev_hovered_encoder_keycode = hovered_encoder_keycode;

        if layout_h > avail.y {
            ui.allocate_space(Vec2::new(0.0, (layout_h - avail.y).max(0.0)));
        }
    }

    pub(super) fn draw_placeholder(&mut self, ui: &mut egui::Ui) {
        let key_w = 52.0_f32;
        let key_h = 52.0_f32;
        let gap = 6.0_f32;
        let total_w = 6.0 * (key_w + gap);
        let start_x = (ui.available_width() - total_w * 2.0 - 40.0) / 2.0;
        let start_y = ui.min_rect().top() + 40.0;

        let mut keys: Vec<(usize, egui::Rect, egui::Response)> = vec![];
        for half in 0..2_usize {
            let half_offset = if half == 0 { 0.0 } else { total_w + 40.0 };
            for row in 0..4_usize {
                for col in 0..6_usize {
                    let key_idx = half * 24 + row * 6 + col;
                    let rect = egui::Rect::from_min_size(
                        egui::pos2(
                            start_x + half_offset + col as f32 * (key_w + gap),
                            start_y + row as f32 * (key_h + gap),
                        ),
                        Vec2::new(key_w, key_h),
                    );
                    let response = ui.allocate_rect(rect, Sense::click());
                    keys.push((key_idx, rect, response));
                }
            }
        }

        for (key_idx, _, response) in &keys {
            if response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if response.clicked() {
                self.selected_key = Some((self.selected_layer, *key_idx));
            }
        }

        let painter = ui.painter();
        for (key_idx, rect, _) in &keys {
            let is_selected = self.selected_key == Some((self.selected_layer, *key_idx));
            let bg = if is_selected {
                Color32::from_rgb(70, 110, 190)
            } else {
                Color32::from_gray(45)
            };
            painter.rect(
                *rect,
                6.0,
                bg,
                Stroke::new(1.0, Color32::from_gray(80)),
                egui::StrokeKind::Inside,
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("K{key_idx}"),
                FontId::proportional(11.0),
                Color32::WHITE,
            );
        }
    }
}
