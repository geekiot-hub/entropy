use crate::device::DeviceManager;

fn layer_names_path() -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    std::fs::create_dir_all(&dir).ok();
    dir.join("layer_names.json")
}

fn load_layer_names() -> Vec<String> {
    let path = layer_names_path();
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(mut v) = serde_json::from_str::<Vec<String>>(&data) {
            if !v.is_empty() {
                // Pad to at least 16 so indexing is always safe
                while v.len() < 16 {
                    let n = v.len();
                    v.push(n.to_string());
                }
                return v;
            }
        }
    }
    let mut v: Vec<String> = (0..16).map(|i| i.to_string()).collect();
    v[0] = "Main".to_string();
    v
}

fn save_layer_names(names: &[String]) {
    // Always save at least 16 slots so load_layer_names can detect a valid file
    let mut full = names.to_vec();
    while full.len() < 16 {
        let n = full.len();
        full.push(n.to_string());
    }
    if let Ok(data) = serde_json::to_string(&full) {
        std::fs::write(layer_names_path(), data).ok();
    }
}
use crate::keyboard::KeyboardLayout;
use crate::keycode::keycode_label_with_names;
use crate::keycode_picker::KeycodePicker;
use egui::{Color32, FontId, RichText, Sense, Stroke, Vec2};

#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc;

/// Result sent back from the background connect thread.
#[cfg(not(target_arch = "wasm32"))]
struct ConnectResult {
    device_name: String,
    layout: KeyboardLayout,
    layer_count: usize,
}

#[cfg(not(target_arch = "wasm32"))]
enum ConnectState {
    Idle,
    Loading(mpsc::Receiver<Result<ConnectResult, String>>),
}

pub struct EntropyApp {
    device_manager: DeviceManager,
    selected_device: Option<usize>,
    selected_layer: usize,
    selected_key: Option<(usize, usize)>,
    layout: Option<KeyboardLayout>,
    layer_count: usize,
    keycode_picker: KeycodePicker,
    status_msg: String,
    #[cfg(not(target_arch = "wasm32"))]
    connect_state: ConnectState,
    /// Persistent open HID device for real-time writes
    #[cfg(not(target_arch = "wasm32"))]
    hid_device: Option<crate::hid::HidDevice>,
    dark_mode: bool,
    layer_names: Vec<String>,
    editing_layer: Option<usize>, // layer being renamed
    editing_layer_text: String,
}

