use crate::device::{Device, DeviceManager};
use crate::firmware::FirmwareProtocol;

#[cfg(target_os = "windows")]
static TRAY_QUIT_REQUESTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

const MATRIX_TESTER_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(16);
const MATRIX_TESTER_LOCK_CHECK_INTERVAL: std::time::Duration =
    std::time::Duration::from_millis(750);
const UI_SCALE_MIN: f32 = 0.5;
const UI_SCALE_MAX: f32 = 2.0;
const UI_SCALE_STEP: f32 = 0.1;
const ONBOARDING_TOUR_VERSION: u16 = 1;

/// Sanitize a device name into a filesystem-safe slug.
fn device_id_slug(device_name: &str) -> String {
    device_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn layer_names_path(device_name: &str) -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    std::fs::create_dir_all(&dir).ok();
    let slug = device_id_slug(device_name);
    dir.join(format!("layer_names_{}.json", slug))
}

fn single_instance_signal_path() -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    std::fs::create_dir_all(&dir).ok();
    dir.join("single_instance_signal")
}

#[derive(Clone, Copy)]
enum SettingsFieldUnit {
    Milliseconds,
    Minutes,
    CursorSteps,
    SpeedSteps,
}

impl SettingsFieldUnit {
    fn tooltip_key(self) -> &'static str {
        match self {
            SettingsFieldUnit::Milliseconds => "field_units.milliseconds",
            SettingsFieldUnit::Minutes => "field_units.minutes",
            SettingsFieldUnit::CursorSteps => "field_units.cursor_steps",
            SettingsFieldUnit::SpeedSteps => "field_units.speed_steps",
        }
    }
}

fn settings_field_unit_tooltip(
    response: egui::Response,
    language: crate::i18n::Language,
    suppress_tooltip: bool,
    unit: SettingsFieldUnit,
) -> egui::Response {
    if suppress_tooltip {
        response
    } else {
        response.on_hover_text(crate::i18n::tr_catalog(language, unit.tooltip_key()))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TourTarget {
    MainNavigation,
    DeviceSelector,
    LayerSwitcher,
    KeyboardArea,
    SettingsMenu,
    BottomHints,
}

#[derive(Clone, Copy)]
struct TourStep {
    target: Option<TourTarget>,
    title_key: &'static str,
    body_key: &'static str,
}

const ONBOARDING_TOUR_STEPS: [TourStep; 7] = [
    TourStep {
        target: None,
        title_key: "onboarding_tour.welcome_title",
        body_key: "onboarding_tour.welcome_body",
    },
    TourStep {
        target: Some(TourTarget::MainNavigation),
        title_key: "onboarding_tour.navigation_title",
        body_key: "onboarding_tour.navigation_body",
    },
    TourStep {
        target: Some(TourTarget::DeviceSelector),
        title_key: "onboarding_tour.device_title",
        body_key: "onboarding_tour.device_body",
    },
    TourStep {
        target: Some(TourTarget::LayerSwitcher),
        title_key: "onboarding_tour.layers_title",
        body_key: "onboarding_tour.layers_body",
    },
    TourStep {
        target: Some(TourTarget::KeyboardArea),
        title_key: "onboarding_tour.keyboard_title",
        body_key: "onboarding_tour.keyboard_body",
    },
    TourStep {
        target: Some(TourTarget::SettingsMenu),
        title_key: "onboarding_tour.settings_title",
        body_key: "onboarding_tour.settings_body",
    },
    TourStep {
        target: Some(TourTarget::BottomHints),
        title_key: "onboarding_tour.hints_title",
        body_key: "onboarding_tour.hints_body",
    },
];

#[derive(Default)]
struct TourState {
    active: bool,
    step: usize,
}

fn read_single_instance_signal() -> String {
    std::fs::read_to_string(single_instance_signal_path())
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn app_settings_path() -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    std::fs::create_dir_all(&dir).ok();
    dir.join("app_settings.json")
}

const TEXT_EXPANDER_MAIN_RULES_FILE: &str = "text_expansion_rules.json";

fn text_expander_config_dir() -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    std::fs::create_dir_all(&dir).ok();
    dir
}

fn text_expander_rules_dir() -> std::path::PathBuf {
    let dir = text_expander_config_dir().join("text_expander_rules");
    std::fs::create_dir_all(&dir).ok();
    dir
}

fn legacy_text_expander_rules_path() -> std::path::PathBuf {
    text_expander_config_dir().join(TEXT_EXPANDER_MAIN_RULES_FILE)
}

fn text_expander_rules_path() -> std::path::PathBuf {
    text_expander_rules_dir().join(TEXT_EXPANDER_MAIN_RULES_FILE)
}

fn migrate_legacy_text_expander_rules_file() {
    let legacy = legacy_text_expander_rules_path();
    let current = text_expander_rules_path();
    if !current.exists() && legacy.exists() {
        if let Err(e) = std::fs::copy(&legacy, &current) {
            log::warn!("migrate_legacy_text_expander_rules_file failed: {e}");
        }
    }
}

fn path_modified(path: &std::path::Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
}

fn normalize_text_expander_rules_file_name(raw: &str) -> Option<String> {
    let name = std::path::Path::new(raw)
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::trim)?;
    if name.is_empty()
        || name.eq_ignore_ascii_case(TEXT_EXPANDER_MAIN_RULES_FILE)
        || !name.to_ascii_lowercase().ends_with(".json")
    {
        return None;
    }
    Some(name.to_owned())
}

fn normalize_text_expander_rule_files(files: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();
    for file in files {
        let Some(name) = normalize_text_expander_rules_file_name(file) else {
            continue;
        };
        if !normalized
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(&name))
        {
            normalized.push(name);
        }
    }
    normalized
}

fn text_expander_extra_rules_path(file_name: &str) -> std::path::PathBuf {
    text_expander_rules_dir().join(file_name)
}

fn text_expander_available_extra_rule_files(selected_files: &[String]) -> Vec<String> {
    let selected = normalize_text_expander_rule_files(selected_files);
    let mut files = std::fs::read_dir(text_expander_rules_dir())
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return None;
            }
            let name = path.file_name()?.to_str()?;
            normalize_text_expander_rules_file_name(name)
        })
        .filter(|name| {
            !selected
                .iter()
                .any(|selected_name| selected_name.eq_ignore_ascii_case(name))
        })
        .collect::<Vec<_>>();
    files.sort_by_key(|name| name.to_ascii_lowercase());
    files
}

fn text_expander_rules_signature(
    extra_files: &[String],
) -> Vec<(String, Option<std::time::SystemTime>)> {
    let normalized_extra_files = normalize_text_expander_rule_files(extra_files);
    let mut files = Vec::with_capacity(normalized_extra_files.len() + 1);
    let primary = text_expander_rules_path();
    files.push((
        primary.to_string_lossy().to_string(),
        path_modified(&primary),
    ));
    for file in normalized_extra_files {
        let path = text_expander_extra_rules_path(&file);
        files.push((file, path_modified(&path)));
    }
    files
}

fn display_preset_restore_path(device_name: &str) -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    std::fs::create_dir_all(&dir).ok();
    dir.join(format!(
        "display_preset_restore_{}.txt",
        device_id_slug(device_name)
    ))
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum AppAccentColor {
    Rose,
    Violet,
    Blue,
    Amber,
    Copper,
    Slate,
}

impl AppAccentColor {
    const ALL: [Self; 6] = [
        Self::Rose,
        Self::Violet,
        Self::Blue,
        Self::Amber,
        Self::Copper,
        Self::Slate,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Rose => "Rose",
            Self::Violet => "Violet",
            Self::Blue => "Blue",
            Self::Amber => "Amber",
            Self::Copper => "Copper",
            Self::Slate => "Teal",
        }
    }

    fn color(self) -> Color32 {
        match self {
            Self::Rose => Color32::from_rgb(196, 132, 144),
            Self::Violet => Color32::from_rgb(146, 128, 184),
            Self::Blue => Color32::from_rgb(116, 154, 212),
            Self::Amber => Color32::from_rgb(210, 156, 92),
            Self::Copper => Color32::from_rgb(192, 116, 88),
            Self::Slate => Color32::from_rgb(88, 158, 148),
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct AppSettings {
    #[serde(default)]
    minimize_to_tray_on_close: bool,
    #[serde(default = "default_show_shifted_number_symbols")]
    show_shifted_number_symbols: bool,
    #[serde(default = "default_layer_hover_preview")]
    layer_hover_preview: bool,
    #[serde(default)]
    sticky_layout_window: bool,
    #[serde(default = "default_sticky_layout_always_on_top")]
    sticky_layout_always_on_top: bool,
    #[serde(default = "default_sticky_layout_opacity")]
    sticky_layout_opacity: f32,
    #[serde(default)]
    sticky_layout_dark_mode: bool,
    #[serde(default)]
    sticky_layout_window_size: Option<[f32; 2]>,
    #[serde(default = "crate::i18n::default_language")]
    language: crate::i18n::Language,
    #[serde(default = "default_encoder_hover_enlarge")]
    encoder_hover_enlarge: bool,
    #[serde(default)]
    key_legend_layout: KeyLegendLayout,
    #[serde(default = "default_app_accent_color")]
    accent_color: AppAccentColor,
    #[serde(default = "default_ui_scale")]
    ui_scale: f32,
    #[serde(default)]
    onboarding_tour_seen_version: u16,
    #[serde(default)]
    text_expander_enabled: bool,
    #[serde(default)]
    text_expander_app_blacklist: String,
    #[serde(default)]
    text_expander_rule_files: Vec<String>,
    #[serde(default)]
    text_expansion_rules: Vec<crate::text_expander::TextExpansionRule>,
}

fn default_show_shifted_number_symbols() -> bool {
    true
}

fn default_layer_hover_preview() -> bool {
    true
}

fn default_encoder_hover_enlarge() -> bool {
    true
}

fn default_sticky_layout_always_on_top() -> bool {
    true
}

fn default_sticky_layout_opacity() -> f32 {
    1.0
}

fn default_app_accent_color() -> AppAccentColor {
    AppAccentColor::Rose
}

fn default_ui_scale() -> f32 {
    1.0
}

fn clamp_ui_scale(scale: f32) -> f32 {
    let scale = if scale.is_finite() {
        scale
    } else {
        default_ui_scale()
    };
    (scale / UI_SCALE_STEP)
        .round()
        .mul_add(UI_SCALE_STEP, 0.0)
        .clamp(UI_SCALE_MIN, UI_SCALE_MAX)
}

fn responsive_settings_editor_scale(ctx: &egui::Context) -> f32 {
    crate::ui_style::ResponsiveMetrics::from_ctx(ctx).scale
}

fn responsive_settings_visible_rows(
    ctx: &egui::Context,
    available_height: f32,
    total_rows: usize,
    bottom_reserve: f32,
) -> usize {
    const BASE_ROWS: usize = 6;
    const MAX_ROWS: usize = 11;
    const EXTRA_ROW_START_PHYSICAL_HEIGHT: f32 = 1_300.0;
    const EXTRA_ROW_STEP_PHYSICAL_HEIGHT: f32 = 180.0;

    if total_rows == 0 {
        return 1;
    }

    let native_scale = ctx
        .native_pixels_per_point()
        .unwrap_or_else(|| ctx.pixels_per_point() / ctx.zoom_factor().max(0.1))
        .max(1.0);
    let logical_height = available_height.max(ctx.screen_rect().height());
    let usable_physical_height = (logical_height - bottom_reserve).max(0.0) * native_scale;
    let extra_rows = ((usable_physical_height - EXTRA_ROW_START_PHYSICAL_HEIGHT)
        / EXTRA_ROW_STEP_PHYSICAL_HEIGHT)
        .floor()
        .max(0.0) as usize;
    (BASE_ROWS + extra_rows).clamp(1, MAX_ROWS).min(total_rows)
}

struct AdaptiveSettingsListViewport {
    viewport: egui::Rect,
    content_rect: egui::Rect,
    track_rect: egui::Rect,
    handle_height: f32,
    scroll_ratio: f32,
    track_hovered: bool,
    suppress_tooltips: bool,
    first_visible_row: usize,
    last_visible_row: usize,
    row_content_width: f32,
    row_height: f32,
    has_scrollbar: bool,
}

fn allocate_adaptive_settings_list_viewport(
    ui: &mut egui::Ui,
    id_salt: &'static str,
    metrics: crate::ui_style::ResponsiveMetrics,
    total_rows: usize,
    bottom_reserve: f32,
) -> AdaptiveSettingsListViewport {
    let viewport_width = metrics.settings_content_width();
    let row_content_width = metrics.settings_row_content_width();
    let row_height = metrics.settings_row_height();
    let visible_rows = responsive_settings_visible_rows(
        ui.ctx(),
        ui.available_height(),
        total_rows,
        bottom_reserve,
    );
    let list_height = row_height * visible_rows as f32;
    let content_height = row_height * total_rows as f32;
    let max_offset = (content_height - list_height).max(0.0);
    let offset_id = ui.id().with((id_salt, "smooth_offset"));
    let target_id = ui.id().with((id_salt, "smooth_target"));
    let mut scroll_offset = ui
        .ctx()
        .data_mut(|d| d.get_persisted::<f32>(offset_id).unwrap_or(0.0))
        .clamp(0.0, max_offset);
    let mut target_offset = ui
        .ctx()
        .data_mut(|d| d.get_persisted::<f32>(target_id).unwrap_or(scroll_offset))
        .clamp(0.0, max_offset);
    let (viewport, _) =
        ui.allocate_exact_size(egui::vec2(viewport_width, list_height), Sense::hover());

    let track_width = metrics.value(6.0);
    let track_rect = egui::Rect::from_min_max(
        egui::pos2(viewport.right() - track_width, viewport.top()),
        egui::pos2(viewport.right(), viewport.bottom()),
    );
    let scrollbar_resp = if max_offset > 0.0 {
        Some(ui.interact(
            track_rect.expand2(egui::vec2(metrics.value(5.0), 0.0)),
            ui.id().with((id_salt, "scrollbar")),
            Sense::click_and_drag(),
        ))
    } else {
        None
    };

    let mut scroll_active = false;
    let popup_open = ui.memory(|m| m.any_popup_open());
    let viewport_hovered = !popup_open
        && ui.input(|i| {
            i.pointer
                .hover_pos()
                .is_some_and(|pos| viewport.contains(pos))
        });
    let scroll_delta = if viewport_hovered {
        ui.input(|i| {
            if i.smooth_scroll_delta.y.abs() > 0.0 {
                i.smooth_scroll_delta.y
            } else {
                i.raw_scroll_delta.y
            }
        })
    } else {
        0.0
    };
    if scroll_delta.abs() > 0.0 && max_offset > 0.0 {
        scroll_active = true;
        target_offset = (target_offset - scroll_delta * 0.72).clamp(0.0, max_offset);
    }

    let handle_height = if max_offset > 0.0 {
        (list_height / content_height * viewport.height())
            .clamp(metrics.value(42.0), viewport.height())
    } else {
        viewport.height()
    };
    if let Some(resp) = &scrollbar_resp {
        if (resp.dragged() || resp.clicked()) && max_offset > 0.0 {
            if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                scroll_active = true;
                let travel = (track_rect.height() - handle_height).max(1.0);
                let t = ((pointer_pos.y - track_rect.top() - handle_height / 2.0) / travel)
                    .clamp(0.0, 1.0);
                target_offset = t * max_offset;
                scroll_offset = target_offset;
            }
        }
    }

    if (scroll_offset - target_offset).abs() > 0.35 {
        scroll_offset += (target_offset - scroll_offset) * 0.42;
        scroll_active = true;
        ui.ctx().request_repaint();
    } else {
        scroll_offset = target_offset;
    }
    scroll_offset = scroll_offset.clamp(0.0, max_offset);
    target_offset = target_offset.clamp(0.0, max_offset);
    ui.ctx().data_mut(|d| {
        d.insert_persisted(offset_id, scroll_offset);
        d.insert_persisted(target_id, target_offset);
    });

    let first_visible_row = (scroll_offset / row_height).floor() as usize;
    let row_y_offset = scroll_offset - first_visible_row as f32 * row_height;
    let last_visible_row = (first_visible_row + visible_rows + 1).min(total_rows);
    let visible_row_count = last_visible_row.saturating_sub(first_visible_row);
    let content_rect = egui::Rect::from_min_size(
        egui::pos2(viewport.left(), viewport.top() - row_y_offset),
        egui::vec2(row_content_width, row_height * visible_row_count as f32),
    );
    let track_hovered = scrollbar_resp
        .as_ref()
        .map(|resp| resp.hovered() || resp.dragged())
        .unwrap_or(false);

    AdaptiveSettingsListViewport {
        viewport,
        content_rect,
        track_rect,
        handle_height,
        scroll_ratio: if max_offset > 0.0 {
            scroll_offset / max_offset
        } else {
            0.0
        },
        track_hovered,
        suppress_tooltips: scroll_active || ui.input(|i| i.pointer.primary_down()),
        first_visible_row,
        last_visible_row,
        row_content_width,
        row_height,
        has_scrollbar: max_offset > 0.0,
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            minimize_to_tray_on_close: false,
            show_shifted_number_symbols: default_show_shifted_number_symbols(),
            layer_hover_preview: default_layer_hover_preview(),
            sticky_layout_window: false,
            sticky_layout_always_on_top: default_sticky_layout_always_on_top(),
            sticky_layout_opacity: default_sticky_layout_opacity(),
            sticky_layout_dark_mode: false,
            sticky_layout_window_size: None,
            language: crate::i18n::default_language(),
            encoder_hover_enlarge: default_encoder_hover_enlarge(),
            key_legend_layout: KeyLegendLayout::default(),
            accent_color: default_app_accent_color(),
            ui_scale: default_ui_scale(),
            onboarding_tour_seen_version: 0,
            text_expander_enabled: false,
            text_expander_app_blacklist: String::new(),
            text_expander_rule_files: Vec::new(),
            text_expansion_rules: Vec::new(),
        }
    }
}

