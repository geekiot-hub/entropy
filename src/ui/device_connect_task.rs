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

                let touchpad_settings_in_definition =
                    Self::layout_json_has_touchpad_settings(&json);
                progress("Querying QMK settings…");
                let supported_qmk_settings = dev_conn.query_qmk_settings().unwrap_or_else(|e| {
                    log::warn!("qmk settings query failed: {e}");
                    Vec::new()
                });
                let has_qmk_setting = |qsid: u16| supported_qmk_settings.contains(&qsid);

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

                let mut firmware_layer_names = Vec::new();
                if has_qmk_setting(200) {
                    for layer in 0..layer_count.min(16) {
                        let qsid = 200 + layer as u16;
                        if !has_qmk_setting(qsid) {
                            firmware_layer_names.clear();
                            break;
                        }
                        match dev_conn.get_qmk_setting_string(qsid) {
                            Ok(name) if !name.is_empty() => firmware_layer_names.push(name),
                            Ok(_) => firmware_layer_names.push(layer.to_string()),
                            Err(_) => {
                                firmware_layer_names.clear();
                                break;
                            }
                        }
                    }
                }
                if !firmware_layer_names.is_empty() {
                    layout.layer_names = firmware_layer_names;
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

                progress("Reading macros…");
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
                        Vec::new()
                    }
                };

                progress("Reading dynamic feature counts…");
                let (
                    tap_dance_count,
                    combo_count,
                    key_override_count,
                    reported_alt_repeat_count,
                    dynamic_feature_bits,
                ) = match dev_conn.get_dynamic_entry_counts() {
                    Ok(counts) => counts,
                    Err(e) => {
                        log::warn!("get_dynamic_entry_counts: {e}");
                        (0, 0, 0, 0, 0)
                    }
                };
                if reported_alt_repeat_count > 0 {
                    log::warn!(
                        "Skipping Alt Repeat preload: firmware reported {reported_alt_repeat_count} entries, but this optional command can hang on RMK/Vial devices"
                    );
                }
                let vial_features = VialFeatureSupport {
                    caps_word: dynamic_feature_bits & (1 << 0) != 0,
                    layer_lock: dynamic_feature_bits & (1 << 1) != 0,
                    persistent_default_layer: key_override_count > 0,
                    repeat_key: false,
                };

                progress("Reading combos…");
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

                progress("Reading QMK settings values…");
                let combo_term = if has_qmk_setting(2) {
                    match dev_conn.get_qmk_setting_u16(2) {
                        Ok(value) => Some(value),
                        Err(e) => {
                            log::warn!("get_qmk_setting_u16(combo_term): {e}");
                            None
                        }
                    }
                } else {
                    None
                };
                let auto_shift_options = if has_qmk_setting(3) {
                    match dev_conn.get_qmk_setting_u8(3) {
                        Ok(value) => Some(AutoShiftOptionsState::from_bits(value)),
                        Err(e) => {
                            log::warn!("get_qmk_setting_u8(auto_shift_flags): {e}");
                            None
                        }
                    }
                } else {
                    None
                };
                let auto_shift_timeout = if has_qmk_setting(4) {
                    match dev_conn.get_qmk_setting_u16(4) {
                        Ok(value) => Some(value),
                        Err(e) => {
                            log::warn!("get_qmk_setting_u16(auto_shift_timeout): {e}");
                            None
                        }
                    }
                } else {
                    None
                };

                let mouse_keys_settings = {
                    let mut mk = MouseKeysSettingsState::default();
                    match has_qmk_setting(9).then(|| dev_conn.get_qmk_setting_u8(9)) {
                        Some(Ok(v)) => {
                            mk.delay = v as u16;
                            mk.supported = true;
                            let read = |qsid: u16| -> u16 {
                                if !has_qmk_setting(qsid) {
                                    return 0;
                                }
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
                        Some(Err(e)) => {
                            log::warn!("get_qmk_setting_u8(mouse_keys delay): {e}");
                        }
                        None => {}
                    }
                    mk
                };

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

                progress("Reading module settings…");
                let module_settings =
                    Self::read_module_settings(&json, &supported_qmk_settings, &dev_conn);

                let tap_hold_settings = {
                    let mut th = TapHoldSettingsState::default();
                    match has_qmk_setting(7).then(|| dev_conn.get_qmk_setting_u16(7)) {
                        Some(Ok(v)) => {
                            th.tapping_term = v;
                            th.supported = true;
                            let read_bool = |qsid: u16| -> bool {
                                if !has_qmk_setting(qsid) {
                                    return false;
                                }
                                match dev_conn.get_qmk_setting_u8(qsid) {
                                    Ok(val) => val != 0,
                                    Err(e) => {
                                        log::warn!("get_qmk_setting_u8(tap_hold qsid {qsid}): {e}");
                                        false
                                    }
                                }
                            };
                            let read_u16 = |qsid: u16| -> u16 {
                                if !has_qmk_setting(qsid) {
                                    return 0;
                                }
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
                            th.tapping_toggle = if has_qmk_setting(20) {
                                dev_conn
                                    .get_qmk_setting_u8(20)
                                    .map(|value| value as u16)
                                    .unwrap_or_else(|e| {
                                        log::warn!("get_qmk_setting_u8(tap_hold qsid 20): {e}");
                                        0
                                    })
                            } else {
                                0
                            };
                            th.chordal_hold = read_bool(26);
                            th.flow_tap = read_u16(27);
                        }
                        Some(Err(e)) => {
                            log::warn!("get_qmk_setting_u16(tap_hold tapping_term): {e}");
                        }
                        None => {}
                    }
                    th
                };

                let magic_settings = {
                    match has_qmk_setting(21).then(|| dev_conn.get_qmk_setting_u16(21)) {
                        Some(Ok(bits)) => MagicSettingsState {
                            bits,
                            supported: true,
                        },
                        Some(Err(e)) => {
                            log::warn!("get_qmk_setting_u16(magic qsid 21): {e}");
                            MagicSettingsState::default()
                        }
                        None => MagicSettingsState::default(),
                    }
                };

                let one_shot_settings = {
                    let mut os = OneShotSettingsState::default();
                    match has_qmk_setting(5).then(|| dev_conn.get_qmk_setting_u8(5)) {
                        Some(Ok(v)) => {
                            os.tap_toggle = v;
                            os.supported = true;
                            os.timeout = if has_qmk_setting(6) {
                                dev_conn.get_qmk_setting_u16(6).unwrap_or_else(|e| {
                                    log::warn!("get_qmk_setting_u16(one_shot timeout qsid 6): {e}");
                                    0
                                })
                            } else {
                                0
                            };
                        }
                        Some(Err(e)) => {
                            log::warn!("get_qmk_setting_u8(one_shot tap toggle qsid 5): {e}");
                        }
                        None => {}
                    }
                    os
                };

                let grave_escape_settings = {
                    match has_qmk_setting(1).then(|| dev_conn.get_qmk_setting_u8(1)) {
                        Some(Ok(bits)) => GraveEscapeSettingsState {
                            bits,
                            supported: true,
                        },
                        Some(Err(e)) => {
                            log::warn!("get_qmk_setting_u8(grave_escape qsid 1): {e}");
                            GraveEscapeSettingsState::default()
                        }
                        None => GraveEscapeSettingsState::default(),
                    }
                };

                let layer_led_settings = {
                    let mut leds = LayerLedSettingsState::default();
                    match has_qmk_setting(300).then(|| dev_conn.get_qmk_setting_u8(300)) {
                        Some(Ok(v)) => {
                            leds.layer_colors[0] = v;
                            leds.supported = true;
                            for layer in 1..16 {
                                let qsid = 300 + layer as u16;
                                if has_qmk_setting(qsid) {
                                    leds.layer_colors[layer] =
                                        dev_conn.get_qmk_setting_u8(qsid).unwrap_or_else(|e| {
                                            log::warn!(
                                                "get_qmk_setting_u8(layer_led qsid {qsid}): {e}"
                                            );
                                            0
                                        });
                                }
                            }
                            leds.brightness = if has_qmk_setting(316) {
                                dev_conn
                                    .get_qmk_setting_u16(316)
                                    .unwrap_or_else(|e| {
                                        log::warn!(
                                            "get_qmk_setting_u16(layer_led brightness): {e}"
                                        );
                                        0
                                    })
                                    .min(255)
                            } else {
                                0
                            };
                            leds.timeout_mins = if has_qmk_setting(317) {
                                dev_conn.get_qmk_setting_u8(317).unwrap_or_else(|e| {
                                    log::warn!("get_qmk_setting_u8(layer_led timeout): {e}");
                                    0
                                })
                            } else {
                                0
                            };
                        }
                        Some(Err(e)) => {
                            log::warn!("get_qmk_setting_u8(layer_led layer 0 color): {e}");
                        }
                        None => {}
                    }
                    leds
                };

                progress("Reading RGB settings…");
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

                progress("Reading tap dance entries…");
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

                progress("Reading key overrides…");
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

                let alt_repeat_entries = Vec::new();

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
