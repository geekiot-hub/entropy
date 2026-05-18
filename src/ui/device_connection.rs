use super::*;

impl EntropyApp {
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn clear_connected_keyboard_state(&mut self, status_msg: impl Into<String>) {
        self.layout = None;
        self.selected_key = None;
        self.selected_encoder = None;
        self.selected_layer = 0;
        self.layer_count = 0;
        self.qmk_hid_hosts.clear();
        self.hid_device = None;
        self.connect_state = ConnectState::Idle;
        self.unlock_open = false;
        self.vial_unlock_polling = false;
        self.vial_unlock_last_poll = None;
        self.pending_layout_indicator_open_after_unlock = false;
        self.keycode_picker.open = false;
        self.current_device_name.clear();
        self.mouse_keys_settings = MouseKeysSettingsState::default();
        self.touchpad_settings = TouchpadSettingsState::default();
        self.module_settings = ModuleSettingsState::default();
        self.tap_hold_settings = TapHoldSettingsState::default();
        self.magic_settings = MagicSettingsState::default();
        self.one_shot_settings = OneShotSettingsState::default();
        self.grave_escape_settings = GraveEscapeSettingsState::default();
        self.layer_led_settings = LayerLedSettingsState::default();
        self.rgb_settings = RgbSettingsState::default();
        self.layout_options_value = None;
        self.sticky_layout_prev_pressed.clear();
        self.sticky_layout_pressed_key_layers.clear();
        self.sticky_layout_toggled_layers.clear();
        self.sticky_layout_base_layer = 0;
        self.status_msg = status_msg.into();
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn start_device_scan(&mut self) {
        if !matches!(self.device_scan_state, DeviceScanState::Idle) {
            return;
        }

        let (tx, rx) = mpsc::channel();
        self.device_scan_state = DeviceScanState::Scanning(rx);
        std::thread::spawn(move || {
            let _ = tx.send(DeviceManager::scan_devices());
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn poll_device_scan(&mut self, ctx: &egui::Context) {
        let devices = match &self.device_scan_state {
            DeviceScanState::Idle => return,
            DeviceScanState::Scanning(rx) => match rx.try_recv() {
                Ok(devices) => Some(devices),
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(25));
                    return;
                }
                Err(mpsc::TryRecvError::Disconnected) => Some(Vec::new()),
            },
        };

        self.device_scan_state = DeviceScanState::Idle;
        if let Some(devices) = devices {
            self.apply_device_scan_result(devices);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn apply_device_scan_result(&mut self, devices: Vec<Device>) {
        let previous_path = self
            .selected_device
            .and_then(|idx| self.device_manager.devices().get(idx))
            .map(|dev| dev.path.clone());
        let was_loading = matches!(self.connect_state, ConnectState::Loading(_));

        self.device_manager.replace_devices(devices);

        if self.device_manager.devices().is_empty() {
            if self.selected_device.is_some() || self.layout.is_some() || was_loading {
                self.selected_device = None;
                self.clear_connected_keyboard_state("No keyboard detected");
            } else {
                self.qmk_hid_hosts.clear();
            }
            return;
        }

        if let Some(path) = previous_path {
            if let Some(idx) = self
                .device_manager
                .devices()
                .iter()
                .position(|dev| dev.path == path)
            {
                self.selected_device = Some(idx);
                if self.layout.is_none() && !was_loading {
                    self.start_connect(idx);
                } else {
                    self.sync_qmk_hid_host_bridges();
                }
                return;
            }
        }

        self.selected_device = Some(0);
        self.start_connect(0);
    }

    /// Spawn background thread to connect + load layout/keycodes.
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
        self.sync_qmk_hid_host_bridges();
        self.hid_device = None;
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
        self.connect_state = ConnectState::Loading(rx);

        std::thread::spawn(move || {
            let result = (|| -> Result<ConnectResult, String> {
                use crate::hid::HidDevice;

                log::info!("Opening HID device: {}", dev.path);
                let dev_conn =
                    HidDevice::open(&dev.path).map_err(|e| format!("Open failed: {e}"))?;

                log::info!("Getting protocol version…");
                match dev_conn.get_protocol_version() {
                    Ok(v) => log::info!("VIA protocol version: {v}"),
                    Err(e) => log::warn!("get_protocol_version failed: {e}"),
                }

                log::info!("Getting layer count…");
                let layer_count = dev_conn
                    .get_layer_count()
                    .map(|c| c as usize)
                    .unwrap_or_else(|e| {
                        log::warn!("get_layer_count failed: {e}, defaulting to 4");
                        4
                    });
                log::info!("Layer count: {layer_count}");

                log::info!("Getting layout JSON…");
                let json = dev_conn
                    .get_layout_json()
                    .map_err(|e| format!("Layout read failed: {e}"))?;

                let touchpad_settings_in_definition =
                    Self::layout_json_has_touchpad_settings(&json);
                let supported_qmk_settings = dev_conn.query_qmk_settings().unwrap_or_else(|e| {
                    log::warn!("qmk settings query failed: {e}");
                    Vec::new()
                });

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

                let mut firmware_layer_names = Vec::new();
                for layer in 0..layer_count.min(16) {
                    match dev_conn.get_qmk_setting_string(200 + layer as u16) {
                        Ok(name) if !name.is_empty() => firmware_layer_names.push(name),
                        Ok(_) => firmware_layer_names.push(layer.to_string()),
                        Err(_) => {
                            firmware_layer_names.clear();
                            break;
                        }
                    }
                }
                if !firmware_layer_names.is_empty() {
                    layout.layer_names = firmware_layer_names;
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

                let (
                    tap_dance_count,
                    combo_count,
                    key_override_count,
                    alt_repeat_count,
                    dynamic_feature_bits,
                ) = match dev_conn.get_dynamic_entry_counts() {
                    Ok(counts) => counts,
                    Err(e) => {
                        log::warn!("get_dynamic_entry_counts: {e}");
                        (0, 0, 0, 0, 0)
                    }
                };
                let vial_features = VialFeatureSupport {
                    caps_word: dynamic_feature_bits & (1 << 0) != 0,
                    layer_lock: dynamic_feature_bits & (1 << 1) != 0,
                    persistent_default_layer: key_override_count > 0,
                    repeat_key: alt_repeat_count > 0,
                };

                let combo_entries = {
                    let count = combo_count;
                    log::info!("Combo count: {count}");
                    let mut entries = Vec::new();
                    for i in 0..count {
                        match dev_conn.get_combo(i) {
                            Ok((keys, output)) => entries.push(ComboEntry { keys, output }),
                            Err(e) => {
                                log::warn!("get_combo({i}): {e}");
                                entries.push(Default::default());
                            }
                        }
                    }
                    entries
                };

                let combo_term = match dev_conn.get_qmk_setting_u16(2) {
                    Ok(value) => Some(value),
                    Err(e) => {
                        log::warn!("get_qmk_setting_u16(combo_term): {e}");
                        None
                    }
                };
                let auto_shift_options = match dev_conn.get_qmk_setting_u8(3) {
                    Ok(value) => Some(AutoShiftOptionsState::from_bits(value)),
                    Err(e) => {
                        log::warn!("get_qmk_setting_u8(auto_shift_flags): {e}");
                        None
                    }
                };
                let auto_shift_timeout = match dev_conn.get_qmk_setting_u16(4) {
                    Ok(value) => Some(value),
                    Err(e) => {
                        log::warn!("get_qmk_setting_u16(auto_shift_timeout): {e}");
                        None
                    }
                };

                // Mouse keys settings (qsid 9..=17, all u16). If qsid 9 is unsupported,
                // we assume the whole group is unavailable.
                let mouse_keys_settings = {
                    let mut mk = MouseKeysSettingsState::default();
                    match dev_conn.get_qmk_setting_u8(9) {
                        Ok(v) => {
                            mk.delay = v as u16;
                            mk.supported = true;
                            let read = |qsid: u16| -> u16 {
                                match dev_conn.get_qmk_setting_u8(qsid) {
                                    Ok(val) => val as u16,
                                    Err(e) => {
                                        log::warn!(
                                            "get_qmk_setting_u8(mouse_keys qsid {qsid}): {e}"
                                        );
                                        0
                                    }
                                }
                            };
                            mk.interval = read(10);
                            mk.move_delta = read(11);
                            mk.max_speed = read(12);
                            mk.time_to_max = read(13);
                            mk.wheel_delay = read(14);
                            mk.wheel_interval = read(15);
                            mk.wheel_max_speed = read(16);
                            mk.wheel_time_to_max = read(17);
                        }
                        Err(e) => {
                            log::warn!("get_qmk_setting_u16(mouse_keys delay): {e}");
                        }
                    }
                    mk
                };

                // Ergohaven K:03 Pro touchpad settings (qsid 120..=124). These qsids
                // overlap with other Ergohaven pointing devices, so expose the page only
                // for known K:03 Pro identities.
                let touchpad_settings = {
                    let mut tp = TouchpadSettingsState::default();
                    if touchpad_settings_in_definition
                        && [120u16, 121, 122, 123, 124]
                            .iter()
                            .all(|qsid| supported_qmk_settings.contains(qsid))
                    {
                        tp.dpi_variants = Self::touchpad_setting_variants(&json, 120);
                        let dpi_read = if tp.dpi_variants.is_empty() {
                            dev_conn.get_qmk_setting_u16(120)
                        } else {
                            dev_conn.get_qmk_setting_u8(120).map(|value| value as u16)
                        };
                        match dpi_read {
                            Ok(v) => {
                                tp.dpi = v;
                                tp.supported = true;
                                tp.sniper_sens =
                                    dev_conn.get_qmk_setting_u8(121).unwrap_or_else(|e| {
                                        log::warn!("get_qmk_setting_u8(touchpad sniper sens): {e}");
                                        0
                                    });
                                tp.scroll_sens =
                                    dev_conn.get_qmk_setting_u8(122).unwrap_or_else(|e| {
                                        log::warn!("get_qmk_setting_u8(touchpad scroll sens): {e}");
                                        0
                                    });
                                tp.text_sens =
                                    dev_conn.get_qmk_setting_u8(123).unwrap_or_else(|e| {
                                        log::warn!("get_qmk_setting_u8(touchpad text sens): {e}");
                                        0
                                    });
                                tp.bits = dev_conn.get_qmk_setting_u8(124).unwrap_or_else(|e| {
                                    log::warn!("get_qmk_setting_u8(touchpad bits): {e}");
                                    0
                                });
                                if supported_qmk_settings.contains(&142)
                                    && Self::touchpad_setting_exists(&json, 142)
                                {
                                    tp.auto_layer_enable_supported = true;
                                    tp.auto_layer_enable = dev_conn
                                                .get_qmk_setting_u8(142)
                                                .map(|value| value != 0)
                                                .unwrap_or_else(|e| {
                                                    log::warn!(
                                                        "get_qmk_setting_u8(touchpad auto layer enable): {e}"
                                                    );
                                                    false
                                                });
                                }
                                if supported_qmk_settings.contains(&143)
                                    && Self::touchpad_setting_exists(&json, 143)
                                {
                                    tp.auto_layer_variants =
                                        Self::touchpad_setting_variants(&json, 143);
                                    tp.auto_layer =
                                        dev_conn.get_qmk_setting_u8(143).unwrap_or_else(|e| {
                                            log::warn!(
                                                "get_qmk_setting_u8(touchpad auto layer): {e}"
                                            );
                                            0
                                        });
                                }
                            }
                            Err(e) => {
                                log::warn!("get_qmk_setting(touchpad dpi): {e}");
                            }
                        }
                    }
                    tp
                };

                let module_settings =
                    Self::read_module_settings(&json, &supported_qmk_settings, &dev_conn);

                // Tap-Hold settings. If qsid 7 is unsupported, we treat the page as unavailable.
                let tap_hold_settings = {
                    let mut th = TapHoldSettingsState::default();
                    match dev_conn.get_qmk_setting_u16(7) {
                        Ok(v) => {
                            th.tapping_term = v;
                            th.supported = true;
                            let read_bool = |qsid: u16| -> bool {
                                match dev_conn.get_qmk_setting_u8(qsid) {
                                    Ok(val) => val != 0,
                                    Err(e) => {
                                        log::warn!("get_qmk_setting_u8(tap_hold qsid {qsid}): {e}");
                                        false
                                    }
                                }
                            };
                            let read_u16 = |qsid: u16| -> u16 {
                                match dev_conn.get_qmk_setting_u16(qsid) {
                                    Ok(val) => val,
                                    Err(e) => {
                                        log::warn!(
                                            "get_qmk_setting_u16(tap_hold qsid {qsid}): {e}"
                                        );
                                        0
                                    }
                                }
                            };
                            th.permissive_hold = read_bool(22);
                            th.hold_on_other_key_press = read_bool(23);
                            th.retro_tapping = read_bool(24);
                            th.quick_tap_term = read_u16(25);
                            th.tap_code_delay = read_u16(18);
                            th.tap_hold_caps_delay = read_u16(19);
                            th.tapping_toggle = dev_conn
                                .get_qmk_setting_u8(20)
                                .map(|value| value as u16)
                                .unwrap_or_else(|e| {
                                    log::warn!("get_qmk_setting_u8(tap_hold qsid 20): {e}");
                                    0
                                });
                            th.chordal_hold = read_bool(26);
                            th.flow_tap = read_u16(27);
                        }
                        Err(e) => {
                            log::warn!("get_qmk_setting_u16(tap_hold tapping_term): {e}");
                        }
                    }
                    th
                };

                // Magic settings (qsid 21 bits 0..=9). These are global QMK runtime swaps/options.
                let magic_settings = {
                    match dev_conn.get_qmk_setting_u16(21) {
                        Ok(bits) => MagicSettingsState {
                            bits,
                            supported: true,
                        },
                        Err(e) => {
                            log::warn!("get_qmk_setting_u16(magic qsid 21): {e}");
                            MagicSettingsState::default()
                        }
                    }
                };

                // One Shot Keys settings (qsid 5..=6). These affect OSM(...) and OSL(...).
                let one_shot_settings = {
                    let mut os = OneShotSettingsState::default();
                    match dev_conn.get_qmk_setting_u8(5) {
                        Ok(v) => {
                            os.tap_toggle = v;
                            os.supported = true;
                            os.timeout = dev_conn.get_qmk_setting_u16(6).unwrap_or_else(|e| {
                                log::warn!("get_qmk_setting_u16(one_shot timeout qsid 6): {e}");
                                0
                            });
                        }
                        Err(e) => {
                            log::warn!("get_qmk_setting_u8(one_shot tap toggle qsid 5): {e}");
                        }
                    }
                    os
                };

                // Grave Escape settings (qsid 1 bits 0..=3). These affect KC_GESC,
                // not the physical Escape key.
                let grave_escape_settings = {
                    match dev_conn.get_qmk_setting_u8(1) {
                        Ok(bits) => GraveEscapeSettingsState {
                            bits,
                            supported: true,
                        },
                        Err(e) => {
                            log::warn!("get_qmk_setting_u8(grave_escape qsid 1): {e}");
                            GraveEscapeSettingsState::default()
                        }
                    }
                };

                // Ergohaven per-layer LED settings (qsid 300..=317). If qsid 300 is
                // unsupported, we assume the whole group is unavailable.
                let layer_led_settings = {
                    let mut leds = LayerLedSettingsState::default();
                    match dev_conn.get_qmk_setting_u8(300) {
                        Ok(v) => {
                            leds.layer_colors[0] = v;
                            leds.supported = true;
                            for layer in 1..16 {
                                let qsid = 300 + layer as u16;
                                leds.layer_colors[layer] =
                                    dev_conn.get_qmk_setting_u8(qsid).unwrap_or_else(|e| {
                                        log::warn!(
                                            "get_qmk_setting_u8(layer_led qsid {qsid}): {e}"
                                        );
                                        0
                                    });
                            }
                            leds.brightness = dev_conn
                                .get_qmk_setting_u16(316)
                                .unwrap_or_else(|e| {
                                    log::warn!("get_qmk_setting_u16(layer_led brightness): {e}");
                                    0
                                })
                                .min(255);
                            leds.timeout_mins =
                                dev_conn.get_qmk_setting_u8(317).unwrap_or_else(|e| {
                                    log::warn!("get_qmk_setting_u8(layer_led timeout): {e}");
                                    0
                                });
                        }
                        Err(e) => {
                            log::warn!("get_qmk_setting_u8(layer_led layer 0 color): {e}");
                        }
                    }
                    leds
                };

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

                let rgb_settings = if layer_led_settings.supported && layout.lighting_mode.is_none()
                {
                    // hpd3-style Ergohaven boards use QMK RGBLight internally only as a
                    // transport for per-layer LEDs. If the Vial definition does not
                    // explicitly advertise a standard lighting backend, expose Layer LEDs
                    // instead of the generic RGB page.
                    RgbSettingsState::default()
                } else {
                    load_rgb_settings(&dev_conn, &layout)
                };

                // Read tap dance entries
                let tap_dance_entries = {
                    let count = tap_dance_count;
                    log::info!("Tap dance count: {count}");
                    let mut entries = Vec::new();
                    for i in 0..count {
                        match dev_conn.get_tap_dance(i) {
                            Ok((tap, hold, dtap, taphold, term)) => {
                                entries.push(crate::keycode_picker::TapDanceEntry {
                                    on_tap: tap,
                                    on_hold: hold,
                                    on_double_tap: dtap,
                                    on_tap_hold: taphold,
                                    tapping_term: term,
                                });
                            }
                            Err(e) => {
                                log::warn!("get_tap_dance({i}): {e}");
                                entries.push(Default::default());
                            }
                        }
                    }
                    entries
                };
                let key_override_entries = {
                    let count = key_override_count;
                    log::info!("Key Override count: {count}");
                    let mut entries = Vec::new();
                    for i in 0..count {
                        match dev_conn.get_key_override(i) {
                            Ok((
                                trigger,
                                replacement,
                                layers,
                                trigger_mods,
                                negative_mod_mask,
                                suppressed_mods,
                                options,
                            )) => {
                                entries.push(KeyOverrideEntry {
                                    trigger,
                                    replacement,
                                    layers,
                                    trigger_mods,
                                    negative_mod_mask,
                                    suppressed_mods,
                                    options: KeyOverrideOptionsState::from_bits(options),
                                });
                            }
                            Err(e) => {
                                log::warn!("get_key_override({i}): {e}");
                                entries.push(Default::default());
                            }
                        }
                    }
                    entries
                };

                let alt_repeat_entries = {
                    let count = alt_repeat_count;
                    log::info!("Alt Repeat count: {count}");
                    let mut entries = Vec::new();
                    for i in 0..count {
                        match dev_conn.get_alt_repeat_key(i) {
                            Ok((keycode, alt_keycode, allowed_mods, options)) => {
                                entries.push(AltRepeatKeyEntry {
                                    keycode,
                                    alt_keycode,
                                    allowed_mods,
                                    options: AltRepeatKeyOptionsState::from_bits(options),
                                });
                            }
                            Err(e) => {
                                log::warn!("get_alt_repeat_key({i}): {e}");
                                entries.push(Default::default());
                            }
                        }
                    }
                    entries
                };

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

            let _ = tx.send(result);
        });
    }

    /// Poll background thread for connect result.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn poll_connect(&mut self, ctx: &egui::Context) {
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
                self.current_device_name = r.device_name.clone();
                if let Some(dev) = self
                    .selected_device
                    .and_then(|idx| self.device_manager.devices().get(idx))
                {
                    self.device_display_names
                        .insert(dev.path.clone(), r.device_name.clone());
                }
                self.keycode_picker.tap_dance_entries = r.tap_dance_entries.clone();
                self.combo_entries = r.combo_entries.clone();
                self.key_override_entries = r.key_override_entries.clone();
                self.alt_repeat_entries = r.alt_repeat_entries.clone();
                self.alt_repeat_names = load_alt_repeat_names(&self.current_device_name);
                self.alt_repeat_names
                    .resize(self.alt_repeat_entries.len(), String::new());
                self.alt_repeat_undo_stack.clear();
                self.selected_alt_repeat = 0;
                self.alt_repeat_visible_count = if self.alt_repeat_entries.is_empty() {
                    1
                } else {
                    1.min(self.alt_repeat_entries.len())
                };
                self.key_override_names = load_key_override_names(&self.current_device_name);
                self.key_override_names
                    .resize(self.key_override_entries.len(), String::new());
                self.key_override_visible_count = 1;
                self.key_override_undo_stack.clear();
                self.selected_key_override = 0;
                self.combo_names = load_combo_names(&self.current_device_name);
                self.combo_names
                    .resize(self.combo_entries.len(), String::new());
                self.combo_term = r.combo_term.or(Some(50));
                self.auto_shift_options = r.auto_shift_options;
                self.auto_shift_timeout = r.auto_shift_timeout;
                self.auto_shift_timeout_text = r
                    .auto_shift_timeout
                    .map(|timeout| timeout.to_string())
                    .unwrap_or_default();
                self.mouse_keys_settings = r.mouse_keys_settings;
                self.touchpad_settings = r.touchpad_settings;
                self.module_settings = r.module_settings;
                self.tap_hold_settings = r.tap_hold_settings;
                self.magic_settings = r.magic_settings;
                self.one_shot_settings = r.one_shot_settings;
                self.grave_escape_settings = r.grave_escape_settings;
                self.layer_led_settings = r.layer_led_settings;
                self.rgb_settings = r.rgb_settings;
                self.layout_options_value = r.layout_options_value;
                let highest_used_combo = self
                    .combo_entries
                    .iter()
                    .enumerate()
                    .filter(|(i, combo)| {
                        combo.output != 0
                            || combo.keys.iter().any(|&k| k != 0)
                            || self
                                .combo_names
                                .get(*i)
                                .map(|n| !n.trim().is_empty())
                                .unwrap_or(false)
                    })
                    .map(|(i, _)| i + 1)
                    .max()
                    .unwrap_or(1);
                self.combo_visible_count = highest_used_combo.min(self.combo_entries.len().max(1));
                self.selected_combo = self
                    .selected_combo
                    .min(self.combo_visible_count.saturating_sub(1));
                if !r.macro_texts.is_empty() {
                    self.keycode_picker.macro_count = r.macro_texts.len();
                    self.keycode_picker.macro_texts = r.macro_texts.clone();
                    self.keycode_picker.macro_names = vec![String::new(); r.macro_texts.len()];
                    // Parse macro texts into actions
                    // Parse macro texts → actions (Vial protocol v2: prefix 0x01 before actions)
                    self.keycode_picker.macro_actions = r
                        .macro_texts
                        .iter()
                        .map(|text| {
                            let bytes = text.as_bytes();
                            let mut actions = Vec::new();
                            let mut i = 0;
                            while i < bytes.len() {
                                if bytes[i] == 1 && i + 1 < bytes.len() {
                                    // SS_QMK_PREFIX
                                    match bytes[i + 1] {
                                        1 if i + 2 < bytes.len() => {
                                            // SS_TAP
                                            actions.push(crate::keycode_picker::MacroAction::Tap(
                                                bytes[i + 2],
                                            ));
                                            i += 3;
                                        }
                                        2 if i + 2 < bytes.len() => {
                                            // SS_DOWN
                                            actions.push(crate::keycode_picker::MacroAction::Down(
                                                bytes[i + 2],
                                            ));
                                            i += 3;
                                        }
                                        3 if i + 2 < bytes.len() => {
                                            // SS_UP
                                            actions.push(crate::keycode_picker::MacroAction::Up(
                                                bytes[i + 2],
                                            ));
                                            i += 3;
                                        }
                                        4 if i + 3 < bytes.len() => {
                                            // SS_DELAY
                                            let ms = (bytes[i + 2] as u16 - 1)
                                                + (bytes[i + 3] as u16 - 1) * 255;
                                            actions.push(
                                                crate::keycode_picker::MacroAction::Delay(ms),
                                            );
                                            i += 4;
                                        }
                                        _ => {
                                            i += 2;
                                        } // skip unknown
                                    }
                                } else {
                                    // Text character
                                    let start = i;
                                    while i < bytes.len() && bytes[i] != 1 {
                                        i += 1;
                                    }
                                    if let Ok(s) = std::str::from_utf8(&bytes[start..i]) {
                                        actions.push(crate::keycode_picker::MacroAction::Text(
                                            s.to_string(),
                                        ));
                                    }
                                }
                            }
                            actions
                        })
                        .collect();
                }

                self.status_msg = format!("Connected: {}", r.device_name);

                // Load per-device layer names
                let device_name = r.device_name.clone();
                // Prefer names from descriptor/firmware, then overlay local overrides only if a real saved file exists
                let mut layer_names = r.layout.layer_names.clone();
                if let Some(local_layer_names) = load_saved_layer_names(&device_name) {
                    layer_names = local_layer_names;
                }
                if layer_names.is_empty() {
                    layer_names = load_layer_names(&device_name);
                }
                self.layer_names = layer_names;

                let encoder_count = r.layout.encoder_count();
                self.encoder_visibility = load_encoder_visibility(&device_name, encoder_count);

                // Populate picker
                self.keycode_picker.supports_rgb =
                    r.layout.supports_rgb || self.rgb_settings.supported;
                self.keycode_picker.supports_macro = !r.macro_texts.is_empty();
                self.keycode_picker.supports_tap_dance = !r.tap_dance_entries.is_empty();
                self.keycode_picker.supports_mouse_keys = self.mouse_keys_settings.supported;
                self.keycode_picker.supports_combo = !self.combo_entries.is_empty();
                self.keycode_picker.supports_auto_shift = self.auto_shift_timeout.is_some();
                self.keycode_picker.supports_caps_word = r.vial_features.caps_word;
                self.keycode_picker.supports_repeat_key = r.vial_features.repeat_key;
                self.keycode_picker.supports_layer_lock = r.vial_features.layer_lock;
                self.keycode_picker.supports_persistent_default_layer =
                    r.vial_features.persistent_default_layer;
                self.keycode_picker.layer_count = r.layout.layers.len().max(1);
                self.keycode_picker.tap_dance_names = load_tap_dance_names(&device_name);
                // Vial GUI maps customKeycodes to USER00.. at QK_KB + index.
                // Protocol v6: QK_KB = 0x7E00. Do not use QK_USER (0x7E40):
                // assigning those values writes the wrong keycodes to firmware.
                const QK_KB: u16 = 0x7E00;
                self.keycode_picker.custom_keycodes = r
                    .layout
                    .custom_keycodes
                    .iter()
                    .enumerate()
                    .map(|(i, custom)| {
                        (
                            custom.name.clone(),
                            custom.label.clone(),
                            custom.title.clone(),
                            QK_KB + i as u16,
                        )
                    })
                    .collect();
                self.keycode_picker.layer_names = self.layer_names.clone();
                self.sticky_layout_prev_pressed.clear();
                self.sticky_layout_pressed_key_layers.clear();
                self.sticky_layout_toggled_layers = vec![false; r.layout.layers.len().max(1)];
                self.sticky_layout_base_layer = 0;

                self.layout = Some(r.layout);
                self.refresh_layer_picker_content_flags();

                // Open persistent HID connection for Vial real-time writes
                if self.firmware == FirmwareProtocol::Vial {
                    if let Some(dev) = self
                        .selected_device
                        .and_then(|i| self.device_manager.devices().get(i))
                    {
                        match crate::hid::HidDevice::open(&dev.path) {
                            Ok(v) => {
                                self.hid_device = Some(v);
                                self.restore_entropy_display_preset_after_connect();
                            }
                            Err(e) => log::warn!("Could not open persistent HID: {e}"),
                        }
                        self.sync_qmk_hid_host_bridges();
                    }
                }

                log::info!(
                    "Connected: {} ({} layers, {:?})",
                    r.device_name,
                    r.layer_count,
                    self.firmware
                );
            }
            Err(e) => {
                self.status_msg = e;
            }
        }
    }
}
