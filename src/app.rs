use crate::device::DeviceManager;
use crate::firmware::FirmwareProtocol;
use crate::zmk::{zmk_binding_label, zmk_binding_tooltip, ZmkBinding};

/// Sanitize a device name into a filesystem-safe slug.
fn device_id_slug(device_name: &str) -> String {
    device_name
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect()
}

fn layer_names_path(device_name: &str) -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    std::fs::create_dir_all(&dir).ok();
    let slug = device_id_slug(device_name);
    dir.join(format!("layer_names_{}.json", slug))
}

fn load_layer_names(device_name: &str) -> Vec<String> {
    let path = layer_names_path(device_name);
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

fn save_layer_names(names: &[String], device_name: &str) {
    // Always save at least 16 slots so load_layer_names can detect a valid file
    let mut full = names.to_vec();
    while full.len() < 16 {
        let n = full.len();
        full.push(n.to_string());
    }
    if let Ok(data) = serde_json::to_string(&full) {
        let path = layer_names_path(device_name);
        if let Err(e) = std::fs::write(&path, &data) {
            log::warn!("save_layer_names failed at {:?}: {e}", path);
        } else {
            log::info!("save_layer_names ok → {:?}", path);
        }
    }
}
use crate::keyboard::KeyboardLayout;
use crate::keycode::{keycode_label_with_names, keycode_tooltip};
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
    /// Persistent ZMK connection (if ZMK device)
    zmk_conn: Option<crate::zmk::ZmkConnection>,
    /// ZMK lock state at connect time
    zmk_lock_state: i32,
    /// Macro texts read from device
    macro_texts: Vec<String>,
}

#[cfg(not(target_arch = "wasm32"))]
enum ConnectState {
    Idle,
    Loading(mpsc::Receiver<Result<ConnectResult, String>>),
}

/// Message from background ZMK operation thread
#[cfg(not(target_arch = "wasm32"))]
enum ZmkOpResult {
    AddLayerOk { layer_idx: u32, layer_name: String },
    AddLayerFail(String),
    RemoveLayerOk,
    RemoveLayerFail(String),
    SaveOk,
    SaveFail(String),
    DiscardOk,
    DiscardFail(String),
    LockStateChanged(i32),
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
    /// Persistent open HID device for real-time writes (Vial)
    #[cfg(not(target_arch = "wasm32"))]
    hid_device: Option<crate::hid::HidDevice>,
    /// Persistent ZMK connection passed from the connect thread
    #[cfg(not(target_arch = "wasm32"))]
    zmk_conn: Option<crate::zmk::ZmkConnection>,
    /// Current firmware type (mirrors layout.firmware)
    firmware: FirmwareProtocol,
    zmk_base_layer_count: usize,
    zmk_no_extra_layers: bool,
    /// Undo stack: each entry is (layer, key_idx, old_vial_kc, old_zmk_binding)
    undo_stack: Vec<(usize, usize, u16, ZmkBinding)>,
    /// Frame counter for periodic device scan
    scan_frame: u32,
    /// Layer to preview on hover (None = show selected_layer)
    hover_layer: Option<usize>,
    /// Key index hovered in previous frame (for hint display)
    prev_hovered_key: Option<usize>,
    /// Set when secondary click was handled by a key (prevents global jump-back)
    secondary_click_handled: bool,
    /// Animation progress for hover layer preview (0.0 = hidden, 1.0 = fully shown)
    hover_layer_progress: f32,
    /// Stack of layers to return to on right-click (last = most recent)
    jump_back_stack: Vec<usize>,
    dark_mode: bool,
    layer_names: Vec<String>,
    editing_layer: Option<usize>, // layer being renamed
    editing_layer_text: String,
    editing_layer_focus_requested: bool,
    /// Current connected device name (for per-device layer names)
    current_device_name: String,
    /// ZMK lock state: 0=Locked, 1=Unlocked
    zmk_lock_state: i32,
    /// Whether ZMK has unsaved changes
    zmk_has_unsaved: bool,
    /// Vial unlock dialog open
    unlock_open: bool,
    vial_unlock_keys: Vec<(u8, u8)>,
    vial_unlock_polling: bool,
    vial_unlock_counter: u8,
    vial_unlock_total: u8,
    /// Channel for ZMK background operation results
    #[cfg(not(target_arch = "wasm32"))]
    zmk_op_rx: Option<mpsc::Receiver<ZmkOpResult>>,
}

