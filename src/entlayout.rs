use super::*;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const ENTLAYOUT_FORMAT: &str = "entropy.layout";
const ENTLAYOUT_VERSION: u16 = 1;

#[derive(Clone, Serialize, Deserialize)]
struct EntLayoutFile {
    format: String,
    version: u16,
    created_by: String,
    created_at: String,
    keyboard: EntLayoutKeyboard,
    data: EntLayoutData,
}

#[derive(Clone, Serialize, Deserialize)]
struct EntLayoutKeyboard {
    name: String,
    firmware: FirmwareProtocol,
    keyboard_id: Option<u64>,
    layout_hash: String,
    layers: usize,
    keys: usize,
    encoders: usize,
    rows: usize,
    cols: usize,
}

#[derive(Clone, Serialize, Deserialize)]
struct EntLayoutData {
    keymap: Vec<Vec<u16>>,
    encoder_keymap: Vec<Vec<u16>>,
    encoder_visibility: Vec<bool>,
    layout_options: Option<u32>,
    layer_names: Vec<String>,
    app_settings: AppSettings,
    text_expander: EntTextExpanderData,
    macros: EntMacroData,
    combos: EntComboData,
    tap_dance: EntTapDanceData,
    key_overrides: EntKeyOverrideData,
    alt_repeat: EntAltRepeatData,
}

#[derive(Clone, Serialize, Deserialize)]
struct EntTextExpanderData {
    enabled: bool,
    app_blacklist: String,
    rule_files: Vec<String>,
    primary_rules: Vec<crate::text_expander::TextExpansionRule>,
    extra_files: Vec<EntTextExpanderRuleFile>,
}

#[derive(Clone, Serialize, Deserialize)]
struct EntTextExpanderRuleFile {
    file_name: String,
    rules: Vec<crate::text_expander::TextExpansionRule>,
}

