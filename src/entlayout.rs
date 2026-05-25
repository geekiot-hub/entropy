use super::*;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const ENTLAYOUT_FORMAT: &str = "entropy.layout";
const ENTLAYOUT_VERSION: u16 = 2;

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_layout: Option<EntLayoutSourceLayout>,
    keymap: Vec<Vec<u16>>,
    encoder_keymap: Vec<Vec<u16>>,
    encoder_visibility: Vec<bool>,
    layout_options: Option<u32>,
    layer_names: Vec<String>,
    text_expander: EntTextExpanderData,
    macros: EntMacroData,
    combos: EntComboData,
    tap_dance: EntTapDanceData,
    key_overrides: EntKeyOverrideData,
    alt_repeat: EntAltRepeatData,
}

#[derive(Clone, Serialize, Deserialize)]
struct EntLayoutSourceLayout {
    keys: Vec<EntLayoutSourceKey>,
    encoders: Vec<EntLayoutSourceEncoder>,
    custom_keycodes: Vec<EntLayoutSourceCustomKeycode>,
}

#[derive(Clone, Serialize, Deserialize)]
struct EntLayoutSourceKey {
    index: usize,
    row: u8,
    col: u8,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rotation: f32,
    rotation_x: f32,
    rotation_y: f32,
    label: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct EntLayoutSourceEncoder {
    visual_index: usize,
    encoder_idx: u8,
    direction: u8,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rotation: f32,
    rotation_x: f32,
    rotation_y: f32,
    label: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct EntLayoutSourceCustomKeycode {
    index: usize,
    name: String,
    label: String,
    title: String,
}

struct EntLayoutImportMapping {
    exact_layout: bool,
    key_mapping: Vec<Option<usize>>,
    encoder_mapping: Vec<Option<usize>>,
}

impl EntLayoutImportMapping {
    fn mapped_keys(&self) -> usize {
        self.key_mapping.iter().filter(|idx| idx.is_some()).count()
    }

    fn mapped_encoders(&self) -> usize {
        self.encoder_mapping
            .iter()
            .filter(|idx| idx.is_some())
            .count()
    }
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
        let (firmware_failures, mapping) = self.apply_entlayout(&bundle)?;
        Ok(self.entlayout_import_report(&bundle, &backup_path, &firmware_failures, &mapping))
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
                source_layout: Some(entlayout_source_layout(layout)),
                keymap: layout.layers.clone(),
                encoder_keymap: layout.encoder_layers.clone(),
                encoder_visibility: self.encoder_visibility.clone(),
                layout_options: self.layout_options_value,
                layer_names: self.layer_names.clone(),
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
        if bundle.version == 0 || bundle.version > ENTLAYOUT_VERSION {
            bail!("unsupported .entlayout version: {}", bundle.version);
        }
        let _layout = self
            .layout
            .as_ref()
            .context("connect the target keyboard before importing")?;
        if bundle.data.keymap.is_empty() {
            bail!(".entlayout keymap is empty");
        }
        let source_key_count = bundle_source_key_count(bundle);
        if source_key_count == 0 {
            bail!(".entlayout has no source keys");
        }
        if bundle
            .data
            .keymap
            .iter()
            .any(|layer| layer.len() != source_key_count)
        {
            bail!(".entlayout keymap layers have inconsistent key counts");
        }
        let source_encoder_count = bundle_source_encoder_count(bundle);
        if bundle
            .data
            .encoder_keymap
            .iter()
            .any(|layer| layer.len() != source_encoder_count)
        {
            bail!(".entlayout encoder layers have inconsistent encoder counts");
        }
        Ok(())
    }

    fn entlayout_import_mapping(&self, bundle: &EntLayoutFile) -> Result<EntLayoutImportMapping> {
        let layout = self
            .layout
            .as_ref()
            .context("connect the target keyboard before importing")?;
        let exact_layout = bundle.keyboard.layout_hash == entlayout_hash(layout)
            && bundle.keyboard.layers == layout.layers.len()
            && bundle_source_key_count(bundle) == layout.keys.len()
            && bundle_source_encoder_count(bundle) == layout.encoders.len();

        if exact_layout {
            return Ok(EntLayoutImportMapping {
                exact_layout: true,
                key_mapping: (0..bundle_source_key_count(bundle)).map(Some).collect(),
                encoder_mapping: (0..bundle_source_encoder_count(bundle)).map(Some).collect(),
            });
        }

        Ok(EntLayoutImportMapping {
            exact_layout: false,
            key_mapping: universal_key_mapping(bundle, layout),
            encoder_mapping: universal_encoder_mapping(bundle, layout),
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn apply_entlayout(
        &mut self,
        bundle: &EntLayoutFile,
    ) -> Result<(Vec<String>, EntLayoutImportMapping)> {
        let mapping = self.entlayout_import_mapping(bundle)?;
        let firmware_failures = self.apply_entlayout_firmware_state(bundle, &mapping)?;
        self.apply_entlayout_local_state(bundle, &mapping)?;
        self.refresh_layer_picker_content_flags();
        Ok((firmware_failures, mapping))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn entlayout_import_report(
        &self,
        bundle: &EntLayoutFile,
        backup_path: &Path,
        firmware_failures: &[String],
        mapping: &EntLayoutImportMapping,
    ) -> String {
        let mut imported = vec![
            "keymap",
            "encoder keymap",
            "encoder visibility",
            "layer names",
            "Text Expander",
        ];
        let mut skipped = Vec::new();

        if mapping.exact_layout {
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

            if !bundle.data.key_overrides.entries.is_empty()
                && !self.key_override_entries.is_empty()
            {
                imported.push("key overrides");
            } else {
                skipped.push("key overrides (not available)");
            }

            if !bundle.data.alt_repeat.entries.is_empty() && !self.alt_repeat_entries.is_empty() {
                imported.push("alt repeat");
            } else {
                skipped.push("alt repeat (not available)");
            }
        } else {
            skipped.push(
                "layout options/dynamic features (universal mode imports physical keymap only)",
            );
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
        let mode = if mapping.exact_layout {
            "exact"
        } else {
            "universal"
        };
        format!(
            "Imported .entlayout ({mode}): {}. Mapped: {}/{} keys, {}/{} encoder slots. Skipped: {}. Firmware failures: {}. Auto-backup: {}",
            imported.join(", "),
            mapping.mapped_keys(),
            mapping.key_mapping.len(),
            mapping.mapped_encoders(),
            mapping.encoder_mapping.len(),
            skipped,
            firmware_failures,
            backup_path.display()
        )
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn apply_entlayout_firmware_state(
        &mut self,
        bundle: &EntLayoutFile,
        mapping: &EntLayoutImportMapping,
    ) -> Result<Vec<String>> {
        let Some(hid) = &self.hid_device else {
            bail!("no active keyboard connection for firmware import");
        };
        let layout = self.layout.as_ref().context("no connected layout")?.clone();
        let mut failures = Vec::new();

        if let Err(err) = (|| -> Result<()> {
            for (source_layer_idx, layer_codes) in bundle.data.keymap.iter().enumerate() {
                let Some(target_layer_idx) = map_layer_index(source_layer_idx, self.layer_count)
                else {
                    continue;
                };
                for (source_key_idx, keycode) in layer_codes.iter().copied().enumerate() {
                    let Some(target_key_idx) =
                        mapping.key_mapping.get(source_key_idx).copied().flatten()
                    else {
                        continue;
                    };
                    let keycode = map_entlayout_keycode(keycode, bundle, &layout);
                    if layout
                        .layers
                        .get(target_layer_idx)
                        .and_then(|layer| layer.get(target_key_idx))
                        .copied()
                        == Some(keycode)
                    {
                        continue;
                    }
                    let key = &layout.keys[target_key_idx];
                    hid.set_keycode(target_layer_idx as u8, key.row, key.col, keycode)?;
                }
            }
            Ok(())
        })() {
            failures.push(format!("keymap ({err})"));
        }

        if let Err(err) = (|| -> Result<()> {
            for (source_layer_idx, layer_codes) in bundle.data.encoder_keymap.iter().enumerate() {
                let Some(target_layer_idx) = map_layer_index(source_layer_idx, self.layer_count)
                else {
                    continue;
                };
                for (source_encoder_idx, keycode) in layer_codes.iter().copied().enumerate() {
                    let Some(target_encoder_idx) = mapping
                        .encoder_mapping
                        .get(source_encoder_idx)
                        .copied()
                        .flatten()
                    else {
                        continue;
                    };
                    let keycode = map_entlayout_keycode(keycode, bundle, &layout);
                    if layout
                        .encoder_layers
                        .get(target_layer_idx)
                        .and_then(|layer| layer.get(target_encoder_idx))
                        .copied()
                        == Some(keycode)
                    {
                        continue;
                    }
                    let encoder = &layout.encoders[target_encoder_idx];
                    hid.set_encoder(
                        target_layer_idx as u8,
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

        if mapping.exact_layout {
            if let Some(options) = bundle.data.layout_options {
                if self.layout_options_value.is_some() && self.layout_options_value != Some(options)
                {
                    if let Err(err) = hid.set_layout_options(options) {
                        failures.push(format!("layout options ({err})"));
                    }
                }
            }
        }

        if mapping.exact_layout
            && !bundle.data.macros.texts.is_empty()
            && self.keycode_picker.macro_count > 0
        {
            if let Err(err) = (|| -> Result<()> {
                let mut macro_texts = bundle.data.macros.texts.clone();
                macro_texts.resize(self.keycode_picker.macro_count, String::new());
                macro_texts.truncate(self.keycode_picker.macro_count);

                let mut current_macro_texts = self.keycode_picker.macro_texts.clone();
                current_macro_texts.resize(self.keycode_picker.macro_count, String::new());
                current_macro_texts.truncate(self.keycode_picker.macro_count);
                if macro_texts == current_macro_texts {
                    return Ok(());
                }

                let size = hid.get_macro_buffer_size()?;
                let buf = crate::hid::HidDevice::encode_macros(&macro_texts, size);
                hid.set_macro_buffer(&buf)?;
                Ok(())
            })() {
                failures.push(format!("macros ({err})"));
            }
        }

        if mapping.exact_layout {
            if let Err(err) = (|| -> Result<()> {
                for (idx, combo) in bundle
                    .data
                    .combos
                    .entries
                    .iter()
                    .take(self.combo_entries.len())
                    .enumerate()
                {
                    let current = &self.combo_entries[idx];
                    if current.keys == combo.keys && current.output == combo.output {
                        continue;
                    }
                    hid.set_combo(idx as u8, combo.keys, combo.output)?;
                }
                Ok(())
            })() {
                failures.push(format!("combos ({err})"));
            }
            if let Some(term) = bundle.data.combos.term {
                if self.combo_term.is_some() && self.combo_term != Some(term) {
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
                    let current = &self.keycode_picker.tap_dance_entries[idx];
                    if current.on_tap == td.on_tap
                        && current.on_hold == td.on_hold
                        && current.on_double_tap == td.on_double_tap
                        && current.on_tap_hold == td.on_tap_hold
                        && current.tapping_term == td.tapping_term
                    {
                        continue;
                    }
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
                    let current = &self.key_override_entries[idx];
                    if current.trigger == ko.trigger
                        && current.replacement == ko.replacement
                        && current.layers == ko.layers
                        && current.trigger_mods == ko.trigger_mods
                        && current.negative_mod_mask == ko.negative_mod_mask
                        && current.suppressed_mods == ko.suppressed_mods
                        && current.options.bits() == ko.options
                    {
                        continue;
                    }
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
                    let current = &self.alt_repeat_entries[idx];
                    if current.keycode == ar.keycode
                        && current.alt_keycode == ar.alt_keycode
                        && current.allowed_mods == ar.allowed_mods
                        && current.options.bits() == ar.options
                    {
                        continue;
                    }
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
        }
        Ok(failures)
    }

    fn apply_entlayout_local_state(
        &mut self,
        bundle: &EntLayoutFile,
        mapping: &EntLayoutImportMapping,
    ) -> Result<()> {
        if let Some(layout) = &mut self.layout {
            let mut target_layers = layout.layers.clone();
            for (source_layer_idx, layer_codes) in bundle.data.keymap.iter().enumerate() {
                let Some(target_layer_idx) = map_layer_index(source_layer_idx, target_layers.len())
                else {
                    continue;
                };
                for (source_key_idx, keycode) in layer_codes.iter().copied().enumerate() {
                    let Some(target_key_idx) =
                        mapping.key_mapping.get(source_key_idx).copied().flatten()
                    else {
                        continue;
                    };
                    if let Some(slot) = target_layers
                        .get_mut(target_layer_idx)
                        .and_then(|layer| layer.get_mut(target_key_idx))
                    {
                        *slot = map_entlayout_keycode(keycode, bundle, layout);
                    }
                }
            }
            layout.layers = target_layers;

            let mut target_encoder_layers = layout.encoder_layers.clone();
            for (source_layer_idx, layer_codes) in bundle.data.encoder_keymap.iter().enumerate() {
                let Some(target_layer_idx) =
                    map_layer_index(source_layer_idx, target_encoder_layers.len())
                else {
                    continue;
                };
                for (source_encoder_idx, keycode) in layer_codes.iter().copied().enumerate() {
                    let Some(target_encoder_idx) = mapping
                        .encoder_mapping
                        .get(source_encoder_idx)
                        .copied()
                        .flatten()
                    else {
                        continue;
                    };
                    if let Some(slot) = target_encoder_layers
                        .get_mut(target_layer_idx)
                        .and_then(|layer| layer.get_mut(target_encoder_idx))
                    {
                        *slot = map_entlayout_keycode(keycode, bundle, layout);
                    }
                }
            }
            layout.encoder_layers = target_encoder_layers;
        }

        let mut encoder_visibility = self.encoder_visibility.clone();
        encoder_visibility.resize(
            self.layout
                .as_ref()
                .map(|layout| layout.encoder_count())
                .unwrap_or(0),
            true,
        );
        for (source_idx, visible) in bundle.data.encoder_visibility.iter().copied().enumerate() {
            let Some(target_idx) = mapping.encoder_mapping.get(source_idx).copied().flatten()
            else {
                continue;
            };
            if let Some(slot) = encoder_visibility.get_mut(target_idx) {
                *slot = visible;
            }
        }
        self.encoder_visibility = encoder_visibility;
        if !self.current_encoder_visibility_id.is_empty() {
            save_encoder_visibility(
                &self.encoder_visibility,
                &self.current_encoder_visibility_id,
            );
        }

        if mapping.exact_layout && self.layout_options_value.is_some() {
            self.layout_options_value = bundle.data.layout_options;
        }
        for (idx, name) in bundle
            .data
            .layer_names
            .iter()
            .take(self.layer_names.len().min(self.layer_count))
            .enumerate()
        {
            self.layer_names[idx] = name.clone();
        }
        self.layer_names.truncate(self.layer_count);
        if !self.current_device_name.is_empty() {
            save_layer_names(&self.layer_names, &self.current_device_name);
        }

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

        if mapping.exact_layout {
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
            self.combo_names =
                normalized_strings(&bundle.data.combos.names, self.combo_entries.len());
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
        }
        Ok(())
    }
}

fn entlayout_source_layout(layout: &KeyboardLayout) -> EntLayoutSourceLayout {
    EntLayoutSourceLayout {
        keys: layout
            .keys
            .iter()
            .enumerate()
            .map(|(index, key)| EntLayoutSourceKey {
                index,
                row: key.row,
                col: key.col,
                x: key.x,
                y: key.y,
                w: key.w,
                h: key.h,
                rotation: key.rotation,
                rotation_x: key.rotation_x,
                rotation_y: key.rotation_y,
                label: key.label.clone(),
            })
            .collect(),
        encoders: layout
            .encoders
            .iter()
            .enumerate()
            .map(|(visual_index, encoder)| EntLayoutSourceEncoder {
                visual_index,
                encoder_idx: encoder.encoder_idx,
                direction: encoder.direction,
                x: encoder.x,
                y: encoder.y,
                w: encoder.w,
                h: encoder.h,
                rotation: encoder.rotation,
                rotation_x: encoder.rotation_x,
                rotation_y: encoder.rotation_y,
                label: encoder.label.clone(),
            })
            .collect(),
        custom_keycodes: layout
            .custom_keycodes
            .iter()
            .enumerate()
            .map(|(index, custom)| EntLayoutSourceCustomKeycode {
                index,
                name: custom.name.clone(),
                label: custom.label.clone(),
                title: custom.title.clone(),
            })
            .collect(),
    }
}

fn bundle_source_key_count(bundle: &EntLayoutFile) -> usize {
    bundle
        .data
        .source_layout
        .as_ref()
        .map(|source| source.keys.len())
        .unwrap_or(bundle.keyboard.keys)
}

fn bundle_source_encoder_count(bundle: &EntLayoutFile) -> usize {
    bundle
        .data
        .source_layout
        .as_ref()
        .map(|source| source.encoders.len())
        .unwrap_or_else(|| {
            bundle
                .data
                .encoder_keymap
                .iter()
                .map(Vec::len)
                .max()
                .unwrap_or(bundle.keyboard.encoders)
        })
}

fn map_layer_index(source_idx: usize, target_len: usize) -> Option<usize> {
    (source_idx < target_len).then_some(source_idx)
}

fn universal_key_mapping(bundle: &EntLayoutFile, layout: &KeyboardLayout) -> Vec<Option<usize>> {
    let Some(source) = bundle.data.source_layout.as_ref() else {
        let len = bundle_source_key_count(bundle);
        return (0..len)
            .map(|idx| (idx < layout.keys.len()).then_some(idx))
            .collect();
    };
    let source_points: Vec<_> = source.keys.iter().map(source_key_point).collect();
    let target_points: Vec<_> = layout.keys.iter().map(target_key_point).collect();
    let (source_bounds, target_bounds) =
        (points_bounds(&source_points), points_bounds(&target_points));
    let mut used = vec![false; layout.keys.len()];
    source
        .keys
        .iter()
        .zip(source_points.iter().copied())
        .map(|(source_key, source_point)| {
            let normalized = normalize_point(source_point, source_bounds);
            let mut best: Option<(usize, f32)> = None;
            for (target_idx, target_key) in layout.keys.iter().enumerate() {
                if used[target_idx] {
                    continue;
                }
                let target_normalized = normalize_point(target_points[target_idx], target_bounds);
                let mut score = point_distance_squared(normalized, target_normalized);
                let size_penalty = ((source_key.w - target_key.w).abs()
                    + (source_key.h - target_key.h).abs())
                    * 0.03;
                score += size_penalty;
                if source_key.label == target_key.label {
                    score *= 0.65;
                }
                if source_key.row == target_key.row && source_key.col == target_key.col {
                    score *= 0.75;
                }
                if best
                    .map(|(_, best_score)| score < best_score)
                    .unwrap_or(true)
                {
                    best = Some((target_idx, score));
                }
            }
            let Some((target_idx, score)) = best else {
                return None;
            };
            let threshold = if source.keys.len() == layout.keys.len() {
                0.045
            } else {
                0.032
            };
            if score <= threshold {
                used[target_idx] = true;
                Some(target_idx)
            } else {
                None
            }
        })
        .collect()
}

fn universal_encoder_mapping(
    bundle: &EntLayoutFile,
    layout: &KeyboardLayout,
) -> Vec<Option<usize>> {
    let Some(source) = bundle.data.source_layout.as_ref() else {
        let len = bundle_source_encoder_count(bundle);
        return (0..len)
            .map(|idx| (idx < layout.encoders.len()).then_some(idx))
            .collect();
    };
    let source_points: Vec<_> = source.encoders.iter().map(source_encoder_point).collect();
    let target_points: Vec<_> = layout.encoders.iter().map(target_encoder_point).collect();
    let (source_bounds, target_bounds) =
        (points_bounds(&source_points), points_bounds(&target_points));
    let mut used = vec![false; layout.encoders.len()];
    source
        .encoders
        .iter()
        .zip(source_points.iter().copied())
        .map(|(source_encoder, source_point)| {
            let normalized = normalize_point(source_point, source_bounds);
            let mut best: Option<(usize, f32)> = None;
            for (target_idx, target_encoder) in layout.encoders.iter().enumerate() {
                if used[target_idx] || source_encoder.direction != target_encoder.direction {
                    continue;
                }
                let target_normalized = normalize_point(target_points[target_idx], target_bounds);
                let mut score = point_distance_squared(normalized, target_normalized);
                if source_encoder.encoder_idx == target_encoder.encoder_idx {
                    score *= 0.8;
                }
                if best
                    .map(|(_, best_score)| score < best_score)
                    .unwrap_or(true)
                {
                    best = Some((target_idx, score));
                }
            }
            let Some((target_idx, score)) = best else {
                return None;
            };
            if score <= 0.05 {
                used[target_idx] = true;
                Some(target_idx)
            } else {
                None
            }
        })
        .collect()
}

fn source_key_point(key: &EntLayoutSourceKey) -> (f32, f32) {
    rotated_layout_point(
        key.x + key.w * 0.5,
        key.y + key.h * 0.5,
        key.rotation_x,
        key.rotation_y,
        key.rotation,
    )
}

fn target_key_point(key: &crate::keyboard::PhysicalKey) -> (f32, f32) {
    rotated_layout_point(
        key.x + key.w * 0.5,
        key.y + key.h * 0.5,
        key.rotation_x,
        key.rotation_y,
        key.rotation,
    )
}

fn source_encoder_point(encoder: &EntLayoutSourceEncoder) -> (f32, f32) {
    rotated_layout_point(
        encoder.x + encoder.w * 0.5,
        encoder.y + encoder.h * 0.5,
        encoder.rotation_x,
        encoder.rotation_y,
        encoder.rotation,
    )
}

fn target_encoder_point(encoder: &crate::keyboard::PhysicalEncoder) -> (f32, f32) {
    rotated_layout_point(
        encoder.x + encoder.w * 0.5,
        encoder.y + encoder.h * 0.5,
        encoder.rotation_x,
        encoder.rotation_y,
        encoder.rotation,
    )
}

fn rotated_layout_point(
    x: f32,
    y: f32,
    origin_x: f32,
    origin_y: f32,
    rotation_deg: f32,
) -> (f32, f32) {
    if rotation_deg == 0.0 {
        return (x, y);
    }
    let angle = rotation_deg.to_radians();
    let dx = x - origin_x;
    let dy = y - origin_y;
    (
        origin_x + dx * angle.cos() - dy * angle.sin(),
        origin_y + dx * angle.sin() + dy * angle.cos(),
    )
}

fn points_bounds(points: &[(f32, f32)]) -> (f32, f32, f32, f32) {
    if points.is_empty() {
        return (0.0, 0.0, 1.0, 1.0);
    }
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for &(x, y) in points {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    (min_x, min_y, max_x, max_y)
}

fn normalize_point(
    (x, y): (f32, f32),
    (min_x, min_y, max_x, max_y): (f32, f32, f32, f32),
) -> (f32, f32) {
    let w = (max_x - min_x).max(0.001);
    let h = (max_y - min_y).max(0.001);
    ((x - min_x) / w, (y - min_y) / h)
}

fn point_distance_squared(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    dx * dx + dy * dy
}

fn map_entlayout_keycode(keycode: u16, bundle: &EntLayoutFile, layout: &KeyboardLayout) -> u16 {
    map_custom_keycode_by_name(keycode, bundle, layout)
        .unwrap_or_else(|| map_layer_keycode(keycode, layout.layers.len()))
}

fn map_custom_keycode_by_name(
    keycode: u16,
    bundle: &EntLayoutFile,
    layout: &KeyboardLayout,
) -> Option<u16> {
    const QK_KB: u16 = 0x7E00;
    let source_idx = keycode.checked_sub(QK_KB)? as usize;
    let source_custom = bundle
        .data
        .source_layout
        .as_ref()?
        .custom_keycodes
        .iter()
        .find(|custom| custom.index == source_idx)?;
    let target_idx = layout
        .custom_keycodes
        .iter()
        .position(|custom| custom.name == source_custom.name)?;
    Some(QK_KB + target_idx as u16)
}

fn map_layer_keycode(keycode: u16, target_layer_count: usize) -> u16 {
    if (0x5200..0x5300).contains(&keycode) {
        let op = (keycode >> 5) & 0x7;
        if op != 5 {
            let target = (keycode & 0x1F) as usize;
            if target >= target_layer_count {
                return 0;
            }
        }
    } else if keycode & 0xF000 == 0x4000 {
        let target = ((keycode >> 8) & 0xF) as usize;
        if target >= target_layer_count {
            return 0;
        }
    }
    keycode
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
