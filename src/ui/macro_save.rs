use super::*;

#[cfg(not(target_arch = "wasm32"))]
impl EntropyApp {
    pub(super) fn poll_macro_save(&mut self, ctx: &egui::Context) {
        let Some(rx) = self.macro_save_rx.take() else {
            return;
        };

        match rx.try_recv() {
            Ok(Ok(())) => {
                self.macro_saving = false;
                self.status_msg = crate::i18n::tr_catalog(
                    self.app_settings.language,
                    "status_messages.macros_saved",
                )
                .into();
                ctx.request_repaint();
            }
            Ok(Err(error)) => {
                self.macro_saving = false;
                self.status_msg = format!("Macro write error: {error}");
                ctx.request_repaint();
            }
            Err(mpsc::TryRecvError::Empty) => {
                self.macro_save_rx = Some(rx);
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.macro_saving = false;
                self.status_msg = "Macro write error: save task died".into();
                ctx.request_repaint();
            }
        }
    }

    pub(super) fn start_macro_save(&mut self) {
        let macro_texts = self.keycode_picker.macro_texts.clone();
        if let Some(hid) = &self.hid_device {
            match hid.get_macro_buffer_size() {
                Ok(size) => {
                    let buf = crate::hid::HidDevice::encode_macros(&macro_texts, size);
                    match hid.set_macro_buffer(&buf) {
                        Ok(()) => {
                            self.status_msg = crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "status_messages.macros_saved",
                            )
                            .into();
                        }
                        Err(e) => self.status_msg = format!("Macro write error: {e}"),
                    }
                }
                Err(e) => self.status_msg = format!("Macro write error: {e}"),
            }
            return;
        }

        let Some(dev) = self
            .selected_device
            .and_then(|idx| self.device_manager.devices().get(idx))
            .cloned()
        else {
            self.status_msg = "Macro write error: no selected device".into();
            return;
        };

        let (tx, rx) = mpsc::channel();
        self.macro_save_rx = Some(rx);
        self.macro_saving = true;
        self.status_msg = "Saving macros…".into();

        std::thread::spawn(move || {
            let result = (|| -> Result<(), String> {
                let hid = crate::hid::HidDevice::open_fresh_for(&dev)
                    .map_err(|e| format!("open failed: {e}"))?;
                let size = hid
                    .get_macro_buffer_size()
                    .map_err(|e| format!("buffer size failed: {e}"))?;
                let buf = crate::hid::HidDevice::encode_macros(&macro_texts, size);
                hid.set_macro_buffer(&buf)
                    .map_err(|e| format!("buffer write failed: {e}"))?;
                Ok(())
            })();
            let _ = tx.send(result);
        });
    }
}