impl EntropyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            #[cfg(not(target_arch = "wasm32"))]
            hid_device: None,

            device_manager: DeviceManager::new(),
            selected_device: None,
            selected_layer: 0,
            selected_key: None,
            layout: None,
            layer_count: 4,
            keycode_picker: KeycodePicker::default(),
            status_msg: String::new(),
            dark_mode: false,
            layer_names: load_layer_names(),
            editing_layer: None,
            editing_layer_text: String::new(),
            #[cfg(not(target_arch = "wasm32"))]
            connect_state: ConnectState::Idle,
        };
        // Auto-connect to first device if available
        #[cfg(not(target_arch = "wasm32"))]
        if !app.device_manager.devices().is_empty() {
            app.selected_device = Some(0);
            app.start_connect(0);
        }
        app
    }

    /// Spawn background thread to connect + load layout/keycodes.
    #[cfg(not(target_arch = "wasm32"))]
    fn start_connect(&mut self, device_idx: usize) {
        let dev = match self.device_manager.devices().get(device_idx) {
            Some(d) => d.clone(),
            None => {
                self.status_msg = "Device not found".into();
                return;
            }
        };

        self.status_msg = format!("Connecting to {}…", dev.name);
        self.layout = None;
        self.selected_key = None;

        let (tx, rx) = mpsc::channel();
        self.connect_state = ConnectState::Loading(rx);

        std::thread::spawn(move || {
            let result = (|| -> Result<ConnectResult, String> {
                use crate::hid::HidDevice;

                log::info!("Opening HID device: {}", dev.path);
                let dev_conn = HidDevice::open(&dev.path)
                    .map_err(|e| format!("Open failed: {e}"))?;

                log::info!("Getting protocol version…");
                match dev_conn.get_protocol_version() {
                    Ok(v) => log::info!("VIA protocol version: {v}"),
                    Err(e) => log::warn!("get_protocol_version failed: {e}"),
                }

                log::info!("Getting layer count…");
                let layer_count = dev_conn.get_layer_count()
                    .map(|c| c as usize)
                    .unwrap_or_else(|e| { log::warn!("get_layer_count failed: {e}, defaulting to 4"); 4 });
                log::info!("Layer count: {layer_count}");

                log::info!("Getting layout JSON…");
                let json = dev_conn.get_layout_json()
                    .map_err(|e| format!("Layout read failed: {e}"))?;
                log::info!("Layout JSON received, parsing…");

                let mut layout = KeyboardLayout::from_vial_json(&json)
                    .map_err(|e| format!("Layout parse failed: {e}"))?;
                log::info!("Layout parsed: {} keys, {}x{}", layout.keys.len(), layout.rows, layout.cols);

                let num_keys = layout.keys.len();
                layout.layers = vec![vec![0u16; num_keys]; layer_count];

                log::info!("Reading keymap buffer…");
                match dev_conn.get_keymap_buffer(layer_count, layout.rows, layout.cols) {
                    Ok(buf) => {
                        for layer in 0..layer_count {
                            for (ki, key) in layout.keys.iter().enumerate() {
                                let idx = layer * layout.rows * layout.cols
                                    + key.row as usize * layout.cols
                                    + key.col as usize;
                                if let Some(&kc) = buf.get(idx) {
                                    layout.layers[layer][ki] = kc;
                                }
                            }
                        }
                        log::info!("Keymap loaded from buffer");
                    }
                    Err(e) => {
                        log::warn!("get_keymap_buffer failed: {e}, skipping keycodes");
                    }
                }

                Ok(ConnectResult {
                    device_name: dev.name.clone(),
                    layout,
                    layer_count,
                })
            })();

            let _ = tx.send(result);
        });
    }

    /// Poll background thread for connect result.
    #[cfg(not(target_arch = "wasm32"))]
    fn poll_connect(&mut self, ctx: &egui::Context) {
        let result = match &self.connect_state {
            ConnectState::Loading(rx) => match rx.try_recv() {
                Ok(r) => Some(r),
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint(); // keep polling
                    return;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.status_msg = "Connect thread died".into();
                    self.connect_state = ConnectState::Idle;
                    return;
                }
            },
            ConnectState::Idle => return,
        };

        self.connect_state = ConnectState::Idle;

        match result.unwrap() {
            Ok(r) => {
                self.layer_count = r.layer_count;
                self.status_msg = format!("Connected: {}", r.device_name);
                // Populate custom keycodes in picker
                const USER_BASE: u16 = 0x7E40;
                self.keycode_picker.custom_keycodes = r.layout.custom_keycodes.iter().enumerate()
                    .map(|(i, (name, label))| (name.clone(), label.clone(), USER_BASE + i as u16))
                    .collect();
                let layout_rows = r.layout.rows;
                let layout_cols = r.layout.cols;
                self.layout = Some(r.layout);
                // Open persistent HID connection for real-time writes
                if let Some(dev) = self.selected_device.and_then(|i| self.device_manager.devices().get(i)) {
                    match crate::hid::HidDevice::open(&dev.path) {
                        Ok(v) => {

                            self.hid_device = Some(v);
                        }
                        Err(e) => log::warn!("Could not open persistent HID: {e}"),
                    }
                }


                log::info!("Connected: {} ({} layers)", r.device_name, r.layer_count);
            }
            Err(e) => {
                self.status_msg = e;
            }
        }
    }

    /// Assign keycode and immediately write to device (blocking, but single HID op — fast).
    #[cfg(not(target_arch = "wasm32"))]
    fn assign_keycode(&mut self, layer: usize, ki: usize, kc_value: u16) {
        // Update in-memory layout
        if let Some(layout) = &mut self.layout {
            layout.set_keycode(layer, ki, kc_value);
        }

        let key = match self.layout.as_ref().and_then(|l| l.keys.get(ki)) {
            Some(k) => k.clone(),
            None => return,
        };

        // Use persistent connection if available, otherwise open fresh
        let result = if let Some(conn) = &self.hid_device {
            conn.set_keycode(layer as u8, key.row, key.col, kc_value)
        } else if let Some(dev) = self.selected_device
            .and_then(|i| self.device_manager.devices().get(i))
        {
            match crate::hid::HidDevice::open(&dev.path) {
                Ok(conn) => conn.set_keycode(layer as u8, key.row, key.col, kc_value),
                Err(e) => Err(anyhow::anyhow!("{e}")),
            }
        } else {
            return;
        };

        match result {
            Ok(()) => self.status_msg = "✓ Saved".into(),
            Err(e) => {
                self.status_msg = format!("Write error: {e}");
                // Connection lost — reopen
                self.hid_device = None;
            }
        }
    }

    /// Reload all keycodes from device in background.
    #[cfg(not(target_arch = "wasm32"))]
    fn load_from_device(&mut self) {
        if let Some(idx) = self.selected_device {
            self.start_connect(idx);
        }
    }
}

