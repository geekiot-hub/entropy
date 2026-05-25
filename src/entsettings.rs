use super::*;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const ENTSETTINGS_FORMAT: &str = "entropy.app-settings";
const ENTSETTINGS_VERSION: u16 = 1;

#[derive(Clone, Serialize, Deserialize)]
struct EntSettingsFile {
    format: String,
    version: u16,
    created_by: String,
    created_at: String,
    settings: AppSettings,
    #[serde(default)]
    text_expander_extra_files: Vec<EntSettingsTextExpanderRuleFile>,
}

#[derive(Clone, Serialize, Deserialize)]
struct EntSettingsTextExpanderRuleFile {
    file_name: String,
    rules: Vec<crate::text_expander::TextExpansionRule>,
}

impl EntropyApp {
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn export_entsettings_dialog(&mut self) {
        let bundle = self.entsettings_snapshot();
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Entropy app settings", &["entsettings"])
            .set_file_name("entropy-app-settings.entsettings")
            .save_file()
        else {
            return;
        };
        match write_entsettings_file(&path, &bundle) {
            Ok(()) => self.status_msg = format!("Exported app settings: {}", path.display()),
            Err(e) => self.status_msg = format!("Export app settings failed: {e}"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn import_entsettings_dialog(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Entropy app settings", &["entsettings"])
            .pick_file()
        else {
            return;
        };
        match self.import_entsettings_from_path(ctx, &path) {
            Ok(report) => {
                self.status_msg = "Imported app settings".into();
                self.import_report_title = "App settings import report".into();
                self.import_report_body = report;
                self.import_report_open = true;
            }
            Err(e) => self.status_msg = format!("Import app settings failed: {e}"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn import_entsettings_from_path(&mut self, ctx: &egui::Context, path: &Path) -> Result<String> {
        let data = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let bundle: EntSettingsFile = serde_json::from_str(&data)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        validate_entsettings_file(&bundle)?;
        let backup_path = write_entsettings_auto_backup(&self.entsettings_snapshot())?;
        self.apply_entsettings(ctx, bundle)?;
        Ok(format!(
            "Imported app settings: {}. Auto-backup: {}",
            path.display(),
            backup_path.display()
        ))
    }

    fn entsettings_snapshot(&self) -> EntSettingsFile {
        let mut settings = self.app_settings.clone();
        settings.ui_scale = clamp_ui_scale(settings.ui_scale);
        settings.text_expander_rule_files =
            normalize_text_expander_rule_files(&settings.text_expander_rule_files);

        let text_expander_extra_files = settings
            .text_expander_rule_files
            .iter()
            .filter_map(|file_name| {
                load_text_expansion_rules_from_path(&text_expander_extra_rules_path(file_name)).map(
                    |rules| EntSettingsTextExpanderRuleFile {
                        file_name: file_name.clone(),
                        rules,
                    },
                )
            })
            .collect();

        EntSettingsFile {
            format: ENTSETTINGS_FORMAT.to_owned(),
            version: ENTSETTINGS_VERSION,
            created_by: "Entropy".to_owned(),
            created_at: chrono::Local::now().to_rfc3339(),
            settings,
            text_expander_extra_files,
        }
    }

    fn apply_entsettings(&mut self, ctx: &egui::Context, bundle: EntSettingsFile) -> Result<()> {
        let mut settings = bundle.settings;
        settings.ui_scale = clamp_ui_scale(settings.ui_scale);
        settings.text_expander_rule_files =
            normalize_text_expander_rule_files(&settings.text_expander_rule_files);

        self.app_settings = settings;
        save_app_settings(&self.app_settings);
        save_text_expansion_rules(&self.app_settings.text_expansion_rules);

        for file in &bundle.text_expander_extra_files {
            if let Some(file_name) = normalize_text_expander_rules_file_name(&file.file_name) {
                let path = text_expander_extra_rules_path(&file_name);
                let json = serde_json::to_string_pretty(&file.rules)
                    .context("failed to serialize text expander rules")?;
                std::fs::write(&path, json)
                    .with_context(|| format!("failed to write {}", path.display()))?;
            }
        }

        crate::ui_style::set_accent(self.app_settings.accent_color.color());
        ctx.set_zoom_factor(self.app_settings.ui_scale);
        self.sticky_layout_last_size = None;
        if !self.app_settings.layer_hover_preview {
            self.hover_layer = None;
        }
        #[cfg(target_os = "windows")]
        if !self.app_settings.minimize_to_tray_on_close {
            self.tray_icon = None;
        }
        self.text_expander_rules_signature =
            text_expander_rules_signature(&self.app_settings.text_expander_rule_files);
        self.sync_text_expander_runtime();
        ctx.request_repaint();
        Ok(())
    }
}

fn validate_entsettings_file(bundle: &EntSettingsFile) -> Result<()> {
    if bundle.format != ENTSETTINGS_FORMAT {
        bail!("unsupported format: {}", bundle.format);
    }
    if bundle.version == 0 || bundle.version > ENTSETTINGS_VERSION {
        bail!("unsupported .entsettings version: {}", bundle.version);
    }
    Ok(())
}

fn write_entsettings_auto_backup(bundle: &EntSettingsFile) -> Result<PathBuf> {
    let base_dir = app_settings_path()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let dir = base_dir.join("backups");
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create backup dir {}", dir.display()))?;
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let path = dir.join(format!("auto-backup-app-settings-{stamp}.entsettings"));
    write_entsettings_file(&path, bundle)?;
    Ok(path)
}

fn write_entsettings_file(path: &Path, bundle: &EntSettingsFile) -> Result<()> {
    let json = serde_json::to_string_pretty(bundle).context("failed to serialize .entsettings")?;
    std::fs::write(path, json).with_context(|| format!("failed to write {}", path.display()))
}
