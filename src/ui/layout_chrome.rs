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
            let top_tabs = self.draw_layout_top_tabs(
                ui,
                ctx,
                top_base_y,
                self.unlock_open || self.vial_unlock_polling,
            );

            if self.unlock_open || self.vial_unlock_polling {
                self.close_top_dropdowns(ctx);
            } else {
                self.draw_layout_top_dropdowns(
                    ui,
                    layout,
                    ctx,
                    top_tabs.lang,
                    top_tabs.device_tab_rect,
                    top_tabs.device_tab_hovered,
                    top_tabs.advanced_tab_rect,
                    top_tabs.advanced_tab_hovered,
                    top_tabs.settings_tab_rect,
                    top_tabs.settings_tab_hovered,
                );
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