#[derive(Clone, Serialize, Deserialize)]
struct EntMacroData {
    texts: Vec<String>,
    names: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct EntComboData {
    entries: Vec<EntComboEntry>,
    names: Vec<String>,
    term: Option<u16>,
}

#[derive(Clone, Serialize, Deserialize)]
struct EntComboEntry {
    keys: [u16; 4],
    output: u16,
}

#[derive(Clone, Serialize, Deserialize)]
struct EntTapDanceData {
    entries: Vec<EntTapDanceEntry>,
    names: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct EntTapDanceEntry {
    on_tap: u16,
    on_hold: u16,
    on_double_tap: u16,
    on_tap_hold: u16,
    tapping_term: u16,
}

#[derive(Clone, Serialize, Deserialize)]
struct EntKeyOverrideData {
    entries: Vec<EntKeyOverrideEntry>,
    names: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct EntKeyOverrideEntry {
    trigger: u16,
    replacement: u16,
    layers: u16,
    trigger_mods: u8,
    negative_mod_mask: u8,
    suppressed_mods: u8,
    options: u8,
}

#[derive(Clone, Serialize, Deserialize)]
struct EntAltRepeatData {
    entries: Vec<EntAltRepeatEntry>,
    names: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct EntAltRepeatEntry {
    keycode: u16,
    alt_keycode: u16,
    allowed_mods: u8,
    options: u8,
}

impl EntropyApp {
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn export_entlayout_dialog(&mut self) {
        let Some(bundle) = self.entlayout_snapshot() else {
            self.status_msg = "Connect a keyboard before exporting layout".into();
            return;
        };
        let file_name = format!("{}.entlayout", device_id_slug(&bundle.keyboard.name));
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Entropy layout", &["entlayout"])
            .set_file_name(&file_name)
            .save_file()
        else {
            return;
        };
        match write_entlayout_file(&path, &bundle) {
            Ok(()) => self.status_msg = format!("Exported layout: {}", path.display()),
            Err(e) => self.status_msg = format!("Export failed: {e}"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn import_entlayout_dialog(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Entropy layout", &["entlayout"])
            .pick_file()
        else {
            return;
        };
        match self.import_entlayout_from_path(&path) {
            Ok(report) => self.status_msg = report,
            Err(e) => self.status_msg = format!("Import failed: {e}"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn import_entlayout_from_path(&mut self, path: &Path) -> Result<String> {
        let data = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let bundle: EntLayoutFile =
            serde_json::from_str(&data).context("invalid .entlayout JSON")?;
        self.validate_entlayout_file(&bundle)?;
        let backup_path = self.write_entlayout_auto_backup()?;
        let firmware_failures = self.apply_entlayout(&bundle)?;
        Ok(self.entlayout_import_report(&bundle, &backup_path, &firmware_failures))
    }

    fn entlayout_snapshot(&self) -> Option<EntLayoutFile> {
        let layout = self.layout.as_ref()?;
        let keyboard = EntLayoutKeyboard {
            name: self.current_device_name_if_known(layout),
            firmware: self.firmware,
            keyboard_id: self.current_keyboard_id,
            layout_hash: entlayout_hash(layout),
            layers: layout.layers.len(),
            keys: layout.keys.len(),
            encoders: layout.encoder_count(),
            rows: layout.rows,
            cols: layout.cols,
        };
        Some(EntLayoutFile {
            format: ENTLAYOUT_FORMAT.to_owned(),
            version: ENTLAYOUT_VERSION,
            created_by: "Entropy".to_owned(),
            created_at: chrono::Local::now().to_rfc3339(),
            keyboard,
            data: EntLayoutData {
                keymap: layout.layers.clone(),
                encoder_keymap: layout.encoder_layers.clone(),
                encoder_visibility: self.encoder_visibility.clone(),
                layout_options: self.layout_options_value,
                layer_names: self.layer_names.clone(),
                app_settings: self.app_settings.clone(),
                text_expander: self.text_expander_entlayout_snapshot(),
                macros: EntMacroData {
                    texts: self.keycode_picker.macro_texts.clone(),
                    names: self.keycode_picker.macro_names.clone(),
                },
                combos: EntComboData {
                    entries: self
                        .combo_entries
                        .iter()
                        .map(|entry| EntComboEntry {
                            keys: entry.keys,
                            output: entry.output,
                        })
                        .collect(),
                    names: self.combo_names.clone(),
                    term: self.combo_term,
                },
                tap_dance: EntTapDanceData {
                    entries: self
                        .keycode_picker
                        .tap_dance_entries
                        .iter()
                        .map(|entry| EntTapDanceEntry {
                            on_tap: entry.on_tap,
                            on_hold: entry.on_hold,
                            on_double_tap: entry.on_double_tap,
                            on_tap_hold: entry.on_tap_hold,
                            tapping_term: entry.tapping_term,
                        })
                        .collect(),
                    names: self.keycode_picker.tap_dance_names.clone(),
                },
                key_overrides: EntKeyOverrideData {
                    entries: self
                        .key_override_entries
                        .iter()
                        .map(|entry| EntKeyOverrideEntry {
                            trigger: entry.trigger,
                            replacement: entry.replacement,
                            layers: entry.layers,
                            trigger_mods: entry.trigger_mods,
                            negative_mod_mask: entry.negative_mod_mask,
                            suppressed_mods: entry.suppressed_mods,
                            options: entry.options.bits(),
                        })
                        .collect(),
                    names: self.key_override_names.clone(),
                },
                alt_repeat: EntAltRepeatData {
                    entries: self
                        .alt_repeat_entries
                        .iter()
                        .map(|entry| EntAltRepeatEntry {
                            keycode: entry.keycode,
                            alt_keycode: entry.alt_keycode,
                            allowed_mods: entry.allowed_mods,
                            options: entry.options.bits(),
                        })
                        .collect(),
                    names: self.alt_repeat_names.clone(),
                },
            },
        })
    }

    fn text_expander_entlayout_snapshot(&self) -> EntTextExpanderData {
        let rule_files =
            normalize_text_expander_rule_files(&self.app_settings.text_expander_rule_files);
        let extra_files = rule_files
            .iter()
            .filter_map(|file_name| {
                load_text_expansion_rules_from_path(&text_expander_extra_rules_path(file_name)).map(
                    |rules| EntTextExpanderRuleFile {
                        file_name: file_name.clone(),
                        rules,
                    },
                )
            })
            .collect();
        EntTextExpanderData {
            enabled: self.app_settings.text_expander_enabled,
            app_blacklist: self.app_settings.text_expander_app_blacklist.clone(),
            rule_files,
            primary_rules: self.app_settings.text_expansion_rules.clone(),
            extra_files,
        }
    }

    fn current_device_name_if_known(&self, layout: &KeyboardLayout) -> String {
        if self.current_device_name.is_empty() {
            layout.name.clone()
        } else {
            self.current_device_name.clone()
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn write_entlayout_auto_backup(&self) -> Result<PathBuf> {
        let bundle = self
            .entlayout_snapshot()
            .context("cannot create auto-backup without a connected keyboard")?;
        let dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("entropy")
            .join("backups");
        std::fs::create_dir_all(&dir).context("failed to create backup directory")?;
        let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let path = dir.join(format!(
            "auto-backup-{}-{stamp}.entlayout",
            device_id_slug(&bundle.keyboard.name)
        ));
        write_entlayout_file(&path, &bundle)?;
        Ok(path)
    }

    fn validate_entlayout_file(&self, bundle: &EntLayoutFile) -> Result<()> {
        if bundle.format != ENTLAYOUT_FORMAT {
            bail!("unsupported format: {}", bundle.format);
        }
        if bundle.version != ENTLAYOUT_VERSION {
            bail!("unsupported .entlayout version: {}", bundle.version);
        }
        let layout = self
            .layout
            .as_ref()
            .context("connect the target keyboard before importing")?;
        if let (Some(file_id), Some(current_id)) =
            (bundle.keyboard.keyboard_id, self.current_keyboard_id)
        {
            if file_id != current_id {
                bail!("keyboard id mismatch: file {file_id:016x}, connected {current_id:016x}");
            }
        }
        let current_hash = entlayout_hash(layout);
        if bundle.keyboard.layout_hash != current_hash {
            bail!("layout hash mismatch: this file belongs to a different layout definition");
        }
        if bundle.keyboard.layers != layout.layers.len()
            || bundle.keyboard.keys != layout.keys.len()
            || bundle.keyboard.encoders != layout.encoder_count()
        {
            bail!("layout dimensions mismatch");
        }
        if bundle.data.keymap.len() != layout.layers.len()
            || bundle
                .data
                .keymap
                .iter()
                .any(|layer| layer.len() != layout.keys.len())
        {
            bail!("keymap size in file does not match connected keyboard");
        }
        if bundle.data.encoder_keymap.len() != layout.layers.len()
            || bundle
                .data
                .encoder_keymap
                .iter()
                .any(|layer| layer.len() != layout.encoders.len())
        {
            bail!("encoder keymap size in file does not match connected keyboard");
        }
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn apply_entlayout(&mut self, bundle: &EntLayoutFile) -> Result<Vec<String>> {
        let firmware_failures = self.apply_entlayout_firmware_state(bundle)?;
        self.apply_entlayout_local_state(bundle)?;
        self.refresh_layer_picker_content_flags();
        Ok(firmware_failures)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn entlayout_import_report(
        &self,
        bundle: &EntLayoutFile,
        backup_path: &Path,
        firmware_failures: &[String],
    ) -> String {
        let mut imported = vec![
            "keymap",
            "encoder keymap",
            "encoder visibility",
            "layer names",
            "app settings",
            "Text Expander",
        ];
        let mut skipped = Vec::new();

        if bundle.data.layout_options.is_some() && self.layout_options_value.is_some() {
            imported.push("layout options");
        } else if bundle.data.layout_options.is_some() {
            skipped.push("layout options (unsupported by connected keyboard)");
        }

        if !bundle.data.macros.texts.is_empty() && self.keycode_picker.macro_count > 0 {
            imported.push("macros");
        } else {
            skipped.push("macros (not available)");
        }

        if !bundle.data.combos.entries.is_empty() && !self.combo_entries.is_empty() {
            imported.push("combos");
        } else {
            skipped.push("combos (not available)");
        }

        if !bundle.data.tap_dance.entries.is_empty()
            && !self.keycode_picker.tap_dance_entries.is_empty()
        {
            imported.push("tap dance");
        } else {
            skipped.push("tap dance (not available)");
        }

        if !bundle.data.key_overrides.entries.is_empty() && !self.key_override_entries.is_empty() {
            imported.push("key overrides");
        } else {
            skipped.push("key overrides (not available)");
        }

        if !bundle.data.alt_repeat.entries.is_empty() && !self.alt_repeat_entries.is_empty() {
            imported.push("alt repeat");
        } else {
            skipped.push("alt repeat (not available)");
        }

        let skipped = if skipped.is_empty() {
            "none".to_owned()
        } else {
            skipped.join(", ")
        };
        let firmware_failures = if firmware_failures.is_empty() {
            "none".to_owned()
        } else {
            firmware_failures.join(", ")
        };
        format!(
            "Imported .entlayout: {}. Skipped: {}. Firmware failures: {}. Auto-backup: {}",
            imported.join(", "),
            skipped,
            firmware_failures,
            backup_path.display()
        )
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn apply_entlayout_firmware_state(&mut self, bundle: &EntLayoutFile) -> Result<Vec<String>> {
        let Some(hid) = &self.hid_device else {
            bail!("no active keyboard connection for firmware import");
        };
        let layout = self.layout.as_ref().context("no connected layout")?.clone();
        let mut failures = Vec::new();

        if let Err(err) = (|| -> Result<()> {
            for (layer_idx, layer_codes) in bundle.data.keymap.iter().enumerate() {
                for (key_idx, keycode) in layer_codes.iter().copied().enumerate() {
                    let key = &layout.keys[key_idx];
                    hid.set_keycode(layer_idx as u8, key.row, key.col, keycode)?;
                }
            }
            Ok(())
        })() {
            failures.push(format!("keymap ({err})"));
        }

        if let Err(err) = (|| -> Result<()> {
            for (layer_idx, layer_codes) in bundle.data.encoder_keymap.iter().enumerate() {
                for (visual_idx, keycode) in layer_codes.iter().copied().enumerate() {
                    let encoder = &layout.encoders[visual_idx];
                    hid.set_encoder(
                        layer_idx as u8,
                        encoder.encoder_idx,
                        encoder.direction,
                        keycode,
                    )?;
                }
            }
            Ok(())
        })() {
            failures.push(format!("encoder keymap ({err})"));
        }

        if let Some(options) = bundle.data.layout_options {
            if self.layout_options_value.is_some() {
                if let Err(err) = hid.set_layout_options(options) {
                    failures.push(format!("layout options ({err})"));
                }
            }
        }

        if !bundle.data.macros.texts.is_empty() && self.keycode_picker.macro_count > 0 {
            if let Err(err) = (|| -> Result<()> {
                let size = hid.get_macro_buffer_size()?;
                let mut macro_texts = bundle.data.macros.texts.clone();
                macro_texts.resize(self.keycode_picker.macro_count, String::new());
                macro_texts.truncate(self.keycode_picker.macro_count);
                let buf = crate::hid::HidDevice::encode_macros(&macro_texts, size);
                hid.set_macro_buffer(&buf)?;
                Ok(())
            })() {
                failures.push(format!("macros ({err})"));
            }
        }

        if let Err(err) = (|| -> Result<()> {
            for (idx, combo) in bundle
                .data
                .combos
                .entries
                .iter()
                .take(self.combo_entries.len())
                .enumerate()
            {
                hid.set_combo(idx as u8, combo.keys, combo.output)?;
            }
            Ok(())
        })() {
            failures.push(format!("combos ({err})"));
        }
        if let Some(term) = bundle.data.combos.term {
            if self.combo_term.is_some() {
                if let Err(err) = hid.set_qmk_setting_u16(2, term) {
                    failures.push(format!("combo term ({err})"));
                }
            }
        }

        if let Err(err) = (|| -> Result<()> {
            for (idx, td) in bundle
                .data
                .tap_dance
                .entries
                .iter()
                .take(self.keycode_picker.tap_dance_entries.len())
                .enumerate()
            {
                hid.set_tap_dance(
                    idx as u8,
                    td.on_tap,
                    td.on_hold,
                    td.on_double_tap,
                    td.on_tap_hold,
                    td.tapping_term,
                )?;
            }
            Ok(())
        })() {
            failures.push(format!("tap dance ({err})"));
        }

        if let Err(err) = (|| -> Result<()> {
            for (idx, ko) in bundle
                .data
                .key_overrides
                .entries
                .iter()
                .take(self.key_override_entries.len())
                .enumerate()
            {
                hid.set_key_override(
                    idx as u8,
                    ko.trigger,
                    ko.replacement,
                    ko.layers,
                    ko.trigger_mods,
                    ko.negative_mod_mask,
                    ko.suppressed_mods,
                    ko.options,
                )?;
            }
            Ok(())
        })() {
            failures.push(format!("key overrides ({err})"));
        }

        if let Err(err) = (|| -> Result<()> {
            for (idx, ar) in bundle
                .data
                .alt_repeat
                .entries
                .iter()
                .take(self.alt_repeat_entries.len())
                .enumerate()
            {
                hid.set_alt_repeat_key(
                    idx as u8,
                    ar.keycode,
                    ar.alt_keycode,
                    ar.allowed_mods,
                    ar.options,
                )?;
            }
            Ok(())
        })() {
            failures.push(format!("alt repeat ({err})"));
        }
        Ok(failures)
    }

    fn apply_entlayout_local_state(&mut self, bundle: &EntLayoutFile) -> Result<()> {
        if let Some(layout) = &mut self.layout {
            layout.layers = bundle.data.keymap.clone();
            layout.encoder_layers = bundle.data.encoder_keymap.clone();
        }

        self.encoder_visibility = normalized_bool_vec(
            &bundle.data.encoder_visibility,
            self.layout
                .as_ref()
                .map(|layout| layout.encoder_count())
                .unwrap_or(0),
            true,
        );
        if !self.current_encoder_visibility_id.is_empty() {
            save_encoder_visibility(
                &self.encoder_visibility,
                &self.current_encoder_visibility_id,
            );
        }

        if self.layout_options_value.is_some() {
            self.layout_options_value = bundle.data.layout_options;
        }
        self.layer_names = normalized_strings(&bundle.data.layer_names, self.layer_count);
        if !self.current_device_name.is_empty() {
            save_layer_names(&self.layer_names, &self.current_device_name);
        }

        self.app_settings = bundle.data.app_settings.clone();
        self.app_settings.text_expander_enabled = bundle.data.text_expander.enabled;
        self.app_settings.text_expander_app_blacklist =
            bundle.data.text_expander.app_blacklist.clone();
        self.app_settings.text_expander_rule_files =
            normalize_text_expander_rule_files(&bundle.data.text_expander.rule_files);
        self.app_settings.text_expansion_rules = bundle.data.text_expander.primary_rules.clone();
        save_app_settings(&self.app_settings);
        save_text_expansion_rules(&self.app_settings.text_expansion_rules);
        for file in &bundle.data.text_expander.extra_files {
            if let Some(file_name) = normalize_text_expander_rules_file_name(&file.file_name) {
                let path = text_expander_extra_rules_path(&file_name);
                if let Ok(json) = serde_json::to_string_pretty(&file.rules) {
                    std::fs::write(path, json)?;
                }
            }
        }
        self.text_expander_rules_signature =
            text_expander_rules_signature(&self.app_settings.text_expander_rule_files);
        self.sync_text_expander_runtime();

        self.keycode_picker.macro_texts = normalized_strings(
            &bundle.data.macros.texts,
            self.keycode_picker
                .macro_count
                .max(bundle.data.macros.texts.len()),
        );
        self.keycode_picker
            .macro_texts
            .truncate(self.keycode_picker.macro_count);
        self.keycode_picker.macro_names = normalized_strings(
            &bundle.data.macros.names,
            self.keycode_picker
                .macro_count
                .max(bundle.data.macros.names.len()),
        );
        self.keycode_picker
            .macro_names
            .truncate(self.keycode_picker.macro_count);
        self.keycode_picker.macros_dirty = false;

        let mut combo_entries = self.combo_entries.clone();
        for (idx, entry) in bundle
            .data
            .combos
            .entries
            .iter()
            .take(combo_entries.len())
            .enumerate()
        {
            combo_entries[idx] = ComboEntry {
                keys: entry.keys,
                output: entry.output,
            };
        }
        self.combo_entries = combo_entries;
        self.combo_names = normalized_strings(&bundle.data.combos.names, self.combo_entries.len());
        self.combo_term = bundle.data.combos.term.or(self.combo_term);
        save_combo_names(&self.combo_names, &self.current_device_name);
        self.combo_dirty = false;
        self.combo_names_dirty = false;
        self.combo_term_dirty = false;

        let mut tap_dance_entries = self.keycode_picker.tap_dance_entries.clone();
        for (idx, entry) in bundle
            .data
            .tap_dance
            .entries
            .iter()
            .take(tap_dance_entries.len())
            .enumerate()
        {
            tap_dance_entries[idx] = crate::keycode_picker::TapDanceEntry {
                on_tap: entry.on_tap,
                on_hold: entry.on_hold,
                on_double_tap: entry.on_double_tap,
                on_tap_hold: entry.on_tap_hold,
                tapping_term: entry.tapping_term,
            };
        }
        self.keycode_picker.tap_dance_entries = tap_dance_entries;
        self.keycode_picker.tap_dance_names = normalized_strings(
            &bundle.data.tap_dance.names,
            self.keycode_picker.tap_dance_entries.len(),
        );
        save_tap_dance_names(
            &self.keycode_picker.tap_dance_names,
            &self.current_device_name,
        );
        self.keycode_picker.tap_dance_dirty = false;

        let mut key_override_entries = self.key_override_entries.clone();
        for (idx, entry) in bundle
            .data
            .key_overrides
            .entries
            .iter()
            .take(key_override_entries.len())
            .enumerate()
        {
            let mut mapped = KeyOverrideEntry {
                trigger: entry.trigger,
                replacement: entry.replacement,
                layers: entry.layers,
                trigger_mods: entry.trigger_mods,
                negative_mod_mask: entry.negative_mod_mask,
                suppressed_mods: entry.suppressed_mods,
                options: KeyOverrideOptionsState::from_bits(entry.options),
            };
            Self::normalize_key_override_entry(&mut mapped);
            key_override_entries[idx] = mapped;
        }
        self.key_override_entries = key_override_entries;
        self.key_override_names = normalized_strings(
            &bundle.data.key_overrides.names,
            self.key_override_entries.len(),
        );
        save_key_override_names(&self.key_override_names, &self.current_device_name);

        let mut alt_repeat_entries = self.alt_repeat_entries.clone();
        for (idx, entry) in bundle
            .data
            .alt_repeat
            .entries
            .iter()
            .take(alt_repeat_entries.len())
            .enumerate()
        {
            let mut mapped = AltRepeatKeyEntry {
                keycode: entry.keycode,
                alt_keycode: entry.alt_keycode,
                allowed_mods: entry.allowed_mods,
                options: AltRepeatKeyOptionsState::from_bits(entry.options),
            };
            Self::normalize_alt_repeat_entry(&mut mapped);
            alt_repeat_entries[idx] = mapped;
        }
        self.alt_repeat_entries = alt_repeat_entries;
        self.alt_repeat_names =
            normalized_strings(&bundle.data.alt_repeat.names, self.alt_repeat_entries.len());
        save_alt_repeat_names(&self.alt_repeat_names, &self.current_device_name);
        Ok(())
    }
}

fn write_entlayout_file(path: &Path, bundle: &EntLayoutFile) -> Result<()> {
    let json = serde_json::to_string_pretty(bundle).context("failed to serialize .entlayout")?;
    std::fs::write(path, json).with_context(|| format!("failed to write {}", path.display()))
}

fn normalized_strings(values: &[String], len: usize) -> Vec<String> {
    let mut out = values.to_vec();
    out.resize(len, String::new());
    out.truncate(len);
    out
}

fn normalized_bool_vec(values: &[bool], len: usize, default: bool) -> Vec<bool> {
    let mut out = values.to_vec();
    out.resize(len, default);
    out.truncate(len);
    out
}

fn entlayout_hash(layout: &KeyboardLayout) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    fn feed(hash: &mut u64, bytes: &[u8]) {
        for byte in bytes {
            *hash ^= *byte as u64;
            *hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    feed(&mut hash, &layout.rows.to_le_bytes());
    feed(&mut hash, &layout.cols.to_le_bytes());
    for key in &layout.keys {
        feed(&mut hash, &key.row.to_le_bytes());
        feed(&mut hash, &key.col.to_le_bytes());
        feed(&mut hash, &key.x.to_le_bytes());
        feed(&mut hash, &key.y.to_le_bytes());
        feed(&mut hash, &key.w.to_le_bytes());
        feed(&mut hash, &key.h.to_le_bytes());
        feed(&mut hash, key.label.as_bytes());
        feed(&mut hash, &[0xff]);
    }
    for encoder in &layout.encoders {
        feed(&mut hash, &encoder.encoder_idx.to_le_bytes());
        feed(&mut hash, &encoder.direction.to_le_bytes());
        feed(&mut hash, &encoder.x.to_le_bytes());
        feed(&mut hash, &encoder.y.to_le_bytes());
        feed(&mut hash, encoder.label.as_bytes());
        feed(&mut hash, &[0xfe]);
    }
    for option in &layout.layout_options {
        feed(&mut hash, option.label.as_bytes());
        feed(&mut hash, &[0xfd]);
        for choice in &option.choices {
            feed(&mut hash, choice.as_bytes());
            feed(&mut hash, &[0xfc]);
        }
    }
    format!("fnv1a64:{hash:016x}")
}