fn parse_text_expansion_rules_json(
    data: &str,
) -> Option<Vec<crate::text_expander::TextExpansionRule>> {
    if let Ok(rules) = serde_json::from_str::<Vec<crate::text_expander::TextExpansionRule>>(data) {
        return Some(rules);
    }

    let value = serde_json::from_str::<serde_json::Value>(data).ok()?;
    for key in ["rules", "text_expansion_rules"] {
        if let Some(rules_value) = value.get(key) {
            if let Ok(rules) = serde_json::from_value::<Vec<crate::text_expander::TextExpansionRule>>(
                rules_value.clone(),
            ) {
                return Some(rules);
            }
        }
    }
    None
}

fn load_text_expansion_rules() -> Option<Vec<crate::text_expander::TextExpansionRule>> {
    load_text_expansion_rules_from_path(&text_expander_rules_path())
}

fn load_text_expansion_rules_from_path(
    path: &std::path::Path,
) -> Option<Vec<crate::text_expander::TextExpansionRule>> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|data| parse_text_expansion_rules_json(&data))
}

fn load_extra_text_expansion_rules(
    files: &[String],
) -> Vec<crate::text_expander::TextExpansionRule> {
    normalize_text_expander_rule_files(files)
        .iter()
        .filter_map(|file| {
            load_text_expansion_rules_from_path(&text_expander_extra_rules_path(file))
        })
        .flatten()
        .collect()
}

fn save_text_expansion_rules(rules: &[crate::text_expander::TextExpansionRule]) {
    match serde_json::to_string_pretty(rules) {
        Ok(json) => {
            if let Err(e) = std::fs::write(text_expander_rules_path(), json) {
                log::warn!("save_text_expansion_rules failed: {e}");
            }
        }
        Err(e) => log::warn!("save_text_expansion_rules serialize failed: {e}"),
    }
}

fn ensure_text_expander_rules_file(rules: &[crate::text_expander::TextExpansionRule]) {
    let path = text_expander_rules_path();
    if !path.exists() {
        save_text_expansion_rules(rules);
    }
}

fn open_path_in_system_editor(path: &std::path::Path) -> bool {
    #[cfg(target_os = "windows")]
    {
        return std::process::Command::new("cmd")
            .args(["/C", "start", "", &path.to_string_lossy()])
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
    }
    #[cfg(target_os = "macos")]
    {
        return std::process::Command::new("open")
            .arg(path)
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
    }
    #[cfg(target_os = "linux")]
    {
        return std::process::Command::new("xdg-open")
            .arg(path)
            .status()
            .map(|status| status.success())
            .unwrap_or(false);
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let _ = path;
        false
    }
}

fn load_app_settings() -> AppSettings {
    migrate_legacy_text_expander_rules_file();
    let path = app_settings_path();
    let settings_data = std::fs::read_to_string(&path).ok();
    let mut settings = settings_data
        .as_deref()
        .and_then(|data| serde_json::from_str::<AppSettings>(data).ok())
        .unwrap_or_default();

    let embedded_rules = settings.text_expansion_rules.clone();
    match load_text_expansion_rules() {
        Some(rules) if !rules.is_empty() => settings.text_expansion_rules = rules,
        Some(_) if !embedded_rules.is_empty() => {
            settings.text_expansion_rules = embedded_rules;
            save_text_expansion_rules(&settings.text_expansion_rules);
        }
        Some(rules) => settings.text_expansion_rules = rules,
        None => save_text_expansion_rules(&settings.text_expansion_rules),
    }
    settings.text_expander_rule_files =
        normalize_text_expander_rule_files(&settings.text_expander_rule_files);

    settings
}

fn save_app_settings(settings: &AppSettings) {
    match serde_json::to_string_pretty(settings) {
        Ok(json) => {
            if let Err(e) = std::fs::write(app_settings_path(), json) {
                log::warn!("save_app_settings failed: {e}");
            }
        }
        Err(e) => log::warn!("save_app_settings serialize failed: {e}"),
    }
}

fn normalize_text_expander_app_name(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_matches(['\'', '"']);
    let name = trimmed
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(trimmed)
        .trim()
        .to_ascii_lowercase();
    (!name.is_empty()).then_some(name)
}

fn parse_text_expander_blacklist(raw: &str) -> Vec<String> {
    let mut entries = Vec::new();
    for entry in raw.split([',', ';', '\n']) {
        let Some(name) = normalize_text_expander_app_name(entry) else {
            continue;
        };
        if !entries.iter().any(|existing| existing == &name) {
            entries.push(name);
        }
    }
    entries
}

fn format_text_expander_blacklist(entries: &[String]) -> String {
    entries.join(", ")
}

fn compact_dropdown_popup_height(option_count: usize, option_height: f32, spacing_y: f32) -> f32 {
    let visible = option_count.max(1).min(5) as f32;
    option_height * visible + spacing_y * (visible - 1.0).max(0.0)
}

fn load_saved_layer_names(device_name: &str) -> Option<Vec<String>> {
    let path = layer_names_path(device_name);
    let data = std::fs::read_to_string(&path).ok()?;
    let mut v = serde_json::from_str::<Vec<String>>(&data).ok()?;
    if v.is_empty() {
        return None;
    }
    while v.len() < 16 {
        let n = v.len();
        v.push(n.to_string());
    }
    Some(v)
}

fn load_layer_names(device_name: &str) -> Vec<String> {
    if let Some(v) = load_saved_layer_names(device_name) {
        return v;
    }
    let mut v: Vec<String> = (0..16).map(|i| i.to_string()).collect();
    v[0] = "Main".to_string();
    v
}

fn encoder_visibility_path(device_name: &str) -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    std::fs::create_dir_all(&dir).ok();
    let slug = device_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();
    dir.join(format!("encoder_visibility_{}.json", slug))
}

fn load_encoder_visibility(device_name: &str, count: usize) -> Vec<bool> {
    if count == 0 {
        return vec![];
    }
    let path = encoder_visibility_path(device_name);
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(mut v) = serde_json::from_str::<Vec<bool>>(&data) {
            v.truncate(count);
            while v.len() < count {
                v.push(true);
            }
            return v;
        }
    }
    vec![true; count]
}

fn save_encoder_visibility(visibility: &[bool], device_name: &str) {
    let path = encoder_visibility_path(device_name);
    match serde_json::to_string_pretty(visibility) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                log::warn!("save_encoder_visibility failed: {}", e);
            }
        }
        Err(e) => log::warn!("save_encoder_visibility serialize failed: {}", e),
    }
}

fn save_layer_names(names: &[String], device_name: &str) {
    // Always save at least 16 slots so load_layer_names can detect a valid file
    let mut full = names.to_vec();
    while full.len() < 16 {
        let n = full.len();
        full.push(n.to_string());
    }
    if let Ok(data) = serde_json::to_string(&full) {
        let path = layer_names_path(device_name);
        if let Err(e) = std::fs::write(&path, &data) {
            log::warn!("save_layer_names failed at {:?}: {e}", path);
        } else {
            log::info!("save_layer_names ok → {:?}", path);
        }
    }
}

fn tap_dance_names_path(device_name: &str) -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    std::fs::create_dir_all(&dir).ok();
    let slug = device_id_slug(device_name);
    dir.join(format!("tap_dance_names_{}.json", slug))
}

fn load_tap_dance_names(device_name: &str) -> Vec<String> {
    let path = tap_dance_names_path(device_name);
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<Vec<String>>(&data) {
            return v;
        }
    }
    vec![]
}

