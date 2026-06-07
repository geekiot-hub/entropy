use super::*;

fn universal_symbols_setup_steps() -> &'static [&'static str] {
    #[cfg(target_os = "windows")]
    {
        &[
            "No extra setup is required on Windows",
            "Keep Entropy running while using Universal Symbols",
            "Assign keys from Symbols → Universal symbols in the key picker",
        ]
    }
    #[cfg(target_os = "macos")]
    {
        &[
            "Open Privacy & Security",
            "Allow Entropy in Accessibility",
            "If prompted, allow Entropy in Input Monitoring too",
            "Restart Entropy after changing permissions",
            "Keep Entropy running while using Universal Symbols",
        ]
    }
    #[cfg(target_os = "linux")]
    {
        &[
            "X11: install xdotool and keep Entropy running",
            "Wayland + IBus: install Entropy Universal Symbols and select it as an input source",
            "Wayland + Fcitx5: install the addon, restart Fcitx5, and enable Entropy Universal Symbols",
            "Assign keys from Symbols → Universal symbols in the key picker",
        ]
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        &["Universal Symbols are not supported on this OS yet"]
    }
}

impl EntropyApp {
    pub(super) fn draw_universal_symbols_setup_page(
        &mut self,
        ui: &mut egui::Ui,
        content_rect: egui::Rect,
    ) {
        let lang = self.app_settings.language;
        let dark = ui.visuals().dark_mode;
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        let content_width = metrics.settings_content_width();
        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(metrics.value(18.0));
                ui.allocate_ui_with_layout(
                    Vec2::new(content_width, 0.0),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        ui.add_sized(
                            Vec2::new(content_width, metrics.value(24.0)),
                            egui::Label::new(
                                RichText::new(crate::i18n::tr(
                                    lang,
                                    crate::i18n::Key::UniversalSymbolsSetupTitle,
                                ))
                                .size(metrics.value(18.0))
                                .strong(),
                            )
                            .halign(egui::Align::Center),
                        );
                        ui.add_space(metrics.value(6.0));
                        ui.add_sized(
                            Vec2::new(content_width, metrics.value(18.0)),
                            egui::Label::new(
                                RichText::new(crate::i18n::tr_text(
                                    lang,
                                    &crate::smart_input::universal_output_status(),
                                ))
                                .size(metrics.value(12.5))
                                .color(app_muted_text(dark)),
                            )
                            .wrap()
                            .halign(egui::Align::Center),
                        );
                        if let Some(hint) = crate::smart_input::universal_output_setup_hint() {
                            ui.add_space(metrics.value(4.0));
                            ui.add_sized(
                                Vec2::new(content_width, metrics.value(18.0)),
                                egui::Label::new(
                                    RichText::new(crate::i18n::tr_text(lang, hint))
                                        .size(metrics.value(11.0))
                                        .color(app_muted_text(dark)),
                                )
                                .wrap()
                                .halign(egui::Align::Center),
                            );
                        }
                        ui.add_space(metrics.value(16.0));

                        let steps = universal_symbols_setup_steps();
                        let step_height = metrics.value(24.0);
                        let block_height = steps.len() as f32 * step_height;
                        let block_rect = ui
                            .allocate_exact_size(
                                Vec2::new(content_width, block_height),
                                egui::Sense::hover(),
                            )
                            .0;
                        let fill = if dark {
                            Color32::from_rgb(38, 38, 41)
                        } else {
                            Color32::from_rgb(252, 252, 254)
                        };
                        ui.painter().rect(
                            block_rect,
                            metrics.value(14.0),
                            fill,
                            crate::ui_style::modal_outline_stroke(dark),
                            egui::StrokeKind::Inside,
                        );
                        for (idx, step) in steps.iter().enumerate() {
                            let y = block_rect.top() + step_height * idx as f32 + step_height / 2.0;
                            ui.painter().text(
                                egui::pos2(block_rect.center().x, y),
                                egui::Align2::CENTER_CENTER,
                                crate::i18n::tr_text(lang, step),
                                FontId::proportional(metrics.value(12.0)),
                                if idx == 0 {
                                    app_accent()
                                } else {
                                    ui.visuals().text_color()
                                },
                            );
                        }

                        ui.add_space(metrics.value(16.0));
                        self.draw_universal_symbols_setup_actions(ui, metrics);
                        if !self.status_msg.is_empty() {
                            ui.add_space(metrics.value(10.0));
                            ui.add_sized(
                                Vec2::new(content_width, metrics.value(36.0)),
                                egui::Label::new(
                                    RichText::new(&self.status_msg)
                                        .size(metrics.value(11.5))
                                        .color(app_muted_text(dark)),
                                )
                                .wrap()
                                .halign(egui::Align::Center),
                            );
                        }
                    },
                );
            });
        });
    }

    fn draw_universal_symbols_setup_actions(
        &mut self,
        ui: &mut egui::Ui,
        metrics: crate::ui_style::ResponsiveMetrics,
    ) {
        let _ = &metrics;
        ui.horizontal_centered(|ui| {
            let _ = &mut *ui;
            #[cfg(target_os = "macos")]
            {
                if crate::ui_style::modern_button(
                    ui,
                    crate::i18n::tr_catalog(self.app_settings.language, "universal_symbols_setup.open_privacy_settings"),
                    metrics.size(184.0, 34.0),
                    true,
                )
                .clicked()
                {
                    let result = std::process::Command::new("open")
                        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
                        .status();
                    self.status_msg = if matches!(result, Ok(status) if status.success()) {
                        "Opened macOS Privacy settings".to_string()
                    } else {
                        "Could not open macOS Privacy settings".to_string()
                    };
                }
            }

            #[cfg(target_os = "linux")]
            {
                if crate::ui_style::modern_button(ui, crate::i18n::tr_catalog(self.app_settings.language, "universal_symbols_setup.install_ibus"), metrics.size(132.0, 34.0), true)
                    .clicked()
                {
                    self.run_linux_universal_symbols_setup("linux/ibus/install-user.sh", "IBus");
                }
                ui.add_space(metrics.value(8.0));
                if crate::ui_style::modern_button(ui, crate::i18n::tr_catalog(self.app_settings.language, "universal_symbols_setup.uninstall_ibus"), metrics.size(144.0, 34.0), true)
                    .clicked()
                {
                    self.run_linux_universal_symbols_setup("linux/ibus/uninstall-user.sh", "IBus");
                }
                ui.add_space(metrics.value(8.0));
                if crate::ui_style::modern_button(ui, crate::i18n::tr_catalog(self.app_settings.language, "universal_symbols_setup.install_fcitx5"), metrics.size(142.0, 34.0), true)
                    .clicked()
                {
                    self.run_linux_universal_symbols_setup("linux/fcitx5/install-user.sh", "Fcitx5");
                }
            }

            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            {
                ui.label(
                    RichText::new(crate::i18n::tr_catalog(self.app_settings.language, "universal_symbols_setup.no_setup_action"))
                        .size(11.0)
                        .color(app_muted_text(ui.visuals().dark_mode)),
                );
            }
        });
    }

    #[cfg(target_os = "linux")]
    fn run_linux_universal_symbols_setup(&mut self, script: &str, backend: &str) {
        let Some(script_path) = linux_universal_symbols_setup_script(script) else {
            self.status_msg = format!("Could not find {script}; run it from the Entropy folder");
            return;
        };
        let output = std::process::Command::new("sh").arg(&script_path).output();
        self.status_msg = match output {
            Ok(output) if output.status.success() => {
                let details = command_output_summary(&output.stdout, &output.stderr);
                if details.is_empty() {
                    format!("{backend} setup completed")
                } else {
                    details
                }
            }
            Ok(output) => {
                let details = command_output_summary(&output.stderr, &output.stdout);
                if details.is_empty() {
                    format!("{backend} setup failed: {}", output.status)
                } else {
                    format!("{backend} setup failed: {details}")
                }
            }
            Err(err) => format!("Could not run {}: {err}", script_path.display()),
        };
    }
}

#[cfg(target_os = "linux")]
fn command_output_summary(primary: &[u8], fallback: &[u8]) -> String {
    let text = if primary.is_empty() {
        fallback
    } else {
        primary
    };
    String::from_utf8_lossy(text)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .rev()
        .take(2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(target_os = "linux")]
fn linux_universal_symbols_setup_script(script: &str) -> Option<std::path::PathBuf> {
    let relative = std::path::Path::new(script);
    if relative.exists() {
        return Some(relative.to_path_buf());
    }
    if let Some(appdir) = std::env::var_os("APPDIR") {
        let path = std::path::PathBuf::from(appdir).join(script);
        if path.exists() {
            return Some(path);
        }
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.to_path_buf()))
        .and_then(|dir| {
            for ancestor in dir.ancestors() {
                let path = ancestor.join(script);
                if path.exists() {
                    return Some(path);
                }
            }
            None
        })
}