impl EntropyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            #[cfg(not(target_arch = "wasm32"))]
            hid_device: None,
            #[cfg(not(target_arch = "wasm32"))]
            zmk_conn: None,
            firmware: FirmwareProtocol::Vial,
            zmk_base_layer_count: 0,
            zmk_no_extra_layers: false,
            undo_stack: Vec::new(),
            scan_frame: 0,
            hover_layer: None,
            prev_hovered_key: None,
            secondary_click_handled: false,
            hover_layer_progress: 0.0,
            jump_back_stack: Vec::new(),
            device_manager: DeviceManager::new(),
            selected_device: None,
            selected_layer: 0,
            selected_key: None,
            layout: None,
            layer_count: 4,
            keycode_picker: KeycodePicker::default(),
            status_msg: String::new(),
            dark_mode: false,
            layer_names: load_layer_names("default"),
            editing_layer: None,
            editing_layer_text: String::new(),
            editing_layer_focus_requested: false,
            current_device_name: String::new(),
            zmk_lock_state: 1, // Unlocked by default
            zmk_has_unsaved: false,
            unlock_open: false,
            vial_unlock_keys: vec![],
            vial_unlock_polling: false,
            vial_unlock_counter: 0,
            vial_unlock_total: 50,
            #[cfg(not(target_arch = "wasm32"))]
            zmk_op_rx: None,
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
        self.selected_layer = 0;
        self.hid_device = None;
        self.zmk_conn = None;
        self.zmk_has_unsaved = false;
        self.zmk_lock_state = 1; // assume unlocked until we know
        self.zmk_no_extra_layers = false;

        let (tx, rx) = mpsc::channel();
        self.connect_state = ConnectState::Loading(rx);

        std::thread::spawn(move || {
            let result = (|| -> Result<ConnectResult, String> {
                match dev.firmware {
                    FirmwareProtocol::Vial => {
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

                        let mut layout = KeyboardLayout::from_vial_json(&json)
                            .map_err(|e| format!("Layout parse failed: {e}"))?;

                        // Override coords from embedded layout
                        log::info!("Vial: looking up embedded layout for '{}'", dev.name);
                        if let Some((emb, ref_keys)) = crate::layouts::lookup_layout(&dev.name) {
                            log::info!("Vial: found embedded layout '{}' with {} keys", emb.name, ref_keys.len());
                            use std::collections::HashMap;
                            let ref_map: HashMap<(u8,u8), &crate::keyboard::PhysicalKey> =
                                ref_keys.iter().map(|k| ((k.row, k.col), k)).collect();
                            let mut patched = 0usize;
                            for key in &mut layout.keys {
                                if let Some(rk) = ref_map.get(&(key.row, key.col)) {
                                    key.x = rk.x;
                                    key.y = rk.y;
                                    key.rotation = 0.0;
                                    key.rotation_x = 0.0;
                                    key.rotation_y = 0.0;
                                    patched += 1;
                                }
                            }
                            log::info!("Vial: patched {} keys", patched);
                        } else {
                            log::info!("Vial: no embedded layout found for '{}'", dev.name);
                        }

                        let num_keys = layout.keys.len();
                        layout.layers = vec![vec![0u16; num_keys]; layer_count];

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
                                log::warn!("get_keymap_buffer failed: {e}");
                            }
                        }

                        // Read macros
                        let macro_texts = match dev_conn.get_macro_count() {
                            Ok(count) => {
                                log::info!("Macro count: {count}");
                                match dev_conn.get_macro_buffer_size() {
                                    Ok(size) => {
                                        log::info!("Macro buffer size: {size}");
                                        match dev_conn.get_macro_buffer(size) {
                                            Ok(buf) => crate::hid::HidDevice::parse_macros(&buf, count),
                                            Err(e) => { log::warn!("get_macro_buffer: {e}"); vec![String::new(); count as usize] }
                                        }
                                    }
                                    Err(e) => { log::warn!("get_macro_buffer_size: {e}"); vec![String::new(); count as usize] }
                                }
                            }
                            Err(e) => { log::warn!("get_macro_count: {e}"); vec![String::new(); 16] }
                        };

                        Ok(ConnectResult {
                            device_name: dev.name.clone(),
                            macro_texts,
                            layout,
                            layer_count,
                            zmk_conn: None,
                            zmk_lock_state: 1,
                        })
                    }

                    FirmwareProtocol::Zmk => {
                        use crate::zmk::ZmkConnection;

                        log::info!("Opening ZMK serial device: {}", dev.path);
                        let mut conn = ZmkConnection::open(&dev.path)
                            .map_err(|e| format!("ZMK open failed: {e}"))?;

                        // Check lock state (don't wait — just get it)
                        let lock_state = conn.get_lock_state()
                            .unwrap_or(crate::zmk_proto::core::LockState::Unlocked as i32);

                        // If locked, return early with the lock state so UI can show unlock modal
                        if lock_state == crate::zmk_proto::core::LockState::Locked as i32 {
                            log::warn!("ZMK keyboard is locked — returning for unlock UI");
                            // Return minimal result with lock state set
                            let layout = KeyboardLayout {
                                name: dev.name.clone(),
                                rows: 0,
                                cols: 0,
                                keys: vec![],
                                layers: vec![],
                                custom_keycodes: vec![],
                                firmware: FirmwareProtocol::Zmk,
                                zmk_bindings: vec![],
                                zmk_behaviors: vec![],
                                zmk_layer_ids: vec![],
                                zmk_layer_names: vec![],
                            };
                            return Ok(ConnectResult {
                                device_name: dev.name.clone(),
                                macro_texts: vec![],
                                layout,
                                layer_count: 0,
                                zmk_conn: Some(conn),
                                zmk_lock_state: lock_state,
                            });
                        }

                        log::info!("Fetching ZMK behaviors…");
                        conn.fetch_all_behaviors()
                            .map_err(|e| format!("ZMK behaviors failed: {e}"))?;

                        log::info!("Fetching ZMK keymap…");
                        let keymap = conn.get_keymap()
                            .map_err(|e| format!("ZMK keymap failed: {e}"))?;

                        log::info!("Fetching ZMK physical layouts…");
                        let phys = conn.get_physical_layouts()
                            .map_err(|e| format!("ZMK layouts failed: {e}"))?;

                        let layer_count = keymap.layers.len();
                        log::info!("ZMK: {} layers", layer_count);

                        // Build layout from device physical layout
                        let mut layout = crate::layouts::build_layout_from_zmk(&phys, &keymap);
                        layout.firmware = FirmwareProtocol::Zmk;
                        layout.zmk_behaviors = conn.behaviors.clone();

                        // Extract bindings
                        layout.zmk_layer_ids = keymap.layers.iter().map(|l| l.id).collect();
                        layout.zmk_layer_names = keymap.layers.iter().map(|l| l.name.clone()).collect();

                        let num_keys = layout.keys.len();
                        layout.zmk_bindings = keymap.layers.iter().map(|layer| {
                            let mut bindings = vec![crate::zmk::ZmkBinding::none(); num_keys];
                            for (i, b) in layer.bindings.iter().enumerate() {
                                if i < num_keys {
                                    bindings[i] = crate::zmk::ZmkBinding::from_proto(b);
                                }
                            }
                            bindings
                        }).collect();

                        Ok(ConnectResult {
                            device_name: dev.name.clone(),
                            macro_texts: vec![],
                            layout,
                            layer_count,
                            zmk_conn: Some(conn),
                            zmk_lock_state: lock_state,
                        })
                    }
                }
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
                self.firmware = r.layout.firmware;
                if r.layout.firmware == FirmwareProtocol::Zmk {
                    self.zmk_base_layer_count = r.layer_count;
                }
                self.current_device_name = r.device_name.clone();
                self.zmk_lock_state = r.zmk_lock_state;
                self.zmk_has_unsaved = false;
                if !r.macro_texts.is_empty() {
                    self.keycode_picker.macro_texts = r.macro_texts.clone();
                }

                // If ZMK keyboard is locked, show the unlock modal
                if r.zmk_lock_state == crate::zmk_proto::core::LockState::Locked as i32 {
                    self.status_msg = "Keyboard locked".into();
                    if let Some(conn) = r.zmk_conn {
                        self.zmk_conn = Some(conn);
                    }
                    // Start polling for unlock in background
                    self.start_zmk_lock_poll(ctx);
                    return;
                }

                self.status_msg = format!("Connected: {}", r.device_name);

                // ZMK: store connection
                if let Some(conn) = r.zmk_conn {
                    self.zmk_conn = Some(conn);
                }

                // Load per-device layer names
                let device_name = r.device_name.clone();
                // For ZMK, use device layer names if available; otherwise load from file
                if !r.layout.zmk_layer_names.is_empty() {
                    self.layer_names = r.layout.zmk_layer_names.clone();
                    while self.layer_names.len() < 16 {
                        let n = self.layer_names.len();
                        self.layer_names.push(n.to_string());
                    }
                } else {
                    self.layer_names = load_layer_names(&device_name);
                }

                // Populate picker based on firmware
                self.keycode_picker.firmware = self.firmware;
                if self.firmware == FirmwareProtocol::Vial {
                    const USER_BASE: u16 = 0x7E40;
                    self.keycode_picker.custom_keycodes = r.layout.custom_keycodes.iter().enumerate()
                        .map(|(i, (name, label))| (name.clone(), label.clone(), USER_BASE + i as u16))
                        .collect();
                } else {
                    self.keycode_picker.zmk_behaviors = r.layout.zmk_behaviors.clone();
                    self.keycode_picker.zmk_layer_count = r.layer_count;
                }
                self.keycode_picker.layer_names = self.layer_names.clone();

                self.layout = Some(r.layout);

                // Open persistent HID connection for Vial real-time writes
                if self.firmware == FirmwareProtocol::Vial {
                    if let Some(dev) = self.selected_device.and_then(|i| self.device_manager.devices().get(i)) {
                        match crate::hid::HidDevice::open(&dev.path) {
                            Ok(v) => { self.hid_device = Some(v); }
                            Err(e) => log::warn!("Could not open persistent HID: {e}"),
                        }
                    }
                }

                log::info!("Connected: {} ({} layers, {:?})", r.device_name, r.layer_count, self.firmware);
            }
            Err(e) => {
                self.status_msg = e;
            }
        }
    }

    /// Start polling for ZMK unlock in background.
    #[cfg(not(target_arch = "wasm32"))]
    fn start_zmk_lock_poll(&mut self, ctx: &egui::Context) {
        // Take the ZMK connection for the background thread
        let conn = match self.zmk_conn.take() {
            Some(c) => c,
            None => return,
        };

        let (tx, rx) = mpsc::channel();
        self.zmk_op_rx = Some(rx);

        let ctx_clone = ctx.clone();
        std::thread::spawn(move || {
            let mut conn = conn;
            // Poll lock state until unlocked
            loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                match conn.get_lock_state() {
                    Ok(state) => {
                        let _ = tx.send(ZmkOpResult::LockStateChanged(state));
                        ctx_clone.request_repaint();
                        if state == crate::zmk_proto::core::LockState::Unlocked as i32 {
                            break;
                        }
                    }
                    Err(e) => {
                        log::warn!("Lock poll error: {e}");
                    }
                }
            }
            // Return connection via a special channel — we just drop it here since
            // we can't easily return it. The UI will reconnect.
            drop(conn);
        });
    }

    /// Poll ZMK background operation results.
    #[cfg(not(target_arch = "wasm32"))]
    fn poll_zmk_ops(&mut self, ctx: &egui::Context) {
        let msg = match &self.zmk_op_rx {
            Some(rx) => match rx.try_recv() {
                Ok(m) => m,
                Err(_) => return,
            },
            None => return,
        };

        match msg {
            ZmkOpResult::LockStateChanged(state) => {
                self.zmk_lock_state = state;
                if state == crate::zmk_proto::core::LockState::Unlocked as i32 {
                    self.zmk_op_rx = None;
                    // Reconnect to load the keymap now that it's unlocked
                    if let Some(idx) = self.selected_device {
                        self.start_connect(idx);
                    }
                } else {
                    ctx.request_repaint();
                }
            }
            ZmkOpResult::AddLayerOk { layer_idx, layer_name } => {
                log::info!("Added layer at index {layer_idx}: {layer_name}");
                self.status_msg = "Added layer".into();
                self.zmk_has_unsaved = false;
                self.zmk_op_rx = None;
                // Reload to get updated layer list
                if let Some(idx) = self.selected_device {
                    self.start_connect(idx);
                }
            }
            ZmkOpResult::AddLayerFail(e) => {
                log::error!("Add layer failed: {e}");
                if e.contains(": 2") || e.contains("NoSpace") {
                    self.zmk_no_extra_layers = true;
                    self.status_msg = "Firmware doesn't support extra layers (CONFIG_ZMK_KEYMAP_LAYERS_EXTRA)".to_string();
                } else {
                    self.status_msg = format!("Add layer failed: {e}");
                }
                self.zmk_op_rx = None;
            }
            ZmkOpResult::RemoveLayerOk => {
                log::info!("Removed layer");
                self.status_msg = "Removed layer".into();
                self.zmk_has_unsaved = false;
                self.zmk_op_rx = None;
                if let Some(idx) = self.selected_device {
                    self.start_connect(idx);
                }
            }
            ZmkOpResult::RemoveLayerFail(e) => {
                log::error!("RemoveLayer error: {e}");
                self.status_msg = format!("RemoveLayer error: {e}");
                self.zmk_op_rx = None;
            }
            ZmkOpResult::SaveOk => {
                log::info!("Saved");
                self.status_msg = "✓ Saved".into();
                self.zmk_has_unsaved = false;
                self.zmk_op_rx = None;
            }
            ZmkOpResult::SaveFail(e) => {
                log::error!("Save error: {e}");
                self.status_msg = format!("Save error: {e}");
                self.zmk_op_rx = None;
            }
            ZmkOpResult::DiscardOk => {
                log::info!("Discard ok");
                self.status_msg = "Discarded".into();
                self.zmk_has_unsaved = false;
                self.zmk_op_rx = None;
                // Reload after discard
                if let Some(idx) = self.selected_device {
                    self.start_connect(idx);
                }
            }
            ZmkOpResult::DiscardFail(e) => {
                log::error!("Discard error: {e}");
                self.status_msg = format!("Discard error: {e}");
                self.zmk_op_rx = None;
            }
        }
    }

    /// Assign keycode and immediately write to device (blocking, but single HID op — fast).
    #[cfg(not(target_arch = "wasm32"))]
    fn assign_keycode(&mut self, layer: usize, ki: usize, kc_value: u16) {
        // Save old value for undo
        let old_kc = self.layout.as_ref().map(|l| l.get_keycode(layer, ki)).unwrap_or(0);
        self.undo_stack.push((layer, ki, old_kc, ZmkBinding::none()));
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

    /// ZMK: assign binding and write to device.
    #[cfg(not(target_arch = "wasm32"))]
    fn assign_zmk_binding(&mut self, layer: usize, ki: usize, binding: ZmkBinding) {
        let layer_id = self.layout.as_ref()
            .and_then(|l| l.zmk_layer_ids.get(layer).copied())
            .unwrap_or(layer as u32);

        // Save old binding for undo
        let old_binding = self.layout.as_ref().map(|l| l.get_zmk_binding(layer, ki)).unwrap_or_else(ZmkBinding::none);
        self.undo_stack.push((layer, ki, 0, old_binding));

        if let Some(layout) = &mut self.layout {
            layout.set_zmk_binding(layer, ki, binding.clone());
        }

        let conn = match self.zmk_conn.take() {
            Some(c) => c,
            None => {
                self.status_msg = "ZMK not connected".into();
                return;
            }
        };
        let mut conn = conn;
        match conn.set_layer_binding(layer_id, ki as i32, &binding) {
            Ok(()) => {
                self.status_msg = "✓ Saved".into();
                self.zmk_has_unsaved = true;
            }
            Err(e) => {
                self.status_msg = format!("ZMK write error: {e}");
            }
        }
        self.zmk_conn = Some(conn);
    }

    /// ZMK: save changes to flash in background.
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(not(target_arch = "wasm32"))]
    fn undo(&mut self) {
        let Some((layer, ki, old_kc, old_binding)) = self.undo_stack.pop() else { return };
        if self.firmware == FirmwareProtocol::Zmk {
            self.assign_zmk_binding(layer, ki, old_binding);
            // Remove the undo entry that assign_zmk_binding just pushed
            self.undo_stack.pop();
        } else {
            self.assign_keycode(layer, ki, old_kc);
            // Remove the undo entry that assign_keycode just pushed
            self.undo_stack.pop();
        }
    }

    fn zmk_save(&mut self) {
        if self.zmk_op_rx.is_some() { return; } // operation in progress
        let conn = match self.zmk_conn.take() {
            Some(c) => c,
            None => {
                self.status_msg = "No ZMK connection".into();
                return;
            }
        };

        let (tx, rx) = mpsc::channel();
        self.zmk_op_rx = Some(rx);

        std::thread::spawn(move || {
            let mut conn = conn;
            match conn.save_changes() {
                Ok(()) => {
                    let _ = tx.send(ZmkOpResult::SaveOk);
                }
                Err(e) => {
                    let _ = tx.send(ZmkOpResult::SaveFail(e.to_string()));
                }
            }
            // Put connection back — we drop it here since we can't easily return it
            // The caller will need to reconnect if needed
            drop(conn);
        });
        self.status_msg = "Saving…".into();
    }

    /// ZMK: discard unsaved changes in background.
    #[cfg(not(target_arch = "wasm32"))]
    fn zmk_discard(&mut self) {
        if self.zmk_op_rx.is_some() { return; }
        // Discard = just reload from device
        self.zmk_has_unsaved = false;
        self.status_msg = "Discarded".into();
        if let Some(idx) = self.selected_device {
            self.start_connect(idx);
        }
    }

    /// ZMK: add layer in background.
    #[cfg(not(target_arch = "wasm32"))]
    fn zmk_add_layer(&mut self) {
        if self.zmk_op_rx.is_some() { return; }
        let conn = match self.zmk_conn.take() {
            Some(c) => c,
            None => {
                self.status_msg = "No ZMK connection".into();
                return;
            }
        };

        let (tx, rx) = mpsc::channel();
        self.zmk_op_rx = Some(rx);

        std::thread::spawn(move || {
            let mut conn = conn;
            let res = conn.add_layer().and_then(|(idx, name)| {
                log::info!("save_changes after add_layer: layer={idx}");
                conn.save_changes()?;
                Ok((idx, name))
            });
            match res {
                Ok((idx, name)) => {
                    let _ = tx.send(ZmkOpResult::AddLayerOk { layer_idx: idx, layer_name: name });
                }
                Err(e) => {
                    let _ = tx.send(ZmkOpResult::AddLayerFail(e.to_string()));
                }
            }
            drop(conn);
        });
        self.status_msg = "Adding layer…".into();
    }

    /// ZMK: remove last layer in background.
    #[cfg(not(target_arch = "wasm32"))]
    fn zmk_remove_layer(&mut self) {
        if self.zmk_op_rx.is_some() { return; }
        if self.layer_count <= 1 {
            self.status_msg = "Cannot remove last layer".into();
            return;
        }
        let last_idx = (self.layer_count - 1) as u32;
        let conn = match self.zmk_conn.take() {
            Some(c) => c,
            None => {
                self.status_msg = "No ZMK connection".into();
                return;
            }
        };

        let (tx, rx) = mpsc::channel();
        self.zmk_op_rx = Some(rx);

        std::thread::spawn(move || {
            let mut conn = conn;
            let res = conn.remove_layer(last_idx).and_then(|()| {
                log::info!("save_changes after remove_layer: layer={last_idx}");
                conn.save_changes()
            });
            match res {
                Ok(()) => {
                    let _ = tx.send(ZmkOpResult::RemoveLayerOk);
                }
                Err(e) => {
                    let _ = tx.send(ZmkOpResult::RemoveLayerFail(e.to_string()));
                }
            }
            drop(conn);
        });
        self.status_msg = "Removing layer…".into();
    }
}

