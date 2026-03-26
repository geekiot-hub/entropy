/// ORYX-style keycode picker modal.

use crate::keycode::{KeycodeCategory, KEYCODES};
use egui::{Color32, RichText, Vec2};

pub struct KeycodePicker {
    pub open: bool,
    pub selected_tab: KeycodeTab,
    pub search_query: String,
    /// The keycode chosen by the user, if any.
    pub result: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeycodeTab {
    Basic,
    Modifiers,
    Function,
    Media,
    Special,
    Layers,
}

impl KeycodeTab {
    pub const ALL: &'static [KeycodeTab] = &[
        KeycodeTab::Basic,
        KeycodeTab::Modifiers,
        KeycodeTab::Function,
        KeycodeTab::Media,
        KeycodeTab::Special,
        KeycodeTab::Layers,
    ];

    pub fn label(self) -> &'static str {
        match self {
            KeycodeTab::Basic => "Basic",
            KeycodeTab::Modifiers => "Modifiers",
            KeycodeTab::Function => "Function",
            KeycodeTab::Media => "Media",
            KeycodeTab::Special => "Special",
            KeycodeTab::Layers => "Layers",
        }
    }

    fn matches_category(self, cat: &KeycodeCategory) -> bool {
        matches!(
            (self, cat),
            (KeycodeTab::Basic, KeycodeCategory::Basic)
                | (KeycodeTab::Modifiers, KeycodeCategory::Modifier)
                | (KeycodeTab::Function, KeycodeCategory::Function)
                | (KeycodeTab::Media, KeycodeCategory::Media)
                | (KeycodeTab::Special, KeycodeCategory::Special)
                | (KeycodeTab::Layers, KeycodeCategory::Layer)
        )
    }
}

impl Default for KeycodePicker {
    fn default() -> Self {
        Self {
            open: false,
            selected_tab: KeycodeTab::Basic,
            search_query: String::new(),
            result: None,
        }
    }
}

impl KeycodePicker {
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }

        let mut still_open = true;
        egui::Window::new("Pick Keycode")
            .open(&mut still_open)
            .collapsible(false)
            .resizable(true)
            .min_size(Vec2::new(500.0, 400.0))
            .show(ctx, |ui| {
                // Search bar
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    ui.text_edit_singleline(&mut self.search_query);
                });
                ui.separator();

                // Tabs
                ui.horizontal(|ui| {
                    for tab in KeycodeTab::ALL {
                        let active = self.selected_tab == *tab;
                        let btn = egui::Button::new(tab.label()).fill(
                            if active {
                                Color32::from_rgb(80, 120, 200)
                            } else {
                                Color32::TRANSPARENT
                            },
                        );
                        if ui.add(btn).clicked() {
                            self.selected_tab = *tab;
                        }
                    }
                });
                ui.separator();

                // Keycode grid
                let search_lower = self.search_query.to_lowercase();
                let filtered: Vec<_> = KEYCODES
                    .iter()
                    .filter(|kc| {
                        if !search_lower.is_empty() {
                            kc.name.to_lowercase().contains(&search_lower)
                                || kc.label.to_lowercase().contains(&search_lower)
                        } else {
                            self.selected_tab.matches_category(&kc.category)
                        }
                    })
                    .collect();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for kc in &filtered {
                            let btn = egui::Button::new(
                                RichText::new(kc.label).size(13.0),
                            )
                            .min_size(Vec2::new(54.0, 36.0));

                            let resp = ui.add(btn);
                            if resp.clicked() {
                                self.result = Some(kc.value);
                                self.open = false;
                            }
                            resp.on_hover_text(kc.name);
                        }
                    });
                });
            });

        if !still_open {
            self.open = false;
        }
    }
}