fn save_tap_dance_names(names: &[String], device_name: &str) {
    if let Ok(data) = serde_json::to_string(names) {
        let path = tap_dance_names_path(device_name);
        if let Err(e) = std::fs::write(&path, &data) {
            log::warn!("save_tap_dance_names failed at {:?}: {e}", path);
        } else {
            log::info!("save_tap_dance_names ok → {:?}", path);
        }
    }
}

fn combo_names_path(device_name: &str) -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    std::fs::create_dir_all(&dir).ok();
    let slug = device_id_slug(device_name);
    dir.join(format!("combo_names_{}.json", slug))
}

fn load_combo_names(device_name: &str) -> Vec<String> {
    let path = combo_names_path(device_name);
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<Vec<String>>(&data) {
            return v;
        }
    }
    vec![]
}

fn save_combo_names(names: &[String], device_name: &str) {
    if let Ok(data) = serde_json::to_string(names) {
        let path = combo_names_path(device_name);
        if let Err(e) = std::fs::write(&path, &data) {
            log::warn!("save_combo_names failed at {:?}: {e}", path);
        } else {
            log::info!("save_combo_names ok → {:?}", path);
        }
    }
}

fn combo_display_name(combo_names: &[String], idx: usize) -> String {
    match combo_names.get(idx) {
        Some(name) if !name.trim().is_empty() => name.clone(),
        _ => format!("C{}", idx),
    }
}

fn key_override_names_path(device_name: &str) -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    std::fs::create_dir_all(&dir).ok();
    let slug = device_id_slug(device_name);
    dir.join(format!("key_override_names_{}.json", slug))
}

fn load_key_override_names(device_name: &str) -> Vec<String> {
    let path = key_override_names_path(device_name);
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<Vec<String>>(&data) {
            return v;
        }
    }
    vec![]
}

fn save_key_override_names(names: &[String], device_name: &str) {
    if let Ok(data) = serde_json::to_string(names) {
        let path = key_override_names_path(device_name);
        if let Err(e) = std::fs::write(&path, &data) {
            log::warn!("save_key_override_names failed at {:?}: {e}", path);
        } else {
            log::info!("save_key_override_names ok → {:?}", path);
        }
    }
}

fn key_override_display_name(key_override_names: &[String], idx: usize) -> String {
    match key_override_names.get(idx) {
        Some(name) if !name.trim().is_empty() => name.clone(),
        _ => format!("KO{}", idx),
    }
}

fn alt_repeat_names_path(device_name: &str) -> std::path::PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("entropy");
    std::fs::create_dir_all(&dir).ok();
    let slug = device_id_slug(device_name);
    dir.join(format!("alt_repeat_names_{}.json", slug))
}

fn load_alt_repeat_names(device_name: &str) -> Vec<String> {
    let path = alt_repeat_names_path(device_name);
    if let Ok(data) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<Vec<String>>(&data) {
            return v;
        }
    }
    vec![]
}

fn save_alt_repeat_names(names: &[String], device_name: &str) {
    if let Ok(data) = serde_json::to_string(names) {
        let path = alt_repeat_names_path(device_name);
        if let Err(e) = std::fs::write(&path, &data) {
            log::warn!("save_alt_repeat_names failed at {:?}: {e}", path);
        } else {
            log::info!("save_alt_repeat_names ok → {:?}", path);
        }
    }
}