impl eframe::App for EntropyApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        std::process::exit(0);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-scan for new devices every ~2 seconds (120 frames at 60fps)
        self.secondary_click_handled = false;
        self.scan_frame += 1;
        if self.scan_frame >= 120 {
            self.scan_frame = 0;
            let prev_count = self.device_manager.devices().len();
            self.device_manager.scan();
            // Auto-connect if a new device appeared and nothing is connected
            if self.device_manager.devices().len() > prev_count
                && self.layout.is_none()
                && !matches!(self.connect_state, ConnectState::Loading(_))
            {
                self.start_connect(0);
            }
        }

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

        // Poll ZMK background operations
        #[cfg(not(target_arch = "wasm32"))]
        self.poll_zmk_ops(ctx);



        // Handle Vial keycode picker result
        // Don't consume result while macro editor is open
        if self.keycode_picker.macro_editor_open.is_none() || self.keycode_picker.macro_editor_open == Some(255) {
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
        }

        // Handle ZMK binding picker result
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(binding) = self.keycode_picker.zmk_result.take() {
            if let Some((layer, ki)) = self.selected_key {
                self.assign_zmk_binding(layer, ki, binding);
            }
            self.selected_key = None;
        }

        // Deselect key when picker is closed without choosing
        if !self.keycode_picker.open && self.selected_key.is_some() {
            self.selected_key = None;
        }

        // Arrow keys Left/Right switch layers (when picker is closed)
        if !self.keycode_picker.open {
            let layer_count = self.layer_count;
            ctx.input(|i| {
                if i.key_pressed(egui::Key::ArrowLeft) && self.selected_layer > 0 {
                    self.selected_layer -= 1;
                    self.jump_back_stack.clear();
                }
                if i.key_pressed(egui::Key::ArrowRight) && self.selected_layer + 1 < layer_count {
                    self.selected_layer += 1;
                    self.jump_back_stack.clear();
                }
            });
        }

        // Check if loading
        #[cfg(not(target_arch = "wasm32"))]
        let is_loading = matches!(self.connect_state, ConnectState::Loading(_));
        #[cfg(target_arch = "wasm32")]
        let is_loading = false;

        // ZMK unlock modal — show before everything else if locked
        #[cfg(not(target_arch = "wasm32"))]
        {
            let is_zmk_locked = self.firmware == FirmwareProtocol::Zmk
                && self.zmk_lock_state == crate::zmk_proto::core::LockState::Locked as i32
                && !is_loading;

            if is_zmk_locked {
                self.show_zmk_unlock_modal(ctx);
                return;
            }
        }

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


                #[cfg(not(target_arch = "wasm32"))]
                if !self.undo_stack.is_empty() {
                    if ui.button("↩ Undo").on_hover_text("Undo last change").clicked() {
                        self.undo();
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    {


                        // Vial: Unlock button
                        if self.firmware == FirmwareProtocol::Vial && self.layout.is_some() {
                            let is_unlocked = self.hid_device.as_ref()
                                .and_then(|hid| hid.get_unlock_status().ok())
                                .map(|(unlocked, _)| unlocked)
                                .unwrap_or(false);
                            if !is_unlocked {
                                if ui.add(egui::Button::new(RichText::new("🔒 Unlock")
                                    .color(Color32::from_rgb(220, 120, 60)))
                                    .fill(Color32::TRANSPARENT))
                                    .on_hover_text("Keyboard is locked — click to unlock")
                                    .clicked()
                                {
                                    self.unlock_open = true;
                                }
                            } else {
                                ui.label(RichText::new("🔓").size(14.0))
                                    .on_hover_text("Keyboard is unlocked");
                            }
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
                self.draw_layout(ui, &layout, ctx);
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
        // Vial unlock modal
        if self.unlock_open && self.firmware == FirmwareProtocol::Vial {
            // Start unlock if not yet polling
            if !self.vial_unlock_polling {
                if let Some(hid) = &self.hid_device {
                    match hid.unlock_start() {
                        Ok(keys) => {
                            self.vial_unlock_keys = keys;
                            self.vial_unlock_polling = true;
                        }
                        Err(e) => {
                            self.status_msg = format!("Unlock start failed: {e}");
                            self.unlock_open = false;
                        }
                    }
                }
            }
            // Poll unlock
            if self.vial_unlock_polling {
                if let Some(hid) = &self.hid_device {
                    match hid.unlock_poll() {
                        Ok((unlocked, _in_progress, counter)) => {
                            self.vial_unlock_counter = counter;
                            if unlocked {
                                self.status_msg = "Keyboard unlocked!".into();
                                self.unlock_open = false;
                                self.vial_unlock_polling = false;
                            }
                        }
                        Err(_) => {}
                    }
                }
                ctx.request_repaint_after(std::time::Duration::from_millis(50));
            }
            // Fullscreen overlay with layout and highlighted keys
            let unlock_keys = self.vial_unlock_keys.clone();
            let counter = self.vial_unlock_counter;
            let total = self.vial_unlock_total;

            egui::Area::new(egui::Id::new("unlock_overlay"))
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let screen = ui.ctx().screen_rect();
                    // Dim background
                    ui.painter().rect_filled(screen, 0.0, Color32::from_rgba_unmultiplied(0, 0, 0, 180));

                    let center_x = screen.center().x;
                    let top_y = screen.min.y + 40.0;

                    // Title
                    ui.painter().text(egui::pos2(center_x, top_y), egui::Align2::CENTER_CENTER,
                        "🔓 Unlock Keyboard", FontId::proportional(24.0), Color32::WHITE);

                    ui.painter().text(egui::pos2(center_x, top_y + 30.0), egui::Align2::CENTER_CENTER,
                        "Hold the highlighted keys simultaneously", FontId::proportional(14.0), Color32::from_gray(180));

                    // Progress bar
                    let progress = if total > 0 { 1.0 - (counter as f32 / total as f32) } else { 0.0 };
                    let bar_w = 300.0f32;
                    let bar_h = 12.0f32;
                    let bar_y = top_y + 55.0;
                    let bar_rect = egui::Rect::from_min_size(
                        egui::pos2(center_x - bar_w / 2.0, bar_y),
                        egui::Vec2::new(bar_w, bar_h),
                    );
                    ui.painter().rect(bar_rect, 4.0, Color32::from_gray(40), egui::Stroke::NONE, egui::StrokeKind::Inside);
                    let fill_rect = egui::Rect::from_min_size(
                        bar_rect.min,
                        egui::Vec2::new(bar_w * progress, bar_h),
                    );
                    ui.painter().rect(fill_rect, 4.0, Color32::from_rgb(91, 104, 223), egui::Stroke::NONE, egui::StrokeKind::Inside);

                    // Draw layout keys with highlighted unlock keys
                    if let Some(layout) = &self.layout {
                        let base_unit = 54.0f32 * 0.85;
                        let padding = 3.0f32;
                        let mut min_x = f32::MAX; let mut min_y = f32::MAX;
                        let mut max_x = f32::MIN; let mut max_y = f32::MIN;
                        for key in &layout.keys {
                            min_x = min_x.min(key.x); min_y = min_y.min(key.y);
                            max_x = max_x.max(key.x + key.w); max_y = max_y.max(key.y + key.h);
                        }
                        let span_x = max_x - min_x; let span_y = max_y - min_y;
                        let avail_w = screen.width() - 80.0;
                        let avail_h = screen.height() - 160.0;
                        let scale = (avail_w / (span_x * base_unit)).min(avail_h / (span_y * base_unit)).min(1.0);
                        let unit = base_unit * scale;
                        let layout_w = span_x * unit;
                        let layout_h = span_y * unit;
                        let off_x = center_x - layout_w / 2.0 - min_x * unit;
                        let off_y = bar_y + 30.0 + (avail_h - layout_h) / 2.0 - min_y * unit;

                        for (ki, key) in layout.keys.iter().enumerate() {
                            let is_unlock = unlock_keys.iter().any(|(r, c)| key.row == *r && key.col == *c);
                            let rect = egui::Rect::from_min_size(
                                egui::pos2(off_x + key.x * unit + padding, off_y + key.y * unit + padding),
                                egui::Vec2::new(key.w * unit - padding * 2.0, key.h * unit - padding * 2.0),
                            );
                            let bg = if is_unlock {
                                Color32::from_rgb(91, 104, 223)
                            } else {
                                Color32::from_rgba_unmultiplied(48, 48, 52, 120)
                            };
                            let border = if is_unlock { Color32::from_rgb(120, 130, 255) } else { Color32::from_gray(60) };
                            ui.painter().rect(rect, 5.0, bg, Stroke::new(1.0, border), egui::StrokeKind::Inside);

                            let kc = layout.get_keycode(0, ki);
                            let label = crate::keycode::keycode_label_with_names(kc, &layout.custom_keycodes, &self.layer_names);
                            let text_color = if is_unlock { Color32::WHITE } else { Color32::from_gray(80) };
                            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, &label,
                                FontId::proportional(9.0 * scale), text_color);
                        }
                    }

                    // Cancel button at bottom
                    let btn_rect = egui::Rect::from_center_size(
                        egui::pos2(center_x, screen.max.y - 40.0),
                        egui::Vec2::new(100.0, 32.0),
                    );
                    let btn_resp = ui.put(btn_rect, egui::Button::new("Cancel"));
                    if btn_resp.clicked() {
                        self.unlock_open = false;
                        self.vial_unlock_polling = false;
                    }
                });
        }

        self.keycode_picker.show(ctx);

        // Write macros to device if changed
        if self.keycode_picker.macros_dirty {
            self.keycode_picker.macros_dirty = false;
            if let Some(hid) = &self.hid_device {
                // Check unlock status first
                let unlocked = hid.get_unlock_status().map(|(u, _)| u).unwrap_or(false);
                if !unlocked {
                    self.status_msg = "Keyboard is locked — unlock first to save macros".into();
                } else if let Ok(size) = hid.get_macro_buffer_size() {
                    let buf = crate::hid::HidDevice::encode_macros(&self.keycode_picker.macro_texts, size);
                    match hid.set_macro_buffer(&buf) {
                        Ok(()) => self.status_msg = "Macros saved".into(),
                        Err(e) => self.status_msg = format!("Macro write error: {e}"),
                    }
                }
            }
        }

        // Right-click anywhere = pop back one step (only if NOT hovering a layer key and not handled by key)
        if !self.jump_back_stack.is_empty() && !self.keycode_picker.open && !self.secondary_click_handled {
            let esc_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));
            let rclick = self.hover_layer.is_none() && ctx.input(|i| i.pointer.secondary_clicked());
            if rclick || esc_pressed {
                if let Some(back_layer) = self.jump_back_stack.pop() {
                    self.selected_layer = back_layer;
                }
            }
        }
    }
}

