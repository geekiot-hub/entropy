use crate::device::{Device, DeviceManager};
use crate::keyboard::KeyboardLayout;
use egui::{Color32, FontId, RichText, Sense, Stroke, Vec2};

pub struct EntropyApp {
    device_manager: DeviceManager,
    selected_device: Option<usize>,
    selected_layer: usize,
    selected_key: Option<(usize, usize)>, // (layer, key_index)
    layouts: Vec<KeyboardLayout>,
    keycode_picker_open: bool,
    pending_assign: Option<(usize, usize)>, // key to assign
}

impl EntropyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            device_manager: DeviceManager::new(),
            selected_device: None,
            selected_layer: 0,
            selected_key: None,
            layouts: vec![],
            keycode_picker_open: false,
            pending_assign: None,
        }
    }
}

impl eframe::App for EntropyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top bar
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("⌨ Entropy").size(18.0).strong());
                ui.separator();

                // Device selector
                let label = match self.selected_device {
                    Some(i) => self.device_manager.devices().get(i)
                        .map(|d| d.name.clone())
                        .unwrap_or("Unknown".into()),
                    None => "No device selected".into(),
                };
                egui::ComboBox::from_id_salt("device_selector")
                    .selected_text(&label)
                    .show_ui(ui, |ui| {
                        for (i, dev) in self.device_manager.devices().iter().enumerate() {
                            ui.selectable_value(&mut self.selected_device, Some(i), &dev.name);
                        }
                        if self.device_manager.devices().is_empty() {
                            ui.label("No devices found");
                        }
                    });

                if ui.button("🔄 Refresh").clicked() {
                    self.device_manager.scan();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⬆ Flash").clicked() {
                        // TODO: flash to device
                    }
                    if ui.button("⬇ Load").clicked() {
                        // TODO: load from device
                    }
                });
            });
        });

        // Layer tabs
        egui::TopBottomPanel::top("layers").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Layer:");
                for i in 0..4 {
                    let active = self.selected_layer == i;
                    let btn = egui::Button::new(format!(" {i} "))
                        .fill(if active { Color32::from_rgb(80, 120, 200) } else { Color32::TRANSPARENT });
                    if ui.add(btn).clicked() {
                        self.selected_layer = i;
                    }
                }
            });
        });

        // Key info panel
        egui::SidePanel::right("key_info").min_width(220.0).show(ctx, |ui| {
            ui.heading("Key");
            ui.separator();
            if let Some((_layer, _idx)) = self.selected_key {
                ui.label("Selected: [placeholder]");
                ui.label("Keycode: KC_TRNS");
                if ui.button("✎ Change").clicked() {
                    self.keycode_picker_open = true;
                }
            } else {
                ui.label(RichText::new("Click a key to edit").color(Color32::GRAY));
            }
        });

        // Main canvas — keyboard layout
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.selected_device.is_none() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("Connect a keyboard and press Refresh")
                            .size(16.0)
                            .color(Color32::GRAY),
                    );
                });
                return;
            }

            // Placeholder: draw a simple split layout grid
            let key_w = 52.0_f32;
            let key_h = 52.0_f32;
            let gap = 6.0_f32;

            let total_w = 6.0 * (key_w + gap);
            let start_x = (ui.available_width() - total_w * 2.0 - 40.0) / 2.0;
            let start_y = ui.min_rect().top() + 40.0;

            let painter = ui.painter();

            for half in 0..2_usize {
                let half_offset = if half == 0 { 0.0 } else { total_w + 40.0 };

                for row in 0..4_usize {
                    for col in 0..6_usize {
                        let key_idx = half * 24 + row * 6 + col;
                        let x = start_x + half_offset + col as f32 * (key_w + gap);
                        let y = start_y + row as f32 * (key_h + gap);

                        let rect = egui::Rect::from_min_size(
                            egui::pos2(x, y),
                            Vec2::new(key_w, key_h),
                        );

                        let is_selected = self.selected_key == Some((self.selected_layer, key_idx));
                        let bg = if is_selected {
                            Color32::from_rgb(70, 110, 190)
                        } else {
                            Color32::from_gray(45)
                        };

                        let response = ui.allocate_rect(rect, Sense::click());
                        if response.clicked() {
                            self.selected_key = Some((self.selected_layer, key_idx));
                        }

                        painter.rect(rect, 6.0, bg, Stroke::new(1.0, Color32::from_gray(80)));
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
        });

        // Keycode picker modal
        if self.keycode_picker_open {
            egui::Window::new("Pick Keycode")
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.label("Keycode picker — coming soon");
                    if ui.button("Close").clicked() {
                        self.keycode_picker_open = false;
                    }
                });
        }
    }
}
