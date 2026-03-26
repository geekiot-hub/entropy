use crate::device::DeviceManager;
use crate::keyboard::KeyboardLayout;
use crate::keycode::keycode_label;
use crate::keycode_picker::KeycodePicker;
use egui::{Color32, FontId, RichText, Sense, Stroke, Vec2};

pub struct EntropyApp {
    device_manager: DeviceManager,
    selected_device: Option<usize>,
    selected_layer: usize,
    selected_key: Option<(usize, usize)>, // (layer, key_index)
    layout: Option<KeyboardLayout>,
    layer_count: usize,
    keycode_picker: KeycodePicker,
    status_msg: String,
}

impl EntropyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            device_manager: DeviceManager::new(),
            selected_device: None,
            selected_layer: 0,
            selected_key: None,
            layout: None,
            layer_count: 4,
            keycode_picker: KeycodePicker::default(),
            status_msg: String::new(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn connect_device(&mut self, device_idx: usize) {
        use crate::vial::VialDevice;

        let dev = match self.device_manager.devices().get(device_idx) {
            Some(d) => d.clone(),
            None => {
                self.status_msg = "Device not found".into();
                return;
            }
        };

        let vial = match VialDevice::open(&dev.path) {
            Ok(v) => v,
            Err(e) => {
                self.status_msg = format!("Connect failed: {e}");
                log::error!("Failed to open device: {e}");
                return;
            }
        };

        // Get layer count
        let layer_count = vial.get_layer_count().unwrap_or(4);
        self.layer_count = layer_count as usize;

        // Get layout JSON
        let json = match vial.get_layout_json() {
            Ok(j) => j,
            Err(e) => {
                self.status_msg = format!("Failed to read layout: {e}");
                log::error!("Failed to get layout JSON: {e}");
                return;
            }
        };

        // Parse layout
        let mut layout = match KeyboardLayout::from_vial_json(&json) {
            Ok(l) => l,
            Err(e) => {
                self.status_msg = format!("Failed to parse layout: {e}");
                log::error!("Failed to parse vial JSON: {e}");
                return;
            }
        };

        // Load keycodes for all layers
        let num_keys = layout.keys.len();
        layout.layers = vec![vec![0u16; num_keys]; self.layer_count];
        for layer in 0..self.layer_count {
            for (ki, key) in layout.keys.iter().enumerate() {
                match vial.get_keycode(layer as u8, key.row, key.col) {
                    Ok(kc) => layout.layers[layer][ki] = kc,
                    Err(e) => log::warn!("get_keycode({layer},{},{}) failed: {e}", key.row, key.col),
                }
            }
        }

        self.layout = Some(layout);
        self.selected_key = None;
        self.status_msg = format!("Connected: {}", dev.name);
        log::info!("Connected to {} ({} layers, {} keys)", dev.name, self.layer_count, num_keys);
    }

    /// Assign a keycode to selected key and immediately write to device.
    #[cfg(not(target_arch = "wasm32"))]
    fn assign_keycode(&mut self, layer: usize, ki: usize, kc_value: u16) {
        use crate::vial::VialDevice;

        if let Some(layout) = &mut self.layout {
            layout.set_keycode(layer, ki, kc_value);
        }

        let dev = match self.selected_device.and_then(|i| self.device_manager.devices().get(i)) {
            Some(d) => d.clone(),
            None => return,
        };
        let layout = match &self.layout {
            Some(l) => l,
            None => return,
        };
        let key = match layout.keys.get(ki) {
            Some(k) => k.clone(),
            None => return,
        };
        match VialDevice::open(&dev.path) {
            Ok(vial) => {
                if let Err(e) = vial.set_keycode(layer as u8, key.row, key.col, kc_value) {
                    self.status_msg = format!("Write error: {e}");
                    log::error!("set_keycode failed: {e}");
                } else {
                    self.status_msg = format!("✓ Saved");
                }
            }
            Err(e) => {
                self.status_msg = format!("Connect error: {e}");
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_from_device(&mut self) {
        use crate::vial::VialDevice;

        let layout = match &mut self.layout {
            Some(l) => l,
            None => return,
        };
        let dev = match self.selected_device.and_then(|i| self.device_manager.devices().get(i)) {
            Some(d) => d.clone(),
            None => return,
        };
        let vial = match VialDevice::open(&dev.path) {
            Ok(v) => v,
            Err(e) => {
                self.status_msg = format!("Load failed: {e}");
                return;
            }
        };

        for layer in 0..self.layer_count {
            for (ki, key) in layout.keys.iter().enumerate() {
                match vial.get_keycode(layer as u8, key.row, key.col) {
                    Ok(kc) => layout.layers[layer][ki] = kc,
                    Err(e) => log::warn!("reload get_keycode failed: {e}"),
                }
            }
        }
        self.status_msg = "Loaded keycodes from device".into();
    }
}

impl eframe::App for EntropyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keycode picker result — real-time write to device
        if let Some(kc_value) = self.keycode_picker.result.take() {
            if let Some((layer, ki)) = self.selected_key {
                #[cfg(not(target_arch = "wasm32"))]
                self.assign_keycode(layer, ki, kc_value);
                #[cfg(target_arch = "wasm32")]
                if let Some(layout) = &mut self.layout {
                    layout.set_keycode(layer, ki, kc_value);
                }
            }
        }

        // Top bar
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("⌨ Entropy").size(18.0).strong());
                ui.separator();

                // Device selector
                let label = match self.selected_device {
                    Some(i) => self
                        .device_manager
                        .devices()
                        .get(i)
                        .map(|d| d.name.clone())
                        .unwrap_or("Unknown".into()),
                    None => "No device selected".into(),
                };

                let prev_selected = self.selected_device;
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

                // Auto-connect on device selection change
                #[cfg(not(target_arch = "wasm32"))]
                if self.selected_device != prev_selected {
                    if let Some(idx) = self.selected_device {
                        self.connect_device(idx);
                    }
                }

                if ui.button("🔄 Refresh").clicked() {
                    self.device_manager.scan();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("⬇ Load").clicked() {
                        self.load_from_device();
                    }

                    if !self.status_msg.is_empty() {
                        ui.label(
                            RichText::new(&self.status_msg)
                                .size(11.0)
                                .color(Color32::from_rgb(180, 180, 100)),
                        );
                    }
                });
            });
        });

        // Layer tabs
        egui::TopBottomPanel::top("layers").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Layer:");
                for i in 0..self.layer_count {
                    let active = self.selected_layer == i;
                    let btn = egui::Button::new(format!(" {i} ")).fill(if active {
                        Color32::from_rgb(80, 120, 200)
                    } else {
                        Color32::TRANSPARENT
                    });
                    if ui.add(btn).clicked() {
                        self.selected_layer = i;
                    }
                }
            });
        });

        // Key info panel
        egui::SidePanel::right("key_info")
            .min_width(220.0)
            .show(ctx, |ui| {
                ui.heading("Key");
                ui.separator();
                if let Some((layer, idx)) = self.selected_key {
                    if let Some(layout) = &self.layout {
                        if let Some(key) = layout.keys.get(idx) {
                            ui.label(format!("Key: {}", key.label));
                            ui.label(format!("Matrix: [{}, {}]", key.row, key.col));
                            let kc = layout.get_keycode(layer, idx);
                            ui.label(format!("Keycode: 0x{kc:04X} ({})", keycode_label(kc)));

                        }
                    } else {
                        ui.label(format!("Selected: key {idx} layer {layer}"));
                    }
                    if ui.button("✎ Change Keycode").clicked() {
                        self.keycode_picker.open = true;
                        self.keycode_picker.search_query.clear();
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

            if self.layout.is_some() {
                let layout = self.layout.clone().unwrap();
                self.draw_layout(ui, &layout);
            } else {
                // Fallback placeholder grid
                self.draw_placeholder(ui);
            }
        });

        // Keycode picker modal
        self.keycode_picker.show(ctx);
    }
}

