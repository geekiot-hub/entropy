use super::*;

impl EntropyApp {
    pub(super) fn text_expander_rule_issue(
        &self,
        idx: usize,
        rule: &crate::text_expander::TextExpansionRule,
    ) -> Option<&'static str> {
        if rule.trigger.is_empty() {
            return Some("text_expander.issue_empty_trigger");
        }
        if !crate::text_expander::valid_trigger(&rule.trigger) {
            return Some("text_expander.issue_invalid_trigger");
        }
        if crate::text_expander::prepare_replacement(&rule.replacement)
            .0
            .is_empty()
        {
            return Some("text_expander.issue_empty_replacement");
        }
        if self
            .app_settings
            .text_expansion_rules
            .iter()
            .enumerate()
            .any(|(other_idx, other)| other_idx != idx && other.trigger == rule.trigger)
        {
            return Some("text_expander.issue_duplicate_trigger");
        }
        None
    }

    pub(super) fn active_text_expansion_rules(
        &self,
    ) -> Vec<crate::text_expander::TextExpansionRule> {
        let mut rules = self.app_settings.text_expansion_rules.clone();
        rules.extend(load_extra_text_expansion_rules(
            &self.app_settings.text_expander_rule_files,
        ));
        rules
    }

    pub(super) fn sync_text_expander_runtime(&mut self) {
        crate::smart_input::set_text_expander_config(
            self.app_settings.text_expander_enabled,
            self.active_text_expansion_rules(),
            parse_text_expander_blacklist(&self.app_settings.text_expander_app_blacklist),
        );
    }

    pub(super) fn save_text_expander_settings(&mut self) {
        save_text_expansion_rules(&self.app_settings.text_expansion_rules);
        self.text_expander_rules_signature =
            text_expander_rules_signature(&self.app_settings.text_expander_rule_files);
        save_app_settings(&self.app_settings);
        self.sync_text_expander_runtime();
    }

    pub(super) fn add_text_expander_blacklist_app(&mut self, app_name: &str) -> bool {
        let Some(app_name) = normalize_text_expander_app_name(app_name) else {
            return false;
        };
        let mut entries =
            parse_text_expander_blacklist(&self.app_settings.text_expander_app_blacklist);
        if entries.iter().any(|entry| entry == &app_name) {
            return false;
        }
        entries.push(app_name);
        self.app_settings.text_expander_app_blacklist = format_text_expander_blacklist(&entries);
        self.save_text_expander_settings();
        true
    }

    pub(super) fn remove_text_expander_blacklist_app(&mut self, app_name: &str) -> bool {
        let Some(app_name) = normalize_text_expander_app_name(app_name) else {
            return false;
        };
        let mut entries =
            parse_text_expander_blacklist(&self.app_settings.text_expander_app_blacklist);
        let old_len = entries.len();
        entries.retain(|entry| entry != &app_name);
        if entries.len() == old_len {
            return false;
        }
        self.app_settings.text_expander_app_blacklist = format_text_expander_blacklist(&entries);
        self.save_text_expander_settings();
        true
    }

    pub(super) fn text_expander_window_candidates(
        &self,
        blacklist_entries: &[String],
    ) -> Vec<crate::smart_input::TextExpanderAppCandidate> {
        let mut candidates = Vec::new();
        for candidate in crate::smart_input::text_expander_app_candidates() {
            let Some(exe) = normalize_text_expander_app_name(&candidate.exe) else {
                continue;
            };
            if blacklist_entries.iter().any(|blocked| blocked == &exe) {
                continue;
            }
            if candidates
                .iter()
                .any(|existing: &crate::smart_input::TextExpanderAppCandidate| existing.exe == exe)
            {
                continue;
            }
            candidates.push(crate::smart_input::TextExpanderAppCandidate {
                exe,
                title: candidate.title,
            });
        }
        candidates.truncate(8);
        candidates
    }

    pub(super) fn reload_text_expander_rules_file(&mut self) -> bool {
        if let Some(rules) = load_text_expansion_rules() {
            self.app_settings.text_expansion_rules = rules;
            self.save_text_expander_settings();
            true
        } else {
            false
        }
    }

    pub(super) fn auto_reload_text_expander_rules_file(&mut self, now: f64) {
        if now - self.text_expander_rules_last_check_at < 1.0 {
            return;
        }
        self.text_expander_rules_last_check_at = now;
        let current_signature =
            text_expander_rules_signature(&self.app_settings.text_expander_rule_files);
        if current_signature != self.text_expander_rules_signature {
            if self.reload_text_expander_rules_file() {
                self.sync_text_expander_runtime();
                self.text_expander_rules_signature = current_signature;
                self.status_msg = crate::i18n::tr_catalog(
                    self.app_settings.language,
                    "text_expander.rules_auto_reloaded_status",
                )
                .to_owned();
            } else {
                self.status_msg = crate::i18n::tr_catalog(
                    self.app_settings.language,
                    "text_expander.rules_reload_failed_status",
                )
                .to_owned();
                self.text_expander_rules_signature = current_signature;
            }
        }
    }

    pub(super) fn open_text_expander_rules_folder(&mut self) {
        ensure_text_expander_rules_file(&self.app_settings.text_expansion_rules);
        let path = text_expander_rules_dir();
        let result = open_path_in_system_editor(&path);
        self.status_msg = if result {
            crate::i18n::tr_catalog(
                self.app_settings.language,
                "text_expander.rules_file_opened_status",
            )
            .to_owned()
        } else {
            crate::i18n::tr_catalog(
                self.app_settings.language,
                "text_expander.rules_file_open_failed_status",
            )
            .to_owned()
        };
    }

    pub(super) fn remove_text_expander_rules_file(&mut self, remove_idx: usize) {
        if remove_idx >= self.app_settings.text_expander_rule_files.len() {
            return;
        }
        self.app_settings
            .text_expander_rule_files
            .remove(remove_idx);
        self.text_expander_rules_signature =
            text_expander_rules_signature(&self.app_settings.text_expander_rule_files);
        save_app_settings(&self.app_settings);
        self.sync_text_expander_runtime();
    }

    pub(super) fn draw_text_expander_settings_page(
        &mut self,
        ui: &mut egui::Ui,
        content_rect: egui::Rect,
    ) {
        let lang = self.app_settings.language;
        let dark = ui.visuals().dark_mode;
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(metrics.value(18.0));
                ui.label(
                    RichText::new(crate::i18n::tr_catalog(lang, "text_expander.title"))
                        .size(metrics.value(18.0))
                        .strong(),
                );
                ui.add_space(metrics.value(6.0));
                ui.add_sized(
                    Vec2::new(metrics.settings_content_width(), metrics.value(34.0)),
                    egui::Label::new(
                        RichText::new(crate::i18n::tr_catalog(lang, "text_expander.description"))
                            .size(metrics.value(13.0))
                            .color(app_muted_text(dark)),
                    )
                    .wrap()
                    .halign(egui::Align::Center),
                );
                ui.add_sized(
                    Vec2::new(metrics.settings_content_width(), metrics.value(28.0)),
                    egui::Label::new(
                        RichText::new(crate::i18n::tr_catalog(lang, "text_expander.quick_help"))
                            .size(metrics.value(11.5))
                            .color(app_muted_text(dark)),
                    )
                    .wrap()
                    .halign(egui::Align::Center),
                );
                ui.add_space(metrics.value(10.0));

                let rule_row_count = self.app_settings.text_expansion_rules.len().max(1);
                let row_count = 4 + rule_row_count;
                let list = allocate_adaptive_settings_list_viewport(
                    ui,
                    "text_expander_settings",
                    metrics,
                    row_count,
                    metrics.value(44.0),
                );
                ui.allocate_ui_at_rect(list.content_rect, |ui| {
                    ui.set_clip_rect(list.viewport);
                    ui.set_min_size(list.content_rect.size());
                    ui.spacing_mut().item_spacing.y = 0.0;
                    self.draw_text_expander_editor_content(
                        ui,
                        list.first_visible_row..list.last_visible_row,
                        list.row_content_width,
                        list.row_height,
                        metrics,
                        list.suppress_tooltips,
                    );
                });

                if list.has_scrollbar {
                    crate::ui_style::paint_floating_scrollbar_handle(
                        ui,
                        list.track_rect,
                        list.handle_height,
                        list.scroll_ratio,
                        list.track_hovered,
                    );
                }

                let action_anchor_rows = responsive_settings_visible_rows(
                    ui.ctx(),
                    ui.available_height(),
                    6,
                    metrics.value(44.0),
                );
                let action_anchor_bottom =
                    list.viewport.top() + list.row_height * action_anchor_rows as f32;
                let button_size = metrics.size(126.0, 34.0);
                let button_gap = metrics.value(10.0);
                let actions_width = button_size.x * 2.0 + button_gap;
                let actions_rect = egui::Rect::from_center_size(
                    egui::pos2(
                        list.viewport.center().x,
                        action_anchor_bottom + metrics.value(26.0),
                    ),
                    egui::vec2(actions_width, button_size.y),
                );
                ui.allocate_ui_at_rect(actions_rect, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = button_gap;
                        if crate::ui_style::modern_button(
                            ui,
                            crate::i18n::tr_catalog(lang, "text_expander.add_rule"),
                            button_size,
                            true,
                        )
                        .on_hover_text(crate::i18n::tr_catalog(
                            lang,
                            "text_expander.add_rule_tooltip",
                        ))
                        .clicked()
                        {
                            self.app_settings
                                .text_expansion_rules
                                .push(crate::text_expander::TextExpansionRule::default());
                            self.save_text_expander_settings();
                        }

                        let restore_enabled = self.text_expander_deleted_rule.is_some();
                        if crate::ui_style::modern_button(
                            ui,
                            crate::i18n::tr_catalog(lang, "text_expander.restore_deleted_rule"),
                            button_size,
                            restore_enabled,
                        )
                        .on_hover_text(crate::i18n::tr_catalog(
                            lang,
                            "text_expander.restore_deleted_rule_tooltip",
                        ))
                        .clicked()
                            && restore_enabled
                        {
                            if let Some((rule_idx, rule)) = self.text_expander_deleted_rule.take() {
                                let insert_idx =
                                    rule_idx.min(self.app_settings.text_expansion_rules.len());
                                self.app_settings
                                    .text_expansion_rules
                                    .insert(insert_idx, rule);
                                self.save_text_expander_settings();
                            }
                        }
                    });
                });
            });
        });
    }

    pub(super) fn draw_text_expander_editor_content(
        &mut self,
        ui: &mut egui::Ui,
        row_range: std::ops::Range<usize>,
        content_width: f32,
        row_height: f32,
        metrics: crate::ui_style::ResponsiveMetrics,
        suppress_tooltips: bool,
    ) {
        let lang = self.app_settings.language;
        let switch_size = metrics.size(46.0, 24.0);
        let switch_width = metrics.value(46.0);
        let tooltip = |text: &'static str| (!suppress_tooltips).then_some(text);

        for row_idx in row_range {
            if row_idx == 0 {
                let mut enabled = self.app_settings.text_expander_enabled;
                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    content_width,
                    row_height,
                    crate::i18n::tr_catalog(lang, "text_expander.enabled_label"),
                    true,
                    tooltip(crate::i18n::tr_catalog(
                        lang,
                        "text_expander.enabled_tooltip",
                    )),
                    switch_width,
                    |ui| {
                        crate::ui_style::settings_switch_sized_stable(
                            ui,
                            "text_expander_enabled",
                            &mut enabled,
                            switch_size,
                        );
                    },
                );
                if enabled != self.app_settings.text_expander_enabled {
                    self.app_settings.text_expander_enabled = enabled;
                    self.save_text_expander_settings();
                }
                continue;
            }

            let blacklist_entries =
                parse_text_expander_blacklist(&self.app_settings.text_expander_app_blacklist);
            let window_candidates = self.text_expander_window_candidates(&blacklist_entries);
            if row_idx == 1 {
                let control_width = metrics.value(250.0);
                let mut add_app: Option<String> = None;
                let mut remove_app: Option<String> = None;
                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    content_width,
                    row_height,
                    crate::i18n::tr_catalog(lang, "text_expander.blacklist_label"),
                    true,
                    tooltip(crate::i18n::tr_catalog(
                        lang,
                        "text_expander.blacklist_tooltip",
                    )),
                    control_width,
                    |ui| {
                        let control_rect = ui.max_rect();
                        let dark = ui.visuals().dark_mode;
                        let chip_height = metrics.value(26.0);
                        let gap = metrics.value(6.0);
                        let add_width = metrics.value(44.0);
                        let more_width = metrics.value(42.0);
                        let visible_count = blacklist_entries.len().min(2);
                        let has_more = blacklist_entries.len() > visible_count;
                        let reserved_right =
                            add_width + if has_more { gap + more_width } else { 0.0 };
                        let chip_width = if visible_count > 0 {
                            ((control_width - reserved_right - gap * visible_count as f32)
                                / visible_count as f32)
                                .clamp(metrics.value(70.0), metrics.value(96.0))
                        } else {
                            0.0
                        };
                        let y = control_rect.center().y - chip_height / 2.0;
                        let mut x = control_rect.left();

                        if blacklist_entries.is_empty() {
                            let hint_rect = egui::Rect::from_min_max(
                                control_rect.left_top(),
                                egui::pos2(
                                    control_rect.right() - add_width - gap,
                                    control_rect.bottom(),
                                ),
                            );
                            ui.painter().text(
                                hint_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                crate::i18n::tr_catalog(lang, "text_expander.no_blacklist_apps"),
                                FontId::proportional(metrics.value(12.0)),
                                app_muted_text(dark),
                            );
                        }

                        for app_name in blacklist_entries.iter().take(visible_count) {
                            let display = if app_name.chars().count() > 14 {
                                format!("{}…", app_name.chars().take(13).collect::<String>())
                            } else {
                                app_name.clone()
                            };
                            let chip_rect = egui::Rect::from_min_size(
                                egui::pos2(x, y),
                                egui::vec2(chip_width, chip_height),
                            );
                            let resp = ui
                                .interact(
                                    chip_rect,
                                    ui.make_persistent_id((
                                        "text_expander_blacklist_chip",
                                        app_name,
                                    )),
                                    Sense::click(),
                                )
                                .on_hover_text(app_name.as_str());
                            if resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            let fill = if resp.hovered() {
                                crate::ui_style::hover_fill(dark)
                            } else {
                                crate::ui_style::surface_fill(dark)
                            };
                            ui.painter().rect(
                                chip_rect,
                                8.0,
                                fill,
                                crate::ui_style::modal_outline_stroke(dark),
                                egui::StrokeKind::Inside,
                            );
                            let text_clip_rect = egui::Rect::from_min_max(
                                egui::pos2(chip_rect.left() + metrics.value(7.0), chip_rect.top()),
                                egui::pos2(
                                    chip_rect.right() - metrics.value(24.0),
                                    chip_rect.bottom(),
                                ),
                            );
                            ui.painter().with_clip_rect(text_clip_rect).text(
                                egui::pos2(
                                    chip_rect.left() + metrics.value(9.0),
                                    chip_rect.center().y,
                                ),
                                egui::Align2::LEFT_CENTER,
                                display,
                                FontId::proportional(metrics.value(11.5)),
                                ui.visuals().text_color(),
                            );
                            ui.painter().text(
                                egui::pos2(
                                    chip_rect.right() - metrics.value(11.0),
                                    chip_rect.center().y,
                                ),
                                egui::Align2::CENTER_CENTER,
                                "×",
                                FontId::proportional(metrics.value(13.0)),
                                app_muted_text(dark),
                            );
                            if resp.clicked() {
                                remove_app = Some(app_name.clone());
                            }
                            x += chip_width + gap;
                        }

                        if has_more {
                            let remaining = blacklist_entries.len() - visible_count;
                            let more_rect = egui::Rect::from_min_size(
                                egui::pos2(control_rect.right() - add_width - gap - more_width, y),
                                egui::vec2(more_width, chip_height),
                            );
                            let popup_id =
                                ui.make_persistent_id("text_expander_blacklist_more_popup");
                            let more_resp = ui.interact(
                                more_rect,
                                ui.make_persistent_id("text_expander_blacklist_more_chip"),
                                Sense::click(),
                            );
                            if more_resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if more_resp.clicked() {
                                ui.memory_mut(|m| m.toggle_popup(popup_id));
                            }
                            let more_open = ui.memory(|m| m.is_popup_open(popup_id));
                            let fill = if more_resp.hovered() || more_open {
                                crate::ui_style::hover_fill(dark)
                            } else {
                                crate::ui_style::surface_fill(dark)
                            };
                            ui.painter().rect(
                                more_rect,
                                8.0,
                                fill,
                                crate::ui_style::modal_outline_stroke(dark),
                                egui::StrokeKind::Inside,
                            );
                            ui.painter().text(
                                more_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("+{remaining}"),
                                FontId::proportional(metrics.value(12.0)),
                                ui.visuals().text_color(),
                            );
                            egui::popup_below_widget(
                                ui,
                                popup_id,
                                &more_resp,
                                egui::PopupCloseBehavior::CloseOnClickOutside,
                                |ui| {
                                    ui.set_min_width(control_width);
                                    ui.set_max_width(control_width);
                                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 2.0);
                                    let option_height = metrics.value(30.0);
                                    let option_spacing = metrics.value(2.0);
                                    egui::ScrollArea::vertical()
                                        .max_height(compact_dropdown_popup_height(
                                            blacklist_entries.len(),
                                            option_height,
                                            option_spacing,
                                        ))
                                        .auto_shrink([false, true])
                                        .show(ui, |ui| {
                                            ui.set_min_width(control_width);
                                            for app_name in blacklist_entries.iter() {
                                                let display = if app_name.chars().count() > 28 {
                                                    format!(
                                                        "{}…",
                                                        app_name
                                                            .chars()
                                                            .take(27)
                                                            .collect::<String>()
                                                    )
                                                } else {
                                                    app_name.clone()
                                                };
                                                let (option_rect, option_resp) = ui
                                                    .allocate_exact_size(
                                                        egui::vec2(control_width, option_height),
                                                        Sense::click(),
                                                    );
                                                let option_resp =
                                                    option_resp.on_hover_text(app_name.as_str());
                                                if option_resp.hovered() {
                                                    ui.ctx().set_cursor_icon(
                                                        egui::CursorIcon::PointingHand,
                                                    );
                                                }
                                                let option_fill = if option_resp.hovered() {
                                                    crate::ui_style::hover_fill(dark)
                                                } else {
                                                    Color32::TRANSPARENT
                                                };
                                                ui.painter().rect_filled(
                                                    option_rect,
                                                    7.0,
                                                    option_fill,
                                                );
                                                let text_clip_rect = egui::Rect::from_min_max(
                                                    egui::pos2(
                                                        option_rect.left() + metrics.value(8.0),
                                                        option_rect.top(),
                                                    ),
                                                    egui::pos2(
                                                        option_rect.right() - metrics.value(28.0),
                                                        option_rect.bottom(),
                                                    ),
                                                );
                                                ui.painter().with_clip_rect(text_clip_rect).text(
                                                    egui::pos2(
                                                        option_rect.left() + metrics.value(10.0),
                                                        option_rect.center().y,
                                                    ),
                                                    egui::Align2::LEFT_CENTER,
                                                    display,
                                                    FontId::proportional(metrics.value(12.0)),
                                                    ui.visuals().text_color(),
                                                );
                                                ui.painter().text(
                                                    egui::pos2(
                                                        option_rect.right() - metrics.value(12.0),
                                                        option_rect.center().y,
                                                    ),
                                                    egui::Align2::CENTER_CENTER,
                                                    "×",
                                                    FontId::proportional(metrics.value(13.0)),
                                                    app_muted_text(dark),
                                                );
                                                if option_resp.clicked() {
                                                    remove_app = Some(app_name.clone());
                                                    ui.memory_mut(|m| m.close_popup());
                                                }
                                            }
                                        });
                                },
                            );
                        }

                        let add_rect = egui::Rect::from_min_size(
                            egui::pos2(control_rect.right() - add_width, y),
                            egui::vec2(add_width, chip_height),
                        );
                        let add_popup_id =
                            ui.make_persistent_id("text_expander_blacklist_add_window_popup");
                        let add_resp = ui
                            .interact(
                                add_rect,
                                ui.make_persistent_id("text_expander_blacklist_add_window_chip"),
                                Sense::click(),
                            )
                            .on_hover_text(crate::i18n::tr_catalog(
                                lang,
                                "text_expander.blacklist_add_tooltip",
                            ));
                        if add_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if add_resp.clicked() {
                            ui.memory_mut(|m| m.toggle_popup(add_popup_id));
                        }
                        let add_open = ui.memory(|m| m.is_popup_open(add_popup_id));
                        let fill = if add_resp.hovered() || add_open {
                            crate::ui_style::hover_fill(dark)
                        } else {
                            crate::ui_style::surface_fill(dark)
                        };
                        ui.painter().rect(
                            add_rect,
                            8.0,
                            fill,
                            crate::ui_style::modal_outline_stroke(dark),
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().text(
                            add_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "+",
                            FontId::proportional(metrics.value(15.0)),
                            ui.visuals().text_color(),
                        );
                        egui::popup_below_widget(
                            ui,
                            add_popup_id,
                            &add_resp,
                            egui::PopupCloseBehavior::CloseOnClickOutside,
                            |ui| {
                                ui.set_min_width(control_width);
                                ui.set_max_width(control_width);
                                ui.spacing_mut().item_spacing = egui::vec2(0.0, 2.0);
                                let option_height = metrics.value(34.0);
                                let option_spacing = metrics.value(2.0);
                                egui::ScrollArea::vertical()
                                    .max_height(compact_dropdown_popup_height(
                                        window_candidates.len(),
                                        option_height,
                                        option_spacing,
                                    ))
                                    .auto_shrink([false, true])
                                    .show(ui, |ui| {
                                        ui.set_min_width(control_width);
                                        if window_candidates.is_empty() {
                                            let (option_rect, _) = ui.allocate_exact_size(
                                                egui::vec2(control_width, option_height),
                                                Sense::hover(),
                                            );
                                            ui.painter().text(
                                                egui::pos2(
                                                    option_rect.left() + metrics.value(10.0),
                                                    option_rect.center().y,
                                                ),
                                                egui::Align2::LEFT_CENTER,
                                                crate::i18n::tr_catalog(
                                                    lang,
                                                    "text_expander.no_windows_hint",
                                                ),
                                                FontId::proportional(metrics.value(12.0)),
                                                app_muted_text(dark),
                                            );
                                        } else {
                                            for candidate in window_candidates.iter() {
                                                let title = if candidate.title.is_empty() {
                                                    candidate.exe.clone()
                                                } else {
                                                    format!(
                                                        "{} — {}",
                                                        candidate.title, candidate.exe
                                                    )
                                                };
                                                let display = if title.chars().count() > 30 {
                                                    format!(
                                                        "{}…",
                                                        title.chars().take(29).collect::<String>()
                                                    )
                                                } else {
                                                    title
                                                };
                                                let (option_rect, option_resp) = ui
                                                    .allocate_exact_size(
                                                        egui::vec2(control_width, option_height),
                                                        Sense::click(),
                                                    );
                                                if option_resp.hovered() {
                                                    ui.ctx().set_cursor_icon(
                                                        egui::CursorIcon::PointingHand,
                                                    );
                                                }
                                                let option_fill = if option_resp.hovered() {
                                                    crate::ui_style::hover_fill(dark)
                                                } else {
                                                    Color32::TRANSPARENT
                                                };
                                                ui.painter().rect_filled(
                                                    option_rect,
                                                    7.0,
                                                    option_fill,
                                                );
                                                ui.painter().text(
                                                    egui::pos2(
                                                        option_rect.left() + metrics.value(10.0),
                                                        option_rect.center().y,
                                                    ),
                                                    egui::Align2::LEFT_CENTER,
                                                    display,
                                                    FontId::proportional(metrics.value(12.0)),
                                                    ui.visuals().text_color(),
                                                );
                                                if option_resp.clicked() {
                                                    add_app = Some(candidate.exe.clone());
                                                    ui.memory_mut(|m| m.close_popup());
                                                }
                                            }
                                        }
                                    });
                            },
                        );
                    },
                );
                if let Some(app_name) = add_app {
                    self.add_text_expander_blacklist_app(&app_name);
                }
                if let Some(app_name) = remove_app {
                    self.remove_text_expander_blacklist_app(&app_name);
                }
                continue;
            }

            if row_idx == 2 {
                let button_width = metrics.value(118.0);
                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    content_width,
                    row_height,
                    crate::i18n::tr_catalog(lang, "text_expander.rules_file_label"),
                    true,
                    tooltip(crate::i18n::tr_catalog(
                        lang,
                        "text_expander.rules_file_tooltip",
                    )),
                    button_width,
                    |ui| {
                        if crate::ui_style::modern_button(
                            ui,
                            crate::i18n::tr_catalog(lang, "text_expander.open_rules_file"),
                            metrics.size(button_width, metrics.settings_control_height()),
                            true,
                        )
                        .clicked()
                        {
                            self.open_text_expander_rules_folder();
                        }
                    },
                );
                continue;
            }

            if row_idx == 3 {
                let control_width = metrics.value(250.0);
                let mut add_file: Option<String> = None;
                let mut remove_file: Option<usize> = None;
                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    content_width,
                    row_height,
                    crate::i18n::tr_catalog(lang, "text_expander.extra_rules_file_label"),
                    true,
                    tooltip(crate::i18n::tr_catalog(
                        lang,
                        "text_expander.extra_rules_file_select_tooltip",
                    )),
                    control_width,
                    |ui| {
                        let control_rect = ui.max_rect();
                        let dark = ui.visuals().dark_mode;
                        let chip_height = metrics.value(26.0);
                        let gap = metrics.value(6.0);
                        let add_width = metrics.value(44.0);
                        let selected_count = self.app_settings.text_expander_rule_files.len();
                        let visible_count = selected_count.min(2);
                        let has_more = selected_count > visible_count;
                        let more_width = metrics.value(42.0);
                        let reserved_right =
                            add_width + if has_more { gap + more_width } else { 0.0 };
                        let chip_width = if visible_count > 0 {
                            ((control_width - reserved_right - gap * visible_count as f32)
                                / visible_count as f32)
                                .clamp(metrics.value(64.0), metrics.value(92.0))
                        } else {
                            0.0
                        };
                        let y = control_rect.center().y - chip_height / 2.0;
                        let mut x = control_rect.left();

                        for (file_idx, file_name) in self
                            .app_settings
                            .text_expander_rule_files
                            .iter()
                            .take(visible_count)
                            .enumerate()
                        {
                            let file_ok = load_text_expansion_rules_from_path(
                                &text_expander_extra_rules_path(file_name),
                            )
                            .is_some();
                            let display = if file_name.chars().count() > 12 {
                                format!("{}…", file_name.chars().take(11).collect::<String>())
                            } else {
                                file_name.clone()
                            };
                            let chip_rect = egui::Rect::from_min_size(
                                egui::pos2(x, y),
                                egui::vec2(chip_width, chip_height),
                            );
                            let resp = ui.interact(
                                chip_rect,
                                ui.make_persistent_id(("text_expander_rules_file_chip", file_name)),
                                Sense::click(),
                            );
                            if resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            let fill = if resp.hovered() {
                                crate::ui_style::hover_fill(dark)
                            } else {
                                crate::ui_style::surface_fill(dark)
                            };
                            ui.painter().rect(
                                chip_rect,
                                8.0,
                                fill,
                                crate::ui_style::modal_outline_stroke(dark),
                                egui::StrokeKind::Inside,
                            );
                            let prefix = if file_ok { "✓ " } else { "⚠ " };
                            ui.painter().text(
                                egui::pos2(
                                    chip_rect.left() + metrics.value(8.0),
                                    chip_rect.center().y,
                                ),
                                egui::Align2::LEFT_CENTER,
                                format!("{prefix}{display}"),
                                FontId::proportional(metrics.value(11.0)),
                                ui.visuals().text_color(),
                            );
                            ui.painter().text(
                                egui::pos2(
                                    chip_rect.right() - metrics.value(10.0),
                                    chip_rect.center().y,
                                ),
                                egui::Align2::CENTER_CENTER,
                                "×",
                                FontId::proportional(metrics.value(13.0)),
                                app_muted_text(dark),
                            );
                            if resp.clicked() {
                                remove_file = Some(file_idx);
                            }
                            x += chip_width + gap;
                        }

                        if has_more {
                            let remaining = selected_count - visible_count;
                            let more_rect = egui::Rect::from_min_size(
                                egui::pos2(control_rect.right() - add_width - gap - more_width, y),
                                egui::vec2(more_width, chip_height),
                            );
                            let popup_id =
                                ui.make_persistent_id("text_expander_rules_files_more_popup");
                            let more_resp = ui.interact(
                                more_rect,
                                ui.make_persistent_id("text_expander_rules_files_more_chip"),
                                Sense::click(),
                            );
                            if more_resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if more_resp.clicked() {
                                ui.memory_mut(|m| m.toggle_popup(popup_id));
                            }
                            let more_open = ui.memory(|m| m.is_popup_open(popup_id));
                            let fill = if more_resp.hovered() || more_open {
                                crate::ui_style::hover_fill(dark)
                            } else {
                                crate::ui_style::surface_fill(dark)
                            };
                            ui.painter().rect(
                                more_rect,
                                8.0,
                                fill,
                                crate::ui_style::modal_outline_stroke(dark),
                                egui::StrokeKind::Inside,
                            );
                            ui.painter().text(
                                more_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("+{remaining}"),
                                FontId::proportional(metrics.value(12.0)),
                                ui.visuals().text_color(),
                            );
                            egui::popup_below_widget(
                                ui,
                                popup_id,
                                &more_resp,
                                egui::PopupCloseBehavior::CloseOnClickOutside,
                                |ui| {
                                    ui.set_min_width(control_width);
                                    ui.set_max_width(control_width);
                                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 2.0);
                                    let option_height = metrics.value(30.0);
                                    let option_spacing = metrics.value(2.0);
                                    egui::ScrollArea::vertical()
                                        .max_height(compact_dropdown_popup_height(
                                            selected_count,
                                            option_height,
                                            option_spacing,
                                        ))
                                        .auto_shrink([false, true])
                                        .show(ui, |ui| {
                                            ui.set_min_width(control_width);
                                            for (file_idx, file_name) in self
                                                .app_settings
                                                .text_expander_rule_files
                                                .iter()
                                                .enumerate()
                                            {
                                                let file_ok = load_text_expansion_rules_from_path(
                                                    &text_expander_extra_rules_path(file_name),
                                                )
                                                .is_some();
                                                let display = if file_name.chars().count() > 28 {
                                                    format!(
                                                        "{}…",
                                                        file_name
                                                            .chars()
                                                            .take(27)
                                                            .collect::<String>()
                                                    )
                                                } else {
                                                    file_name.clone()
                                                };
                                                let (option_rect, option_resp) = ui
                                                    .allocate_exact_size(
                                                        egui::vec2(control_width, option_height),
                                                        Sense::click(),
                                                    );
                                                if option_resp.hovered() {
                                                    ui.ctx().set_cursor_icon(
                                                        egui::CursorIcon::PointingHand,
                                                    );
                                                }
                                                let option_fill = if option_resp.hovered() {
                                                    crate::ui_style::hover_fill(dark)
                                                } else {
                                                    Color32::TRANSPARENT
                                                };
                                                ui.painter().rect_filled(
                                                    option_rect,
                                                    7.0,
                                                    option_fill,
                                                );
                                                let prefix = if file_ok { "✓ " } else { "⚠ " };
                                                ui.painter().text(
                                                    egui::pos2(
                                                        option_rect.left() + metrics.value(10.0),
                                                        option_rect.center().y,
                                                    ),
                                                    egui::Align2::LEFT_CENTER,
                                                    format!("{prefix}{display}"),
                                                    FontId::proportional(metrics.value(12.0)),
                                                    ui.visuals().text_color(),
                                                );
                                                ui.painter().text(
                                                    egui::pos2(
                                                        option_rect.right() - metrics.value(12.0),
                                                        option_rect.center().y,
                                                    ),
                                                    egui::Align2::CENTER_CENTER,
                                                    "×",
                                                    FontId::proportional(metrics.value(13.0)),
                                                    app_muted_text(dark),
                                                );
                                                if option_resp.clicked() {
                                                    remove_file = Some(file_idx);
                                                    ui.memory_mut(|m| m.close_popup());
                                                }
                                            }
                                        });
                                },
                            );
                        }

                        let add_rect = egui::Rect::from_min_size(
                            egui::pos2(control_rect.right() - add_width, y),
                            egui::vec2(add_width, chip_height),
                        );
                        let add_popup_id =
                            ui.make_persistent_id("text_expander_rules_files_add_popup");
                        let add_resp = ui
                            .interact(
                                add_rect,
                                ui.make_persistent_id("text_expander_rules_files_add_chip"),
                                Sense::click(),
                            )
                            .on_hover_text(crate::i18n::tr_catalog(
                                lang,
                                "text_expander.extra_rules_file_add_tooltip",
                            ));
                        if add_resp.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                        }
                        if add_resp.clicked() {
                            ui.memory_mut(|m| m.toggle_popup(add_popup_id));
                        }
                        let add_open = ui.memory(|m| m.is_popup_open(add_popup_id));
                        let fill = if add_resp.hovered() || add_open {
                            crate::ui_style::hover_fill(dark)
                        } else {
                            crate::ui_style::surface_fill(dark)
                        };
                        ui.painter().rect(
                            add_rect,
                            8.0,
                            fill,
                            crate::ui_style::modal_outline_stroke(dark),
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().text(
                            add_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "+",
                            FontId::proportional(metrics.value(15.0)),
                            ui.visuals().text_color(),
                        );
                        egui::popup_below_widget(
                            ui,
                            add_popup_id,
                            &add_resp,
                            egui::PopupCloseBehavior::CloseOnClickOutside,
                            |ui| {
                                let selected_files =
                                    self.app_settings.text_expander_rule_files.clone();
                                let options =
                                    text_expander_available_extra_rule_files(&selected_files);
                                ui.set_min_width(control_width);
                                ui.set_max_width(control_width);
                                ui.spacing_mut().item_spacing = egui::vec2(0.0, 2.0);
                                let option_height = metrics.value(30.0);
                                let option_spacing = metrics.value(2.0);
                                egui::ScrollArea::vertical()
                                    .max_height(compact_dropdown_popup_height(
                                        options.len(),
                                        option_height,
                                        option_spacing,
                                    ))
                                    .auto_shrink([false, true])
                                    .show(ui, |ui| {
                                        ui.set_min_width(control_width);
                                        if options.is_empty() {
                                            let (option_rect, _) = ui.allocate_exact_size(
                                                egui::vec2(control_width, option_height),
                                                Sense::hover(),
                                            );
                                            ui.painter().text(
                                                egui::pos2(
                                                    option_rect.left() + metrics.value(10.0),
                                                    option_rect.center().y,
                                                ),
                                                egui::Align2::LEFT_CENTER,
                                                crate::i18n::tr_catalog(
                                                    lang,
                                                    "text_expander.no_extra_rules_files",
                                                ),
                                                FontId::proportional(metrics.value(12.0)),
                                                app_muted_text(dark),
                                            );
                                        } else {
                                            for option in options.iter() {
                                                let display = if option.chars().count() > 28 {
                                                    format!(
                                                        "{}…",
                                                        option.chars().take(27).collect::<String>()
                                                    )
                                                } else {
                                                    option.clone()
                                                };
                                                let (option_rect, option_resp) = ui
                                                    .allocate_exact_size(
                                                        egui::vec2(control_width, option_height),
                                                        Sense::click(),
                                                    );
                                                if option_resp.hovered() {
                                                    ui.ctx().set_cursor_icon(
                                                        egui::CursorIcon::PointingHand,
                                                    );
                                                }
                                                let option_fill = if option_resp.hovered() {
                                                    crate::ui_style::hover_fill(dark)
                                                } else {
                                                    Color32::TRANSPARENT
                                                };
                                                ui.painter().rect_filled(
                                                    option_rect,
                                                    7.0,
                                                    option_fill,
                                                );
                                                ui.painter().text(
                                                    egui::pos2(
                                                        option_rect.left() + metrics.value(10.0),
                                                        option_rect.center().y,
                                                    ),
                                                    egui::Align2::LEFT_CENTER,
                                                    display,
                                                    FontId::proportional(metrics.value(12.0)),
                                                    ui.visuals().text_color(),
                                                );
                                                if option_resp.clicked() {
                                                    add_file = Some(option.clone());
                                                    ui.memory_mut(|m| m.close_popup());
                                                }
                                            }
                                        }
                                    });
                            },
                        );
                    },
                );
                if let Some(file_name) = add_file {
                    self.app_settings.text_expander_rule_files.push(file_name);
                    self.text_expander_rules_signature =
                        text_expander_rules_signature(&self.app_settings.text_expander_rule_files);
                    save_app_settings(&self.app_settings);
                    self.sync_text_expander_runtime();
                }
                if let Some(file_idx) = remove_file {
                    self.remove_text_expander_rules_file(file_idx);
                }
                continue;
            }

            let rules_start_row = 4;
            let idx = row_idx - rules_start_row;
            if self.app_settings.text_expansion_rules.is_empty() {
                let control_width = metrics.value(250.0);
                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    content_width,
                    row_height,
                    crate::i18n::tr_catalog(lang, "text_expander.empty_rules_label"),
                    true,
                    tooltip(crate::i18n::tr_catalog(
                        lang,
                        "text_expander.empty_rules_tooltip",
                    )),
                    control_width,
                    |ui| {
                        let rect = ui.max_rect();
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            crate::i18n::tr_catalog(lang, "text_expander.empty_rules_hint"),
                            FontId::proportional(metrics.value(12.0)),
                            app_muted_text(ui.visuals().dark_mode),
                        );
                    },
                );
                continue;
            }
            let Some(original_rule) = self.app_settings.text_expansion_rules.get(idx).cloned()
            else {
                continue;
            };
            let mut rule = original_rule.clone();
            let mut delete_rule = false;
            let mut changed = false;
            let issue = self.text_expander_rule_issue(idx, &rule);
            let label = format!(
                "{}{} {}",
                if issue.is_some() { "⚠ " } else { "" },
                crate::i18n::tr_catalog(lang, "text_expander.rule_label"),
                idx + 1
            );
            let control_width = metrics.value(344.0);
            let rule_tooltip = issue
                .map(|key| crate::i18n::tr_catalog(lang, key))
                .unwrap_or_else(|| crate::i18n::tr_catalog(lang, "text_expander.rule_tooltip"));
            crate::ui_style::settings_list_row_with_tooltip(
                ui,
                content_width,
                row_height,
                label.as_str(),
                true,
                tooltip(rule_tooltip),
                control_width,
                |ui| {
                    let control_rect = ui.max_rect();
                    let field_height = metrics.settings_control_height();
                    let switch_size = metrics.size(34.0, 20.0);
                    let trigger_width = metrics.value(82.0);
                    let delete_size = metrics.size(30.0, field_height);
                    let gap = metrics.value(8.0);
                    let switch_gap = metrics.value(8.0);
                    let replacement_width = (control_width
                        - switch_size.x
                        - switch_gap
                        - trigger_width
                        - gap
                        - delete_size.x
                        - gap)
                        .max(metrics.value(120.0));
                    let top = control_rect.center().y - field_height / 2.0;
                    let switch_top = control_rect.center().y - switch_size.y / 2.0;
                    let mut x = control_rect.left();

                    let switch_rect =
                        egui::Rect::from_min_size(egui::pos2(x, switch_top), switch_size);
                    x += switch_size.x + switch_gap;
                    let trigger_rect = egui::Rect::from_min_size(
                        egui::pos2(x, top),
                        egui::vec2(trigger_width, field_height),
                    );
                    x += trigger_width + gap;
                    let replacement_rect = egui::Rect::from_min_size(
                        egui::pos2(x, top),
                        egui::vec2(replacement_width, field_height),
                    );
                    let delete_rect = egui::Rect::from_min_size(
                        egui::pos2(control_rect.right() - delete_size.x, top),
                        delete_size,
                    );

                    let mut rule_enabled = rule.enabled;
                    let mut switch_resp = None;
                    ui.allocate_ui_at_rect(switch_rect, |ui| {
                        switch_resp = Some(crate::ui_style::settings_switch_sized_stable(
                            ui,
                            ("text_expander_rule_enabled", idx),
                            &mut rule_enabled,
                            switch_size,
                        ));
                    });
                    if switch_resp.is_some_and(|resp| resp.changed()) {
                        rule.enabled = rule_enabled;
                        changed = true;
                    }

                    let mut trigger_resp = None;
                    ui.allocate_ui_at_rect(trigger_rect, |ui| {
                        trigger_resp = Some(
                            crate::ui_style::modern_text_field_sized(
                                ui,
                                ui.make_persistent_id(("text_expander_trigger", idx)),
                                &mut rule.trigger,
                                trigger_width,
                                field_height,
                                crate::i18n::tr_catalog(lang, "text_expander.trigger_hint"),
                                32,
                                egui::Align::Center,
                            )
                            .on_hover_text(crate::i18n::tr_catalog(
                                lang,
                                "text_expander.trigger_tooltip",
                            )),
                        );
                    });
                    if trigger_resp.is_some_and(|resp| resp.changed()) {
                        changed = true;
                    }

                    let mut replacement_resp = None;
                    ui.allocate_ui_at_rect(replacement_rect, |ui| {
                        replacement_resp = Some(
                            crate::ui_style::modern_text_field_sized(
                                ui,
                                ui.make_persistent_id(("text_expander_replacement", idx)),
                                &mut rule.replacement,
                                replacement_width,
                                field_height,
                                crate::i18n::tr_catalog(lang, "text_expander.replacement_hint"),
                                480,
                                egui::Align::Min,
                            )
                            .on_hover_text(crate::i18n::tr_catalog(
                                lang,
                                "text_expander.replacement_tooltip",
                            )),
                        );
                    });
                    if replacement_resp.is_some_and(|resp| resp.changed()) {
                        changed = true;
                    }

                    let mut delete_clicked = false;
                    ui.allocate_ui_at_rect(delete_rect, |ui| {
                        delete_clicked =
                            crate::ui_style::modern_button(ui, "×", delete_size, true).clicked();
                    });
                    if delete_clicked {
                        delete_rule = true;
                    }
                },
            );

            if delete_rule {
                let removed_rule = self.app_settings.text_expansion_rules.remove(idx);
                self.text_expander_deleted_rule = Some((idx, removed_rule));
                self.save_text_expander_settings();
            } else if changed && rule != original_rule {
                if let Some(stored_rule) = self.app_settings.text_expansion_rules.get_mut(idx) {
                    *stored_rule = rule;
                }
                self.save_text_expander_settings();
            }
        }
    }
}
