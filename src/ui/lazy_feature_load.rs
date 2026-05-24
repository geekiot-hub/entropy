use super::*;

#[cfg(not(target_arch = "wasm32"))]
impl EntropyApp {
    pub(super) fn poll_lazy_feature_loads(&mut self, ctx: &egui::Context) {
        if self.keycode_picker.macro_load_requested {
            self.keycode_picker.macro_load_requested = false;
            self.start_macro_lazy_load();
        }

        if let Some(rx) = self.macro_load_rx.take() {
            match rx.try_recv() {
                Ok(Ok(result)) => {
                    self.apply_lazy_macro_texts(result.macro_texts);
                    self.keycode_picker.macros_loading = false;
                    self.keycode_picker.macro_load_error = None;
                    self.status_msg = "Macros loaded".into();
                    ctx.request_repaint();
                }
                Ok(Err(error)) => {
                    self.keycode_picker.macros_loading = false;
                    self.keycode_picker.macro_load_error = Some(error.clone());
                    self.status_msg = format!("Macro load failed: {error}");
                    ctx.request_repaint();
                }
                Err(mpsc::TryRecvError::Empty) => {
                    self.macro_load_rx = Some(rx);
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.keycode_picker.macros_loading = false;
                    self.keycode_picker.macro_load_error = Some("Macro load task died".into());
                    ctx.request_repaint();
                }
            }
        }

        if let Some(rx) = self.alt_repeat_load_rx.take() {
            match rx.try_recv() {
                Ok(Ok(result)) => {
                    self.alt_repeat_entries = result.entries;
                    self.alt_repeat_names = load_alt_repeat_names(&self.current_device_name);
                    self.alt_repeat_names
                        .resize(self.alt_repeat_entries.len(), String::new());
                    self.alt_repeat_loaded = true;
                    self.alt_repeat_loading = false;
                    self.alt_repeat_load_error = None;
                    self.status_msg = "Alt Repeat loaded".into();
                    ctx.request_repaint();
                }
                Ok(Err(error)) => {
                    self.alt_repeat_loading = false;
                    self.alt_repeat_load_error = Some(error.clone());
                    self.status_msg = format!("Alt Repeat load failed: {error}");
                    ctx.request_repaint();
                }
                Err(mpsc::TryRecvError::Empty) => {
                    self.alt_repeat_load_rx = Some(rx);
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.alt_repeat_loading = false;
                    self.alt_repeat_load_error = Some("Alt Repeat load task died".into());
                    ctx.request_repaint();
                }
            }
        }
    }

    fn start_macro_lazy_load(&mut self) {
        if self.keycode_picker.macros_loading {
            return;
        }
        let dev = match self
            .selected_device
            .and_then(|idx| self.device_manager.devices().get(idx))
            .cloned()
        {
            Some(dev) => dev,
            None => {
                self.keycode_picker.macro_load_error = Some("No selected device".into());
                return;
            }
        };

        let (tx, rx) = mpsc::channel();
        self.macro_load_rx = Some(rx);
        self.keycode_picker.macros_loading = true;
        self.keycode_picker.macro_load_error = None;
        self.status_msg = "Loading macros…".into();

        std::thread::spawn(move || {
            let result = (|| -> Result<MacroLoadResult, String> {
                let dev_conn = crate::hid::HidDevice::open_fresh_for(&dev)
                    .map_err(|e| format!("Open failed: {e}"))?;
                let count = dev_conn
                    .get_macro_count()
                    .map_err(|e| format!("Macro count failed: {e}"))?;
                let size = dev_conn
                    .get_macro_buffer_size()
                    .map_err(|e| format!("Macro buffer size failed: {e}"))?;
                let buf = dev_conn
                    .get_macro_buffer(size)
                    .map_err(|e| format!("Macro buffer read failed: {e}"))?;
                Ok(MacroLoadResult {
                    macro_texts: crate::hid::HidDevice::parse_macros(&buf, count),
                })
            })();
            let _ = tx.send(result);
        });
    }

    pub(super) fn start_alt_repeat_lazy_load(&mut self) {
        if self.alt_repeat_loading {
            return;
        }
        let dev = match self
            .selected_device
            .and_then(|idx| self.device_manager.devices().get(idx))
            .cloned()
        {
            Some(dev) => dev,
            None => {
                self.alt_repeat_load_error = Some("No selected device".into());
                return;
            }
        };

        let (tx, rx) = mpsc::channel();
        self.alt_repeat_load_rx = Some(rx);
        self.alt_repeat_loading = true;
        self.alt_repeat_load_error = None;
        self.status_msg = "Loading Alt Repeat…".into();

        std::thread::spawn(move || {
            let result = (|| -> Result<AltRepeatLoadResult, String> {
                let dev_conn = crate::hid::HidDevice::open_fresh_for(&dev)
                    .map_err(|e| format!("Open failed: {e}"))?;
                let (_, _, _, alt_repeat_count, _) = dev_conn
                    .get_dynamic_entry_counts()
                    .map_err(|e| format!("Dynamic feature count failed: {e}"))?;
                let mut entries = Vec::new();
                for i in 0..alt_repeat_count {
                    match dev_conn.get_alt_repeat_key(i) {
                        Ok((keycode, alt_keycode, allowed_mods, options)) => {
                            entries.push(AltRepeatKeyEntry {
                                keycode,
                                alt_keycode,
                                allowed_mods,
                                options: AltRepeatKeyOptionsState::from_bits(options),
                            });
                        }
                        Err(e) => return Err(format!("Alt Repeat entry {i} failed: {e}")),
                    }
                }
                Ok(AltRepeatLoadResult { entries })
            })();
            let _ = tx.send(result);
        });
    }

    fn apply_lazy_macro_texts(&mut self, macro_texts: Vec<String>) {
        self.keycode_picker.macro_count = macro_texts.len();
        self.keycode_picker.macro_texts = macro_texts.clone();
        self.keycode_picker.macro_names = vec![String::new(); macro_texts.len()];
        self.keycode_picker.macro_actions = macro_texts
            .iter()
            .map(|text| {
                let bytes = text.as_bytes();
                let mut actions = Vec::new();
                let mut i = 0;
                while i < bytes.len() {
                    if bytes[i] == 1 && i + 1 < bytes.len() {
                        match bytes[i + 1] {
                            1 if i + 2 < bytes.len() => {
                                actions.push(crate::keycode_picker::MacroAction::Tap(bytes[i + 2]));
                                i += 3;
                            }
                            2 if i + 2 < bytes.len() => {
                                actions
                                    .push(crate::keycode_picker::MacroAction::Down(bytes[i + 2]));
                                i += 3;
                            }
                            3 if i + 2 < bytes.len() => {
                                actions.push(crate::keycode_picker::MacroAction::Up(bytes[i + 2]));
                                i += 3;
                            }
                            4 if i + 3 < bytes.len() => {
                                let ms =
                                    (bytes[i + 2] as u16 - 1) + (bytes[i + 3] as u16 - 1) * 255;
                                actions.push(crate::keycode_picker::MacroAction::Delay(ms));
                                i += 4;
                            }
                            _ => i += 2,
                        }
                    } else {
                        let start = i;
                        while i < bytes.len() && bytes[i] != 1 {
                            i += 1;
                        }
                        if let Ok(s) = std::str::from_utf8(&bytes[start..i]) {
                            actions.push(crate::keycode_picker::MacroAction::Text(s.to_string()));
                        }
                    }
                }
                actions
            })
            .collect();
        self.keycode_picker.macros_loaded = true;
    }
}