impl EntropyApp {
    fn draw_layout(&mut self, ui: &mut egui::Ui, layout: &KeyboardLayout) {
        let unit = 54.0_f32; // 1 KLE unit in pixels
        let padding = 4.0_f32;

        let avail = ui.available_size();
        // Calculate bounding box for centering
        let mut max_x: f32 = 0.0;
        let mut max_y: f32 = 0.0;
        for key in &layout.keys {
            max_x = max_x.max(key.x + key.w);
            max_y = max_y.max(key.y + key.h);
        }
        let layout_w = max_x * unit;
        let layout_h = max_y * unit;
        let offset_x = (avail.x - layout_w).max(0.0) / 2.0 + ui.min_rect().left();
        let offset_y = ui.min_rect().top() + 20.0;

        // First pass: allocate rects, collect responses
        let mut rects: Vec<(usize, egui::Rect, egui::Response)> = Vec::with_capacity(layout.keys.len());
        for (ki, key) in layout.keys.iter().enumerate() {
            let x = offset_x + key.x * unit + padding;
            let y = offset_y + key.y * unit + padding;
            let w = key.w * unit - padding * 2.0;
            let h = key.h * unit - padding * 2.0;
            let rect = egui::Rect::from_min_size(egui::pos2(x, y), Vec2::new(w, h));
            let response = ui.allocate_rect(rect, Sense::click());
            rects.push((ki, rect, response));
        }

        // Handle clicks
        for (ki, _, response) in &rects {
            if response.clicked() {
                self.selected_key = Some((self.selected_layer, *ki));
            }
        }

        // Second pass: paint
        let painter = ui.painter();
        let layer = self.selected_layer;
        for (ki, rect, _) in &rects {
            let is_selected = self.selected_key == Some((layer, *ki));
            let bg = if is_selected {
                Color32::from_rgb(70, 110, 190)
            } else {
                Color32::from_gray(45)
            };
            let border = Stroke::new(1.0, Color32::from_gray(80));
            painter.rect(*rect, 6.0, bg, border, egui::StrokeKind::Inside);

            let kc = layout.get_keycode(layer, *ki);
            let label = keycode_label(kc);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                label,
                FontId::proportional(11.0),
                Color32::WHITE,
            );
        }

        // Scroll hint for large layouts
        if layout_h > avail.y {
            ui.allocate_space(Vec2::new(0.0, (layout_h - avail.y).max(0.0)));
        }
    }

    fn draw_placeholder(&mut self, ui: &mut egui::Ui) {
        let key_w = 52.0_f32;
        let key_h = 52.0_f32;
        let gap = 6.0_f32;

        let total_w = 6.0 * (key_w + gap);
        let start_x = (ui.available_width() - total_w * 2.0 - 40.0) / 2.0;
        let start_y = ui.min_rect().top() + 40.0;

        // First pass: allocate rects, collect responses
        let mut keys: Vec<(usize, egui::Rect, egui::Response)> = vec![];
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
                    let response = ui.allocate_rect(rect, Sense::click());
                    keys.push((key_idx, rect, response));
                }
            }
        }

        // Handle clicks
        for (key_idx, _, response) in &keys {
            if response.clicked() {
                self.selected_key = Some((self.selected_layer, *key_idx));
            }
        }

        // Second pass: paint
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