impl eframe::App for EntropyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply theme
        if self.dark_mode {
            let mut v = egui::Visuals::dark();
            // VS Code Dark+ style
            v.panel_fill = Color32::from_rgb(30, 30, 30);
            v.window_fill = Color32::from_rgb(37, 37, 38);
            v.faint_bg_color = Color32::from_rgb(37, 37, 38);
            v.extreme_bg_color = Color32::from_rgb(24, 24, 24);
            v.widgets.noninteractive.bg_fill = Color32::from_rgb(37, 37, 38);
            v.widgets.inactive.bg_fill = Color32::from_rgb(45, 45, 48);
            v.widgets.hovered.bg_fill = Color32::from_rgb(60, 60, 65);
            v.widgets.active.bg_fill = Color32::from_rgb(91, 104, 223);
            v.selection.bg_fill = Color32::from_rgba_unmultiplied(91, 104, 223, 120);
            ctx.set_visuals(v);
        } else {
            let mut v = egui::Visuals::light();
            // Figma/Linear-style light
            v.panel_fill = Color32::from_rgb(245, 245, 245);
            v.window_fill = Color32::from_rgb(255, 255, 255);
            v.faint_bg_color = Color32::from_rgb(245, 245, 245);
            v.extreme_bg_color = Color32::from_rgb(235, 235, 235);
            v.widgets.noninteractive.bg_fill = Color32::from_rgb(245, 245, 245);
            v.widgets.inactive.bg_fill = Color32::from_rgb(255, 255, 255);
            v.widgets.hovered.bg_fill = Color32::from_rgb(235, 235, 235);
            v.widgets.active.bg_fill = Color32::from_rgb(91, 104, 223);
            v.selection.bg_fill = Color32::from_rgba_unmultiplied(91, 104, 223, 80);
            ctx.set_visuals(v);
        }

        // Poll background connect thread
        #[cfg(not(target_arch = "wasm32"))]
        self.poll_connect(ctx);



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
            self.selected_key = None;
        }

        // Deselect key when picker is closed without choosing
        if !self.keycode_picker.open && self.selected_key.is_some() {
            self.selected_key = None;
        }

        // Check if loading
        #[cfg(not(target_arch = "wasm32"))]
        let is_loading = matches!(self.connect_state, ConnectState::Loading(_));
        #[cfg(target_arch = "wasm32")]
        let is_loading = false;

        // Top bar
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("⌨ Entropy").size(18.0).strong());
                ui.separator();

                let label = match self.selected_device {
                    Some(i) => self.device_manager.devices().get(i)
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

                #[cfg(not(target_arch = "wasm32"))]
                if self.selected_device != prev_selected {
                    if let Some(idx) = self.selected_device {
                        self.start_connect(idx);
                    }
                }

                if ui.button("🔄 Refresh").clicked() {
                    self.device_manager.scan();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let load_btn = egui::Button::new("⬇ Load")
                            .sense(if is_loading { Sense::hover() } else { Sense::click() });
                        if ui.add(load_btn).clicked() && !is_loading {
                            self.load_from_device();
                        }


                    }

                    if !self.status_msg.is_empty() {
                        let color = if self.status_msg.starts_with("✓") {
                            Color32::from_rgb(100, 200, 100)
                        } else if self.status_msg.contains("error") || self.status_msg.contains("failed") {
                            Color32::from_rgb(220, 80, 80)
                        } else {
                            Color32::from_rgb(180, 180, 100)
                        };
                        ui.label(RichText::new(&self.status_msg).size(11.0).color(color));
                    }

                    if is_loading {
                        ui.spinner();
                    }
                });
            });
        });





        // Main canvas
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

            if is_loading {
                ui.centered_and_justified(|ui| {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(RichText::new("Loading keyboard…").size(16.0).color(Color32::GRAY));
                    });
                });
                return;
            }

            if self.layout.is_some() {
                let layout = self.layout.clone().unwrap();
                self.draw_layout(ui, &layout);
            } else {
                self.draw_placeholder(ui);
            }
        });

        // Bottom bar — theme toggle
        egui::TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let theme_label = if self.dark_mode { "☀ Light" } else { "🌙 Dark" };
                if ui.small_button(theme_label).clicked() {
                    self.dark_mode = !self.dark_mode;
                }
            });
        });

        // Keycode picker modal
        self.keycode_picker.show(ctx);
    }
}

