use super::*;

impl EntropyApp {
    pub(super) fn draw_layout_chrome(
        &mut self,
        ui: &mut egui::Ui,
        layout: &KeyboardLayout,
        ctx: &egui::Context,
        top_base_y: f32,
        main_tabs_h: f32,
        layer_bar_h: f32,
        top_reserved_h: f32,
    ) -> bool {
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
                return true;
            }

            self.draw_layout_layer_switcher_and_hints(ui, top_base_y, main_tabs_h, layer_bar_h);
        }
        false
    }
}
