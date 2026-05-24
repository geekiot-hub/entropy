use super::*;

impl EntropyApp {
    pub(super) fn start_connect(&mut self, device_idx: usize) {
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
        self.selected_encoder = None;
        self.selected_layer = 0;
        self.hid_device = None;
        self.qmk_hid_hosts.clear();
        self.combo_visible_count = 1;
        self.combo_capture_open = false;
        self.combo_capture_keys.clear();
        self.combo_undo_stack.clear();
        self.combo_pick_target = None;
        self.combo_dirty = false;
        self.combo_names_dirty = false;
        self.combo_term_dirty = false;
        self.auto_shift_options = AutoShiftOptionsState::default();
        self.auto_shift_timeout = None;
        self.auto_shift_timeout_text.clear();
        self.mouse_keys_settings = MouseKeysSettingsState::default();
        self.touchpad_settings = TouchpadSettingsState::default();
        self.tap_hold_settings = TapHoldSettingsState::default();
        self.magic_settings = MagicSettingsState::default();
        self.one_shot_settings = OneShotSettingsState::default();
        self.layer_led_settings = LayerLedSettingsState::default();
        self.alt_repeat_entries.clear();
        self.alt_repeat_names.clear();
        self.alt_repeat_undo_stack.clear();
        self.selected_alt_repeat = 0;
        self.alt_repeat_visible_count = 1;
        self.alt_repeat_pick_target = None;
        self.rgb_settings = RgbSettingsState::default();
        self.layout_options_value = None;
        self.encoder_visibility.clear();
        self.key_override_entries.clear();
        self.key_override_names.clear();
        self.key_override_visible_count = 1;
        self.key_override_undo_stack.clear();
        self.selected_key_override = 0;
        self.key_override_pick_target = None;
        self.reset_matrix_tester_state();

        let (tx, rx) = mpsc::channel();
        self.connect_state = ConnectState::Loading {
            rx,
            started_at: std::time::Instant::now(),
        };

        std::thread::spawn(move || {
            let progress = |message: &str| {
                let _ = tx.send(ConnectTaskMessage::Progress(message.to_owned()));
            };
            let result = (|| -> Result<ConnectResult, String> {
                use crate::hid::HidDevice;

                progress("Opening HID device…");
                log::info!(
                    "Opening HID device: {} {:04X}:{:04X}",
                    dev.name,
                    dev.vendor_id,
                    dev.product_id
                );
                let dev_conn =
                    HidDevice::open_fresh_for(&dev).map_err(|e| format!("Open failed: {e}"))?;

                progress("Reading VIA protocol version…");
                log::info!("Getting protocol version…");
                match dev_conn.get_protocol_version() {
                    Ok(v) => log::info!("VIA protocol version: {v}"),
                    Err(e) => log::warn!("get_protocol_version failed: {e}"),
                }

                progress("Reading Vial keyboard id…");
                match dev_conn.get_keyboard_id() {
                    Ok((vial_protocol, keyboard_id)) => {
                        log::info!(
                            "Vial protocol: {vial_protocol}, keyboard id: {keyboard_id:016X}"
                        );
                    }
                    Err(e) => log::warn!("get_keyboard_id failed: {e}"),
                }

                progress("Reading Vial layout definition…");
                log::info!("Getting layout JSON…");
                let json = dev_conn
                    .get_layout_json()
                    .map_err(|e| format!("Layout read failed: {e}"))?;

                progress("Parsing keyboard layout…");
                let mut layout = KeyboardLayout::from_vial_json(&json)
                    .map_err(|e| format!("Layout parse failed: {e}"))?;

                log::info!("Looking up embedded layout for '{}'", dev.name);
                if let Some((embedded, reference_keys)) = crate::layouts::lookup_layout(&dev.name) {
                    log::info!(
                        "Found embedded layout '{}' with {} keys",
                        embedded.name,
                        reference_keys.len()
                    );
                    use std::collections::HashMap;
                    let reference_by_matrix: HashMap<(u8, u8), &crate::keyboard::PhysicalKey> =
                        reference_keys
                            .iter()
                            .map(|key| ((key.row, key.col), key))
                            .collect();
                    let mut patched = 0usize;
                    for key in &mut layout.keys {
                        if let Some(reference_key) = reference_by_matrix.get(&(key.row, key.col)) {
                            key.x = reference_key.x;
                            key.y = reference_key.y;
                            key.rotation = reference_key.rotation;
                            key.rotation_x = reference_key.rotation_x;
                            key.rotation_y = reference_key.rotation_y;
                            patched += 1;
                        }
                    }
                    log::info!("Patched {} key coordinates from embedded layout", patched);
                }

                progress("Reading layer count…");
                log::info!("Getting layer count…");
                let layer_count = dev_conn
                    .get_layer_count()
                    .map(|c| c as usize)
                    .unwrap_or_else(|e| {
                        log::warn!("get_layer_count failed: {e}, defaulting to 4");
                        4
                    });
                log::info!("Layer count: {layer_count}");

                let num_keys = layout.keys.len();
                layout.layers = vec![vec![0u16; num_keys]; layer_count];

                progress("Reading keymap…");
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

                progress("Reading Vial-core extras…");
                if !layout.encoders.is_empty() {
                    layout.encoder_layers = vec![vec![0u16; layout.encoders.len()]; layer_count];
                    let encoder_count = layout.encoder_count();
                    for layer in 0..layer_count {
                        let mut per_encoder = vec![(0u16, 0u16); encoder_count];
                        for encoder_idx in 0..encoder_count {
                            match dev_conn.get_encoder(layer as u8, encoder_idx as u8) {
                                Ok((ccw, cw)) => per_encoder[encoder_idx] = (ccw, cw),
                                Err(e) => log::warn!(
                                    "get_encoder(layer={}, idx={}): {}",
                                    layer,
                                    encoder_idx,
                                    e
                                ),
                            }
                        }
                        for (visual_idx, encoder) in layout.encoders.iter().enumerate() {
                            if let Some((ccw, cw)) = per_encoder.get(encoder.encoder_idx as usize) {
                                layout.encoder_layers[layer][visual_idx] =
                                    if encoder.direction == 0 { *ccw } else { *cw };
                            }
                        }
                    }
                }

                let layout_options_value = if layout.layout_options.is_empty() {
                    None
                } else {
                    match dev_conn.get_layout_options() {
                        Ok(value) => Some(value),
                        Err(e) => {
                            log::warn!("get_layout_options: {e}");
                            None
                        }
                    }
                };

                let macro_texts = match dev_conn.get_macro_count() {
                    Ok(count) => {
                        log::info!("Macro count: {count}");
                        match dev_conn.get_macro_buffer_size() {
                            Ok(size) => {
                                log::info!("Macro buffer size: {size}");
                                match dev_conn.get_macro_buffer(size) {
                                    Ok(buf) => crate::hid::HidDevice::parse_macros(&buf, count),
                                    Err(e) => {
                                        log::warn!("get_macro_buffer: {e}");
                                        vec![String::new(); count as usize]
                                    }
                                }
                            }
                            Err(e) => {
                                log::warn!("get_macro_buffer_size: {e}");
                                vec![String::new(); count as usize]
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("get_macro_count: {e}");
                        vec![]
                    }
                };

                let tap_dance_entries = Vec::new();
                let combo_entries = Vec::new();
                let combo_term = None;
                let auto_shift_options = None;
                let auto_shift_timeout = None;
                let mouse_keys_settings = MouseKeysSettingsState::default();
                let touchpad_settings = TouchpadSettingsState::default();
                let module_settings = ModuleSettingsState::default();
                let tap_hold_settings = TapHoldSettingsState::default();
                let magic_settings = MagicSettingsState::default();
                let one_shot_settings = OneShotSettingsState::default();
                let grave_escape_settings = GraveEscapeSettingsState::default();
                let layer_led_settings = LayerLedSettingsState::default();
                let rgb_settings = RgbSettingsState::default();
                let key_override_entries = Vec::new();
                let alt_repeat_entries = Vec::new();
                let vial_features = VialFeatureSupport::default();

                progress("Applying keyboard layout…");
                Ok(ConnectResult {
                    device_name: dev.name.clone(),
                    macro_texts,
                    tap_dance_entries,
                    combo_entries,
                    combo_term,
                    auto_shift_options: auto_shift_options.unwrap_or_default(),
                    auto_shift_timeout,
                    mouse_keys_settings,
                    touchpad_settings,
                    module_settings,
                    tap_hold_settings,
                    magic_settings,
                    one_shot_settings,
                    grave_escape_settings,
                    layer_led_settings,
                    rgb_settings,
                    layout_options_value,
                    key_override_entries,
                    alt_repeat_entries,
                    vial_features,
                    layout,
                    layer_count,
                })
            })();

            let _ = tx.send(ConnectTaskMessage::Done(result));
        });
    }
}