impl EntropyApp {
    fn draw_layout(&mut self, ui: &mut egui::Ui, layout: &KeyboardLayout) {
        let base_unit = 54.0_f32 * 1.15; // +15%
        let padding = 4.0_f32;

        let avail = ui.available_size();

        // Calculate layout bounding box (min AND max to handle rx offsets)
        let mut min_x: f32 = f32::MAX;
        let mut min_y: f32 = f32::MAX;
        let mut max_x: f32 = f32::MIN;
        let mut max_y: f32 = f32::MIN;
        for key in &layout.keys {
            min_x = min_x.min(key.x);
            min_y = min_y.min(key.y);
            max_x = max_x.max(key.x + key.w);
            max_y = max_y.max(key.y + key.h);
        }
        if min_x == f32::MAX { min_x = 0.0; min_y = 0.0; max_x = 1.0; max_y = 1.0; }

        let span_x = max_x - min_x;
        let span_y = max_y - min_y;

        // Scale unit to fit available space with some margin
        let margin = 40.0_f32;
        let scale_x = (avail.x - margin) / (span_x * base_unit).max(1.0);
        let scale_y = (avail.y - margin) / (span_y * base_unit).max(1.0);
        let scale = scale_x.min(scale_y).min(1.0);
        let unit = base_unit * scale;

        let layout_w = span_x * unit;
        let layout_h = span_y * unit;
        // Reserve space at top for layer switcher
        let layer_bar_h = 68.0_f32;
        let offset_x = (avail.x - layout_w) / 2.0 + ui.min_rect().left() - min_x * unit;
        let offset_y = (avail.y - layout_h - layer_bar_h) / 2.0 + ui.min_rect().top() - min_y * unit + layer_bar_h;

        // ── Layer switcher ─────────────────────────────────────────────────
        {
            let layer_count = self.layer_count;
            let selected = self.selected_layer;
            // raw_name — чистое имя без префикса, хранится в layer_names
            let raw_name = self.layer_names.get(selected).cloned().unwrap_or_else(|| selected.to_string());
            // display_name — с префиксом для отображения
            let display_name = if !raw_name.is_empty() && raw_name != selected.to_string() {
                format!("{}. {}", selected, raw_name)
            } else {
                raw_name.clone()
            };
            let name = display_name;
            let center_x = ui.min_rect().center().x;
            let bar_y = ui.min_rect().top() + (avail.y - layout_h - layer_bar_h) / 2.0 + 4.0;



            // Layer name / edit field
            let name_rect = egui::Rect::from_min_size(egui::pos2(center_x - 85.0, bar_y), Vec2::new(170.0, 52.0));

            let label_font = egui::FontId { size: 39.0, family: egui::FontFamily::Proportional };
            let text_color = if self.dark_mode { Color32::from_gray(245) } else { Color32::from_gray(60) };

            if self.editing_layer == Some(selected) {
                // Limit input to 7 chars
                if self.editing_layer_text.chars().count() > 7 {
                    let s: String = self.editing_layer_text.chars().take(7).collect();
                    self.editing_layer_text = s;
                }
                let resp = ui.put(name_rect,
                    egui::TextEdit::singleline(&mut self.editing_layer_text)
                        .font(label_font.clone())
                        .horizontal_align(egui::Align::Center)
                        .char_limit(7)
                        .frame(false)
                );
                resp.request_focus();
                // Commit on Enter, lost focus (click outside), or Escape
                let commit = resp.lost_focus() || ui.input(|inp| inp.key_pressed(egui::Key::Enter));
                let cancel = ui.input(|inp| inp.key_pressed(egui::Key::Escape));
                if commit || cancel {
                    if commit && !self.editing_layer_text.trim().is_empty() {
                        let new_name = self.editing_layer_text.trim().to_string();
                        while self.layer_names.len() <= selected { self.layer_names.push(self.layer_names.len().to_string()); }
                        self.layer_names[selected] = new_name;
                        save_layer_names(&self.layer_names);
                    }
                    self.editing_layer = None;
                }
            } else {
                // Measure text width first; clamp to a minimum so arrows
                // land in the right place even on the very first frame before
                // the font atlas is populated.
                let text_w = ui.fonts(|f| f.layout_no_wrap(name.clone(), label_font.clone(), text_color).size().x);
                let text_w = text_w.max(40.0); // never 0 → arrows always visible
                let half = (text_w / 2.0).ceil();
                let gap = 18.0_f32;
                let mid_y = bar_y + layer_bar_h / 2.0;

                // Allocate arrows FIRST (so they get input priority)
                let left_center  = egui::pos2(center_x - half - gap - 22.0, mid_y);
                let right_center = egui::pos2(center_x + half + gap + 22.0, mid_y);
                let left_hit  = egui::Rect::from_center_size(left_center,  Vec2::splat(48.0));
                let right_hit = egui::Rect::from_center_size(right_center, Vec2::splat(48.0));
                let left_r  = ui.allocate_rect(left_hit,  Sense::click());
                let right_r = ui.allocate_rect(right_hit, Sense::click());
                if left_r.hovered()  { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
                if right_r.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
                if left_r.clicked()  && selected > 0              { self.selected_layer = selected - 1; }
                if right_r.clicked() && selected + 1 < layer_count { self.selected_layer = selected + 1; }

                // Then allocate name (smaller, only over the text itself)
                let name_hit = egui::Rect::from_center_size(egui::pos2(center_x, mid_y), Vec2::new(text_w + 12.0, 52.0));
                let name_r = ui.allocate_rect(name_hit, Sense::click());
                if name_r.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
                if name_r.clicked() {
                    self.editing_layer = Some(selected);
                    self.editing_layer_text = raw_name.clone();
                }

                // Paint
                let dis = if self.dark_mode { Color32::from_gray(60) } else { Color32::from_gray(200) };
                let ac_l = if left_r.hovered()  { Color32::from_rgb(91,104,223) } else if self.dark_mode { Color32::from_gray(140) } else { Color32::from_gray(120) };
                let ac_r = if right_r.hovered() { Color32::from_rgb(91,104,223) } else if self.dark_mode { Color32::from_gray(140) } else { Color32::from_gray(120) };
                ui.painter().text(left_center,  egui::Align2::CENTER_CENTER, "‹", FontId::proportional(52.0), if selected == 0 { dis } else { ac_l });
                ui.painter().text(right_center, egui::Align2::CENTER_CENTER, "›", FontId::proportional(52.0), if selected + 1 >= layer_count { dis } else { ac_r });
                ui.painter().text(egui::pos2(center_x, mid_y), egui::Align2::CENTER_CENTER, &name, label_font, text_color);

                // Edit icon after text on hover
                if name_r.hovered() {
                    let icon_pos = egui::pos2(center_x + half + 6.0, mid_y);
                    ui.painter().text(icon_pos, egui::Align2::LEFT_CENTER, "✎", FontId::proportional(24.0), Color32::from_rgb(91, 104, 223));
                }
            }
        }

        // Pass 1: allocate
        let mut rects: Vec<(usize, egui::Rect, egui::Response)> =
            Vec::with_capacity(layout.keys.len());
        for (ki, key) in layout.keys.iter().enumerate() {
            let rect = egui::Rect::from_min_size(
                egui::pos2(
                    offset_x + key.x * unit + padding,
                    offset_y + key.y * unit + padding,
                ),
                Vec2::new(key.w * unit - padding * 2.0, key.h * unit - padding * 2.0),
            );
            let response = ui.allocate_rect(rect, Sense::click());
            rects.push((ki, rect, response));
        }

        // Pass 2: hover + clicks
        let mut hovered_key: Option<usize> = None;
        for (ki, _, response) in &rects {
            if response.hovered() {
                hovered_key = Some(*ki);
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if response.clicked() {
                self.selected_key = Some((self.selected_layer, *ki));
                self.keycode_picker.open = true;
                self.keycode_picker.search_query.clear();
                self.keycode_picker.layer_names = self.layer_names.clone();
            }
        }

        // Pass 3: paint
        let painter = ui.painter();
        let layer = self.selected_layer;
        let dark = self.dark_mode;
        for (ki, rect, _) in &rects {
            let key = &layout.keys[*ki];
            let is_selected = self.selected_key == Some((layer, *ki));
            let is_hovered = hovered_key == Some(*ki);
            // Accent: #5B68DF indigo
            let bg = if is_selected {
                Color32::from_rgb(91, 104, 223)
            } else if is_hovered {
                if dark { Color32::from_rgb(60, 60, 65) } else { Color32::from_rgb(232, 232, 240) }
            } else {
                if dark { Color32::from_rgb(48, 48, 52) } else { Color32::from_rgb(255, 255, 255) }
            };

            let draw_rect = if key.rotation != 0.0 {
                let angle_rad = key.rotation.to_radians();
                let ax = offset_x + key.rotation_x * unit;
                let ay = offset_y + key.rotation_y * unit;
                let anchor = egui::pos2(ax, ay);
                let center = rect.center();
                let dx = center.x - anchor.x;
                let dy = center.y - anchor.y;
                let rx = anchor.x + dx * angle_rad.cos() - dy * angle_rad.sin();
                let ry = anchor.y + dx * angle_rad.sin() + dy * angle_rad.cos();
                egui::Rect::from_center_size(egui::pos2(rx, ry), rect.size())
            } else {
                *rect
            };

            let kc = layout.get_keycode(layer, *ki);

            if kc == 0x0001 {
                // TRNS — transparent background, show fallback from lower layer dimmed
                // TRNS: same bg as normal key, same border — only text is dimmed
                painter.rect(draw_rect, 6.0, bg, Stroke::new(1.0, if dark { Color32::from_rgb(55, 55, 60) } else { Color32::from_rgb(210, 210, 218) }), egui::StrokeKind::Inside);
                let fallback_kc = (0..layer).rev()
                    .map(|l| layout.get_keycode(l, *ki))
                    .find(|&k| k != 0x0001)
                    .unwrap_or(0x0000);
                let label = if fallback_kc == 0x0000 || fallback_kc == 0x0001 {
                    "\u{25BD}".to_string()
                } else {
                    keycode_label_with_names(fallback_kc, &layout.custom_keycodes, &self.layer_names)
                };
                draw_key_label_dimmed(&painter, draw_rect, &label, dark);
            } else if kc == 0x0000 {
                let no_bg = if dark { Color32::from_rgb(20, 20, 22) } else { Color32::from_rgb(238, 238, 242) };
                let no_border = if dark { Color32::from_rgb(40, 40, 44) } else { Color32::from_rgb(210, 210, 218) };
                let no_text = if dark { Color32::from_rgb(55, 55, 65) } else { Color32::from_rgb(180, 180, 195) };
                painter.rect(draw_rect, 6.0, no_bg, Stroke::new(1.0, no_border), egui::StrokeKind::Inside);
                painter.text(draw_rect.center(), egui::Align2::CENTER_CENTER, "\u{2715}", FontId::proportional(10.0), no_text);
            } else {
                let border = if dark { Color32::from_rgb(55, 55, 60) } else { Color32::from_rgb(210, 210, 218) };
                painter.rect(draw_rect, 6.0, bg, Stroke::new(1.0, border), egui::StrokeKind::Inside);
                let label = keycode_label_with_names(kc, &layout.custom_keycodes, &self.layer_names);
                draw_key_label(&painter, draw_rect, &label, dark);
            }
        }

        if layout_h > avail.y {
            ui.allocate_space(Vec2::new(0.0, (layout_h - avail.y).max(0.0)));
        }
    }
}

fn draw_key_label_dimmed(painter: &egui::Painter, rect: egui::Rect, label: &str, dark: bool) {
    let dim = if dark { Color32::from_rgb(120, 120, 130) } else { Color32::from_rgb(160, 160, 170) };
    let dim_top = if dark { Color32::from_rgb(90, 90, 100) } else { Color32::from_rgb(185, 185, 195) };
    let (top, bottom) = if let Some(pos) = label.find('/') {
        (Some(&label[..pos]), &label[pos+1..])
    } else if label.contains('\n') {
        let mut parts = label.splitn(2, '\n');
        let t = parts.next().unwrap_or("");
        let b = parts.next().unwrap_or(label);
        (Some(t), b)
    } else {
        (None, label)
    };

    if let Some(top_str) = top {
        let center = rect.center();
        painter.text(egui::pos2(center.x, center.y - 7.0), egui::Align2::CENTER_CENTER, top_str, FontId::proportional(9.0), dim_top);
        painter.text(egui::pos2(center.x, center.y + 6.0), egui::Align2::CENTER_CENTER, bottom, FontId::proportional(11.0), dim);
    } else {
        painter.text(rect.center(), egui::Align2::CENTER_CENTER, bottom, FontId::proportional(11.0), dim);
    }
}

fn draw_key_label(painter: &egui::Painter, rect: egui::Rect, label: &str, dark: bool) {
    // Split on "/" or "\n" — show top part small+dim, bottom part normal
    let (top, bottom) = if let Some(pos) = label.find('/') {
        let t = &label[..pos];
        let b = &label[pos+1..];
        (Some(t), b)
    } else if label.contains('\n') {
        let mut parts = label.splitn(2, '\n');
        let t = parts.next().unwrap_or("");
        let b = parts.next().unwrap_or(label);
        (Some(t), b)
    } else {
        (None, label)
    };

    if let Some(top_str) = top {
        // Two-line layout
        let center = rect.center();
        let top_pos = egui::pos2(center.x, center.y - 7.0);
        let bot_pos = egui::pos2(center.x, center.y + 6.0);

        let top_color = if dark { Color32::from_rgb(130, 130, 145) } else { Color32::from_rgb(130, 130, 150) };
        let main_color = if dark { Color32::from_rgb(232, 232, 240) } else { Color32::from_rgb(26, 26, 30) };
        painter.text(top_pos, egui::Align2::CENTER_CENTER, top_str, FontId::proportional(9.0), top_color);
        painter.text(bot_pos, egui::Align2::CENTER_CENTER, bottom, FontId::proportional(11.0), main_color);
    } else {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            bottom,
            FontId::proportional(11.0),
            if dark { Color32::from_rgb(232, 232, 240) } else { Color32::from_rgb(26, 26, 30) },
        );
    }
}

impl EntropyApp {
    fn draw_placeholder(&mut self, ui: &mut egui::Ui) {
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
            if response.clicked() {
                self.selected_key = Some((self.selected_layer, *key_idx));
            }
        }

        let painter = ui.painter();
        for (key_idx, rect, _) in &keys {
            let is_selected = self.selected_key == Some((self.selected_layer, *key_idx));
            let bg = if is_selected { Color32::from_rgb(70, 110, 190) } else { Color32::from_gray(45) };
            painter.rect(*rect, 6.0, bg, Stroke::new(1.0, Color32::from_gray(80)), egui::StrokeKind::Inside);
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