fn macro_custom_name(macro_names: &[String], idx: usize) -> Option<String> {
    macro_names
        .get(idx)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn macro_display_name(macro_names: &[String], idx: usize) -> String {
    macro_custom_name(macro_names, idx).unwrap_or_else(|| format!("M{}", idx))
}

fn tap_dance_custom_name(tap_dance_names: &[String], idx: usize) -> Option<String> {
    tap_dance_names
        .get(idx)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn tap_dance_display_name(tap_dance_names: &[String], idx: usize) -> String {
    tap_dance_custom_name(tap_dance_names, idx).unwrap_or_else(|| format!("TD{}", idx))
}

fn app_accent() -> Color32 {
    crate::ui_style::accent()
}
fn app_panel_fill(dark: bool) -> Color32 {
    crate::ui_style::panel_fill(dark)
}
fn app_window_fill(dark: bool) -> Color32 {
    crate::ui_style::window_fill(dark)
}
fn app_surface_fill(dark: bool) -> Color32 {
    crate::ui_style::surface_fill(dark)
}
fn app_hover_fill(dark: bool) -> Color32 {
    crate::ui_style::hover_fill(dark)
}
fn app_border_color(dark: bool) -> Color32 {
    crate::ui_style::border_color(dark)
}
fn app_muted_text(dark: bool) -> Color32 {
    crate::ui_style::muted_text(dark)
}

fn app_inactive_entry_text(dark: bool) -> Color32 {
    if dark {
        Color32::from_gray(105)
    } else {
        Color32::from_gray(165)
    }
}

fn keycode_label_with_macro_names(
    value: u16,
    custom: &[crate::keyboard::CustomKeycode],
    layer_names: &[String],
    macro_names: &[String],
    tap_dance_names: &[String],
    key_legend_layout: KeyLegendLayout,
) -> String {
    if (0x7700..=0x77FF).contains(&value) {
        let idx = (value - 0x7700) as usize;
        if let Some(name) = macro_custom_name(macro_names, idx) {
            return format!("M{}\n{}", idx, name);
        }
        return format!("M{}", idx);
    }
    if (0x5700..=0x57FF).contains(&value) {
        let idx = (value - 0x5700) as usize;
        if let Some(name) = tap_dance_custom_name(tap_dance_names, idx) {
            return format!("TD{}\n{}", idx, name);
        }
        return format!("TD{}", idx);
    }
    keycode_label_with_names_and_layout(value, custom, layer_names, key_legend_layout)
}

fn keycode_tooltip_with_macro_names(
    value: u16,
    custom: &[crate::keyboard::CustomKeycode],
    layer_names: &[String],
    macro_names: &[String],
    tap_dance_names: &[String],
) -> String {
    if (0x7700..=0x77FF).contains(&value) {
        let idx = (value - 0x7700) as usize;
        let name = macro_display_name(macro_names, idx);
        return format!("{} — macro {}", name, idx);
    }
    if (0x5700..=0x57FF).contains(&value) {
        let idx = (value - 0x5700) as usize;
        let name = tap_dance_display_name(tap_dance_names, idx);
        return format!("{} — tap dance {}", name, idx);
    }
    keycode_tooltip(value, custom, layer_names)
}
use crate::keyboard::{KeyboardLayout, LayoutOption, PhysicalEncoder, PhysicalKey};
use crate::keycode::{
    key_label_font_sizes, keycode_label_with_names_and_layout, keycode_tooltip, KeyLegendLayout,
};
use crate::keycode_picker::{egui_key_to_qmk, KeycodePicker, KeycodeTab};
use egui::{Color32, FontId, RichText, Sense, Stroke, Vec2};

#[path = "ui/alt_repeat_settings.rs"]
mod alt_repeat_settings_ui;
#[path = "ui/app_settings.rs"]
mod app_settings_ui;
#[path = "ui/auto_shift_settings.rs"]
mod auto_shift_settings_ui;
#[path = "ui/combo_settings.rs"]
mod combo_settings_ui;
#[path = "ui/device_connection.rs"]
mod device_connection;
#[path = "ui/device_settings_helpers.rs"]
mod device_settings_helpers;
#[path = "ui/encoder_visibility_settings.rs"]
mod encoder_visibility_settings_ui;
#[path = "ui/grave_escape_settings.rs"]
mod grave_escape_settings_ui;
#[path = "ui/key_assignment.rs"]
mod key_assignment;
#[path = "ui/key_override_settings.rs"]
mod key_override_settings_ui;
#[path = "ui/layer_led_settings.rs"]
mod layer_led_settings_ui;
#[path = "ui/layout_indicator.rs"]
mod layout_indicator;
#[path = "ui/layout_options_settings.rs"]
mod layout_options_settings_ui;
#[path = "ui/layout_shared.rs"]
mod layout_shared;
use layout_shared::*;
#[path = "ui/top_dropdown.rs"]
mod top_dropdown;
use top_dropdown::*;
#[path = "ui/layout_view.rs"]
mod layout_view;
#[path = "ui/live_features_settings.rs"]
mod live_features_settings_ui;
#[path = "ui/magic_settings.rs"]
mod magic_settings_ui;
#[path = "ui/matrix_tester.rs"]
mod matrix_tester;
#[path = "ui/module_settings.rs"]
mod module_settings_ui;
#[path = "ui/mouse_keys_settings.rs"]
mod mouse_keys_settings_ui;
#[path = "ui/onboarding_tour.rs"]
mod onboarding_tour;
#[path = "ui/rgb_settings.rs"]
mod rgb_settings_ui;
#[path = "ui/tap_hold_settings.rs"]
mod tap_hold_settings_ui;
#[path = "ui/text_expander_settings.rs"]
mod text_expander_settings_ui;
#[path = "ui/touchpad_settings.rs"]
mod touchpad_settings_ui;
#[path = "ui/ui_scale.rs"]
mod ui_scale;
#[path = "ui/universal_symbols_setup.rs"]
mod universal_symbols_setup;
#[path = "vial/unlock.rs"]
mod vial_unlock;
#[path = "ui/window_lifecycle.rs"]
mod window_lifecycle;

const LAYOUT_BASE_UNIT: f32 = 54.0_f32 * 1.15;
const LAYOUT_KEY_PADDING: f32 = 2.5_f32;
const LAYOUT_FIT_MARGIN: f32 = 40.0_f32;
const LAYOUT_ENCODER_RADIUS_FACTOR: f32 = 0.47_f32;
const LAYOUT_ENCODER_FILL_EXTRA: f32 = 1.0_f32;
const LAYOUT_TOP_RESERVED_H: f32 = 32.0_f32 + 4.0_f32 + 68.0_f32;
const LAYOUT_BOTTOM_RESERVED_H: f32 = 76.0_f32;

fn draw_theme_selector_labels(
    ui: &mut egui::Ui,
    lang: crate::i18n::Language,
    dark_mode: &mut bool,
    dark_first: bool,
) {
    ui.horizontal(|ui| {
        let active = app_accent();
        let inactive = app_muted_text(*dark_mode);
        let mut theme_label = |ui: &mut egui::Ui, is_dark: bool| {
            let key = if is_dark {
                "app_chrome.dark_dark"
            } else {
                "app_chrome.light_light"
            };
            let selected = *dark_mode == is_dark;
            let resp = ui.add(
                egui::Label::new(
                    RichText::new(crate::i18n::tr_catalog(lang, key))
                        .size(11.0)
                        .color(if selected { active } else { inactive }),
                )
                .selectable(false)
                .sense(egui::Sense::click()),
            );
            if resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if resp.clicked() {
                *dark_mode = is_dark;
            }
        };

        let (first, second) = if dark_first {
            (true, false)
        } else {
            (false, true)
        };

        theme_label(ui, first);
        ui.add(egui::Label::new(RichText::new("|").size(11.0).color(inactive)).selectable(false));
        theme_label(ui, second);
    });
}

#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc;

#[derive(Debug, Clone, Default)]
struct VialFeatureSupport {
    caps_word: bool,
    layer_lock: bool,
    persistent_default_layer: bool,
    repeat_key: bool,
}

/// Result sent back from the background connect thread.
#[cfg(not(target_arch = "wasm32"))]
struct ConnectResult {
    device_name: String,
    layout: KeyboardLayout,
    layer_count: usize,
    /// Macro texts read from device
    macro_texts: Vec<String>,
    /// Tap dance entries
    tap_dance_entries: Vec<crate::keycode_picker::TapDanceEntry>,
    /// Combo entries
    combo_entries: Vec<ComboEntry>,
    /// Global combo timeout/term from QMK settings, if supported
    combo_term: Option<u16>,
    /// Auto Shift flags from QMK settings, if supported
    auto_shift_options: AutoShiftOptionsState,
    /// Auto Shift timeout from QMK settings, if supported
    auto_shift_timeout: Option<u16>,
    /// Mouse Keys settings from QMK settings, if supported (qsid 9..=17)
    mouse_keys_settings: MouseKeysSettingsState,
    /// Ergohaven K:03 Pro touchpad settings from QMK settings, if supported
    touchpad_settings: TouchpadSettingsState,
    /// Keyboard-specific module settings from QMK Settings, if supported
    module_settings: ModuleSettingsState,
    /// Tap-Hold settings from QMK settings, if supported
    tap_hold_settings: TapHoldSettingsState,
    /// Magic settings from QMK settings, if supported
    magic_settings: MagicSettingsState,
    /// One Shot Keys settings from QMK settings, if supported
    one_shot_settings: OneShotSettingsState,
    /// Grave Escape settings from QMK settings, if supported (qsid 1 bits 0..=3)
    grave_escape_settings: GraveEscapeSettingsState,
    /// Ergohaven per-layer LED settings from QMK settings, if supported (qsid 300..=317)
    layer_led_settings: LayerLedSettingsState,
    /// Runtime RGB settings, if supported by the current Vial/QMK lighting backend
    rgb_settings: RgbSettingsState,
    /// Vial layout/display option bitfield, if exposed by `layouts.labels`
    layout_options_value: Option<u32>,
    /// Key Override entries
    key_override_entries: Vec<KeyOverrideEntry>,
    /// Alt Repeat entries
    alt_repeat_entries: Vec<AltRepeatKeyEntry>,
    /// Feature bits reported by Vial dynamic entries.
    vial_features: VialFeatureSupport,
}

#[cfg(not(target_arch = "wasm32"))]
enum ConnectState {
    Idle,
    Loading(mpsc::Receiver<Result<ConnectResult, String>>),
}

#[cfg(not(target_arch = "wasm32"))]
enum DeviceScanState {
    Idle,
    Scanning(mpsc::Receiver<Vec<Device>>),
}

fn toggle_handed_modifier(value: u16) -> Option<u16> {
    match value {
        0x00E0 => Some(0x00E4),
        0x00E4 => Some(0x00E0),
        0x00E1 => Some(0x00E5),
        0x00E5 => Some(0x00E1),
        0x00E2 => Some(0x00E6),
        0x00E6 => Some(0x00E2),
        0x00E3 => Some(0x00E7),
        0x00E7 => Some(0x00E3),
        0x52A1 => Some(0x52B1),
        0x52B1 => Some(0x52A1),
        0x52A2 => Some(0x52B2),
        0x52B2 => Some(0x52A2),
        0x52A4 => Some(0x52B4),
        0x52B4 => Some(0x52A4),
        0x52A8 => Some(0x52B8),
        0x52B8 => Some(0x52A8),
        _ => {
            let base = value & 0xFF00;
            let low = value & 0x00FF;
            match base {
                0x2100 => Some(0x3100 | low),
                0x3100 => Some(0x2100 | low),
                0x2200 => Some(0x3200 | low),
                0x3200 => Some(0x2200 | low),
                0x2400 => Some(0x3400 | low),
                0x3400 => Some(0x2400 | low),
                0x2800 => Some(0x3800 | low),
                0x3800 => Some(0x2800 | low),
                _ => None,
            }
        }
    }
}

fn vial_layer_target(kc: u16) -> Option<usize> {
    if (0x5200..0x5300).contains(&kc) {
        let op = (kc >> 5) & 0x7;
        // QK_ONE_SHOT_MOD also lives in the 0x52xx range (op=5), but it is a
        // modifier keycode, not a layer key. Do not preview/jump layers for OSM.
        (op != 5).then_some((kc & 0x1F) as usize)
    } else if kc & 0xF000 == 0x4000 {
        Some(((kc >> 8) & 0xF) as usize)
    } else {
        None
    }
}

fn vial_layer_op_target(kc: u16) -> Option<(u16, usize)> {
    if (0x5200..0x5300).contains(&kc) {
        let op = (kc >> 5) & 0x7;
        (op != 5).then_some((op, (kc & 0x1F) as usize))
    } else {
        None
    }
}

fn vial_layer_retarget_base(kc: u16) -> Option<u16> {
    if (0x5200..0x5300).contains(&kc) {
        let op = (kc >> 5) & 0x7;
        (op != 5).then_some(kc & 0xFFE0)
    } else if kc & 0xF000 == 0x4000 {
        Some(0x4000)
    } else {
        None
    }
}

#[derive(Clone, Debug, Default)]
struct ComboEntry {
    keys: [u16; 4],
    output: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct KeyOverrideOptionsState {
    activation_trigger_down: bool,
    activation_required_mod_down: bool,
    activation_negative_mod_up: bool,
    one_mod: bool,
    no_reregister_trigger: bool,
    no_unregister_on_other_key_down: bool,
    enabled: bool,
}

impl KeyOverrideOptionsState {
    fn from_bits(bits: u8) -> Self {
        Self {
            activation_trigger_down: bits & (1 << 0) != 0,
            activation_required_mod_down: bits & (1 << 1) != 0,
            activation_negative_mod_up: bits & (1 << 2) != 0,
            one_mod: bits & (1 << 3) != 0,
            no_reregister_trigger: bits & (1 << 4) != 0,
            no_unregister_on_other_key_down: bits & (1 << 5) != 0,
            enabled: bits & (1 << 7) != 0,
        }
    }

    fn bits(&self) -> u8 {
        (self.activation_trigger_down as u8) << 0
            | (self.activation_required_mod_down as u8) << 1
            | (self.activation_negative_mod_up as u8) << 2
            | (self.one_mod as u8) << 3
            | (self.no_reregister_trigger as u8) << 4
            | (self.no_unregister_on_other_key_down as u8) << 5
            | (self.enabled as u8) << 7
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct KeyOverrideEntry {
    trigger: u16,
    replacement: u16,
    layers: u16,
    trigger_mods: u8,
    negative_mod_mask: u8,
    suppressed_mods: u8,
    options: KeyOverrideOptionsState,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct AltRepeatKeyOptionsState {
    default_to_this_alt_key: bool,
    bidirectional: bool,
    ignore_mod_handedness: bool,
    enabled: bool,
}

impl AltRepeatKeyOptionsState {
    fn from_bits(bits: u8) -> Self {
        Self {
            default_to_this_alt_key: bits & (1 << 0) != 0,
            bidirectional: bits & (1 << 1) != 0,
            ignore_mod_handedness: bits & (1 << 2) != 0,
            enabled: bits & (1 << 3) != 0,
        }
    }

    fn bits(self) -> u8 {
        (self.default_to_this_alt_key as u8)
            | ((self.bidirectional as u8) << 1)
            | ((self.ignore_mod_handedness as u8) << 2)
            | ((self.enabled as u8) << 3)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct AltRepeatKeyEntry {
    keycode: u16,
    alt_keycode: u16,
    allowed_mods: u8,
    options: AltRepeatKeyOptionsState,
}

#[derive(Clone, Copy, Debug, Default)]
struct AutoShiftOptionsState {
    enabled: bool,
    enable_for_modifiers: bool,
    no_special: bool,
    no_numeric: bool,
    no_alpha: bool,
    enable_keyrepeat: bool,
    disable_keyrepeat_timeout: bool,
}

impl AutoShiftOptionsState {
    fn from_bits(bits: u8) -> Self {
        Self {
            enabled: bits & (1 << 0) != 0,
            enable_for_modifiers: bits & (1 << 1) != 0,
            no_special: bits & (1 << 2) != 0,
            no_numeric: bits & (1 << 3) != 0,
            no_alpha: bits & (1 << 4) != 0,
            enable_keyrepeat: bits & (1 << 5) != 0,
            disable_keyrepeat_timeout: bits & (1 << 6) != 0,
        }
    }

    fn bits(self) -> u8 {
        (self.enabled as u8)
            | ((self.enable_for_modifiers as u8) << 1)
            | ((self.no_special as u8) << 2)
            | ((self.no_numeric as u8) << 3)
            | ((self.no_alpha as u8) << 4)
            | ((self.enable_keyrepeat as u8) << 5)
            | ((self.disable_keyrepeat_timeout as u8) << 6)
    }
}

/// Mirrors Vial GUI Mouse keys settings (qsid 9..=17). All values are u16.
#[derive(Clone, Copy, Debug, Default)]
struct MouseKeysSettingsState {
    /// qsid 9: Delay between pressing a movement key and cursor movement
    delay: u16,
    /// qsid 10: Time between cursor movements in milliseconds
    interval: u16,
    /// qsid 11: Step size
    max_speed: u16,
    /// qsid 12: Maximum cursor speed at which acceleration stops
    time_to_max: u16,
    /// qsid 13: Time until maximum cursor speed is reached
    move_delta: u16,
    /// qsid 14: Delay between pressing a wheel key and wheel movement
    wheel_delay: u16,
    /// qsid 15: Time between wheel movements
    wheel_interval: u16,
    /// qsid 16: Maximum number of scroll steps per scroll action
    wheel_max_speed: u16,
    /// qsid 17: Time until maximum scroll speed is reached
    wheel_time_to_max: u16,
    /// Whether any of the qsids were readable (firmware support flag)
    supported: bool,
}

/// Ergohaven K:03 Pro touchpad settings exposed by firmware QMK Settings.
#[derive(Clone, Debug, Default)]
struct TouchpadSettingsState {
    /// qsid 120: touchpad DPI/CPI, either direct value or select index depending on definition
    dpi: u16,
    /// qsid 120 variants when the firmware exposes DPI as a select setting
    dpi_variants: Vec<String>,
    /// qsid 121: sensitivity in sniper mode
    sniper_sens: u8,
    /// qsid 122: sensitivity in scroll mode
    scroll_sens: u8,
    /// qsid 123: sensitivity in text mode
    text_sens: u8,
    /// qsid 124 bits 0..=2: invert scroll, acceleration, sticky mode
    bits: u8,
    /// qsid 142: auto layer enable, if exposed by this firmware
    auto_layer_enable: bool,
    /// Whether qsid 142 is exposed by this firmware
    auto_layer_enable_supported: bool,
    /// qsid 143: auto layer select, if exposed by this firmware
    auto_layer: u8,
    /// qsid 143 variants when exposed by this firmware
    auto_layer_variants: Vec<String>,
    /// Whether qsid 120..124 were readable and advertised by firmware definition/query
    supported: bool,
}

impl TouchpadSettingsState {
    fn bit(&self, bit: u8) -> bool {
        self.bits & (1 << bit) != 0
    }

    fn set_bit(&mut self, bit: u8, enabled: bool) {
        if enabled {
            self.bits |= 1 << bit;
        } else {
            self.bits &= !(1 << bit);
        }
    }

    fn auto_layer_supported(&self) -> bool {
        self.auto_layer_enable_supported && !self.auto_layer_variants.is_empty()
    }

    fn row_count(&self) -> usize {
        7 + self.auto_layer_enable_supported as usize + self.auto_layer_supported() as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModuleSettingKind {
    Boolean,
    Integer,
    Select,
}

#[derive(Clone, Debug)]
struct ModuleSettingField {
    title: String,
    qsid: u16,
    kind: ModuleSettingKind,
    bit: u8,
    width: u8,
    min: u16,
    max: u16,
    variants: Vec<String>,
}

/// Keyboard-specific module settings exposed by firmware QMK Settings.
#[derive(Clone, Debug, Default)]
struct ModuleSettingsState {
    fields: Vec<ModuleSettingField>,
    values: std::collections::BTreeMap<u16, u16>,
    supported: bool,
}

impl ModuleSettingsState {
    fn row_count(&self) -> usize {
        self.fields.len()
    }

    fn value(&self, qsid: u16) -> u16 {
        self.values.get(&qsid).copied().unwrap_or(0)
    }

    fn set_value(&mut self, qsid: u16, value: u16) {
        self.values.insert(qsid, value);
    }
}

/// Mirrors Vial GUI Tap-Hold settings. Values are QMK settings qsids.
#[derive(Clone, Copy, Debug, Default)]
struct TapHoldSettingsState {
    /// qsid 7: Global tap-vs-hold decision window in milliseconds
    tapping_term: u16,
    /// qsid 22: Prefer hold for nested taps
    permissive_hold: bool,
    /// qsid 23: Prefer hold as soon as another key is pressed
    hold_on_other_key_press: bool,
    /// qsid 24: Send tap when a dual-role key is held and released alone
    retro_tapping: bool,
    /// qsid 25: Tap-then-hold repeat window in milliseconds
    quick_tap_term: u16,
    /// qsid 18: Delay between register_code and unregister_code in tap_code
    tap_code_delay: u16,
    /// qsid 19: Delay for LT/MT keys when tap key is KC_CAPS_LOCK
    tap_hold_caps_delay: u16,
    /// qsid 20: Number of taps needed for TT(layer) toggle
    tapping_toggle: u16,
    /// qsid 26: Same-hand chords prefer tap for tap-hold keys
    chordal_hold: bool,
    /// qsid 27: Fast-typing timeout that forces MT/LT tap behavior
    flow_tap: u16,
    /// Whether qsid 7 was readable (firmware support flag)
    supported: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct MagicSettingsState {
    /// qsid 21 bits 0..=9: QMK Magic runtime swaps/options
    bits: u16,
    /// Whether qsid 21 was readable (firmware support flag)
    supported: bool,
}

impl MagicSettingsState {
    fn bit(self, bit: u8) -> bool {
        self.bits & (1u16 << bit) != 0
    }

    fn set_bit(&mut self, bit: u8, enabled: bool) {
        if enabled {
            self.bits |= 1u16 << bit;
        } else {
            self.bits &= !(1u16 << bit);
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct OneShotSettingsState {
    /// qsid 5: Tap count that makes a one-shot key stay held until tapped again
    tap_toggle: u8,
    /// qsid 6: Timeout in milliseconds before one-shot state is released
    timeout: u16,
    /// Whether qsid 5 was readable (firmware support flag)
    supported: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct GraveEscapeSettingsState {
    /// qsid 1 bits 0..=3: force Esc when Alt/Ctrl/GUI/Shift is held for KC_GESC.
    bits: u8,
    /// Whether qsid 1 was readable (firmware support flag)
    supported: bool,
}

impl GraveEscapeSettingsState {
    fn bit(self, bit: u8) -> bool {
        self.bits & (1 << bit) != 0
    }

    fn set_bit(&mut self, bit: u8, enabled: bool) {
        if enabled {
            self.bits |= 1 << bit;
        } else {
            self.bits &= !(1 << bit);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LayerLedSettingsState {
    /// qsid 300..=315: palette color index for each logical layer
    layer_colors: [u8; 16],
    /// qsid 316: global LED brightness, clamped by firmware to 0..=255
    brightness: u16,
    /// qsid 317: timeout in minutes, 0 disables timeout
    timeout_mins: u8,
    /// Whether qsid 300 was readable (firmware support flag)
    supported: bool,
}

impl Default for LayerLedSettingsState {
    fn default() -> Self {
        Self {
            layer_colors: [0; 16],
            brightness: 0,
            timeout_mins: 0,
            supported: false,
        }
    }
}

const LAYER_LED_PALETTE: [&str; 25] = [
    "Off",
    "White",
    "Red",
    "Orange",
    "Goldenrod",
    "Gold",
    "Yellow",
    "Chartreuse",
    "Lime",
    "Green",
    "Spring Green",
    "Turquoise",
    "Teal",
    "Cyan",
    "Azure",
    "Sky",
    "Blue",
    "Indigo",
    "Purple",
    "Magenta",
    "Pink",
    "Coral",
    "Salmon",
    "Warm White",
    "Amber",
];

fn layer_led_palette_name(index: u8) -> &'static str {
    LAYER_LED_PALETTE
        .get(index as usize)
        .copied()
        .unwrap_or("Unknown")
}

const LAYER_LED_PALETTE_HSV: [(u8, u8, u8); 25] = [
    (0, 0, 0),
    (0, 0, 255),
    (0, 255, 255),
    (16, 255, 255),
    (27, 255, 255),
    (38, 255, 255),
    (53, 255, 255),
    (74, 255, 255),
    (90, 255, 255),
    (106, 255, 255),
    (117, 255, 255),
    (128, 255, 255),
    (138, 255, 170),
    (149, 255, 255),
    (160, 255, 255),
    (165, 255, 255),
    (170, 255, 255),
    (186, 255, 255),
    (202, 255, 255),
    (213, 255, 255),
    (234, 180, 255),
    (8, 176, 255),
    (14, 128, 255),
    (32, 64, 255),
    (22, 255, 255),
];

fn layer_led_palette_color(index: u8) -> Color32 {
    let (h, s, v) = LAYER_LED_PALETTE_HSV
        .get(index as usize)
        .copied()
        .unwrap_or((0, 0, 0));
    if v == 0 {
        Color32::from_rgb(18, 18, 20)
    } else {
        let pastel_s = (s as f32 / 255.0 * 0.68).clamp(0.0, 1.0);
        let pastel_v = (v as f32 / 255.0 * 0.82 + 0.12).clamp(0.0, 0.96);
        Color32::from(egui::ecolor::Hsva::new(
            h as f32 / 255.0,
            pastel_s,
            pastel_v,
            1.0,
        ))
    }
}

fn layer_led_outline_color(index: u8) -> Color32 {
    let (h, s, v) = LAYER_LED_PALETTE_HSV
        .get(index as usize)
        .copied()
        .unwrap_or((0, 0, 0));
    if v == 0 {
        Color32::from_rgb(18, 18, 20)
    } else {
        let pastel_s = (s as f32 / 255.0 * 0.26).clamp(0.0, 1.0);
        let pastel_v = (v as f32 / 255.0 * 0.48 + 0.22).clamp(0.0, 0.72);
        Color32::from(egui::ecolor::Hsva::new(
            h as f32 / 255.0,
            pastel_s,
            pastel_v,
            1.0,
        ))
    }
}

fn blend_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(mix(a.r(), b.r()), mix(a.g(), b.g()), mix(a.b(), b.b()))
}

fn layer_led_hover_fill(index: u8, dark: bool) -> Color32 {
    let (h, s, v) = LAYER_LED_PALETTE_HSV
        .get(index as usize)
        .copied()
        .unwrap_or((0, 0, 0));
    let base = crate::ui_style::hover_fill(dark);
    if v == 0 {
        base
    } else {
        let tint_s = (s as f32 / 255.0 * 0.22).clamp(0.0, 1.0);
        let tint_v = if dark { 0.36 } else { 0.92 };
        let tint = Color32::from(egui::ecolor::Hsva::new(
            h as f32 / 255.0,
            tint_s,
            tint_v,
            1.0,
        ));
        blend_color(base, tint, if dark { 0.62 } else { 0.52 })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum RgbSupportKind {
    #[default]
    None,
    QmkRgblight,
    VialRgb,
}

#[derive(Clone, Debug, Default)]
struct RgbSettingsState {
    supported: bool,
    kind: RgbSupportKind,
    effect: u16,
    brightness: u8,
    speed: u8,
    hue: u8,
    saturation: u8,
    max_brightness: u8,
    supported_effects: Vec<u16>,
    last_enabled_effect: u16,
}

impl RgbSettingsState {
    fn is_enabled(&self) -> bool {
        self.supported && self.effect != 0
    }

    fn fallback_effect(&self) -> u16 {
        match self.kind {
            RgbSupportKind::QmkRgblight => 1,
            RgbSupportKind::VialRgb => 2,
            RgbSupportKind::None => 0,
        }
    }

    fn effect_or_default(&self) -> u16 {
        let candidate = if self.last_enabled_effect != 0 {
            self.last_enabled_effect
        } else {
            self.fallback_effect()
        };
        match self.kind {
            RgbSupportKind::VialRgb => {
                if self.supported_effects.is_empty() || self.supported_effects.contains(&candidate)
                {
                    candidate
                } else {
                    self.supported_effects.first().copied().unwrap_or(candidate)
                }
            }
            _ => candidate,
        }
    }
}

const QMK_RGBLIGHT_EFFECTS: &[(u16, &str)] = &[
    (0, "All Off"),
    (1, "Solid Color"),
    (2, "Breathing 1"),
    (3, "Breathing 2"),
    (4, "Breathing 3"),
    (5, "Breathing 4"),
    (6, "Rainbow Mood 1"),
    (7, "Rainbow Mood 2"),
    (8, "Rainbow Mood 3"),
    (9, "Rainbow Swirl 1"),
    (10, "Rainbow Swirl 2"),
    (11, "Rainbow Swirl 3"),
    (12, "Rainbow Swirl 4"),
    (13, "Rainbow Swirl 5"),
    (14, "Rainbow Swirl 6"),
    (15, "Snake 1"),
    (16, "Snake 2"),
    (17, "Snake 3"),
    (18, "Snake 4"),
    (19, "Snake 5"),
    (20, "Snake 6"),
    (21, "Knight 1"),
    (22, "Knight 2"),
    (23, "Knight 3"),
    (24, "Christmas"),
    (25, "Gradient 1"),
    (26, "Gradient 2"),
    (27, "Gradient 3"),
    (28, "Gradient 4"),
    (29, "Gradient 5"),
    (30, "Gradient 6"),
    (31, "Gradient 7"),
    (32, "Gradient 8"),
    (33, "Gradient 9"),
    (34, "Gradient 10"),
    (35, "RGB Test"),
    (36, "Alternating"),
];

const VIALRGB_EFFECTS: &[(u16, &str)] = &[
    (0, "Disable"),
    (1, "Direct Control"),
    (2, "Solid Color"),
    (3, "Alphas Mods"),
    (4, "Gradient Up Down"),
    (5, "Gradient Left Right"),
    (6, "Breathing"),
    (7, "Band Sat"),
    (8, "Band Val"),
    (9, "Band Pinwheel Sat"),
    (10, "Band Pinwheel Val"),
    (11, "Band Spiral Sat"),
    (12, "Band Spiral Val"),
    (13, "Cycle All"),
    (14, "Cycle Left Right"),
    (15, "Cycle Up Down"),
    (16, "Rainbow Moving Chevron"),
    (17, "Cycle Out In"),
    (18, "Cycle Out In Dual"),
    (19, "Cycle Pinwheel"),
    (20, "Cycle Spiral"),
    (21, "Dual Beacon"),
    (22, "Rainbow Beacon"),
    (23, "Rainbow Pinwheels"),
    (24, "Raindrops"),
    (25, "Jellybean Raindrops"),
    (26, "Hue Breathing"),
    (27, "Hue Pendulum"),
    (28, "Hue Wave"),
    (29, "Typing Heatmap"),
    (30, "Digital Rain"),
    (31, "Solid Reactive Simple"),
    (32, "Solid Reactive"),
    (33, "Solid Reactive Wide"),
    (34, "Solid Reactive Multiwide"),
    (35, "Solid Reactive Cross"),
    (36, "Solid Reactive Multicross"),
    (37, "Solid Reactive Nexus"),
    (38, "Solid Reactive Multinexus"),
    (39, "Splash"),
    (40, "Multisplash"),
    (41, "Solid Splash"),
    (42, "Solid Multisplash"),
    (43, "Pixel Rain"),
    (44, "Pixel Fractal"),
];

fn load_rgb_settings(
    dev_conn: &crate::hid::HidDevice,
    layout: &KeyboardLayout,
) -> RgbSettingsState {
    let mut candidates = Vec::new();
    match layout.lighting_mode.as_deref() {
        Some("vialrgb") => {
            candidates.extend([RgbSupportKind::VialRgb, RgbSupportKind::QmkRgblight])
        }
        Some("qmk_rgblight") | Some("qmk_backlight_rgblight") => {
            candidates.extend([RgbSupportKind::QmkRgblight, RgbSupportKind::VialRgb]);
        }
        _ => {
            // Some QMK/Vial definitions do not advertise `lighting` in vial.json even
            // though the firmware still exposes runtime lighting commands. Probe both
            // backends and enable the RGB page if either one responds.
            candidates.extend([RgbSupportKind::VialRgb, RgbSupportKind::QmkRgblight]);
        }
    }

    for kind in candidates {
        match kind {
            RgbSupportKind::VialRgb => {
                let Ok((version, max_brightness)) = dev_conn.get_vialrgb_info() else {
                    continue;
                };
                if version != 1 {
                    continue;
                }
                let Ok((effect, speed, hue, saturation, brightness)) = dev_conn.get_vialrgb_mode()
                else {
                    continue;
                };
                let mut supported_effects =
                    dev_conn.get_vialrgb_supported_effects().unwrap_or_default();
                if !supported_effects.contains(&0) {
                    supported_effects.insert(0, 0);
                }
                let mut state = RgbSettingsState {
                    supported: true,
                    kind,
                    effect,
                    brightness,
                    speed,
                    hue,
                    saturation,
                    max_brightness,
                    supported_effects,
                    last_enabled_effect: effect,
                };
                if state.last_enabled_effect == 0 {
                    state.last_enabled_effect = state.fallback_effect();
                }
                return state;
            }
            RgbSupportKind::QmkRgblight => {
                let Ok(brightness) = dev_conn.get_qmk_rgblight_brightness() else {
                    continue;
                };
                let Ok(effect) = dev_conn.get_qmk_rgblight_effect() else {
                    continue;
                };
                let speed = dev_conn.get_qmk_rgblight_effect_speed().unwrap_or(0);
                let (hue, saturation) = dev_conn.get_qmk_rgblight_color().unwrap_or((0, 0));
                let mut state = RgbSettingsState {
                    supported: true,
                    kind,
                    effect: effect as u16,
                    brightness,
                    speed,
                    hue,
                    saturation,
                    max_brightness: u8::MAX,
                    supported_effects: vec![],
                    last_enabled_effect: effect as u16,
                };
                if state.last_enabled_effect == 0 {
                    state.last_enabled_effect = state.fallback_effect();
                }
                return state;
            }
            RgbSupportKind::None => {}
        }
    }

    RgbSettingsState::default()
}

/// Returns true if the given Vial keycode is a QMK mouse key (0x00CD..=0x00DF).
fn is_mouse_keycode(kc: u16) -> bool {
    (0x00CD..=0x00DF).contains(&kc)
}

fn is_alt_repeat_keycode(kc: u16) -> bool {
    kc == 0x7C7A
}

#[derive(Clone, Debug)]
enum UndoAction {
    Key {
        layer: usize,
        key_idx: usize,
        old_kc: u16,
    },
    Encoder {
        layer: usize,
        encoder_visual_idx: usize,
        old_kc: u16,
    },
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyOverridePickField {
    Trigger,
    Replacement,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AltRepeatPickField {
    LastKey,
    AltKey,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MainMenuTab {
    Keyboard,
    Advanced,
    Settings,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ComboPickField {
    Trigger(usize),
    Output,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SettingsTab {
    AppSettings,
    MatrixTester,
    UniversalSymbolsSetup,
    TextExpander,
    AutoShift,
    Rgb,
    LayerLeds,
    Encoders,
    Magic,
    TapHold,
    GraveEscape,
    LayoutOptions,
    Modules,
    Touchpad,
    LiveFeatures,
    Combo,
    KeyOverrides,
    AltRepeat,
    MouseKeys,
}

pub struct EntropyApp {
    device_manager: DeviceManager,
    selected_device: Option<usize>,
    selected_layer: usize,
    selected_key: Option<(usize, usize)>,
    selected_encoder: Option<(usize, usize)>,
    layout: Option<KeyboardLayout>,
    layer_count: usize,
    keycode_picker: KeycodePicker,
    status_msg: String,
    #[cfg(not(target_arch = "wasm32"))]
    connect_state: ConnectState,
    #[cfg(not(target_arch = "wasm32"))]
    device_scan_state: DeviceScanState,
    /// Persistent open HID device for real-time writes (Vial)
    #[cfg(not(target_arch = "wasm32"))]
    hid_device: Option<crate::hid::HidDevice>,
    /// Built-in qmk-hid-host bridges for displays/presets that need host data
    #[cfg(not(target_arch = "wasm32"))]
    qmk_hid_hosts: std::collections::HashMap<String, crate::qmk_hid_host::QmkHidHostBridge>,
    /// Current firmware type (mirrors layout.firmware)
    firmware: FirmwareProtocol,
    /// Undo stack for key and encoder assignments
    undo_stack: Vec<UndoAction>,
    /// Frame counter for periodic device scan
    scan_frame: u32,
    /// Last device scan timestamp in egui seconds
    last_device_scan_at: f64,
    /// Layer to preview on hover (None = show selected_layer)
    hover_layer: Option<usize>,
    /// Last main keyboard layout geometry: offset_x, offset_y, unit, padding
    last_layout_geometry: Option<(f32, f32, f32, f32)>,
    /// Key index hovered in previous frame (for hint display)
    prev_hovered_key: Option<usize>,
    prev_hovered_encoder: bool,
    prev_hovered_encoder_keycode: Option<u16>,
    /// Set when secondary click was handled by a key (prevents global jump-back)
    secondary_click_handled: bool,
    /// Deferred left/right modifier swap, applied after Ctrl is released
    pending_handed_swap: Option<(usize, usize, u16)>,
    /// Animation progress for hover layer preview (0.0 = hidden, 1.0 = fully shown)
    hover_layer_progress: f32,
    /// Stack of layers to return to on right-click (last = most recent)
    jump_back_stack: Vec<usize>,
    dark_mode: bool,
    app_settings: AppSettings,
    text_expander_rules_signature: Vec<(String, Option<std::time::SystemTime>)>,
    text_expander_rules_last_check_at: f64,
    #[cfg(target_os = "windows")]
    tray_icon: Option<tray_icon::TrayIcon>,
    #[cfg(target_os = "windows")]
    windows_hwnd: Option<isize>,
    main_menu_tab: MainMenuTab,
    combo_entries: Vec<ComboEntry>,
    combo_names: Vec<String>,
    selected_combo: usize,
    combo_dirty: bool,
    combo_names_dirty: bool,
    combo_term: Option<u16>,
    auto_shift_options: AutoShiftOptionsState,
    auto_shift_timeout: Option<u16>,
    auto_shift_timeout_text: String,
    mouse_keys_settings: MouseKeysSettingsState,
    touchpad_settings: TouchpadSettingsState,
    module_settings: ModuleSettingsState,
    tap_hold_settings: TapHoldSettingsState,
    magic_settings: MagicSettingsState,
    one_shot_settings: OneShotSettingsState,
    grave_escape_settings: GraveEscapeSettingsState,
    layer_led_settings: LayerLedSettingsState,
    alt_repeat_entries: Vec<AltRepeatKeyEntry>,
    alt_repeat_names: Vec<String>,
    alt_repeat_undo_stack: Vec<(Vec<AltRepeatKeyEntry>, Vec<String>, usize)>,
    selected_alt_repeat: usize,
    alt_repeat_visible_count: usize,
    alt_repeat_pick_target: Option<AltRepeatPickField>,
    last_single_instance_signal: String,
    rgb_settings: RgbSettingsState,
    layout_options_value: Option<u32>,
    encoder_visibility: Vec<bool>,
    combo_term_dirty: bool,
    combo_visible_count: usize,
    combo_capture_open: bool,
    combo_capture_keys: Vec<u16>,
    combo_undo_stack: Vec<(Vec<ComboEntry>, Vec<String>, Option<u16>, usize, usize)>,
    combo_pick_target: Option<(usize, ComboPickField)>,
    key_override_entries: Vec<KeyOverrideEntry>,
    key_override_names: Vec<String>,
    key_override_visible_count: usize,
    key_override_undo_stack: Vec<(Vec<KeyOverrideEntry>, Vec<String>, usize, usize)>,
    text_expander_deleted_rule: Option<(usize, crate::text_expander::TextExpansionRule)>,
    selected_key_override: usize,
    key_override_pick_target: Option<KeyOverridePickField>,
    matrix_tester_pressed: Vec<bool>,
    matrix_tester_ever_pressed: Vec<bool>,
    sticky_layout_prev_pressed: Vec<bool>,
    sticky_layout_pressed_key_layers: Vec<Option<usize>>,
    sticky_layout_toggled_layers: Vec<bool>,
    sticky_layout_base_layer: usize,
    sticky_layout_last_size: Option<Vec2>,
    sticky_layout_resize_opacity_hold_frames: u8,
    pending_layout_indicator_open_after_unlock: bool,
    matrix_tester_last_poll: std::time::Instant,
    matrix_tester_last_lock_check: std::time::Instant,
    matrix_tester_unlock_prompted: bool,
    matrix_tester_lock_checked: bool,
    macro_auto_unlock_cancelled: bool,
    settings_tab: SettingsTab,
    layer_names: Vec<String>,
    editing_layer: Option<usize>, // layer being renamed
    editing_layer_text: String,
    editing_layer_focus_requested: bool,
    /// Current connected device name (for per-device layer names)
    current_device_name: String,
    /// Friendly names learned from firmware/device info, keyed by device path.
    device_display_names: std::collections::HashMap<String, String>,
    tour_state: TourState,
    tour_target_rects: Vec<(TourTarget, egui::Rect)>,
    /// Vial unlock dialog open
    unlock_open: bool,
    vial_unlock_keys: Vec<(u8, u8)>,
    vial_unlock_polling: bool,
    vial_unlock_counter: u8,
    vial_unlock_best: u8,
    vial_unlock_total: u8,
    vial_unlock_last_poll: Option<std::time::Instant>,
    vial_unlock_animation_nonce: u64,
}

impl EntropyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app_settings = load_app_settings();
        app_settings.ui_scale = clamp_ui_scale(app_settings.ui_scale);
        cc.egui_ctx.set_zoom_factor(app_settings.ui_scale);
        crate::ui_style::set_accent(app_settings.accent_color.color());
        crate::smart_input::set_text_expander_config(
            app_settings.text_expander_enabled,
            {
                let mut rules = app_settings.text_expansion_rules.clone();
                rules.extend(load_extra_text_expansion_rules(
                    &app_settings.text_expander_rule_files,
                ));
                rules
            },
            parse_text_expander_blacklist(&app_settings.text_expander_app_blacklist),
        );

        let text_expander_rules_signature =
            text_expander_rules_signature(&app_settings.text_expander_rule_files);

        let mut app = Self {
            #[cfg(not(target_arch = "wasm32"))]
            hid_device: None,
            #[cfg(not(target_arch = "wasm32"))]
            qmk_hid_hosts: std::collections::HashMap::new(),
            firmware: FirmwareProtocol::Vial,
            undo_stack: Vec::new(),
            scan_frame: 0,
            last_device_scan_at: 0.0,
            hover_layer: None,
            last_layout_geometry: None,
            prev_hovered_key: None,
            prev_hovered_encoder: false,
            prev_hovered_encoder_keycode: None,
            secondary_click_handled: false,
            pending_handed_swap: None,
            hover_layer_progress: 0.0,
            jump_back_stack: Vec::new(),
            device_manager: DeviceManager::new(),
            selected_device: None,
            selected_layer: 0,
            selected_key: None,
            selected_encoder: None,
            layout: None,
            layer_count: 4,
            keycode_picker: KeycodePicker::default(),
            status_msg: String::new(),
            dark_mode: false,
            app_settings,
            text_expander_rules_signature,
            text_expander_rules_last_check_at: 0.0,
            #[cfg(target_os = "windows")]
            tray_icon: None,
            #[cfg(target_os = "windows")]
            windows_hwnd: None,
            main_menu_tab: MainMenuTab::Keyboard,
            combo_entries: vec![],
            combo_names: vec![],
            selected_combo: 0,
            combo_dirty: false,
            combo_names_dirty: false,
            combo_term: None,
            auto_shift_options: AutoShiftOptionsState::default(),
            auto_shift_timeout: None,
            auto_shift_timeout_text: String::new(),
            mouse_keys_settings: MouseKeysSettingsState::default(),
            touchpad_settings: TouchpadSettingsState::default(),
            module_settings: ModuleSettingsState::default(),
            tap_hold_settings: TapHoldSettingsState::default(),
            magic_settings: MagicSettingsState::default(),
            one_shot_settings: OneShotSettingsState::default(),
            grave_escape_settings: GraveEscapeSettingsState::default(),
            layer_led_settings: LayerLedSettingsState::default(),
            alt_repeat_entries: vec![],
            alt_repeat_names: vec![],
            alt_repeat_undo_stack: Vec::new(),
            selected_alt_repeat: 0,
            alt_repeat_visible_count: 1,
            alt_repeat_pick_target: None,
            last_single_instance_signal: read_single_instance_signal(),
            rgb_settings: RgbSettingsState::default(),
            layout_options_value: None,
            encoder_visibility: vec![],
            combo_term_dirty: false,
            combo_visible_count: 1,
            combo_capture_open: false,
            combo_capture_keys: Vec::new(),
            combo_undo_stack: Vec::new(),
            combo_pick_target: None,
            key_override_entries: Vec::new(),
            key_override_names: vec![],
            key_override_visible_count: 1,
            key_override_undo_stack: Vec::new(),
            text_expander_deleted_rule: None,
            selected_key_override: 0,
            key_override_pick_target: None,
            matrix_tester_pressed: Vec::new(),
            matrix_tester_ever_pressed: Vec::new(),
            sticky_layout_prev_pressed: Vec::new(),
            sticky_layout_pressed_key_layers: Vec::new(),
            sticky_layout_toggled_layers: Vec::new(),
            sticky_layout_base_layer: 0,
            sticky_layout_last_size: None,
            sticky_layout_resize_opacity_hold_frames: 0,
            pending_layout_indicator_open_after_unlock: false,
            matrix_tester_last_poll: std::time::Instant::now(),
            matrix_tester_last_lock_check: std::time::Instant::now()
                - MATRIX_TESTER_LOCK_CHECK_INTERVAL,
            matrix_tester_unlock_prompted: false,
            matrix_tester_lock_checked: false,
            macro_auto_unlock_cancelled: false,
            settings_tab: SettingsTab::MatrixTester,
            layer_names: load_layer_names("default"),
            editing_layer: None,
            editing_layer_text: String::new(),
            editing_layer_focus_requested: false,
            current_device_name: String::new(),
            device_display_names: std::collections::HashMap::new(),
            tour_state: TourState::default(),
            tour_target_rects: Vec::new(),
            unlock_open: false,
            vial_unlock_keys: vec![],
            vial_unlock_polling: false,
            vial_unlock_counter: 0,
            vial_unlock_best: 50,
            vial_unlock_total: 50,
            vial_unlock_last_poll: None,
            vial_unlock_animation_nonce: 0,
            #[cfg(not(target_arch = "wasm32"))]
            connect_state: ConnectState::Idle,
            #[cfg(not(target_arch = "wasm32"))]
            device_scan_state: DeviceScanState::Idle,
        };
        // Auto-connect to first device if available
        #[cfg(not(target_arch = "wasm32"))]
        if !app.device_manager.devices().is_empty() {
            app.selected_device = Some(0);
            app.start_connect(0);
        }
        app
    }

    /// Assign keycode and immediately write to device (blocking, but single HID op — fast).
    #[cfg(not(target_arch = "wasm32"))]
    fn refresh_layer_picker_content_flags(&mut self) {
        if let Some(layout) = &self.layout {
            self.keycode_picker.layer_has_content = layout
                .layers
                .iter()
                .map(|keys| keys.iter().any(|&kc| kc != 0x0000 && kc != 0x0001))
                .collect();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn is_vial_locked(&self) -> bool {
        self.firmware == FirmwareProtocol::Vial
            && self.layout.is_some()
            && !self.vial_unlock_polling
            && self
                .hid_device
                .as_ref()
                .and_then(|hid| hid.get_unlock_status().ok())
                .map(|(unlocked, _)| unlocked)
                .map(|unlocked| !unlocked)
                .unwrap_or(false)
    }

    #[cfg(target_arch = "wasm32")]
    fn is_vial_locked(&self) -> bool {
        false
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn reopen_vial_hid(&mut self) {
        if let Some(dev) = self
            .selected_device
            .and_then(|i| self.device_manager.devices().get(i))
        {
            match crate::hid::HidDevice::open(&dev.path) {
                Ok(hid) => {
                    self.hid_device = Some(hid);
                }
                Err(e) => {
                    self.hid_device = None;
                    self.status_msg = format!("Reconnect failed: {e}");
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn cancel_vial_unlock(&mut self, suppress_macro_auto_unlock: bool) {
        if let Some(hid) = &self.hid_device {
            match hid.lock() {
                Ok(()) => {
                    self.status_msg = "Device unlock cancelled".into();
                }
                Err(e) => {
                    self.status_msg = format!("Cancel unlock failed: {e}");
                }
            }
        }
        self.unlock_open = false;
        self.vial_unlock_polling = false;
        self.vial_unlock_last_poll = None;
        self.pending_layout_indicator_open_after_unlock = false;
        self.vial_unlock_counter = 0;
        self.vial_unlock_best = 50;
        self.matrix_tester_unlock_prompted = false;
        self.matrix_tester_lock_checked = false;
        if suppress_macro_auto_unlock {
            self.macro_auto_unlock_cancelled = true;
        }
        self.reopen_vial_hid();
    }

    fn draw_settings_screen(
        &mut self,
        ui: &mut egui::Ui,
        layout: &KeyboardLayout,
        ctx: &egui::Context,
        content_top: f32,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        if self.settings_tab == SettingsTab::MatrixTester {
            self.poll_matrix_tester(ctx, layout);
        }

        if self.settings_tab == SettingsTab::MatrixTester {
            if let Some(id) = ctx.memory(|m| m.focused()) {
                ctx.memory_mut(|m| m.surrender_focus(id));
            }
        }

        let dark = ui.visuals().dark_mode;
        let content_rect = egui::Rect::from_min_max(
            egui::pos2(ui.min_rect().left() + 20.0, content_top),
            egui::pos2(ui.min_rect().right() - 20.0, ui.max_rect().bottom() - 76.0),
        );

        match self.settings_tab {
            SettingsTab::AppSettings => {
                self.draw_app_settings_page(ui, content_rect);
            }
            SettingsTab::MatrixTester => {
                self.draw_matrix_tester_settings(ui, layout, content_rect, dark);
            }
            SettingsTab::UniversalSymbolsSetup => {
                self.draw_universal_symbols_setup_page(ui, content_rect);
            }
            SettingsTab::TextExpander => {
                self.draw_text_expander_settings_page(ui, content_rect);
            }
            SettingsTab::AutoShift => {
                self.draw_auto_shift_settings_page(ui, content_rect, dark);
            }
            SettingsTab::Rgb => {
                self.draw_rgb_settings_page(ui, content_rect, dark);
            }
            SettingsTab::LayerLeds => {
                self.draw_layer_led_settings_page(ui, content_rect);
            }
            SettingsTab::Encoders => {
                self.draw_encoder_visibility_settings_page(ui, content_rect, dark);
            }
            SettingsTab::TapHold => {
                self.draw_tap_hold_settings_page(ui, content_rect);
            }
            SettingsTab::Magic => {
                self.draw_magic_settings_page(ui, content_rect);
            }
            SettingsTab::GraveEscape => {
                self.draw_grave_escape_settings_page(ui, content_rect);
            }
            SettingsTab::LayoutOptions => {
                self.draw_layout_options_settings_page(ui, content_rect);
            }
            SettingsTab::Modules => {
                self.draw_module_settings_page(ui, content_rect);
            }
            SettingsTab::Touchpad => {
                self.draw_touchpad_settings_page(ui, content_rect);
            }
            SettingsTab::LiveFeatures => {
                self.draw_live_features_settings_page(ui, content_rect);
            }
            SettingsTab::Combo => {
                self.draw_combo_settings_page(ui, ctx, content_rect);
            }
            SettingsTab::KeyOverrides => {
                self.draw_key_override_settings_page(ui, content_rect);
            }
            SettingsTab::AltRepeat => {
                self.draw_alt_repeat_settings_page(ui, content_rect);
            }
            SettingsTab::MouseKeys => {
                self.draw_mouse_keys_settings_page(ui, content_rect);
            }
        }

        if self.settings_tab != SettingsTab::MatrixTester {
            self.draw_settings_navigation_hint(ui);
        }
    }

    fn draw_settings_navigation_hint(&self, ui: &mut egui::Ui) {
        let hint_color = if ui.visuals().dark_mode {
            Color32::from_gray(100)
        } else {
            Color32::from_gray(160)
        };
        ui.painter().text(
            egui::pos2(ui.max_rect().center().x, ui.max_rect().bottom() - 36.0),
            egui::Align2::CENTER_CENTER,
            crate::i18n::tr_catalog(
                self.app_settings.language,
                "navigation.return_to_layout_hint",
            ),
            FontId::proportional(11.0),
            hint_color,
        );
    }
}

impl eframe::App for EntropyApp {
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        app_panel_fill(visuals.dark_mode).to_normalized_gamma_f32()
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        #[cfg(not(target_arch = "wasm32"))]
        self.fallback_entropy_display_presets_before_exit();
        std::process::exit(0);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.apply_ui_scale(ctx);
        self.handle_ui_scale_shortcuts(ctx);

        #[cfg(target_os = "windows")]
        self.cache_windows_hwnd(frame);
        self.handle_close_to_tray(ctx);
        #[cfg(target_os = "windows")]
        self.poll_tray_events(ctx);
        #[cfg(target_os = "windows")]
        self.handle_tray_quit_request();

        self.tour_target_rects.clear();

        let combo_capture_open_at_frame_start = self.combo_capture_open;
        let keyboard_input_wanted_at_frame_start = ctx.wants_keyboard_input();
        let modal_or_popup_open_at_frame_start = self.keycode_picker.open
            || self.unlock_open
            || self.vial_unlock_polling
            || self.top_dropdown_open(ctx)
            || ctx.memory(|m| m.any_popup_open());

        // Keep lightweight device detection alive even when the UI is otherwise idle.
        #[cfg(not(target_arch = "wasm32"))]
        ctx.request_repaint_after(std::time::Duration::from_millis(250));

        #[cfg(not(target_arch = "wasm32"))]
        self.poll_device_scan(ctx);

        // Auto-scan for device connect/disconnect changes.
        self.secondary_click_handled = false;
        if let Some((layer, ki, kc)) = self.pending_handed_swap {
            if !ctx.input(|i| i.modifiers.ctrl) {
                #[cfg(not(target_arch = "wasm32"))]
                self.assign_keycode(layer, ki, kc);
                #[cfg(target_arch = "wasm32")]
                if let Some(layout) = &mut self.layout {
                    layout.set_keycode(layer, ki, kc);
                }
                self.pending_handed_swap = None;
            }
        }
        let now = ctx.input(|i| i.time);
        self.auto_reload_text_expander_rules_file(now);
        if (self.last_device_scan_at == 0.0 || now - self.last_device_scan_at >= 1.0)
            && !self.vial_unlock_polling
        {
            self.scan_frame = self.scan_frame.wrapping_add(1);
            self.last_device_scan_at = now;
            #[cfg(not(target_arch = "wasm32"))]
            self.start_device_scan();
        }

        #[cfg(not(target_arch = "wasm32"))]
        self.poll_single_instance_signal(ctx);

        // Apply theme
        if self.dark_mode {
            let mut v = egui::Visuals::dark();
            v.panel_fill = app_panel_fill(true);
            v.window_fill = app_window_fill(true);
            v.faint_bg_color = app_window_fill(true);
            v.extreme_bg_color = Color32::from_rgb(24, 24, 24);
            v.widgets.noninteractive.bg_fill = app_window_fill(true);
            v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, app_border_color(true));
            v.widgets.inactive.bg_fill = app_surface_fill(true);
            v.widgets.inactive.weak_bg_fill = app_surface_fill(true);
            v.widgets.inactive.bg_stroke = Stroke::new(1.0, app_border_color(true));
            v.widgets.hovered.bg_fill = app_hover_fill(true);
            v.widgets.hovered.weak_bg_fill = app_hover_fill(true);
            v.widgets.hovered.bg_stroke = Stroke::new(1.0, app_accent());
            v.widgets.active.bg_fill = app_accent();
            v.widgets.active.weak_bg_fill = app_accent();
            v.widgets.active.bg_stroke = Stroke::new(1.0, app_accent());
            v.selection.bg_fill = Color32::from_rgba_unmultiplied(82, 82, 86, 140);
            v.selection.stroke = Stroke::new(1.0, Color32::from_rgb(245, 245, 245));
            v.hyperlink_color = app_accent();
            v.interact_cursor = Some(egui::CursorIcon::PointingHand);
            ctx.set_visuals(v);
        } else {
            let mut v = egui::Visuals::light();
            v.panel_fill = app_panel_fill(false);
            v.window_fill = app_window_fill(false);
            v.faint_bg_color = app_panel_fill(false);
            v.extreme_bg_color = Color32::from_rgb(235, 235, 235);
            v.widgets.noninteractive.bg_fill = app_panel_fill(false);
            v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, app_border_color(false));
            v.widgets.inactive.bg_fill = app_surface_fill(false);
            v.widgets.inactive.weak_bg_fill = app_surface_fill(false);
            v.widgets.inactive.bg_stroke = Stroke::new(1.0, app_border_color(false));
            v.widgets.hovered.bg_fill = app_hover_fill(false);
            v.widgets.hovered.weak_bg_fill = app_hover_fill(false);
            v.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(230, 230, 233));
            v.widgets.active.bg_fill = app_accent();
            v.widgets.active.weak_bg_fill = app_accent();
            v.widgets.active.bg_stroke = Stroke::new(1.0, app_accent());
            v.selection.bg_fill = Color32::from_rgba_unmultiplied(82, 82, 86, 72);
            v.selection.stroke = Stroke::new(1.0, Color32::from_rgb(38, 38, 40));
            v.hyperlink_color = app_accent();
            v.interact_cursor = Some(egui::CursorIcon::PointingHand);
            ctx.set_visuals(v);
        }

        // Poll background connect thread
        #[cfg(not(target_arch = "wasm32"))]
        self.poll_connect(ctx);

        self.apply_picker_results();

        // Deselect key when picker is closed without choosing
        if !self.keycode_picker.open
            && (self.selected_key.is_some() || self.selected_encoder.is_some())
            && self.keycode_picker.result.is_none()
        {
            self.selected_key = None;
            self.selected_encoder = None;
        }

        if !self.keycode_picker.open || self.keycode_picker.selected_tab != KeycodeTab::Macro {
            self.macro_auto_unlock_cancelled = false;
        }

        if self.firmware == FirmwareProtocol::Vial
            && self.keycode_picker.open
            && self.keycode_picker.selected_tab == KeycodeTab::Macro
            && !self.unlock_open
            && !self.vial_unlock_polling
            && !self.macro_auto_unlock_cancelled
            && self.is_vial_locked()
        {
            self.unlock_open = true;
            self.status_msg = crate::i18n::tr_catalog(
                self.app_settings.language,
                "connection.keyboard_locked_edit_macros",
            )
            .into();
        }

        // Arrow keys Left/Right switch layers (when picker is closed and no text field is focused)
        if !self.tour_state.active && !self.keycode_picker.open && !ctx.wants_keyboard_input() {
            let layer_count = self.layer_count;
            ctx.input(|i| {
                if i.key_pressed(egui::Key::ArrowLeft) && self.selected_layer > 0 {
                    self.selected_layer -= 1;
                    self.jump_back_stack.clear();
                }
                if i.key_pressed(egui::Key::ArrowRight) && self.selected_layer + 1 < layer_count {
                    self.selected_layer += 1;
                    self.jump_back_stack.clear();
                }
            });
        }

        // Check if loading
        #[cfg(not(target_arch = "wasm32"))]
        let is_loading = matches!(self.connect_state, ConnectState::Loading(_));
        #[cfg(target_arch = "wasm32")]
        let is_loading = false;

        // Main canvas
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.selected_device.is_none() {
                let rect = ui.max_rect();
                let empty_rect = egui::Rect::from_center_size(
                    rect.center(),
                    egui::vec2(rect.width().min(520.0), 150.0),
                );
                ui.allocate_ui_at_rect(empty_rect, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(4.0);
                        ui.label(RichText::new("✦").size(28.0).color(app_accent()));
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new(crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "connection.waiting_for_keyboard",
                            ))
                            .size(20.0)
                            .strong()
                            .color(if self.dark_mode {
                                Color32::from_rgb(235, 235, 235)
                            } else {
                                Color32::from_rgb(42, 42, 44)
                            }),
                        );
                        ui.add_space(7.0);
                        ui.label(
                            RichText::new(crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "connection.connect_vial_device",
                            ))
                            .size(13.0)
                            .color(app_muted_text(self.dark_mode)),
                        );
                    });
                });
                return;
            }

            if is_loading {
                let rect = ui.max_rect();
                let text = crate::i18n::tr_catalog(
                    self.app_settings.language,
                    "connection.loading_keyboard",
                );
                let font_id = FontId::proportional(16.0);
                let text_width = ui.fonts(|f| {
                    f.layout_no_wrap(text.to_owned(), font_id.clone(), Color32::GRAY)
                        .size()
                        .x
                });
                let spinner_size = 18.0;
                let gap = 8.0;
                let row_width = spinner_size + gap + text_width;
                let row_left = rect.center().x - row_width * 0.5;
                let spinner_rect = egui::Rect::from_center_size(
                    egui::pos2(row_left + spinner_size * 0.5, rect.center().y),
                    egui::vec2(spinner_size, spinner_size),
                );
                egui::Spinner::new()
                    .size(spinner_size)
                    .color(Color32::GRAY)
                    .paint_at(ui, spinner_rect);
                ui.painter().text(
                    egui::pos2(row_left + spinner_size + gap, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    text,
                    font_id,
                    Color32::GRAY,
                );
                return;
            }

            if self.layout.is_some() {
                let layout = self.layout.clone().unwrap();
                self.draw_layout(ui, &layout, ctx);
            } else {
                self.draw_placeholder(ui);
            }
        });

        self.draw_sticky_layout_window(ctx);

        egui::Area::new(egui::Id::new("made_by_signature"))
            .anchor(egui::Align2::LEFT_BOTTOM, [16.0, -12.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let muted = app_muted_text(self.dark_mode);
                    ui.spacing_mut().item_spacing.x = 3.0;
                    ui.label(
                        RichText::new("tools of the future by")
                            .size(11.0)
                            .color(muted),
                    );
                    let (site_label, site_url) =
                        if matches!(self.app_settings.language, crate::i18n::Language::Russian) {
                            ("eh.works", "https://eh.works")
                        } else {
                            ("eh.industries", "https://eh.industries")
                        };
                    ui.add(egui::Hyperlink::from_label_and_url(
                        RichText::new(site_label).size(11.0),
                        site_url,
                    ));
                });
            });

        egui::Area::new(egui::Id::new("theme_selector"))
            .anchor(egui::Align2::RIGHT_BOTTOM, [-16.0, -12.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                draw_theme_selector_labels(
                    ui,
                    self.app_settings.language,
                    &mut self.dark_mode,
                    false,
                );
            });

        // Keycode picker modal
        self.draw_vial_unlock_overlay(ctx);

        if self.keycode_picker.open {
            let screen_rect = ctx.screen_rect();
            egui::Area::new("window_backdrop".into())
                .order(egui::Order::Foreground)
                .fixed_pos(screen_rect.min)
                .show(ctx, |ui| {
                    let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, screen_rect.size());
                    let response =
                        ui.interact(rect, ui.id().with("backdrop_click"), egui::Sense::click());
                    ui.painter().rect_filled(
                        rect,
                        0.0,
                        Color32::from_black_alpha(crate::ui_style::modal_backdrop_alpha(
                            ctx.style().visuals.dark_mode,
                        )),
                    );
                    if response.clicked() {
                        self.keycode_picker.open = false;
                        if let Some(id) = ctx.memory(|m| m.focused()) {
                            ctx.memory_mut(|m| m.surrender_focus(id));
                        }
                    }
                });
        }

        if !self.unlock_open && !self.vial_unlock_polling {
            self.keycode_picker.language = self.app_settings.language;
            self.keycode_picker.key_legend_layout = self.app_settings.key_legend_layout;
            self.keycode_picker.show_shifted_number_symbols =
                self.app_settings.show_shifted_number_symbols;
            self.keycode_picker.show(ctx);
            self.apply_picker_results();
        }

        if self.combo_pick_target.is_some()
            && !self.keycode_picker.open
            && self.keycode_picker.result.is_none()
        {
            self.combo_pick_target = None;
        }
        if self.key_override_pick_target.is_some()
            && !self.keycode_picker.open
            && self.keycode_picker.result.is_none()
        {
            self.key_override_pick_target = None;
        }
        if self.alt_repeat_pick_target.is_some()
            && !self.keycode_picker.open
            && self.keycode_picker.result.is_none()
        {
            self.alt_repeat_pick_target = None;
        }

        // Write macros to device if changed
        self.maybe_start_onboarding_tour(ctx);
        self.draw_onboarding_tour(ctx);

        if self.keycode_picker.macros_dirty {
            if self.unlock_open || self.vial_unlock_polling {
                // Defer macro write until unlock flow fully finishes.
            } else if self.is_vial_locked() {
                self.unlock_open = true;
                self.status_msg = crate::i18n::tr_catalog(
                    self.app_settings.language,
                    "connection.keyboard_locked_edit_macros",
                )
                .into();
            } else {
                self.keycode_picker.macros_dirty = false;
                if let Some(hid) = &self.hid_device {
                    if let Ok(size) = hid.get_macro_buffer_size() {
                        let buf = crate::hid::HidDevice::encode_macros(
                            &self.keycode_picker.macro_texts,
                            size,
                        );
                        match hid.set_macro_buffer(&buf) {
                            Ok(()) => {
                                self.status_msg = crate::i18n::tr_catalog(
                                    self.app_settings.language,
                                    "status_messages.macros_saved",
                                )
                                .into()
                            }
                            Err(e) => self.status_msg = format!("Macro write error: {e}"),
                        }
                    }
                }
            }
        }

        // Write combos to device if changed
        if self.combo_dirty && !self.keycode_picker.open {
            let mut combo_save_ok = true;
            if let Some(hid) = &self.hid_device {
                for (i, combo) in self.combo_entries.iter().enumerate() {
                    match hid.set_combo(i as u8, combo.keys, combo.output) {
                        Ok(()) => {}
                        Err(e) => {
                            self.status_msg = format!("Combo write error: {e}");
                            combo_save_ok = false;
                            break;
                        }
                    }
                }
            }
            if combo_save_ok {
                self.combo_dirty = false;
                self.status_msg = crate::i18n::tr_catalog(
                    self.app_settings.language,
                    "status_messages.combos_saved",
                )
                .into();
            }
        }

        if self.combo_term_dirty && !self.keycode_picker.open {
            let mut term_save_ok = true;
            if let (Some(hid), Some(value)) = (&self.hid_device, self.combo_term) {
                if let Err(e) = hid.set_qmk_setting_u16(2, value) {
                    self.status_msg = format!("Combo timeout write error: {e}");
                    term_save_ok = false;
                }
            }
            if term_save_ok {
                self.combo_term_dirty = false;
                self.status_msg = crate::i18n::tr_catalog(
                    self.app_settings.language,
                    "status_messages.combo_timeout_saved",
                )
                .into();
            }
        }

        if self.combo_names_dirty {
            save_combo_names(&self.combo_names, &self.current_device_name);
            self.combo_names_dirty = false;
        }

        // Write tap dance to device if changed
        if self.keycode_picker.tap_dance_dirty && !self.keycode_picker.open {
            let mut td_save_ok = true;
            if let Some(hid) = &self.hid_device {
                for (i, td) in self.keycode_picker.tap_dance_entries.iter().enumerate() {
                    match hid.set_tap_dance(
                        i as u8,
                        td.on_tap,
                        td.on_hold,
                        td.on_double_tap,
                        td.on_tap_hold,
                        td.tapping_term,
                    ) {
                        Ok(()) => {}
                        Err(e) => {
                            self.status_msg = format!("Tap dance write error: {e}");
                            td_save_ok = false;
                            break;
                        }
                    }
                }
            }
            if td_save_ok {
                save_tap_dance_names(
                    &self.keycode_picker.tap_dance_names,
                    &self.current_device_name,
                );
                self.keycode_picker.tap_dance_dirty = false;
                if self.status_msg.is_empty() || self.status_msg.starts_with("✓") {
                    self.status_msg = "✓ Tap dance saved".into();
                }
            }
        }

        let mut settings_page_navigation_handled = false;
        if self.can_return_from_settings_page(
            ctx,
            modal_or_popup_open_at_frame_start,
            combo_capture_open_at_frame_start,
            keyboard_input_wanted_at_frame_start,
        ) {
            let esc_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));
            let rclick = ctx.input(|i| i.pointer.secondary_clicked());
            if esc_pressed || rclick {
                self.close_top_dropdowns(ctx);
                self.main_menu_tab = MainMenuTab::Keyboard;
                settings_page_navigation_handled = true;
            }
        }

        // Right-click anywhere = pop back one step (only if NOT hovering a layer key and not handled by key)
        if !settings_page_navigation_handled
            && !self.jump_back_stack.is_empty()
            && !self.keycode_picker.open
            && !self.secondary_click_handled
        {
            let esc_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));
            let rclick = self.hover_layer.is_none() && ctx.input(|i| i.pointer.secondary_clicked());
            if rclick || esc_pressed {
                if let Some(back_layer) = self.jump_back_stack.pop() {
                    self.selected_layer = back_layer;
                }
            }
        }
    }
}

impl EntropyApp {}

impl EntropyApp {
    fn open_mouse_keys_settings_page(&mut self) {
        self.settings_tab = SettingsTab::MouseKeys;
        self.main_menu_tab = MainMenuTab::Settings;
    }

    fn open_app_settings_page(&mut self) {
        self.settings_tab = SettingsTab::AppSettings;
        self.main_menu_tab = MainMenuTab::Settings;
    }

    fn open_universal_symbols_setup_page(&mut self) {
        self.settings_tab = SettingsTab::UniversalSymbolsSetup;
        self.main_menu_tab = MainMenuTab::Settings;
    }

    fn open_text_expander_settings_page(&mut self) {
        self.settings_tab = SettingsTab::TextExpander;
        self.main_menu_tab = MainMenuTab::Advanced;
    }

    fn open_layer_led_settings_page(&mut self) {
        self.settings_tab = SettingsTab::LayerLeds;
        self.main_menu_tab = MainMenuTab::Settings;
    }

    fn open_tap_hold_settings_page(&mut self) {
        self.settings_tab = SettingsTab::TapHold;
        self.main_menu_tab = MainMenuTab::Settings;
    }

    fn open_magic_settings_page(&mut self) {
        self.settings_tab = SettingsTab::Magic;
        self.main_menu_tab = MainMenuTab::Settings;
    }

    fn open_grave_escape_settings_page(&mut self) {
        self.settings_tab = SettingsTab::GraveEscape;
        self.main_menu_tab = MainMenuTab::Settings;
    }

    fn can_return_from_settings_page(
        &self,
        ctx: &egui::Context,
        modal_or_popup_open_at_frame_start: bool,
        combo_capture_open_at_frame_start: bool,
        keyboard_input_wanted_at_frame_start: bool,
    ) -> bool {
        matches!(
            self.main_menu_tab,
            MainMenuTab::Settings | MainMenuTab::Advanced
        ) && self.settings_tab != SettingsTab::MatrixTester
            && !self.secondary_click_handled
            && !self.keycode_picker.open
            && !self.unlock_open
            && !self.vial_unlock_polling
            && !self.combo_capture_open
            && !modal_or_popup_open_at_frame_start
            && !combo_capture_open_at_frame_start
            && !keyboard_input_wanted_at_frame_start
            && !ctx.wants_keyboard_input()
            && !ctx.memory(|m| m.any_popup_open())
            && !self.top_dropdown_open(ctx)
    }
}
