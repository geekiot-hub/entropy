use super::*;

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
                            Vec2::new(content_width, metrics.value(34.0)),
                            egui::Label::new(
                                RichText::new(crate::i18n::tr_catalog(
                                    lang,
                                    universal_symbols_intro_key(),
                                ))
                                .size(metrics.value(13.0))
                                .color(app_muted_text(dark)),
                            )
                            .wrap()
                            .halign(egui::Align::Center),
                        );
                        ui.add_space(metrics.value(18.0));

                        self.draw_universal_symbols_setup_rows(ui, metrics, lang, dark);

                        ui.add_space(metrics.value(18.0));
                        self.draw_universal_symbols_setup_actions(ui, metrics);
                        if !self.status_msg.is_empty() {
                            ui.add_space(metrics.value(12.0));
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

    fn draw_universal_symbols_setup_rows(
        &mut self,
        ui: &mut egui::Ui,
        metrics: crate::ui_style::ResponsiveMetrics,
        lang: crate::i18n::Language,
        dark: bool,
    ) {
        let row_height = metrics.settings_row_height();
        let row_content_width = metrics.settings_row_content_width();
        let tooltip = |key: &'static str| Some(crate::i18n::tr_catalog(lang, key));

        crate::ui_style::settings_list_row_with_tooltip(
            ui,
            row_content_width,
            row_height,
            crate::i18n::tr_catalog(lang, "universal_symbols_setup.current_backend"),
            true,
            tooltip("universal_symbols_setup.current_backend_tooltip"),
            metrics.settings_control_width(),
            |ui| {
                draw_universal_symbols_value(
                    ui,
                    metrics,
                    168.0,
                    crate::i18n::tr_catalog(lang, universal_symbols_backend_value_key()),
                    ui.visuals().text_color(),
                );
            },
        );

        crate::ui_style::settings_list_row_with_tooltip(
            ui,
            row_content_width,
            row_height,
            crate::i18n::tr_catalog(lang, "universal_symbols_setup.recommended_setup"),
            true,
            tooltip("universal_symbols_setup.recommended_setup_tooltip"),
            metrics.settings_control_width(),
            |ui| self.draw_universal_symbols_recommended_control(ui, metrics, lang),
        );

        crate::ui_style::settings_list_row_with_tooltip(
            ui,
            row_content_width,
            row_height,
            crate::i18n::tr_catalog(lang, "universal_symbols_setup.next_step"),
            true,
            tooltip("universal_symbols_setup.next_step_tooltip"),
            metrics.value(220.0),
            |ui| {
                draw_universal_symbols_value(
                    ui,
                    metrics,
                    220.0,
                    crate::i18n::tr_catalog(lang, universal_symbols_next_step_key()),
                    app_muted_text(dark),
                );
            },
        );

        crate::ui_style::settings_list_row_with_tooltip(
            ui,
            row_content_width,
            row_height,
            crate::i18n::tr_catalog(lang, "universal_symbols_setup.text_expander"),
            true,
            tooltip("universal_symbols_setup.text_expander_tooltip"),
            metrics.value(220.0),
            |ui| {
                draw_universal_symbols_value(
                    ui,
                    metrics,
                    220.0,
                    crate::i18n::tr_catalog(lang, universal_symbols_text_expander_key()),
                    app_muted_text(dark),
                );
            },
        );
    }

    fn draw_universal_symbols_recommended_control(
        &mut self,
        ui: &mut egui::Ui,
        metrics: crate::ui_style::ResponsiveMetrics,
        lang: crate::i18n::Language,
    ) {
        #[cfg(target_os = "linux")]
        {
            match crate::smart_input::linux_recommended_input_backend() {
                crate::smart_input::LinuxRecommendedInputBackend::X11Native => {
                    draw_universal_symbols_value(
                        ui,
                        metrics,
                        168.0,
                        crate::i18n::tr_catalog(lang, "universal_symbols_setup.no_install_needed"),
                        ui.visuals().text_color(),
                    );
                }
                crate::smart_input::LinuxRecommendedInputBackend::IBus => {
                    if crate::ui_style::modern_button(
                        ui,
                        crate::i18n::tr_catalog(
                            lang,
                            "universal_symbols_setup.install_recommended",
                        ),
                        metrics.size(168.0, 34.0),
                        true,
                    )
                    .clicked()
                    {
                        self.run_linux_universal_symbols_setup(
                            "linux/ibus/install-user.sh",
                            "IBus",
                        );
                    }
                }
                crate::smart_input::LinuxRecommendedInputBackend::Fcitx5 => {
                    if crate::ui_style::modern_button(
                        ui,
                        crate::i18n::tr_catalog(
                            lang,
                            "universal_symbols_setup.install_recommended",
                        ),
                        metrics.size(168.0, 34.0),
                        true,
                    )
                    .clicked()
                    {
                        self.run_linux_universal_symbols_setup(
                            "linux/fcitx5/install-user.sh",
                            "Fcitx5",
                        );
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if crate::ui_style::modern_button(
                ui,
                crate::i18n::tr_catalog(lang, "universal_symbols_setup.open_privacy_settings"),
                metrics.size(168.0, 34.0),
                true,
            )
            .clicked()
            {
                self.open_macos_universal_symbols_privacy_settings(lang);
            }
        }

        #[cfg(target_os = "windows")]
        {
            draw_universal_symbols_value(
                ui,
                metrics,
                168.0,
                crate::i18n::tr_catalog(lang, "universal_symbols_setup.no_install_needed"),
                ui.visuals().text_color(),
            );
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            draw_universal_symbols_value(
                ui,
                metrics,
                168.0,
                crate::i18n::tr_catalog(lang, "universal_symbols_setup.unsupported"),
                app_muted_text(ui.visuals().dark_mode),
            );
        }
    }

    fn draw_universal_symbols_setup_actions(
        &mut self,
        ui: &mut egui::Ui,
        metrics: crate::ui_style::ResponsiveMetrics,
    ) {
        #[cfg(target_os = "windows")]
        {
            let _ = (ui, metrics);
        }

        #[cfg(target_os = "macos")]
        {
            let _ = (ui, metrics);
        }

        #[cfg(target_os = "linux")]
        {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "universal_symbols_setup.advanced",
                    ))
                    .size(metrics.value(11.0))
                    .color(app_muted_text(ui.visuals().dark_mode)),
                );
                ui.add_space(metrics.value(6.0));
                ui.horizontal_centered(|ui| {
                    if crate::ui_style::modern_button(
                        ui,
                        crate::i18n::tr_catalog(
                            self.app_settings.language,
                            "universal_symbols_setup.install_ibus",
                        ),
                        metrics.size(132.0, 34.0),
                        true,
                    )
                    .clicked()
                    {
                        self.run_linux_universal_symbols_setup(
                            "linux/ibus/install-user.sh",
                            "IBus",
                        );
                    }
                    ui.add_space(metrics.value(8.0));
                    if crate::ui_style::modern_button(
                        ui,
                        crate::i18n::tr_catalog(
                            self.app_settings.language,
                            "universal_symbols_setup.uninstall_ibus",
                        ),
                        metrics.size(144.0, 34.0),
                        true,
                    )
                    .clicked()
                    {
                        self.run_linux_universal_symbols_setup(
                            "linux/ibus/uninstall-user.sh",
                            "IBus",
                        );
                    }
                    ui.add_space(metrics.value(8.0));
                    if crate::ui_style::modern_button(
                        ui,
                        crate::i18n::tr_catalog(
                            self.app_settings.language,
                            "universal_symbols_setup.install_fcitx5",
                        ),
                        metrics.size(142.0, 34.0),
                        true,
                    )
                    .clicked()
                    {
                        self.run_linux_universal_symbols_setup(
                            "linux/fcitx5/install-user.sh",
                            "Fcitx5",
                        );
                    }
                });
            });
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            ui.horizontal_centered(|ui| {
                ui.label(
                    RichText::new(crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "universal_symbols_setup.no_setup_action",
                    ))
                    .size(11.0)
                    .color(app_muted_text(ui.visuals().dark_mode)),
                );
            });
        }
    }

    #[cfg(target_os = "linux")]
    fn run_linux_universal_symbols_setup(&mut self, script: &str, backend: &str) {
        let Some(script_path) = linux_universal_symbols_setup_script(script) else {
            self.status_msg = format!("Could not find {script}; run it from the Entropy folder");
            return;
        };
        let output = std::process::Command::new("sh").arg(&script_path).output();
        self.status_msg = match output {
            Ok(output) if output.status.success() => crate::i18n::tr_catalog(
                self.app_settings.language,
                linux_setup_success_status_key(script, backend),
            )
            .to_owned(),
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

    #[cfg(target_os = "macos")]
    fn open_macos_universal_symbols_privacy_settings(&mut self, lang: crate::i18n::Language) {
        let result = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .status();
        self.status_msg = if matches!(result, Ok(status) if status.success()) {
            crate::i18n::tr_catalog(lang, "universal_symbols_setup.macos_privacy_opened_status")
                .to_string()
        } else {
            crate::i18n::tr_catalog(
                lang,
                "universal_symbols_setup.macos_privacy_open_failed_status",
            )
            .to_string()
        };
    }
}

fn draw_universal_symbols_value(
    ui: &mut egui::Ui,
    metrics: crate::ui_style::ResponsiveMetrics,
    width: f32,
    text: &str,
    color: Color32,
) {
    ui.allocate_ui_with_layout(
        metrics.size(width, 44.0),
        egui::Layout::right_to_left(egui::Align::Center),
        |ui| {
            ui.add_sized(
                metrics.size(width, 44.0),
                egui::Label::new(RichText::new(text).size(metrics.value(12.0)).color(color))
                    .wrap()
                    .halign(egui::Align::RIGHT),
            );
        },
    );
}

fn universal_symbols_intro_key() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        match crate::smart_input::linux_recommended_input_backend() {
            crate::smart_input::LinuxRecommendedInputBackend::X11Native => {
                "universal_symbols_setup.intro_linux_x11"
            }
            crate::smart_input::LinuxRecommendedInputBackend::IBus => {
                "universal_symbols_setup.intro_linux_ibus"
            }
            crate::smart_input::LinuxRecommendedInputBackend::Fcitx5 => {
                "universal_symbols_setup.intro_linux_fcitx5"
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        "universal_symbols_setup.intro_windows"
    }
    #[cfg(target_os = "macos")]
    {
        "universal_symbols_setup.intro_macos"
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "universal_symbols_setup.intro_unsupported"
    }
}

fn universal_symbols_backend_value_key() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        match crate::smart_input::linux_recommended_input_backend() {
            crate::smart_input::LinuxRecommendedInputBackend::X11Native => {
                "universal_symbols_setup.backend_linux_x11"
            }
            crate::smart_input::LinuxRecommendedInputBackend::IBus => {
                "universal_symbols_setup.backend_linux_ibus"
            }
            crate::smart_input::LinuxRecommendedInputBackend::Fcitx5 => {
                "universal_symbols_setup.backend_linux_fcitx5"
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        "universal_symbols_setup.backend_windows"
    }
    #[cfg(target_os = "macos")]
    {
        "universal_symbols_setup.backend_macos"
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "universal_symbols_setup.unsupported"
    }
}

fn universal_symbols_next_step_key() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        match crate::smart_input::linux_recommended_input_backend() {
            crate::smart_input::LinuxRecommendedInputBackend::X11Native => {
                "universal_symbols_setup.next_step_x11"
            }
            crate::smart_input::LinuxRecommendedInputBackend::IBus => {
                "universal_symbols_setup.next_step_ibus"
            }
            crate::smart_input::LinuxRecommendedInputBackend::Fcitx5 => {
                "universal_symbols_setup.next_step_fcitx5"
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        "universal_symbols_setup.next_step_windows"
    }
    #[cfg(target_os = "macos")]
    {
        "universal_symbols_setup.next_step_macos"
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "universal_symbols_setup.unsupported"
    }
}

fn universal_symbols_text_expander_key() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        match crate::smart_input::linux_recommended_input_backend() {
            crate::smart_input::LinuxRecommendedInputBackend::X11Native => {
                "universal_symbols_setup.text_expander_x11"
            }
            crate::smart_input::LinuxRecommendedInputBackend::IBus => {
                "universal_symbols_setup.text_expander_ibus"
            }
            crate::smart_input::LinuxRecommendedInputBackend::Fcitx5 => {
                "universal_symbols_setup.text_expander_fcitx5"
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        "universal_symbols_setup.text_expander_native"
    }
    #[cfg(target_os = "macos")]
    {
        "universal_symbols_setup.text_expander_native"
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "universal_symbols_setup.unsupported"
    }
}

#[cfg(target_os = "linux")]
fn linux_setup_success_status_key(script: &str, backend: &str) -> &'static str {
    if script.contains("uninstall") {
        "universal_symbols_setup.ibus_uninstalled_status"
    } else if backend == "Fcitx5" {
        "universal_symbols_setup.fcitx5_installed_status"
    } else {
        "universal_symbols_setup.ibus_installed_status"
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