impl EntropyApp {
    /// Show ZMK unlock modal overlay.
    #[cfg(not(target_arch = "wasm32"))]
    fn show_zmk_unlock_modal(&self, ctx: &egui::Context) {
        // Draw dimmed background panels
        egui::TopBottomPanel::top("topbar_locked").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("⌨ Entropy").size(18.0).strong());
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(
                        RichText::new("🔒 Keyboard locked")
                            .size(24.0)
                            .strong()
                            .color(Color32::from_rgb(220, 160, 60)),
                    );
                    ui.add_space(16.0);

                    // Check if there's a studio unlock behavior
                    let has_unlock_key = self.zmk_conn.is_none(); // after poll the conn is taken
                    if has_unlock_key {
                        ui.label(
                            RichText::new("Press the unlock key on your keyboard to allow editing.")
                                .size(14.0),
                        );
                    } else {
                        ui.label(
                            RichText::new("Hold the unlock combo on your keyboard to allow editing.")
                                .size(14.0),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Hold simultaneously: Studio Unlock key")
                                .size(12.0)
                                .color(Color32::GRAY),
                        );
                    }

                    ui.add_space(24.0);
                    ui.label(
                        RichText::new("Keep holding… Unlock ZMK Studio")
                            .size(12.0)
                            .color(Color32::GRAY),
                    );
                    ui.add_space(16.0);
                    ui.spinner();
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Waiting for unlock…")
                            .size(11.0)
                            .color(Color32::GRAY),
                    );
                });
            });
        });

        // Keep repainting while waiting
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
    }
}

