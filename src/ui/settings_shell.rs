use super::*;

impl EntropyApp {
    pub(super) fn draw_settings_screen(
        &mut self,
        ui: &mut egui::Ui,
        layout: &KeyboardLayout,
        ctx: &egui::Context,
        content_top: f32,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        if self.settings_tab == SettingsTab::MatrixTester {
            self.poll_matrix_tester(ctx, layout);
        }

        if self.settings_tab == SettingsTab::MatrixTester {
            if let Some(id) = ctx.memory(|m| m.focused()) {
                ctx.memory_mut(|m| m.surrender_focus(id));
            }
        }

        let dark = ui.visuals().dark_mode;
        let content_rect = egui::Rect::from_min_max(
            egui::pos2(ui.min_rect().left() + 20.0, content_top),
            egui::pos2(ui.min_rect().right() - 20.0, ui.max_rect().bottom() - 76.0),
        );

        match self.settings_tab {
            SettingsTab::AppSettings => {
                self.draw_app_settings_page(ui, content_rect);
            }
            SettingsTab::MatrixTester => {
                self.draw_matrix_tester_settings(ui, layout, content_rect, dark);
            }
            SettingsTab::UniversalSymbolsSetup => {
                self.draw_universal_symbols_setup_page(ui, content_rect);
            }
            SettingsTab::TextExpander => {
                self.draw_text_expander_settings_page(ui, content_rect);
            }
            SettingsTab::AutoShift => {
                self.draw_auto_shift_settings_page(ui, content_rect, dark);
            }
            SettingsTab::Rgb => {
                self.draw_rgb_settings_page(ui, content_rect, dark);
            }
            SettingsTab::LayerLeds => {
                self.draw_layer_led_settings_page(ui, content_rect);
            }
            SettingsTab::Encoders => {
                self.draw_encoder_visibility_settings_page(ui, content_rect, dark);
            }
            SettingsTab::TapHold => {
                self.draw_tap_hold_settings_page(ui, content_rect);
            }
            SettingsTab::Magic => {
                self.draw_magic_settings_page(ui, content_rect);
            }
            SettingsTab::GraveEscape => {
                self.draw_grave_escape_settings_page(ui, content_rect);
            }
            SettingsTab::LayoutOptions => {
                self.draw_layout_options_settings_page(ui, content_rect);
            }
            SettingsTab::Modules => {
                self.draw_module_settings_page(ui, content_rect);
            }
            SettingsTab::Touchpad => {
                self.draw_touchpad_settings_page(ui, content_rect);
            }
            SettingsTab::LiveFeatures => {
                self.draw_live_features_settings_page(ui, content_rect);
            }
            SettingsTab::Combo => {
                self.draw_combo_settings_page(ui, ctx, content_rect);
            }
            SettingsTab::KeyOverrides => {
                self.draw_key_override_settings_page(ui, content_rect);
            }
            SettingsTab::AltRepeat => {
                self.draw_alt_repeat_settings_page(ui, content_rect);
            }
            SettingsTab::MouseKeys => {
                self.draw_mouse_keys_settings_page(ui, content_rect);
            }
        }

        if self.settings_tab != SettingsTab::MatrixTester {
            self.draw_settings_navigation_hint(ui);
        }
    }

    pub(super) fn draw_settings_navigation_hint(&self, ui: &mut egui::Ui) {
        let hint_color = if ui.visuals().dark_mode {
            Color32::from_gray(100)
        } else {
            Color32::from_gray(160)
        };
        ui.painter().text(
            egui::pos2(ui.max_rect().center().x, ui.max_rect().bottom() - 36.0),
            egui::Align2::CENTER_CENTER,
            crate::i18n::tr_catalog(
                self.app_settings.language,
                "navigation.return_to_layout_hint",
            ),
            FontId::proportional(11.0),
            hint_color,
        );
    }

    pub(super) fn open_mouse_keys_settings_page(&mut self) {
        self.settings_tab = SettingsTab::MouseKeys;
        self.main_menu_tab = MainMenuTab::Settings;
    }

    pub(super) fn open_app_settings_page(&mut self) {
        self.settings_tab = SettingsTab::AppSettings;
        self.main_menu_tab = MainMenuTab::Settings;
    }

    pub(super) fn open_universal_symbols_setup_page(&mut self) {
        self.settings_tab = SettingsTab::UniversalSymbolsSetup;
        self.main_menu_tab = MainMenuTab::Settings;
    }

    pub(super) fn open_text_expander_settings_page(&mut self) {
        self.settings_tab = SettingsTab::TextExpander;
        self.main_menu_tab = MainMenuTab::Advanced;
    }

    pub(super) fn open_layer_led_settings_page(&mut self) {
        self.settings_tab = SettingsTab::LayerLeds;
        self.main_menu_tab = MainMenuTab::Settings;
    }

    pub(super) fn open_tap_hold_settings_page(&mut self) {
        self.settings_tab = SettingsTab::TapHold;
        self.main_menu_tab = MainMenuTab::Settings;
    }

    pub(super) fn open_magic_settings_page(&mut self) {
        self.settings_tab = SettingsTab::Magic;
        self.main_menu_tab = MainMenuTab::Settings;
    }

    pub(super) fn open_grave_escape_settings_page(&mut self) {
        self.settings_tab = SettingsTab::GraveEscape;
        self.main_menu_tab = MainMenuTab::Settings;
    }

    pub(super) fn can_return_from_settings_page(
        &self,
        ctx: &egui::Context,
        modal_or_popup_open_at_frame_start: bool,
        combo_capture_open_at_frame_start: bool,
        keyboard_input_wanted_at_frame_start: bool,
    ) -> bool {
        matches!(
            self.main_menu_tab,
            MainMenuTab::Settings | MainMenuTab::Advanced
        ) && self.settings_tab != SettingsTab::MatrixTester
            && !self.secondary_click_handled
            && !self.keycode_picker.open
            && !self.unlock_open
            && !self.vial_unlock_polling
            && !self.combo_capture_open
            && !modal_or_popup_open_at_frame_start
            && !combo_capture_open_at_frame_start
            && !keyboard_input_wanted_at_frame_start
            && !ctx.wants_keyboard_input()
            && !ctx.memory(|m| m.any_popup_open())
            && !self.top_dropdown_open(ctx)
    }
}