impl EntropyApp {
    fn draw_layout(&mut self, ui: &mut egui::Ui, layout: &KeyboardLayout, ctx: &egui::Context) {
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

            // ZMK layer management buttons (+ Add / - Remove)
            if self.firmware == FirmwareProtocol::Zmk {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let op_busy = self.zmk_op_rx.is_some();
                    let can_remove = !op_busy && layer_count > self.zmk_base_layer_count.max(1);
                    // Place +/− just to the right of the → arrow
                    let fixed_half = 85.0_f32;
                    let gap = 16.0_f32;
                    let right_arrow_x = center_x + fixed_half + gap + 24.0;
                    let btn_x = right_arrow_x + 36.0;
                    let mid_y = bar_y + layer_bar_h / 2.0;
                    let sym_font = FontId::proportional(14.0);
                    let active_color = if self.dark_mode { Color32::from_gray(200) } else { Color32::from_gray(60) };
                    let disabled_color = if self.dark_mode { Color32::from_gray(70) } else { Color32::from_gray(180) };

                    let add_rect = egui::Rect::from_center_size(egui::pos2(btn_x, mid_y - 8.0), Vec2::splat(16.0));
                    let remove_rect = egui::Rect::from_center_size(egui::pos2(btn_x, mid_y + 8.0), Vec2::splat(16.0));
                    let can_add = !op_busy && !self.zmk_no_extra_layers;
                    let add_resp = ui.allocate_rect(add_rect, if can_add { Sense::click() } else { Sense::hover() });
                    let remove_resp = ui.allocate_rect(remove_rect, if can_remove { Sense::click() } else { Sense::hover() });
                    if add_resp.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
                    if remove_resp.hovered() && can_remove { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
                    let add_col = if add_resp.hovered() && can_add { Color32::from_rgb(91,104,223) } else if can_add { active_color } else { disabled_color };
                    let rem_col = if remove_resp.hovered() && can_remove { Color32::from_rgb(91,104,223) } else if can_remove { active_color } else { disabled_color };
                    ui.painter().text(add_rect.center(), egui::Align2::CENTER_CENTER, "+", sym_font.clone(), add_col);
                    ui.painter().text(remove_rect.center(), egui::Align2::CENTER_CENTER, "−", sym_font, rem_col);
                    let add_hover_text = if self.zmk_no_extra_layers { "Firmware doesn't support extra layers" } else { "Add layer" };
                    let add_clicked = add_resp.on_hover_text(add_hover_text).clicked();
                    let rem_clicked = remove_resp.on_hover_text(if can_remove { "Remove last layer" } else { "Cannot remove base layers" }).clicked();
                    if add_clicked && can_add { self.zmk_add_layer(); }
                    if rem_clicked && can_remove { self.zmk_remove_layer(); }
                }
            }


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
                // Request focus only on the first frame so lost_focus() works correctly.
                if !self.editing_layer_focus_requested {
                    resp.request_focus();
                    self.editing_layer_focus_requested = true;
                }
                // Commit on Enter or lost focus (click outside); cancel on Escape.
                let commit = resp.lost_focus() || ui.input(|inp| inp.key_pressed(egui::Key::Enter));
                let cancel = ui.input(|inp| inp.key_pressed(egui::Key::Escape));
                if commit || cancel {
                    if commit && !self.editing_layer_text.trim().is_empty() {
                        let new_name = self.editing_layer_text.trim().to_string();
                        while self.layer_names.len() <= selected { self.layer_names.push(self.layer_names.len().to_string()); }
                        self.layer_names[selected] = new_name.clone();
                        #[cfg(not(target_arch = "wasm32"))]
                        save_layer_names(&self.layer_names, &self.current_device_name);
                        #[cfg(target_arch = "wasm32")]
                        save_layer_names(&self.layer_names, "default");
                        // ZMK: also write name to device
                        #[cfg(not(target_arch = "wasm32"))]
                        if self.firmware == FirmwareProtocol::Zmk {
                            if let Some(conn) = &mut self.zmk_conn {
                                let layer_id = self.layout.as_ref()
                                    .and_then(|l| l.zmk_layer_ids.get(selected).copied())
                                    .unwrap_or(selected as u32);
                                if let Err(e) = conn.set_layer_name(layer_id, &new_name) {
                                    log::warn!("ZMK set_layer_name failed: {e}");
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
                let left_center  = egui::pos2(center_x - fixed_half - gap - 24.0, mid_y);
                let right_center = egui::pos2(center_x + fixed_half + gap + 24.0, mid_y);

                // Still measure actual text width for painting the name and edit icon.
                let text_w = ui.fonts(|f| f.layout_no_wrap(name.clone(), label_font.clone(), text_color).size().x);

                // Allocate name FIRST — arrows are allocated last and win in egui's
                // hit-test order (last allocation = highest priority).
                let name_hit = egui::Rect::from_center_size(egui::pos2(center_x, mid_y), Vec2::new(text_w + 12.0, 52.0));
                let name_r = ui.allocate_rect(name_hit, Sense::click());

                // Scroll wheel over the name area switches layers (down = next, up = prev)
                if name_r.hovered() {
                    let scroll = ui.input(|i| i.raw_scroll_delta.y);
                    if scroll < 0.0 && selected > 0 {
                        self.selected_layer = selected - 1;
                    } else if scroll > 0.0 && selected + 1 < layer_count {
                        self.selected_layer = selected + 1;
                    }
                }

                // Allocate arrows LAST so they have click priority over the name rect.
                let left_hit  = egui::Rect::from_center_size(left_center,  Vec2::splat(48.0));
                let right_hit = egui::Rect::from_center_size(right_center, Vec2::splat(48.0));
                let left_r  = ui.allocate_rect(left_hit,  Sense::click());
                let right_r = ui.allocate_rect(right_hit, Sense::click());
                if left_r.hovered()  { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
                if right_r.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }
                if left_r.clicked()  && selected > 0              { self.selected_layer = selected - 1; self.jump_back_stack.clear(); }
                if right_r.clicked() && selected + 1 < layer_count { self.selected_layer = selected + 1; self.jump_back_stack.clear(); }
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

                // Hint text below layer name
                let hint_color = if self.dark_mode { Color32::from_gray(100) } else { Color32::from_gray(160) };
                let hint_font = FontId::proportional(11.0);
                let hint_y = bar_y + layer_bar_h - 2.0;
                let any_hovered = self.prev_hovered_key.is_some();
                if let Some(hl) = self.hover_layer {
                    let hl_name = self.layer_names.get(hl).cloned().unwrap_or_else(|| hl.to_string());
                    let mut line = 0i32;
                    let line_h = 13.0f32;
                    let base_y = hint_y - 15.0;
                    // Line 1: always
                    ui.painter().text(egui::pos2(center_x, base_y + line as f32 * line_h), egui::Align2::CENTER_CENTER,
                        "Left click to change this key", hint_font.clone(), hint_color);
                    line += 1;
                    // Line 2: go to layer (if not current)
                    if hl != self.selected_layer {
                        ui.painter().text(egui::pos2(center_x, base_y + line as f32 * line_h), egui::Align2::CENTER_CENTER,
                            &format!("Right click to go to layer {}: {}", hl, hl_name), hint_font.clone(), hint_color);
                        line += 1;
                    }
                    // Line 3: change layer number
                    ui.painter().text(egui::pos2(center_x, base_y + line as f32 * line_h), egui::Align2::CENTER_CENTER,
                        "Ctrl + Right click to change layer number", hint_font.clone(), hint_color);
                    line += 1;
                    // Line 4: go back (if in jump mode)
                    if !self.jump_back_stack.is_empty() {
                        ui.painter().text(egui::pos2(center_x, base_y + line as f32 * line_h), egui::Align2::CENTER_CENTER,
                            "Esc to go back", hint_font.clone(), hint_color);
                    }
                    let _ = hint_font;
                } else if !self.jump_back_stack.is_empty() {
                    if any_hovered {
                        ui.painter().text(egui::pos2(center_x, hint_y - 9.0), egui::Align2::CENTER_CENTER,
                            "Left click to change this key", hint_font.clone(), hint_color);
                    }
                    ui.painter().text(egui::pos2(center_x, if any_hovered { hint_y + 5.0 } else { hint_y }), egui::Align2::CENTER_CENTER,
                        "Right-click or Esc to go back", hint_font, hint_color);
                } else if any_hovered {
                    // Check if hovered key is a mod key
                    let hovered_is_mod = if self.firmware == FirmwareProtocol::Vial {
                        self.prev_hovered_key.and_then(|ki| {
                            self.layout.as_ref().map(|l| {
                                let kc = l.get_keycode(self.selected_layer, ki);
                                (kc >= 0x2000 && kc < 0x4000) || (kc >= 0x0100 && kc < 0x2000 && (kc & 0xFF) != 0)
                            })
                        }).unwrap_or(false)
                    } else { false };
                    if hovered_is_mod {
                        ui.painter().text(egui::pos2(center_x, hint_y - 9.0), egui::Align2::CENTER_CENTER,
                            "Left click to change this key", hint_font.clone(), hint_color);
                        ui.painter().text(egui::pos2(center_x, hint_y + 5.0), egui::Align2::CENTER_CENTER,
                            "Right click to change the modifier key", hint_font, hint_color);
                    } else {
                        ui.painter().text(egui::pos2(center_x, hint_y), egui::Align2::CENTER_CENTER,
                            "Left click to change this key", hint_font, hint_color);
                    }
                }

                // Edit icon after text on hover
                if name_r.hovered() {
                    let icon_pos = egui::pos2(center_x + text_w / 2.0 + 6.0, mid_y);
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

        let is_zmk = self.firmware == FirmwareProtocol::Zmk;

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
                self.keycode_picker.search_query.clear();
                self.keycode_picker.layer_names = self.layer_names.clone();
                self.keycode_picker.firmware = self.firmware;
                self.keycode_picker.vial_quantum_pending_mod = None;
                self.keycode_picker.vial_quantum_pending_mt = None;
                if is_zmk {
                    self.keycode_picker.zmk_behaviors = self.layout.as_ref()
                        .map(|l| l.zmk_behaviors.clone()).unwrap_or_default();
                    self.keycode_picker.zmk_layer_count = self.layer_count;
                    self.keycode_picker.selected_tab = crate::keycode_picker::KeycodeTab::Basic;
                }
            }

            // Right-click on mod key — open second picker to change tap/key  
            if response.secondary_clicked() && !is_zmk {
                let kc = layout.get_keycode(self.selected_layer, *ki);
                let ctrl_held = ui.input(|i| i.modifiers.ctrl);

                // Ctrl+RClick on layer keys: change layer number
                if ctrl_held {
                    let layer_base: Option<u16> = if kc >= 0x5200 && kc < 0x5300 {
                        Some(kc & 0xFFE0)
                    } else if kc & 0xF000 == 0x4000 {
                        Some(0x4000)
                    } else {
                        None
                    };
                    if let Some(base) = layer_base {
                        self.selected_key = Some((self.selected_layer, *ki));
                        self.keycode_picker.open = true;
                        self.keycode_picker.layer_names = self.layer_names.clone();
                        self.keycode_picker.firmware = self.firmware;
                        self.keycode_picker.vial_layer_pending = Some(base);
                        self.secondary_click_handled = true;
                    }
                }
                // MT: 0x2000..0x3FFF, Mod+Key: 0x0100..0x1FFF with kc != 0
                let is_layer_key = (kc >= 0x5200 && kc < 0x5300) || (kc & 0xF000 == 0x4000);
                let pending_base: Option<u16> = if is_layer_key {
                    None // layer keys handled above (Ctrl+RClick)
                } else if kc >= 0x2000 && kc < 0x4000 {
                    Some(kc & 0xFF00) // MT base
                } else if kc >= 0x0100 && kc < 0x2000 && (kc & 0xFF) != 0 {
                    Some(kc & 0xFF00) // Mod+Key base
                } else {
                    None
                };
                if let Some(base) = pending_base {
                    self.selected_key = Some((self.selected_layer, *ki));
                    self.keycode_picker.open = true;
                    self.keycode_picker.layer_names = self.layer_names.clone();
                    self.keycode_picker.firmware = self.firmware;
                    if kc >= 0x2000 {
                        self.keycode_picker.vial_quantum_pending_mt = Some(base);
                        self.keycode_picker.vial_quantum_pending_mod = None;
                    } else {
                        self.keycode_picker.vial_quantum_pending_mod = Some(base);
                        self.keycode_picker.vial_quantum_pending_mt = None;
                    }
                    self.secondary_click_handled = true;
                }
            }

            // Tooltip — for layer keys show mini layout preview
            let preview_layer: Option<usize> = if is_zmk {
                let binding = layout.get_zmk_binding(self.selected_layer, *ki);
                let beh_name = layout.zmk_behaviors.iter()
                    .find(|b| b.id == binding.behavior_id as u32)
                    .map(|b| b.display_name.as_str())
                    .unwrap_or("");
                match beh_name {
                    "Momentary Layer" | "Toggle Layer" | "To Layer" | "Sticky Layer" | "Layer-Tap" => {
                        Some(binding.param1 as usize)
                    }
                    _ => None,
                }
            } else {
                let kc = layout.get_keycode(self.selected_layer, *ki);
                // MO/TG/TO/OSL/TT/DF range: 0x5200..0x52FF, LT: 0x4000..0x4FFF
                if kc >= 0x5200 && kc < 0x5300 {
                    Some((kc & 0x1F) as usize)
                } else if kc & 0xF000 == 0x4000 {
                    Some(((kc >> 8) & 0xF) as usize)
                } else {
                    None
                }
            };

            if let Some(preview_layer_idx) = preview_layer {
                if response.hovered() {
                    self.hover_layer = Some(preview_layer_idx);
                    hovered_key = Some(*ki); // keep hovered_key for layer keys too
                }
                if response.secondary_clicked() && preview_layer_idx != self.selected_layer {
                    // Right-click: jump to that layer
                    self.jump_back_stack.push(self.selected_layer);
                    self.selected_layer = preview_layer_idx;
                    self.hover_layer = None;
                    self.secondary_click_handled = true;
                }
            } else if is_zmk {
                let binding = layout.get_zmk_binding(self.selected_layer, *ki);
                let tip = zmk_binding_tooltip(&binding, &layout.zmk_behaviors, &self.layer_names);
                *response = response.clone().on_hover_text(tip);
            } else {
                let kc = layout.get_keycode(self.selected_layer, *ki);
                let is_mod_key = (kc >= 0x2000 && kc < 0x4000) || (kc >= 0x0100 && kc < 0x2000 && (kc & 0xFF) != 0);
                let tip = if is_mod_key {
                    let base_tip = keycode_tooltip(kc, &layout.custom_keycodes, &self.layer_names);
                    format!("{}\nRight-click to change the key", base_tip)
                } else {
                    keycode_tooltip(kc, &layout.custom_keycodes, &self.layer_names)
                };
                *response = response.clone().on_hover_text(tip);
            }
        }

        // Animate hover_layer_progress
        let target_progress = if self.hover_layer.is_some() { 1.0f32 } else { 0.0f32 };
        let speed = 4.0f32;
        let dt = ctx.input(|i| i.stable_dt).min(0.1);
        self.hover_layer_progress += (target_progress - self.hover_layer_progress) * (speed * dt).min(1.0);
        if (self.hover_layer_progress - target_progress).abs() > 0.01 {
            ctx.request_repaint();
        }

        // Pass 3: paint
        let painter = ui.painter();
        let hover_target = self.hover_layer.unwrap_or(prev_hover.unwrap_or(self.selected_layer));
        let hover_alpha = self.hover_layer_progress;
        let dark = self.dark_mode;
        // Use hover layer for logic (TRNS resolution etc) when mostly visible
        let layer = if hover_alpha > 0.5 { hover_target } else { self.selected_layer };
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

            let is_hovering = hover_alpha > 0.05;

            if is_zmk {
                // ZMK binding display
                let binding = layout.get_zmk_binding(layer, *ki);
                let is_trans = layout.zmk_behaviors.iter()
                    .find(|b| b.id == binding.behavior_id as u32)
                    .map(|b| b.display_name == "Transparent")
                    .unwrap_or(false);
                let border = if dark { Color32::from_rgb(55, 55, 60) } else { Color32::from_rgb(210, 210, 218) };
                painter.rect(draw_rect, 6.0, bg, Stroke::new(1.0, border), egui::StrokeKind::Inside);
                if is_trans && layer > 0 {
                    if is_hovering {
                        // During hover preview — TRNS keys are empty (no text)
                    } else {
                        // Normal display — show TRNS with fallback
                        let fallback = (0..layer).rev()
                            .map(|l| layout.get_zmk_binding(l, *ki))
                            .find(|b| {
                                !layout.zmk_behaviors.iter()
                                    .find(|beh| beh.id == b.behavior_id as u32)
                                    .map(|beh| beh.display_name == "Transparent")
                                    .unwrap_or(false)
                            });
                        let label = if let Some(fb) = fallback {
                            zmk_binding_label(&fb, &layout.zmk_behaviors, &self.layer_names)
                        } else {
                            "▽".to_string()
                        };
                        draw_key_label_dimmed(&painter, draw_rect, &label, dark);
                    }
                } else {
                    let label = zmk_binding_label(&binding, &layout.zmk_behaviors, &self.layer_names);
                    draw_key_label(&painter, draw_rect, &label, dark);
                }
            } else {
                let kc = layout.get_keycode(layer, *ki);

                if kc == 0x0001 {
                    painter.rect(draw_rect, 6.0, bg, Stroke::new(1.0, if dark { Color32::from_rgb(55, 55, 60) } else { Color32::from_rgb(210, 210, 218) }), egui::StrokeKind::Inside);
                    if !is_hovering {
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
                    }
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


        }

        self.prev_hovered_key = hovered_key;

        if layout_h > avail.y {
            ui.allocate_space(Vec2::new(0.0, (layout_h - avail.y).max(0.0)));
        }
    }
}

fn draw_key_label_dimmed(painter: &egui::Painter, rect: egui::Rect, label: &str, dark: bool) {
    let dim = if dark { Color32::from_rgb(60, 60, 65) } else { Color32::from_rgb(200, 200, 208) };
    let dim_top = if dark { Color32::from_rgb(45, 45, 50) } else { Color32::from_rgb(215, 215, 220) };
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

fn with_alpha(color: Color32, alpha: f32) -> Color32 {
    let a = (color.a() as f32 * alpha.clamp(0.0, 1.0)) as u8;
    Color32::from_rgba_premultiplied(
        (color.r() as f32 * alpha.clamp(0.0, 1.0)) as u8,
        (color.g() as f32 * alpha.clamp(0.0, 1.0)) as u8,
        (color.b() as f32 * alpha.clamp(0.0, 1.0)) as u8,
        a,
    )
}

fn draw_key_label_alpha(painter: &egui::Painter, rect: egui::Rect, label: &str, dark: bool, alpha: f32) {
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
    let top_color = with_alpha(if dark { Color32::from_rgb(130, 130, 145) } else { Color32::from_rgb(130, 130, 150) }, alpha);
    let main_color = with_alpha(if dark { Color32::from_rgb(232, 232, 240) } else { Color32::from_rgb(26, 26, 30) }, alpha);
    if let Some(top_str) = top {
        let center = rect.center();
        painter.text(egui::pos2(center.x, center.y - 7.0), egui::Align2::CENTER_CENTER, top_str, FontId::proportional(9.0), top_color);
        painter.text(egui::pos2(center.x, center.y + 6.0), egui::Align2::CENTER_CENTER, bottom, FontId::proportional(11.0), main_color);
    } else {
        painter.text(rect.center(), egui::Align2::CENTER_CENTER, bottom, FontId::proportional(11.0), main_color);
    }
}

fn draw_key_label_dimmed_alpha(painter: &egui::Painter, rect: egui::Rect, label: &str, dark: bool, alpha: f32) {
    let color = with_alpha(if dark { Color32::from_rgb(80, 80, 90) } else { Color32::from_rgb(180, 180, 195) }, alpha);
    painter.text(rect.center(), egui::Align2::CENTER_CENTER, label, FontId::proportional(11.0), color);
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
