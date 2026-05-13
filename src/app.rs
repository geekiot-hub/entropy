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

fn clamp_sticky_layout_opacity(opacity: f32) -> f32 {
    if opacity.is_finite() {
        opacity.clamp(0.50, 1.0)
    } else {
        default_sticky_layout_opacity()
    }
}

#[cfg(target_os = "windows")]
fn set_windows_window_opacity_by_title(title: &str, opacity: f32) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        FindWindowW, GetWindowLongPtrW, SetLayeredWindowAttributes, SetWindowLongPtrW, GWL_EXSTYLE,
        LWA_ALPHA, WS_EX_LAYERED,
    };

    let alpha = (clamp_sticky_layout_opacity(opacity) * 255.0).round() as u8;
    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let hwnd = FindWindowW(std::ptr::null(), title_wide.as_ptr());
        if hwnd.is_null() {
            return;
        }
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style | WS_EX_LAYERED as isize);
        SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA);
    }
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

#[derive(Clone, Copy)]
struct RgbModalLayout {
    content_width: f32,
    top_padding: f32,
    row_height: f32,
    color_row_height: f32,
}

impl RgbModalLayout {
    fn responsive(ctx: &egui::Context) -> Self {
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ctx);
        Self {
            content_width: metrics.settings_content_width(),
            top_padding: metrics.value(4.0),
            row_height: metrics.settings_row_height(),
            color_row_height: metrics.settings_row_height(),
        }
    }

    fn modal_layout(self) -> crate::ui_style::ModalLayout {
        crate::ui_style::ModalLayout::new(self.content_width).with_top_padding(self.top_padding)
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

fn top_dropdown_frame(dark: bool) -> egui::Frame {
    egui::Frame::new()
        .fill(app_surface_fill(dark))
        .stroke(crate::ui_style::modal_outline_stroke(dark))
        .corner_radius(12.0)
        .inner_margin(egui::Margin::symmetric(8, 6))
}

fn top_dropdown_item(
    ui: &mut egui::Ui,
    width: f32,
    label: &str,
    enabled: bool,
    selected: bool,
) -> egui::Response {
    let dark = ui.visuals().dark_mode;
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 30.0), sense);
    let hovered = resp.hovered() && enabled;
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    if ui.is_rect_visible(rect) {
        if selected || hovered {
            let fill = app_hover_fill(dark);
            ui.painter().rect_filled(rect, 8.0, fill);
        }

        let text_color = if !enabled {
            app_muted_text(dark)
        } else if selected {
            app_accent()
        } else {
            ui.visuals().text_color()
        };
        let text_clip = if selected {
            egui::Rect::from_min_max(rect.min, egui::pos2(rect.right() - 24.0, rect.bottom()))
        } else {
            rect
        };
        ui.painter().with_clip_rect(text_clip).text(
            egui::pos2(rect.left() + 10.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(13.0),
            text_color,
        );

        if selected {
            ui.painter().circle_filled(
                egui::pos2(rect.right() - 12.0, rect.center().y),
                2.5,
                app_accent(),
            );
        }
    }

    resp
}

fn top_menu_text_width(ui: &egui::Ui, label: &str, font_size: f32) -> f32 {
    ui.fonts(|f| {
        f.layout_no_wrap(
            label.to_owned(),
            egui::FontId::proportional(font_size),
            ui.visuals().widgets.inactive.fg_stroke.color,
        )
        .size()
        .x
    })
}

fn adaptive_top_dropdown_width<'a>(
    ui: &egui::Ui,
    labels: impl IntoIterator<Item = &'a str>,
    min_width: f32,
) -> f32 {
    let text_width = labels
        .into_iter()
        .filter(|label| !label.is_empty())
        .map(|label| top_menu_text_width(ui, label, 13.0))
        .fold(0.0, f32::max);

    // 16px frame margins + 10px left text inset + selected-dot reserve + breathing room.
    (text_width + 56.0).max(min_width).min(360.0)
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

const LAYOUT_BASE_UNIT: f32 = 54.0_f32 * 1.15;
const LAYOUT_KEY_PADDING: f32 = 2.5_f32;
const LAYOUT_FIT_MARGIN: f32 = 40.0_f32;
const LAYOUT_ENCODER_RADIUS_FACTOR: f32 = 0.47_f32;
const LAYOUT_ENCODER_FILL_EXTRA: f32 = 1.0_f32;
const LAYOUT_TOP_RESERVED_H: f32 = 32.0_f32 + 4.0_f32 + 68.0_f32;
const LAYOUT_BOTTOM_RESERVED_H: f32 = 76.0_f32;
const STICKY_LAYOUT_WINDOW_W: f32 = 720.0_f32;
const STICKY_LAYOUT_WINDOW_H: f32 = 360.0_f32;
const STICKY_LAYOUT_WINDOW_MARGIN: f32 = 1.0_f32;
const STICKY_LAYOUT_WINDOW_TITLE_H: f32 = 34.0_f32;
const STICKY_LAYOUT_KEYBOARD_MARGIN: f32 = 1.0_f32;

#[derive(Clone, Copy)]
enum StickyLayoutWindowButton {
    Pin,
    Close,
}

fn draw_theme_selector_labels(
    ui: &mut egui::Ui,
    lang: crate::i18n::Language,
    dark_mode: &mut bool,
) {
    ui.horizontal(|ui| {
        let active = app_accent();
        let inactive = app_muted_text(*dark_mode);

        let light_resp = ui.add(
            egui::Label::new(
                RichText::new(crate::i18n::tr_catalog(lang, "app_chrome.light_light"))
                    .size(11.0)
                    .color(if *dark_mode { inactive } else { active }),
            )
            .selectable(false)
            .sense(egui::Sense::click()),
        );
        if light_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if light_resp.clicked() {
            *dark_mode = false;
        }

        ui.add(egui::Label::new(RichText::new("|").size(11.0).color(inactive)).selectable(false));

        let dark_resp = ui.add(
            egui::Label::new(
                RichText::new(crate::i18n::tr_catalog(lang, "app_chrome.dark_dark"))
                    .size(11.0)
                    .color(if *dark_mode { active } else { inactive }),
            )
            .selectable(false)
            .sense(egui::Sense::click()),
        );
        if dark_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if dark_resp.clicked() {
            *dark_mode = true;
        }
    });
}

fn draw_sticky_layout_transparency_dropdown(
    ui: &mut egui::Ui,
    lang: crate::i18n::Language,
    dark: bool,
    opacity: &mut f32,
) -> bool {
    const OPACITY_VALUES: [f32; 6] = [1.0, 0.90, 0.80, 0.70, 0.60, 0.50];

    let current = clamp_sticky_layout_opacity(*opacity);
    let selected_idx = OPACITY_VALUES
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            (*a - current)
                .abs()
                .partial_cmp(&(*b - current).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(idx, _)| idx)
        .unwrap_or(0);
    let label_prefix = crate::i18n::tr_catalog(lang, "ui.sticky_layout_transparency_short");
    let selected_text = format!(
        "{} {}%",
        label_prefix,
        ((1.0 - OPACITY_VALUES[selected_idx]) * 100.0).round() as i32
    );
    let dropdown_id = ui.id().with("sticky_layout_transparency_dropdown");
    let width = 104.0;
    let dropdown_resp = crate::ui_style::modern_dropdown_button_sized(
        ui,
        dropdown_id,
        &selected_text,
        if dark {
            Color32::from_rgb(235, 235, 235)
        } else {
            Color32::from_rgb(42, 42, 44)
        },
        width,
        24.0,
        11.0,
    );

    let mut changed = false;
    egui::popup_below_widget(
        ui,
        dropdown_id,
        &dropdown_resp,
        egui::PopupCloseBehavior::CloseOnClickOutside,
        |ui| {
            ui.set_min_width(width);
            ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
            for (idx, value) in OPACITY_VALUES.iter().copied().enumerate() {
                let option_text = format!(
                    "{} {}%",
                    label_prefix,
                    ((1.0 - value) * 100.0).round() as i32
                );
                let selected = idx == selected_idx;
                let (option_rect, option_resp) =
                    ui.allocate_exact_size(Vec2::new(width, 24.0), Sense::click());
                if option_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                let option_fill = if selected || option_resp.hovered() {
                    app_hover_fill(dark)
                } else {
                    Color32::TRANSPARENT
                };
                ui.painter().rect_filled(option_rect, 7.0, option_fill);
                ui.painter().text(
                    egui::pos2(option_rect.left() + 10.0, option_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    option_text,
                    FontId::proportional(11.0),
                    if selected {
                        if dark {
                            Color32::from_rgb(235, 235, 235)
                        } else {
                            Color32::from_rgb(42, 42, 44)
                        }
                    } else {
                        app_muted_text(dark)
                    },
                );
                if option_resp.clicked() {
                    *opacity = value;
                    changed = true;
                    ui.memory_mut(|m| m.close_popup());
                }
            }
        },
    );

    changed
}

fn sticky_layout_window_icon_button(
    ui: &mut egui::Ui,
    dark: bool,
    kind: StickyLayoutWindowButton,
    active: bool,
    tooltip: &str,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::click());
    let response = response.on_hover_text(tooltip);
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    let fill = if active || response.hovered() {
        app_hover_fill(dark)
    } else {
        Color32::TRANSPARENT
    };
    let stroke_color = if active {
        app_accent()
    } else {
        app_border_color(dark)
    };
    ui.painter().rect(
        rect,
        8.0,
        fill,
        Stroke::new(if active { 1.2 } else { 0.8 }, stroke_color),
        egui::StrokeKind::Inside,
    );

    let color = if active {
        app_accent()
    } else {
        app_muted_text(dark)
    };
    let stroke = Stroke::new(1.7, color);
    match kind {
        StickyLayoutWindowButton::Close => {
            let a = rect.center() + egui::vec2(-4.5, -4.5);
            let b = rect.center() + egui::vec2(4.5, 4.5);
            let c = rect.center() + egui::vec2(4.5, -4.5);
            let d = rect.center() + egui::vec2(-4.5, 4.5);
            ui.painter().line_segment([a, b], stroke);
            ui.painter().line_segment([c, d], stroke);
        }
        StickyLayoutWindowButton::Pin => {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "📌",
                FontId::proportional(14.0),
                color,
            );
        }
    }

    response
}

#[derive(Clone, Copy)]
struct LayoutGeometry {
    offset_x: f32,
    offset_y: f32,
    unit: f32,
    padding: f32,
    layout_h: f32,
}

fn responsive_layout_max_scale(ctx: &egui::Context, viewport: egui::Rect) -> f32 {
    let native_scale = ctx
        .native_pixels_per_point()
        .unwrap_or_else(|| ctx.pixels_per_point() / ctx.zoom_factor().max(0.1))
        .max(1.0);
    let physical_short_side = (viewport.width().min(viewport.height()) * native_scale).max(0.0);
    let t = ((physical_short_side - 1_080.0) / (2_160.0 - 1_080.0)).clamp(0.0, 1.0);
    1.0 + 0.35 * t
}

fn layout_geometry(
    ctx: &egui::Context,
    layout: &KeyboardLayout,
    viewport: egui::Rect,
    ui_scale: f32,
) -> LayoutGeometry {
    layout_geometry_with_reserved(
        ctx,
        layout,
        viewport,
        ui_scale,
        LAYOUT_TOP_RESERVED_H,
        LAYOUT_BOTTOM_RESERVED_H,
        LAYOUT_FIT_MARGIN,
        None,
    )
}

fn preview_layout_geometry(
    ctx: &egui::Context,
    layout: &KeyboardLayout,
    viewport: egui::Rect,
    ui_scale: f32,
) -> LayoutGeometry {
    layout_geometry_with_reserved(
        ctx,
        layout,
        viewport,
        ui_scale,
        2.0,
        2.0,
        6.0,
        Some(f32::INFINITY),
    )
}

fn sticky_layout_default_window_size() -> Vec2 {
    egui::vec2(STICKY_LAYOUT_WINDOW_W, STICKY_LAYOUT_WINDOW_H)
}

fn sticky_layout_saved_window_size(settings: &AppSettings) -> Vec2 {
    settings
        .sticky_layout_window_size
        .map(|[w, h]| egui::vec2(w.max(STICKY_LAYOUT_WINDOW_W), h.max(STICKY_LAYOUT_WINDOW_H)))
        .unwrap_or_else(sticky_layout_default_window_size)
}

fn sticky_layout_content_aspect(layout: Option<&KeyboardLayout>) -> f32 {
    let Some(layout) = layout else {
        return STICKY_LAYOUT_WINDOW_W / (STICKY_LAYOUT_WINDOW_H - STICKY_LAYOUT_WINDOW_TITLE_H);
    };
    let mut min_x: f32 = f32::MAX;
    let mut min_y: f32 = f32::MAX;
    let mut max_x: f32 = f32::MIN;
    let mut max_y: f32 = f32::MIN;
    for key in &layout.keys {
        min_x = min_x.min(key.x);
        min_y = min_y.min(key.y);
        max_x = max_x.max(key.x + key.w);
        max_y = max_y.max(key.y + key.h);
    }
    for encoder in &layout.encoders {
        min_x = min_x.min(encoder.x);
        min_y = min_y.min(encoder.y);
        max_x = max_x.max(encoder.x + encoder.w);
        max_y = max_y.max(encoder.y + encoder.h);
    }
    if min_x == f32::MAX {
        return STICKY_LAYOUT_WINDOW_W / (STICKY_LAYOUT_WINDOW_H - STICKY_LAYOUT_WINDOW_TITLE_H);
    }
    ((max_x - min_x) / (max_y - min_y).max(0.1)).clamp(0.4, 8.0)
}

fn sticky_layout_aspect_adjusted_window_size(
    layout: Option<&KeyboardLayout>,
    requested: Vec2,
    previous: Vec2,
) -> Vec2 {
    let min_size = sticky_layout_default_window_size();
    let requested = egui::vec2(requested.x.max(min_size.x), requested.y.max(min_size.y));
    let content_aspect = sticky_layout_content_aspect(layout);
    let width_changed = (requested.x - previous.x).abs() >= (requested.y - previous.y).abs();
    let mut size = if width_changed {
        egui::vec2(
            requested.x,
            (requested.x / content_aspect) + STICKY_LAYOUT_WINDOW_TITLE_H,
        )
    } else {
        egui::vec2(
            ((requested.y - STICKY_LAYOUT_WINDOW_TITLE_H).max(1.0)) * content_aspect,
            requested.y,
        )
    };
    if size.x < min_size.x {
        size.x = min_size.x;
        size.y = (size.x / content_aspect) + STICKY_LAYOUT_WINDOW_TITLE_H;
    }
    if size.y < min_size.y {
        size.y = min_size.y;
        size.x = ((size.y - STICKY_LAYOUT_WINDOW_TITLE_H).max(1.0)) * content_aspect;
    }
    size
}

fn layout_geometry_with_reserved(
    ctx: &egui::Context,
    layout: &KeyboardLayout,
    viewport: egui::Rect,
    ui_scale: f32,
    top_reserved: f32,
    bottom_reserved: f32,
    fit_margin: f32,
    max_scale_override: Option<f32>,
) -> LayoutGeometry {
    let mut min_x: f32 = f32::MAX;
    let mut min_y: f32 = f32::MAX;
    let mut max_x: f32 = f32::MIN;
    let mut max_y: f32 = f32::MIN;
    for key in &layout.keys {
        min_x = min_x.min(key.x);
        min_y = min_y.min(key.y);
        max_x = max_x.max(key.x + key.w);
        max_y = max_y.max(key.y + key.h);
    }
    for encoder in &layout.encoders {
        min_x = min_x.min(encoder.x);
        min_y = min_y.min(encoder.y);
        max_x = max_x.max(encoder.x + encoder.w);
        max_y = max_y.max(encoder.y + encoder.h);
    }
    if min_x == f32::MAX {
        min_x = 0.0;
        min_y = 0.0;
        max_x = 1.0;
        max_y = 1.0;
    }

    let span_x = max_x - min_x;
    let span_y = max_y - min_y;
    let fit_width = viewport.width() * ui_scale;
    let fit_height = viewport.height() * ui_scale;
    let scale_x = (fit_width - fit_margin) / (span_x * LAYOUT_BASE_UNIT).max(1.0);
    let scale_y = (fit_height - fit_margin) / (span_y * LAYOUT_BASE_UNIT).max(1.0);
    let max_scale =
        max_scale_override.unwrap_or_else(|| responsive_layout_max_scale(ctx, viewport));
    let scale = scale_x.min(scale_y).min(max_scale);
    let unit = LAYOUT_BASE_UNIT * scale;
    let layout_w = span_x * unit;
    let layout_h = span_y * unit;
    let content_top = viewport.top() + top_reserved;
    let content_bottom = viewport.bottom() - bottom_reserved;

    LayoutGeometry {
        offset_x: viewport.center().x - layout_w / 2.0 - min_x * unit,
        offset_y: ((content_top + content_bottom) - layout_h) / 2.0 - min_y * unit,
        unit,
        padding: LAYOUT_KEY_PADDING,
        layout_h,
    }
}

fn layout_keycap_rect(
    offset_x: f32,
    offset_y: f32,
    unit: f32,
    padding: f32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(offset_x + x * unit + padding, offset_y + y * unit + padding),
        Vec2::new(w * unit - padding * 2.0, h * unit - padding * 2.0),
    )
}

fn layout_physical_key_rect(key: &PhysicalKey, geometry: LayoutGeometry) -> egui::Rect {
    layout_keycap_rect(
        geometry.offset_x,
        geometry.offset_y,
        geometry.unit,
        geometry.padding,
        key.x,
        key.y,
        key.w,
        key.h,
    )
}

fn layout_physical_encoder_rect(encoder: &PhysicalEncoder, geometry: LayoutGeometry) -> egui::Rect {
    layout_keycap_rect(
        geometry.offset_x,
        geometry.offset_y,
        geometry.unit,
        geometry.padding,
        encoder.x,
        encoder.y,
        encoder.w,
        encoder.h,
    )
}

fn paint_layout_keycap(
    painter: &egui::Painter,
    rect: egui::Rect,
    rotation: f32,
    fill: Color32,
    stroke: Stroke,
) {
    if rotation == 0.0 {
        painter.rect(rect, 6.0, fill, stroke, egui::StrokeKind::Inside);
        return;
    }

    let angle = rotation.to_radians();
    let center = rect.center();
    let rotate = |pos: egui::Pos2| {
        let dx = pos.x - center.x;
        let dy = pos.y - center.y;
        egui::pos2(
            center.x + dx * angle.cos() - dy * angle.sin(),
            center.y + dx * angle.sin() + dy * angle.cos(),
        )
    };

    let radius = 6.0_f32.min(rect.width() * 0.5).min(rect.height() * 0.5);
    let corner_segments = 5;
    let mut points = Vec::with_capacity((corner_segments + 1) * 4);
    let corners = [
        (
            egui::pos2(rect.right() - radius, rect.top() + radius),
            -std::f32::consts::FRAC_PI_2,
            0.0,
        ),
        (
            egui::pos2(rect.right() - radius, rect.bottom() - radius),
            0.0,
            std::f32::consts::FRAC_PI_2,
        ),
        (
            egui::pos2(rect.left() + radius, rect.bottom() - radius),
            std::f32::consts::FRAC_PI_2,
            std::f32::consts::PI,
        ),
        (
            egui::pos2(rect.left() + radius, rect.top() + radius),
            std::f32::consts::PI,
            std::f32::consts::PI * 1.5,
        ),
    ];
    for (corner_center, start, end) in corners {
        for step in 0..=corner_segments {
            let t = step as f32 / corner_segments as f32;
            let theta = start + (end - start) * t;
            let point = egui::pos2(
                corner_center.x + radius * theta.cos(),
                corner_center.y + radius * theta.sin(),
            );
            points.push(rotate(point));
        }
    }

    painter.add(egui::Shape::convex_polygon(points, fill, stroke));
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

fn sticky_momentary_layer_target(kc: u16) -> Option<usize> {
    if let Some((op, target)) = vial_layer_op_target(kc) {
        return matches!(op, 1 | 4 | 6).then_some(target); // MO / OSL / TT while held
    }
    if kc & 0xF000 == 0x4000 {
        return Some(((kc >> 8) & 0xF) as usize); // LT
    }
    if (0x5000..0x5200).contains(&kc) {
        return Some(((kc >> 4) & 0xF) as usize); // LM
    }
    None
}

fn sticky_toggle_layer_target(kc: u16) -> Option<usize> {
    // TG
    vial_layer_op_target(kc).and_then(|(op, target)| (op == 3).then_some(target))
}

fn sticky_base_layer_target(kc: u16) -> Option<usize> {
    // TO / DF / PDF
    vial_layer_op_target(kc).and_then(|(op, target)| matches!(op, 0 | 2 | 7).then_some(target))
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

fn rgb_effect_options(state: &RgbSettingsState) -> Vec<(u16, &'static str)> {
    match state.kind {
        RgbSupportKind::QmkRgblight => QMK_RGBLIGHT_EFFECTS.to_vec(),
        RgbSupportKind::VialRgb => VIALRGB_EFFECTS
            .iter()
            .copied()
            .filter(|(id, _)| {
                *id == 0
                    || state.supported_effects.is_empty()
                    || state.supported_effects.contains(id)
            })
            .collect(),
        RgbSupportKind::None => vec![],
    }
}

fn rgb_effect_supports_color(kind: RgbSupportKind, effect: u16) -> bool {
    match kind {
        RgbSupportKind::QmkRgblight => matches!(effect, 1..=5 | 15..=36),
        RgbSupportKind::VialRgb => effect != 0,
        RgbSupportKind::None => false,
    }
}

fn rgb_effect_supports_speed(kind: RgbSupportKind, effect: u16) -> bool {
    match kind {
        RgbSupportKind::QmkRgblight => matches!(effect, 2..=36),
        RgbSupportKind::VialRgb => !matches!(effect, 0..=5),
        RgbSupportKind::None => false,
    }
}

fn rgb_picker_contrast(color: impl Into<egui::Rgba>) -> Color32 {
    if color.into().intensity() < 0.5 {
        Color32::WHITE
    } else {
        Color32::BLACK
    }
}

fn compact_rgb_slider_1d(
    ui: &mut egui::Ui,
    value: &mut f32,
    color_at: impl Fn(f32) -> Color32,
) -> bool {
    let desired_size = Vec2::new(ui.spacing().slider_width, 18.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
    let mut changed = false;

    if let Some(pos) = response.interact_pointer_pos() {
        let new_value = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        if (*value - new_value).abs() > f32::EPSILON {
            *value = new_value;
            changed = true;
        }
    }

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let rect = rect.expand(visuals.expansion);
        let mut mesh = egui::epaint::Mesh::default();
        const N: u32 = 36;
        for i in 0..N {
            let t = i as f32 / (N - 1) as f32;
            let x = egui::lerp(rect.x_range(), t);
            mesh.colored_vertex(egui::pos2(x, rect.top()), color_at(t));
            mesh.colored_vertex(egui::pos2(x, rect.bottom()), color_at(t));
            if i + 1 < N {
                let idx = i * 2;
                mesh.add_triangle(idx, idx + 1, idx + 2);
                mesh.add_triangle(idx + 1, idx + 2, idx + 3);
            }
        }
        ui.painter().add(egui::Shape::mesh(mesh));

        let x = egui::lerp(rect.x_range(), *value);
        let picked_color = color_at(*value);
        let stroke = Stroke::new(1.2, rgb_picker_contrast(picked_color));
        let handle_rect = egui::Rect::from_center_size(
            egui::pos2(x, rect.center().y),
            Vec2::new(10.0, rect.height() + 6.0),
        );
        ui.painter().rect(
            handle_rect,
            3.0,
            picked_color,
            stroke,
            egui::StrokeKind::Inside,
        );
        ui.painter()
            .rect_stroke(rect, 2.0, visuals.bg_stroke, egui::StrokeKind::Inside);
    }

    changed
}

fn compact_rgb_slider_2d(
    ui: &mut egui::Ui,
    x_value: &mut f32,
    y_value: &mut f32,
    color_at: impl Fn(f32, f32) -> Color32,
) -> bool {
    let desired_size = Vec2::splat(ui.spacing().slider_width);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());
    let mut changed = false;

    if let Some(pos) = response.interact_pointer_pos() {
        let new_x_value = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        let new_y_value = ((rect.bottom() - pos.y) / rect.height()).clamp(0.0, 1.0);
        if (*x_value - new_x_value).abs() > f32::EPSILON {
            *x_value = new_x_value;
            changed = true;
        }
        if (*y_value - new_y_value).abs() > f32::EPSILON {
            *y_value = new_y_value;
            changed = true;
        }
    }

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let rect = rect.expand(visuals.expansion);
        let mut mesh = egui::epaint::Mesh::default();
        const N: u32 = 36;
        for xi in 0..N {
            let xt = xi as f32 / (N - 1) as f32;
            let x = egui::lerp(rect.x_range(), xt);
            for yi in 0..N {
                let yt = yi as f32 / (N - 1) as f32;
                let y = egui::lerp(rect.y_range(), 1.0 - yt);
                mesh.colored_vertex(egui::pos2(x, y), color_at(xt, yt));
                if xi + 1 < N && yi + 1 < N {
                    let tl = yi + xi * N;
                    mesh.add_triangle(tl, tl + 1, tl + N);
                    mesh.add_triangle(tl + 1, tl + N, tl + N + 1);
                }
            }
        }
        ui.painter().add(egui::Shape::mesh(mesh));

        let x = egui::lerp(rect.x_range(), *x_value);
        let y = egui::lerp(rect.y_range(), 1.0 - *y_value);
        let picked_color = color_at(*x_value, *y_value);
        let stroke = Stroke::new(1.6, rgb_picker_contrast(picked_color));
        ui.painter().circle_stroke(egui::pos2(x, y), 10.0, stroke);
        ui.painter()
            .rect_stroke(rect, 2.0, visuals.bg_stroke, egui::StrokeKind::Inside);
    }

    changed
}

fn compact_rgb_color_picker(ui: &mut egui::Ui, hsva: &mut egui::ecolor::Hsva) -> bool {
    let mut changed = false;
    let mut h = hsva.h.rem_euclid(1.0);
    let mut s = hsva.s.clamp(0.0, 1.0);
    let mut v = hsva.v.clamp(0.0, 1.0);

    changed |= compact_rgb_slider_2d(ui, &mut s, &mut v, |s, v| {
        egui::ecolor::Hsva { h, s, v, a: 1.0 }.into()
    });
    ui.add_space(6.0);
    changed |= compact_rgb_slider_1d(ui, &mut h, |h| {
        egui::ecolor::Hsva {
            h,
            s: 1.0,
            v: 1.0,
            a: 1.0,
        }
        .into()
    });

    if changed {
        hsva.h = h;
        hsva.s = s;
        hsva.v = v;
    }
    changed
}

#[cfg(not(target_arch = "wasm32"))]
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

    #[cfg(not(target_arch = "wasm32"))]
    fn clear_connected_keyboard_state(&mut self, status_msg: impl Into<String>) {
        self.layout = None;
        self.selected_key = None;
        self.selected_encoder = None;
        self.selected_layer = 0;
        self.layer_count = 0;
        self.qmk_hid_hosts.clear();
        self.hid_device = None;
        self.connect_state = ConnectState::Idle;
        self.unlock_open = false;
        self.vial_unlock_polling = false;
        self.keycode_picker.open = false;
        self.current_device_name.clear();
        self.mouse_keys_settings = MouseKeysSettingsState::default();
        self.touchpad_settings = TouchpadSettingsState::default();
        self.module_settings = ModuleSettingsState::default();
        self.tap_hold_settings = TapHoldSettingsState::default();
        self.magic_settings = MagicSettingsState::default();
        self.one_shot_settings = OneShotSettingsState::default();
        self.grave_escape_settings = GraveEscapeSettingsState::default();
        self.layer_led_settings = LayerLedSettingsState::default();
        self.rgb_settings = RgbSettingsState::default();
        self.layout_options_value = None;
        self.sticky_layout_prev_pressed.clear();
        self.sticky_layout_pressed_key_layers.clear();
        self.sticky_layout_toggled_layers.clear();
        self.sticky_layout_base_layer = 0;
        self.status_msg = status_msg.into();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn start_device_scan(&mut self) {
        if !matches!(self.device_scan_state, DeviceScanState::Idle) {
            return;
        }

        let (tx, rx) = mpsc::channel();
        self.device_scan_state = DeviceScanState::Scanning(rx);
        std::thread::spawn(move || {
            let _ = tx.send(DeviceManager::scan_devices());
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn poll_device_scan(&mut self, ctx: &egui::Context) {
        let devices = match &self.device_scan_state {
            DeviceScanState::Idle => return,
            DeviceScanState::Scanning(rx) => match rx.try_recv() {
                Ok(devices) => Some(devices),
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(25));
                    return;
                }
                Err(mpsc::TryRecvError::Disconnected) => Some(Vec::new()),
            },
        };

        self.device_scan_state = DeviceScanState::Idle;
        if let Some(devices) = devices {
            self.apply_device_scan_result(devices);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn apply_device_scan_result(&mut self, devices: Vec<Device>) {
        let previous_path = self
            .selected_device
            .and_then(|idx| self.device_manager.devices().get(idx))
            .map(|dev| dev.path.clone());
        let was_loading = matches!(self.connect_state, ConnectState::Loading(_));

        self.device_manager.replace_devices(devices);

        if self.device_manager.devices().is_empty() {
            if self.selected_device.is_some() || self.layout.is_some() || was_loading {
                self.selected_device = None;
                self.clear_connected_keyboard_state("No keyboard detected");
            } else {
                self.qmk_hid_hosts.clear();
            }
            return;
        }

        if let Some(path) = previous_path {
            if let Some(idx) = self
                .device_manager
                .devices()
                .iter()
                .position(|dev| dev.path == path)
            {
                self.selected_device = Some(idx);
                if self.layout.is_none() && !was_loading {
                    self.start_connect(idx);
                } else {
                    self.sync_qmk_hid_host_bridges();
                }
                return;
            }
        }

        self.selected_device = Some(0);
        self.start_connect(0);
    }

    /// Spawn background thread to connect + load layout/keycodes.
    fn start_connect(&mut self, device_idx: usize) {
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
        self.sync_qmk_hid_host_bridges();
        self.hid_device = None;
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
        self.connect_state = ConnectState::Loading(rx);

        std::thread::spawn(move || {
            let result = (|| -> Result<ConnectResult, String> {
                use crate::hid::HidDevice;

                log::info!("Opening HID device: {}", dev.path);
                let dev_conn =
                    HidDevice::open(&dev.path).map_err(|e| format!("Open failed: {e}"))?;

                log::info!("Getting protocol version…");
                match dev_conn.get_protocol_version() {
                    Ok(v) => log::info!("VIA protocol version: {v}"),
                    Err(e) => log::warn!("get_protocol_version failed: {e}"),
                }

                log::info!("Getting layer count…");
                let layer_count = dev_conn
                    .get_layer_count()
                    .map(|c| c as usize)
                    .unwrap_or_else(|e| {
                        log::warn!("get_layer_count failed: {e}, defaulting to 4");
                        4
                    });
                log::info!("Layer count: {layer_count}");

                log::info!("Getting layout JSON…");
                let json = dev_conn
                    .get_layout_json()
                    .map_err(|e| format!("Layout read failed: {e}"))?;

                let touchpad_settings_in_definition =
                    Self::layout_json_has_touchpad_settings(&json);
                let supported_qmk_settings = dev_conn.query_qmk_settings().unwrap_or_else(|e| {
                    log::warn!("qmk settings query failed: {e}");
                    Vec::new()
                });

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

                let num_keys = layout.keys.len();
                layout.layers = vec![vec![0u16; num_keys]; layer_count];

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

                let mut firmware_layer_names = Vec::new();
                for layer in 0..layer_count.min(16) {
                    match dev_conn.get_qmk_setting_string(200 + layer as u16) {
                        Ok(name) if !name.is_empty() => firmware_layer_names.push(name),
                        Ok(_) => firmware_layer_names.push(layer.to_string()),
                        Err(_) => {
                            firmware_layer_names.clear();
                            break;
                        }
                    }
                }
                if !firmware_layer_names.is_empty() {
                    layout.layer_names = firmware_layer_names;
                }

                // Read macros
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
                        vec![]
                    }
                };

                let (
                    tap_dance_count,
                    combo_count,
                    key_override_count,
                    alt_repeat_count,
                    dynamic_feature_bits,
                ) = match dev_conn.get_dynamic_entry_counts() {
                    Ok(counts) => counts,
                    Err(e) => {
                        log::warn!("get_dynamic_entry_counts: {e}");
                        (0, 0, 0, 0, 0)
                    }
                };
                let vial_features = VialFeatureSupport {
                    caps_word: dynamic_feature_bits & (1 << 0) != 0,
                    layer_lock: dynamic_feature_bits & (1 << 1) != 0,
                    persistent_default_layer: key_override_count > 0,
                    repeat_key: alt_repeat_count > 0,
                };

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

                let combo_term = match dev_conn.get_qmk_setting_u16(2) {
                    Ok(value) => Some(value),
                    Err(e) => {
                        log::warn!("get_qmk_setting_u16(combo_term): {e}");
                        None
                    }
                };
                let auto_shift_options = match dev_conn.get_qmk_setting_u8(3) {
                    Ok(value) => Some(AutoShiftOptionsState::from_bits(value)),
                    Err(e) => {
                        log::warn!("get_qmk_setting_u8(auto_shift_flags): {e}");
                        None
                    }
                };
                let auto_shift_timeout = match dev_conn.get_qmk_setting_u16(4) {
                    Ok(value) => Some(value),
                    Err(e) => {
                        log::warn!("get_qmk_setting_u16(auto_shift_timeout): {e}");
                        None
                    }
                };

                // Mouse keys settings (qsid 9..=17, all u16). If qsid 9 is unsupported,
                // we assume the whole group is unavailable.
                let mouse_keys_settings = {
                    let mut mk = MouseKeysSettingsState::default();
                    match dev_conn.get_qmk_setting_u8(9) {
                        Ok(v) => {
                            mk.delay = v as u16;
                            mk.supported = true;
                            let read = |qsid: u16| -> u16 {
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
                        Err(e) => {
                            log::warn!("get_qmk_setting_u16(mouse_keys delay): {e}");
                        }
                    }
                    mk
                };

                // Ergohaven K:03 Pro touchpad settings (qsid 120..=124). These qsids
                // overlap with other Ergohaven pointing devices, so expose the page only
                // for known K:03 Pro identities.
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

                let module_settings =
                    Self::read_module_settings(&json, &supported_qmk_settings, &dev_conn);

                // Tap-Hold settings. If qsid 7 is unsupported, we treat the page as unavailable.
                let tap_hold_settings = {
                    let mut th = TapHoldSettingsState::default();
                    match dev_conn.get_qmk_setting_u16(7) {
                        Ok(v) => {
                            th.tapping_term = v;
                            th.supported = true;
                            let read_bool = |qsid: u16| -> bool {
                                match dev_conn.get_qmk_setting_u8(qsid) {
                                    Ok(val) => val != 0,
                                    Err(e) => {
                                        log::warn!("get_qmk_setting_u8(tap_hold qsid {qsid}): {e}");
                                        false
                                    }
                                }
                            };
                            let read_u16 = |qsid: u16| -> u16 {
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
                            th.tapping_toggle = dev_conn
                                .get_qmk_setting_u8(20)
                                .map(|value| value as u16)
                                .unwrap_or_else(|e| {
                                    log::warn!("get_qmk_setting_u8(tap_hold qsid 20): {e}");
                                    0
                                });
                            th.chordal_hold = read_bool(26);
                            th.flow_tap = read_u16(27);
                        }
                        Err(e) => {
                            log::warn!("get_qmk_setting_u16(tap_hold tapping_term): {e}");
                        }
                    }
                    th
                };

                // Magic settings (qsid 21 bits 0..=9). These are global QMK runtime swaps/options.
                let magic_settings = {
                    match dev_conn.get_qmk_setting_u16(21) {
                        Ok(bits) => MagicSettingsState {
                            bits,
                            supported: true,
                        },
                        Err(e) => {
                            log::warn!("get_qmk_setting_u16(magic qsid 21): {e}");
                            MagicSettingsState::default()
                        }
                    }
                };

                // One Shot Keys settings (qsid 5..=6). These affect OSM(...) and OSL(...).
                let one_shot_settings = {
                    let mut os = OneShotSettingsState::default();
                    match dev_conn.get_qmk_setting_u8(5) {
                        Ok(v) => {
                            os.tap_toggle = v;
                            os.supported = true;
                            os.timeout = dev_conn.get_qmk_setting_u16(6).unwrap_or_else(|e| {
                                log::warn!("get_qmk_setting_u16(one_shot timeout qsid 6): {e}");
                                0
                            });
                        }
                        Err(e) => {
                            log::warn!("get_qmk_setting_u8(one_shot tap toggle qsid 5): {e}");
                        }
                    }
                    os
                };

                // Grave Escape settings (qsid 1 bits 0..=3). These affect KC_GESC,
                // not the physical Escape key.
                let grave_escape_settings = {
                    match dev_conn.get_qmk_setting_u8(1) {
                        Ok(bits) => GraveEscapeSettingsState {
                            bits,
                            supported: true,
                        },
                        Err(e) => {
                            log::warn!("get_qmk_setting_u8(grave_escape qsid 1): {e}");
                            GraveEscapeSettingsState::default()
                        }
                    }
                };

                // Ergohaven per-layer LED settings (qsid 300..=317). If qsid 300 is
                // unsupported, we assume the whole group is unavailable.
                let layer_led_settings = {
                    let mut leds = LayerLedSettingsState::default();
                    match dev_conn.get_qmk_setting_u8(300) {
                        Ok(v) => {
                            leds.layer_colors[0] = v;
                            leds.supported = true;
                            for layer in 1..16 {
                                let qsid = 300 + layer as u16;
                                leds.layer_colors[layer] =
                                    dev_conn.get_qmk_setting_u8(qsid).unwrap_or_else(|e| {
                                        log::warn!(
                                            "get_qmk_setting_u8(layer_led qsid {qsid}): {e}"
                                        );
                                        0
                                    });
                            }
                            leds.brightness = dev_conn
                                .get_qmk_setting_u16(316)
                                .unwrap_or_else(|e| {
                                    log::warn!("get_qmk_setting_u16(layer_led brightness): {e}");
                                    0
                                })
                                .min(255);
                            leds.timeout_mins =
                                dev_conn.get_qmk_setting_u8(317).unwrap_or_else(|e| {
                                    log::warn!("get_qmk_setting_u8(layer_led timeout): {e}");
                                    0
                                });
                        }
                        Err(e) => {
                            log::warn!("get_qmk_setting_u8(layer_led layer 0 color): {e}");
                        }
                    }
                    leds
                };

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

                // Read tap dance entries
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

                let alt_repeat_entries = {
                    let count = alt_repeat_count;
                    log::info!("Alt Repeat count: {count}");
                    let mut entries = Vec::new();
                    for i in 0..count {
                        match dev_conn.get_alt_repeat_key(i) {
                            Ok((keycode, alt_keycode, allowed_mods, options)) => {
                                entries.push(AltRepeatKeyEntry {
                                    keycode,
                                    alt_keycode,
                                    allowed_mods,
                                    options: AltRepeatKeyOptionsState::from_bits(options),
                                });
                            }
                            Err(e) => {
                                log::warn!("get_alt_repeat_key({i}): {e}");
                                entries.push(Default::default());
                            }
                        }
                    }
                    entries
                };

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

            let _ = tx.send(result);
        });
    }

    /// Poll background thread for connect result.
    #[cfg(not(target_arch = "wasm32"))]
    fn poll_connect(&mut self, ctx: &egui::Context) {
        let result = match &self.connect_state {
            ConnectState::Loading(rx) => match rx.try_recv() {
                Ok(r) => Some(r),
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint(); // keep polling
                    return;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.status_msg = "Connect thread died".into();
                    self.connect_state = ConnectState::Idle;
                    return;
                }
            },
            ConnectState::Idle => return,
        };

        self.connect_state = ConnectState::Idle;

        match result.unwrap() {
            Ok(r) => {
                self.layer_count = r.layer_count;
                self.firmware = r.layout.firmware;
                self.current_device_name = r.device_name.clone();
                if let Some(dev) = self
                    .selected_device
                    .and_then(|idx| self.device_manager.devices().get(idx))
                {
                    self.device_display_names
                        .insert(dev.path.clone(), r.device_name.clone());
                }
                self.keycode_picker.tap_dance_entries = r.tap_dance_entries.clone();
                self.combo_entries = r.combo_entries.clone();
                self.key_override_entries = r.key_override_entries.clone();
                self.alt_repeat_entries = r.alt_repeat_entries.clone();
                self.alt_repeat_names = load_alt_repeat_names(&self.current_device_name);
                self.alt_repeat_names
                    .resize(self.alt_repeat_entries.len(), String::new());
                self.alt_repeat_undo_stack.clear();
                self.selected_alt_repeat = 0;
                self.alt_repeat_visible_count = if self.alt_repeat_entries.is_empty() {
                    1
                } else {
                    1.min(self.alt_repeat_entries.len())
                };
                self.key_override_names = load_key_override_names(&self.current_device_name);
                self.key_override_names
                    .resize(self.key_override_entries.len(), String::new());
                self.key_override_visible_count = 1;
                self.key_override_undo_stack.clear();
                self.selected_key_override = 0;
                self.combo_names = load_combo_names(&self.current_device_name);
                self.combo_names
                    .resize(self.combo_entries.len(), String::new());
                self.combo_term = r.combo_term.or(Some(50));
                self.auto_shift_options = r.auto_shift_options;
                self.auto_shift_timeout = r.auto_shift_timeout;
                self.auto_shift_timeout_text = r
                    .auto_shift_timeout
                    .map(|timeout| timeout.to_string())
                    .unwrap_or_default();
                self.mouse_keys_settings = r.mouse_keys_settings;
                self.touchpad_settings = r.touchpad_settings;
                self.module_settings = r.module_settings;
                self.tap_hold_settings = r.tap_hold_settings;
                self.magic_settings = r.magic_settings;
                self.one_shot_settings = r.one_shot_settings;
                self.grave_escape_settings = r.grave_escape_settings;
                self.layer_led_settings = r.layer_led_settings;
                self.rgb_settings = r.rgb_settings;
                self.layout_options_value = r.layout_options_value;
                let highest_used_combo = self
                    .combo_entries
                    .iter()
                    .enumerate()
                    .filter(|(i, combo)| {
                        combo.output != 0
                            || combo.keys.iter().any(|&k| k != 0)
                            || self
                                .combo_names
                                .get(*i)
                                .map(|n| !n.trim().is_empty())
                                .unwrap_or(false)
                    })
                    .map(|(i, _)| i + 1)
                    .max()
                    .unwrap_or(1);
                self.combo_visible_count = highest_used_combo.min(self.combo_entries.len().max(1));
                self.selected_combo = self
                    .selected_combo
                    .min(self.combo_visible_count.saturating_sub(1));
                if !r.macro_texts.is_empty() {
                    self.keycode_picker.macro_count = r.macro_texts.len();
                    self.keycode_picker.macro_texts = r.macro_texts.clone();
                    self.keycode_picker.macro_names = vec![String::new(); r.macro_texts.len()];
                    // Parse macro texts into actions
                    // Parse macro texts → actions (Vial protocol v2: prefix 0x01 before actions)
                    self.keycode_picker.macro_actions = r
                        .macro_texts
                        .iter()
                        .map(|text| {
                            let bytes = text.as_bytes();
                            let mut actions = Vec::new();
                            let mut i = 0;
                            while i < bytes.len() {
                                if bytes[i] == 1 && i + 1 < bytes.len() {
                                    // SS_QMK_PREFIX
                                    match bytes[i + 1] {
                                        1 if i + 2 < bytes.len() => {
                                            // SS_TAP
                                            actions.push(crate::keycode_picker::MacroAction::Tap(
                                                bytes[i + 2],
                                            ));
                                            i += 3;
                                        }
                                        2 if i + 2 < bytes.len() => {
                                            // SS_DOWN
                                            actions.push(crate::keycode_picker::MacroAction::Down(
                                                bytes[i + 2],
                                            ));
                                            i += 3;
                                        }
                                        3 if i + 2 < bytes.len() => {
                                            // SS_UP
                                            actions.push(crate::keycode_picker::MacroAction::Up(
                                                bytes[i + 2],
                                            ));
                                            i += 3;
                                        }
                                        4 if i + 3 < bytes.len() => {
                                            // SS_DELAY
                                            let ms = (bytes[i + 2] as u16 - 1)
                                                + (bytes[i + 3] as u16 - 1) * 255;
                                            actions.push(
                                                crate::keycode_picker::MacroAction::Delay(ms),
                                            );
                                            i += 4;
                                        }
                                        _ => {
                                            i += 2;
                                        } // skip unknown
                                    }
                                } else {
                                    // Text character
                                    let start = i;
                                    while i < bytes.len() && bytes[i] != 1 {
                                        i += 1;
                                    }
                                    if let Ok(s) = std::str::from_utf8(&bytes[start..i]) {
                                        actions.push(crate::keycode_picker::MacroAction::Text(
                                            s.to_string(),
                                        ));
                                    }
                                }
                            }
                            actions
                        })
                        .collect();
                }

                self.status_msg = format!("Connected: {}", r.device_name);

                // Load per-device layer names
                let device_name = r.device_name.clone();
                // Prefer names from descriptor/firmware, then overlay local overrides only if a real saved file exists
                let mut layer_names = r.layout.layer_names.clone();
                if let Some(local_layer_names) = load_saved_layer_names(&device_name) {
                    layer_names = local_layer_names;
                }
                if layer_names.is_empty() {
                    layer_names = load_layer_names(&device_name);
                }
                self.layer_names = layer_names;

                let encoder_count = r.layout.encoder_count();
                self.encoder_visibility = load_encoder_visibility(&device_name, encoder_count);

                // Populate picker
                self.keycode_picker.supports_rgb =
                    r.layout.supports_rgb || self.rgb_settings.supported;
                self.keycode_picker.supports_macro = !r.macro_texts.is_empty();
                self.keycode_picker.supports_tap_dance = !r.tap_dance_entries.is_empty();
                self.keycode_picker.supports_mouse_keys = self.mouse_keys_settings.supported;
                self.keycode_picker.supports_combo = !self.combo_entries.is_empty();
                self.keycode_picker.supports_auto_shift = self.auto_shift_timeout.is_some();
                self.keycode_picker.supports_caps_word = r.vial_features.caps_word;
                self.keycode_picker.supports_repeat_key = r.vial_features.repeat_key;
                self.keycode_picker.supports_layer_lock = r.vial_features.layer_lock;
                self.keycode_picker.supports_persistent_default_layer =
                    r.vial_features.persistent_default_layer;
                self.keycode_picker.layer_count = r.layout.layers.len().max(1);
                self.keycode_picker.tap_dance_names = load_tap_dance_names(&device_name);
                // Vial GUI maps customKeycodes to USER00.. at QK_KB + index.
                // Protocol v6: QK_KB = 0x7E00. Do not use QK_USER (0x7E40):
                // assigning those values writes the wrong keycodes to firmware.
                const QK_KB: u16 = 0x7E00;
                self.keycode_picker.custom_keycodes = r
                    .layout
                    .custom_keycodes
                    .iter()
                    .enumerate()
                    .map(|(i, custom)| {
                        (
                            custom.name.clone(),
                            custom.label.clone(),
                            custom.title.clone(),
                            QK_KB + i as u16,
                        )
                    })
                    .collect();
                self.keycode_picker.layer_names = self.layer_names.clone();
                self.sticky_layout_prev_pressed.clear();
                self.sticky_layout_pressed_key_layers.clear();
                self.sticky_layout_toggled_layers = vec![false; r.layout.layers.len().max(1)];
                self.sticky_layout_base_layer = 0;

                self.layout = Some(r.layout);
                self.refresh_layer_picker_content_flags();

                // Open persistent HID connection for Vial real-time writes
                if self.firmware == FirmwareProtocol::Vial {
                    if let Some(dev) = self
                        .selected_device
                        .and_then(|i| self.device_manager.devices().get(i))
                    {
                        match crate::hid::HidDevice::open(&dev.path) {
                            Ok(v) => {
                                self.hid_device = Some(v);
                                self.restore_entropy_display_preset_after_connect();
                            }
                            Err(e) => log::warn!("Could not open persistent HID: {e}"),
                        }
                        self.sync_qmk_hid_host_bridges();
                    }
                }

                log::info!(
                    "Connected: {} ({} layers, {:?})",
                    r.device_name,
                    r.layer_count,
                    self.firmware
                );
            }
            Err(e) => {
                self.status_msg = e;
            }
        }
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

    fn reset_matrix_tester_state(&mut self) {
        self.matrix_tester_pressed.clear();
        self.matrix_tester_ever_pressed.clear();
        self.sticky_layout_prev_pressed.clear();
        self.sticky_layout_pressed_key_layers.clear();
        self.sticky_layout_toggled_layers.clear();
        self.sticky_layout_base_layer = 0;
        self.matrix_tester_last_poll = std::time::Instant::now() - MATRIX_TESTER_POLL_INTERVAL;
        self.matrix_tester_last_lock_check =
            std::time::Instant::now() - MATRIX_TESTER_LOCK_CHECK_INTERVAL;
        self.matrix_tester_unlock_prompted = false;
        self.matrix_tester_lock_checked = false;
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

    #[cfg(not(target_arch = "wasm32"))]
    fn prompt_if_vial_locked_for_matrix_poll(&mut self) {
        if self.firmware != FirmwareProtocol::Vial
            || self.layout.is_none()
            || self.hid_device.is_none()
            || self.unlock_open
            || self.vial_unlock_polling
            || self.matrix_tester_unlock_prompted
        {
            return;
        }

        let now = std::time::Instant::now();
        if self.matrix_tester_lock_checked
            && now.duration_since(self.matrix_tester_last_lock_check)
                < MATRIX_TESTER_LOCK_CHECK_INTERVAL
        {
            return;
        }

        self.matrix_tester_lock_checked = true;
        self.matrix_tester_last_lock_check = now;
        if self.is_vial_locked() {
            self.unlock_open = true;
            self.matrix_tester_unlock_prompted = true;
            self.status_msg = crate::i18n::tr_catalog(
                self.app_settings.language,
                "matrix_tester.keyboard_is_locked_unlock_it_to_use_matrix_tester",
            )
            .into();
        }
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
                    self.status_msg = "Keyboard unlock cancelled".into();
                }
                Err(e) => {
                    self.status_msg = format!("Cancel unlock failed: {e}");
                }
            }
        }
        self.unlock_open = false;
        self.vial_unlock_polling = false;
        self.vial_unlock_counter = 0;
        self.vial_unlock_best = 50;
        self.matrix_tester_unlock_prompted = false;
        self.matrix_tester_lock_checked = false;
        if suppress_macro_auto_unlock {
            self.macro_auto_unlock_cancelled = true;
        }
        self.reopen_vial_hid();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn poll_switch_matrix_state(
        &mut self,
        ctx: &egui::Context,
        rows: usize,
        cols: usize,
        remember_ever_pressed: bool,
    ) {
        if self.firmware != FirmwareProtocol::Vial {
            return;
        }
        if self.unlock_open || self.vial_unlock_polling {
            return;
        }
        let Some(hid) = &self.hid_device else {
            return;
        };

        let now = std::time::Instant::now();
        if now.duration_since(self.matrix_tester_last_poll) >= MATRIX_TESTER_POLL_INTERVAL {
            self.matrix_tester_last_poll = now;
            match hid.get_switch_matrix(rows, cols) {
                Ok(pressed) => {
                    if remember_ever_pressed {
                        if self.matrix_tester_ever_pressed.len() != pressed.len() {
                            self.matrix_tester_ever_pressed = vec![false; pressed.len()];
                        }
                        for (idx, &is_pressed) in pressed.iter().enumerate() {
                            if is_pressed {
                                if let Some(seen) = self.matrix_tester_ever_pressed.get_mut(idx) {
                                    *seen = true;
                                }
                            }
                        }
                    }
                    self.matrix_tester_pressed = pressed;
                }
                Err(e) => {
                    log::warn!("Matrix poll error: {e}");
                    self.matrix_tester_lock_checked = false;
                    self.matrix_tester_last_lock_check =
                        std::time::Instant::now() - MATRIX_TESTER_LOCK_CHECK_INTERVAL;
                }
            }
        }
        ctx.request_repaint_after(MATRIX_TESTER_POLL_INTERVAL);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn poll_matrix_tester(&mut self, ctx: &egui::Context, layout: &KeyboardLayout) {
        if self.main_menu_tab != MainMenuTab::Settings {
            return;
        }
        self.poll_switch_matrix_state(ctx, layout.rows, layout.cols, true);
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

    fn draw_app_settings_page(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect) {
        use crate::i18n::Key as TrKey;

        let dark = ui.visuals().dark_mode;
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        let lang = self.app_settings.language;
        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(metrics.value(18.0));
                ui.label(
                    RichText::new(crate::i18n::tr(lang, TrKey::AppSettingsTitle))
                        .size(metrics.value(18.0))
                        .strong(),
                );
                ui.add_space(metrics.value(6.0));
                ui.label(
                    RichText::new(crate::i18n::tr(lang, TrKey::AppSettingsDescription))
                        .size(metrics.value(13.0))
                        .color(app_muted_text(dark)),
                );
                ui.add_space(metrics.value(24.0));

                const TOTAL_APP_SETTINGS_ROWS: usize = 9;
                let list = allocate_adaptive_settings_list_viewport(
                    ui,
                    "app_settings",
                    metrics,
                    TOTAL_APP_SETTINGS_ROWS,
                    metrics.value(44.0),
                );

                ui.allocate_ui_at_rect(list.content_rect, |ui| {
                    ui.set_clip_rect(list.viewport);
                    ui.set_min_size(list.content_rect.size());
                    ui.spacing_mut().item_spacing.y = 0.0;
                    self.draw_app_settings_editor_content(
                        ui,
                        list.first_visible_row..list.last_visible_row,
                        list.row_content_width,
                        list.row_height,
                        metrics,
                        dark,
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
            });
        });
    }

    fn draw_app_settings_editor_content(
        &mut self,
        ui: &mut egui::Ui,
        row_range: std::ops::Range<usize>,
        content_width: f32,
        row_height: f32,
        metrics: crate::ui_style::ResponsiveMetrics,
        dark: bool,
        suppress_tooltips: bool,
    ) {
        use crate::i18n::Key as TrKey;

        let lang = self.app_settings.language;
        let switch_width = metrics.value(46.0);
        let switch_size = metrics.size(46.0, 24.0);
        let tooltip = |text: &'static str| (!suppress_tooltips).then_some(text);

        for row_idx in row_range {
            match row_idx {
                0 => {
                    let mut selected_language = self.app_settings.language;
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        crate::i18n::tr(lang, TrKey::LanguageLabel),
                        true,
                        tooltip(crate::i18n::tr(lang, TrKey::LanguageTooltip)),
                        metrics.settings_control_width(),
                        |ui| {
                            let dropdown_id = ui.make_persistent_id("app_language_dropdown");
                            let dropdown_resp = crate::ui_style::modern_dropdown_button_sized(
                                ui,
                                dropdown_id,
                                selected_language.native_name(),
                                ui.visuals().text_color(),
                                metrics.settings_control_width(),
                                metrics.settings_control_height(),
                                metrics.settings_control_font_size(),
                            );
                            egui::popup_below_widget(
                                ui,
                                dropdown_id,
                                &dropdown_resp,
                                egui::PopupCloseBehavior::CloseOnClickOutside,
                                |ui| {
                                    ui.set_min_width(metrics.settings_control_width());
                                    ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
                                    for language in crate::i18n::Language::ALL {
                                        let selected = language == selected_language;
                                        let (option_rect, option_resp) = ui.allocate_exact_size(
                                            metrics.size(168.0, 28.0),
                                            Sense::click(),
                                        );
                                        if option_resp.hovered() {
                                            ui.ctx()
                                                .set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }
                                        let option_fill = if selected {
                                            if dark {
                                                Color32::from_rgb(58, 58, 61)
                                            } else {
                                                Color32::from_rgb(236, 236, 238)
                                            }
                                        } else if option_resp.hovered() {
                                            crate::ui_style::hover_fill(dark)
                                        } else {
                                            Color32::TRANSPARENT
                                        };
                                        ui.painter().rect_filled(option_rect, 7.0, option_fill);
                                        ui.painter().text(
                                            egui::pos2(
                                                option_rect.left() + metrics.value(10.0),
                                                option_rect.center().y,
                                            ),
                                            egui::Align2::LEFT_CENTER,
                                            language.native_name(),
                                            FontId::proportional(metrics.value(12.0)),
                                            if selected {
                                                ui.visuals().text_color()
                                            } else {
                                                app_muted_text(dark)
                                            },
                                        );
                                        if option_resp.clicked() {
                                            selected_language = language;
                                            ui.memory_mut(|m| m.close_popup());
                                        }
                                    }
                                },
                            );
                        },
                    );
                    if selected_language != self.app_settings.language {
                        self.app_settings.language = selected_language;
                        save_app_settings(&self.app_settings);
                    }
                }
                1 => {
                    let mut selected_key_legend_layout = self.app_settings.key_legend_layout;
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        crate::i18n::tr_catalog(lang, "ui.key_legends_label"),
                        true,
                        tooltip(crate::i18n::tr_catalog(lang, "ui.key_legends_tooltip")),
                        metrics.settings_control_width(),
                        |ui| {
                            let dropdown_id = ui.make_persistent_id("app_key_legends_dropdown");
                            let dropdown_resp = crate::ui_style::modern_dropdown_button_sized(
                                ui,
                                dropdown_id,
                                crate::i18n::tr_catalog(
                                    lang,
                                    selected_key_legend_layout.i18n_key(),
                                ),
                                ui.visuals().text_color(),
                                metrics.settings_control_width(),
                                metrics.settings_control_height(),
                                metrics.settings_control_font_size(),
                            );
                            egui::popup_below_widget(
                                ui,
                                dropdown_id,
                                &dropdown_resp,
                                egui::PopupCloseBehavior::CloseOnClickOutside,
                                |ui| {
                                    ui.set_min_width(metrics.settings_control_width());
                                    ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
                                    for key_legend_layout in KeyLegendLayout::ALL {
                                        let selected =
                                            key_legend_layout == selected_key_legend_layout;
                                        let (option_rect, option_resp) = ui.allocate_exact_size(
                                            metrics.size(168.0, 28.0),
                                            Sense::click(),
                                        );
                                        if option_resp.hovered() {
                                            ui.ctx()
                                                .set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }
                                        let option_fill = if selected {
                                            if dark {
                                                Color32::from_rgb(58, 58, 61)
                                            } else {
                                                Color32::from_rgb(236, 236, 238)
                                            }
                                        } else if option_resp.hovered() {
                                            crate::ui_style::hover_fill(dark)
                                        } else {
                                            Color32::TRANSPARENT
                                        };
                                        ui.painter().rect_filled(option_rect, 7.0, option_fill);
                                        ui.painter().text(
                                            egui::pos2(
                                                option_rect.left() + metrics.value(10.0),
                                                option_rect.center().y,
                                            ),
                                            egui::Align2::LEFT_CENTER,
                                            crate::i18n::tr_catalog(
                                                lang,
                                                key_legend_layout.i18n_key(),
                                            ),
                                            FontId::proportional(metrics.value(12.0)),
                                            if selected {
                                                ui.visuals().text_color()
                                            } else {
                                                app_muted_text(dark)
                                            },
                                        );
                                        if option_resp.clicked() {
                                            selected_key_legend_layout = key_legend_layout;
                                            ui.memory_mut(|m| m.close_popup());
                                        }
                                    }
                                },
                            );
                        },
                    );
                    if selected_key_legend_layout != self.app_settings.key_legend_layout {
                        self.app_settings.key_legend_layout = selected_key_legend_layout;
                        save_app_settings(&self.app_settings);
                    }
                }
                2 => {
                    let mut minimize_to_tray = self.app_settings.minimize_to_tray_on_close;
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        crate::i18n::tr(lang, TrKey::CloseToTrayLabel),
                        true,
                        tooltip(crate::i18n::tr(lang, TrKey::CloseToTrayTooltip)),
                        switch_width,
                        |ui| {
                            let _ = crate::ui_style::settings_switch_sized_stable(
                                ui,
                                "app_settings_minimize_to_tray",
                                &mut minimize_to_tray,
                                switch_size,
                            );
                        },
                    );
                    if minimize_to_tray != self.app_settings.minimize_to_tray_on_close {
                        self.app_settings.minimize_to_tray_on_close = minimize_to_tray;
                        if !minimize_to_tray {
                            #[cfg(target_os = "windows")]
                            {
                                self.tray_icon = None;
                            }
                        }
                        save_app_settings(&self.app_settings);
                    }
                }
                3 => {
                    let mut show_shifted_symbols = self.app_settings.show_shifted_number_symbols;
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        crate::i18n::tr(lang, TrKey::ShiftedNumberSymbolsLabel),
                        true,
                        tooltip(crate::i18n::tr(lang, TrKey::ShiftedNumberSymbolsTooltip)),
                        switch_width,
                        |ui| {
                            let _ = crate::ui_style::settings_switch_sized_stable(
                                ui,
                                "app_settings_shifted_symbols",
                                &mut show_shifted_symbols,
                                switch_size,
                            );
                        },
                    );
                    if show_shifted_symbols != self.app_settings.show_shifted_number_symbols {
                        self.app_settings.show_shifted_number_symbols = show_shifted_symbols;
                        save_app_settings(&self.app_settings);
                    }
                }
                4 => {
                    let mut layer_hover_preview = self.app_settings.layer_hover_preview;
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        crate::i18n::tr(lang, TrKey::LayerHoverPreviewLabel),
                        true,
                        tooltip(crate::i18n::tr(lang, TrKey::LayerHoverPreviewTooltip)),
                        switch_width,
                        |ui| {
                            let _ = crate::ui_style::settings_switch_sized_stable(
                                ui,
                                "app_settings_layer_hover_preview",
                                &mut layer_hover_preview,
                                switch_size,
                            );
                        },
                    );
                    if layer_hover_preview != self.app_settings.layer_hover_preview {
                        self.app_settings.layer_hover_preview = layer_hover_preview;
                        if !layer_hover_preview {
                            self.hover_layer = None;
                        }
                        save_app_settings(&self.app_settings);
                    }
                }
                5 => {
                    let mut encoder_hover_enlarge = self.app_settings.encoder_hover_enlarge;
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        crate::i18n::tr(lang, TrKey::EncoderHoverZoomLabel),
                        true,
                        tooltip(crate::i18n::tr(lang, TrKey::EncoderHoverZoomTooltip)),
                        switch_width,
                        |ui| {
                            let _ = crate::ui_style::settings_switch_sized_stable(
                                ui,
                                "app_settings_encoder_hover_enlarge",
                                &mut encoder_hover_enlarge,
                                switch_size,
                            );
                        },
                    );
                    if encoder_hover_enlarge != self.app_settings.encoder_hover_enlarge {
                        self.app_settings.encoder_hover_enlarge = encoder_hover_enlarge;
                        save_app_settings(&self.app_settings);
                    }
                }
                6 => {
                    let mut sticky_layout_window = self.app_settings.sticky_layout_window;
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        crate::i18n::tr_catalog(lang, "ui.sticky_layout_window_label"),
                        true,
                        tooltip(crate::i18n::tr_catalog(
                            lang,
                            "ui.sticky_layout_window_tooltip",
                        )),
                        switch_width,
                        |ui| {
                            let _ = crate::ui_style::settings_switch_sized_stable(
                                ui,
                                "app_settings_sticky_layout_window",
                                &mut sticky_layout_window,
                                switch_size,
                            );
                        },
                    );
                    if sticky_layout_window != self.app_settings.sticky_layout_window {
                        self.app_settings.sticky_layout_window = sticky_layout_window;
                        save_app_settings(&self.app_settings);
                    }
                }
                7 => {
                    let mut selected_accent = self.app_settings.accent_color;
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        crate::i18n::tr(lang, TrKey::AccentColorLabel),
                        true,
                        tooltip(crate::i18n::tr(lang, TrKey::AccentColorTooltip)),
                        metrics.value(218.0),
                        |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 7.0;
                                for accent in AppAccentColor::ALL {
                                    let color = accent.color();
                                    let selected = accent == selected_accent;
                                    let (rect, resp) = ui.allocate_exact_size(
                                        Vec2::new(26.0, 26.0),
                                        egui::Sense::click(),
                                    );
                                    if resp.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    }
                                    if resp.clicked() {
                                        selected_accent = accent;
                                    }
                                    let stroke = if selected {
                                        Stroke::new(2.0, color)
                                    } else {
                                        crate::ui_style::modal_outline_stroke(dark)
                                    };
                                    ui.painter().circle_filled(rect.center(), 8.5, color);
                                    ui.painter().circle_stroke(rect.center(), 11.0, stroke);
                                    if !suppress_tooltips {
                                        resp.on_hover_text(crate::i18n::tr_catalog(
                                            lang,
                                            accent.name(),
                                        ));
                                    }
                                }
                            });
                        },
                    );
                    if selected_accent != self.app_settings.accent_color {
                        self.app_settings.accent_color = selected_accent;
                        crate::ui_style::set_accent(selected_accent.color());
                        #[cfg(target_os = "windows")]
                        {
                            self.tray_icon = None;
                        }
                        save_app_settings(&self.app_settings);
                    }
                }
                8 => {
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        crate::i18n::tr_catalog(lang, "onboarding_tour.settings_row_label"),
                        true,
                        tooltip(crate::i18n::tr_catalog(
                            lang,
                            "onboarding_tour.settings_row_tooltip",
                        )),
                        metrics.settings_control_width(),
                        |ui| {
                            if crate::ui_style::modern_button(
                                ui,
                                crate::i18n::tr_catalog(lang, "onboarding_tour.show_again"),
                                metrics.size(168.0, 32.0),
                                true,
                            )
                            .clicked()
                            {
                                self.start_onboarding_tour(ui.ctx());
                            }
                        },
                    );
                }
                _ => {}
            }
        }
    }

    fn text_expander_rule_issue(
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

    fn active_text_expansion_rules(&self) -> Vec<crate::text_expander::TextExpansionRule> {
        let mut rules = self.app_settings.text_expansion_rules.clone();
        rules.extend(load_extra_text_expansion_rules(
            &self.app_settings.text_expander_rule_files,
        ));
        rules
    }

    fn sync_text_expander_runtime(&mut self) {
        crate::smart_input::set_text_expander_config(
            self.app_settings.text_expander_enabled,
            self.active_text_expansion_rules(),
            parse_text_expander_blacklist(&self.app_settings.text_expander_app_blacklist),
        );
    }

    fn save_text_expander_settings(&mut self) {
        save_text_expansion_rules(&self.app_settings.text_expansion_rules);
        self.text_expander_rules_signature =
            text_expander_rules_signature(&self.app_settings.text_expander_rule_files);
        save_app_settings(&self.app_settings);
        self.sync_text_expander_runtime();
    }

    fn add_text_expander_blacklist_app(&mut self, app_name: &str) -> bool {
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

    fn remove_text_expander_blacklist_app(&mut self, app_name: &str) -> bool {
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

    fn text_expander_window_candidates(
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

    fn reload_text_expander_rules_file(&mut self) -> bool {
        if let Some(rules) = load_text_expansion_rules() {
            self.app_settings.text_expansion_rules = rules;
            self.save_text_expander_settings();
            true
        } else {
            false
        }
    }

    fn auto_reload_text_expander_rules_file(&mut self, now: f64) {
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

    fn open_text_expander_rules_folder(&mut self) {
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

    fn remove_text_expander_rules_file(&mut self, remove_idx: usize) {
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

    fn draw_text_expander_settings_page(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect) {
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

    fn draw_text_expander_editor_content(
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

    fn draw_universal_symbols_setup_page(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect) {
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
        let result = std::process::Command::new("sh").arg(script).status();
        self.status_msg = if matches!(result, Ok(status) if status.success()) {
            format!("{backend} backend installed; restart/select it in your input method")
        } else {
            format!("Could not run {script}; run it from the Entropy folder")
        };
    }

    fn draw_mouse_keys_settings_page(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect) {
        let lang = self.app_settings.language;
        let dark = ui.visuals().dark_mode;
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());

        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(18.0);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::MouseKeysTitle))
                        .size(18.0)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(crate::i18n::tr(
                        lang,
                        crate::i18n::Key::MouseKeysDescription,
                    ))
                    .size(13.0)
                    .color(app_muted_text(dark)),
                );
                ui.add_space(24.0);

                if !self.mouse_keys_settings.supported {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::MouseKeysUnavailable),
                        Some(crate::i18n::tr(lang, crate::i18n::Key::MouseKeysEnableHint)),
                    );
                    return;
                }

                const TOTAL_MOUSE_KEY_ROWS: usize = 9;
                let list = allocate_adaptive_settings_list_viewport(
                    ui,
                    "mouse_keys_settings",
                    metrics,
                    TOTAL_MOUSE_KEY_ROWS,
                    0.0,
                );
                ui.allocate_ui_at_rect(list.content_rect, |ui| {
                    ui.set_clip_rect(list.viewport);
                    ui.set_min_size(list.content_rect.size());
                    ui.spacing_mut().item_spacing.y = 0.0;
                    self.draw_mouse_keys_editor_content(
                        ui,
                        list.first_visible_row..list.last_visible_row,
                        list.row_content_width,
                        list.row_height,
                        metrics.value(86.0),
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
            });
        });
    }

    fn draw_matrix_tester_settings(
        &mut self,
        ui: &mut egui::Ui,
        layout: &KeyboardLayout,
        content_rect: egui::Rect,
        dark: bool,
    ) {
        let title_y = content_rect.top() + 30.0;
        let desc_y = title_y + 28.0;
        let status_y = desc_y + 30.0;
        let supported = self.firmware == FirmwareProtocol::Vial;
        let hid_ready = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.hid_device.is_some()
            }
            #[cfg(target_arch = "wasm32")]
            {
                false
            }
        };

        if supported && hid_ready {
            #[cfg(not(target_arch = "wasm32"))]
            self.prompt_if_vial_locked_for_matrix_poll();
        }

        let total_keys = layout.keys.len();
        let tested_count = layout
            .keys
            .iter()
            .filter(|key| {
                let idx = key.row as usize * layout.cols + key.col as usize;
                self.matrix_tester_ever_pressed
                    .get(idx)
                    .copied()
                    .unwrap_or(false)
            })
            .count();

        ui.allocate_ui_at_rect(
            egui::Rect::from_min_max(
                egui::pos2(content_rect.left(), content_rect.top()),
                egui::pos2(content_rect.right(), desc_y + 10.0),
            ),
            |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(18.0);
                    ui.label(
                        RichText::new(crate::i18n::tr(
                            self.app_settings.language,
                            crate::i18n::Key::MatrixTesterTitle,
                        ))
                        .size(18.0)
                        .strong(),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(crate::i18n::tr(
                            self.app_settings.language,
                            crate::i18n::Key::MatrixTesterDescription,
                        ))
                        .size(13.0)
                        .color(app_muted_text(dark)),
                    );
                });
            },
        );

        let painter = ui.painter().clone();
        let complete = tested_count == total_keys && total_keys > 0;
        let status_prefix =
            crate::i18n::tr_catalog(self.app_settings.language, "matrix_tester.tested");
        let status_text = format!("{status_prefix}: {tested_count}/{total_keys}");
        let status_rect = egui::Rect::from_center_size(
            egui::pos2(content_rect.center().x, status_y),
            Vec2::new(132.0, 30.0),
        );
        let status_resp = ui.interact(
            status_rect,
            ui.id().with("matrix_tester_status_reset"),
            egui::Sense::click(),
        );
        if status_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if status_resp.clicked() {
            self.reset_matrix_tester_state();
        }
        let status_hovered = status_resp.hovered();
        status_resp.on_hover_text(crate::i18n::tr_catalog(
            self.app_settings.language,
            crate::i18n::tr_catalog(
                self.app_settings.language,
                "matrix_tester.click_to_reset_matrix_tester",
            ),
        ));
        painter.rect(
            status_rect,
            9.0,
            if status_hovered {
                crate::ui_style::hover_fill(dark)
            } else {
                app_surface_fill(dark)
            },
            crate::ui_style::modal_outline_stroke(dark),
            egui::StrokeKind::Inside,
        );
        painter.text(
            status_rect.center(),
            egui::Align2::CENTER_CENTER,
            status_text,
            FontId::proportional(13.0),
            if complete {
                app_accent()
            } else {
                app_muted_text(dark)
            },
        );
        let idle_fill = if dark {
            Color32::from_rgb(34, 34, 38)
        } else {
            Color32::from_rgb(252, 252, 254)
        };
        let tested_fill = crate::ui_style::hover_fill(dark);

        let board_top = content_rect.top() + 104.0;
        let hint_y = ui.max_rect().bottom() - 36.0;
        let board_rect = egui::Rect::from_min_max(
            egui::pos2(content_rect.left(), board_top),
            egui::pos2(content_rect.right(), hint_y - 22.0),
        );

        if !supported {
            painter.text(
                board_rect.center(),
                egui::Align2::CENTER_CENTER,
                crate::i18n::tr_catalog(
                    self.app_settings.language,
                    "matrix_tester.matrix_tester_is_currently_available_only_for_vial_keyboards",
                ),
                FontId::proportional(15.0),
                app_muted_text(dark),
            );
            return;
        }

        if !hid_ready {
            painter.text(
                board_rect.center(),
                egui::Align2::CENTER_CENTER,
                crate::i18n::tr_catalog(
                    self.app_settings.language,
                    "matrix_tester.connect_a_vial_keyboard_to_start_live_switch_testing",
                ),
                FontId::proportional(15.0),
                app_muted_text(dark),
            );
            return;
        }

        let viewport = egui::Rect::from_min_max(
            ui.min_rect().min,
            egui::pos2(
                ui.min_rect().left() + ui.available_size().x,
                ui.max_rect().bottom(),
            ),
        );
        let geometry = layout_geometry(
            ui.ctx(),
            layout,
            viewport,
            clamp_ui_scale(self.app_settings.ui_scale),
        );

        let hint_color = if dark {
            Color32::from_gray(100)
        } else {
            Color32::from_gray(160)
        };
        painter.text(
            egui::pos2(content_rect.center().x, hint_y),
            egui::Align2::CENTER_CENTER,
            crate::i18n::tr_catalog(
                self.app_settings.language,
                "matrix_tester.click_tested_to_reset_progress",
            ),
            FontId::proportional(11.0),
            hint_color,
        );

        for key in &layout.keys {
            let matrix_idx = key.row as usize * layout.cols + key.col as usize;
            let is_pressed = self
                .matrix_tester_pressed
                .get(matrix_idx)
                .copied()
                .unwrap_or(false);
            let was_pressed = self
                .matrix_tester_ever_pressed
                .get(matrix_idx)
                .copied()
                .unwrap_or(false);
            let rect = layout_physical_key_rect(key, geometry);

            let fill = if is_pressed {
                app_accent()
            } else if was_pressed {
                tested_fill
            } else {
                idle_fill
            };
            let stroke = if is_pressed {
                app_accent()
            } else if was_pressed {
                app_accent()
            } else {
                app_border_color(dark)
            };
            paint_layout_keycap(&painter, rect, key.rotation, fill, Stroke::new(1.0, stroke));
        }
    }

    fn draw_rgb_settings_page(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect, dark: bool) {
        let lang = self.app_settings.language;
        let hid_ready = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.hid_device.is_some()
            }
            #[cfg(target_arch = "wasm32")]
            {
                false
            }
        };

        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(18.0);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::RgbTitle))
                        .size(18.0)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::RgbDescription))
                        .size(13.0)
                        .color(app_muted_text(dark)),
                );
                ui.add_space(24.0);

                if !self.rgb_settings.supported {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::RgbUnavailableTooltip),
                        None,
                    );
                    return;
                }

                if !hid_ready {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::RgbConnect),
                        None,
                    );
                    return;
                }

                self.draw_rgb_editor_content(ui, dark, &RgbModalLayout::responsive(ui.ctx()));
            });
        });
    }

    fn draw_alt_repeat_settings_page(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect) {
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        let alt_repeat_page_width = metrics.settings_content_width();
        let alt_repeat_title_y_offset = metrics.value(30.0);
        let alt_repeat_desc_gap = metrics.value(28.0);
        let alt_repeat_block_top_gap = metrics.value(22.0);

        let dark = ui.visuals().dark_mode;
        let center_x = content_rect.center().x;
        let title_y = content_rect.top() + alt_repeat_title_y_offset;
        let desc_y = title_y + alt_repeat_desc_gap;
        let block_top = desc_y + alt_repeat_block_top_gap;
        let block_rect = egui::Rect::from_min_max(
            egui::pos2(center_x - alt_repeat_page_width / 2.0, block_top),
            egui::pos2(
                center_x + alt_repeat_page_width / 2.0,
                content_rect.bottom(),
            ),
        );

        ui.painter().text(
            egui::pos2(center_x, title_y),
            egui::Align2::CENTER_CENTER,
            crate::i18n::tr(self.app_settings.language, crate::i18n::Key::AltRepeatTitle),
            FontId::proportional(metrics.value(18.0)),
            ui.visuals().text_color(),
        );
        ui.painter().text(
            egui::pos2(center_x, desc_y),
            egui::Align2::CENTER_CENTER,
            crate::i18n::tr(
                self.app_settings.language,
                crate::i18n::Key::AltRepeatDescription,
            ),
            FontId::proportional(metrics.value(13.0)),
            app_muted_text(dark),
        );

        ui.allocate_ui_at_rect(block_rect, |ui| {
            self.draw_alt_repeat_editor_content(ui);
        });
    }

    fn draw_key_override_settings_page(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect) {
        let lang = self.app_settings.language;
        const KEY_OVERRIDE_PAGE_WIDTH: f32 = 548.0;
        const KEY_OVERRIDE_TITLE_Y_OFFSET: f32 = 18.0;
        const KEY_OVERRIDE_DESC_GAP: f32 = 6.0;
        const KEY_OVERRIDE_BLOCK_TOP_GAP: f32 = 18.0;

        let dark = ui.visuals().dark_mode;
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        let scale = metrics.scale;
        let page_width = KEY_OVERRIDE_PAGE_WIDTH * scale;

        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(KEY_OVERRIDE_TITLE_Y_OFFSET * scale);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::KeyOverridesTitle))
                        .size(18.0 * scale)
                        .strong(),
                );
                ui.add_space(KEY_OVERRIDE_DESC_GAP * scale);
                ui.label(
                    RichText::new(crate::i18n::tr(
                        lang,
                        crate::i18n::Key::KeyOverridesDescription,
                    ))
                    .size(13.0 * scale)
                    .color(app_muted_text(dark)),
                );
                ui.add_space(KEY_OVERRIDE_BLOCK_TOP_GAP * scale);
                let editor_height = (content_rect.height()
                    - KEY_OVERRIDE_TITLE_Y_OFFSET * scale
                    - KEY_OVERRIDE_DESC_GAP * scale
                    - KEY_OVERRIDE_BLOCK_TOP_GAP * scale
                    - 64.0 * scale)
                    .max(360.0 * scale);
                ui.allocate_ui_with_layout(
                    egui::vec2(page_width, editor_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        self.draw_key_override_editor_content(ui, true);
                    },
                );
            });
        });
    }

    fn draw_combo_settings_page(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        content_rect: egui::Rect,
    ) {
        self.handle_combo_editor_input(ctx, false);
        let lang = self.app_settings.language;
        let dark = ui.visuals().dark_mode;

        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                let scale = responsive_settings_editor_scale(ui.ctx());
                ui.add_space(18.0 * scale);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::ComboTitle))
                        .size(18.0 * scale)
                        .strong(),
                );
                ui.add_space(6.0 * scale);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::ComboDescription))
                        .size(13.0 * scale)
                        .color(app_muted_text(dark)),
                );
                ui.add_space(18.0 * scale);
                self.draw_combo_editor_content(ui, false);
            });
        });
    }

    fn draw_encoder_visibility_settings_page(
        &mut self,
        ui: &mut egui::Ui,
        content_rect: egui::Rect,
        dark: bool,
    ) {
        let lang = self.app_settings.language;
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        let encoders_content_width = metrics.settings_content_width();
        let encoders_row_height = metrics.settings_row_height();
        let encoders_top_padding = metrics.value(4.0);
        let switch_width = metrics.value(46.0);
        let switch_size = metrics.size(46.0, 24.0);

        let (encoder_indices, device_name) = self
            .layout
            .as_ref()
            .map(|layout| {
                let indices = layout
                    .encoders
                    .iter()
                    .map(|encoder| encoder.encoder_idx as usize)
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();
                (indices, layout.name.clone())
            })
            .unwrap_or((Vec::new(), String::new()));
        let visibility_len = encoder_indices
            .iter()
            .copied()
            .max()
            .map(|idx| idx + 1)
            .unwrap_or(0);

        if self.encoder_visibility.len() < visibility_len {
            self.encoder_visibility.resize(visibility_len, true);
        }
        self.encoder_visibility.truncate(visibility_len);

        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(18.0);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::EncodersTitle))
                        .size(18.0)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::EncodersDescription))
                        .size(13.0)
                        .color(app_muted_text(dark)),
                );
                ui.add_space(24.0);

                if encoder_indices.is_empty() {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::EncodersUnavailable),
                        None,
                    );
                    return;
                }

                crate::ui_style::modal_content(
                    ui,
                    crate::ui_style::ModalLayout::new(encoders_content_width)
                        .with_top_padding(encoders_top_padding),
                    |ui| {
                        for encoder_idx in &encoder_indices {
                            let mut visible = self.encoder_visibility[*encoder_idx];
                            let label = if matches!(
                                self.app_settings.language,
                                crate::i18n::Language::Russian
                            ) {
                                format!("Энкодер {}", encoder_idx + 1)
                            } else {
                                format!("Encoder {}", encoder_idx + 1)
                            };
                            crate::ui_style::settings_list_row(
                                ui,
                                encoders_content_width,
                                encoders_row_height,
                                &label,
                                true,
                                switch_width,
                                |ui| {
                                    let resp = crate::ui_style::settings_switch_sized(
                                        ui,
                                        &mut visible,
                                        switch_size,
                                    );
                                    if resp.changed() {
                                        self.encoder_visibility[*encoder_idx] = visible;
                                        if !device_name.is_empty() {
                                            save_encoder_visibility(
                                                &self.encoder_visibility,
                                                &device_name,
                                            );
                                        }
                                    }
                                },
                            );
                        }
                    },
                );
            });
        });
    }

    fn draw_layout_options_settings_page(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect) {
        let lang = self.app_settings.language;
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        let row_content_width = metrics.settings_row_content_width();
        let row_height = metrics.settings_row_height();
        let dropdown_width = metrics.value(220.0);
        let switch_width = metrics.value(46.0);
        let switch_size = metrics.size(46.0, 24.0);

        let dark = ui.visuals().dark_mode;
        let hid_ready = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.hid_device.is_some()
            }
            #[cfg(target_arch = "wasm32")]
            {
                false
            }
        };
        let options = self
            .layout
            .as_ref()
            .map(|layout| layout.layout_options.clone())
            .unwrap_or_default();
        let display_option_indices: Vec<usize> = options
            .iter()
            .enumerate()
            .filter_map(|(idx, option)| (!Self::is_encoder_layout_option(option)).then_some(idx))
            .collect();

        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(18.0);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::DisplayPresetsTitle))
                        .size(18.0)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(crate::i18n::tr(
                        lang,
                        crate::i18n::Key::DisplayPresetsDescription,
                    ))
                    .size(13.0)
                    .color(app_muted_text(dark)),
                );
                ui.add_space(24.0);

                if display_option_indices.is_empty() {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::DisplayPresetsUnavailable),
                        None,
                    );
                    return;
                }

                if !hid_ready || self.layout_options_value.is_none() {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::DisplayPresetsConnect),
                        None,
                    );
                    return;
                }

                let total_rows = display_option_indices.len();
                let list = allocate_adaptive_settings_list_viewport(
                    ui,
                    "layout_options",
                    metrics,
                    total_rows,
                    0.0,
                );
                let values = Self::unpack_layout_option_values(
                    &options,
                    self.layout_options_value.unwrap_or(0),
                );
                ui.allocate_ui_at_rect(list.content_rect, |ui| {
                    ui.set_clip_rect(list.viewport);
                    ui.set_min_size(list.content_rect.size());
                    ui.spacing_mut().item_spacing.y = 0.0;
                    for display_row_idx in list.first_visible_row..list.last_visible_row {
                        let Some(&row_idx) = display_option_indices.get(display_row_idx) else {
                            continue;
                        };
                        let option = &options[row_idx];
                        let translated_option_label =
                            crate::i18n::tr_text(self.app_settings.language, &option.label);
                        if option.choices.is_empty() {
                            let mut enabled = values.get(row_idx).copied().unwrap_or(0) != 0;
                            crate::ui_style::settings_list_row_with_tooltip(
                                ui,
                                row_content_width,
                                row_height,
                                translated_option_label.as_str(),
                                true,
                                Some(crate::i18n::tr_catalog(
                                    self.app_settings.language,
                                    "auto_shift_settings.toggle_firmware_layout_display_option",
                                )),
                                switch_width,
                                |ui| {
                                    let resp = crate::ui_style::settings_switch_sized_stable(
                                        ui,
                                        ("layout_options", row_idx),
                                        &mut enabled,
                                        switch_size,
                                    );
                                    if resp.changed() {
                                        self.set_layout_option_value(row_idx, u32::from(enabled));
                                    }
                                },
                            );
                        } else {
                            let selected_idx = values
                                .get(row_idx)
                                .copied()
                                .unwrap_or(0)
                                .min(option.choices.len().saturating_sub(1) as u32)
                                as usize;
                            let selected_raw_text = option
                                .choices
                                .get(selected_idx)
                                .map(|s| s.as_str())
                                .unwrap_or("Unknown");
                            let selected_text = Self::display_preset_choice_label(
                                self.app_settings.language,
                                selected_raw_text,
                            );
                            let translated_label = translated_option_label.as_str();
                            let tooltip = if matches!(
                                self.app_settings.language,
                                crate::i18n::Language::Russian
                            ) {
                                format!("Выбрать пресет прошивки для {translated_label}")
                            } else {
                                format!("Choose firmware preset for {}", option.label)
                            };
                            crate::ui_style::settings_list_row_with_tooltip(
                                ui,
                                row_content_width,
                                row_height,
                                translated_label,
                                true,
                                Some(&tooltip),
                                dropdown_width,
                                |ui| {
                                    let dropdown_id =
                                        ui.make_persistent_id(("layout_option_dropdown", row_idx));
                                    let dropdown_resp =
                                        crate::ui_style::modern_dropdown_button_sized(
                                            ui,
                                            dropdown_id,
                                            &selected_text,
                                            ui.visuals().text_color(),
                                            dropdown_width,
                                            metrics.settings_control_height(),
                                            metrics.settings_control_font_size(),
                                        );

                                    egui::popup_below_widget(
                                        ui,
                                        dropdown_id,
                                        &dropdown_resp,
                                        egui::PopupCloseBehavior::CloseOnClickOutside,
                                        |ui| {
                                            ui.set_min_width(dropdown_width);
                                            ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
                                            egui::ScrollArea::vertical()
                                                .id_salt(("layout_option_dropdown_scroll", row_idx))
                                                .max_height(metrics.value(142.0))
                                                .auto_shrink([false, true])
                                                .show(ui, |ui| {
                                                    for (choice_idx, label) in
                                                        option.choices.iter().enumerate()
                                                    {
                                                        let selected = choice_idx == selected_idx;
                                                        let (option_rect, option_resp) = ui
                                                            .allocate_exact_size(
                                                                metrics.size(220.0, 28.0),
                                                                Sense::click(),
                                                            );
                                                        if option_resp.hovered() {
                                                            ui.ctx().set_cursor_icon(
                                                                egui::CursorIcon::PointingHand,
                                                            );
                                                        }
                                                        let option_fill = if selected {
                                                            if dark {
                                                                Color32::from_rgb(58, 58, 61)
                                                            } else {
                                                                Color32::from_rgb(236, 236, 238)
                                                            }
                                                        } else if option_resp.hovered() {
                                                            crate::ui_style::hover_fill(dark)
                                                        } else {
                                                            Color32::TRANSPARENT
                                                        };
                                                        ui.painter().rect_filled(
                                                            option_rect,
                                                            7.0,
                                                            option_fill,
                                                        );
                                                        let display_label =
                                                            Self::display_preset_choice_label(
                                                                self.app_settings.language,
                                                                label,
                                                            );
                                                        ui.painter().text(
                                                            egui::pos2(
                                                                option_rect.left()
                                                                    + metrics.value(10.0),
                                                                option_rect.center().y,
                                                            ),
                                                            egui::Align2::LEFT_CENTER,
                                                            display_label,
                                                            FontId::proportional(
                                                                metrics.value(12.0),
                                                            ),
                                                            if selected {
                                                                ui.visuals().text_color()
                                                            } else {
                                                                app_muted_text(dark)
                                                            },
                                                        );
                                                        if option_resp.clicked() {
                                                            self.set_layout_option_value(
                                                                row_idx,
                                                                choice_idx as u32,
                                                            );
                                                            ui.memory_mut(|m| m.close_popup());
                                                        }
                                                    }
                                                });
                                        },
                                    );
                                },
                            );
                        }
                    }
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
            });
        });
    }

    fn draw_auto_shift_settings_page(
        &mut self,
        ui: &mut egui::Ui,
        content_rect: egui::Rect,
        dark: bool,
    ) {
        let lang = self.app_settings.language;
        let hid_ready = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.hid_device.is_some()
            }
            #[cfg(target_arch = "wasm32")]
            {
                false
            }
        };

        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(18.0);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::AutoShiftTitle))
                        .size(18.0)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(crate::i18n::tr(
                        lang,
                        crate::i18n::Key::AutoShiftDescription,
                    ))
                    .size(13.0)
                    .color(app_muted_text(dark)),
                );
                ui.add_space(24.0);

                if self.auto_shift_timeout.is_none() {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::AutoShiftUnavailable),
                        Some(crate::i18n::tr(lang, crate::i18n::Key::AutoShiftEnableHint)),
                    );
                    return;
                }

                if !hid_ready {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr_catalog(self.app_settings.language, "auto_shift_settings.connect_a_vial_keyboard_to_edit_auto_shift_settings"),
                        None,
                    );
                    return;
                }

                if self.is_vial_locked() {
                    crate::ui_style::modal_centered_text_block(ui, 360.0, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(crate::i18n::tr(
                                    lang,
                                    crate::i18n::Key::KeyboardLocked,
                                ))
                                .size(14.0),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(crate::i18n::tr(
                                    lang,
                                    crate::i18n::Key::AutoShiftUnlockHint,
                                ))
                                .size(12.5)
                                .color(app_muted_text(dark)),
                            );
                        });
                    });
                    ui.add_space(18.0);
                }

                let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
                self.draw_auto_shift_editor_content(ui, dark, metrics);
            });
        });
    }

    fn draw_auto_shift_editor_content(
        &mut self,
        ui: &mut egui::Ui,
        dark: bool,
        metrics: crate::ui_style::ResponsiveMetrics,
    ) {
        const TOTAL_ROWS: usize = 8;
        let field_width = metrics.value(86.0);
        let list = allocate_adaptive_settings_list_viewport(
            ui,
            "auto_shift_settings",
            metrics,
            TOTAL_ROWS,
            0.0,
        );

        ui.allocate_ui_at_rect(list.content_rect, |ui| {
            ui.set_clip_rect(list.viewport);
            ui.set_min_size(list.content_rect.size());
            ui.spacing_mut().item_spacing.y = 0.0;
            for row_idx in list.first_visible_row..list.last_visible_row {
                self.draw_auto_shift_row(
                    ui,
                    row_idx,
                    list.row_content_width,
                    list.row_height,
                    field_width,
                    dark,
                    list.suppress_tooltips,
                );
            }
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
    }

    fn draw_auto_shift_row(
        &mut self,
        ui: &mut egui::Ui,
        row_idx: usize,
        content_width: f32,
        row_height: f32,
        field_width: f32,
        _dark: bool,
        suppress_tooltips: bool,
    ) {
        let row = match row_idx {
            0 => (
                crate::i18n::tr_catalog(self.app_settings.language, "common.enable"),
                crate::i18n::tr_catalog(self.app_settings.language, "auto_shift.enable_tooltip"),
                true,
            ),
            1 => (
                crate::i18n::tr_catalog(
                    self.app_settings.language,
                    "auto_shift.enable_for_modifiers",
                ),
                crate::i18n::tr_catalog(
                    self.app_settings.language,
                    "auto_shift.enable_for_modifiers_tooltip",
                ),
                true,
            ),
            2 => (
                crate::i18n::tr_catalog(self.app_settings.language, "auto_shift.no_special_keys"),
                crate::i18n::tr_catalog(
                    self.app_settings.language,
                    "auto_shift.no_special_keys_tooltip",
                ),
                true,
            ),
            3 => (
                crate::i18n::tr_catalog(self.app_settings.language, "auto_shift.no_numeric_keys"),
                crate::i18n::tr_catalog(
                    self.app_settings.language,
                    "auto_shift.no_numeric_keys_tooltip",
                ),
                true,
            ),
            4 => (
                crate::i18n::tr_catalog(self.app_settings.language, "auto_shift.no_alpha_keys"),
                crate::i18n::tr_catalog(
                    self.app_settings.language,
                    "auto_shift.no_alpha_keys_tooltip",
                ),
                true,
            ),
            5 => (
                crate::i18n::tr_catalog(self.app_settings.language, "auto_shift.enable_keyrepeat"),
                crate::i18n::tr_catalog(
                    self.app_settings.language,
                    "auto_shift.enable_keyrepeat_tooltip",
                ),
                true,
            ),
            6 => (
                crate::i18n::tr_catalog(
                    self.app_settings.language,
                    "auto_shift.stop_repeat_after_timeout",
                ),
                crate::i18n::tr_catalog(
                    self.app_settings.language,
                    "auto_shift.stop_repeat_after_timeout_tooltip",
                ),
                true,
            ),
            7 => (
                crate::i18n::tr_catalog(self.app_settings.language, "common.timeout"),
                crate::i18n::tr_catalog(self.app_settings.language, "auto_shift.timeout_tooltip"),
                false,
            ),
            _ => return,
        };
        let enabled = row_idx == 0 || self.auto_shift_options.enabled;
        let tooltip = if suppress_tooltips { None } else { Some(row.1) };
        if row.2 {
            let mut value = match row_idx {
                0 => self.auto_shift_options.enabled,
                1 => self.auto_shift_options.enable_for_modifiers,
                2 => self.auto_shift_options.no_special,
                3 => self.auto_shift_options.no_numeric,
                4 => self.auto_shift_options.no_alpha,
                5 => self.auto_shift_options.enable_keyrepeat,
                6 => self.auto_shift_options.disable_keyrepeat_timeout,
                _ => false,
            };
            crate::ui_style::settings_list_row_with_tooltip(
                ui,
                content_width,
                row_height,
                row.0,
                enabled,
                tooltip,
                46.0,
                |ui| {
                    let resp = crate::ui_style::settings_switch_sized_stable_interactive(
                        ui,
                        ("auto_shift_settings", row_idx),
                        &mut value,
                        egui::vec2(46.0, 24.0),
                        enabled,
                    );
                    if resp.changed() {
                        match row_idx {
                            0 => self.auto_shift_options.enabled = value,
                            1 => self.auto_shift_options.enable_for_modifiers = value,
                            2 => self.auto_shift_options.no_special = value,
                            3 => self.auto_shift_options.no_numeric = value,
                            4 => self.auto_shift_options.no_alpha = value,
                            5 => self.auto_shift_options.enable_keyrepeat = value,
                            6 => self.auto_shift_options.disable_keyrepeat_timeout = value,
                            _ => {}
                        }
                        self.write_auto_shift_flags();
                    }
                },
            );
        } else {
            let timeout_value = self.auto_shift_timeout.unwrap_or(175);
            if self.auto_shift_timeout_text.is_empty() {
                self.auto_shift_timeout_text = timeout_value.to_string();
            }
            crate::ui_style::settings_list_row_with_tooltip(
                ui,
                content_width,
                row_height,
                row.0,
                enabled,
                tooltip,
                field_width,
                |ui| {
                    let edit_id = egui::Id::new("auto_shift_timeout");
                    let resp = crate::ui_style::modern_text_field_interactive(
                        ui,
                        edit_id,
                        &mut self.auto_shift_timeout_text,
                        field_width,
                        "",
                        5,
                        egui::Align::RIGHT,
                        enabled,
                    );
                    let resp = settings_field_unit_tooltip(
                        resp,
                        self.app_settings.language,
                        suppress_tooltips,
                        SettingsFieldUnit::Milliseconds,
                    );
                    if resp.changed() {
                        let filtered: String = self
                            .auto_shift_timeout_text
                            .chars()
                            .filter(|c: &char| c.is_ascii_digit())
                            .collect();
                        if filtered != self.auto_shift_timeout_text {
                            self.auto_shift_timeout_text = filtered.clone();
                        }
                        if let Ok(parsed) = filtered.parse::<u16>() {
                            let timeout_value = parsed.max(1);
                            self.auto_shift_timeout = Some(timeout_value);
                            self.auto_shift_timeout_text = timeout_value.to_string();
                            self.write_auto_shift_timeout();
                        }
                    }
                },
            );
        }
    }

    fn apply_picker_results(&mut self) {
        if let Some(kc_value) = self.keycode_picker.result.take() {
            if let Some((combo_idx, field)) = self.combo_pick_target.take() {
                self.push_combo_undo();
                if let Some(combo) = self.combo_entries.get_mut(combo_idx) {
                    match field {
                        ComboPickField::Trigger(key_idx) => combo.keys[key_idx] = kc_value,
                        ComboPickField::Output => combo.output = kc_value,
                    }
                    self.combo_dirty = true;
                }
            } else if let Some(field) = self.key_override_pick_target.take() {
                let idx = self
                    .selected_key_override
                    .min(self.key_override_entries.len().saturating_sub(1));
                self.push_key_override_undo();
                if let Some(entry) = self.key_override_entries.get_mut(idx) {
                    match field {
                        KeyOverridePickField::Trigger => entry.trigger = kc_value,
                        KeyOverridePickField::Replacement => entry.replacement = kc_value,
                    }
                    Self::normalize_key_override_entry(entry);
                }
                self.write_key_override(idx);
            } else if let Some(field) = self.alt_repeat_pick_target.take() {
                let idx = self
                    .selected_alt_repeat
                    .min(self.alt_repeat_entries.len().saturating_sub(1));
                if let Some(entry) = self.alt_repeat_entries.get_mut(idx) {
                    match field {
                        AltRepeatPickField::LastKey => entry.keycode = kc_value,
                        AltRepeatPickField::AltKey => entry.alt_keycode = kc_value,
                    }
                }
                self.write_alt_repeat_entry(idx);
            } else if let Some((layer, encoder_visual_idx)) = self.selected_encoder {
                #[cfg(not(target_arch = "wasm32"))]
                self.assign_encoder_keycode(layer, encoder_visual_idx, kc_value);
                #[cfg(target_arch = "wasm32")]
                if let Some(layout) = &mut self.layout {
                    if let Some(layer_codes) = layout.encoder_layers.get_mut(layer) {
                        if let Some(slot) = layer_codes.get_mut(encoder_visual_idx) {
                            *slot = kc_value;
                        }
                    }
                }
                if is_alt_repeat_keycode(kc_value) {
                    self.open_alt_repeat_window_compact();
                }
            } else if let Some((layer, ki)) = self.selected_key {
                #[cfg(not(target_arch = "wasm32"))]
                self.assign_keycode(layer, ki, kc_value);
                #[cfg(target_arch = "wasm32")]
                if let Some(layout) = &mut self.layout {
                    layout.set_keycode(layer, ki, kc_value);
                }
                if is_alt_repeat_keycode(kc_value) {
                    self.open_alt_repeat_window_compact();
                }
            }
            self.selected_key = None;
            self.selected_encoder = None;
        }
    }

    fn assign_encoder_keycode(&mut self, layer: usize, encoder_visual_idx: usize, kc_value: u16) {
        let encoder = match self
            .layout
            .as_ref()
            .and_then(|l| l.encoders.get(encoder_visual_idx))
        {
            Some(e) => e.clone(),
            None => return,
        };
        let old_kc = self
            .layout
            .as_ref()
            .map(|l| l.get_encoder_keycode(layer, encoder_visual_idx))
            .unwrap_or(0);
        self.undo_stack.push(UndoAction::Encoder {
            layer,
            encoder_visual_idx,
            old_kc,
        });

        if let Some(layout) = &mut self.layout {
            if let Some(layer_codes) = layout.encoder_layers.get_mut(layer) {
                if let Some(slot) = layer_codes.get_mut(encoder_visual_idx) {
                    *slot = kc_value;
                }
            }
        }

        let result = if let Some(conn) = &self.hid_device {
            conn.set_encoder(
                layer as u8,
                encoder.encoder_idx,
                encoder.direction,
                kc_value,
            )
        } else if let Some(dev) = self
            .selected_device
            .and_then(|i| self.device_manager.devices().get(i))
        {
            match crate::hid::HidDevice::open(&dev.path) {
                Ok(conn) => conn.set_encoder(
                    layer as u8,
                    encoder.encoder_idx,
                    encoder.direction,
                    kc_value,
                ),
                Err(e) => Err(anyhow::anyhow!("{e}")),
            }
        } else {
            return;
        };

        match result {
            Ok(()) => {
                self.status_msg = format!(
                    "Assigned encoder {} direction {} on layer {}",
                    encoder.encoder_idx,
                    encoder.direction,
                    layer + 1
                );
            }
            Err(e) => {
                self.status_msg = format!("Set encoder failed: {e}");
            }
        }
    }

    fn open_picker_for_target(&mut self, key_target: Option<usize>, encoder_target: Option<usize>) {
        self.selected_key = key_target.map(|ki| (self.selected_layer, ki));
        self.selected_encoder = encoder_target.map(|ei| (self.selected_layer, ei));
        self.keycode_picker.open = true;
        self.keycode_picker.result = None;
        self.keycode_picker.search_query.clear();
        self.keycode_picker.layer_names = self.layer_names.clone();
        self.keycode_picker.vial_quantum_pending_mod = None;
        self.keycode_picker.vial_quantum_pending_mt = None;
        self.keycode_picker.vial_layer_pending = None;
        self.keycode_picker.tap_dance_editor_open = None;
        self.keycode_picker.selected_tab = crate::keycode_picker::KeycodeTab::Basic;
    }

    fn handle_secondary_target(
        &mut self,
        ctrl_held: bool,
        kc: u16,
        key_target: Option<usize>,
        encoder_target: Option<usize>,
    ) {
        if !ctrl_held {
            if let Some(target_layer) = vial_layer_target(kc) {
                if target_layer != self.selected_layer {
                    self.jump_back_stack.push(self.selected_layer);
                    self.selected_layer = target_layer;
                    self.hover_layer = None;
                }
                self.secondary_click_handled = true;
                return;
            }
        }
        if ctrl_held {
            if let Some(swapped) = toggle_handed_modifier(kc) {
                if let Some(visual_idx) = encoder_target {
                    self.assign_encoder_keycode(self.selected_layer, visual_idx, swapped);
                } else if let Some(ki) = key_target {
                    self.pending_handed_swap = Some((self.selected_layer, ki, swapped));
                }
                self.secondary_click_handled = true;
            } else {
                if let Some(base) = vial_layer_retarget_base(kc) {
                    self.open_picker_for_target(key_target, encoder_target);
                    self.keycode_picker.vial_layer_pending = Some(base);
                    self.secondary_click_handled = true;
                }
            }
            if self.secondary_click_handled {
                return;
            }
        }
        if kc >= 0x7700 && kc <= 0x77FF {
            let macro_n = (kc - 0x7700) as u8;
            self.open_picker_for_target(key_target, encoder_target);
            self.keycode_picker.selected_tab = crate::keycode_picker::KeycodeTab::Macro;
            self.keycode_picker.macro_inline_selected = Some(macro_n);
            self.secondary_click_handled = true;
            return;
        }
        if kc >= 0x5700 && kc <= 0x57FF {
            let td_n = (kc - 0x5700) as u8;
            self.open_picker_for_target(key_target, encoder_target);
            self.keycode_picker.selected_tab = crate::keycode_picker::KeycodeTab::TapDance;
            self.keycode_picker.tap_dance_editor_open = Some(td_n);
            self.secondary_click_handled = true;
            return;
        }
        if is_mouse_keycode(kc) {
            self.open_mouse_keys_settings_page();
            self.secondary_click_handled = true;
            return;
        }
        if is_alt_repeat_keycode(kc) {
            self.open_alt_repeat_window_compact();
            self.secondary_click_handled = true;
            return;
        }
        let is_layer_key = vial_layer_target(kc).is_some();
        let pending_base: Option<u16> = if is_layer_key {
            None
        } else if kc >= 0x2000 && kc < 0x4000 {
            Some(kc & 0xFF00)
        } else if kc >= 0x0100 && kc < 0x2000 && (kc & 0xFF) != 0 {
            Some(kc & 0xFF00)
        } else {
            None
        };
        if let Some(base) = pending_base {
            self.open_picker_for_target(key_target, encoder_target);
            if kc >= 0x2000 {
                self.keycode_picker.vial_quantum_pending_mt = Some(base);
                self.keycode_picker.vial_quantum_pending_mod = None;
            } else {
                self.keycode_picker.vial_quantum_pending_mod = Some(base);
                self.keycode_picker.vial_quantum_pending_mt = None;
            }
            self.secondary_click_handled = true;
        }
    }

    fn assign_keycode(&mut self, layer: usize, ki: usize, kc_value: u16) {
        // Save old value for undo
        let old_kc = self
            .layout
            .as_ref()
            .map(|l| l.get_keycode(layer, ki))
            .unwrap_or(0);
        self.undo_stack.push(UndoAction::Key {
            layer,
            key_idx: ki,
            old_kc,
        });
        // Update in-memory layout
        if let Some(layout) = &mut self.layout {
            layout.set_keycode(layer, ki, kc_value);
        }
        self.refresh_layer_picker_content_flags();

        let key = match self.layout.as_ref().and_then(|l| l.keys.get(ki)) {
            Some(k) => k.clone(),
            None => return,
        };

        // Use persistent connection if available, otherwise open fresh
        let result = if let Some(conn) = &self.hid_device {
            conn.set_keycode(layer as u8, key.row, key.col, kc_value)
        } else if let Some(dev) = self
            .selected_device
            .and_then(|i| self.device_manager.devices().get(i))
        {
            match crate::hid::HidDevice::open(&dev.path) {
                Ok(conn) => conn.set_keycode(layer as u8, key.row, key.col, kc_value),
                Err(e) => Err(anyhow::anyhow!("{e}")),
            }
        } else {
            return;
        };

        match result {
            Ok(()) => self.status_msg = "✓ Saved".into(),
            Err(e) => {
                self.status_msg = format!("Write error: {e}");
                // Connection lost — reopen
                self.hid_device = None;
            }
        }
    }

    /// Reload all keycodes from device in background.
    #[cfg(not(target_arch = "wasm32"))]
    fn load_from_device(&mut self) {
        if let Some(idx) = self.selected_device {
            self.start_connect(idx);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn undo(&mut self) {
        let Some(action) = self.undo_stack.pop() else {
            return;
        };
        match action {
            UndoAction::Key {
                layer,
                key_idx,
                old_kc,
            } => {
                self.assign_keycode(layer, key_idx, old_kc);
                self.undo_stack.pop();
            }
            UndoAction::Encoder {
                layer,
                encoder_visual_idx,
                old_kc,
            } => {
                self.assign_encoder_keycode(layer, encoder_visual_idx, old_kc);
                self.undo_stack.pop();
            }
        }
    }

    fn ui_scale_percent(&self) -> i32 {
        (clamp_ui_scale(self.app_settings.ui_scale) * 100.0).round() as i32
    }

    fn apply_ui_scale(&mut self, ctx: &egui::Context) {
        let scale = clamp_ui_scale(self.app_settings.ui_scale);
        if (scale - self.app_settings.ui_scale).abs() > f32::EPSILON {
            self.app_settings.ui_scale = scale;
            save_app_settings(&self.app_settings);
        }
        if (ctx.zoom_factor() - scale).abs() > 0.001 {
            ctx.set_zoom_factor(scale);
        }
    }

    fn set_ui_scale(&mut self, ctx: &egui::Context, scale: f32) {
        let scale = clamp_ui_scale(scale);
        if (scale - self.app_settings.ui_scale).abs() <= 0.001 {
            return;
        }
        self.app_settings.ui_scale = scale;
        save_app_settings(&self.app_settings);
        ctx.set_zoom_factor(scale);
        ctx.request_repaint();
    }

    fn step_ui_scale(&mut self, ctx: &egui::Context, steps: i32) {
        self.set_ui_scale(
            ctx,
            self.app_settings.ui_scale + UI_SCALE_STEP * steps as f32,
        );
    }

    fn handle_ui_scale_shortcuts(&mut self, ctx: &egui::Context) {
        let action = ctx.input(|i| {
            if !i.modifiers.ctrl {
                return None;
            }
            if i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals) {
                Some(1)
            } else if i.key_pressed(egui::Key::Minus) {
                Some(-1)
            } else if i.key_pressed(egui::Key::Num0) {
                Some(0)
            } else {
                None
            }
        });

        match action {
            Some(1) => self.step_ui_scale(ctx, 1),
            Some(-1) => self.step_ui_scale(ctx, -1),
            Some(0) => self.set_ui_scale(ctx, default_ui_scale()),
            _ => {}
        }
    }

    fn start_onboarding_tour(&mut self, ctx: &egui::Context) {
        self.close_top_dropdowns(ctx);
        self.keycode_picker.open = false;
        if self.layout.is_some() {
            self.main_menu_tab = MainMenuTab::Keyboard;
        }
        self.tour_state.active = true;
        self.tour_state.step = 0;
        ctx.request_repaint();
    }

    fn complete_onboarding_tour(&mut self) {
        self.tour_state.active = false;
        self.tour_state.step = 0;
        self.app_settings.onboarding_tour_seen_version = ONBOARDING_TOUR_VERSION;
        save_app_settings(&self.app_settings);
    }

    fn maybe_start_onboarding_tour(&mut self, ctx: &egui::Context) {
        if self.tour_state.active
            || self.app_settings.onboarding_tour_seen_version >= ONBOARDING_TOUR_VERSION
            || self.layout.is_none()
            || self.unlock_open
            || self.vial_unlock_polling
            || self.keycode_picker.open
        {
            return;
        }
        self.start_onboarding_tour(ctx);
    }

    fn register_tour_target(&mut self, target: TourTarget, rect: egui::Rect) {
        if rect.is_positive() {
            self.tour_target_rects.retain(|(t, _)| *t != target);
            self.tour_target_rects.push((target, rect));
        }
    }

    fn tour_target_rect(&self, target: TourTarget) -> Option<egui::Rect> {
        self.tour_target_rects
            .iter()
            .find_map(|(registered, rect)| (*registered == target).then_some(*rect))
    }

    fn draw_onboarding_tour(&mut self, ctx: &egui::Context) {
        if !self.tour_state.active || self.unlock_open || self.vial_unlock_polling {
            return;
        }
        if self.keycode_picker.open {
            return;
        }

        let step_count = ONBOARDING_TOUR_STEPS.len();
        if self.tour_state.step >= step_count {
            self.complete_onboarding_tour();
            return;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.complete_onboarding_tour();
            return;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::Enter)) {
            if self.tour_state.step + 1 >= step_count {
                self.complete_onboarding_tour();
            } else {
                self.tour_state.step += 1;
            }
            ctx.request_repaint();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) && self.tour_state.step > 0 {
            self.tour_state.step -= 1;
            ctx.request_repaint();
        }

        let step = ONBOARDING_TOUR_STEPS[self.tour_state.step];
        let target_rect = step.target.and_then(|target| self.tour_target_rect(target));
        let lang = self.app_settings.language;
        let title = crate::i18n::tr_catalog(lang, step.title_key).to_owned();
        let body = crate::i18n::tr_catalog(lang, step.body_key).to_owned();
        let progress = crate::i18n::tr_catalog_format(
            lang,
            "onboarding_tour.progress",
            &[
                ("current", &(self.tour_state.step + 1).to_string()),
                ("total", &step_count.to_string()),
            ],
        );
        let prev_label = crate::i18n::tr_catalog(lang, "onboarding_tour.previous").to_owned();
        let next_label = if self.tour_state.step + 1 >= step_count {
            crate::i18n::tr_catalog(lang, "onboarding_tour.done").to_owned()
        } else {
            crate::i18n::tr_catalog(lang, "onboarding_tour.next").to_owned()
        };
        let skip_label = crate::i18n::tr_catalog(lang, "onboarding_tour.skip").to_owned();
        let sample_hint = crate::i18n::tr_catalog(lang, "onboarding_tour.sample_hint").to_owned();

        egui::Area::new(egui::Id::new("onboarding_tour_overlay"))
            .order(egui::Order::Tooltip)
            .fixed_pos(ctx.screen_rect().min)
            .show(ctx, |ui| {
                let screen = ctx.screen_rect();
                let local_screen = egui::Rect::from_min_size(egui::Pos2::ZERO, screen.size());
                ui.set_min_size(screen.size());
                let _block = ui.interact(
                    local_screen,
                    ui.make_persistent_id("onboarding_tour_blocker"),
                    Sense::click(),
                );
                let dark = ui.visuals().dark_mode;
                let painter = ui.painter();
                painter.rect_filled(
                    local_screen,
                    0.0,
                    Color32::from_black_alpha(if dark { 176 } else { 128 }),
                );

                let local_target = target_rect.map(|rect| rect.translate(-screen.min.to_vec2()));
                if let Some(rect) = local_target {
                    let highlight = rect.expand(8.0);
                    painter.rect(
                        highlight,
                        15.0,
                        Color32::from_rgba_unmultiplied(255, 255, 255, 22),
                        Stroke::new(2.0, app_accent()),
                        egui::StrokeKind::Inside,
                    );
                    if matches!(step.target, Some(TourTarget::BottomHints)) {
                        painter.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            sample_hint.as_str(),
                            FontId::proportional(12.0),
                            Color32::from_rgb(245, 245, 245),
                        );
                    }
                }

                let card_w = 382.0_f32.min(local_screen.width() - 32.0).max(280.0);
                let card_h = 244.0_f32;
                let margin = 18.0;
                let card_x = local_target
                    .map(|rect| rect.center().x - card_w / 2.0)
                    .unwrap_or_else(|| local_screen.center().x - card_w / 2.0)
                    .clamp(
                        local_screen.left() + margin,
                        local_screen.right() - card_w - margin,
                    );
                let card_y = if let Some(rect) = local_target {
                    let below = rect.bottom() + 18.0;
                    let above = rect.top() - card_h - 18.0;
                    if below + card_h <= local_screen.bottom() - margin {
                        below
                    } else if above >= local_screen.top() + margin {
                        above
                    } else {
                        local_screen.center().y - card_h / 2.0
                    }
                } else {
                    local_screen.center().y - card_h / 2.0
                };
                let card_rect = egui::Rect::from_min_size(
                    egui::pos2(card_x, card_y),
                    Vec2::new(card_w, card_h),
                );

                painter.rect(
                    card_rect,
                    18.0,
                    app_window_fill(dark),
                    Stroke::new(1.0, app_border_color(dark)),
                    egui::StrokeKind::Inside,
                );
                ui.allocate_ui_at_rect(card_rect.shrink2(Vec2::new(18.0, 16.0)), |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(progress.as_str())
                                    .size(11.5)
                                    .color(app_muted_text(dark)),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let close = ui.add(
                                        egui::Label::new(
                                            RichText::new("×")
                                                .size(18.0)
                                                .color(app_muted_text(dark)),
                                        )
                                        .selectable(false)
                                        .sense(Sense::click()),
                                    );
                                    if close.hovered() {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                    }
                                    if close.clicked() {
                                        self.complete_onboarding_tour();
                                    }
                                },
                            );
                        });
                        ui.add_space(10.0);
                        ui.label(RichText::new(title.as_str()).size(20.0).strong());
                        ui.add_space(8.0);
                        ui.add(
                            egui::Label::new(RichText::new(body.as_str()).size(13.5).color(
                                if dark {
                                    Color32::from_gray(205)
                                } else {
                                    Color32::from_gray(66)
                                },
                            ))
                            .wrap(),
                        );
                        ui.add_space(10.0);
                        let button_row_height = 32.0;
                        let spacer_height = (ui.available_height() - button_row_height).max(0.0);
                        ui.add_space(spacer_height);
                        ui.horizontal(|ui| {
                            if crate::ui_style::modern_button(
                                ui,
                                skip_label.as_str(),
                                Vec2::new(96.0, button_row_height),
                                true,
                            )
                            .clicked()
                            {
                                self.complete_onboarding_tour();
                            }

                            let trailing_width = 88.0 + 94.0 + ui.spacing().item_spacing.x;
                            ui.add_space((ui.available_width() - trailing_width).max(0.0));

                            let prev_enabled = self.tour_state.step > 0;
                            if crate::ui_style::modern_button(
                                ui,
                                prev_label.as_str(),
                                Vec2::new(88.0, button_row_height),
                                prev_enabled,
                            )
                            .clicked()
                                && prev_enabled
                            {
                                self.tour_state.step -= 1;
                                ctx.request_repaint();
                            }
                            if crate::ui_style::modern_button(
                                ui,
                                next_label.as_str(),
                                Vec2::new(94.0, button_row_height),
                                true,
                            )
                            .clicked()
                            {
                                if self.tour_state.step + 1 >= step_count {
                                    self.complete_onboarding_tour();
                                } else {
                                    self.tour_state.step += 1;
                                    ctx.request_repaint();
                                }
                            }
                        });
                    });
                });
            });
    }

    fn sync_sticky_layout_layer_state(&mut self, layout: &KeyboardLayout) -> usize {
        let layer_count = layout.layers.len().max(1);
        let pressed = self.matrix_tester_pressed.clone();

        if self.sticky_layout_prev_pressed.len() != pressed.len() {
            self.sticky_layout_prev_pressed = vec![false; pressed.len()];
        }
        if self.sticky_layout_pressed_key_layers.len() != pressed.len() {
            self.sticky_layout_pressed_key_layers = vec![None; pressed.len()];
        }
        if self.sticky_layout_toggled_layers.len() != layer_count {
            self.sticky_layout_toggled_layers = vec![false; layer_count];
        }
        self.sticky_layout_base_layer = self.sticky_layout_base_layer.min(layer_count - 1);

        for (key_idx, key) in layout.keys.iter().enumerate() {
            let matrix_idx = key.row as usize * layout.cols + key.col as usize;
            let is_pressed = pressed.get(matrix_idx).copied().unwrap_or(false);
            let was_pressed = self
                .sticky_layout_prev_pressed
                .get(matrix_idx)
                .copied()
                .unwrap_or(false);
            if !is_pressed {
                if let Some(source_layer) =
                    self.sticky_layout_pressed_key_layers.get_mut(matrix_idx)
                {
                    *source_layer = None;
                }
                continue;
            }
            if was_pressed {
                continue;
            }

            let layer_before = sticky_layout_active_layer(
                layout,
                &self.sticky_layout_prev_pressed,
                &self.sticky_layout_toggled_layers,
                self.sticky_layout_base_layer,
            );
            let kc = layout_effective_keycode(layout, layer_before, key_idx);
            if sticky_momentary_layer_target(kc).is_some()
                || sticky_toggle_layer_target(kc).is_some()
                || sticky_base_layer_target(kc).is_some()
            {
                if let Some(source_layer) =
                    self.sticky_layout_pressed_key_layers.get_mut(matrix_idx)
                {
                    *source_layer = Some(layer_before);
                }
            }
            if let Some(target) =
                sticky_toggle_layer_target(kc).filter(|target| *target < layer_count)
            {
                if let Some(enabled) = self.sticky_layout_toggled_layers.get_mut(target) {
                    *enabled = !*enabled;
                }
            } else if let Some(target) =
                sticky_base_layer_target(kc).filter(|target| *target < layer_count)
            {
                self.sticky_layout_base_layer = target;
                self.sticky_layout_toggled_layers.fill(false);
            }
        }

        self.sticky_layout_prev_pressed = pressed;
        sticky_layout_active_layer(
            layout,
            &self.matrix_tester_pressed,
            &self.sticky_layout_toggled_layers,
            self.sticky_layout_base_layer,
        )
    }

    fn draw_sticky_layout_window(&mut self, ctx: &egui::Context) {
        if !self.app_settings.sticky_layout_window {
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        self.prompt_if_vial_locked_for_matrix_poll();

        #[cfg(not(target_arch = "wasm32"))]
        if let Some((rows, cols)) = self
            .layout
            .as_ref()
            .map(|layout| (layout.rows, layout.cols))
        {
            self.poll_switch_matrix_state(ctx, rows, cols, false);
        }

        let viewport_id = egui::ViewportId::from_hash_of("entropy_sticky_layout_window");
        let lang = self.app_settings.language;
        let layout = self.layout.clone();
        let selected_device_name = self
            .selected_device
            .and_then(|idx| self.device_manager.devices().get(idx))
            .map(|device| device.name.clone());
        let title = selected_device_name
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_owned)
            .or_else(|| {
                layout
                    .as_ref()
                    .map(|layout| layout.name.trim())
                    .filter(|name| !name.is_empty())
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| {
                crate::i18n::tr_catalog(lang, "ui.sticky_layout_window_title").to_string()
            });
        let sticky_layer = layout
            .as_ref()
            .map(|layout| self.sync_sticky_layout_layer_state(layout))
            .unwrap_or(0);
        let layer_names = self.layer_names.clone();
        let macro_names = self.keycode_picker.macro_names.clone();
        let tap_dance_names = self.keycode_picker.tap_dance_names.clone();
        let key_legend_layout = self.app_settings.key_legend_layout;
        let show_shifted_number_symbols = self.app_settings.show_shifted_number_symbols;
        let encoder_visibility = self.encoder_visibility.clone();
        let matrix_pressed = self.matrix_tester_pressed.clone();
        let pressed_key_layers = self.sticky_layout_pressed_key_layers.clone();
        let ui_scale = clamp_ui_scale(self.app_settings.ui_scale);
        let dark = self.app_settings.sticky_layout_dark_mode;
        let mut sticky_dark_mode = self.app_settings.sticky_layout_dark_mode;
        let mut sticky_opacity =
            clamp_sticky_layout_opacity(self.app_settings.sticky_layout_opacity);
        let mut sticky_always_on_top = self.app_settings.sticky_layout_always_on_top;
        let sticky_window_size = sticky_layout_saved_window_size(&self.app_settings);
        let sticky_previous_size = self.sticky_layout_last_size.unwrap_or(sticky_window_size);
        let mut observed_sticky_size: Option<Vec2> = None;
        let mut should_close = false;
        let mut should_save_settings = false;

        ctx.show_viewport_immediate(
            viewport_id,
            egui::ViewportBuilder::default()
                .with_title(title.clone())
                .with_inner_size(sticky_window_size)
                .with_min_inner_size(sticky_layout_default_window_size())
                .with_resizable(true)
                .with_decorations(false)
                .with_window_level(if sticky_always_on_top {
                    egui::WindowLevel::AlwaysOnTop
                } else {
                    egui::WindowLevel::Normal
                }),
            |viewport_ctx, viewport_class| {
                if viewport_ctx.input(|i| i.viewport().close_requested()) {
                    should_close = true;
                    return;
                }

                if let Some(current_rect) = viewport_ctx.input(|i| i.viewport().inner_rect) {
                    let current_size = current_rect.size();
                    if current_size.x.is_finite()
                        && current_size.y.is_finite()
                        && current_size.x > 0.0
                        && current_size.y > 0.0
                    {
                        let adjusted_size = sticky_layout_aspect_adjusted_window_size(
                            layout.as_ref(),
                            current_size,
                            sticky_previous_size,
                        );
                        if (adjusted_size.x - current_size.x).abs() > 1.5
                            || (adjusted_size.y - current_size.y).abs() > 1.5
                        {
                            viewport_ctx
                                .send_viewport_cmd(egui::ViewportCommand::InnerSize(adjusted_size));
                            observed_sticky_size = Some(adjusted_size);
                        } else {
                            observed_sticky_size = Some(current_size);
                        }
                    }
                }

                let mut draw_contents = |ui: &mut egui::Ui, should_close: &mut bool| {
                    #[cfg(not(target_os = "windows"))]
                    ui.set_opacity(sticky_opacity);
                    #[cfg(target_os = "windows")]
                    set_windows_window_opacity_by_title(&title, sticky_opacity);
                    let panel_bg = app_panel_fill(dark);
                    let full_rect = ui.max_rect();
                    ui.painter().rect_filled(full_rect, 0.0, panel_bg);
                    ui.painter().rect(
                        full_rect.shrink(0.5),
                        0.0,
                        Color32::TRANSPARENT,
                        Stroke::new(1.0, app_border_color(dark)),
                        egui::StrokeKind::Inside,
                    );
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        *should_close = true;
                    }

                    let title_rect = egui::Rect::from_min_max(
                        full_rect.min,
                        egui::pos2(
                            full_rect.right(),
                            full_rect.top() + STICKY_LAYOUT_WINDOW_TITLE_H,
                        ),
                    );
                    let buttons_w = 60.0;
                    let drag_rect = egui::Rect::from_min_max(
                        title_rect.min,
                        egui::pos2(title_rect.right() - buttons_w, title_rect.bottom()),
                    );
                    let drag_response = ui.interact(
                        drag_rect,
                        ui.id().with("sticky_layout_window_drag"),
                        Sense::click_and_drag(),
                    );
                    if drag_response.drag_started() {
                        viewport_ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }

                    ui.painter().text(
                        egui::pos2(title_rect.left() + 12.0, title_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        title.as_str(),
                        FontId::proportional(13.0),
                        app_muted_text(dark),
                    );

                    ui.allocate_ui_at_rect(title_rect.shrink2(Vec2::new(6.0, 4.0)), |ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if sticky_layout_window_icon_button(
                                ui,
                                dark,
                                StickyLayoutWindowButton::Close,
                                false,
                                crate::i18n::tr_catalog(
                                    lang,
                                    "ui.sticky_layout_window_close_tooltip",
                                ),
                            )
                            .clicked()
                            {
                                *should_close = true;
                            }
                            ui.add_space(4.0);
                            if sticky_layout_window_icon_button(
                                ui,
                                dark,
                                StickyLayoutWindowButton::Pin,
                                sticky_always_on_top,
                                crate::i18n::tr_catalog(
                                    lang,
                                    "ui.sticky_layout_window_pin_tooltip",
                                ),
                            )
                            .clicked()
                            {
                                sticky_always_on_top = !sticky_always_on_top;
                                should_save_settings = true;
                                viewport_ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                                    if sticky_always_on_top {
                                        egui::WindowLevel::AlwaysOnTop
                                    } else {
                                        egui::WindowLevel::Normal
                                    },
                                ));
                            }
                        });
                    });

                    let preview_rect = egui::Rect::from_min_max(
                        egui::pos2(full_rect.left(), title_rect.bottom()),
                        full_rect.right_bottom(),
                    );
                    let rect = preview_rect.shrink(STICKY_LAYOUT_WINDOW_MARGIN);
                    if let Some(layout) = &layout {
                        Self::paint_sticky_layout_preview(
                            ui,
                            layout,
                            sticky_layer,
                            &layer_names,
                            &macro_names,
                            &tap_dance_names,
                            key_legend_layout,
                            show_shifted_number_symbols,
                            &encoder_visibility,
                            &matrix_pressed,
                            &pressed_key_layers,
                            ui_scale,
                            dark,
                            rect,
                        );
                    } else {
                        ui.painter().rect(
                            rect,
                            16.0,
                            app_surface_fill(dark),
                            Stroke::new(1.0, app_border_color(dark)),
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            crate::i18n::tr_catalog(lang, "ui.sticky_layout_no_keyboard"),
                            FontId::proportional(14.0),
                            app_muted_text(dark),
                        );
                    }

                    let transparency_rect = egui::Rect::from_min_size(
                        egui::pos2(full_rect.left() + 8.0, full_rect.bottom() - 28.0),
                        egui::vec2(108.0, 24.0),
                    );
                    ui.allocate_ui_at_rect(transparency_rect, |ui| {
                        if draw_sticky_layout_transparency_dropdown(
                            ui,
                            lang,
                            dark,
                            &mut sticky_opacity,
                        ) {
                            should_save_settings = true;
                        }
                    });

                    let theme_rect = egui::Rect::from_min_size(
                        egui::pos2(full_rect.right() - 150.0, full_rect.bottom() - 27.0),
                        egui::vec2(118.0, 22.0),
                    );
                    ui.allocate_ui_at_rect(theme_rect, |ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            draw_theme_selector_labels(ui, lang, &mut sticky_dark_mode);
                        });
                    });

                    let resize_rect = egui::Rect::from_min_size(
                        egui::pos2(full_rect.right() - 24.0, full_rect.bottom() - 24.0),
                        egui::vec2(24.0, 24.0),
                    );
                    let resize_resp = ui.interact(
                        resize_rect,
                        ui.id().with("sticky_layout_resize_grip"),
                        Sense::click_and_drag(),
                    );
                    if resize_resp.hovered() || resize_resp.dragged() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeSouthEast);
                    }
                    if resize_resp.drag_started() {
                        viewport_ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(
                            egui::ResizeDirection::SouthEast,
                        ));
                    }
                    let grip_color = app_muted_text(dark);
                    for offset in [6.0, 11.0, 16.0] {
                        ui.painter().line_segment(
                            [
                                egui::pos2(full_rect.right() - offset, full_rect.bottom() - 4.0),
                                egui::pos2(full_rect.right() - 4.0, full_rect.bottom() - offset),
                            ],
                            Stroke::new(1.0, grip_color),
                        );
                    }
                };

                if matches!(viewport_class, egui::ViewportClass::Embedded) {
                    let mut open = true;
                    egui::Window::new(title.as_str())
                        .open(&mut open)
                        .default_size(sticky_window_size)
                        .min_size(sticky_layout_default_window_size())
                        .resizable(true)
                        .show(viewport_ctx, |ui| {
                            draw_contents(ui, &mut should_close);
                        });
                    if !open {
                        should_close = true;
                    }
                } else {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::NONE.fill(Color32::TRANSPARENT))
                        .show(viewport_ctx, |ui| {
                            draw_contents(ui, &mut should_close);
                        });
                }
            },
        );

        if let Some(size) = observed_sticky_size {
            self.sticky_layout_last_size = Some(size);
            let saved_size = sticky_layout_saved_window_size(&self.app_settings);
            if (saved_size.x - size.x).abs() > 1.0 || (saved_size.y - size.y).abs() > 1.0 {
                self.app_settings.sticky_layout_window_size = Some([size.x, size.y]);
                should_save_settings = true;
            }
        }

        if should_close {
            self.app_settings.sticky_layout_window = false;
            should_save_settings = true;
        }
        if self.app_settings.sticky_layout_dark_mode != sticky_dark_mode {
            self.app_settings.sticky_layout_dark_mode = sticky_dark_mode;
            should_save_settings = true;
        }
        sticky_opacity = clamp_sticky_layout_opacity(sticky_opacity);
        if (self.app_settings.sticky_layout_opacity - sticky_opacity).abs() > f32::EPSILON {
            self.app_settings.sticky_layout_opacity = sticky_opacity;
            should_save_settings = true;
        }
        if self.app_settings.sticky_layout_always_on_top != sticky_always_on_top {
            self.app_settings.sticky_layout_always_on_top = sticky_always_on_top;
            should_save_settings = true;
        }
        if should_save_settings {
            save_app_settings(&self.app_settings);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_sticky_layout_preview(
        ui: &mut egui::Ui,
        layout: &KeyboardLayout,
        layer: usize,
        layer_names: &[String],
        macro_names: &[String],
        tap_dance_names: &[String],
        key_legend_layout: KeyLegendLayout,
        show_shifted_number_symbols: bool,
        encoder_visibility: &[bool],
        matrix_pressed: &[bool],
        pressed_key_layers: &[Option<usize>],
        ui_scale: f32,
        dark: bool,
        rect: egui::Rect,
    ) {
        let painter = ui.painter_at(rect);
        let keyboard_rect = rect.shrink(STICKY_LAYOUT_KEYBOARD_MARGIN);
        let geometry = preview_layout_geometry(ui.ctx(), layout, keyboard_rect, ui_scale);
        let outline = if dark {
            Color32::from_rgb(58, 58, 62)
        } else {
            Color32::from_rgb(225, 225, 229)
        };
        let key_fill = if dark {
            Color32::from_rgb(48, 48, 52)
        } else {
            Color32::from_rgb(255, 255, 255)
        };
        let empty_fill = if dark {
            Color32::from_rgb(28, 28, 31)
        } else {
            Color32::from_rgb(248, 248, 250)
        };

        let key_rects: Vec<(usize, egui::Rect)> = layout
            .keys
            .iter()
            .enumerate()
            .map(|(ki, key)| (ki, layout_physical_key_rect(key, geometry)))
            .collect();

        let mut encoder_groups: Vec<(u8, egui::Rect, Option<(usize, u16)>, Option<(usize, u16)>)> =
            Vec::new();
        for (encoder_idx, encoder) in layout.encoders.iter().enumerate() {
            if !encoder_visibility
                .get(encoder.encoder_idx as usize)
                .copied()
                .unwrap_or(true)
            {
                continue;
            }

            let encoder_rect = layout_physical_encoder_rect(encoder, geometry);
            let kc = layout.get_encoder_keycode(layer, encoder_idx);
            if let Some((_, group_rect, ccw, cw)) = encoder_groups
                .iter_mut()
                .find(|(idx, _, _, _)| *idx == encoder.encoder_idx)
            {
                *group_rect = group_rect.union(encoder_rect);
                if encoder.direction == 0 {
                    *ccw = Some((encoder_idx, kc));
                } else {
                    *cw = Some((encoder_idx, kc));
                }
            } else {
                encoder_groups.push((
                    encoder.encoder_idx,
                    encoder_rect,
                    (encoder.direction == 0).then_some((encoder_idx, kc)),
                    (encoder.direction != 0).then_some((encoder_idx, kc)),
                ));
            }
        }

        let mut encoder_press_rects: Vec<(usize, egui::Rect)> = Vec::new();
        for (_, group_rect, _, _) in &encoder_groups {
            let center = group_rect.center();
            let radius = group_rect.width().min(group_rect.height()) * 0.5;
            let mut best_key: Option<(usize, f32)> = None;
            for (ki, key_rect) in &key_rects {
                if encoder_press_rects
                    .iter()
                    .any(|(assigned_ki, _)| assigned_ki == ki)
                {
                    continue;
                }
                let dist = key_rect.center().distance(center);
                if dist > radius * 0.38 {
                    continue;
                }
                match best_key {
                    Some((_, best_dist)) if dist >= best_dist => {}
                    _ => best_key = Some((*ki, dist)),
                }
            }
            if let Some((ki, _)) = best_key {
                let press_rect = egui::Rect::from_center_size(
                    center,
                    Vec2::new(
                        (radius * 0.88).min(group_rect.width() * 0.44),
                        (radius * 0.48).min(group_rect.height() * 0.22),
                    ),
                );
                encoder_press_rects.push((ki, press_rect));
            }
        }

        for (ki, key_rect) in &key_rects {
            if encoder_press_rects
                .iter()
                .any(|(press_ki, _)| press_ki == ki)
            {
                continue;
            }

            let key = &layout.keys[*ki];
            let matrix_idx = key.row as usize * layout.cols + key.col as usize;
            let is_pressed = layout_matrix_key_pressed(layout, matrix_pressed, key.row, key.col);
            let key_layer = if is_pressed {
                pressed_key_layers
                    .get(matrix_idx)
                    .and_then(|source_layer| *source_layer)
                    .filter(|source_layer| *source_layer < layout.layers.len())
                    .unwrap_or(layer)
            } else {
                layer
            };
            let kc = layout.get_keycode(key_layer, *ki);
            let is_transparent = kc == 0x0001;
            let fill = if is_pressed {
                app_hover_fill(dark)
            } else if kc == 0x0000 {
                empty_fill
            } else {
                key_fill
            };
            let stroke = if is_pressed { app_accent() } else { outline };
            paint_layout_keycap(
                &painter,
                *key_rect,
                key.rotation,
                fill,
                Stroke::new(1.0, stroke),
            );

            if kc == 0x0000 {
                continue;
            }

            let label_kc = if is_transparent {
                (0..key_layer)
                    .rev()
                    .map(|fallback_layer| layout.get_keycode(fallback_layer, *ki))
                    .find(|fallback| !matches!(*fallback, 0x0000 | 0x0001))
                    .unwrap_or(0x0000)
            } else {
                kc
            };
            if label_kc == 0x0000 {
                continue;
            }
            let label = number_row_shifted_label(
                keycode_label_with_macro_names(
                    label_kc,
                    &layout.custom_keycodes,
                    layer_names,
                    macro_names,
                    tap_dance_names,
                    key_legend_layout,
                ),
                show_shifted_number_symbols,
                key_legend_layout,
            );
            draw_sticky_key_label(
                &painter,
                *key_rect,
                &label,
                dark,
                key.rotation.to_radians(),
                is_transparent,
            );
        }

        let label_for = |kc: Option<u16>| -> String {
            let label = match kc.unwrap_or(0) {
                0x0000 => "✕".to_string(),
                0x0001 => "▽".to_string(),
                value => keycode_label_with_macro_names(
                    value,
                    &layout.custom_keycodes,
                    layer_names,
                    macro_names,
                    tap_dance_names,
                    key_legend_layout,
                )
                .replace('\n', " "),
            };
            sticky_compact_label(&label, 9)
        };
        let text_color = if dark {
            Color32::from_gray(232)
        } else {
            Color32::from_gray(32)
        };

        for (_, group_rect, ccw, cw) in encoder_groups {
            let center = group_rect.center();
            let radius = (group_rect.width().min(group_rect.height())
                * LAYOUT_ENCODER_RADIUS_FACTOR)
                .max(14.0);
            let fill_radius = radius + LAYOUT_ENCODER_FILL_EXTRA;
            let press_slot = encoder_press_rects
                .iter()
                .find(|(_, press_rect)| press_rect.center().distance(center) < 1.0)
                .map(|(press_ki, press_rect)| (*press_ki, *press_rect));
            let press_is_pressed = press_slot
                .map(|(press_ki, _)| {
                    let key = &layout.keys[press_ki];
                    layout_matrix_key_pressed(layout, matrix_pressed, key.row, key.col)
                })
                .unwrap_or(false);

            let (top_rect, middle_rect, bottom_rect) = if let Some((_, press_rect)) = press_slot {
                let divider_gap = radius * 0.06;
                let top_divider_y = press_rect.top() - divider_gap;
                let bottom_divider_y = press_rect.bottom() + divider_gap;
                (
                    egui::Rect::from_min_max(
                        egui::pos2(center.x - fill_radius, center.y - fill_radius),
                        egui::pos2(center.x + fill_radius, top_divider_y),
                    ),
                    Some(egui::Rect::from_min_max(
                        egui::pos2(center.x - fill_radius, top_divider_y),
                        egui::pos2(center.x + fill_radius, bottom_divider_y),
                    )),
                    egui::Rect::from_min_max(
                        egui::pos2(center.x - fill_radius, bottom_divider_y),
                        egui::pos2(center.x + fill_radius, center.y + fill_radius),
                    ),
                )
            } else {
                (
                    egui::Rect::from_min_max(
                        egui::pos2(center.x - fill_radius, center.y - fill_radius),
                        egui::pos2(center.x + fill_radius, center.y),
                    ),
                    None,
                    egui::Rect::from_min_max(
                        egui::pos2(center.x - fill_radius, center.y),
                        egui::pos2(center.x + fill_radius, center.y + fill_radius),
                    ),
                )
            };

            painter.circle_filled(center, fill_radius, key_fill);
            painter
                .with_clip_rect(top_rect)
                .circle_filled(center, fill_radius, key_fill);
            if let Some(middle_rect) = middle_rect {
                let middle_fill = if press_is_pressed {
                    app_hover_fill(dark)
                } else {
                    key_fill
                };
                painter
                    .with_clip_rect(middle_rect)
                    .circle_filled(center, fill_radius, middle_fill);
            }
            painter
                .with_clip_rect(bottom_rect)
                .circle_filled(center, fill_radius, key_fill);
            painter.circle_stroke(center, radius, Stroke::new(1.0, outline));

            let has_press_button = press_slot.is_some();
            let top_label = label_for(cw.map(|(_, kc)| kc));
            let bottom_label = label_for(ccw.map(|(_, kc)| kc));
            let top_font = if has_press_button {
                egui::FontId::proportional(if top_label.chars().count() > 9 {
                    6.6
                } else {
                    7.4
                })
            } else {
                egui::FontId::proportional(if top_label.chars().count() > 9 {
                    8.5
                } else {
                    9.5
                })
            };
            let bottom_font = if has_press_button {
                egui::FontId::proportional(if bottom_label.chars().count() > 9 {
                    6.6
                } else {
                    7.4
                })
            } else {
                egui::FontId::proportional(if bottom_label.chars().count() > 9 {
                    8.5
                } else {
                    9.5
                })
            };
            let top_label_y = center.y - radius * if has_press_button { 0.52 } else { 0.30 };
            let bottom_label_y = center.y + radius * if has_press_button { 0.52 } else { 0.30 };
            painter.text(
                egui::pos2(center.x, top_label_y),
                egui::Align2::CENTER_CENTER,
                top_label,
                top_font,
                text_color,
            );
            painter.text(
                egui::pos2(center.x, bottom_label_y),
                egui::Align2::CENTER_CENTER,
                bottom_label,
                bottom_font,
                text_color,
            );

            draw_sticky_encoder_arrow(&painter, center, radius, true, outline);
            draw_sticky_encoder_arrow(&painter, center, radius, false, outline);

            if let Some((press_ki, _)) = press_slot {
                let middle_rect = middle_rect.unwrap();
                let top_divider_y = middle_rect.top();
                let bottom_divider_y = middle_rect.bottom();
                let divider_radius = (radius - 0.5).max(0.0);
                let top_divider_half_width = (divider_radius * divider_radius
                    - (top_divider_y - center.y) * (top_divider_y - center.y))
                    .max(0.0)
                    .sqrt();
                let bottom_divider_half_width = (divider_radius * divider_radius
                    - (bottom_divider_y - center.y) * (bottom_divider_y - center.y))
                    .max(0.0)
                    .sqrt();
                painter.line_segment(
                    [
                        egui::pos2(center.x - top_divider_half_width, top_divider_y),
                        egui::pos2(center.x + top_divider_half_width, top_divider_y),
                    ],
                    Stroke::new(1.0, outline),
                );
                painter.line_segment(
                    [
                        egui::pos2(center.x - bottom_divider_half_width, bottom_divider_y),
                        egui::pos2(center.x + bottom_divider_half_width, bottom_divider_y),
                    ],
                    Stroke::new(1.0, outline),
                );

                let press_label = {
                    let key = &layout.keys[press_ki];
                    let matrix_idx = key.row as usize * layout.cols + key.col as usize;
                    let press_layer = pressed_key_layers
                        .get(matrix_idx)
                        .and_then(|source_layer| *source_layer)
                        .filter(|source_layer| *source_layer < layout.layers.len())
                        .unwrap_or(layer);
                    let kc = layout.get_keycode(press_layer, press_ki);
                    if kc == 0x0001 {
                        let fallback_kc = (0..press_layer)
                            .rev()
                            .map(|fallback_layer| layout.get_keycode(fallback_layer, press_ki))
                            .find(|fallback| !matches!(*fallback, 0x0000 | 0x0001))
                            .unwrap_or(0x0000);
                        if fallback_kc == 0x0000 {
                            "▽".to_string()
                        } else {
                            keycode_label_with_macro_names(
                                fallback_kc,
                                &layout.custom_keycodes,
                                layer_names,
                                macro_names,
                                tap_dance_names,
                                key_legend_layout,
                            )
                        }
                    } else if kc == 0x0000 {
                        "✕".to_string()
                    } else {
                        keycode_label_with_macro_names(
                            kc,
                            &layout.custom_keycodes,
                            layer_names,
                            macro_names,
                            tap_dance_names,
                            key_legend_layout,
                        )
                    }
                }
                .replace('\n', " ");
                let press_label = sticky_compact_label(&press_label, 8);
                let press_text_rect = middle_rect.shrink2(egui::vec2(4.0, 2.0));
                let press_font = FontId::proportional(if press_label.chars().count() > 8 {
                    7.2
                } else {
                    8.2
                });
                painter.with_clip_rect(press_text_rect).text(
                    press_text_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    press_label,
                    press_font,
                    text_color,
                );
            } else {
                let divider_half_width = (radius - 0.5).max(0.0);
                painter.line_segment(
                    [
                        egui::pos2(center.x - divider_half_width, center.y),
                        egui::pos2(center.x + divider_half_width, center.y),
                    ],
                    Stroke::new(1.0, outline),
                );
            }
        }
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
                draw_theme_selector_labels(ui, self.app_settings.language, &mut self.dark_mode);
            });

        // Keycode picker modal
        // Vial unlock modal
        if self.unlock_open && self.firmware == FirmwareProtocol::Vial {
            // Start unlock if not yet polling
            if !self.vial_unlock_polling {
                if let Some(hid) = &self.hid_device {
                    // Get unlock keys from get_unlock_status
                    match hid.get_unlock_status() {
                        Ok((_, keys)) => {
                            self.vial_unlock_keys = keys;
                        }
                        Err(_) => {}
                    }
                    // Start the unlock process
                    match hid.unlock_start() {
                        Ok(()) => {
                            self.vial_unlock_polling = true;
                            self.vial_unlock_best = 50;
                        }
                        Err(e) => {
                            self.status_msg = format!("Unlock start failed: {e}");
                            self.unlock_open = false;
                        }
                    }
                }
            }
            // Poll unlock
            if self.vial_unlock_polling {
                if let Some(hid) = &self.hid_device {
                    match hid.unlock_poll() {
                        Ok((unlocked, _in_progress, counter)) => {
                            self.vial_unlock_counter = counter;
                            if counter < self.vial_unlock_best {
                                self.vial_unlock_best = counter;
                            }
                            if unlocked {
                                self.status_msg = "Keyboard unlocked!".into();
                                self.unlock_open = false;
                                self.vial_unlock_polling = false;
                                self.macro_auto_unlock_cancelled = false;
                            }
                        }
                        Err(_) => {}
                    }
                }
                // Poll at ~120ms intervals (firmware timer threshold is 100ms)
                ctx.request_repaint_after(std::time::Duration::from_millis(120));
            }
            // Fullscreen overlay with layout and highlighted keys
            let unlock_keys = self.vial_unlock_keys.clone();
            let counter = self.vial_unlock_best;
            let total = self.vial_unlock_total;

            egui::Area::new(egui::Id::new("unlock_overlay"))
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let screen = ui.ctx().screen_rect();
                    let dark = ui.visuals().dark_mode;
                    let screen_bg = app_panel_fill(dark);
                    let title_color = if dark {
                        Color32::WHITE
                    } else {
                        Color32::from_gray(28)
                    };
                    let subtitle_color = if dark {
                        Color32::from_gray(180)
                    } else {
                        Color32::from_gray(96)
                    };
                    let bar_bg = if dark {
                        Color32::from_gray(40)
                    } else {
                        Color32::from_gray(220)
                    };
                    let inactive_key_bg = if dark {
                        Color32::from_rgb(48, 48, 52)
                    } else {
                        Color32::from_rgb(255, 255, 255)
                    };
                    let inactive_key_border = if dark {
                        Color32::from_rgb(54, 54, 58)
                    } else {
                        Color32::from_rgb(230, 230, 233)
                    };
                    ui.painter().rect_filled(screen, 0.0, screen_bg);

                    let center_x = screen.center().x;
                    let top_y = screen.min.y + 40.0;

                    // Title
                    ui.painter().text(
                        egui::pos2(center_x, top_y),
                        egui::Align2::CENTER_CENTER,
                        crate::i18n::tr_catalog(
                            self.app_settings.language,
                            "app_chrome.unlock_unlock_keyboard",
                        ),
                        FontId::proportional(24.0),
                        title_color,
                    );

                    ui.painter().text(
                        egui::pos2(center_x, top_y + 30.0),
                        egui::Align2::CENTER_CENTER,
                        crate::i18n::tr_catalog(
                            self.app_settings.language,
                            "unlock.highlighted_keys_hint",
                        ),
                        FontId::proportional(14.0),
                        subtitle_color,
                    );

                    // Progress bar
                    let progress = if total > 0 {
                        1.0 - (counter as f32 / total as f32)
                    } else {
                        0.0
                    };
                    let bar_w = 300.0f32;
                    let bar_h = 12.0f32;
                    let bar_y = top_y + 55.0;
                    let bar_rect = egui::Rect::from_min_size(
                        egui::pos2(center_x - bar_w / 2.0, bar_y),
                        egui::Vec2::new(bar_w, bar_h),
                    );
                    ui.painter().rect(
                        bar_rect,
                        4.0,
                        bar_bg,
                        egui::Stroke::NONE,
                        egui::StrokeKind::Inside,
                    );
                    let fill_rect = egui::Rect::from_min_size(
                        bar_rect.min,
                        egui::Vec2::new(bar_w * progress, bar_h),
                    );
                    ui.painter().rect(
                        fill_rect,
                        4.0,
                        app_accent(),
                        egui::Stroke::NONE,
                        egui::StrokeKind::Inside,
                    );

                    // Draw layout keys with highlighted unlock keys
                    if let Some(layout) = &self.layout {
                        let (off_x, off_y, unit, padding) =
                            self.last_layout_geometry.unwrap_or_else(|| {
                                let geometry = layout_geometry(
                                    ui.ctx(),
                                    layout,
                                    screen,
                                    clamp_ui_scale(self.app_settings.ui_scale),
                                );
                                (
                                    geometry.offset_x,
                                    geometry.offset_y,
                                    geometry.unit,
                                    geometry.padding,
                                )
                            });
                        for key in &layout.keys {
                            let is_unlock = unlock_keys
                                .iter()
                                .any(|(r, c)| key.row == *r && key.col == *c);
                            let geometry = LayoutGeometry {
                                offset_x: off_x,
                                offset_y: off_y,
                                unit,
                                padding,
                                layout_h: 0.0,
                            };
                            let rect = layout_physical_key_rect(key, geometry);
                            let bg = if is_unlock {
                                app_accent()
                            } else {
                                inactive_key_bg
                            };
                            let border = if is_unlock {
                                app_accent()
                            } else {
                                inactive_key_border
                            };
                            paint_layout_keycap(
                                ui.painter(),
                                rect,
                                key.rotation,
                                bg,
                                Stroke::new(1.0, border),
                            );
                        }
                    }
                });
        }

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
    fn push_combo_undo(&mut self) {
        self.combo_undo_stack.push((
            self.combo_entries.clone(),
            self.combo_names.clone(),
            self.combo_term,
            self.selected_combo,
            self.combo_visible_count,
        ));
        if self.combo_undo_stack.len() > 64 {
            self.combo_undo_stack.remove(0);
        }
    }

    fn apply_combo_capture(&mut self) {
        if !(2..=4).contains(&self.combo_capture_keys.len()) {
            return;
        }
        self.push_combo_undo();
        if let Some(combo) = self.combo_entries.get_mut(self.selected_combo) {
            combo.keys = [0; 4];
            for (idx, kc) in self.combo_capture_keys.iter().copied().enumerate().take(4) {
                combo.keys[idx] = kc;
            }
            self.combo_dirty = true;
        }
        self.combo_capture_open = false;
        self.combo_capture_keys.clear();
    }

    fn cancel_combo_capture(&mut self) {
        self.combo_capture_open = false;
        self.combo_capture_keys.clear();
    }

    fn push_key_override_undo(&mut self) {
        self.key_override_undo_stack.push((
            self.key_override_entries.clone(),
            self.key_override_names.clone(),
            self.selected_key_override,
            self.key_override_visible_count,
        ));
        if self.key_override_undo_stack.len() > 64 {
            self.key_override_undo_stack.remove(0);
        }
    }

    fn write_all_key_overrides(&mut self) {
        for idx in 0..self.key_override_entries.len() {
            self.write_key_override(idx);
        }
    }

    fn key_override_entry_exists(entry: &KeyOverrideEntry) -> bool {
        entry.trigger != 0
            || entry.replacement != 0
            || entry.layers != 0
            || entry.trigger_mods != 0
            || entry.negative_mod_mask != 0
            || entry.suppressed_mods != 0
            || entry.options.activation_trigger_down
            || entry.options.activation_required_mod_down
            || entry.options.activation_negative_mod_up
            || entry.options.one_mod
            || entry.options.no_reregister_trigger
            || entry.options.no_unregister_on_other_key_down
    }

    fn normalize_key_override_entry(entry: &mut KeyOverrideEntry) {
        entry.options.enabled = Self::key_override_entry_exists(entry);
    }

    fn write_key_override(&mut self, idx: usize) {
        let Some(entry) = self.key_override_entries.get_mut(idx) else {
            return;
        };
        Self::normalize_key_override_entry(entry);
        let entry = entry.clone();
        let Some(hid) = &self.hid_device else {
            return;
        };
        if let Err(e) = hid.set_key_override(
            idx as u8,
            entry.trigger,
            entry.replacement,
            entry.layers,
            entry.trigger_mods,
            entry.negative_mod_mask,
            entry.suppressed_mods,
            entry.options.bits(),
        ) {
            self.status_msg = format!("Failed to save Key Override {}: {}", idx + 1, e);
            log::warn!("set_key_override({idx}) failed: {e}");
        }
    }

    fn open_key_override_picker(&mut self, target: KeyOverridePickField) {
        self.key_override_pick_target = Some(target);
        self.keycode_picker.result = None;
        self.keycode_picker.open = true;
    }

    fn push_alt_repeat_undo(&mut self) {
        self.alt_repeat_undo_stack.push((
            self.alt_repeat_entries.clone(),
            self.alt_repeat_names.clone(),
            self.selected_alt_repeat,
        ));
        if self.alt_repeat_undo_stack.len() > 64 {
            self.alt_repeat_undo_stack.remove(0);
        }
    }

    fn alt_repeat_entry_exists(entry: &AltRepeatKeyEntry) -> bool {
        entry.keycode != 0 || entry.alt_keycode != 0 || entry.allowed_mods != 0
    }

    fn normalize_alt_repeat_entry(entry: &mut AltRepeatKeyEntry) {
        entry.options.enabled = Self::alt_repeat_entry_exists(entry);
    }

    fn write_alt_repeat_entry(&mut self, idx: usize) {
        let Some(entry) = self.alt_repeat_entries.get_mut(idx) else {
            return;
        };
        Self::normalize_alt_repeat_entry(entry);
        let entry = entry.clone();
        let Some(hid) = &self.hid_device else {
            return;
        };
        if let Err(e) = hid.set_alt_repeat_key(
            idx as u8,
            entry.keycode,
            entry.alt_keycode,
            entry.allowed_mods,
            entry.options.bits(),
        ) {
            self.status_msg = format!("Failed to save Alt Repeat {}: {}", idx + 1, e);
            log::warn!("set_alt_repeat_key({idx}) failed: {e}");
        }
    }

    fn open_alt_repeat_picker(&mut self, target: AltRepeatPickField) {
        self.alt_repeat_pick_target = Some(target);
        self.keycode_picker.result = None;
        self.keycode_picker.open = true;
    }

    fn open_alt_repeat_window_compact(&mut self) {
        self.selected_alt_repeat = 0;
        self.alt_repeat_visible_count = 1;
        self.settings_tab = SettingsTab::AltRepeat;
        self.main_menu_tab = MainMenuTab::Settings;
    }

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

    fn is_encoder_layout_option(option: &LayoutOption) -> bool {
        option
            .label
            .trim_start()
            .to_ascii_lowercase()
            .starts_with("hide encoder")
    }

    fn display_preset_choice_label(language: crate::i18n::Language, label: &str) -> String {
        let label = label
            .replace(" (qmk-hid-host)", " (Entropy)")
            .replace("qmk-hid-host", "Entropy");
        if matches!(language, crate::i18n::Language::Russian) {
            let lower = label.to_ascii_lowercase();
            for (prefix, translated) in [
                ("oled master", "OLED мастер"),
                ("oled slave", "OLED ведомый"),
            ] {
                if lower == prefix || lower.starts_with(&format!("{} ", prefix)) {
                    return format!("{}{}", translated, &label[prefix.len()..]);
                }
            }
        }
        crate::i18n::tr_text(language, &label)
    }

    fn display_preset_needs_entropy(label: &str) -> bool {
        let lower = label.to_ascii_lowercase();
        lower.contains("qmk-hid-host")
            || lower.contains("clock")
            || lower.contains("volume")
            || lower.contains("media")
    }

    fn static_display_preset_fallback_idx(option: &LayoutOption) -> Option<usize> {
        option
            .choices
            .iter()
            .position(|choice| choice.eq_ignore_ascii_case("disabled"))
    }

    fn is_display_preset_layout_option(option: &LayoutOption) -> bool {
        !Self::is_encoder_layout_option(option)
            && option
                .choices
                .iter()
                .any(|choice| Self::display_preset_needs_entropy(choice))
            && Self::static_display_preset_fallback_idx(option).is_some()
    }

    fn selected_layout_option_idx(option: &LayoutOption, values: &[u32], idx: usize) -> usize {
        values
            .get(idx)
            .copied()
            .unwrap_or(0)
            .min(option.choices.len().saturating_sub(1) as u32) as usize
    }

    fn restore_display_preset_packed(
        layout: &KeyboardLayout,
        current_packed: u32,
        restore_packed: u32,
    ) -> Option<u32> {
        let mut current_values =
            Self::unpack_layout_option_values(&layout.layout_options, current_packed);
        let restore_values =
            Self::unpack_layout_option_values(&layout.layout_options, restore_packed);
        let mut changed = false;

        for (idx, option) in layout.layout_options.iter().enumerate() {
            if !Self::is_display_preset_layout_option(option) {
                continue;
            }
            let Some(disabled_idx) = Self::static_display_preset_fallback_idx(option) else {
                continue;
            };
            let current_idx = Self::selected_layout_option_idx(option, &current_values, idx);
            if current_idx != disabled_idx {
                continue;
            }
            let restore_idx = Self::selected_layout_option_idx(option, &restore_values, idx);
            let restore_needs_entropy = option
                .choices
                .get(restore_idx)
                .map(|choice| Self::display_preset_needs_entropy(choice))
                .unwrap_or(false);
            if !restore_needs_entropy {
                continue;
            }
            current_values[idx] = restore_idx as u32;
            changed = true;
        }

        changed.then(|| Self::pack_layout_option_values(&layout.layout_options, &current_values))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save_display_preset_restore(&self, packed: u32) {
        if self.current_device_name.is_empty() {
            return;
        }
        if let Err(e) = std::fs::write(
            display_preset_restore_path(&self.current_device_name),
            packed.to_string(),
        ) {
            log::warn!("save display preset restore failed: {e}");
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_display_preset_restore(&self) -> Option<u32> {
        if self.current_device_name.is_empty() {
            return None;
        }
        std::fs::read_to_string(display_preset_restore_path(&self.current_device_name))
            .ok()
            .and_then(|text| text.trim().parse::<u32>().ok())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn clear_display_preset_restore(&self) {
        if self.current_device_name.is_empty() {
            return;
        }
        let path = display_preset_restore_path(&self.current_device_name);
        if let Err(e) = std::fs::remove_file(path) {
            if e.kind() != std::io::ErrorKind::NotFound {
                log::warn!("clear display preset restore failed: {e}");
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn restore_entropy_display_preset_after_connect(&mut self) {
        let Some(layout) = self.layout.as_ref() else {
            return;
        };
        let Some(current_packed) = self.layout_options_value else {
            return;
        };
        let Some(restore_packed) = self.load_display_preset_restore() else {
            return;
        };
        let Some(packed) =
            Self::restore_display_preset_packed(layout, current_packed, restore_packed)
        else {
            return;
        };

        self.layout_options_value = Some(packed);
        if let Some(hid) = &self.hid_device {
            if let Err(e) = hid.set_layout_options(packed) {
                log::warn!("restore display preset after connect failed: {e}");
                self.layout_options_value = Some(current_packed);
                return;
            }
        }
        self.sync_qmk_hid_host_bridges();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn device_uses_automatic_display_host_data(device: &crate::device::Device) -> bool {
        if device.firmware != FirmwareProtocol::Vial {
            return false;
        }

        let name = device.name.to_ascii_lowercase();
        let ergohaven_macropad_display =
            device.vendor_id == 0xE126 && matches!(device.product_id, 0x0041 | 0x0042);

        ergohaven_macropad_display || name.contains("m4cr0pad v2") || name.contains("m4cr0pad v3")
    }

    fn touchpad_setting_field(json: &serde_json::Value, qsid: u16) -> Option<&serde_json::Value> {
        json.get("settings")
            .and_then(|value| value.as_array())?
            .iter()
            .find(|tab| {
                tab.get("name")
                    .and_then(|value| value.as_str())
                    .map(|name| name.to_ascii_lowercase().contains("touchpad"))
                    .unwrap_or(false)
            })?
            .get("fields")
            .and_then(|value| value.as_array())?
            .iter()
            .find(|field| field.get("qsid").and_then(|value| value.as_u64()) == Some(qsid as u64))
    }

    fn touchpad_setting_exists(json: &serde_json::Value, qsid: u16) -> bool {
        Self::touchpad_setting_field(json, qsid).is_some()
    }

    fn touchpad_setting_variants(json: &serde_json::Value, qsid: u16) -> Vec<String> {
        Self::touchpad_setting_field(json, qsid)
            .and_then(|field| field.get("variants"))
            .and_then(|value| value.as_array())
            .map(|variants| {
                variants
                    .iter()
                    .filter_map(|value| value.as_str().map(|s| s.trim().to_string()))
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn layout_json_has_touchpad_settings(json: &serde_json::Value) -> bool {
        let Some(tabs) = json.get("settings").and_then(|value| value.as_array()) else {
            return false;
        };

        tabs.iter().any(|tab| {
            let tab_name = tab
                .get("name")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let has_touchpad_name = tab_name.contains("touchpad");
            let has_touchpad_qsids = tab
                .get("fields")
                .and_then(|value| value.as_array())
                .map(|fields| {
                    [120u64, 121, 122, 123, 124].iter().all(|qsid| {
                        fields.iter().any(|field| {
                            field.get("qsid").and_then(|value| value.as_u64()) == Some(*qsid)
                        })
                    })
                })
                .unwrap_or(false);

            has_touchpad_name && has_touchpad_qsids
        })
    }

    fn module_settings_fields(
        json: &serde_json::Value,
        supported_qmk_settings: &[u16],
    ) -> Vec<ModuleSettingField> {
        let Some(tabs) = json.get("settings").and_then(|value| value.as_array()) else {
            return Vec::new();
        };
        let Some(tab) = tabs.iter().find(|tab| {
            tab.get("name")
                .and_then(|value| value.as_str())
                .map(|name| name.to_ascii_lowercase().contains("module"))
                .unwrap_or(false)
        }) else {
            return Vec::new();
        };
        let Some(fields) = tab.get("fields").and_then(|value| value.as_array()) else {
            return Vec::new();
        };

        fields
            .iter()
            .filter_map(|field| {
                let qsid = field.get("qsid")?.as_u64()? as u16;
                if !supported_qmk_settings.contains(&qsid) {
                    return None;
                }
                let title = field.get("title")?.as_str()?.trim().to_string();
                if title.is_empty() {
                    return None;
                }
                let kind = match field.get("type").and_then(|value| value.as_str())? {
                    "boolean" => ModuleSettingKind::Boolean,
                    "integer" => ModuleSettingKind::Integer,
                    "select" => ModuleSettingKind::Select,
                    _ => return None,
                };
                let width = field
                    .get("width")
                    .and_then(|value| value.as_u64())
                    .unwrap_or(1)
                    .clamp(1, 2) as u8;
                let variants = field
                    .get("variants")
                    .and_then(|value| value.as_array())
                    .map(|variants| {
                        variants
                            .iter()
                            .filter_map(|value| value.as_str().map(|s| s.trim().to_string()))
                            .filter(|s| !s.is_empty())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                Some(ModuleSettingField {
                    title,
                    qsid,
                    kind,
                    bit: field
                        .get("bit")
                        .and_then(|value| value.as_u64())
                        .unwrap_or(0)
                        .min(15) as u8,
                    width,
                    min: field
                        .get("min")
                        .and_then(|value| value.as_u64())
                        .unwrap_or(0)
                        .min(u16::MAX as u64) as u16,
                    max: field
                        .get("max")
                        .and_then(|value| value.as_u64())
                        .unwrap_or(if matches!(kind, ModuleSettingKind::Select) {
                            variants.len().saturating_sub(1) as u64
                        } else if width > 1 {
                            u16::MAX as u64
                        } else {
                            u8::MAX as u64
                        })
                        .min(u16::MAX as u64) as u16,
                    variants,
                })
            })
            .collect()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn read_module_settings(
        json: &serde_json::Value,
        supported_qmk_settings: &[u16],
        dev_conn: &crate::hid::HidDevice,
    ) -> ModuleSettingsState {
        let fields = Self::module_settings_fields(json, supported_qmk_settings);
        if fields.is_empty() {
            return ModuleSettingsState::default();
        }

        let mut settings = ModuleSettingsState {
            fields,
            values: std::collections::BTreeMap::new(),
            supported: true,
        };
        let mut widths = std::collections::BTreeMap::<u16, u8>::new();
        for field in &settings.fields {
            widths
                .entry(field.qsid)
                .and_modify(|width| *width = (*width).max(field.width))
                .or_insert(field.width);
        }
        for (qsid, width) in widths {
            let value = if width > 1 {
                dev_conn.get_qmk_setting_u16(qsid)
            } else {
                dev_conn.get_qmk_setting_u8(qsid).map(|value| value as u16)
            }
            .unwrap_or_else(|e| {
                log::warn!("get_qmk_setting(module qsid {qsid}): {e}");
                0
            });
            settings.values.insert(qsid, value);
        }
        settings
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn qmk_hid_host_mode_for(
        layout: &KeyboardLayout,
        packed: Option<u32>,
    ) -> crate::qmk_hid_host::HostDataMode {
        let values = Self::unpack_layout_option_values(&layout.layout_options, packed.unwrap_or(0));
        let mut mode = crate::qmk_hid_host::HostDataMode::default();
        for (idx, option) in layout.layout_options.iter().enumerate() {
            if Self::is_encoder_layout_option(option) || option.choices.is_empty() {
                continue;
            }
            let selected_idx = values
                .get(idx)
                .copied()
                .unwrap_or(0)
                .min(option.choices.len().saturating_sub(1) as u32)
                as usize;
            let selected = option
                .choices
                .get(selected_idx)
                .map(|s| s.as_str())
                .unwrap_or("");
            let selected_lower = selected.to_ascii_lowercase();
            if Self::display_preset_needs_entropy(selected)
                && (selected_lower.contains("clock") || selected_lower.contains("volume"))
            {
                mode.clock_volume = true;
            }
            if Self::display_preset_needs_entropy(selected) && selected_lower.contains("media") {
                mode.media = true;
            }
        }
        mode
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn fallback_entropy_display_presets_before_exit(&mut self) {
        let Some(layout) = self.layout.as_ref() else {
            return;
        };
        if layout.layout_options.is_empty() {
            return;
        }

        let mut values = Self::unpack_layout_option_values(
            &layout.layout_options,
            self.layout_options_value.unwrap_or(0),
        );
        let mut changed = false;

        for (idx, option) in layout.layout_options.iter().enumerate() {
            if Self::is_encoder_layout_option(option) || option.choices.is_empty() {
                continue;
            }
            let selected_idx = values
                .get(idx)
                .copied()
                .unwrap_or(0)
                .min(option.choices.len().saturating_sub(1) as u32)
                as usize;
            let selected = option
                .choices
                .get(selected_idx)
                .map(|s| s.as_str())
                .unwrap_or("");
            if !Self::display_preset_needs_entropy(selected) {
                continue;
            }
            if let Some(fallback_idx) = Self::static_display_preset_fallback_idx(option) {
                if fallback_idx != selected_idx {
                    values[idx] = fallback_idx as u32;
                    changed = true;
                }
            }
        }

        if !changed {
            return;
        }

        let original_packed = self.layout_options_value.unwrap_or(0);
        let packed = Self::pack_layout_option_values(&layout.layout_options, &values);
        self.save_display_preset_restore(original_packed);
        self.layout_options_value = Some(packed);
        self.qmk_hid_hosts.clear();
        if let Some(hid) = &self.hid_device {
            if let Err(e) = hid.set_layout_options(packed) {
                log::warn!("fallback display preset before exit failed: {e}");
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn sync_qmk_hid_host_bridges(&mut self) {
        let selected_path = self
            .selected_device
            .and_then(|idx| self.device_manager.devices().get(idx))
            .map(|device| device.path.as_str());

        let mut desired =
            std::collections::HashMap::<String, crate::qmk_hid_host::HostDataMode>::new();

        for device in self.device_manager.devices() {
            if device.firmware != FirmwareProtocol::Vial {
                continue;
            }

            let mut mode = crate::qmk_hid_host::HostDataMode::default();
            if Some(device.path.as_str()) == selected_path {
                if let Some(layout) = self.layout.as_ref() {
                    mode = Self::qmk_hid_host_mode_for(layout, self.layout_options_value);
                }
            }

            if Self::device_uses_automatic_display_host_data(device) {
                mode.clock_volume = true;
                mode.media = true;
            }

            if !mode.is_empty() {
                desired.insert(device.path.clone(), mode);
            }
        }

        self.qmk_hid_hosts
            .retain(|path, bridge| desired.get(path).copied() == Some(bridge.mode()));

        for (path, mode) in desired {
            self.qmk_hid_hosts
                .entry(path.clone())
                .or_insert_with(|| crate::qmk_hid_host::QmkHidHostBridge::start(path, mode));
        }
    }

    fn open_layout_options_settings_page(&mut self) {
        self.settings_tab = SettingsTab::LayoutOptions;
        self.main_menu_tab = MainMenuTab::Settings;
    }

    fn open_modules_settings_page(&mut self) {
        self.settings_tab = SettingsTab::Modules;
        self.main_menu_tab = MainMenuTab::Settings;
    }

    fn open_touchpad_settings_page(&mut self) {
        self.settings_tab = SettingsTab::Touchpad;
        self.main_menu_tab = MainMenuTab::Settings;
    }

    fn open_live_features_settings_page(&mut self) {
        self.settings_tab = SettingsTab::LiveFeatures;
        self.main_menu_tab = MainMenuTab::Settings;
    }

    fn layout_option_width(option: &LayoutOption) -> usize {
        if option.choices.is_empty() {
            1
        } else {
            let max_value = option.choices.len().saturating_sub(1).max(1);
            (usize::BITS - max_value.leading_zeros()) as usize
        }
    }

    fn unpack_layout_option_values(options: &[LayoutOption], packed: u32) -> Vec<u32> {
        let mut values = vec![0; options.len()];
        let mut remaining = packed;
        for (idx, option) in options.iter().enumerate().rev() {
            let width = Self::layout_option_width(option);
            let mask = if width >= 32 {
                u32::MAX
            } else {
                (1u32 << width) - 1
            };
            values[idx] = remaining & mask;
            remaining >>= width.min(31);
        }
        values
    }

    fn pack_layout_option_values(options: &[LayoutOption], values: &[u32]) -> u32 {
        let mut packed = 0u32;
        for (idx, option) in options.iter().enumerate() {
            let width = Self::layout_option_width(option);
            let mask = if width >= 32 {
                u32::MAX
            } else {
                (1u32 << width) - 1
            };
            packed = (packed << width.min(31)) | (values.get(idx).copied().unwrap_or(0) & mask);
        }
        packed
    }

    fn set_layout_option_value(&mut self, option_idx: usize, value: u32) {
        let Some(options) = self
            .layout
            .as_ref()
            .map(|layout| layout.layout_options.clone())
        else {
            return;
        };
        if option_idx >= options.len() {
            return;
        }
        let mut values =
            Self::unpack_layout_option_values(&options, self.layout_options_value.unwrap_or(0));
        if let Some(slot) = values.get_mut(option_idx) {
            *slot = value;
        }
        let packed = Self::pack_layout_option_values(&options, &values);
        self.layout_options_value = Some(packed);
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(option) = options.get(option_idx) {
            if Self::is_display_preset_layout_option(option) {
                let selected_label = option
                    .choices
                    .get(value as usize)
                    .map(|choice| choice.as_str())
                    .unwrap_or("");
                if Self::display_preset_needs_entropy(selected_label) {
                    self.save_display_preset_restore(packed);
                } else {
                    self.clear_display_preset_restore();
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(hid) = &self.hid_device {
            if let Err(e) = hid.set_layout_options(packed) {
                self.status_msg = format!("Failed to save layout option: {e}");
                log::warn!("set_layout_options failed: {e}");
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.sync_qmk_hid_host_bridges();
    }

    fn close_top_dropdowns(&self, ctx: &egui::Context) {
        ctx.data_mut(|d| {
            d.insert_temp(egui::Id::new("device_dropdown_open"), false);
            d.insert_temp(egui::Id::new("advanced_dropdown_open"), false);
            d.insert_temp(egui::Id::new("settings_dropdown_open"), false);
        });
    }

    fn top_dropdown_open(&self, ctx: &egui::Context) -> bool {
        ctx.data(|d| {
            d.get_temp::<bool>(egui::Id::new("device_dropdown_open"))
                .unwrap_or(false)
                || d.get_temp::<bool>(egui::Id::new("advanced_dropdown_open"))
                    .unwrap_or(false)
                || d.get_temp::<bool>(egui::Id::new("settings_dropdown_open"))
                    .unwrap_or(false)
        })
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

    #[cfg(not(target_arch = "wasm32"))]
    fn poll_single_instance_signal(&mut self, ctx: &egui::Context) {
        let signal = read_single_instance_signal();
        if signal.is_empty() || signal == self.last_single_instance_signal {
            return;
        }
        self.last_single_instance_signal = signal;
        self.status_msg = "Entropy refreshed from a repeated launch".into();
        self.restore_from_tray(ctx);
        self.start_device_scan();
        ctx.request_repaint();
    }

    fn restore_from_tray(&mut self, ctx: &egui::Context) {
        #[cfg(target_os = "windows")]
        if let Some(hwnd) = self.windows_hwnd {
            unsafe {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOW,
                };
                let hwnd = hwnd as windows_sys::Win32::Foundation::HWND;
                ShowWindow(hwnd, SW_SHOW);
                ShowWindow(hwnd, SW_RESTORE);
                SetForegroundWindow(hwnd);
            }
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    #[cfg(target_os = "windows")]
    fn cache_windows_hwnd(&mut self, frame: &eframe::Frame) {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        if self.windows_hwnd.is_some() {
            return;
        }
        if let Ok(handle) = frame.window_handle() {
            if let RawWindowHandle::Win32(win32) = handle.as_raw() {
                self.windows_hwnd = Some(win32.hwnd.get());
            }
        }
    }

    fn handle_close_to_tray(&mut self, ctx: &egui::Context) {
        if !self.app_settings.minimize_to_tray_on_close {
            return;
        }
        if !ctx.input(|i| i.viewport().close_requested()) {
            return;
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        #[cfg(target_os = "windows")]
        {
            self.ensure_tray_icon(ctx);
            if let Some(hwnd) = self.windows_hwnd {
                unsafe {
                    use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
                    ShowWindow(hwnd as windows_sys::Win32::Foundation::HWND, SW_HIDE);
                }
                self.status_msg = "Entropy is running in the tray".into();
                return;
            }
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        self.status_msg = "Entropy is running in the tray".into();
    }

    #[cfg(target_os = "windows")]
    fn ensure_tray_icon(&mut self, ctx: &egui::Context) {
        if self.tray_icon.is_some() {
            return;
        }
        let mut rgba = Vec::with_capacity(16 * 16 * 4);
        let accent = app_accent();
        for y in 0..16 {
            for x in 0..16 {
                let inside = (3..=12).contains(&x) && (3..=12).contains(&y);
                let alpha = if inside { 255 } else { 0 };
                rgba.extend_from_slice(&[accent.r(), accent.g(), accent.b(), alpha]);
            }
        }
        let Ok(icon) = tray_icon::Icon::from_rgba(rgba, 16, 16) else {
            return;
        };
        let ctx_for_handler = ctx.clone();
        let hwnd_for_handler = self.windows_hwnd;
        tray_icon::TrayIconEvent::set_event_handler(Some(move |event| {
            use tray_icon::{MouseButton, MouseButtonState, TrayIconEvent};
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
                | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                } => {
                    if let Some(hwnd) = hwnd_for_handler {
                        unsafe {
                            use windows_sys::Win32::UI::WindowsAndMessaging::{
                                SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOW,
                            };
                            let hwnd = hwnd as windows_sys::Win32::Foundation::HWND;
                            ShowWindow(hwnd, SW_SHOW);
                            ShowWindow(hwnd, SW_RESTORE);
                            SetForegroundWindow(hwnd);
                        }
                    }
                    ctx_for_handler.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx_for_handler.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    ctx_for_handler.send_viewport_cmd(egui::ViewportCommand::Focus);
                    ctx_for_handler.request_repaint();
                }
                _ => {}
            }
        }));
        let ctx_for_menu = ctx.clone();
        tray_icon::menu::MenuEvent::set_event_handler(Some(
            move |event: tray_icon::menu::MenuEvent| {
                if event.id == "entropy_tray_quit" {
                    TRAY_QUIT_REQUESTED.store(true, std::sync::atomic::Ordering::Relaxed);
                    ctx_for_menu.request_repaint();
                }
            },
        ));
        let tray_menu = tray_icon::menu::Menu::new();
        let quit_item =
            tray_icon::menu::MenuItem::with_id("entropy_tray_quit", "Quit Entropy", true, None);
        if let Err(e) = tray_menu.append(&quit_item) {
            log::warn!("failed to create tray menu: {e}");
        }
        match tray_icon::TrayIconBuilder::new()
            .with_tooltip("Entropy")
            .with_menu(Box::new(tray_menu))
            .with_menu_on_left_click(false)
            .with_menu_on_right_click(true)
            .with_icon(icon)
            .build()
        {
            Ok(icon) => self.tray_icon = Some(icon),
            Err(e) => log::warn!("failed to create tray icon: {e}"),
        }
    }

    #[cfg(target_os = "windows")]
    fn handle_tray_quit_request(&mut self) {
        if TRAY_QUIT_REQUESTED.swap(false, std::sync::atomic::Ordering::Relaxed) {
            self.fallback_entropy_display_presets_before_exit();
            std::process::exit(0);
        }
    }

    #[cfg(target_os = "windows")]
    fn poll_tray_events(&mut self, ctx: &egui::Context) {
        use tray_icon::{MouseButton, MouseButtonState, TrayIconEvent};
        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
                | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                } => self.restore_from_tray(ctx),
                _ => {}
            }
        }
    }

    fn write_auto_shift_flags(&mut self) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        if let Err(e) = hid.set_qmk_setting_u8(3, self.auto_shift_options.bits()) {
            self.status_msg = format!("Failed to save Auto Shift flags: {}", e);
            log::warn!("set_qmk_setting_u8(auto_shift_flags) failed: {e}");
        }
    }

    fn write_auto_shift_timeout(&mut self) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let Some(timeout) = self.auto_shift_timeout else {
            return;
        };
        if let Err(e) = hid.set_qmk_setting_u16(4, timeout) {
            self.status_msg = format!("Failed to save Auto Shift timeout: {}", e);
            log::warn!("set_qmk_setting_u16(auto_shift_timeout) failed: {e}");
        }
    }

    fn set_rgb_effect(&mut self, effect: u16) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let result = match self.rgb_settings.kind {
            RgbSupportKind::QmkRgblight => {
                hid.set_qmk_rgblight_effect(effect.min(u8::MAX as u16) as u8)
            }
            RgbSupportKind::VialRgb => hid.set_vialrgb_mode(
                effect,
                self.rgb_settings.speed,
                self.rgb_settings.hue,
                self.rgb_settings.saturation,
                self.rgb_settings.brightness,
            ),
            RgbSupportKind::None => return,
        };
        match result {
            Ok(()) => {
                self.rgb_settings.effect = effect;
                if effect != 0 {
                    self.rgb_settings.last_enabled_effect = effect;
                }
                self.autosave_rgb_settings();
            }
            Err(e) => {
                self.status_msg = format!("Failed to update RGB effect: {}", e);
                log::warn!("set_rgb_effect failed: {e}");
            }
        }
    }

    fn set_rgb_brightness(&mut self, brightness: u8) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let result = match self.rgb_settings.kind {
            RgbSupportKind::QmkRgblight => hid.set_qmk_rgblight_brightness(brightness),
            RgbSupportKind::VialRgb => hid.set_vialrgb_mode(
                self.rgb_settings.effect,
                self.rgb_settings.speed,
                self.rgb_settings.hue,
                self.rgb_settings.saturation,
                brightness,
            ),
            RgbSupportKind::None => return,
        };
        match result {
            Ok(()) => {
                self.rgb_settings.brightness = brightness;
                self.autosave_rgb_settings();
            }
            Err(e) => {
                self.status_msg = format!("Failed to update RGB brightness: {}", e);
                log::warn!("set_rgb_brightness failed: {e}");
            }
        }
    }

    fn set_rgb_color(&mut self, hue: u8, saturation: u8) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let result = match self.rgb_settings.kind {
            RgbSupportKind::QmkRgblight => hid.set_qmk_rgblight_color(hue, saturation),
            RgbSupportKind::VialRgb => hid.set_vialrgb_mode(
                self.rgb_settings.effect,
                self.rgb_settings.speed,
                hue,
                saturation,
                self.rgb_settings.brightness,
            ),
            RgbSupportKind::None => return,
        };
        match result {
            Ok(()) => {
                self.rgb_settings.hue = hue;
                self.rgb_settings.saturation = saturation;
                self.autosave_rgb_settings();
            }
            Err(e) => {
                self.status_msg = format!("Failed to update RGB color: {}", e);
                log::warn!("set_rgb_color failed: {e}");
            }
        }
    }

    fn set_rgb_speed(&mut self, speed: u8) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let result = match self.rgb_settings.kind {
            RgbSupportKind::QmkRgblight => hid.set_qmk_rgblight_effect_speed(speed),
            RgbSupportKind::VialRgb => hid.set_vialrgb_mode(
                self.rgb_settings.effect,
                speed,
                self.rgb_settings.hue,
                self.rgb_settings.saturation,
                self.rgb_settings.brightness,
            ),
            RgbSupportKind::None => return,
        };
        match result {
            Ok(()) => {
                self.rgb_settings.speed = speed;
                self.autosave_rgb_settings();
            }
            Err(e) => {
                self.status_msg = format!("Failed to update RGB speed: {}", e);
                log::warn!("set_rgb_speed failed: {e}");
            }
        }
    }

    fn autosave_rgb_settings(&mut self) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        if let Err(e) = hid.save_rgb() {
            self.status_msg = format!("Failed to save RGB settings: {}", e);
            log::warn!("save_rgb failed: {e}");
        }
    }

    fn draw_rgb_editor_content(&mut self, ui: &mut egui::Ui, dark: bool, layout: &RgbModalLayout) {
        let options = rgb_effect_options(&self.rgb_settings);
        let mut enabled = self.rgb_settings.is_enabled();
        let mut selected_effect = self.rgb_settings.effect;
        let brightness_max = self.rgb_settings.max_brightness.max(1);
        let current_percent = ((self.rgb_settings.brightness as f32 / brightness_max as f32)
            * 100.0)
            .round()
            .clamp(0.0, 100.0);
        let mut brightness_percent = current_percent;
        let selected_effect_name = options
            .iter()
            .find(|(id, _)| *id == self.rgb_settings.effect)
            .map(|(_, name)| *name)
            .unwrap_or("Unknown");
        let speed_max = 255.0_f32;
        let mut speed_percent = ((self.rgb_settings.speed as f32 / speed_max) * 100.0)
            .round()
            .clamp(0.0, 100.0);
        let mut color_hsva = egui::ecolor::Hsva {
            h: self.rgb_settings.hue as f32 / 255.0,
            s: self.rgb_settings.saturation as f32 / 255.0,
            v: 1.0,
            a: 1.0,
        };

        crate::ui_style::modal_content(ui, layout.modal_layout(), |ui| {
            let content_width = layout.content_width;
            let scale = (layout.row_height / 54.0).clamp(1.0, 1.12);
            let rgb_value_width = 36.0 * scale;
            let rgb_slider_width = 160.0 * scale;
            let rgb_slider_size = egui::vec2(168.0 * scale, 24.0 * scale);
            let rgb_control_width = rgb_slider_size.x + rgb_value_width;
            let rgb_control_height = 32.0 * scale;
            let rgb_font_size = 12.5 * scale;

            crate::ui_style::settings_list_row(
                ui,
                content_width,
                layout.row_height,
                crate::i18n::tr_catalog(self.app_settings.language, "common.enable"),
                true,
                46.0 * scale,
                |ui| {
                    let enable_resp = crate::ui_style::settings_switch_sized(
                        ui,
                        &mut enabled,
                        egui::vec2(46.0 * scale, 24.0 * scale),
                    );
                    if enable_resp.changed() {
                        let next_effect = if enabled {
                            self.rgb_settings.effect_or_default()
                        } else {
                            if self.rgb_settings.effect != 0 {
                                self.rgb_settings.last_enabled_effect = self.rgb_settings.effect;
                            }
                            0
                        };
                        self.set_rgb_effect(next_effect);
                        selected_effect = self.rgb_settings.effect;
                    }
                },
            );

            crate::ui_style::settings_list_row(
                ui,
                content_width,
                layout.row_height,
                crate::i18n::tr_catalog(self.app_settings.language, "common.effect"),
                true,
                rgb_slider_size.x,
                |ui| {
                    let dropdown_id = ui.make_persistent_id("rgb_effect_dropdown");
                    let dropdown_resp = crate::ui_style::modern_dropdown_button_sized(
                        ui,
                        dropdown_id,
                        selected_effect_name,
                        ui.visuals().text_color(),
                        rgb_slider_size.x,
                        rgb_control_height,
                        rgb_font_size,
                    );

                    egui::popup_below_widget(
                        ui,
                        dropdown_id,
                        &dropdown_resp,
                        egui::PopupCloseBehavior::CloseOnClickOutside,
                        |ui| {
                            ui.set_min_width(rgb_slider_size.x);
                            ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
                            egui::ScrollArea::vertical()
                                .id_salt("rgb_effect_dropdown_scroll")
                                .max_height(142.0 * scale)
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    for (id, label) in &options {
                                        let selected = *id == selected_effect;
                                        let (option_rect, option_resp) = ui.allocate_exact_size(
                                            Vec2::new(rgb_slider_size.x, 28.0 * scale),
                                            Sense::click(),
                                        );
                                        if option_resp.hovered() {
                                            ui.ctx()
                                                .set_cursor_icon(egui::CursorIcon::PointingHand);
                                        }
                                        let option_fill = if selected {
                                            if dark {
                                                Color32::from_rgb(58, 58, 61)
                                            } else {
                                                Color32::from_rgb(236, 236, 238)
                                            }
                                        } else if option_resp.hovered() {
                                            crate::ui_style::hover_fill(dark)
                                        } else {
                                            Color32::TRANSPARENT
                                        };
                                        ui.painter().rect_filled(option_rect, 7.0, option_fill);
                                        ui.painter().text(
                                            egui::pos2(
                                                option_rect.left() + 10.0,
                                                option_rect.center().y,
                                            ),
                                            egui::Align2::LEFT_CENTER,
                                            *label,
                                            FontId::proportional(12.0 * scale),
                                            if selected {
                                                ui.visuals().text_color()
                                            } else {
                                                app_muted_text(dark)
                                            },
                                        );
                                        if option_resp.clicked() {
                                            selected_effect = *id;
                                            self.set_rgb_effect(selected_effect);
                                            ui.memory_mut(|m| m.close_popup());
                                        }
                                    }
                                });
                        },
                    );
                },
            );

            let color_enabled = rgb_effect_supports_color(self.rgb_settings.kind, selected_effect);
            crate::ui_style::settings_list_row(
                ui,
                content_width,
                layout.color_row_height,
                crate::i18n::tr_catalog(self.app_settings.language, "common.color"),
                color_enabled,
                64.0 * scale,
                |ui| {
                    let popup_id = ui.make_persistent_id("rgb_color_popup");
                    let popup_hsva_id = popup_id.with("hsva");
                    let popup_open = ui.memory(|m| m.is_popup_open(popup_id));
                    let border = if dark {
                        Color32::from_gray(95)
                    } else {
                        Color32::from_gray(185)
                    };
                    let swatch_border = if color_enabled && popup_open {
                        app_accent()
                    } else {
                        border
                    };
                    let swatch_color: Color32 = color_hsva.into();
                    let swatch_sense = if color_enabled {
                        Sense::click()
                    } else {
                        Sense::hover()
                    };
                    let (swatch_rect, swatch_resp) =
                        ui.allocate_exact_size(Vec2::new(64.0 * scale, 34.0 * scale), swatch_sense);
                    if color_enabled && swatch_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if color_enabled && swatch_resp.clicked() {
                        ui.ctx()
                            .data_mut(|d| d.insert_temp(popup_hsva_id, color_hsva));
                        ui.memory_mut(|m| m.toggle_popup(popup_id));
                    }
                    ui.painter().rect(
                        swatch_rect,
                        9.0,
                        app_surface_fill(dark),
                        Stroke::new(1.0, swatch_border),
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().rect(
                        swatch_rect.shrink(5.0 * scale),
                        6.0,
                        if color_enabled {
                            swatch_color
                        } else {
                            swatch_color.gamma_multiply(0.45)
                        },
                        Stroke::new(1.0, swatch_border.gamma_multiply(0.85)),
                        egui::StrokeKind::Inside,
                    );

                    if color_enabled {
                        let mut picked_hsva = ui
                            .ctx()
                            .data(|d| d.get_temp::<egui::ecolor::Hsva>(popup_hsva_id))
                            .unwrap_or(color_hsva);
                        egui::popup_below_widget(
                            ui,
                            popup_id,
                            &swatch_resp,
                            egui::PopupCloseBehavior::CloseOnClickOutside,
                            |ui| {
                                ui.spacing_mut().slider_width = 136.0 * scale;
                                if compact_rgb_color_picker(ui, &mut picked_hsva) {
                                    let hue = (picked_hsva.h.rem_euclid(1.0) * 255.0)
                                        .round()
                                        .clamp(0.0, 255.0)
                                        as u8;
                                    let saturation = (picked_hsva.s.clamp(0.0, 1.0) * 255.0)
                                        .round()
                                        .clamp(0.0, 255.0)
                                        as u8;
                                    self.set_rgb_color(hue, saturation);
                                    color_hsva = picked_hsva;
                                    ui.ctx()
                                        .data_mut(|d| d.insert_temp(popup_hsva_id, picked_hsva));
                                }
                            },
                        );
                    }
                },
            );

            let speed_enabled = rgb_effect_supports_speed(self.rgb_settings.kind, selected_effect);
            let rgb_slider_fill: Color32 = Color32::from(color_hsva).gamma_multiply(0.5);
            crate::ui_style::settings_list_row(
                ui,
                content_width,
                layout.row_height,
                crate::i18n::tr_catalog(self.app_settings.language, "common.speed"),
                speed_enabled,
                rgb_control_width,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let value_color = if speed_enabled {
                        if dark {
                            Color32::from_gray(230)
                        } else {
                            Color32::from_gray(55)
                        }
                    } else {
                        app_muted_text(dark)
                    };
                    ui.visuals_mut().selection.bg_fill = if speed_enabled {
                        rgb_slider_fill
                    } else {
                        rgb_slider_fill.gamma_multiply(0.5)
                    };
                    ui.visuals_mut().widgets.active.bg_fill = rgb_slider_fill;
                    ui.visuals_mut().widgets.active.weak_bg_fill = rgb_slider_fill;
                    ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::new(1.0, rgb_slider_fill);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_sized(
                            [rgb_value_width, layout.row_height],
                            egui::Label::new(
                                RichText::new(format!("{}%", speed_percent as u8))
                                    .size(12.0 * scale)
                                    .color(value_color),
                            )
                            .halign(egui::Align::RIGHT),
                        );
                        ui.add_enabled_ui(speed_enabled, |ui| {
                            ui.spacing_mut().slider_width = rgb_slider_width;
                            let slider = egui::Slider::new(&mut speed_percent, 0.0..=100.0)
                                .step_by(1.0)
                                .show_value(false)
                                .trailing_fill(true);
                            let resp = ui.add_sized(rgb_slider_size, slider);
                            if resp.changed() {
                                let raw_value = ((speed_percent / 100.0) * speed_max)
                                    .round()
                                    .clamp(0.0, speed_max)
                                    as u8;
                                self.set_rgb_speed(raw_value);
                            }
                        });
                    });
                },
            );

            let brightness_enabled = enabled;
            crate::ui_style::settings_list_row(
                ui,
                content_width,
                layout.row_height,
                crate::i18n::tr_catalog(self.app_settings.language, "common.brightness"),
                brightness_enabled,
                rgb_control_width,
                |ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let value_color = if brightness_enabled {
                        if dark {
                            Color32::from_gray(230)
                        } else {
                            Color32::from_gray(55)
                        }
                    } else {
                        app_muted_text(dark)
                    };
                    ui.visuals_mut().selection.bg_fill = if brightness_enabled {
                        rgb_slider_fill
                    } else {
                        rgb_slider_fill.gamma_multiply(0.5)
                    };
                    ui.visuals_mut().widgets.active.bg_fill = rgb_slider_fill;
                    ui.visuals_mut().widgets.active.weak_bg_fill = rgb_slider_fill;
                    ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::new(1.0, rgb_slider_fill);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_sized(
                            [rgb_value_width, layout.row_height],
                            egui::Label::new(
                                RichText::new(format!("{}%", brightness_percent as u8))
                                    .size(12.0 * scale)
                                    .color(value_color),
                            )
                            .halign(egui::Align::RIGHT),
                        );
                        ui.add_enabled_ui(brightness_enabled, |ui| {
                            ui.spacing_mut().slider_width = rgb_slider_width;
                            let slider = egui::Slider::new(&mut brightness_percent, 0.0..=100.0)
                                .step_by(1.0)
                                .show_value(false)
                                .trailing_fill(true);
                            let resp = ui.add_sized(rgb_slider_size, slider);
                            if resp.changed() {
                                let raw_value = ((brightness_percent / 100.0)
                                    * brightness_max as f32)
                                    .round()
                                    .clamp(0.0, brightness_max as f32)
                                    as u8;
                                self.set_rgb_brightness(raw_value);
                            }
                        });
                    });
                },
            );
        });
    }

    fn draw_alt_repeat_editor_content(&mut self, ui: &mut egui::Ui) {
        let dark = ui.visuals().dark_mode;
        if self.alt_repeat_entries.is_empty() {
            crate::ui_style::modal_empty_state(
                ui,
                crate::i18n::tr(
                    self.app_settings.language,
                    crate::i18n::Key::AltRepeatUnavailable,
                ),
                None,
            );
            return;
        }

        if self.selected_alt_repeat >= self.alt_repeat_entries.len() {
            self.selected_alt_repeat = 0;
        }
        self.selected_alt_repeat = self
            .selected_alt_repeat
            .min(self.alt_repeat_entries.len().saturating_sub(1));
        self.alt_repeat_names
            .resize(self.alt_repeat_entries.len(), String::new());

        let idx = self.selected_alt_repeat;
        let current = self.alt_repeat_entries[idx].clone();
        let mut edited = current.clone();
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        let content_width = metrics.settings_content_width();
        let row_height = metrics.settings_row_height();
        const TOTAL_ROWS: usize = 11;
        let row_content_width = metrics.settings_row_content_width();
        let control_width = metrics.settings_control_width();
        let mod_control_width = metrics.value(210.0);
        let switch_size = metrics.size(46.0, 24.0);
        let custom = self
            .layout
            .as_ref()
            .map(|l| l.custom_keycodes.as_slice())
            .unwrap_or(&[]);
        let last_key_label = if edited.keycode == 0 {
            crate::i18n::tr_catalog(self.app_settings.language, "key_picker_text.pick_key")
                .to_string()
        } else {
            keycode_label_with_macro_names(
                edited.keycode,
                custom,
                &self.layer_names,
                &self.keycode_picker.macro_names,
                &self.keycode_picker.tap_dance_names,
                self.app_settings.key_legend_layout,
            )
            .replace("\n", " ")
        };
        let alt_key_label = if edited.alt_keycode == 0 {
            crate::i18n::tr_catalog(self.app_settings.language, "key_picker_text.pick_key")
                .to_string()
        } else {
            keycode_label_with_macro_names(
                edited.alt_keycode,
                custom,
                &self.layer_names,
                &self.keycode_picker.macro_names,
                &self.keycode_picker.tap_dance_names,
                self.app_settings.key_legend_layout,
            )
            .replace("\n", " ")
        };
        let last_key_tip = keycode_tooltip_with_macro_names(
            edited.keycode,
            custom,
            &self.layer_names,
            &self.keycode_picker.macro_names,
            &self.keycode_picker.tap_dance_names,
        );
        let alt_key_tip = keycode_tooltip_with_macro_names(
            edited.alt_keycode,
            custom,
            &self.layer_names,
            &self.keycode_picker.macro_names,
            &self.keycode_picker.tap_dance_names,
        );
        let selected_empty = !Self::alt_repeat_entry_exists(&edited)
            && self
                .alt_repeat_names
                .get(idx)
                .map(|name| name.trim().is_empty())
                .unwrap_or(true);
        let selected_text = match self.alt_repeat_names.get(idx) {
            Some(name) if !name.trim().is_empty() => format!("AR{}: {}", idx, name.trim()),
            _ => format!("AR{}", idx),
        };
        let selected_text_color = if selected_empty {
            app_inactive_entry_text(dark)
        } else {
            ui.visuals().text_color()
        };
        let gui = crate::keycode::gui_mod_name();

        crate::ui_style::modal_content(
            ui,
            crate::ui_style::ModalLayout::new(content_width).with_top_padding(4.0),
            |ui| {
                let list = allocate_adaptive_settings_list_viewport(
                    ui,
                    "alt_repeat_settings",
                    metrics,
                    TOTAL_ROWS,
                    metrics.value(86.0),
                );
                ui.allocate_ui_at_rect(list.content_rect, |ui| {
                    ui.set_clip_rect(list.viewport);
                    ui.set_min_size(list.content_rect.size());
                    ui.spacing_mut().item_spacing.y = 0.0;
                    for row_idx in list.first_visible_row..list.last_visible_row {
                        match row_idx {
                            0 => {
                                crate::ui_style::settings_list_row_with_tooltip(
                                    ui,
                                    row_content_width,
                                    row_height,
                                    crate::i18n::tr_catalog(
                                        self.app_settings.language,
                                        "alt_repeat_editor.entry",
                                    ),
                                    true,
                                    Some(crate::i18n::tr_catalog(
                                        self.app_settings.language,
                                        "alt_repeat_editor.select_alt_repeat_slot",
                                    )),
                                    control_width,
                                    |ui| {
                                        let dropdown_id =
                                            ui.make_persistent_id("alt_repeat_entry_dropdown");
                                        let dropdown_resp =
                                            crate::ui_style::modern_dropdown_button_sized(
                                                ui,
                                                dropdown_id,
                                                selected_text.as_str(),
                                                selected_text_color,
                                                control_width,
                                                metrics.settings_control_height(),
                                                metrics.settings_control_font_size(),
                                            );

                                        ui.style_mut().visuals.window_stroke =
                                            crate::ui_style::modal_outline_stroke(dark);
                                        ui.style_mut().visuals.window_fill = app_surface_fill(dark);
                                        egui::popup_below_widget(
                                            ui,
                                            dropdown_id,
                                            &dropdown_resp,
                                            egui::PopupCloseBehavior::CloseOnClickOutside,
                                            |ui| {
                                                ui.set_min_width(control_width);
                                                ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
                                                egui::ScrollArea::vertical()
                                                    .id_salt("alt_repeat_entry_dropdown_scroll")
                                                    .max_height(metrics.value(142.0))
                                                    .auto_shrink([false, true])
                                                    .show(ui, |ui| {
                                                        for entry_idx in
                                                            0..self.alt_repeat_entries.len()
                                                        {
                                                            let empty = self
                                                                .alt_repeat_entries
                                                                .get(entry_idx)
                                                                .map(|entry| {
                                                                    !Self::alt_repeat_entry_exists(
                                                                        entry,
                                                                    )
                                                                })
                                                                .unwrap_or(true)
                                                                && self
                                                                    .alt_repeat_names
                                                                    .get(entry_idx)
                                                                    .map(|name| {
                                                                        name.trim().is_empty()
                                                                    })
                                                                    .unwrap_or(true);
                                                            let option_text = match self
                                                                .alt_repeat_names
                                                                .get(entry_idx)
                                                            {
                                                                Some(name)
                                                                    if !name.trim().is_empty() =>
                                                                {
                                                                    format!(
                                                                        "AR{}: {}",
                                                                        entry_idx,
                                                                        name.trim()
                                                                    )
                                                                }
                                                                _ => format!("AR{}", entry_idx),
                                                            };
                                                            let selected = entry_idx
                                                                == self.selected_alt_repeat;
                                                            let (option_rect, option_resp) = ui
                                                                .allocate_exact_size(
                                                                    metrics.size(168.0, 28.0),
                                                                    Sense::click(),
                                                                );
                                                            if option_resp.hovered() {
                                                                ui.ctx().set_cursor_icon(
                                                                    egui::CursorIcon::PointingHand,
                                                                );
                                                            }
                                                            let option_fill = if selected {
                                                                if dark {
                                                                    Color32::from_rgb(58, 58, 61)
                                                                } else {
                                                                    Color32::from_rgb(236, 236, 238)
                                                                }
                                                            } else if option_resp.hovered() {
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
                                                                    option_rect.left()
                                                                        + metrics.value(10.0),
                                                                    option_rect.center().y,
                                                                ),
                                                                egui::Align2::LEFT_CENTER,
                                                                option_text,
                                                                FontId::proportional(
                                                                    metrics.value(12.0),
                                                                ),
                                                                if selected {
                                                                    ui.visuals().text_color()
                                                                } else if empty {
                                                                    app_inactive_entry_text(
                                                                        ui.visuals().dark_mode,
                                                                    )
                                                                } else {
                                                                    app_muted_text(
                                                                        ui.visuals().dark_mode,
                                                                    )
                                                                },
                                                            );
                                                            if option_resp.clicked() {
                                                                self.selected_alt_repeat =
                                                                    entry_idx;
                                                                ui.memory_mut(|m| m.close_popup());
                                                            }
                                                        }
                                                    });
                                            },
                                        );
                                    },
                                );
                            }
                            1 => {
                                let mut name_changed = false;
                                crate::ui_style::settings_list_row_with_tooltip(
                                    ui,
                                    row_content_width,
                                    row_height,
                                    crate::i18n::tr_catalog(
                                        self.app_settings.language,
                                        "alt_repeat_editor.name",
                                    ),
                                    true,
                                    Some(crate::i18n::tr_catalog(
                                        self.app_settings.language,
                                        "alt_repeat_editor.local_name_for_this_slot",
                                    )),
                                    control_width,
                                    |ui| {
                                        if let Some(name) = self.alt_repeat_names.get_mut(idx) {
                                            let resp = crate::ui_style::modern_text_field_sized(
                                                ui,
                                                egui::Id::new(("alt_repeat_name", idx)),
                                                name,
                                                control_width,
                                                metrics.settings_control_height(),
                                                crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.name"),
                                                12,
                                                egui::Align::Center,
                                            );
                                            name_changed = resp.changed();
                                            resp.clone().on_hover_text(crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.stored_locally_in_entropy"));
                                        }
                                    },
                                );
                                if name_changed {
                                    self.push_alt_repeat_undo();
                                    save_alt_repeat_names(
                                        &self.alt_repeat_names,
                                        &self.current_device_name,
                                    );
                                }
                            }
                            2 => {
                                crate::ui_style::settings_list_row_with_tooltip(
                                    ui,
                                    row_content_width,
                                    row_height,
                                    crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.last_key"),
                                    true,
                                    Some(crate::i18n::tr_catalog(
                                        self.app_settings.language,
                                        "alt_repeat_editor.key_that_triggers_alternate_repeat_behavior",
                                    )),
                                    control_width,
                                    |ui| {
                                        let resp = crate::ui_style::modern_button_with_font(
                                            ui,
                                            last_key_label.as_str(),
                                            metrics.size(168.0, 32.0),
                                            metrics.settings_control_font_size(),
                                            true,
                                        );
                                        if resp.clicked() {
                                            self.open_alt_repeat_picker(
                                                AltRepeatPickField::LastKey,
                                            );
                                        }
                                        resp.on_hover_text(crate::i18n::tr_text(
                                            self.app_settings.language,
                                            last_key_tip.trim_end_matches('.'),
                                        ));
                                    },
                                );
                            }
                            3 => {
                                crate::ui_style::settings_list_row_with_tooltip(
                                    ui,
                                    row_content_width,
                                    row_height,
                                    crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.alt_key"),
                                    true,
                                    Some(crate::i18n::tr_catalog(
                                        self.app_settings.language,
                                        "alt_repeat_editor.key_repeated_when_alternate_repeat_activates",
                                    )),
                                    control_width,
                                    |ui| {
                                        let resp = crate::ui_style::modern_button_with_font(
                                            ui,
                                            alt_key_label.as_str(),
                                            metrics.size(168.0, 32.0),
                                            metrics.settings_control_font_size(),
                                            true,
                                        );
                                        if resp.clicked() {
                                            self.open_alt_repeat_picker(AltRepeatPickField::AltKey);
                                        }
                                        resp.on_hover_text(crate::i18n::tr_text(
                                            self.app_settings.language,
                                            alt_key_tip.trim_end_matches('.'),
                                        ));
                                    },
                                );
                            }
                            4..=7 => {
                                let (row_label, left_bit, right_bit) = match row_idx {
                                    4 => (
                                        crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.ctrl_mods")
                                        .to_string(),
                                        0,
                                        4,
                                    ),
                                    5 => (
                                        crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.shift_mods")
                                        .to_string(),
                                        1,
                                        5,
                                    ),
                                    6 => (
                                        crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.alt_mods")
                                        .to_string(),
                                        2,
                                        6,
                                    ),
                                    _ => (
                                        if matches!(
                                            self.app_settings.language,
                                            crate::i18n::Language::Russian
                                        ) {
                                            format!("{gui}-моды")
                                        } else {
                                            format!("{} mods", gui)
                                        },
                                        3,
                                        7,
                                    ),
                                };
                                crate::ui_style::settings_list_row_with_tooltip(
                                    ui,
                                    row_content_width,
                                    row_height,
                                    row_label.as_str(),
                                    true,
                                    Some(match row_idx {
                                        4 => crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.allowed_ctrl_modifiers"),
                                        5 => crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.allowed_shift_modifiers"),
                                        6 => crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.allowed_alt_modifiers"),
                                        _ => crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.allowed_os_modifiers"),
                                    }),
                                    mod_control_width,
                                    |ui| {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                let mut right_checked =
                                                    (edited.allowed_mods & (1 << right_bit)) != 0;
                                                let right_resp =
                                                    crate::ui_style::settings_switch_sized(
                                                        ui,
                                                        &mut right_checked,
                                                        switch_size,
                                                    );
                                                if right_resp.changed() {
                                                    if right_checked {
                                                        edited.allowed_mods |= 1 << right_bit;
                                                    } else {
                                                        edited.allowed_mods &= !(1 << right_bit);
                                                    }
                                                }
                                                let right_label_resp = ui.label(
                                                    RichText::new("R")
                                                        .size(metrics.value(12.0))
                                                        .color(app_muted_text(
                                                            ui.visuals().dark_mode,
                                                        )),
                                                );
                                                if right_label_resp.hovered() {
                                                    ui.ctx()
                                                        .set_cursor_icon(egui::CursorIcon::Help);
                                                }
                                                right_label_resp.on_hover_text(
                                                    crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.right_side_modifier"),
                                                );
                                                ui.add_space(metrics.value(10.0));
                                                let mut left_checked =
                                                    (edited.allowed_mods & (1 << left_bit)) != 0;
                                                let left_resp =
                                                    crate::ui_style::settings_switch_sized(
                                                        ui,
                                                        &mut left_checked,
                                                        switch_size,
                                                    );
                                                if left_resp.changed() {
                                                    if left_checked {
                                                        edited.allowed_mods |= 1 << left_bit;
                                                    } else {
                                                        edited.allowed_mods &= !(1 << left_bit);
                                                    }
                                                }
                                                let left_label_resp = ui.label(
                                                    RichText::new("L")
                                                        .size(metrics.value(12.0))
                                                        .color(app_muted_text(
                                                            ui.visuals().dark_mode,
                                                        )),
                                                );
                                                if left_label_resp.hovered() {
                                                    ui.ctx()
                                                        .set_cursor_icon(egui::CursorIcon::Help);
                                                }
                                                left_label_resp.on_hover_text(
                                                    crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.left_side_modifier"),
                                                );
                                            },
                                        );
                                    },
                                );
                            }
                            8 => {
                                crate::ui_style::settings_list_row_with_tooltip(
                                    ui,
                                    row_content_width,
                                    row_height,
                                    crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.default_alt_key"),
                                    true,
                                    Some(crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.use_this_alt_key_by_default")),
                                    metrics.value(46.0),
                                    |ui| {
                                        crate::ui_style::settings_switch_sized(
                                            ui,
                                            &mut edited.options.default_to_this_alt_key,
                                            switch_size,
                                        );
                                    },
                                );
                            }
                            9 => {
                                crate::ui_style::settings_list_row_with_tooltip(
                                    ui,
                                    row_content_width,
                                    row_height,
                                    crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.bidirectional"),
                                    true,
                                    Some(crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.allow_both_keys_to_alternate_each_other")),
                                    metrics.value(46.0),
                                    |ui| {
                                        crate::ui_style::settings_switch_sized(
                                            ui,
                                            &mut edited.options.bidirectional,
                                            switch_size,
                                        );
                                    },
                                );
                            }
                            10 => {
                                crate::ui_style::settings_list_row_with_tooltip(
                                    ui,
                                    row_content_width,
                                    row_height,
                                    crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.ignore_handedness"),
                                    true,
                                    Some(crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.treat_left_and_right_modifiers_as_equivalent")),
                                    metrics.value(46.0),
                                    |ui| {
                                        crate::ui_style::settings_switch_sized(
                                            ui,
                                            &mut edited.options.ignore_mod_handedness,
                                            switch_size,
                                        );
                                    },
                                );
                            }
                            _ => {}
                        }
                    }
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

                let action_size = metrics.size(104.0, 32.0);
                let action_width = action_size.x * 2.0 + ui.spacing().item_spacing.x;
                let actions_rect = egui::Rect::from_min_size(
                    egui::pos2(list.viewport.left(), list.viewport.bottom() + 24.0),
                    egui::vec2(content_width, action_size.y),
                );
                ui.allocate_ui_at_rect(actions_rect, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(((content_width - action_width) / 2.0).max(0.0));
                        let clear_enabled =
                            Self::alt_repeat_entry_exists(&self.alt_repeat_entries[idx])
                                || self
                                    .alt_repeat_names
                                    .get(idx)
                                    .map(|s| !s.trim().is_empty())
                                    .unwrap_or(false);
                        let clear_resp = crate::ui_style::modern_button_with_font(
                            ui,
                            crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "alt_repeat_editor.clear",
                            ),
                            action_size,
                            metrics.settings_control_font_size(),
                            clear_enabled,
                        );
                        if clear_resp.clicked() {
                            self.push_alt_repeat_undo();
                            self.alt_repeat_entries[idx] = AltRepeatKeyEntry::default();
                            if let Some(name) = self.alt_repeat_names.get_mut(idx) {
                                name.clear();
                            }
                            save_alt_repeat_names(
                                &self.alt_repeat_names,
                                &self.current_device_name,
                            );
                            self.write_alt_repeat_entry(idx);
                            edited = self.alt_repeat_entries[idx].clone();
                        }

                        let undo_enabled = !self.alt_repeat_undo_stack.is_empty();
                        let undo_resp = crate::ui_style::modern_button_with_font(
                            ui,
                            crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "alt_repeat_editor.undo",
                            ),
                            action_size,
                            metrics.settings_control_font_size(),
                            undo_enabled,
                        );
                        if undo_resp.clicked() {
                            if let Some((entries, names, selected)) =
                                self.alt_repeat_undo_stack.pop()
                            {
                                self.alt_repeat_entries = entries;
                                self.alt_repeat_names = names;
                                self.selected_alt_repeat =
                                    selected.min(self.alt_repeat_entries.len().saturating_sub(1));
                                save_alt_repeat_names(
                                    &self.alt_repeat_names,
                                    &self.current_device_name,
                                );
                                for entry_idx in 0..self.alt_repeat_entries.len() {
                                    self.write_alt_repeat_entry(entry_idx);
                                }
                            }
                        }
                    });
                });
            },
        );

        Self::normalize_alt_repeat_entry(&mut edited);
        if edited != current {
            self.push_alt_repeat_undo();
            if let Some(slot) = self.alt_repeat_entries.get_mut(idx) {
                *slot = edited;
            }
            self.write_alt_repeat_entry(idx);
        }
    }

    fn draw_layer_led_settings_page(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect) {
        let lang = self.app_settings.language;
        let dark = ui.visuals().dark_mode;
        let hid_ready = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.hid_device.is_some()
            }
            #[cfg(target_arch = "wasm32")]
            {
                false
            }
        };

        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(18.0);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::LayerLedsTitle))
                        .size(18.0)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(crate::i18n::tr(
                        lang,
                        crate::i18n::Key::LayerLedsDescription,
                    ))
                    .size(13.0)
                    .color(app_muted_text(dark)),
                );
                ui.add_space(24.0);

                if !self.layer_led_settings.supported {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::LayerLedsUnavailable),
                        Some(crate::i18n::tr(lang, crate::i18n::Key::LayerLedsEnableHint)),
                    );
                    return;
                }

                if !hid_ready {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::LayerLedsConnect),
                        None,
                    );
                    return;
                }

                const TOTAL_ROWS: usize = 18;
                let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
                let list = allocate_adaptive_settings_list_viewport(
                    ui,
                    "layer_led_settings",
                    metrics,
                    TOTAL_ROWS,
                    0.0,
                );
                ui.allocate_ui_at_rect(list.content_rect, |ui| {
                    ui.set_clip_rect(list.viewport);
                    ui.set_min_size(list.content_rect.size());
                    ui.spacing_mut().item_spacing.y = 0.0;
                    self.draw_layer_led_editor_content(
                        ui,
                        list.first_visible_row..list.last_visible_row,
                        list.row_content_width,
                        list.row_height,
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
            });
        });
    }

    fn draw_layer_led_editor_content(
        &mut self,
        ui: &mut egui::Ui,
        row_range: std::ops::Range<usize>,
        content_width: f32,
        row_height: f32,
        suppress_tooltips: bool,
    ) {
        let scale = (row_height / 54.0).clamp(1.0, 1.12);
        let slider_width = 168.0 * scale;
        let value_width = 36.0 * scale;
        let slider_size = [slider_width, 18.0 * scale];
        let slider_control_width = slider_width + value_width;
        let swatch_width = 64.0 * scale;
        let swatch_size = Vec2::new(64.0 * scale, 34.0 * scale);

        for row_idx in row_range {
            match row_idx {
                0 => {
                    let brightness_max = 255.0_f32;
                    let mut value = (self.layer_led_settings.brightness as f32 / brightness_max
                        * 100.0)
                        .round()
                        .clamp(0.0, 100.0);
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        crate::i18n::tr_catalog(
                            self.app_settings.language,
                            "advanced_settings.led_brightness",
                        ),
                        true,
                        if suppress_tooltips {
                            None
                        } else {
                            Some(crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "advanced_settings.global_led_brightness_for_layer_color_lighting",
                            ))
                        },
                        slider_control_width,
                        |ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            let dark = ui.visuals().dark_mode;
                            let slider_fill = app_accent().gamma_multiply(0.5);
                            ui.visuals_mut().selection.bg_fill = slider_fill;
                            ui.visuals_mut().widgets.active.bg_fill = slider_fill;
                            ui.visuals_mut().widgets.active.weak_bg_fill = slider_fill;
                            ui.visuals_mut().widgets.hovered.bg_stroke =
                                Stroke::new(1.0, slider_fill);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.add_sized(
                                        [value_width, row_height],
                                        egui::Label::new(
                                            RichText::new(format!("{}%", value.round() as u8))
                                                .size(12.0 * scale)
                                                .color(if dark {
                                                    Color32::from_gray(230)
                                                } else {
                                                    Color32::from_gray(55)
                                                }),
                                        )
                                        .halign(egui::Align::RIGHT),
                                    );
                                    ui.spacing_mut().slider_width = slider_width;
                                    let slider = egui::Slider::new(&mut value, 0.0..=100.0)
                                        .step_by(1.0)
                                        .show_value(false)
                                        .trailing_fill(true);
                                    let resp = ui.add_sized(slider_size, slider);
                                    if resp.changed() {
                                        let new_value = ((value / 100.0) * brightness_max)
                                            .round()
                                            .clamp(0.0, brightness_max)
                                            as u16;
                                        if new_value != self.layer_led_settings.brightness {
                                            self.layer_led_settings.brightness = new_value;
                                            self.write_layer_led_brightness(new_value);
                                        }
                                    }
                                },
                            );
                        },
                    );
                }
                1 => {
                    let mut value = self.layer_led_settings.timeout_mins as f32;
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        crate::i18n::tr_catalog(
                            self.app_settings.language,
                            "advanced_settings.led_timeout",
                        ),
                        true,
                        if suppress_tooltips {
                            None
                        } else {
                            Some(crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "advanced_settings.minutes_before_leds_turn_off_automatically_0_disables_timeout",
                            ))
                        },
                        slider_control_width,
                        |ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            let dark = ui.visuals().dark_mode;
                            let slider_fill = if dark {
                                Color32::from_rgb(92, 92, 96)
                            } else {
                                Color32::from_rgb(190, 184, 182)
                            };
                            ui.visuals_mut().selection.bg_fill = slider_fill;
                            ui.visuals_mut().widgets.active.bg_fill = slider_fill;
                            ui.visuals_mut().widgets.active.weak_bg_fill = slider_fill;
                            ui.visuals_mut().widgets.hovered.bg_stroke =
                                Stroke::new(1.0, slider_fill);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let value_text = if value.round() as u8 == 0 {
                                        crate::i18n::tr_catalog(
                                            self.app_settings.language,
                                            "advanced_settings.off",
                                        )
                                        .to_string()
                                    } else {
                                        format!("{}m", value.round() as u8)
                                    };
                                    ui.add_sized(
                                        [value_width, row_height],
                                        egui::Label::new(
                                            RichText::new(value_text).size(12.0 * scale).color(
                                                if dark {
                                                    Color32::from_gray(230)
                                                } else {
                                                    Color32::from_gray(55)
                                                },
                                            ),
                                        )
                                        .halign(egui::Align::RIGHT),
                                    );
                                    ui.spacing_mut().slider_width = slider_width;
                                    let slider = egui::Slider::new(&mut value, 0.0..=255.0)
                                        .step_by(1.0)
                                        .show_value(false)
                                        .trailing_fill(true);
                                    let resp = ui.add_sized(slider_size, slider);
                                    let resp = settings_field_unit_tooltip(
                                        resp,
                                        self.app_settings.language,
                                        suppress_tooltips,
                                        SettingsFieldUnit::Minutes,
                                    );
                                    if resp.changed() {
                                        let new_value = value.round().clamp(0.0, 255.0) as u8;
                                        if new_value != self.layer_led_settings.timeout_mins {
                                            self.layer_led_settings.timeout_mins = new_value;
                                            self.write_layer_led_timeout(new_value);
                                        }
                                    }
                                },
                            );
                        },
                    );
                }
                2..=17 => {
                    let layer = row_idx - 2;
                    let current = self.layer_led_settings.layer_colors[layer];
                    let layer_name = self
                        .layer_names
                        .get(layer)
                        .map(|name| name.trim())
                        .filter(|name| !name.is_empty() && *name != layer.to_string())
                        .map(|name| {
                            let visible: String = name.chars().take(22).collect();
                            if matches!(self.app_settings.language, crate::i18n::Language::Russian)
                            {
                                format!("Слой {layer}: {visible}")
                            } else {
                                format!("Layer {layer}: {visible}")
                            }
                        })
                        .unwrap_or_else(|| {
                            if matches!(self.app_settings.language, crate::i18n::Language::Russian)
                            {
                                format!("Цвет слоя {layer}")
                            } else {
                                format!("Layer {layer} color")
                            }
                        });
                    let label = layer_name;
                    let tooltip =
                        if matches!(self.app_settings.language, crate::i18n::Language::Russian) {
                            format!("Цвет подсветки, когда активен слой {layer}")
                        } else {
                            format!("LED palette color used when layer {layer} is active")
                        };
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        &label,
                        true,
                        if suppress_tooltips {
                            None
                        } else {
                            Some(tooltip.as_str())
                        },
                        swatch_width,
                        |ui| {
                            let dark = ui.visuals().dark_mode;
                            let popup_id = ui.make_persistent_id(("layer_led_color_picker", layer));
                            let popup_open = ui.memory(|m| m.is_popup_open(popup_id));
                            let swatch_color = layer_led_palette_color(current);
                            let swatch_border = if popup_open {
                                app_accent()
                            } else if dark {
                                Color32::from_gray(95)
                            } else {
                                Color32::from_gray(185)
                            };
                            let (swatch_rect, swatch_resp) =
                                ui.allocate_exact_size(swatch_size, Sense::click());
                            if swatch_resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if swatch_resp.clicked() {
                                ui.memory_mut(|m| m.toggle_popup(popup_id));
                            }
                            ui.painter().rect(
                                swatch_rect,
                                9.0,
                                app_surface_fill(dark),
                                Stroke::new(1.0, swatch_border),
                                egui::StrokeKind::Inside,
                            );
                            ui.painter().rect(
                                swatch_rect.shrink(5.0 * scale),
                                6.0,
                                swatch_color,
                                Stroke::new(1.0, swatch_border.gamma_multiply(0.85)),
                                egui::StrokeKind::Inside,
                            );
                            if current == 0 {
                                ui.painter().line_segment(
                                    [
                                        swatch_rect.left_top()
                                            + egui::vec2(10.0 * scale, 10.0 * scale),
                                        swatch_rect.right_bottom()
                                            - egui::vec2(10.0 * scale, 10.0 * scale),
                                    ],
                                    Stroke::new(1.2, app_muted_text(dark)),
                                );
                            }
                            swatch_resp.clone().on_hover_text(crate::i18n::tr_catalog(
                                self.app_settings.language,
                                layer_led_palette_name(current),
                            ));

                            ui.style_mut().visuals.window_stroke =
                                crate::ui_style::modal_outline_stroke(dark);
                            ui.style_mut().visuals.window_fill = app_surface_fill(dark);
                            egui::popup_below_widget(
                                ui,
                                popup_id,
                                &swatch_resp,
                                egui::PopupCloseBehavior::CloseOnClickOutside,
                                |ui| {
                                    let cell = 28.0 * scale;
                                    let gap = 6.0 * scale;
                                    const COLS: usize = 5;
                                    let picker_width = cell * COLS as f32 + gap * (COLS - 1) as f32;
                                    ui.set_min_width(picker_width);
                                    ui.spacing_mut().item_spacing = Vec2::new(gap, gap);
                                    for row in 0..5 {
                                        ui.horizontal(|ui| {
                                            for col in 0..COLS {
                                                let color_idx = row * COLS + col;
                                                let Some(option_label) =
                                                    LAYER_LED_PALETTE.get(color_idx)
                                                else {
                                                    continue;
                                                };
                                                let color_idx_u8 = color_idx as u8;
                                                let selected = color_idx_u8 == current;
                                                let (cell_rect, cell_resp) = ui
                                                    .allocate_exact_size(
                                                        Vec2::splat(cell),
                                                        Sense::click(),
                                                    );
                                                if cell_resp.hovered() {
                                                    ui.ctx().set_cursor_icon(
                                                        egui::CursorIcon::PointingHand,
                                                    );
                                                }
                                                let outline = if selected {
                                                    app_accent()
                                                } else if dark {
                                                    Color32::from_rgb(72, 72, 76)
                                                } else {
                                                    Color32::from_rgb(210, 210, 214)
                                                };
                                                ui.painter().rect(
                                                    cell_rect,
                                                    7.0,
                                                    app_surface_fill(dark),
                                                    Stroke::new(
                                                        if selected { 1.6 } else { 1.0 },
                                                        outline,
                                                    ),
                                                    egui::StrokeKind::Inside,
                                                );
                                                ui.painter().rect(
                                                    cell_rect.shrink(4.5 * scale),
                                                    5.0,
                                                    layer_led_palette_color(color_idx_u8),
                                                    Stroke::NONE,
                                                    egui::StrokeKind::Inside,
                                                );
                                                if color_idx == 0 {
                                                    ui.painter().line_segment(
                                                        [
                                                            cell_rect.left_top()
                                                                + egui::vec2(
                                                                    8.0 * scale,
                                                                    8.0 * scale,
                                                                ),
                                                            cell_rect.right_bottom()
                                                                - egui::vec2(
                                                                    8.0 * scale,
                                                                    8.0 * scale,
                                                                ),
                                                        ],
                                                        Stroke::new(1.1, app_muted_text(dark)),
                                                    );
                                                }
                                                cell_resp.clone().on_hover_text(
                                                    crate::i18n::tr_text(
                                                        self.app_settings.language,
                                                        option_label,
                                                    ),
                                                );
                                                if cell_resp.clicked() {
                                                    self.layer_led_settings.layer_colors[layer] =
                                                        color_idx_u8;
                                                    self.write_layer_led_color(layer, color_idx_u8);
                                                    ui.memory_mut(|m| m.close_popup());
                                                }
                                            }
                                        });
                                    }
                                },
                            );
                        },
                    );
                }
                _ => {}
            }
        }
    }

    fn write_layer_led_color(&mut self, layer: usize, value: u8) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let qsid = 300 + layer as u16;
        if let Err(e) = hid.set_qmk_setting_u8(qsid, value) {
            self.status_msg = format!("Failed to save Layer LED color (layer {layer}): {}", e);
            log::warn!("set_qmk_setting_u8(layer_led qsid {qsid}) failed: {e}");
        }
    }

    fn write_layer_led_brightness(&mut self, value: u16) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let value = value.min(255);
        if let Err(e) = hid.set_qmk_setting_u16(316, value) {
            self.status_msg = format!("Failed to save Layer LED brightness: {}", e);
            log::warn!("set_qmk_setting_u16(layer_led brightness) failed: {e}");
        }
    }

    fn write_layer_led_timeout(&mut self, value: u8) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        if let Err(e) = hid.set_qmk_setting_u8(317, value) {
            self.status_msg = format!("Failed to save Layer LED timeout: {}", e);
            log::warn!("set_qmk_setting_u8(layer_led timeout) failed: {e}");
        }
    }

    fn draw_grave_escape_settings_page(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect) {
        let lang = self.app_settings.language;
        let dark = ui.visuals().dark_mode;
        let hid_ready = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.hid_device.is_some()
            }
            #[cfg(target_arch = "wasm32")]
            {
                false
            }
        };

        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(18.0);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::GraveEscapeTitle))
                        .size(18.0)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(crate::i18n::tr(
                        lang,
                        crate::i18n::Key::GraveEscapeDescription,
                    ))
                    .size(13.0)
                    .color(app_muted_text(dark)),
                );
                ui.add_space(24.0);

                if !self.grave_escape_settings.supported {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::GraveEscapeUnavailable),
                        Some(crate::i18n::tr(
                            lang,
                            crate::i18n::Key::GraveEscapeEnableHint,
                        )),
                    );
                    return;
                }

                if !hid_ready {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::GraveEscapeConnect),
                        None,
                    );
                    return;
                }

                let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
                let content_width = metrics.settings_content_width();
                let row_height = metrics.settings_row_height();
                let row_content_width = metrics.settings_row_content_width();
                let switch_width = metrics.value(46.0);
                let switch_size = metrics.size(46.0, 24.0);
                let gui_name = crate::keycode::gui_mod_name();
                let rows: Vec<(u8, String, String)> = vec![
                    (
                        0,
                        crate::i18n::tr_catalog(self.app_settings.language, "advanced_settings.alt_forces_esc")
                            .to_string(),
                        crate::i18n::tr_catalog(self.app_settings.language,
                            "advanced_settings.when_alt_is_held_grave_escape_sends_esc_instead_of_grave_or_tilde",
                        )
                        .to_string(),
                    ),
                    (
                        1,
                        crate::i18n::tr_catalog(self.app_settings.language, "advanced_settings.control_forces_esc")
                            .to_string(),
                        crate::i18n::tr_catalog(self.app_settings.language,
                            "advanced_settings.when_control_is_held_grave_escape_sends_esc_instead_of_grave_or_tilde",
                        )
                        .to_string(),
                    ),
                    (
                        2,
                        if matches!(self.app_settings.language, crate::i18n::Language::Russian) {
                            format!("{gui_name} отправляет Esc")
                        } else {
                            format!("{gui_name} forces Esc")
                        },
                        if matches!(self.app_settings.language, crate::i18n::Language::Russian) {
                            format!("При удержании {gui_name} Grave Escape отправляет Esc вместо ` или ~")
                        } else {
                            format!(
                                "When {gui_name} is held, Grave Escape sends Esc instead of ` or ~"
                            )
                        },
                    ),
                    (
                        3,
                        crate::i18n::tr_catalog(self.app_settings.language, "advanced_settings.shift_forces_esc")
                            .to_string(),
                        crate::i18n::tr_catalog(self.app_settings.language,
                            "advanced_settings.when_shift_is_held_grave_escape_sends_esc_instead_of_grave_or_tilde",
                        )
                        .to_string(),
                    ),
                ];

                crate::ui_style::modal_content(
                    ui,
                    crate::ui_style::ModalLayout::new(content_width).with_top_padding(0.0),
                    |ui| {
                        for (bit, label, tooltip) in rows {
                            let mut value = self.grave_escape_settings.bit(bit);
                            crate::ui_style::settings_list_row_with_tooltip(
                                ui,
                                row_content_width,
                                row_height,
                                &label,
                                true,
                                Some(&tooltip),
                                switch_width,
                                |ui| {
                                    let resp = crate::ui_style::settings_switch_sized(
                                        ui,
                                        &mut value,
                                        switch_size,
                                    );
                                    if resp.changed() {
                                        self.grave_escape_settings.set_bit(bit, value);
                                        self.write_grave_escape_settings();
                                    }
                                },
                            );
                        }
                    },
                );
            });
        });
    }

    fn write_grave_escape_settings(&mut self) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        if let Err(e) = hid.set_qmk_setting_u8(1, self.grave_escape_settings.bits) {
            self.status_msg = format!("Failed to save Grave Escape settings: {}", e);
            log::warn!("set_qmk_setting_u8(grave_escape qsid 1) failed: {e}");
        }
    }

    fn draw_magic_settings_page(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect) {
        let lang = self.app_settings.language;
        let dark = ui.visuals().dark_mode;
        let hid_ready = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.hid_device.is_some()
            }
            #[cfg(target_arch = "wasm32")]
            {
                false
            }
        };

        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(18.0);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::MagicTitle))
                        .size(18.0)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::MagicDescription))
                        .size(13.0)
                        .color(app_muted_text(dark)),
                );
                ui.add_space(24.0);

                if !self.magic_settings.supported {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::MagicUnavailable),
                        Some(crate::i18n::tr(lang, crate::i18n::Key::MagicEnableHint)),
                    );
                    return;
                }

                if !hid_ready {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::MagicConnect),
                        None,
                    );
                    return;
                }

                const TOTAL_ROWS: usize = 10;
                let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
                let list = allocate_adaptive_settings_list_viewport(
                    ui,
                    "magic_settings",
                    metrics,
                    TOTAL_ROWS,
                    0.0,
                );
                ui.allocate_ui_at_rect(list.content_rect, |ui| {
                    ui.set_clip_rect(list.viewport);
                    ui.set_min_size(list.content_rect.size());
                    ui.spacing_mut().item_spacing.y = 0.0;
                    self.draw_magic_editor_content(
                        ui,
                        list.first_visible_row..list.last_visible_row,
                        list.row_content_width,
                        list.row_height,
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
            });
        });
    }

    fn draw_magic_editor_content(
        &mut self,
        ui: &mut egui::Ui,
        row_range: std::ops::Range<usize>,
        content_width: f32,
        row_height: f32,
        suppress_tooltips: bool,
    ) {
        let switch_width = (row_height / 54.0).clamp(1.0, 1.12) * 46.0;
        let switch_size = egui::vec2(switch_width, (row_height / 54.0).clamp(1.0, 1.12) * 24.0);
        let gui = crate::keycode::gui_mod_name();
        for row_idx in row_range {
            let (bit, label, tooltip) = match row_idx {
                0 => (
                    0,
                    crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "advanced_settings.swap_caps_lock_and_left_control",
                    )
                    .to_owned(),
                    crate::i18n::tr_catalog(self.app_settings.language, "advanced_settings.caps_lock_sends_left_control_and_left_control_sends_caps_lock")
                    .to_owned(),
                ),
                1 => (
                    1,
                    crate::i18n::tr_catalog(self.app_settings.language, "advanced_settings.treat_caps_lock_as_control")
                    .to_owned(),
                    crate::i18n::tr_catalog(self.app_settings.language, "advanced_settings.caps_lock_sends_control_without_swapping_left_control")
                    .to_owned(),
                ),
                2 => (
                    2,
                    if matches!(self.app_settings.language, crate::i18n::Language::Russian) {
                        format!("Поменять Left Alt и {gui}")
                    } else {
                        format!("Swap Left Alt and {gui}")
                    },
                    if matches!(self.app_settings.language, crate::i18n::Language::Russian) {
                        format!("Left Alt отправляет {gui}, а Left {gui} — Alt")
                    } else {
                        format!("Left Alt sends {gui} and Left {gui} sends Alt")
                    },
                ),
                3 => (
                    3,
                    if matches!(self.app_settings.language, crate::i18n::Language::Russian) {
                        format!("Поменять Right Alt и {gui}")
                    } else {
                        format!("Swap Right Alt and {gui}")
                    },
                    if matches!(self.app_settings.language, crate::i18n::Language::Russian) {
                        format!("Right Alt отправляет {gui}, а Right {gui} — Alt")
                    } else {
                        format!("Right Alt sends {gui} and Right {gui} sends Alt")
                    },
                ),
                4 => (
                    4,
                    if matches!(self.app_settings.language, crate::i18n::Language::Russian) {
                        format!("Отключить клавиши {gui}")
                    } else {
                        format!("Disable {gui} keys")
                    },
                    if matches!(self.app_settings.language, crate::i18n::Language::Russian) {
                        format!("Игнорировать обе клавиши {gui}, пока опция включена")
                    } else {
                        format!("Ignore both {gui} keys while this option is enabled")
                    },
                ),
                5 => (
                    5,
                    crate::i18n::tr_catalog(self.app_settings.language, "advanced_settings.swap_grave_and_escape")
                    .to_owned(),
                    crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "advanced_settings.grave_sends_escape_and_escape_sends_grave",
                    )
                    .to_owned(),
                ),
                6 => (
                    6,
                    crate::i18n::tr_catalog(self.app_settings.language, "advanced_settings.swap_backslash_and_backspace")
                    .to_owned(),
                    crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "advanced_settings.backslash_sends_backspace_and_backspace_sends_backslash",
                    )
                    .to_owned(),
                ),
                7 => (
                    7,
                    crate::i18n::tr_catalog(self.app_settings.language, "advanced_settings.enable_n_key_rollover")
                    .to_owned(),
                    crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "advanced_settings.allow_more_simultaneous_key_presses_when_the_keyboard_supports_it",
                    )
                    .to_owned(),
                ),
                8 => (
                    8,
                    if matches!(self.app_settings.language, crate::i18n::Language::Russian) {
                        format!("Поменять Left Control и {gui}")
                    } else {
                        format!("Swap Left Control and {gui}")
                    },
                    if matches!(self.app_settings.language, crate::i18n::Language::Russian) {
                        format!("Left Control отправляет {gui}, а Left {gui} — Control")
                    } else {
                        format!("Left Control sends {gui} and Left {gui} sends Control")
                    },
                ),
                9 => (
                    9,
                    if matches!(self.app_settings.language, crate::i18n::Language::Russian) {
                        format!("Поменять Right Control и {gui}")
                    } else {
                        format!("Swap Right Control and {gui}")
                    },
                    if matches!(self.app_settings.language, crate::i18n::Language::Russian) {
                        format!("Right Control отправляет {gui}, а Right {gui} — Control")
                    } else {
                        format!("Right Control sends {gui} and Right {gui} sends Control")
                    },
                ),
                _ => continue,
            };
            let mut value = self.magic_settings.bit(bit);
            crate::ui_style::settings_list_row_with_tooltip(
                ui,
                content_width,
                row_height,
                label.as_str(),
                true,
                if suppress_tooltips {
                    None
                } else {
                    Some(tooltip.as_str())
                },
                switch_width,
                |ui| {
                    let resp = crate::ui_style::settings_switch_sized_stable(
                        ui,
                        ("magic_settings", bit),
                        &mut value,
                        switch_size,
                    );
                    if resp.changed() {
                        self.magic_settings.set_bit(bit, value);
                        self.write_magic_settings();
                    }
                },
            );
        }
    }

    fn write_magic_settings(&mut self) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        if let Err(e) = hid.set_qmk_setting_u16(21, self.magic_settings.bits) {
            self.status_msg = format!("Failed to save Magic settings: {}", e);
            log::warn!("set_qmk_setting_u16(magic qsid 21) failed: {e}");
        }
    }

    fn draw_tap_hold_settings_page(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect) {
        let lang = self.app_settings.language;
        let dark = ui.visuals().dark_mode;
        let hid_ready = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.hid_device.is_some()
            }
            #[cfg(target_arch = "wasm32")]
            {
                false
            }
        };

        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(18.0);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::TapHoldOneShotTitle))
                        .size(18.0)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(crate::i18n::tr(
                        lang,
                        crate::i18n::Key::TapHoldOneShotDescription,
                    ))
                    .size(13.0)
                    .color(app_muted_text(dark)),
                );
                ui.add_space(24.0);

                if !self.tap_hold_settings.supported && !self.one_shot_settings.supported {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::TapHoldOneShotUnavailable),
                        Some(crate::i18n::tr(
                            lang,
                            crate::i18n::Key::QmkSettingsEnableHint,
                        )),
                    );
                    return;
                }

                if !hid_ready {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::TapHoldOneShotConnect),
                        None,
                    );
                    return;
                }

                let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
                let total_rows = self.tap_hold_one_shot_row_count();
                let list = allocate_adaptive_settings_list_viewport(
                    ui,
                    "tap_hold_settings",
                    metrics,
                    total_rows,
                    0.0,
                );
                ui.allocate_ui_at_rect(list.content_rect, |ui| {
                    ui.set_clip_rect(list.viewport);
                    ui.set_min_size(list.content_rect.size());
                    ui.spacing_mut().item_spacing.y = 0.0;
                    self.draw_tap_hold_editor_content(
                        ui,
                        list.first_visible_row..list.last_visible_row,
                        list.row_content_width,
                        list.row_height,
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
            });
        });
    }

    fn tap_hold_one_shot_row_count(&self) -> usize {
        self.tap_hold_settings.supported as usize * 10
            + self.one_shot_settings.supported as usize * 2
            + (self.tap_hold_settings.supported && self.one_shot_settings.supported) as usize
    }

    fn draw_tap_hold_editor_content(
        &mut self,
        ui: &mut egui::Ui,
        row_range: std::ops::Range<usize>,
        content_width: f32,
        row_height: f32,
        suppress_tooltips: bool,
    ) {
        #[derive(Clone, Copy)]
        enum SettingsRowKind {
            TapHold,
            OneShot,
        }

        enum SettingsRow {
            Section(&'static str),
            Setting {
                kind: SettingsRowKind,
                qsid: u16,
                label: &'static str,
                tooltip: &'static str,
                is_bool: bool,
                max: u32,
            },
        }

        let mut rows: Vec<SettingsRow> = Vec::with_capacity(13);
        if self.tap_hold_settings.supported {
            rows.extend([
                SettingsRow::Setting {
                    kind: SettingsRowKind::TapHold,
                    qsid: 7,
                    label: crate::i18n::tr_catalog(self.app_settings.language, "tap_hold_settings.tapping_term_label"),
                    tooltip: crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "tap_hold_settings.global_tap_vs_hold_decision_window_for_dual_role_keys",
                    ),
                    is_bool: false,
                    max: 10000,
                },
                SettingsRow::Setting {
                    kind: SettingsRowKind::TapHold,
                    qsid: 22,
                    label: crate::i18n::tr_catalog(self.app_settings.language, "tap_hold_settings.permissive_hold"),
                    tooltip: crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "tap_hold_settings.nested_taps_choose_hold_for_mod_tap_and_layer_tap_keys",
                    ),
                    is_bool: true,
                    max: 1,
                },
                SettingsRow::Setting {
                    kind: SettingsRowKind::TapHold,
                    qsid: 23,
                    label: crate::i18n::tr_catalog(self.app_settings.language, "tap_hold_settings.hold_on_other_key"),
                    tooltip: crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "tap_hold_settings.pressing_another_key_immediately_chooses_hold_for_dual_role_keys",
                    ),
                    is_bool: true,
                    max: 1,
                },
                SettingsRow::Setting {
                    kind: SettingsRowKind::TapHold,
                    qsid: 24,
                    label: crate::i18n::tr_catalog(self.app_settings.language, "tap_hold_settings.retro_tapping"),
                    tooltip: crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "tap_hold_settings.a_held_and_released_alone_dual_role_key_still_sends_its_tap_action",
                    ),
                    is_bool: true,
                    max: 1,
                },
                SettingsRow::Setting {
                    kind: SettingsRowKind::TapHold,
                    qsid: 26,
                    label: crate::i18n::tr_catalog(self.app_settings.language, "tap_hold_settings.chordal_hold"),
                    tooltip: crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "tap_hold_settings.same_hand_chords_prefer_tap_to_reduce_home_row_mod_accidents",
                    ),
                    is_bool: true,
                    max: 1,
                },
                SettingsRow::Setting {
                    kind: SettingsRowKind::TapHold,
                    qsid: 25,
                    label: crate::i18n::tr_catalog(self.app_settings.language, "tap_hold_settings.quick_tap_term"),
                    tooltip: crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "tap_hold_settings.tap_then_hold_repeat_window_for_dual_role_key_tap_actions",
                    ),
                    is_bool: false,
                    max: 10000,
                },
                SettingsRow::Setting {
                    kind: SettingsRowKind::TapHold,
                    qsid: 18,
                    label: crate::i18n::tr_catalog(self.app_settings.language, "tap_hold_settings.tap_code_delay"),
                    tooltip: crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "tap_hold_settings.delay_between_register_and_unregister_in_tap_code",
                    ),
                    is_bool: false,
                    max: 1000,
                },
                SettingsRow::Setting {
                    kind: SettingsRowKind::TapHold,
                    qsid: 19,
                    label: crate::i18n::tr_catalog(self.app_settings.language, "tap_hold_settings.tap_hold_caps_delay"),
                    tooltip: crate::i18n::tr_catalog(self.app_settings.language, "tap_hold_settings.extra_delay_for_lt_mt_keys_whose_tap_action_is_caps_lock"),
                    is_bool: false,
                    max: 1000,
                },
                SettingsRow::Setting {
                    kind: SettingsRowKind::TapHold,
                    qsid: 20,
                    label: crate::i18n::tr_catalog(self.app_settings.language, "tap_hold_settings.tapping_toggle"),
                    tooltip: crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "tap_hold_settings.number_of_taps_needed_for_tt_layer_toggle",
                    ),
                    is_bool: false,
                    max: 100,
                },
                SettingsRow::Setting {
                    kind: SettingsRowKind::TapHold,
                    qsid: 27,
                    label: crate::i18n::tr_catalog(self.app_settings.language, "tap_hold_settings.flow_tap"),
                    tooltip: crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "tap_hold_settings.fast_typing_timeout_that_forces_mt_lt_keys_to_tap",
                    ),
                    is_bool: false,
                    max: 10000,
                },
            ]);
        }
        if self.one_shot_settings.supported {
            if self.tap_hold_settings.supported {
                rows.push(SettingsRow::Section(crate::i18n::tr_catalog(
                    self.app_settings.language,
                    "tap_hold_settings.one_shot_keys",
                )));
            }
            rows.extend([
                SettingsRow::Setting {
                    kind: SettingsRowKind::OneShot,
                    qsid: 5,
                    label: crate::i18n::tr_catalog(self.app_settings.language, "tap_hold_settings.one_shot_tap_toggle"),
                    tooltip: crate::i18n::tr_catalog(self.app_settings.language, "tap_hold_settings.tap_this_many_times_to_keep_a_one_shot_key_held_until_tapped_again"),
                    is_bool: false,
                    max: 50,
                },
                SettingsRow::Setting {
                    kind: SettingsRowKind::OneShot,
                    qsid: 6,
                    label: crate::i18n::tr_catalog(self.app_settings.language, "tap_hold_settings.one_shot_timeout"),
                    tooltip: crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "tap_hold_settings.how_long_one_shot_state_waits_before_it_is_released",
                    ),
                    is_bool: false,
                    max: 60000,
                },
            ]);
        }
        let scale = (row_height / 54.0).clamp(1.0, 1.12);
        let field_width = 86.0 * scale;
        let switch_width = 46.0 * scale;
        let switch_size = egui::vec2(46.0 * scale, 24.0 * scale);
        let control_height = 32.0 * scale;

        for row_idx in row_range {
            let Some(row) = rows.get(row_idx) else {
                continue;
            };
            let SettingsRow::Setting {
                kind,
                qsid,
                label,
                tooltip,
                is_bool,
                max,
            } = row
            else {
                if let SettingsRow::Section(title) = row {
                    self.draw_tap_hold_section_divider(ui, content_width, row_height, title);
                }
                continue;
            };
            let kind = *kind;
            let qsid = *qsid;
            let is_bool = *is_bool;
            let max = *max;
            if is_bool {
                let mut value = self.tap_hold_bool_value(qsid);
                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    content_width,
                    row_height,
                    label,
                    true,
                    if suppress_tooltips {
                        None
                    } else {
                        Some(tooltip)
                    },
                    switch_width,
                    |ui| {
                        let resp = crate::ui_style::settings_switch_sized_stable(
                            ui,
                            ("tap_hold_settings", qsid),
                            &mut value,
                            switch_size,
                        );
                        if resp.changed() {
                            self.set_tap_hold_bool_value(qsid, value);
                            self.write_tap_hold_bool_setting(qsid, value);
                        }
                    },
                );
            } else {
                let current = match kind {
                    SettingsRowKind::TapHold => self.tap_hold_numeric_value(qsid),
                    SettingsRowKind::OneShot => self.one_shot_numeric_value(qsid),
                };
                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    content_width,
                    row_height,
                    label,
                    true,
                    if suppress_tooltips {
                        None
                    } else {
                        Some(tooltip)
                    },
                    field_width,
                    |ui| {
                        let edit_id = egui::Id::new((
                            match kind {
                                SettingsRowKind::TapHold => "tap_hold_edit",
                                SettingsRowKind::OneShot => "one_shot_edit",
                            },
                            qsid,
                        ));
                        let mut text = ui.ctx().data_mut(|d| {
                            d.get_temp::<String>(edit_id)
                                .unwrap_or_else(|| current.to_string())
                        });
                        if text.parse::<u16>().ok() != Some(current)
                            && !ui.memory(|m| m.has_focus(edit_id))
                        {
                            text = current.to_string();
                        }
                        let resp = crate::ui_style::modern_text_field_sized(
                            ui,
                            edit_id,
                            &mut text,
                            field_width,
                            control_height,
                            "",
                            5,
                            egui::Align::RIGHT,
                        );
                        let resp = match (kind, qsid) {
                            (SettingsRowKind::TapHold, 7 | 25 | 18 | 19 | 27)
                            | (SettingsRowKind::OneShot, 6) => settings_field_unit_tooltip(
                                resp,
                                self.app_settings.language,
                                suppress_tooltips,
                                SettingsFieldUnit::Milliseconds,
                            ),
                            _ => resp,
                        };
                        if resp.changed() {
                            let filtered: String =
                                text.chars().filter(|c: &char| c.is_ascii_digit()).collect();
                            let parsed = filtered.parse::<u32>().unwrap_or(0).min(max);
                            let new_value = parsed as u16;
                            if new_value != current {
                                match kind {
                                    SettingsRowKind::TapHold => {
                                        self.set_tap_hold_numeric_value(qsid, new_value);
                                        self.write_tap_hold_numeric_setting(qsid, new_value);
                                    }
                                    SettingsRowKind::OneShot => {
                                        self.set_one_shot_numeric_value(qsid, new_value);
                                        self.write_one_shot_numeric_setting(qsid, new_value);
                                    }
                                }
                            }
                            text = filtered;
                        }
                        ui.ctx().data_mut(|d| d.insert_temp(edit_id, text));
                    },
                );
            }
        }
    }

    fn draw_tap_hold_section_divider(
        &self,
        ui: &mut egui::Ui,
        content_width: f32,
        row_height: f32,
        title: &str,
    ) {
        let dark = ui.visuals().dark_mode;
        let (row_rect, _) =
            ui.allocate_exact_size(egui::vec2(content_width, row_height), egui::Sense::hover());
        let separator =
            crate::ui_style::border_color(dark).gamma_multiply(if dark { 0.72 } else { 0.9 });
        ui.painter().line_segment(
            [row_rect.left_bottom(), row_rect.right_bottom()],
            egui::Stroke::new(1.0, separator),
        );
        ui.painter().text(
            row_rect.center(),
            egui::Align2::CENTER_CENTER,
            title,
            egui::FontId::proportional(12.5),
            app_muted_text(dark),
        );
    }

    fn one_shot_numeric_value(&self, qsid: u16) -> u16 {
        match qsid {
            5 => self.one_shot_settings.tap_toggle as u16,
            6 => self.one_shot_settings.timeout,
            _ => 0,
        }
    }

    fn set_one_shot_numeric_value(&mut self, qsid: u16, value: u16) {
        match qsid {
            5 => self.one_shot_settings.tap_toggle = value.min(u8::MAX as u16) as u8,
            6 => self.one_shot_settings.timeout = value,
            _ => {}
        }
    }

    fn write_one_shot_numeric_setting(&mut self, qsid: u16, value: u16) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let result = if qsid == 5 {
            hid.set_qmk_setting_u8(qsid, value.min(u8::MAX as u16) as u8)
        } else {
            hid.set_qmk_setting_u16(qsid, value)
        };
        if let Err(e) = result {
            self.status_msg = format!("Failed to save One Shot setting (qsid {qsid}): {}", e);
            log::warn!("set_qmk_setting(one_shot qsid {qsid}) failed: {e}");
        }
    }

    fn tap_hold_numeric_value(&self, qsid: u16) -> u16 {
        match qsid {
            7 => self.tap_hold_settings.tapping_term,
            25 => self.tap_hold_settings.quick_tap_term,
            18 => self.tap_hold_settings.tap_code_delay,
            19 => self.tap_hold_settings.tap_hold_caps_delay,
            20 => self.tap_hold_settings.tapping_toggle,
            27 => self.tap_hold_settings.flow_tap,
            _ => 0,
        }
    }

    fn set_tap_hold_numeric_value(&mut self, qsid: u16, value: u16) {
        match qsid {
            7 => self.tap_hold_settings.tapping_term = value,
            25 => self.tap_hold_settings.quick_tap_term = value,
            18 => self.tap_hold_settings.tap_code_delay = value,
            19 => self.tap_hold_settings.tap_hold_caps_delay = value,
            20 => self.tap_hold_settings.tapping_toggle = value,
            27 => self.tap_hold_settings.flow_tap = value,
            _ => {}
        }
    }

    fn tap_hold_bool_value(&self, qsid: u16) -> bool {
        match qsid {
            22 => self.tap_hold_settings.permissive_hold,
            23 => self.tap_hold_settings.hold_on_other_key_press,
            24 => self.tap_hold_settings.retro_tapping,
            26 => self.tap_hold_settings.chordal_hold,
            _ => false,
        }
    }

    fn set_tap_hold_bool_value(&mut self, qsid: u16, value: bool) {
        match qsid {
            22 => self.tap_hold_settings.permissive_hold = value,
            23 => self.tap_hold_settings.hold_on_other_key_press = value,
            24 => self.tap_hold_settings.retro_tapping = value,
            26 => self.tap_hold_settings.chordal_hold = value,
            _ => {}
        }
    }

    fn write_tap_hold_numeric_setting(&mut self, qsid: u16, value: u16) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let result = if qsid == 20 {
            hid.set_qmk_setting_u8(qsid, value.min(u8::MAX as u16) as u8)
        } else {
            hid.set_qmk_setting_u16(qsid, value)
        };
        if let Err(e) = result {
            self.status_msg = format!("Failed to save Tap-Hold setting (qsid {qsid}): {}", e);
            log::warn!("set_qmk_setting(tap_hold qsid {qsid}) failed: {e}");
        }
    }

    fn write_tap_hold_bool_setting(&mut self, qsid: u16, value: bool) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        if let Err(e) = hid.set_qmk_setting_u8(qsid, u8::from(value)) {
            self.status_msg = format!("Failed to save Tap-Hold setting (qsid {qsid}): {}", e);
            log::warn!("set_qmk_setting_u8(tap_hold qsid {qsid}) failed: {e}");
        }
    }

    fn touchpad_numeric_value(&self, qsid: u16) -> u16 {
        match qsid {
            121 => self.touchpad_settings.sniper_sens as u16,
            122 => self.touchpad_settings.scroll_sens as u16,
            123 => self.touchpad_settings.text_sens as u16,
            _ => 0,
        }
    }

    fn set_touchpad_numeric_value(&mut self, qsid: u16, value: u16) {
        match qsid {
            121 => self.touchpad_settings.sniper_sens = value.min(u8::MAX as u16) as u8,
            122 => self.touchpad_settings.scroll_sens = value.min(u8::MAX as u16) as u8,
            123 => self.touchpad_settings.text_sens = value.min(u8::MAX as u16) as u8,
            _ => {}
        }
    }

    fn write_touchpad_numeric_setting(&mut self, qsid: u16, value: u16) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let value = value.clamp(1, 255) as u8;
        if let Err(e) = hid.set_qmk_setting_u8(qsid, value) {
            self.status_msg = format!("Failed to save Touchpad setting (qsid {qsid}): {}", e);
            log::warn!("set_qmk_setting_u8(touchpad qsid {qsid}) failed: {e}");
        }
    }

    fn write_touchpad_select_setting(&mut self, qsid: u16, value: u8) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        if let Err(e) = hid.set_qmk_setting_u8(qsid, value) {
            self.status_msg = format!("Failed to save Touchpad setting (qsid {qsid}): {}", e);
            log::warn!("set_qmk_setting_u8(touchpad qsid {qsid}) failed: {e}");
        }
    }

    fn write_touchpad_bool_setting(&mut self, qsid: u16, value: bool) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        if let Err(e) = hid.set_qmk_setting_u8(qsid, u8::from(value)) {
            self.status_msg = format!("Failed to save Touchpad setting (qsid {qsid}): {}", e);
            log::warn!("set_qmk_setting_u8(touchpad qsid {qsid}) failed: {e}");
        }
    }

    fn write_touchpad_bits(&mut self) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        if let Err(e) = hid.set_qmk_setting_u8(124, self.touchpad_settings.bits) {
            self.status_msg = format!("Failed to save Touchpad options: {}", e);
            log::warn!("set_qmk_setting_u8(touchpad qsid 124) failed: {e}");
        }
    }

    fn module_setting_label(&self, title: &str) -> String {
        crate::i18n::tr_text(self.app_settings.language, title)
    }

    fn module_setting_tooltip(&self, field: &ModuleSettingField) -> String {
        let lang = self.app_settings.language;
        let key = match field.title.as_str() {
            "Left mode" | "Right mode" => "modules_settings.mode_tooltip",
            "Left ball axis" | "Right ball axis" => "modules_settings.ball_axis_tooltip",
            "Left touch axis" | "Right touch axis" => "modules_settings.touch_axis_tooltip",
            "Left ball DPI" | "Right ball DPI" => "modules_settings.ball_dpi_tooltip",
            "Left touch DPI" | "Right touch DPI" => "modules_settings.touch_dpi_tooltip",
            "Left scroll sens" | "Right scroll sens" => "modules_settings.scroll_sens_tooltip",
            "Left sniper sens" | "Right sniper sens" => "modules_settings.sniper_sens_tooltip",
            "Left text sens" | "Right text sens" => "modules_settings.text_sens_tooltip",
            "Left invert scroll" | "Right invert scroll" => {
                "modules_settings.invert_scroll_tooltip"
            }
            "Left acceleration" | "Right acceleration" => "modules_settings.acceleration_tooltip",
            "Sticky mode" => "modules_settings.sticky_mode_tooltip",
            "LED blinks" => "modules_settings.led_blinks_tooltip",
            "Auto layer in Normal" => "modules_settings.auto_layer_normal_tooltip",
            "Auto layer" => "modules_settings.auto_layer_tooltip",
            "Auto layer in Sniper" => "modules_settings.auto_layer_sniper_tooltip",
            "Auto layer in Scroll" => "modules_settings.auto_layer_scroll_tooltip",
            "Auto layer in Text" => "modules_settings.auto_layer_text_tooltip",
            _ => "modules_settings.generic_tooltip",
        };
        let field_label = self.module_setting_label(&field.title);
        crate::i18n::tr_catalog_format(lang, key, &[("field", field_label.as_str())])
    }

    fn write_module_setting_value(&mut self, field: &ModuleSettingField, value: u16) {
        self.module_settings.set_value(field.qsid, value);
        let Some(hid) = &self.hid_device else {
            return;
        };
        let result = if field.width > 1 {
            hid.set_qmk_setting_u16(field.qsid, value)
        } else {
            hid.set_qmk_setting_u8(field.qsid, value.min(u8::MAX as u16) as u8)
        };
        if let Err(e) = result {
            self.status_msg = format!("Failed to save module setting (qsid {}): {}", field.qsid, e);
            log::warn!("set_qmk_setting(module qsid {}) failed: {e}", field.qsid);
        }
    }

    fn draw_module_settings_row(
        &mut self,
        ui: &mut egui::Ui,
        row_idx: usize,
        content_width: f32,
        row_height: f32,
        suppress_tooltips: bool,
    ) {
        let Some(field) = self.module_settings.fields.get(row_idx).cloned() else {
            return;
        };
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        let dark = ui.visuals().dark_mode;
        let label = self.module_setting_label(&field.title);
        let tooltip = if suppress_tooltips {
            None
        } else {
            Some(self.module_setting_tooltip(&field))
        };
        let raw_value = self.module_settings.value(field.qsid);
        match field.kind {
            ModuleSettingKind::Boolean => {
                let switch_width = metrics.value(46.0);
                let switch_size = metrics.size(46.0, 24.0);
                let mask = 1u16 << field.bit;
                let mut checked = raw_value & mask != 0;
                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    content_width,
                    row_height,
                    label.as_str(),
                    true,
                    tooltip.as_deref(),
                    switch_width,
                    |ui| {
                        let resp = crate::ui_style::settings_switch_sized_stable(
                            ui,
                            ("module_settings", field.qsid, field.bit),
                            &mut checked,
                            switch_size,
                        );
                        if resp.changed() {
                            let new_value = if checked {
                                raw_value | mask
                            } else {
                                raw_value & !mask
                            };
                            self.write_module_setting_value(&field, new_value);
                        }
                    },
                );
            }
            ModuleSettingKind::Integer => {
                let field_width = metrics.value(86.0);
                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    content_width,
                    row_height,
                    label.as_str(),
                    true,
                    tooltip.as_deref(),
                    field_width,
                    |ui| {
                        let edit_id = egui::Id::new(("module_setting_edit", field.qsid));
                        let current = raw_value.clamp(field.min, field.max);
                        let mut text = ui.ctx().data_mut(|d| {
                            d.get_temp::<String>(edit_id)
                                .unwrap_or_else(|| current.to_string())
                        });
                        if text.parse::<u16>().ok() != Some(current)
                            && !ui.memory(|m| m.has_focus(edit_id))
                        {
                            text = current.to_string();
                        }
                        let resp = crate::ui_style::modern_text_field_sized(
                            ui,
                            edit_id,
                            &mut text,
                            field_width,
                            metrics.settings_control_height(),
                            "",
                            5,
                            egui::Align::Center,
                        );
                        let commit = resp.lost_focus()
                            || (resp.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                        if commit {
                            match text.trim().parse::<u16>() {
                                Ok(value) => {
                                    let value = value.clamp(field.min, field.max);
                                    if value != raw_value {
                                        self.write_module_setting_value(&field, value);
                                    }
                                    text = value.to_string();
                                }
                                Err(_) => text = current.to_string(),
                            }
                        }
                        ui.ctx().data_mut(|d| d.insert_temp(edit_id, text));
                    },
                );
            }
            ModuleSettingKind::Select => {
                let dropdown_width = metrics.value(120.0);
                let selected_idx = (raw_value as usize).min(field.variants.len().saturating_sub(1));
                let variants = field
                    .variants
                    .iter()
                    .map(|variant| crate::i18n::tr_text(self.app_settings.language, variant))
                    .collect::<Vec<_>>();
                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    content_width,
                    row_height,
                    label.as_str(),
                    true,
                    tooltip.as_deref(),
                    dropdown_width,
                    |ui| {
                        let dropdown_id =
                            ui.make_persistent_id(("module_setting_dropdown", field.qsid));
                        let (_, picked) = Self::draw_touchpad_select_control(
                            ui,
                            dark,
                            dropdown_id,
                            selected_idx,
                            &variants,
                            dropdown_width,
                        );
                        if let Some(picked) = picked {
                            self.write_module_setting_value(&field, picked as u16);
                        }
                    },
                );
            }
        }
    }

    fn draw_module_settings_page(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect) {
        let lang = self.app_settings.language;
        let dark = ui.visuals().dark_mode;
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(18.0);
                ui.label(
                    RichText::new(crate::i18n::tr_catalog(lang, "modules_settings.title"))
                        .size(18.0)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(crate::i18n::tr_catalog(
                        lang,
                        "modules_settings.description",
                    ))
                    .size(13.0)
                    .color(app_muted_text(dark)),
                );
                ui.add_space(24.0);

                if !self.module_settings.supported {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr_catalog(lang, "modules_settings.unavailable"),
                        Some(crate::i18n::tr(
                            lang,
                            crate::i18n::Key::QmkSettingsEnableHint,
                        )),
                    );
                    return;
                }

                let list = allocate_adaptive_settings_list_viewport(
                    ui,
                    "module_settings",
                    metrics,
                    self.module_settings.row_count(),
                    0.0,
                );
                ui.allocate_ui_at_rect(list.content_rect, |ui| {
                    ui.set_clip_rect(list.viewport);
                    ui.set_min_size(list.content_rect.size());
                    ui.spacing_mut().item_spacing.y = 0.0;
                    for row_idx in list.first_visible_row..list.last_visible_row {
                        self.draw_module_settings_row(
                            ui,
                            row_idx,
                            list.row_content_width,
                            list.row_height,
                            list.suppress_tooltips,
                        );
                    }
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
            });
        });
    }

    fn draw_touchpad_select_control(
        ui: &mut egui::Ui,
        dark: bool,
        dropdown_id: egui::Id,
        selected_idx: usize,
        variants: &[String],
        width: f32,
    ) -> (egui::Response, Option<usize>) {
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        let selected_text = variants
            .get(selected_idx)
            .map(|s| s.as_str())
            .unwrap_or("Unknown");
        let dropdown_resp = crate::ui_style::modern_dropdown_button_sized(
            ui,
            dropdown_id,
            selected_text,
            ui.visuals().text_color(),
            width,
            metrics.settings_control_height(),
            metrics.settings_control_font_size(),
        );
        let mut picked = None;
        egui::popup_below_widget(
            ui,
            dropdown_id,
            &dropdown_resp,
            egui::PopupCloseBehavior::CloseOnClickOutside,
            |ui| {
                ui.set_min_width(width);
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
                egui::ScrollArea::vertical()
                    .id_salt(("touchpad_select_scroll", dropdown_id))
                    .max_height(metrics.value(142.0))
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for (idx, label) in variants.iter().enumerate() {
                            let selected = idx == selected_idx;
                            let (option_rect, option_resp) = ui.allocate_exact_size(
                                Vec2::new(width, metrics.value(28.0)),
                                Sense::click(),
                            );
                            if option_resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            let option_fill = if selected {
                                if dark {
                                    Color32::from_rgb(58, 58, 61)
                                } else {
                                    Color32::from_rgb(236, 236, 238)
                                }
                            } else if option_resp.hovered() {
                                crate::ui_style::hover_fill(dark)
                            } else {
                                Color32::TRANSPARENT
                            };
                            ui.painter().rect_filled(option_rect, 7.0, option_fill);
                            ui.painter().text(
                                egui::pos2(
                                    option_rect.left() + metrics.value(10.0),
                                    option_rect.center().y,
                                ),
                                egui::Align2::LEFT_CENTER,
                                label,
                                FontId::proportional(metrics.value(12.0)),
                                if selected {
                                    ui.visuals().text_color()
                                } else {
                                    app_muted_text(dark)
                                },
                            );
                            if option_resp.clicked() {
                                picked = Some(idx);
                                ui.memory_mut(|m| m.close_popup());
                            }
                        }
                    });
            },
        );
        (dropdown_resp, picked)
    }

    fn draw_touchpad_editor_content(
        &mut self,
        ui: &mut egui::Ui,
        row_range: std::ops::Range<usize>,
        metrics: crate::ui_style::ResponsiveMetrics,
        suppress_tooltips: bool,
    ) {
        let content_width = metrics.settings_row_content_width();
        let row_height = metrics.settings_row_height();
        let field_width = metrics.value(86.0);
        let switch_width = metrics.value(46.0);
        let switch_size = metrics.size(46.0, 24.0);
        let dropdown_width = metrics.value(120.0);
        let dark = ui.visuals().dark_mode;

        for row_idx in row_range {
            match row_idx {
                0 => {
                    let variants = self.touchpad_settings.dpi_variants.clone();
                    let selected_idx =
                        (self.touchpad_settings.dpi as usize).min(variants.len().saturating_sub(1));
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        crate::i18n::tr_catalog(
                            self.app_settings.language,
                            "touchpad_settings.dpi",
                        ),
                        true,
                        if suppress_tooltips {
                            None
                        } else {
                            Some(crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "touchpad_settings.touchpad_pointer_resolution_in_dots_per_inch",
                            ))
                        },
                        dropdown_width,
                        |ui| {
                            if variants.is_empty() {
                                let current = self.touchpad_settings.dpi;
                                let edit_id = egui::Id::new(("touchpad_edit", 120u16));
                                let mut text = ui.ctx().data_mut(|d| {
                                    d.get_temp::<String>(edit_id)
                                        .unwrap_or_else(|| current.to_string())
                                });
                                if text.parse::<u16>().ok() != Some(current)
                                    && !ui.memory(|m| m.has_focus(edit_id))
                                {
                                    text = current.to_string();
                                }
                                let resp = crate::ui_style::modern_text_field_sized(
                                    ui,
                                    edit_id,
                                    &mut text,
                                    field_width,
                                    metrics.settings_control_height(),
                                    "",
                                    5,
                                    egui::Align::Center,
                                );
                                let commit = resp.lost_focus()
                                    || (resp.has_focus()
                                        && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                                if commit {
                                    match text.trim().parse::<u16>() {
                                        Ok(value) => {
                                            let value = value.clamp(100, 1000);
                                            if value != current {
                                                self.touchpad_settings.dpi = value;
                                                if let Some(hid) = &self.hid_device {
                                                    if let Err(e) =
                                                        hid.set_qmk_setting_u16(120, value)
                                                    {
                                                        self.status_msg = format!(
                                                            "Failed to save Touchpad setting (qsid 120): {}",
                                                            e
                                                        );
                                                        log::warn!(
                                                            "set_qmk_setting_u16(touchpad qsid 120) failed: {e}"
                                                        );
                                                    }
                                                }
                                            }
                                            text = value.to_string();
                                        }
                                        Err(_) => text = current.to_string(),
                                    }
                                }
                                ui.ctx().data_mut(|d| d.insert_temp(edit_id, text));
                            } else {
                                let dropdown_id = ui.make_persistent_id("touchpad_dpi_dropdown");
                                let (_, picked) = Self::draw_touchpad_select_control(
                                    ui,
                                    dark,
                                    dropdown_id,
                                    selected_idx,
                                    &variants,
                                    dropdown_width,
                                );
                                if let Some(picked) = picked {
                                    self.touchpad_settings.dpi = picked as u16;
                                    self.write_touchpad_select_setting(120, picked as u8);
                                }
                            }
                        },
                    );
                }
                1..=3 => {
                    const SENS_MIN: u16 = 1;
                    const SENS_MAX: u16 = 32;
                    let slider_width = metrics.value(124.0);
                    let value_width = metrics.value(34.0);
                    let slider_control_width = slider_width + value_width + metrics.value(8.0);
                    let slider_size = [slider_width, metrics.value(20.0)];
                    let (qsid, label, tooltip) = match row_idx {
                        1 => (
                            121,
                            crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "touchpad_settings.sniper_sens",
                            ),
                            crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "touchpad_settings.sniper_divisor_lower_is_faster_higher_is_more_precise",
                            ),
                        ),
                        2 => (
                            122,
                            crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "touchpad_settings.scroll_sens",
                            ),
                            crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "touchpad_settings.scroll_divisor_lower_is_faster_higher_is_smoother",
                            ),
                        ),
                        3 => (
                            123,
                            crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "touchpad_settings.text_sens",
                            ),
                            crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "touchpad_settings.text_mode_divisor_lower_is_faster_higher_is_slower",
                            ),
                        ),
                        _ => unreachable!(),
                    };
                    let current = self.touchpad_numeric_value(qsid).clamp(SENS_MIN, SENS_MAX);
                    let mut value = current as f32;
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        label,
                        true,
                        if suppress_tooltips {
                            None
                        } else {
                            Some(tooltip)
                        },
                        slider_control_width,
                        |ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            let slider_fill = if dark {
                                Color32::from_rgb(92, 92, 96)
                            } else {
                                Color32::from_rgb(190, 184, 182)
                            };
                            ui.visuals_mut().selection.bg_fill = slider_fill;
                            ui.visuals_mut().widgets.active.bg_fill = slider_fill;
                            ui.visuals_mut().widgets.active.weak_bg_fill = slider_fill;
                            ui.visuals_mut().widgets.hovered.bg_stroke =
                                Stroke::new(1.0, slider_fill);
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.add_sized(
                                        [value_width, row_height],
                                        egui::Label::new(
                                            RichText::new(format!("{}", value.round() as u8))
                                                .size(metrics.value(12.0))
                                                .color(if dark {
                                                    Color32::from_gray(230)
                                                } else {
                                                    Color32::from_gray(55)
                                                }),
                                        )
                                        .halign(egui::Align::RIGHT),
                                    );
                                    ui.spacing_mut().slider_width = slider_width;
                                    let slider = egui::Slider::new(
                                        &mut value,
                                        SENS_MIN as f32..=SENS_MAX as f32,
                                    )
                                    .step_by(1.0)
                                    .show_value(false)
                                    .trailing_fill(true);
                                    let resp = ui.add_sized(slider_size, slider);
                                    if resp.changed() {
                                        let new_value =
                                            value.round().clamp(SENS_MIN as f32, SENS_MAX as f32)
                                                as u16;
                                        if new_value != current {
                                            self.set_touchpad_numeric_value(qsid, new_value);
                                            self.write_touchpad_numeric_setting(qsid, new_value);
                                        }
                                    }
                                },
                            );
                        },
                    );
                }
                4..=6 => {
                    let (bit, label, tooltip) = match row_idx {
                        4 => (0, crate::i18n::tr_catalog(self.app_settings.language, "touchpad_settings.invert_scroll"), crate::i18n::tr_catalog(self.app_settings.language, "touchpad_settings.reverse_the_touchpad_scroll_direction")),
                        5 => (
                            1,
                            crate::i18n::tr_catalog(self.app_settings.language, "touchpad_settings.acceleration"),
                            crate::i18n::tr_catalog(self.app_settings.language, "touchpad_settings.use_firmware_pointer_acceleration_for_touchpad_movement"),
                        ),
                        6 => (
                            2,
                            crate::i18n::tr_catalog(self.app_settings.language, "touchpad_settings.sticky_mode"),
                            crate::i18n::tr_catalog(self.app_settings.language, "touchpad_settings.keep_the_selected_touchpad_mode_active_until_another_mode_is_selected"),
                        ),
                        _ => unreachable!(),
                    };
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        label,
                        true,
                        if suppress_tooltips {
                            None
                        } else {
                            Some(tooltip)
                        },
                        switch_width,
                        |ui| {
                            let mut value = self.touchpad_settings.bit(bit);
                            let resp = crate::ui_style::settings_switch_sized_stable(
                                ui,
                                ("touchpad_settings", bit),
                                &mut value,
                                switch_size,
                            );
                            if resp.changed() {
                                self.touchpad_settings.set_bit(bit, value);
                                self.write_touchpad_bits();
                            }
                        },
                    );
                }
                7 if self.touchpad_settings.auto_layer_enable_supported => {
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        crate::i18n::tr_catalog(
                            self.app_settings.language,
                            "touchpad_settings.auto_layer_enable",
                        ),
                        true,
                        if suppress_tooltips {
                            None
                        } else {
                            Some(crate::i18n::tr_catalog(self.app_settings.language, "touchpad_settings.automatically_switch_to_the_selected_layer_while_the_touchpad_is_activ"))
                        },
                        switch_width,
                        |ui| {
                            let mut value = self.touchpad_settings.auto_layer_enable;
                            let resp = crate::ui_style::settings_switch_sized_stable(
                                ui,
                                "touchpad_settings_auto_layer_enable",
                                &mut value,
                                switch_size,
                            );
                            if resp.changed() {
                                self.touchpad_settings.auto_layer_enable = value;
                                self.write_touchpad_bool_setting(142, value);
                            }
                        },
                    );
                }
                8 if self.touchpad_settings.auto_layer_supported() => {
                    let variants = self.touchpad_settings.auto_layer_variants.clone();
                    let selected_idx = (self.touchpad_settings.auto_layer as usize)
                        .min(variants.len().saturating_sub(1));
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        content_width,
                        row_height,
                        crate::i18n::tr_catalog(
                            self.app_settings.language,
                            "touchpad_settings.auto_layer",
                        ),
                        true,
                        if suppress_tooltips {
                            None
                        } else {
                            Some(crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "touchpad_settings.layer_selected_automatically_while_the_touchpad_is_active",
                            ))
                        },
                        dropdown_width,
                        |ui| {
                            let dropdown_id = ui.make_persistent_id("touchpad_auto_layer_dropdown");
                            let (_, picked) = Self::draw_touchpad_select_control(
                                ui,
                                dark,
                                dropdown_id,
                                selected_idx,
                                &variants,
                                dropdown_width,
                            );
                            if let Some(picked) = picked {
                                self.touchpad_settings.auto_layer = picked as u8;
                                self.write_touchpad_select_setting(143, picked as u8);
                            }
                        },
                    );
                }
                _ => {}
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn selected_live_features_path_and_mode(
        &self,
    ) -> Option<(String, crate::qmk_hid_host::HostDataMode)> {
        let selected = self
            .selected_device
            .and_then(|idx| self.device_manager.devices().get(idx))?;
        if selected.firmware != FirmwareProtocol::Vial {
            return None;
        }

        let mut mode = crate::qmk_hid_host::HostDataMode::default();
        if let Some(layout) = self.layout.as_ref() {
            mode = Self::qmk_hid_host_mode_for(layout, self.layout_options_value);
        }
        if Self::device_uses_automatic_display_host_data(selected) {
            mode.clock_volume = true;
            mode.media = true;
        }

        (!mode.is_empty()).then_some((selected.path.clone(), mode))
    }

    #[cfg(target_arch = "wasm32")]
    fn selected_live_features_path_and_mode(
        &self,
    ) -> Option<(String, crate::qmk_hid_host::HostDataMode)> {
        None
    }

    fn live_features_available_for_selected_device(&self) -> bool {
        self.selected_live_features_path_and_mode().is_some()
    }

    fn draw_live_feature_row(
        ui: &mut egui::Ui,
        metrics: crate::ui_style::ResponsiveMetrics,
        label: &str,
        status: &str,
        ok: bool,
        hint: Option<&str>,
    ) {
        let dark = ui.visuals().dark_mode;
        let status_color = if ok {
            if dark {
                Color32::from_rgb(205, 210, 205)
            } else {
                Color32::from_rgb(65, 70, 65)
            }
        } else if dark {
            Color32::from_rgb(230, 188, 150)
        } else {
            Color32::from_rgb(150, 82, 44)
        };
        crate::ui_style::settings_list_row_with_tooltip(
            ui,
            metrics.settings_row_content_width(),
            metrics.settings_row_height(),
            label,
            true,
            hint,
            metrics.settings_control_width(),
            |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(status)
                            .size(metrics.value(12.0))
                            .color(status_color),
                    );
                });
            },
        );
    }

    fn draw_live_features_settings_page(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect) {
        let lang = self.app_settings.language;
        let dark = ui.visuals().dark_mode;
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        let content_width = metrics.settings_content_width();
        let path_and_mode = self.selected_live_features_path_and_mode();
        let bridge_active = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                path_and_mode
                    .as_ref()
                    .map(|(path, _)| self.qmk_hid_hosts.contains_key(path))
                    .unwrap_or(false)
            }
            #[cfg(target_arch = "wasm32")]
            {
                false
            }
        };

        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(metrics.value(18.0));
                ui.label(
                    RichText::new(crate::i18n::tr(
                        self.app_settings.language,
                        crate::i18n::Key::LiveFeaturesTitle,
                    ))
                    .size(metrics.value(18.0))
                    .strong(),
                );
                ui.add_space(metrics.value(6.0));
                ui.label(
                    RichText::new(crate::i18n::tr(
                        lang,
                        crate::i18n::Key::LiveFeaturesDescription,
                    ))
                    .size(metrics.value(13.0))
                    .color(app_muted_text(dark)),
                );
                ui.add_space(metrics.value(24.0));

                let Some((_, mode)) = path_and_mode else {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::LiveFeaturesInactive),
                        Some(crate::i18n::tr(
                            lang,
                            crate::i18n::Key::LiveFeaturesSelectHint,
                        )),
                    );
                    return;
                };

                ui.set_width(content_width);
                let status = if bridge_active {
                    crate::i18n::tr_catalog(self.app_settings.language, "live_features.active")
                } else {
                    crate::i18n::tr_catalog(self.app_settings.language, "live_features.starting")
                };
                Self::draw_live_feature_row(
                    ui,
                    metrics,
                    crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "live_features.entropy_background",
                    ),
                    status,
                    bridge_active,
                    Some(crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "live_features.keep_entropy_running_in_the_background_for_live_firmware_data",
                    )),
                );
                if mode.clock_volume {
                    Self::draw_live_feature_row(
                        ui,
                        metrics,
                        crate::i18n::tr_catalog(
                            self.app_settings.language,
                            "live_features.time_sync",
                        ),
                        crate::i18n::tr_catalog(self.app_settings.language, "live_features.ready"),
                        true,
                        Some(crate::i18n::tr_catalog(
                            self.app_settings.language,
                            "live_features.uses_the_local_system_clock",
                        )),
                    );
                    let volume = crate::qmk_hid_host::volume_check();
                    Self::draw_live_feature_row(
                        ui,
                        metrics,
                        crate::i18n::tr_catalog(
                            self.app_settings.language,
                            "live_features.volume_sync",
                        ),
                        if volume.ok {
                            crate::i18n::tr_catalog(self.app_settings.language, volume.label)
                        } else {
                            crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "live_features.needs_setup",
                            )
                        },
                        volume.ok,
                        Some(crate::i18n::tr_catalog(
                            self.app_settings.language,
                            volume.hint,
                        )),
                    );
                }
                if mode.media {
                    let media = crate::qmk_hid_host::media_check();
                    Self::draw_live_feature_row(
                        ui,
                        metrics,
                        crate::i18n::tr_catalog(
                            self.app_settings.language,
                            "live_features.media_info",
                        ),
                        if media.ok {
                            crate::i18n::tr_catalog(self.app_settings.language, media.label)
                        } else {
                            crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "live_features.needs_setup",
                            )
                        },
                        media.ok,
                        Some(crate::i18n::tr_catalog(
                            self.app_settings.language,
                            media.hint,
                        )),
                    );
                }

                ui.add_space(metrics.value(18.0));
                ui.label(
                    RichText::new(crate::i18n::tr(
                        lang,
                        crate::i18n::Key::LiveFeaturesReadyNote,
                    ))
                    .size(metrics.value(12.0))
                    .color(app_muted_text(dark)),
                );
            });
        });
    }

    fn draw_touchpad_settings_page(&mut self, ui: &mut egui::Ui, content_rect: egui::Rect) {
        let lang = self.app_settings.language;
        let dark = ui.visuals().dark_mode;
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        let hid_ready = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.hid_device.is_some()
            }
            #[cfg(target_arch = "wasm32")]
            {
                false
            }
        };

        ui.allocate_ui_at_rect(content_rect, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(18.0);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::TouchpadTitle))
                        .size(18.0)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(crate::i18n::tr(lang, crate::i18n::Key::TouchpadDescription))
                        .size(13.0)
                        .color(app_muted_text(dark)),
                );
                ui.add_space(24.0);

                if !self.touchpad_settings.supported {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::TouchpadUnavailable),
                        Some(crate::i18n::tr(lang, crate::i18n::Key::TouchpadEnableHint)),
                    );
                    return;
                }

                if !hid_ready {
                    crate::ui_style::modal_empty_state(
                        ui,
                        crate::i18n::tr(lang, crate::i18n::Key::TouchpadConnect),
                        None,
                    );
                    return;
                }

                let total_rows = self.touchpad_settings.row_count();
                let list = allocate_adaptive_settings_list_viewport(
                    ui,
                    "touchpad_settings",
                    metrics,
                    total_rows,
                    0.0,
                );
                ui.allocate_ui_at_rect(list.content_rect, |ui| {
                    ui.set_clip_rect(list.viewport);
                    ui.set_min_size(list.content_rect.size());
                    ui.spacing_mut().item_spacing.y = 0.0;
                    self.draw_touchpad_editor_content(
                        ui,
                        list.first_visible_row..list.last_visible_row,
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
            });
        });
    }

    fn write_mouse_keys_setting(&mut self, qsid: u16, value: u16) {
        let Some(hid) = &self.hid_device else {
            return;
        };
        let value = value.min(u8::MAX as u16) as u8;
        if let Err(e) = hid.set_qmk_setting_u8(qsid, value) {
            self.status_msg = format!("Failed to save Mouse keys setting (qsid {qsid}): {}", e);
            log::warn!("set_qmk_setting_u8(mouse_keys qsid {qsid}) failed: {e}");
        }
    }

    fn draw_mouse_keys_editor_content(
        &mut self,
        ui: &mut egui::Ui,
        row_range: std::ops::Range<usize>,
        content_width: f32,
        row_height: f32,
        field_width: f32,
        suppress_tooltips: bool,
    ) {
        // Limits match Vial GUI qmk_settings.json.
        let lang = self.app_settings.language;
        let rows: [(u16, &'static str, &'static str, SettingsFieldUnit, u32); 9] = [
            (
                9,
                "mouse_keys_settings.delay_label",
                "mouse_keys_settings.delay_tooltip",
                SettingsFieldUnit::Milliseconds,
                10000,
            ),
            (
                10,
                "mouse_keys_settings.interval_label",
                "mouse_keys_settings.interval_tooltip",
                SettingsFieldUnit::Milliseconds,
                10000,
            ),
            (
                11,
                "mouse_keys_settings.move_delta_label",
                "mouse_keys_settings.move_delta_tooltip",
                SettingsFieldUnit::CursorSteps,
                1000,
            ),
            (
                12,
                "mouse_keys_settings.max_speed_label",
                "mouse_keys_settings.max_speed_tooltip",
                SettingsFieldUnit::SpeedSteps,
                1000,
            ),
            (
                13,
                "mouse_keys_settings.time_to_max_label",
                "mouse_keys_settings.time_to_max_tooltip",
                SettingsFieldUnit::Milliseconds,
                1000,
            ),
            (
                14,
                "mouse_keys_settings.wheel_delay_label",
                "mouse_keys_settings.wheel_delay_tooltip",
                SettingsFieldUnit::Milliseconds,
                10000,
            ),
            (
                15,
                "mouse_keys_settings.wheel_interval_label",
                "mouse_keys_settings.wheel_interval_tooltip",
                SettingsFieldUnit::Milliseconds,
                10000,
            ),
            (
                16,
                "mouse_keys_settings.wheel_max_speed_label",
                "mouse_keys_settings.wheel_max_speed_tooltip",
                SettingsFieldUnit::SpeedSteps,
                1000,
            ),
            (
                17,
                "mouse_keys_settings.wheel_time_to_max_label",
                "mouse_keys_settings.wheel_time_to_max_tooltip",
                SettingsFieldUnit::Milliseconds,
                1000,
            ),
        ];
        let control_height = (row_height / 54.0).clamp(1.0, 1.12) * 32.0;

        for row_idx in row_range {
            let Some((qsid, label, tooltip, unit, max)) = rows.get(row_idx).copied() else {
                continue;
            };
            let current = match qsid {
                9 => self.mouse_keys_settings.delay,
                10 => self.mouse_keys_settings.interval,
                11 => self.mouse_keys_settings.move_delta,
                12 => self.mouse_keys_settings.max_speed,
                13 => self.mouse_keys_settings.time_to_max,
                14 => self.mouse_keys_settings.wheel_delay,
                15 => self.mouse_keys_settings.wheel_interval,
                16 => self.mouse_keys_settings.wheel_max_speed,
                17 => self.mouse_keys_settings.wheel_time_to_max,
                _ => continue,
            };

            crate::ui_style::settings_list_row_with_tooltip(
                ui,
                content_width,
                row_height,
                crate::i18n::tr_catalog(lang, label),
                true,
                if suppress_tooltips {
                    None
                } else {
                    Some(crate::i18n::tr_catalog(lang, tooltip))
                },
                field_width,
                |ui| {
                    let edit_id = egui::Id::new(("mouse_keys_edit", qsid));
                    let mut text = ui.ctx().data_mut(|d| {
                        d.get_temp::<String>(edit_id)
                            .unwrap_or_else(|| current.to_string())
                    });
                    if text.parse::<u16>().ok() != Some(current)
                        && !ui.memory(|m| m.has_focus(edit_id))
                    {
                        text = current.to_string();
                    }

                    let resp = crate::ui_style::modern_text_field_sized(
                        ui,
                        edit_id,
                        &mut text,
                        field_width,
                        control_height,
                        "",
                        5,
                        egui::Align::RIGHT,
                    );
                    let resp = settings_field_unit_tooltip(resp, lang, suppress_tooltips, unit);

                    if resp.changed() {
                        let filtered: String =
                            text.chars().filter(|c: &char| c.is_ascii_digit()).collect();
                        let parsed = filtered.parse::<u32>().unwrap_or(0).min(max);
                        let new_value = parsed as u16;
                        if new_value != current {
                            match qsid {
                                9 => self.mouse_keys_settings.delay = new_value,
                                10 => self.mouse_keys_settings.interval = new_value,
                                11 => self.mouse_keys_settings.move_delta = new_value,
                                12 => self.mouse_keys_settings.max_speed = new_value,
                                13 => self.mouse_keys_settings.time_to_max = new_value,
                                14 => self.mouse_keys_settings.wheel_delay = new_value,
                                15 => self.mouse_keys_settings.wheel_interval = new_value,
                                16 => self.mouse_keys_settings.wheel_max_speed = new_value,
                                17 => self.mouse_keys_settings.wheel_time_to_max = new_value,
                                _ => {}
                            }
                            self.write_mouse_keys_setting(qsid, new_value);
                        }
                        text = filtered;
                    }
                    ui.ctx().data_mut(|d| d.insert_temp(edit_id, text));
                },
            );
        }
    }

    fn key_override_mod_mask_summary(language: crate::i18n::Language, mask: u8) -> String {
        match mask.count_ones() {
            0 => crate::i18n::tr_catalog(language, "key_override_editor.none").to_string(),
            8 => crate::i18n::tr_catalog(language, "key_override_editor.all_mods").to_string(),
            count if matches!(language, crate::i18n::Language::Russian) => {
                format!("{} модиф.", count)
            }
            count => format!("{} mods", count),
        }
    }

    fn key_override_layers_summary(language: crate::i18n::Language, layers: u16) -> String {
        match layers.count_ones() {
            0 => crate::i18n::tr_catalog(language, "key_override_editor.no_layers").to_string(),
            16 => crate::i18n::tr_catalog(language, "key_override_editor.all_layers").to_string(),
            count if matches!(language, crate::i18n::Language::Russian) => {
                format!("{} слоёв", count)
            }
            count => format!("{} layers", count),
        }
    }

    fn draw_key_override_layers_modern(
        ui: &mut egui::Ui,
        layers: &mut u16,
        language: crate::i18n::Language,
    ) -> bool {
        let mut changed = false;
        let dark = ui.visuals().dark_mode;

        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        let popup_width = metrics.value(244.0);
        let layer_button_size = metrics.size(52.0, 30.0);
        let button_gap = metrics.value(6.0);
        let layer_row_width = layer_button_size.x * 4.0 + button_gap * 3.0;
        let layer_row_inset = ((popup_width - layer_row_width) * 0.5).max(0.0);

        ui.set_min_width(popup_width);
        ui.spacing_mut().item_spacing = Vec2::new(button_gap, button_gap);
        for row in 0..4 {
            ui.horizontal(|ui| {
                ui.add_space(layer_row_inset);
                for col in 0..4 {
                    let idx = row * 4 + col;
                    let bit = 1u16 << idx;
                    let selected = (*layers & bit) != 0;
                    let label = idx.to_string();
                    let (rect, resp) = ui.allocate_exact_size(layer_button_size, Sense::click());
                    let active = resp.is_pointer_button_down_on();
                    let hovered = resp.hovered();
                    let fill = if selected {
                        if hovered || active {
                            if dark {
                                Color32::from_rgb(72, 64, 68)
                            } else {
                                Color32::from_rgb(242, 226, 230)
                            }
                        } else if dark {
                            Color32::from_rgb(62, 56, 59)
                        } else {
                            Color32::from_rgb(248, 236, 239)
                        }
                    } else if active {
                        if dark {
                            Color32::from_rgb(56, 56, 59)
                        } else {
                            Color32::from_rgb(232, 232, 235)
                        }
                    } else if hovered {
                        crate::ui_style::hover_fill(dark)
                    } else {
                        crate::ui_style::surface_fill(dark)
                    };
                    let stroke = if selected {
                        Stroke::new(metrics.value(1.35), app_accent())
                    } else {
                        crate::ui_style::modal_outline_stroke(dark)
                    };
                    ui.painter().rect(
                        rect,
                        metrics.value(9.0),
                        fill,
                        stroke,
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        label,
                        FontId::proportional(metrics.value(12.5)),
                        if selected {
                            ui.visuals().text_color()
                        } else {
                            app_muted_text(dark).gamma_multiply(if dark { 0.62 } else { 1.10 })
                        },
                    );
                    if selected {
                        ui.painter().circle_filled(
                            egui::pos2(
                                rect.right() - metrics.value(8.0),
                                rect.top() + metrics.value(8.0),
                            ),
                            metrics.value(2.4),
                            app_accent(),
                        );
                    }
                    if hovered {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if resp.clicked() {
                        if selected {
                            *layers &= !bit;
                        } else {
                            *layers |= bit;
                        }
                        changed = true;
                    }
                }
            });
        }

        ui.add_space(metrics.value(4.0));
        let action_width = metrics.value(112.0) * 2.0 + button_gap;
        let action_inset = ((popup_width - action_width) * 0.5).max(0.0);
        ui.horizontal(|ui| {
            ui.add_space(action_inset);
            let all_resp = crate::ui_style::modern_button(
                ui,
                crate::i18n::tr_catalog(language, "key_override_editor.enable_all"),
                metrics.size(112.0, 30.0),
                true,
            );
            if all_resp.clicked() && *layers != u16::MAX {
                *layers = u16::MAX;
                changed = true;
            }
            let none_resp = crate::ui_style::modern_button(
                ui,
                crate::i18n::tr_catalog(language, "key_override_editor.disable_all"),
                metrics.size(112.0, 30.0),
                true,
            );
            if none_resp.clicked() && *layers != 0 {
                *layers = 0;
                changed = true;
            }
        });
        changed
    }

    fn draw_key_override_mod_mask_modern(
        ui: &mut egui::Ui,
        mask: &mut u8,
        language: crate::i18n::Language,
    ) -> bool {
        let mut changed = false;
        let gui = crate::keycode::gui_mod_name();
        let labels = if matches!(language, crate::i18n::Language::Russian) {
            [
                "Левый Ctrl".to_string(),
                "Левый Shift".to_string(),
                "Левый Alt".to_string(),
                format!("Левый {}", gui),
                "Правый Ctrl".to_string(),
                "Правый Shift".to_string(),
                "Правый Alt".to_string(),
                format!("Правый {}", gui),
            ]
        } else {
            [
                "Left Ctrl".to_string(),
                "Left Shift".to_string(),
                "Left Alt".to_string(),
                format!("Left {}", gui),
                "Right Ctrl".to_string(),
                "Right Shift".to_string(),
                "Right Alt".to_string(),
                format!("Right {}", gui),
            ]
        };

        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        let popup_width = metrics.value(244.0);
        let row_height = metrics.value(34.0);
        let switch_width = metrics.value(46.0);
        let switch_size = metrics.size(46.0, 24.0);
        ui.set_min_width(popup_width);
        ui.spacing_mut().item_spacing.y = 0.0;
        for (idx, label) in labels.iter().enumerate() {
            let bit = 1u8 << idx;
            let mut checked = (*mask & bit) != 0;
            crate::ui_style::settings_list_row_with_tooltip(
                ui,
                popup_width,
                row_height,
                label.as_str(),
                true,
                None,
                switch_width,
                |ui| {
                    let resp =
                        crate::ui_style::settings_switch_sized(ui, &mut checked, switch_size);
                    if resp.changed() {
                        if checked {
                            *mask |= bit;
                        } else {
                            *mask &= !bit;
                        }
                        changed = true;
                    }
                },
            );
        }
        changed
    }

    fn draw_key_override_editor_content(&mut self, ui: &mut egui::Ui, _two_column: bool) {
        let dark = ui.visuals().dark_mode;
        if self.firmware != FirmwareProtocol::Vial {
            crate::ui_style::modal_empty_state(
                ui,
                "Key Overrides are not supported for this firmware",
                None,
            );
            return;
        }

        if self.key_override_entries.is_empty() {
            crate::ui_style::modal_empty_state(
                ui,
                "This keyboard does not report any Key Override slots",
                None,
            );
            return;
        }

        self.selected_key_override = self
            .selected_key_override
            .min(self.key_override_entries.len().saturating_sub(1));
        self.key_override_names
            .resize(self.key_override_entries.len(), String::new());
        self.key_override_visible_count = self.key_override_entries.len().max(1);

        let idx = self.selected_key_override;
        let current = self.key_override_entries[idx].clone();
        let mut edited = current.clone();
        let page_center_x = ui.max_rect().center().x;
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        let scale = metrics.scale;
        let content_width = metrics.settings_content_width();
        let row_content_width = metrics.settings_row_content_width();
        let row_height = metrics.settings_row_height();
        let control_width = metrics.settings_control_width();
        let control_height = metrics.settings_control_height();
        let control_font_size = metrics.settings_control_font_size();
        let switch_width = metrics.value(46.0);
        let switch_size = metrics.size(46.0, 24.0);
        const ROW_COUNT: usize = 14;

        let custom = self
            .layout
            .as_ref()
            .map(|l| l.custom_keycodes.as_slice())
            .unwrap_or(&[]);
        let selected_override_empty = self
            .key_override_entries
            .get(idx)
            .map(|entry| !Self::key_override_entry_exists(entry))
            .unwrap_or(true)
            && self
                .key_override_names
                .get(idx)
                .map(|name| name.trim().is_empty())
                .unwrap_or(true);
        let selected_override_text = match self.key_override_names.get(idx) {
            Some(name) if !name.trim().is_empty() => format!("KO{}: {}", idx, name.trim()),
            _ => format!("KO{}", idx),
        };
        let selected_override_text_color = if selected_override_empty {
            app_inactive_entry_text(dark)
        } else {
            ui.visuals().text_color()
        };
        let trigger_label = if edited.trigger == 0 {
            crate::i18n::tr_catalog(
                self.app_settings.language,
                "key_override_editor.pick_trigger",
            )
            .to_string()
        } else {
            keycode_label_with_macro_names(
                edited.trigger,
                custom,
                &self.layer_names,
                &self.keycode_picker.macro_names,
                &self.keycode_picker.tap_dance_names,
                self.app_settings.key_legend_layout,
            )
            .replace('\n', " ")
        };
        let replacement_label = if edited.replacement == 0 {
            crate::i18n::tr_catalog(
                self.app_settings.language,
                "key_override_editor.pick_replacement",
            )
            .to_string()
        } else {
            keycode_label_with_macro_names(
                edited.replacement,
                custom,
                &self.layer_names,
                &self.keycode_picker.macro_names,
                &self.keycode_picker.tap_dance_names,
                self.app_settings.key_legend_layout,
            )
            .replace('\n', " ")
        };
        let trigger_tip = keycode_tooltip_with_macro_names(
            edited.trigger,
            custom,
            &self.layer_names,
            &self.keycode_picker.macro_names,
            &self.keycode_picker.tap_dance_names,
        );
        let replacement_tip = keycode_tooltip_with_macro_names(
            edited.replacement,
            custom,
            &self.layer_names,
            &self.keycode_picker.macro_names,
            &self.keycode_picker.tap_dance_names,
        );

        crate::ui_style::modal_content(
            ui,
            crate::ui_style::ModalLayout::new(content_width).with_top_padding(4.0 * scale),
            |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let list = allocate_adaptive_settings_list_viewport(
                    ui,
                    "key_override_settings",
                    metrics,
                    ROW_COUNT,
                    metrics.value(86.0),
                );
                ui.allocate_ui_at_rect(list.content_rect, |ui| {
                    ui.set_clip_rect(list.viewport);
                    ui.set_min_size(list.content_rect.size());
                    ui.spacing_mut().item_spacing.y = 0.0;
                    for row_idx in list.first_visible_row..list.last_visible_row {
                                    match row_idx {
                                        0 => {
                                            crate::ui_style::settings_list_row_with_tooltip(
                                                ui,
                                                row_content_width,
                                                row_height,
                                                crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.entry"),
                                                true,
                                                Some(crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.select_key_override_slot")),
                                                control_width,
                                                |ui| {
                                                    let dropdown_id = ui.make_persistent_id("key_override_entry_dropdown");
                                                    let dropdown_resp = crate::ui_style::modern_dropdown_button_sized(
                                                        ui,
                                                        dropdown_id,
                                                        selected_override_text.as_str(),
                                                        selected_override_text_color,
                                                        control_width,
                                                        control_height,
                                                        control_font_size,
                                                    );
                                                    ui.style_mut().visuals.window_stroke =
                                                        crate::ui_style::modal_outline_stroke(dark);
                                                    ui.style_mut().visuals.window_fill = app_surface_fill(dark);
                                                    egui::popup_below_widget(
                                                        ui,
                                                        dropdown_id,
                                                        &dropdown_resp,
                                                        egui::PopupCloseBehavior::CloseOnClickOutside,
                                                        |ui| {
                                                            ui.set_min_width(control_width);
                                                            ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
                                                            egui::ScrollArea::vertical()
                                                                .id_salt("key_override_entry_dropdown_scroll")
                                                                 .max_height(142.0 * scale)
                                                                .auto_shrink([false, true])
                                                                .show(ui, |ui| {
                                                                    for entry_idx in 0..self.key_override_entries.len() {
                                                                        let empty = self.key_override_entries.get(entry_idx)
                                                                            .map(|entry| !Self::key_override_entry_exists(entry))
                                                                            .unwrap_or(true)
                                                                            && self.key_override_names.get(entry_idx)
                                                                                .map(|name| name.trim().is_empty())
                                                                                .unwrap_or(true);
                                                                        let option_text = match self.key_override_names.get(entry_idx) {
                                                                            Some(name) if !name.trim().is_empty() => format!("KO{}: {}", entry_idx, name.trim()),
                                                                            _ => format!("KO{}", entry_idx),
                                                                        };
                                                                        let selected = entry_idx == self.selected_key_override;
                                                                        let (option_rect, option_resp) = ui.allocate_exact_size(
                                                                            Vec2::new(control_width, 28.0 * scale),
                                                                            Sense::click(),
                                                                        );
                                                                        if option_resp.hovered() {
                                                                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                                                        }
                                                                        let option_fill = if selected {
                                                                            if dark { Color32::from_rgb(58, 58, 61) } else { Color32::from_rgb(236, 236, 238) }
                                                                        } else if option_resp.hovered() {
                                                                            crate::ui_style::hover_fill(dark)
                                                                        } else {
                                                                            Color32::TRANSPARENT
                                                                        };
                                                                        ui.painter().rect_filled(option_rect, 7.0, option_fill);
                                                                        ui.painter().text(
                                                                            egui::pos2(option_rect.left() + 10.0, option_rect.center().y),
                                                                            egui::Align2::LEFT_CENTER,
                                                                            option_text,
                                                                            FontId::proportional(12.0 * scale),
                                                                            if selected {
                                                                                ui.visuals().text_color()
                                                                            } else if empty {
                                                                                app_inactive_entry_text(dark)
                                                                            } else {
                                                                                app_muted_text(dark)
                                                                            },
                                                                        );
                                                                        if option_resp.clicked() {
                                                                            self.selected_key_override = entry_idx;
                                                                            ui.memory_mut(|m| m.close_popup());
                                                                        }
                                                                    }
                                                                });
                                                        },
                                                    );
                                                },
                                            );
                                        }
                                        1 => {
                                            let mut name_changed = false;
                                            crate::ui_style::settings_list_row_with_tooltip(
                                                ui,
                                                row_content_width,
                                                row_height,
                                                crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.name"),
                                                true,
                                                Some(crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.local_name_for_this_key_override_slot")),
                                                control_width,
                                                |ui| {
                                                    if let Some(name) = self.key_override_names.get_mut(idx) {
                                                        let resp = crate::ui_style::modern_text_field_sized(
                                                            ui,
                                                            egui::Id::new(("key_override_name", idx)),
                                                            name,
                                                            control_width,
                                                            32.0 * scale,
                                                            crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.name"),
                                                            12,
                                                            egui::Align::Center,
                                                        );
                                                        name_changed = resp.changed();
                                                        resp.clone().on_hover_text(crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.stored_locally_in_entropy"));
                                                    }
                                                },
                                            );
                                            if name_changed {
                                                save_key_override_names(&self.key_override_names, &self.current_device_name);
                                            }
                                        }
                                        2 => {
                                            crate::ui_style::settings_list_row_with_tooltip(
                                                ui,
                                                row_content_width,
                                                row_height,
                                                crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.trigger"),
                                                true,
                                                Some(crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.original_key_that_can_be_overridden")),
                                                control_width,
                                                |ui| {
                                                    let resp = crate::ui_style::modern_button_with_font(ui, trigger_label.as_str(), Vec2::new(control_width, control_height), control_font_size, true);
                                                    if resp.clicked() {
                                                        self.open_key_override_picker(KeyOverridePickField::Trigger);
                                                    }
                                                    resp.on_hover_text(crate::i18n::tr_text(self.app_settings.language, &trigger_tip));
                                                },
                                            );
                                        }
                                        3 => {
                                            crate::ui_style::settings_list_row_with_tooltip(
                                                ui,
                                                row_content_width,
                                                row_height,
                                                crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.replacement"),
                                                true,
                                                Some(crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.keycode_sent_while_override_conditions_match")),
                                                control_width,
                                                |ui| {
                                                    let resp = crate::ui_style::modern_button_with_font(ui, replacement_label.as_str(), Vec2::new(control_width, control_height), control_font_size, true);
                                                    if resp.clicked() {
                                                        self.open_key_override_picker(KeyOverridePickField::Replacement);
                                                    }
                                                    resp.on_hover_text(crate::i18n::tr_text(self.app_settings.language, &replacement_tip));
                                                },
                                            );
                                        }
                                        4 => {
                                            crate::ui_style::settings_list_row_with_tooltip(
                                                ui,
                                                row_content_width,
                                                row_height,
                                                crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.suppressed_mods"),
                                                true,
                                                Some(crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.modifiers_hidden_while_the_replacement_is_active")),
                                                control_width,
                                                |ui| {
                                                    let popup_id = ui.make_persistent_id(("ko_suppressed_mods_popup", idx));
                                                    let summary = Self::key_override_mod_mask_summary(self.app_settings.language, edited.suppressed_mods);
                                                    let resp = crate::ui_style::modern_button_with_font(ui, summary.as_str(), Vec2::new(control_width, control_height), control_font_size, true);
                                                    if resp.clicked() { ui.memory_mut(|m| m.toggle_popup(popup_id)); }
                                                    ui.style_mut().visuals.window_stroke = crate::ui_style::modal_outline_stroke(dark);
                                                    ui.style_mut().visuals.window_fill = app_surface_fill(dark);
                                                    egui::popup_below_widget(ui, popup_id, &resp, egui::PopupCloseBehavior::CloseOnClickOutside, |ui| {
                                                        Self::draw_key_override_mod_mask_modern(ui, &mut edited.suppressed_mods, self.app_settings.language);
                                                    });
                                                },
                                            );
                                        }
                                        5 => {
                                            crate::ui_style::settings_list_row_with_tooltip(
                                                ui,
                                                row_content_width,
                                                row_height,
                                                crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.trigger_mods"),
                                                true,
                                                Some(crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.modifiers_required_for_this_override")),
                                                control_width,
                                                |ui| {
                                                    let popup_id = ui.make_persistent_id(("ko_trigger_mods_popup", idx));
                                                    let summary = Self::key_override_mod_mask_summary(self.app_settings.language, edited.trigger_mods);
                                                    let resp = crate::ui_style::modern_button_with_font(ui, summary.as_str(), Vec2::new(control_width, control_height), control_font_size, true);
                                                    if resp.clicked() { ui.memory_mut(|m| m.toggle_popup(popup_id)); }
                                                    ui.style_mut().visuals.window_stroke = crate::ui_style::modal_outline_stroke(dark);
                                                    ui.style_mut().visuals.window_fill = app_surface_fill(dark);
                                                    egui::popup_below_widget(ui, popup_id, &resp, egui::PopupCloseBehavior::CloseOnClickOutside, |ui| {
                                                        Self::draw_key_override_mod_mask_modern(ui, &mut edited.trigger_mods, self.app_settings.language);
                                                    });
                                                },
                                            );
                                        }
                                        6 => {
                                            crate::ui_style::settings_list_row_with_tooltip(
                                                ui,
                                                row_content_width,
                                                row_height,
                                                crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.negative_mods"),
                                                true,
                                                Some(crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.modifiers_that_block_this_override")),
                                                control_width,
                                                |ui| {
                                                    let popup_id = ui.make_persistent_id(("ko_negative_mods_popup", idx));
                                                    let summary = Self::key_override_mod_mask_summary(self.app_settings.language, edited.negative_mod_mask);
                                                    let resp = crate::ui_style::modern_button_with_font(ui, summary.as_str(), Vec2::new(control_width, control_height), control_font_size, true);
                                                    if resp.clicked() { ui.memory_mut(|m| m.toggle_popup(popup_id)); }
                                                    ui.style_mut().visuals.window_stroke = crate::ui_style::modal_outline_stroke(dark);
                                                    ui.style_mut().visuals.window_fill = app_surface_fill(dark);
                                                    egui::popup_below_widget(ui, popup_id, &resp, egui::PopupCloseBehavior::CloseOnClickOutside, |ui| {
                                                        Self::draw_key_override_mod_mask_modern(ui, &mut edited.negative_mod_mask, self.app_settings.language);
                                                    });
                                                },
                                            );
                                        }
                                        7 => {
                                            crate::ui_style::settings_list_row_with_tooltip(
                                                ui,
                                                row_content_width,
                                                row_height,
                                                crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.enable_on_layers"),
                                                true,
                                                Some(crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.layers_where_this_override_can_activate")),
                                                control_width,
                                                |ui| {
                                                    let popup_id = ui.make_persistent_id(("ko_layers_popup", idx));
                                                    let summary = Self::key_override_layers_summary(self.app_settings.language, edited.layers);
                                                    let resp = crate::ui_style::modern_button_with_font(ui, summary.as_str(), Vec2::new(control_width, control_height), control_font_size, true);
                                                    if resp.clicked() { ui.memory_mut(|m| m.toggle_popup(popup_id)); }
                                                    ui.style_mut().visuals.window_stroke = crate::ui_style::modal_outline_stroke(dark);
                                                    ui.style_mut().visuals.window_fill = app_surface_fill(dark);
                                                    egui::popup_below_widget(ui, popup_id, &resp, egui::PopupCloseBehavior::CloseOnClickOutside, |ui| {
                                                        Self::draw_key_override_layers_modern(ui, &mut edited.layers, self.app_settings.language);
                                                    });
                                                },
                                            );
                                        }
                                        8 => crate::ui_style::settings_list_row_with_tooltip(ui, row_content_width, row_height, crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.trigger_press"), true, Some(crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.activate_when_the_trigger_key_is_pressed")), switch_width, |ui| { crate::ui_style::settings_switch_sized_stable(ui, ("key_override_settings", "trigger_press"), &mut edited.options.activation_trigger_down, switch_size); }),
                                        9 => crate::ui_style::settings_list_row_with_tooltip(ui, row_content_width, row_height, crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.required_mod_press"), true, Some(crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.activate_when_a_required_modifier_is_pressed")), switch_width, |ui| { crate::ui_style::settings_switch_sized_stable(ui, ("key_override_settings", "required_mod_press"), &mut edited.options.activation_required_mod_down, switch_size); }),
                                        10 => crate::ui_style::settings_list_row_with_tooltip(ui, row_content_width, row_height, crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.blocked_mod_release"), true, Some(crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.activate_when_a_blocking_modifier_is_released")), switch_width, |ui| { crate::ui_style::settings_switch_sized_stable(ui, ("key_override_settings", "blocked_mod_release"), &mut edited.options.activation_negative_mod_up, switch_size); }),
                                        11 => crate::ui_style::settings_list_row_with_tooltip(ui, row_content_width, row_height, crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.any_one_mod"), true, Some(crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.any_one_trigger_modifier_is_enough")), switch_width, |ui| { crate::ui_style::settings_switch_sized_stable(ui, ("key_override_settings", "any_one_mod"), &mut edited.options.one_mod, switch_size); }),
                                        12 => crate::ui_style::settings_list_row_with_tooltip(ui, row_content_width, row_height, crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.no_re_send"), true, Some(crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.do_not_resend_the_trigger_after_override_ends")), switch_width, |ui| { crate::ui_style::settings_switch_sized_stable(ui, ("key_override_settings", "no_re_send"), &mut edited.options.no_reregister_trigger, switch_size); }),
                                        13 => crate::ui_style::settings_list_row_with_tooltip(ui, row_content_width, row_height, crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.stay_active"), true, Some(crate::i18n::tr_catalog(self.app_settings.language, "key_override_editor.stay_active_when_another_key_is_pressed")), switch_width, |ui| { crate::ui_style::settings_switch_sized_stable(ui, ("key_override_settings", "stay_active"), &mut edited.options.no_unregister_on_other_key_down, switch_size); }),
                                        _ => {}
                                    }
                                }
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

                let action_size = crate::ui_style::modal_action_button_size() * scale;
                let action_width = action_size.x * 2.0 + 8.0 * scale;
                let action_top = list.viewport.bottom() + 14.0 * scale;
                let action_rect = egui::Rect::from_min_size(
                    egui::pos2(page_center_x - action_width / 2.0, action_top),
                    Vec2::new(action_width, action_size.y),
                );
                ui.allocate_ui_at_rect(action_rect, |ui| {
                    ui.set_min_size(action_rect.size());
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 8.0 * scale;
                        let clear_enabled =
                            Self::key_override_entry_exists(&self.key_override_entries[idx])
                                || self
                                    .key_override_names
                                    .get(idx)
                                    .map(|s| !s.trim().is_empty())
                                    .unwrap_or(false);
                        let clear_resp = crate::ui_style::modern_button_with_font(
                            ui,
                            crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "alt_repeat_editor.clear",
                            ),
                            action_size,
                            control_font_size,
                            clear_enabled,
                        );
                        if clear_resp.clicked() && clear_enabled {
                            self.push_key_override_undo();
                            self.key_override_entries[idx] = KeyOverrideEntry::default();
                            if let Some(name) = self.key_override_names.get_mut(idx) {
                                name.clear();
                            }
                            save_key_override_names(
                                &self.key_override_names,
                                &self.current_device_name,
                            );
                            self.write_key_override(idx);
                        }

                        let undo_enabled = !self.key_override_undo_stack.is_empty();
                        let undo_resp = crate::ui_style::modern_button_with_font(
                            ui,
                            crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "alt_repeat_editor.undo",
                            ),
                            action_size,
                            control_font_size,
                            undo_enabled,
                        );
                        if undo_resp.clicked() && undo_enabled {
                            if let Some((entries, names, selected, visible_count)) =
                                self.key_override_undo_stack.pop()
                            {
                                self.key_override_entries = entries;
                                self.key_override_names = names;
                                self.key_override_visible_count =
                                    visible_count.clamp(1, self.key_override_entries.len().max(1));
                                self.selected_key_override =
                                    selected.min(self.key_override_visible_count.saturating_sub(1));
                                save_key_override_names(
                                    &self.key_override_names,
                                    &self.current_device_name,
                                );
                                self.write_all_key_overrides();
                            }
                        }
                    });
                });
                let reserve_bottom = action_rect.bottom() - list.viewport.bottom();
                ui.allocate_space(Vec2::new(1.0, reserve_bottom.max(action_size.y)));
            },
        );

        Self::normalize_key_override_entry(&mut edited);
        if edited != current {
            self.push_key_override_undo();
            self.key_override_entries[idx] = edited;
            self.write_key_override(idx);
        }
    }

    fn handle_combo_editor_input(&mut self, ctx: &egui::Context, allow_close: bool) -> bool {
        if !self.keycode_picker.open && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.combo_capture_open {
                self.cancel_combo_capture();
            } else if allow_close {
                self.combo_capture_open = false;
                self.combo_capture_keys.clear();
                return true;
            }
        }

        if self.combo_capture_open {
            if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.apply_combo_capture();
            } else {
                for event in ctx.input(|i| i.events.clone()) {
                    if let egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } = event
                    {
                        if matches!(key, egui::Key::Enter | egui::Key::Escape) {
                            continue;
                        }
                        if let Some(kc) = egui_key_to_qmk(key, modifiers) {
                            if !self.combo_capture_keys.contains(&kc)
                                && self.combo_capture_keys.len() < 4
                            {
                                self.combo_capture_keys.push(kc);
                            }
                        }
                    }
                }
            }
        }

        false
    }

    fn draw_combo_editor_content(&mut self, ui: &mut egui::Ui, show_intro: bool) {
        let dark = ui.visuals().dark_mode;
        if show_intro {
            crate::ui_style::modal_hint(
                ui,
                crate::i18n::tr(
                    self.app_settings.language,
                    crate::i18n::Key::ComboDescription,
                ),
            );
        }

        if self.firmware != FirmwareProtocol::Vial {
            crate::ui_style::modal_empty_state(
                ui,
                "Dynamic combos are not supported for this firmware",
                None,
            );
            return;
        }

        if self.combo_entries.is_empty() {
            crate::ui_style::modal_empty_state(
                ui,
                "This keyboard does not report any dynamic combo slots",
                None,
            );
            return;
        }

        self.selected_combo = self
            .selected_combo
            .min(self.combo_entries.len().saturating_sub(1));
        self.combo_names
            .resize(self.combo_entries.len(), String::new());
        self.combo_visible_count = self.combo_entries.len().max(1);

        let combo_idx = self.selected_combo;
        let page_center_x = ui.max_rect().center().x;
        let combo_undo_snapshot = (
            self.combo_entries.clone(),
            self.combo_names.clone(),
            self.combo_term,
            self.selected_combo,
            self.combo_visible_count,
        );
        let metrics = crate::ui_style::ResponsiveMetrics::from_ctx(ui.ctx());
        let scale = metrics.scale;
        let content_width = metrics.settings_content_width();
        let row_content_width = metrics.settings_row_content_width();
        let row_height = metrics.settings_row_height();
        let control_width = metrics.settings_control_width();
        let control_height = metrics.settings_control_height();
        let control_font_size = metrics.settings_control_font_size();
        let timeout_control_width = metrics.value(118.0);
        let custom = self
            .layout
            .as_ref()
            .map(|l| l.custom_keycodes.as_slice())
            .unwrap_or(&[]);
        let selected_combo_empty = self
            .combo_entries
            .get(combo_idx)
            .map(|entry| entry.keys.iter().all(|&k| k == 0) && entry.output == 0)
            .unwrap_or(true)
            && self
                .combo_names
                .get(combo_idx)
                .map(|name| name.trim().is_empty())
                .unwrap_or(true);
        let selected_text = match self.combo_names.get(combo_idx) {
            Some(name) if !name.trim().is_empty() => format!("C{}: {}", combo_idx, name.trim()),
            _ => format!("C{}", combo_idx),
        };
        let selected_text_color = if selected_combo_empty {
            app_inactive_entry_text(dark)
        } else {
            ui.visuals().text_color()
        };
        let input_summary = {
            let keys: Vec<String> = if self.combo_capture_open {
                self.combo_capture_keys
                    .iter()
                    .copied()
                    .map(|kc| {
                        keycode_label_with_macro_names(
                            kc,
                            custom,
                            &self.layer_names,
                            &self.keycode_picker.macro_names,
                            &self.keycode_picker.tap_dance_names,
                            self.app_settings.key_legend_layout,
                        )
                        .replace('\n', " ")
                    })
                    .collect()
            } else {
                self.combo_entries[combo_idx]
                    .keys
                    .iter()
                    .copied()
                    .filter(|&kc| kc != 0)
                    .map(|kc| {
                        keycode_label_with_macro_names(
                            kc,
                            custom,
                            &self.layer_names,
                            &self.keycode_picker.macro_names,
                            &self.keycode_picker.tap_dance_names,
                            self.app_settings.key_legend_layout,
                        )
                        .replace('\n', " ")
                    })
                    .collect()
            };
            if keys.is_empty() {
                if self.combo_capture_open {
                    crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "combo_editor.press_2_4_keys",
                    )
                    .to_string()
                } else {
                    crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "combo_editor.record_2_4_keys",
                    )
                    .to_string()
                }
            } else {
                keys.join(" + ")
            }
        };
        let output_label = if self.combo_entries[combo_idx].output == 0 {
            crate::i18n::tr_catalog(self.app_settings.language, "combo_editor.pick_output")
                .to_string()
        } else {
            keycode_label_with_macro_names(
                self.combo_entries[combo_idx].output,
                custom,
                &self.layer_names,
                &self.keycode_picker.macro_names,
                &self.keycode_picker.tap_dance_names,
                self.app_settings.key_legend_layout,
            )
            .replace('\n', " ")
        };

        crate::ui_style::modal_content(
            ui,
            crate::ui_style::ModalLayout::new(content_width).with_top_padding(4.0 * scale),
            |ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    row_content_width,
                    row_height,
                    crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.entry"),
                    true,
                    Some(crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "combo_editor.select_combo_slot",
                    )),
                    control_width,
                    |ui| {
                        let dropdown_id = ui.make_persistent_id("combo_entry_dropdown");
                        let dropdown_resp = crate::ui_style::modern_dropdown_button_sized(
                            ui,
                            dropdown_id,
                            selected_text.as_str(),
                            selected_text_color,
                            control_width,
                            control_height,
                            control_font_size,
                        );
                        ui.style_mut().visuals.window_stroke =
                            crate::ui_style::modal_outline_stroke(dark);
                        ui.style_mut().visuals.window_fill = app_surface_fill(dark);
                        egui::popup_below_widget(
                            ui,
                            dropdown_id,
                            &dropdown_resp,
                            egui::PopupCloseBehavior::CloseOnClickOutside,
                            |ui| {
                                ui.set_min_width(control_width);
                                ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);
                                egui::ScrollArea::vertical()
                                    .id_salt("combo_entry_dropdown_scroll")
                                    .max_height(142.0 * scale)
                                    .auto_shrink([false, true])
                                    .show(ui, |ui| {
                                        for entry_idx in 0..self.combo_entries.len() {
                                            let empty = self
                                                .combo_entries
                                                .get(entry_idx)
                                                .map(|entry| {
                                                    entry.keys.iter().all(|&k| k == 0)
                                                        && entry.output == 0
                                                })
                                                .unwrap_or(true)
                                                && self
                                                    .combo_names
                                                    .get(entry_idx)
                                                    .map(|name| name.trim().is_empty())
                                                    .unwrap_or(true);
                                            let option_text = match self.combo_names.get(entry_idx)
                                            {
                                                Some(name) if !name.trim().is_empty() => {
                                                    format!("C{}: {}", entry_idx, name.trim())
                                                }
                                                _ => format!("C{}", entry_idx),
                                            };
                                            let selected = entry_idx == self.selected_combo;
                                            let (option_rect, option_resp) = ui
                                                .allocate_exact_size(
                                                    Vec2::new(control_width, 28.0 * scale),
                                                    Sense::click(),
                                                );
                                            if option_resp.hovered() {
                                                ui.ctx().set_cursor_icon(
                                                    egui::CursorIcon::PointingHand,
                                                );
                                            }
                                            let option_fill = if selected {
                                                if dark {
                                                    Color32::from_rgb(58, 58, 61)
                                                } else {
                                                    Color32::from_rgb(236, 236, 238)
                                                }
                                            } else if option_resp.hovered() {
                                                crate::ui_style::hover_fill(dark)
                                            } else {
                                                Color32::TRANSPARENT
                                            };
                                            ui.painter().rect_filled(option_rect, 7.0, option_fill);
                                            ui.painter().text(
                                                egui::pos2(
                                                    option_rect.left() + 10.0,
                                                    option_rect.center().y,
                                                ),
                                                egui::Align2::LEFT_CENTER,
                                                option_text,
                                                FontId::proportional(12.0 * scale),
                                                if selected {
                                                    ui.visuals().text_color()
                                                } else if empty {
                                                    app_inactive_entry_text(dark)
                                                } else {
                                                    app_muted_text(dark)
                                                },
                                            );
                                            if option_resp.clicked() {
                                                self.selected_combo = entry_idx;
                                                ui.memory_mut(|m| m.close_popup());
                                            }
                                        }
                                    });
                            },
                        );
                    },
                );

                let mut combo_name_changed = false;
                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    row_content_width,
                    row_height,
                    crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.name"),
                    true,
                    Some(crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "combo_editor.local_name_for_this_combo_slot",
                    )),
                    control_width,
                    |ui| {
                        if let Some(name) = self.combo_names.get_mut(combo_idx) {
                            let resp = crate::ui_style::modern_text_field_sized(
                                ui,
                                egui::Id::new(("combo_name", combo_idx)),
                                name,
                                control_width,
                                control_height,
                                crate::i18n::tr_catalog(
                                    self.app_settings.language,
                                    "alt_repeat_editor.name",
                                ),
                                12,
                                egui::Align::Center,
                            );
                            combo_name_changed = resp.changed();
                            resp.clone().on_hover_text(crate::i18n::tr_catalog(
                                self.app_settings.language,
                                "alt_repeat_editor.stored_locally_in_entropy",
                            ));
                        }
                    },
                );
                if combo_name_changed {
                    self.combo_undo_stack.push(combo_undo_snapshot.clone());
                    self.combo_names_dirty = true;
                }

                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    row_content_width,
                    row_height,
                    crate::i18n::tr_catalog(self.app_settings.language, "combo_editor.input_keys"),
                    true,
                    Some(crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "combo_editor.keys_that_must_be_pressed_together",
                    )),
                    control_width,
                    |ui| {
                        let field_resp = crate::ui_style::modern_button_with_font(
                            ui,
                            input_summary.as_str(),
                            Vec2::new(control_width, control_height),
                            control_font_size,
                            true,
                        );
                        if field_resp.clicked() {
                            self.combo_capture_keys.clear();
                            self.combo_capture_open = true;
                        }
                        if self.combo_capture_open {
                            let clicked_outside_input = ui.ctx().input(|i| {
                                i.pointer.any_pressed()
                                    && i.pointer
                                        .interact_pos()
                                        .map(|pos| !field_resp.rect.contains(pos))
                                        .unwrap_or(false)
                            });
                            if clicked_outside_input {
                                self.apply_combo_capture();
                            }
                        }
                    },
                );

                crate::ui_style::settings_list_row_with_tooltip(
                    ui,
                    row_content_width,
                    row_height,
                    crate::i18n::tr_catalog(self.app_settings.language, "combo_editor.output_key"),
                    true,
                    Some(crate::i18n::tr_catalog(
                        self.app_settings.language,
                        "combo_editor.keycode_sent_when_the_combo_activates",
                    )),
                    control_width,
                    |ui| {
                        let resp = crate::ui_style::modern_button_with_font(
                            ui,
                            output_label.as_str(),
                            Vec2::new(control_width, control_height),
                            control_font_size,
                            true,
                        );
                        if resp.clicked() {
                            self.combo_pick_target = Some((combo_idx, ComboPickField::Output));
                            self.keycode_picker.result = None;
                            self.keycode_picker.selected_tab = KeycodeTab::Basic;
                            self.keycode_picker.open = true;
                        }
                    },
                );

                if let Some(current_combo_term) = self.combo_term {
                    let mut combo_term_text = current_combo_term.to_string();
                    crate::ui_style::settings_list_row_with_tooltip(
                        ui,
                        row_content_width,
                        row_height,
                        crate::i18n::tr_catalog(self.app_settings.language, "common.timeout"),
                        true,
                        Some(crate::i18n::tr_catalog(
                            self.app_settings.language,
                            "combo_editor.maximum_time_between_combo_key_presses",
                        )),
                        timeout_control_width,
                        |ui| {
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let resp = crate::ui_style::modern_text_field_sized(
                                        ui,
                                        egui::Id::new("combo_term"),
                                        &mut combo_term_text,
                                        70.0 * scale,
                                        control_height,
                                        "",
                                        4,
                                        egui::Align::RIGHT,
                                    );
                                    let resp = settings_field_unit_tooltip(
                                        resp,
                                        self.app_settings.language,
                                        false,
                                        SettingsFieldUnit::Milliseconds,
                                    );
                                    if resp.changed() {
                                        let filtered: String = combo_term_text
                                            .chars()
                                            .filter(|c| c.is_ascii_digit())
                                            .take(4)
                                            .collect();
                                        if let Ok(parsed) = filtered.parse::<u16>() {
                                            self.combo_undo_stack.push(combo_undo_snapshot.clone());
                                            self.combo_term = Some(parsed.max(1));
                                            self.combo_term_dirty = true;
                                        }
                                    }
                                },
                            );
                        },
                    );
                }
            },
        );

        ui.add_space(14.0 * scale);
        let action_size = crate::ui_style::modal_action_button_size() * scale;
        let action_width = action_size.x * 2.0 + 8.0 * scale;
        let action_rect = egui::Rect::from_min_size(
            egui::pos2(page_center_x - action_width / 2.0, ui.cursor().min.y),
            Vec2::new(action_width, action_size.y),
        );
        ui.allocate_ui_at_rect(action_rect, |ui| {
            ui.set_min_size(action_rect.size());
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 8.0 * scale;
                let clear_enabled = combo_idx < self.combo_entries.len()
                    && (self.combo_entries[combo_idx].keys.iter().any(|&k| k != 0)
                        || self.combo_entries[combo_idx].output != 0
                        || self
                            .combo_names
                            .get(combo_idx)
                            .map(|s| !s.trim().is_empty())
                            .unwrap_or(false));
                let clear_resp = crate::ui_style::modern_button_with_font(
                    ui,
                    crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.clear"),
                    action_size,
                    control_font_size,
                    clear_enabled,
                );
                if clear_resp.clicked() && clear_enabled {
                    self.push_combo_undo();
                    self.combo_entries[combo_idx] = ComboEntry::default();
                    if let Some(name) = self.combo_names.get_mut(combo_idx) {
                        name.clear();
                    }
                    self.combo_dirty = true;
                    self.combo_names_dirty = true;
                }
                let undo_enabled = !self.combo_undo_stack.is_empty();
                let undo_resp = crate::ui_style::modern_button_with_font(
                    ui,
                    crate::i18n::tr_catalog(self.app_settings.language, "alt_repeat_editor.undo"),
                    action_size,
                    control_font_size,
                    undo_enabled,
                );
                if undo_resp.clicked() && undo_enabled {
                    if let Some((entries, names, term, selected, visible_count)) =
                        self.combo_undo_stack.pop()
                    {
                        self.combo_entries = entries;
                        self.combo_names = names;
                        self.combo_term = term;
                        self.combo_visible_count =
                            visible_count.clamp(1, self.combo_entries.len().max(1));
                        self.selected_combo =
                            selected.min(self.combo_visible_count.saturating_sub(1));
                        self.combo_dirty = true;
                        self.combo_names_dirty = true;
                        self.combo_term_dirty = true;
                    }
                }
            });
        });
        ui.allocate_space(Vec2::new(1.0, action_size.y));
    }

    fn draw_ui_scale_controls(&mut self, ui: &mut egui::Ui, left_top: egui::Pos2) -> f32 {
        let height = 28.0;
        let minus_w = 24.0;
        let label_w = 52.0;
        let plus_w = 24.0;
        let gap = 4.0;
        let total_w = minus_w + label_w + plus_w + gap * 2.0;
        let text_color = ui.visuals().widgets.inactive.fg_stroke.color;
        let muted = app_muted_text(ui.visuals().dark_mode);
        let hover_fill = app_hover_fill(ui.visuals().dark_mode);
        let font = FontId::proportional(14.0);

        let draw_control =
            |ui: &mut egui::Ui, rect: egui::Rect, label: &str, enabled: bool| -> egui::Response {
                let response = ui.allocate_rect(rect, Sense::CLICK);
                if response.hovered() && enabled {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    ui.painter().rect_filled(rect, 7.0, hover_fill);
                }
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    font.clone(),
                    if enabled { text_color } else { muted },
                );
                response
            };

        let minus_rect = egui::Rect::from_min_size(left_top, Vec2::new(minus_w, height));
        let label_rect = egui::Rect::from_min_size(
            egui::pos2(minus_rect.right() + gap, left_top.y),
            Vec2::new(label_w, height),
        );
        let plus_rect = egui::Rect::from_min_size(
            egui::pos2(label_rect.right() + gap, left_top.y),
            Vec2::new(plus_w, height),
        );

        let can_decrease = self.app_settings.ui_scale > UI_SCALE_MIN + 0.001;
        let can_increase = self.app_settings.ui_scale < UI_SCALE_MAX - 0.001;
        if draw_control(ui, minus_rect, "−", can_decrease).clicked() && can_decrease {
            self.step_ui_scale(ui.ctx(), -1);
        }

        let label_response = ui.allocate_rect(label_rect, Sense::CLICK);
        if label_response.hovered() && self.ui_scale_percent() != 100 {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            ui.painter().rect_filled(label_rect, 7.0, hover_fill);
        }
        if label_response.clicked() {
            self.set_ui_scale(ui.ctx(), default_ui_scale());
        }
        ui.painter().text(
            label_rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("{}%", self.ui_scale_percent()),
            FontId::proportional(13.0),
            text_color,
        );

        if draw_control(ui, plus_rect, "+", can_increase).clicked() && can_increase {
            self.step_ui_scale(ui.ctx(), 1);
        }

        total_w
    }

    fn draw_layout(&mut self, ui: &mut egui::Ui, layout: &KeyboardLayout, ctx: &egui::Context) {
        let avail = ui.available_size();
        let viewport = egui::Rect::from_min_max(
            ui.min_rect().min,
            egui::pos2(ui.min_rect().left() + avail.x, ui.max_rect().bottom()),
        );
        let geometry = layout_geometry(
            ui.ctx(),
            layout,
            viewport,
            clamp_ui_scale(self.app_settings.ui_scale),
        );
        let offset_x = geometry.offset_x;
        let offset_y = geometry.offset_y;
        let unit = geometry.unit;
        let padding = geometry.padding;
        let layout_h = geometry.layout_h;
        let main_tabs_h = 32.0_f32;
        let layer_bar_h = 68.0_f32;
        let top_reserved_h = LAYOUT_TOP_RESERVED_H;
        let top_base_y = ui.min_rect().top() + 6.0;
        self.last_layout_geometry = Some((offset_x, offset_y, unit, padding));

        // ── Main menu tabs ────────────────────────────────────────────────
        {
            use crate::i18n::Key as TrKey;

            let lang = self.app_settings.language;
            let center_x = ui.min_rect().center().x;
            let tabs_y = top_base_y;
            let tab_font_size = 15.0;
            let tab_height = 28.0;
            let tab_gap = 16.0;
            let tabs = [
                (
                    MainMenuTab::Keyboard,
                    crate::i18n::tr(lang, TrKey::MainTabLayout),
                    "main_menu.layout_tooltip",
                ),
                (
                    MainMenuTab::Advanced,
                    crate::i18n::tr(lang, TrKey::MainTabAdvanced),
                    "main_menu.advanced_tooltip",
                ),
                (
                    MainMenuTab::Settings,
                    crate::i18n::tr(lang, TrKey::MainTabConfig),
                    "main_menu.settings_tooltip",
                ),
            ];
            let tab_widths = tabs.map(|(_, label, _)| {
                (top_menu_text_width(ui, label, tab_font_size) + 34.0).max(96.0)
            });
            let total_w = tab_widths.iter().sum::<f32>() + tab_gap * (tabs.len() - 1) as f32;
            let start_x = center_x - total_w / 2.0;
            let mut device_tab_rect = None;
            let mut device_tab_hovered = false;
            let mut advanced_tab_rect = None;
            let mut advanced_tab_hovered = false;
            let mut settings_tab_rect = None;
            let mut settings_tab_hovered = false;

            let mut tab_x = start_x;
            for (idx, (tab, label, tooltip)) in tabs.iter().enumerate() {
                let slot_rect = egui::Rect::from_min_size(
                    egui::pos2(tab_x, tabs_y),
                    Vec2::new(tab_widths[idx], tab_height),
                );
                tab_x += tab_widths[idx] + tab_gap;
                let resp = ui.allocate_rect(slot_rect, Sense::CLICK);
                resp.clone()
                    .on_hover_text(crate::i18n::tr_catalog(lang, tooltip));
                if matches!(tab, MainMenuTab::Keyboard) {
                    device_tab_rect = Some(slot_rect);
                    device_tab_hovered = resp.hovered();
                }
                if matches!(tab, MainMenuTab::Advanced) {
                    advanced_tab_rect = Some(slot_rect);
                    advanced_tab_hovered = resp.hovered();
                }
                if matches!(tab, MainMenuTab::Settings) {
                    settings_tab_rect = Some(slot_rect);
                    settings_tab_hovered = resp.hovered();
                }
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if resp.clicked() {
                    match tab {
                        MainMenuTab::Keyboard => {
                            self.main_menu_tab = MainMenuTab::Keyboard;
                        }
                        MainMenuTab::Advanced => {}
                        MainMenuTab::Settings => {
                            if self.main_menu_tab != MainMenuTab::Settings {
                                self.reset_matrix_tester_state();
                            }
                            self.matrix_tester_unlock_prompted = false;
                            self.matrix_tester_lock_checked = false;
                            self.main_menu_tab = MainMenuTab::Settings;
                        }
                    }
                }

                let is_active = self.main_menu_tab == *tab;
                let text_color = if is_active {
                    ui.visuals().widgets.inactive.fg_stroke.color
                } else if resp.hovered() {
                    if ui.visuals().dark_mode {
                        Color32::from_gray(135)
                    } else {
                        Color32::from_gray(120)
                    }
                } else if ui.visuals().dark_mode {
                    Color32::from_gray(90)
                } else {
                    Color32::from_gray(150)
                };

                ui.painter().text(
                    slot_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    *label,
                    FontId::proportional(tab_font_size),
                    text_color,
                );
            }

            self.register_tour_target(
                TourTarget::MainNavigation,
                egui::Rect::from_min_size(
                    egui::pos2(start_x, tabs_y),
                    Vec2::new(total_w, tab_height),
                ),
            );
            if let Some(device_rect) = device_tab_rect {
                self.register_tour_target(TourTarget::DeviceSelector, device_rect);
            }
            if let Some(settings_rect) = settings_tab_rect {
                self.register_tour_target(TourTarget::SettingsMenu, settings_rect);
            }

            let zoom_width = 108.0;
            let zoom_left_top = egui::pos2(ui.min_rect().right() - 18.0 - zoom_width, tabs_y);
            self.draw_ui_scale_controls(ui, zoom_left_top);

            let undo_enabled = !self.undo_stack.is_empty();
            let undo_label = crate::i18n::tr_catalog(lang, "alt_repeat_editor.undo_curved");
            let undo_font = FontId::proportional(14.0);
            let undo_text_w = ui.fonts(|f| {
                f.layout_no_wrap(
                    undo_label.to_owned(),
                    undo_font.clone(),
                    ui.visuals().widgets.inactive.fg_stroke.color,
                )
                .size()
                .x
            });
            let undo_rect = egui::Rect::from_min_size(
                egui::pos2(ui.min_rect().left() + 24.0, tabs_y),
                Vec2::new(undo_text_w + 12.0, tab_height),
            );
            let undo_resp = ui.allocate_rect(undo_rect, Sense::CLICK);
            if undo_enabled {
                undo_resp.clone().on_hover_text(crate::i18n::tr_catalog(
                    lang,
                    "key_picker_text.undo_last_change",
                ));
            }
            if undo_resp.hovered() && undo_enabled {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if undo_resp.clicked() && undo_enabled {
                self.undo();
                ctx.request_repaint();
            }
            let undo_color = if !undo_enabled {
                if ui.visuals().dark_mode {
                    Color32::from_gray(58)
                } else {
                    Color32::from_gray(178)
                }
            } else if undo_resp.hovered() {
                app_accent()
            } else {
                ui.visuals().widgets.inactive.fg_stroke.color
            };
            let undo_text_pos = egui::pos2(undo_rect.left() + 6.0, undo_rect.center().y);
            ui.painter().text(
                undo_text_pos,
                egui::Align2::LEFT_CENTER,
                undo_label,
                undo_font,
                undo_color,
            );

            let divider_color = if ui.visuals().dark_mode {
                Color32::from_gray(105)
            } else {
                Color32::from_gray(170)
            };
            let divider_top = tabs_y + 4.0;
            let divider_bottom = tabs_y + tab_height - 4.0;
            let mut divider_x = start_x;
            for width in tab_widths.iter().take(tabs.len() - 1) {
                divider_x += *width;
                let x = divider_x + tab_gap / 2.0;
                ui.painter().line_segment(
                    [egui::pos2(x, divider_top), egui::pos2(x, divider_bottom)],
                    egui::Stroke::new(1.5, divider_color),
                );
                divider_x += tab_gap;
            }

            if let Some(device_rect) = device_tab_rect {
                let dropdown_id = ui.make_persistent_id("device_dropdown_open");
                let was_open = ui
                    .ctx()
                    .data(|d| d.get_temp::<bool>(dropdown_id))
                    .unwrap_or(false);
                let has_lock_button = self.firmware == FirmwareProtocol::Vial
                    && self.layout.is_some()
                    && !self.vial_unlock_polling
                    && !self.unlock_open;
                let is_unlocked = if has_lock_button {
                    self.hid_device
                        .as_ref()
                        .and_then(|hid| hid.get_unlock_status().ok())
                        .map(|(unlocked, _keys)| unlocked)
                        .unwrap_or(false)
                } else {
                    false
                };
                let device_count = self.device_manager.devices().len();
                let device_rows = device_count.max(1) as f32;
                let devices_h = 12.0 + device_rows * 30.0;
                let lock_h = if has_lock_button { 36.0 } else { 0.0 };
                let show_key_legend_switcher =
                    self.app_settings.key_legend_layout.is_multilingual();
                let key_legend_switcher_h = if show_key_legend_switcher { 36.0 } else { 0.0 };
                let mut device_menu_labels: Vec<String> =
                    if self.device_manager.devices().is_empty() {
                        vec![crate::i18n::tr(lang, TrKey::NoDevicesFound).to_owned()]
                    } else {
                        self.device_manager
                            .devices()
                            .iter()
                            .map(|dev| {
                                self.device_display_names
                                    .get(&dev.path)
                                    .cloned()
                                    .unwrap_or_else(|| dev.name.clone())
                            })
                            .collect()
                    };
                if has_lock_button {
                    let action = if is_unlocked {
                        crate::i18n::tr(lang, TrKey::LockAction)
                    } else {
                        crate::i18n::tr(lang, TrKey::UnlockAction)
                    };
                    let icon = if is_unlocked { "🔓" } else { "🔒" };
                    device_menu_labels.push(format!("{icon} {action}"));
                }
                if show_key_legend_switcher {
                    if let Some(order_key) = self.app_settings.key_legend_layout.order_i18n_key() {
                        device_menu_labels
                            .push(crate::i18n::tr_catalog(lang, order_key).to_owned());
                    }
                }
                let dropdown_size = Vec2::new(
                    adaptive_top_dropdown_width(
                        ui,
                        device_menu_labels.iter().map(String::as_str),
                        152.0,
                    ),
                    devices_h + lock_h + key_legend_switcher_h + 12.0,
                );
                let dropdown_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        device_rect.center().x - dropdown_size.x / 2.0,
                        device_rect.bottom() + 6.0,
                    ),
                    dropdown_size,
                );
                let hover_bridge_rect = device_rect.union(dropdown_rect).expand(3.0);
                let pointer_over_bridge = ui
                    .ctx()
                    .input(|i| i.pointer.hover_pos())
                    .map(|pos| hover_bridge_rect.contains(pos))
                    .unwrap_or(false);
                let show_dropdown = !advanced_tab_hovered
                    && !settings_tab_hovered
                    && (device_tab_hovered || (was_open && pointer_over_bridge));

                if show_dropdown {
                    let area_id = ui.make_persistent_id("device_dropdown_area");
                    let mut device_clicked = false;
                    egui::Area::new(area_id)
                        .order(egui::Order::Foreground)
                        .fixed_pos(dropdown_rect.min)
                        .show(ctx, |ui| {
                            let dark = ui.visuals().dark_mode;
                            top_dropdown_frame(dark).show(ui, |ui| {
                                ui.set_min_width(dropdown_size.x - 16.0);

                                let prev_selected = self.selected_device;
                                if self.device_manager.devices().is_empty() {
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(dropdown_size.x - 16.0, 30.0),
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            ui.add_space(10.0);
                                            ui.label(
                                                RichText::new(crate::i18n::tr(
                                                    lang,
                                                    TrKey::NoDevicesFound,
                                                ))
                                                .size(13.0)
                                                .color(app_muted_text(ui.visuals().dark_mode)),
                                            );
                                        },
                                    );
                                } else {
                                    for (i, dev) in self.device_manager.devices().iter().enumerate()
                                    {
                                        let is_selected = self.selected_device == Some(i);
                                        let cached_display_name = self
                                            .device_display_names
                                            .get(&dev.path)
                                            .map(String::as_str);
                                        let display_name =
                                            cached_display_name.unwrap_or(dev.name.as_str());
                                        let resp = top_dropdown_item(
                                            ui,
                                            dropdown_size.x - 16.0,
                                            display_name,
                                            true,
                                            is_selected,
                                        );
                                        if resp.clicked() {
                                            self.selected_device = Some(i);
                                            self.main_menu_tab = MainMenuTab::Keyboard;
                                            device_clicked = true;
                                        }
                                    }
                                }

                                #[cfg(not(target_arch = "wasm32"))]
                                if self.selected_device != prev_selected {
                                    if let Some(idx) = self.selected_device {
                                        self.start_connect(idx);
                                    }
                                }

                                if has_lock_button {
                                    ui.add_space(6.0);
                                    let lock_label = if is_unlocked {
                                        format!("🔓 {}", crate::i18n::tr(lang, TrKey::LockAction))
                                    } else {
                                        format!("🔒 {}", crate::i18n::tr(lang, TrKey::UnlockAction))
                                    };
                                    if top_dropdown_item(
                                        ui,
                                        dropdown_size.x - 16.0,
                                        &lock_label,
                                        true,
                                        false,
                                    )
                                    .clicked()
                                    {
                                        if is_unlocked {
                                            if let Some(hid) = &self.hid_device {
                                                match hid.lock() {
                                                    Ok(()) => {
                                                        self.status_msg = "Keyboard locked".into()
                                                    }
                                                    Err(e) => {
                                                        self.status_msg =
                                                            format!("Lock failed: {e}")
                                                    }
                                                }
                                            }
                                        } else {
                                            self.unlock_open = true;
                                        }
                                    }
                                }

                                if show_key_legend_switcher {
                                    if let Some(order_key) =
                                        self.app_settings.key_legend_layout.order_i18n_key()
                                    {
                                        ui.add_space(6.0);
                                        let order_label = crate::i18n::tr_catalog(lang, order_key);
                                        if top_dropdown_item(
                                            ui,
                                            dropdown_size.x - 16.0,
                                            order_label,
                                            true,
                                            false,
                                        )
                                        .clicked()
                                        {
                                            self.app_settings.key_legend_layout =
                                                self.app_settings.key_legend_layout.toggled_order();
                                            save_app_settings(&self.app_settings);
                                            ctx.request_repaint();
                                        }
                                    }
                                }
                            });
                        });

                    ui.ctx().data_mut(|d| {
                        d.insert_temp(
                            dropdown_id,
                            !device_clicked && (device_tab_hovered || pointer_over_bridge),
                        )
                    });
                } else {
                    ui.ctx().data_mut(|d| d.insert_temp(dropdown_id, false));
                }
            }

            if let Some(advanced_rect) = advanced_tab_rect {
                let dropdown_id = ui.make_persistent_id("advanced_dropdown_open");
                let was_open = ui
                    .ctx()
                    .data(|d| d.get_temp::<bool>(dropdown_id))
                    .unwrap_or(false);
                let combo_supported = !self.combo_entries.is_empty();
                let key_override_supported = !self.key_override_entries.is_empty();
                let auto_shift_supported = self.auto_shift_timeout.is_some();
                let advanced_item_count = 1
                    + combo_supported as usize
                    + auto_shift_supported as usize
                    + key_override_supported as usize;
                let mut advanced_menu_labels =
                    vec![crate::i18n::tr_catalog(lang, "text_expander.title")];
                if combo_supported {
                    advanced_menu_labels.push(crate::i18n::tr(lang, TrKey::ComboTitle));
                }
                if auto_shift_supported {
                    advanced_menu_labels.push(crate::i18n::tr(lang, TrKey::AutoShiftTitle));
                }
                if key_override_supported {
                    advanced_menu_labels.push(crate::i18n::tr(lang, TrKey::KeyOverridesTitle));
                }
                let advanced_dropdown_width =
                    adaptive_top_dropdown_width(ui, advanced_menu_labels, 152.0);
                let dropdown_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        advanced_rect.center().x - advanced_dropdown_width / 2.0,
                        advanced_rect.bottom() + 6.0,
                    ),
                    Vec2::new(
                        advanced_dropdown_width,
                        (advanced_item_count.max(1) as f32) * 28.0 + 22.0,
                    ),
                );
                let hover_bridge_rect = advanced_rect.union(dropdown_rect).expand(3.0);
                let pointer_over_bridge = ui
                    .ctx()
                    .input(|i| i.pointer.hover_pos())
                    .map(|pos| hover_bridge_rect.contains(pos))
                    .unwrap_or(false);
                let show_dropdown = advanced_item_count > 0
                    && !device_tab_hovered
                    && !settings_tab_hovered
                    && (advanced_tab_hovered || (was_open && pointer_over_bridge));

                if show_dropdown {
                    let dark = ui.visuals().dark_mode;
                    let item_width = dropdown_rect.width() - 16.0;
                    let (
                        text_expander_hovered,
                        combo_hovered,
                        auto_shift_hovered,
                        key_override_hovered,
                        advanced_clicked,
                    ) = egui::Area::new(egui::Id::new("advanced_dropdown_area"))
                        .order(egui::Order::Foreground)
                        .fixed_pos(dropdown_rect.min)
                        .show(ui.ctx(), |ui| {
                            top_dropdown_frame(dark)
                                .show(ui, |ui| {
                                    ui.set_min_width(item_width);
                                    let text_expander_resp = top_dropdown_item(
                                        ui,
                                        item_width,
                                        crate::i18n::tr_catalog(lang, "text_expander.title"),
                                        true,
                                        self.main_menu_tab == MainMenuTab::Advanced
                                            && self.settings_tab == SettingsTab::TextExpander,
                                    );
                                    let combo_resp = combo_supported.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::ComboTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Advanced
                                                && self.settings_tab == SettingsTab::Combo,
                                        )
                                    });
                                    let auto_shift_resp = auto_shift_supported.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::AutoShiftTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Advanced
                                                && self.settings_tab == SettingsTab::AutoShift,
                                        )
                                    });
                                    let key_override_resp = key_override_supported.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::KeyOverridesTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Advanced
                                                && self.settings_tab == SettingsTab::KeyOverrides,
                                        )
                                    });
                                    if text_expander_resp.clicked() {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_text_expander_settings_page();
                                    }
                                    if combo_resp.as_ref().map(|r| r.clicked()).unwrap_or(false) {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.settings_tab = SettingsTab::Combo;
                                        self.main_menu_tab = MainMenuTab::Advanced;
                                        if self.combo_visible_count == 0 {
                                            self.combo_visible_count = 1;
                                        }
                                    }
                                    if auto_shift_resp
                                        .as_ref()
                                        .map(|r| r.clicked())
                                        .unwrap_or(false)
                                    {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.settings_tab = SettingsTab::AutoShift;
                                        self.main_menu_tab = MainMenuTab::Advanced;
                                    }
                                    if key_override_resp
                                        .as_ref()
                                        .map(|r| r.clicked())
                                        .unwrap_or(false)
                                    {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.settings_tab = SettingsTab::KeyOverrides;
                                        self.main_menu_tab = MainMenuTab::Advanced;
                                    }
                                    (
                                        text_expander_resp.hovered(),
                                        combo_resp.as_ref().map(|r| r.hovered()).unwrap_or(false),
                                        auto_shift_resp
                                            .as_ref()
                                            .map(|r| r.hovered())
                                            .unwrap_or(false),
                                        key_override_resp
                                            .as_ref()
                                            .map(|r| r.hovered())
                                            .unwrap_or(false),
                                        text_expander_resp.clicked()
                                            || combo_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || auto_shift_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || key_override_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false),
                                    )
                                })
                                .inner
                        })
                        .inner;
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(
                            dropdown_id,
                            !advanced_clicked
                                && (advanced_tab_hovered
                                    || text_expander_hovered
                                    || combo_hovered
                                    || auto_shift_hovered
                                    || key_override_hovered
                                    || pointer_over_bridge),
                        )
                    });
                } else {
                    ui.ctx().data_mut(|d| d.insert_temp(dropdown_id, false));
                }
            }

            if let Some(settings_rect) = settings_tab_rect {
                let dropdown_id = ui.make_persistent_id("settings_dropdown_open");
                let was_open = ui
                    .ctx()
                    .data(|d| d.get_temp::<bool>(dropdown_id))
                    .unwrap_or(false);
                let rgb_available_for_menu = self.rgb_settings.supported || layout.supports_rgb;
                let layer_leds_available_for_menu = self.layer_led_settings.supported;
                let show_rgb_item = rgb_available_for_menu;
                let show_layer_leds_item = layer_leds_available_for_menu;
                let show_encoders_item = layout.encoder_count() > 0;
                let show_layout_options_item = layout
                    .layout_options
                    .iter()
                    .any(|option| !Self::is_encoder_layout_option(option));
                let show_modules_item = self.module_settings.supported;
                let show_touchpad_item = self.touchpad_settings.supported;
                let show_live_features_item = self.live_features_available_for_selected_device();
                let show_magic_item = self.magic_settings.supported;
                let show_tap_hold_item =
                    self.tap_hold_settings.supported || self.one_shot_settings.supported;
                let show_matrix_item = self.firmware == FirmwareProtocol::Vial;
                let settings_item_count = 2
                    + show_matrix_item as usize
                    + show_rgb_item as usize
                    + show_layer_leds_item as usize
                    + show_encoders_item as usize
                    + show_layout_options_item as usize
                    + show_modules_item as usize
                    + show_touchpad_item as usize
                    + show_live_features_item as usize
                    + show_magic_item as usize
                    + show_tap_hold_item as usize;
                // Keep hover bridge in sync with actual item height (30px) and frame padding.
                // Underestimating this makes lower items close the dropdown on hover.
                let dropdown_height = settings_item_count as f32 * 30.0 + 12.0;
                let mut settings_menu_labels = vec![
                    crate::i18n::tr(lang, TrKey::AppSettingsTitle),
                    crate::i18n::tr(lang, TrKey::UniversalSymbolsTitle),
                ];
                if show_matrix_item {
                    settings_menu_labels.push(crate::i18n::tr(lang, TrKey::MatrixTesterTitle));
                }
                if show_rgb_item {
                    settings_menu_labels.push(crate::i18n::tr(lang, TrKey::RgbTitle));
                }
                if show_layer_leds_item {
                    settings_menu_labels.push(crate::i18n::tr(lang, TrKey::LayerLedsTitle));
                }
                if show_encoders_item {
                    settings_menu_labels.push(crate::i18n::tr(lang, TrKey::EncodersTitle));
                }
                if show_layout_options_item {
                    settings_menu_labels.push(crate::i18n::tr(lang, TrKey::DisplayPresetsTitle));
                }
                if show_modules_item {
                    settings_menu_labels
                        .push(crate::i18n::tr_catalog(lang, "modules_settings.title"));
                }
                if show_touchpad_item {
                    settings_menu_labels.push(crate::i18n::tr(lang, TrKey::TouchpadTitle));
                }
                if show_live_features_item {
                    settings_menu_labels.push(crate::i18n::tr(lang, TrKey::LiveFeaturesTitle));
                }
                if show_magic_item {
                    settings_menu_labels.push(crate::i18n::tr(lang, TrKey::MagicTitle));
                }
                if show_tap_hold_item {
                    settings_menu_labels.push(crate::i18n::tr(lang, TrKey::TapHoldOneShotTitle));
                }
                let dropdown_width = adaptive_top_dropdown_width(ui, settings_menu_labels, 184.0);
                let dropdown_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        settings_rect.center().x - dropdown_width / 2.0,
                        settings_rect.bottom() + 6.0,
                    ),
                    Vec2::new(dropdown_width, dropdown_height),
                );
                let hover_bridge_rect = settings_rect.union(dropdown_rect).expand(3.0);
                let pointer_over_bridge = ui
                    .ctx()
                    .input(|i| i.pointer.hover_pos())
                    .map(|pos| hover_bridge_rect.contains(pos))
                    .unwrap_or(false);
                let show_dropdown = !device_tab_hovered
                    && !advanced_tab_hovered
                    && (settings_tab_hovered || (was_open && pointer_over_bridge));

                if show_dropdown {
                    let dark = ui.visuals().dark_mode;
                    let rgb_available = rgb_available_for_menu;
                    let item_width = dropdown_rect.width() - 16.0;
                    let (
                        app_hovered,
                        matrix_hovered,
                        universal_symbols_hovered,
                        rgb_hovered,
                        layer_leds_hovered,
                        encoders_hovered,
                        layout_options_hovered,
                        modules_hovered,
                        touchpad_hovered,
                        live_features_hovered,
                        magic_hovered,
                        tap_hold_hovered,
                        settings_clicked,
                    ) = egui::Area::new(egui::Id::new("settings_dropdown_area"))
                        .order(egui::Order::Foreground)
                        .fixed_pos(dropdown_rect.min)
                        .show(ui.ctx(), |ui| {
                            top_dropdown_frame(dark)
                                .show(ui, |ui| {
                                    ui.set_min_width(item_width);
                                    ui.spacing_mut().item_spacing.y = 0.0;
                                    let app_resp = top_dropdown_item(
                                        ui,
                                        item_width,
                                        crate::i18n::tr(lang, TrKey::AppSettingsTitle),
                                        true,
                                        self.main_menu_tab == MainMenuTab::Settings
                                            && self.settings_tab == SettingsTab::AppSettings,
                                    );
                                    let matrix_resp = show_matrix_item.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::MatrixTesterTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::MatrixTester,
                                        )
                                    });
                                    let universal_symbols_resp = top_dropdown_item(
                                        ui,
                                        item_width,
                                        crate::i18n::tr(lang, TrKey::UniversalSymbolsTitle),
                                        true,
                                        self.main_menu_tab == MainMenuTab::Settings
                                            && self.settings_tab
                                                == SettingsTab::UniversalSymbolsSetup,
                                    );
                                    let rgb_resp = if show_rgb_item {
                                        Some(top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::RgbTitle),
                                            rgb_available,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::Rgb,
                                        ))
                                    } else {
                                        None
                                    };
                                    let layer_leds_resp = show_layer_leds_item.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::LayerLedsTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::LayerLeds,
                                        )
                                    });
                                    let encoders_resp = show_encoders_item.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::EncodersTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::Encoders,
                                        )
                                    });
                                    let layout_options_resp = show_layout_options_item.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::DisplayPresetsTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::LayoutOptions,
                                        )
                                    });
                                    let modules_resp = show_modules_item.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr_catalog(lang, "modules_settings.title"),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::Modules,
                                        )
                                    });
                                    let touchpad_resp = show_touchpad_item.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::TouchpadTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::Touchpad,
                                        )
                                    });
                                    let live_features_resp = show_live_features_item.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::LiveFeaturesTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::LiveFeatures,
                                        )
                                    });
                                    let magic_resp = show_magic_item.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::MagicTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::Magic,
                                        )
                                    });
                                    let tap_hold_resp = show_tap_hold_item.then(|| {
                                        top_dropdown_item(
                                            ui,
                                            item_width,
                                            crate::i18n::tr(lang, TrKey::TapHoldOneShotTitle),
                                            true,
                                            self.main_menu_tab == MainMenuTab::Settings
                                                && self.settings_tab == SettingsTab::TapHold,
                                        )
                                    });
                                    if app_resp.clicked() {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_app_settings_page();
                                    }
                                    if matrix_resp.as_ref().map(|r| r.clicked()).unwrap_or(false) {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.settings_tab = SettingsTab::MatrixTester;
                                        if self.main_menu_tab != MainMenuTab::Settings {
                                            self.reset_matrix_tester_state();
                                        }
                                        self.matrix_tester_unlock_prompted = false;
                                        self.matrix_tester_lock_checked = false;
                                        self.main_menu_tab = MainMenuTab::Settings;
                                    }
                                    if universal_symbols_resp.clicked() {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_universal_symbols_setup_page();
                                    }
                                    if let Some(rgb_resp) = &rgb_resp {
                                        if rgb_resp.clicked() && rgb_available {
                                            self.close_top_dropdowns(ui.ctx());
                                            self.settings_tab = SettingsTab::Rgb;
                                            self.main_menu_tab = MainMenuTab::Settings;
                                        }
                                        if !rgb_available {
                                            let _ = rgb_resp.clone().on_hover_text(
                                                crate::i18n::tr(lang, TrKey::RgbUnavailableTooltip),
                                            );
                                        }
                                    }
                                    if layer_leds_resp
                                        .as_ref()
                                        .map(|r| r.clicked())
                                        .unwrap_or(false)
                                    {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_layer_led_settings_page();
                                    }
                                    if encoders_resp.as_ref().map(|r| r.clicked()).unwrap_or(false)
                                    {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.settings_tab = SettingsTab::Encoders;
                                        self.main_menu_tab = MainMenuTab::Settings;
                                    }
                                    if layout_options_resp
                                        .as_ref()
                                        .map(|r| r.clicked())
                                        .unwrap_or(false)
                                    {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_layout_options_settings_page();
                                    }
                                    if modules_resp.as_ref().map(|r| r.clicked()).unwrap_or(false) {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_modules_settings_page();
                                    }
                                    if touchpad_resp.as_ref().map(|r| r.clicked()).unwrap_or(false)
                                    {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_touchpad_settings_page();
                                    }
                                    if live_features_resp
                                        .as_ref()
                                        .map(|r| r.clicked())
                                        .unwrap_or(false)
                                    {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_live_features_settings_page();
                                    }
                                    if magic_resp.as_ref().map(|r| r.clicked()).unwrap_or(false) {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_magic_settings_page();
                                    }
                                    if tap_hold_resp.as_ref().map(|r| r.clicked()).unwrap_or(false)
                                    {
                                        self.close_top_dropdowns(ui.ctx());
                                        self.open_tap_hold_settings_page();
                                    }
                                    (
                                        app_resp.hovered(),
                                        matrix_resp.as_ref().map(|r| r.hovered()).unwrap_or(false),
                                        universal_symbols_resp.hovered(),
                                        rgb_resp
                                            .as_ref()
                                            .map(|resp| resp.hovered())
                                            .unwrap_or(false),
                                        layer_leds_resp
                                            .as_ref()
                                            .map(|r| r.hovered())
                                            .unwrap_or(false),
                                        encoders_resp
                                            .as_ref()
                                            .map(|r| r.hovered())
                                            .unwrap_or(false),
                                        layout_options_resp
                                            .as_ref()
                                            .map(|r| r.hovered())
                                            .unwrap_or(false),
                                        modules_resp.as_ref().map(|r| r.hovered()).unwrap_or(false),
                                        touchpad_resp
                                            .as_ref()
                                            .map(|r| r.hovered())
                                            .unwrap_or(false),
                                        live_features_resp
                                            .as_ref()
                                            .map(|r| r.hovered())
                                            .unwrap_or(false),
                                        magic_resp.as_ref().map(|r| r.hovered()).unwrap_or(false),
                                        tap_hold_resp
                                            .as_ref()
                                            .map(|r| r.hovered())
                                            .unwrap_or(false),
                                        app_resp.clicked()
                                            || matrix_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || universal_symbols_resp.clicked()
                                            || rgb_resp
                                                .as_ref()
                                                .map(|resp| resp.clicked() && rgb_available)
                                                .unwrap_or(false)
                                            || layer_leds_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || encoders_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || layout_options_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || modules_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || touchpad_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || live_features_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || magic_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false)
                                            || tap_hold_resp
                                                .as_ref()
                                                .map(|r| r.clicked())
                                                .unwrap_or(false),
                                    )
                                })
                                .inner
                        })
                        .inner;
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(
                            dropdown_id,
                            !settings_clicked
                                && (settings_tab_hovered
                                    || app_hovered
                                    || matrix_hovered
                                    || universal_symbols_hovered
                                    || rgb_hovered
                                    || layer_leds_hovered
                                    || encoders_hovered
                                    || layout_options_hovered
                                    || modules_hovered
                                    || touchpad_hovered
                                    || live_features_hovered
                                    || magic_hovered
                                    || tap_hold_hovered
                                    || pointer_over_bridge),
                        )
                    });
                } else {
                    ui.ctx().data_mut(|d| d.insert_temp(dropdown_id, false));
                }
            }
            if matches!(
                self.main_menu_tab,
                MainMenuTab::Settings | MainMenuTab::Advanced
            ) {
                self.draw_settings_screen(ui, layout, ctx, ui.min_rect().top() + top_reserved_h);
                return;
            }

            // ── Layer switcher ─────────────────────────────────────────────────
            {
                let layer_count = self.layer_count;
                let selected = self.selected_layer;
                // raw_name — чистое имя без префикса, хранится в layer_names
                let raw_name = self
                    .layer_names
                    .get(selected)
                    .cloned()
                    .unwrap_or_else(|| selected.to_string());
                let visible_raw_name: String = raw_name.chars().take(12).collect();
                // display_name — с префиксом для отображения
                let display_name = if !raw_name.is_empty() && raw_name != selected.to_string() {
                    format!("{}. {}", selected, visible_raw_name)
                } else {
                    visible_raw_name.clone()
                };
                let name = display_name;
                let center_x = ui.max_rect().center().x;
                let bar_y = top_base_y + main_tabs_h + 24.0;
                let any_top_dropdown_open = ui.memory(|m| {
                    m.data
                        .get_temp::<bool>(ui.make_persistent_id("device_dropdown_open"))
                        .unwrap_or(false)
                        || m.data
                            .get_temp::<bool>(ui.make_persistent_id("advanced_dropdown_open"))
                            .unwrap_or(false)
                        || m.data
                            .get_temp::<bool>(ui.make_persistent_id("settings_dropdown_open"))
                            .unwrap_or(false)
                });

                // Layer name / edit field
                let name_rect = egui::Rect::from_min_size(
                    egui::pos2(center_x - 85.0, bar_y),
                    Vec2::new(170.0, 52.0),
                );
                self.register_tour_target(
                    TourTarget::LayerSwitcher,
                    name_rect.expand2(Vec2::new(72.0, 8.0)),
                );

                let display_name_len = visible_raw_name.chars().count();
                let display_label_size = if display_name_len > 10 {
                    26.0
                } else if display_name_len > 7 {
                    31.0
                } else {
                    39.0
                };
                let label_font = egui::FontId {
                    size: display_label_size,
                    family: egui::FontFamily::Proportional,
                };
                let text_color = if self.dark_mode {
                    Color32::from_gray(245)
                } else {
                    Color32::from_gray(60)
                };

                if self.editing_layer == Some(selected) {
                    // Limit input to 12 chars
                    if self.editing_layer_text.chars().count() > 12 {
                        let s: String = self.editing_layer_text.chars().take(12).collect();
                        self.editing_layer_text = s;
                    }
                    let editing_font = egui::FontId {
                        size: 39.0,
                        family: egui::FontFamily::Proportional,
                    };
                    let resp = ui.put(
                        name_rect,
                        egui::TextEdit::singleline(&mut self.editing_layer_text)
                            .font(editing_font)
                            .horizontal_align(egui::Align::Center)
                            .char_limit(12)
                            .frame(false),
                    );
                    // Request focus only on the first frame so lost_focus() works correctly.
                    if !self.editing_layer_focus_requested {
                        resp.request_focus();
                        self.editing_layer_focus_requested = true;
                    }
                    // Commit on Enter or lost focus (click outside); cancel on Escape.
                    let commit =
                        resp.lost_focus() || ui.input(|inp| inp.key_pressed(egui::Key::Enter));
                    let cancel = ui.input(|inp| inp.key_pressed(egui::Key::Escape));
                    if commit || cancel {
                        if commit {
                            let proposed_name = self.editing_layer_text.trim().to_string();
                            if proposed_name.is_empty() {
                                self.editing_layer_text = raw_name.clone();
                            } else {
                                let new_name = proposed_name;
                                while self.layer_names.len() <= selected {
                                    self.layer_names.push(self.layer_names.len().to_string());
                                }
                                self.layer_names[selected] = new_name.clone();
                                #[cfg(not(target_arch = "wasm32"))]
                                save_layer_names(&self.layer_names, &self.current_device_name);
                                #[cfg(target_arch = "wasm32")]
                                save_layer_names(&self.layer_names, "default");
                                // Also write name back to the connected device
                                #[cfg(not(target_arch = "wasm32"))]
                                if self.firmware == FirmwareProtocol::Vial {
                                    if let Some(dev) = &self.hid_device {
                                        if let Err(e) = dev.set_qmk_setting_string(
                                            200 + selected as u16,
                                            &new_name,
                                        ) {
                                            log::warn!(
                                            "Vial set_qmk_setting_string failed for layer {}: {}",
                                            selected,
                                            e
                                        );
                                        }
                                    }
                                }
                            }
                        }
                        self.editing_layer = None;
                        self.editing_layer_focus_requested = false;
                    }
                } else {
                    let mid_y = bar_y + layer_bar_h / 2.0;

                    // Fixed arrow positions based on max 7-char name width so
                    // arrows never jump around as the layer name changes.
                    // name_rect is 170px wide → half = 85px; gap keeps arrows clear.
                    let fixed_half = 85.0_f32;
                    let gap = 16.0_f32;
                    let arrow_y = mid_y - 2.0;
                    let left_center = egui::pos2(center_x - fixed_half - gap - 24.0, arrow_y);
                    let right_center = egui::pos2(center_x + fixed_half + gap + 24.0, arrow_y);

                    // Still measure actual text width for painting the name and edit icon.
                    let text_w = ui.fonts(|f| {
                        f.layout_no_wrap(name.clone(), label_font.clone(), text_color)
                            .size()
                            .x
                    });

                    // Allocate name FIRST — arrows are allocated last and win in egui's
                    // hit-test order (last allocation = highest priority).
                    let name_hit = egui::Rect::from_center_size(
                        egui::pos2(center_x, mid_y),
                        Vec2::new(text_w + 12.0, 52.0),
                    );
                    let name_r = ui.allocate_rect(name_hit, Sense::click());

                    // Full layer switch zone from arrow to arrow for mouse wheel switching.
                    // Keep click/hover hitboxes close to the actual arrow glyph size.
                    let left_hit = egui::Rect::from_center_size(left_center, Vec2::new(28.0, 44.0));
                    let right_hit =
                        egui::Rect::from_center_size(right_center, Vec2::new(28.0, 44.0));
                    let wheel_hit = egui::Rect::from_min_max(
                        egui::pos2(left_hit.left(), mid_y - 26.0),
                        egui::pos2(right_hit.right(), mid_y + 26.0),
                    );
                    let wheel_r = ui.allocate_rect(wheel_hit, Sense::hover());

                    // Scroll wheel over the whole layer bar switches layers (down = next, up = prev)
                    if wheel_r.hovered() {
                        let scroll = ui.input(|i| i.raw_scroll_delta.y);
                        if scroll < 0.0 && selected > 0 {
                            self.selected_layer = selected - 1;
                        } else if scroll > 0.0 && selected + 1 < layer_count {
                            self.selected_layer = selected + 1;
                        }
                    }

                    // Allocate arrows LAST so they have click priority over the name rect.
                    let left_r = ui.allocate_rect(left_hit, Sense::click());
                    let right_r = ui.allocate_rect(right_hit, Sense::click());
                    if left_r.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if right_r.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if left_r.clicked() && selected > 0 {
                        self.selected_layer = selected - 1;
                        self.jump_back_stack.clear();
                    }
                    if right_r.clicked() && selected + 1 < layer_count {
                        self.selected_layer = selected + 1;
                        self.jump_back_stack.clear();
                    }
                    if name_r.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if name_r.clicked() {
                        self.editing_layer = Some(selected);
                        self.editing_layer_text = raw_name.clone();
                    }

                    // Paint
                    let dis = if self.dark_mode {
                        Color32::from_gray(60)
                    } else {
                        Color32::from_gray(200)
                    };
                    let ac_l = if left_r.hovered() {
                        app_accent()
                    } else if self.dark_mode {
                        Color32::from_gray(140)
                    } else {
                        Color32::from_gray(120)
                    };
                    let ac_r = if right_r.hovered() {
                        app_accent()
                    } else if self.dark_mode {
                        Color32::from_gray(140)
                    } else {
                        Color32::from_gray(120)
                    };
                    ui.painter().text(
                        left_center,
                        egui::Align2::CENTER_CENTER,
                        "‹",
                        FontId::proportional(52.0),
                        if selected == 0 { dis } else { ac_l },
                    );
                    ui.painter().text(
                        right_center,
                        egui::Align2::CENTER_CENTER,
                        "›",
                        FontId::proportional(52.0),
                        if selected + 1 >= layer_count {
                            dis
                        } else {
                            ac_r
                        },
                    );
                    ui.painter().text(
                        egui::pos2(center_x, mid_y),
                        egui::Align2::CENTER_CENTER,
                        &name,
                        label_font,
                        text_color,
                    );

                    // Hint text below layer name
                    let hint_color = if self.dark_mode {
                        Color32::from_gray(100)
                    } else {
                        Color32::from_gray(160)
                    };
                    let hint_font = FontId::proportional(11.0);
                    let secondary_hint_font = hint_font.clone();
                    let hint_y = ui.max_rect().bottom() - 36.0;
                    let any_hovered = self.prev_hovered_key.is_some() || self.prev_hovered_encoder;
                    let hint_language = self.app_settings.language;
                    let tr_hint = |key: &'static str| crate::i18n::tr_catalog(hint_language, key);
                    if let Some(hl) = self.hover_layer {
                        let hl_name = self
                            .layer_names
                            .get(hl)
                            .cloned()
                            .unwrap_or_else(|| hl.to_string());
                        let mut line = 0i32;
                        let line_h = 13.0f32;
                        let base_y = hint_y - 15.0;
                        // Line 1: always
                        ui.painter().text(
                            egui::pos2(center_x, base_y + line as f32 * line_h),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.change_key"),
                            hint_font.clone(),
                            hint_color,
                        );
                        line += 1;
                        // Line 2: go to layer (if not current)
                        if hl != self.selected_layer {
                            let layer_index = hl.to_string();
                            let go_to_layer_hint = crate::i18n::tr_catalog_format(
                                hint_language,
                                "key_hints.go_to_layer",
                                &[("layer", layer_index.as_str()), ("name", hl_name.as_str())],
                            );
                            ui.painter().text(
                                egui::pos2(center_x, base_y + line as f32 * line_h),
                                egui::Align2::CENTER_CENTER,
                                go_to_layer_hint,
                                hint_font.clone(),
                                hint_color,
                            );
                            line += 1;
                        }
                        // Line 3: change layer number
                        ui.painter().text(
                            egui::pos2(center_x, base_y + line as f32 * line_h),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.change_layer_number"),
                            hint_font.clone(),
                            hint_color,
                        );
                        line += 1;
                        // Line 4: go back (if in jump mode)
                        if !self.jump_back_stack.is_empty() {
                            ui.painter().text(
                                egui::pos2(center_x, base_y + line as f32 * line_h),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.esc_back"),
                                hint_font.clone(),
                                hint_color,
                            );
                        }
                        let _ = hint_font;
                    } else if !self.jump_back_stack.is_empty() {
                        if any_hovered {
                            ui.painter().text(
                                egui::pos2(center_x, hint_y - 9.0),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.change_key"),
                                hint_font.clone(),
                                hint_color,
                            );
                        }
                        ui.painter().text(
                            egui::pos2(center_x, if any_hovered { hint_y + 5.0 } else { hint_y }),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.right_click_or_esc_back"),
                            hint_font,
                            hint_color,
                        );
                    } else if any_hovered {
                        // Check if hovered key is a mod key
                        let (
                            hovered_is_mod,
                            hovered_can_swap_side,
                            hovered_can_retarget_mod_key,
                            hovered_is_macro,
                            hovered_is_tap_dance,
                            hovered_is_mouse,
                            hovered_is_alt_repeat,
                            hovered_is_grave_escape,
                            hovered_is_layer,
                        ) = {
                            let hint_kc = self
                                .prev_hovered_key
                                .and_then(|ki| {
                                    self.layout
                                        .as_ref()
                                        .map(|l| l.get_keycode(self.selected_layer, ki))
                                })
                                .or(self.prev_hovered_encoder_keycode)
                                .or_else(|| {
                                    self.selected_key.and_then(|(selected_layer, selected_ki)| {
                                        (selected_layer == self.selected_layer)
                                            .then(|| {
                                                self.layout.as_ref().map(|l| {
                                                    l.get_keycode(self.selected_layer, selected_ki)
                                                })
                                            })
                                            .flatten()
                                    })
                                });
                            hint_kc
                                .map(|kc| {
                                    let is_plain_mod = (0x00E0..=0x00E7).contains(&kc)
                                        || matches!(
                                            kc,
                                            0x52A1
                                                | 0x52A2
                                                | 0x52A4
                                                | 0x52A7
                                                | 0x52A8
                                                | 0x52AF
                                                | 0x52B1
                                                | 0x52B2
                                                | 0x52B4
                                                | 0x52B8
                                        );
                                    let is_mod = is_plain_mod
                                        || (kc >= 0x2000 && kc < 0x4000)
                                        || (kc >= 0x0100 && kc < 0x2000 && (kc & 0xFF) != 0);
                                    let can_swap_side = toggle_handed_modifier(kc).is_some();
                                    let is_macro = kc >= 0x7700 && kc <= 0x77FF;
                                    let is_tap_dance = kc >= 0x5700 && kc <= 0x57FF;
                                    let is_mouse = is_mouse_keycode(kc);
                                    let is_alt_repeat = is_alt_repeat_keycode(kc);
                                    let is_grave_escape = kc == 0x7C16;
                                    let is_layer = vial_layer_target(kc).is_some();
                                    let can_retarget_mod_key = !is_layer
                                        && ((kc >= 0x2000 && kc < 0x4000)
                                            || (kc >= 0x0100 && kc < 0x2000 && (kc & 0xFF) != 0));
                                    (
                                        is_mod,
                                        can_swap_side,
                                        can_retarget_mod_key,
                                        is_macro,
                                        is_tap_dance,
                                        is_mouse,
                                        is_alt_repeat,
                                        is_grave_escape,
                                        is_layer,
                                    )
                                })
                                .unwrap_or((
                                    false, false, false, false, false, false, false, false, false,
                                ))
                        };
                        if hovered_is_mod {
                            if hovered_can_swap_side {
                                let show_retarget = hovered_can_retarget_mod_key;
                                ui.painter().text(
                                    egui::pos2(
                                        center_x,
                                        if show_retarget {
                                            hint_y - 22.0
                                        } else {
                                            hint_y - 10.0
                                        },
                                    ),
                                    egui::Align2::CENTER_CENTER,
                                    tr_hint("key_hints.change_key"),
                                    hint_font.clone(),
                                    hint_color,
                                );
                                if show_retarget {
                                    ui.painter().text(
                                        egui::pos2(center_x, hint_y - 4.0),
                                        egui::Align2::CENTER_CENTER,
                                        tr_hint("key_hints.change_modifier_key"),
                                        secondary_hint_font.clone(),
                                        hint_color,
                                    );
                                }
                                ui.painter().text(
                                    egui::pos2(
                                        center_x,
                                        if show_retarget {
                                            hint_y + 12.0
                                        } else {
                                            hint_y + 8.0
                                        },
                                    ),
                                    egui::Align2::CENTER_CENTER,
                                    tr_hint("key_hints.switch_modifier_side"),
                                    secondary_hint_font,
                                    hint_color,
                                );
                            } else {
                                ui.painter().text(
                                    egui::pos2(center_x, hint_y - 14.0),
                                    egui::Align2::CENTER_CENTER,
                                    tr_hint("key_hints.change_key"),
                                    hint_font.clone(),
                                    hint_color,
                                );
                                ui.painter().text(
                                    egui::pos2(center_x, hint_y + 4.0),
                                    egui::Align2::CENTER_CENTER,
                                    tr_hint("key_hints.change_modifier_key"),
                                    secondary_hint_font,
                                    hint_color,
                                );
                            }
                        } else if hovered_is_macro {
                            ui.painter().text(
                                egui::pos2(center_x, hint_y - 14.0),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.change_key"),
                                hint_font.clone(),
                                hint_color,
                            );
                            ui.painter().text(
                                egui::pos2(center_x, hint_y + 4.0),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.edit_macro"),
                                secondary_hint_font.clone(),
                                hint_color,
                            );
                        } else if hovered_is_tap_dance {
                            ui.painter().text(
                                egui::pos2(center_x, hint_y - 14.0),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.change_key"),
                                hint_font.clone(),
                                hint_color,
                            );
                            ui.painter().text(
                                egui::pos2(center_x, hint_y + 4.0),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.edit_tap_dance"),
                                secondary_hint_font,
                                hint_color,
                            );
                        } else if hovered_is_mouse {
                            ui.painter().text(
                                egui::pos2(center_x, hint_y - 14.0),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.change_key"),
                                hint_font.clone(),
                                hint_color,
                            );
                            ui.painter().text(
                                egui::pos2(center_x, hint_y + 4.0),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.open_mouse_keys"),
                                secondary_hint_font.clone(),
                                hint_color,
                            );
                        } else if hovered_is_alt_repeat {
                            ui.painter().text(
                                egui::pos2(center_x, hint_y - 14.0),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.change_key"),
                                hint_font.clone(),
                                hint_color,
                            );
                            ui.painter().text(
                                egui::pos2(center_x, hint_y + 4.0),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.open_alt_repeat"),
                                secondary_hint_font,
                                hint_color,
                            );
                        } else if hovered_is_grave_escape {
                            ui.painter().text(
                                egui::pos2(center_x, hint_y - 14.0),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.change_key"),
                                hint_font.clone(),
                                hint_color,
                            );
                            ui.painter().text(
                                egui::pos2(center_x, hint_y + 4.0),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.open_grave_escape"),
                                secondary_hint_font,
                                hint_color,
                            );
                        } else if hovered_is_layer {
                            ui.painter().text(
                                egui::pos2(center_x, hint_y - 22.0),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.change_key"),
                                hint_font.clone(),
                                hint_color,
                            );
                            ui.painter().text(
                                egui::pos2(center_x, hint_y - 4.0),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.go_to_that_layer"),
                                secondary_hint_font.clone(),
                                hint_color,
                            );
                            ui.painter().text(
                                egui::pos2(center_x, hint_y + 12.0),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.change_layer_target"),
                                secondary_hint_font,
                                hint_color,
                            );
                        } else {
                            ui.painter().text(
                                egui::pos2(center_x, hint_y),
                                egui::Align2::CENTER_CENTER,
                                tr_hint("key_hints.change_key"),
                                hint_font,
                                hint_color,
                            );
                        }
                    } else if name_r.hovered() {
                        ui.painter().text(
                            egui::pos2(center_x, hint_y),
                            egui::Align2::CENTER_CENTER,
                            tr_hint("key_hints.rename_layer"),
                            hint_font,
                            hint_color,
                        );
                    }
                }
            }
        }

        // Pass 1: allocate
        let key_rects: Vec<(usize, egui::Rect)> = layout
            .keys
            .iter()
            .enumerate()
            .map(|(ki, key)| {
                let geometry = LayoutGeometry {
                    offset_x,
                    offset_y,
                    unit,
                    padding,
                    layout_h,
                };
                let rect = layout_physical_key_rect(key, geometry);
                (ki, rect)
            })
            .collect();
        let encoder_rects: Vec<(usize, egui::Rect)> = layout
            .encoders
            .iter()
            .enumerate()
            .map(|(ei, encoder)| {
                let geometry = LayoutGeometry {
                    offset_x,
                    offset_y,
                    unit,
                    padding,
                    layout_h,
                };
                let rect = layout_physical_encoder_rect(encoder, geometry);
                (ei, rect)
            })
            .collect();
        let keyboard_target_rect = key_rects
            .iter()
            .map(|(_, rect)| *rect)
            .chain(encoder_rects.iter().map(|(_, rect)| *rect))
            .reduce(|acc, rect| acc.union(rect));
        if let Some(rect) = keyboard_target_rect {
            self.register_tour_target(TourTarget::KeyboardArea, rect.expand(10.0));
        }
        self.register_tour_target(
            TourTarget::BottomHints,
            egui::Rect::from_center_size(
                egui::pos2(ui.max_rect().center().x, ui.max_rect().bottom() - 34.0),
                Vec2::new(ui.max_rect().width().min(560.0), 52.0),
            ),
        );
        let mut encoder_groups: Vec<(u8, egui::Rect, Option<(usize, u16)>, Option<(usize, u16)>)> =
            Vec::new();
        for (ei, rect) in &encoder_rects {
            let encoder = &layout.encoders[*ei];
            if !self
                .encoder_visibility
                .get(encoder.encoder_idx as usize)
                .copied()
                .unwrap_or(true)
            {
                continue;
            }
            let kc = layout.get_encoder_keycode(self.selected_layer, *ei);
            if let Some((_, group_rect, ccw, cw)) = encoder_groups
                .iter_mut()
                .find(|(idx, _, _, _)| *idx == encoder.encoder_idx)
            {
                *group_rect = group_rect.union(*rect);
                if encoder.direction == 0 {
                    *ccw = Some((*ei, kc));
                } else {
                    *cw = Some((*ei, kc));
                }
            } else {
                encoder_groups.push((
                    encoder.encoder_idx,
                    *rect,
                    if encoder.direction == 0 {
                        Some((*ei, kc))
                    } else {
                        None
                    },
                    if encoder.direction == 0 {
                        None
                    } else {
                        Some((*ei, kc))
                    },
                ));
            }
        }
        let mut encoder_press_rects: Vec<(usize, egui::Rect)> = Vec::new();
        for (_, group_rect, _, _) in &encoder_groups {
            let center = group_rect.center();
            let radius = group_rect.width().min(group_rect.height()) * 0.5;
            let mut best_key: Option<(usize, f32)> = None;
            for (ki, key_rect) in &key_rects {
                if encoder_press_rects
                    .iter()
                    .any(|(assigned_ki, _)| assigned_ki == ki)
                {
                    continue;
                }
                let dist = key_rect.center().distance(center);
                if dist > radius * 0.38 {
                    continue;
                }
                match best_key {
                    Some((_, best_dist)) if dist >= best_dist => {}
                    _ => best_key = Some((*ki, dist)),
                }
            }
            if let Some((ki, _)) = best_key {
                let press_rect = egui::Rect::from_center_size(
                    center,
                    Vec2::new(
                        (radius * 0.88).min(group_rect.width() * 0.44),
                        (radius * 0.48).min(group_rect.height() * 0.22),
                    ),
                );
                encoder_press_rects.push((ki, press_rect));
            }
        }
        let mut rects: Vec<(usize, egui::Rect, egui::Response)> =
            Vec::with_capacity(layout.keys.len());
        for (ki, rect) in &key_rects {
            let response_rect = encoder_press_rects
                .iter()
                .find(|(press_ki, _)| press_ki == ki)
                .map(|(_, press_rect)| *press_rect)
                .unwrap_or(*rect);
            let response = ui.allocate_rect(response_rect, Sense::click());
            rects.push((*ki, response_rect, response));
        }

        // Reset hover_layer each frame — will be set again if a layer key is hovered
        let prev_hover = self.hover_layer;
        self.hover_layer = None;

        // Pass 2: hover + clicks + tooltips
        let mut hovered_key: Option<usize> = None;
        for (ki, _, response) in &mut rects {
            if response.hovered() {
                hovered_key = Some(*ki);
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if response.clicked() {
                self.selected_key = Some((self.selected_layer, *ki));
                self.keycode_picker.open = true;
                self.keycode_picker.result = None;
                self.keycode_picker.search_query.clear();
                self.keycode_picker.layer_names = self.layer_names.clone();
                self.keycode_picker.vial_quantum_pending_mod = None;
                self.keycode_picker.vial_quantum_pending_mt = None;
                self.keycode_picker.vial_layer_pending = None;
                // Reset all editor states so picker opens normally
                self.keycode_picker.tap_dance_editor_open = None;
                self.keycode_picker.selected_tab = crate::keycode_picker::KeycodeTab::Basic;
            }

            // Right-click actions: layer jump/retarget, modifier side swap, editors/settings.
            if response.secondary_clicked() {
                let ctrl_held = ui.input(|i| i.modifiers.ctrl);
                let kc = layout.get_keycode(self.selected_layer, *ki);
                self.handle_secondary_target(ctrl_held, kc, Some(*ki), None);
                if self.secondary_click_handled {
                    continue;
                }
            }

            // Tooltip — for layer keys show mini layout preview
            let kc = layout.get_keycode(self.selected_layer, *ki);
            // MO/TG/TO/OSL/TT/DF range and LT; OSM also lives in 0x52xx
            // but is deliberately excluded by vial_layer_target().
            let preview_layer: Option<usize> = vial_layer_target(kc);

            if let Some(preview_layer_idx) = preview_layer {
                if response.hovered() {
                    hovered_key = Some(*ki); // keep hovered_key for layer keys too
                    if self.app_settings.layer_hover_preview {
                        self.hover_layer = Some(preview_layer_idx);
                    } else {
                        let tip = keycode_tooltip_with_macro_names(
                            kc,
                            &layout.custom_keycodes,
                            &self.layer_names,
                            &self.keycode_picker.macro_names,
                            &self.keycode_picker.tap_dance_names,
                        );
                        *response = response
                            .clone()
                            .on_hover_text(crate::i18n::tr_text(self.app_settings.language, &tip));
                    }
                }
                if response.secondary_clicked() && preview_layer_idx != self.selected_layer {
                    // Right-click: jump to that layer
                    self.jump_back_stack.push(self.selected_layer);
                    self.selected_layer = preview_layer_idx;
                    self.hover_layer = None;
                    self.secondary_click_handled = true;
                }
            } else if response.hovered() {
                let tip = keycode_tooltip_with_macro_names(
                    kc,
                    &layout.custom_keycodes,
                    &self.layer_names,
                    &self.keycode_picker.macro_names,
                    &self.keycode_picker.tap_dance_names,
                );
                *response = response
                    .clone()
                    .on_hover_text(crate::i18n::tr_text(self.app_settings.language, &tip));
            }
        }

        // Animate hover_layer_progress
        let target_progress = if self.hover_layer.is_some() {
            1.0f32
        } else {
            0.0f32
        };
        let speed = 4.0f32;
        let dt = ctx.input(|i| i.stable_dt).min(0.1);
        self.hover_layer_progress +=
            (target_progress - self.hover_layer_progress) * (speed * dt).min(1.0);
        if (self.hover_layer_progress - target_progress).abs() > 0.01 {
            ctx.request_repaint();
        }

        // Pass 3: paint
        let painter = ui.painter();
        let mut hovered_encoder = false;
        let mut hovered_encoder_keycode = None;
        let hover_target = self
            .hover_layer
            .unwrap_or(prev_hover.unwrap_or(self.selected_layer));
        let hover_alpha = self.hover_layer_progress;
        let dark = self.dark_mode;
        // Use hover layer for logic (TRNS resolution etc) when mostly visible
        let layer = if hover_alpha > 0.5 {
            hover_target
        } else {
            self.selected_layer
        };
        let layer_led_color_idx = if self.layer_led_settings.supported {
            self.layer_led_settings
                .layer_colors
                .get(layer.min(15))
                .copied()
                .filter(|color_idx| !matches!(color_idx, 0 | 1))
        } else {
            None
        };
        // Off and White should keep the standard neutral outline/fill so disabled/uncolored
        // layers do not look artificially tinted.
        let layer_led_outline = layer_led_color_idx.map(layer_led_outline_color);
        let layer_led_hover_fill =
            layer_led_color_idx.map(|color_idx| layer_led_hover_fill(color_idx, dark));
        for (ki, rect, _) in &rects {
            let key = &layout.keys[*ki];
            let is_selected = self.selected_key == Some((layer, *ki));
            let is_hovered = hovered_key == Some(*ki);
            // Accent: #5B68DF indigo
            let bg = if is_selected {
                app_accent()
            } else if is_hovered {
                layer_led_hover_fill.unwrap_or_else(|| crate::ui_style::hover_fill(dark))
            } else {
                if dark {
                    Color32::from_rgb(48, 48, 52)
                } else {
                    Color32::from_rgb(255, 255, 255)
                }
            };

            let press_rect_override = encoder_press_rects
                .iter()
                .find(|(press_ki, _)| *press_ki == *ki)
                .map(|(_, press_rect)| *press_rect);
            let draw_rect = press_rect_override.unwrap_or(*rect);

            let is_hovering = hover_alpha > 0.05;

            if press_rect_override.is_some() {
                continue;
            }

            let kc = layout.get_keycode(layer, *ki);

            if kc == 0x0001 {
                paint_layout_keycap(
                    painter,
                    draw_rect,
                    key.rotation,
                    bg,
                    Stroke::new(
                        1.0,
                        layer_led_outline.unwrap_or_else(|| {
                            if dark {
                                Color32::from_rgb(54, 54, 58)
                            } else {
                                Color32::from_rgb(230, 230, 233)
                            }
                        }),
                    ),
                );
                if !is_hovering {
                    let fallback_kc = (0..layer)
                        .rev()
                        .map(|l| layout.get_keycode(l, *ki))
                        .find(|&k| k != 0x0001)
                        .unwrap_or(0x0000);
                    let label = if fallback_kc == 0x0000 || fallback_kc == 0x0001 {
                        String::new()
                    } else {
                        keycode_label_with_macro_names(
                            fallback_kc,
                            &layout.custom_keycodes,
                            &self.layer_names,
                            &self.keycode_picker.macro_names,
                            &self.keycode_picker.tap_dance_names,
                            self.app_settings.key_legend_layout,
                        )
                    };
                    let label = number_row_shifted_label(
                        label,
                        self.app_settings.show_shifted_number_symbols,
                        self.app_settings.key_legend_layout,
                    );
                    draw_key_label_dimmed(
                        &painter,
                        draw_rect,
                        &label,
                        dark,
                        key.rotation.to_radians(),
                    );
                }
            } else if kc == 0x0000 {
                let no_bg = if dark {
                    Color32::from_rgb(20, 20, 22)
                } else {
                    Color32::from_rgb(255, 255, 255)
                };
                let no_border = layer_led_outline.unwrap_or_else(|| {
                    if dark {
                        Color32::from_rgb(40, 40, 44)
                    } else {
                        Color32::from_rgb(230, 230, 233)
                    }
                });
                let fill = if is_selected || is_hovered { bg } else { no_bg };
                paint_layout_keycap(
                    painter,
                    draw_rect,
                    key.rotation,
                    fill,
                    Stroke::new(1.0, no_border),
                );
            } else {
                let border = layer_led_outline.unwrap_or_else(|| {
                    if dark {
                        Color32::from_rgb(54, 54, 58)
                    } else {
                        Color32::from_rgb(230, 230, 233)
                    }
                });
                paint_layout_keycap(
                    painter,
                    draw_rect,
                    key.rotation,
                    bg,
                    Stroke::new(1.0, border),
                );
                let label = number_row_shifted_label(
                    keycode_label_with_macro_names(
                        kc,
                        &layout.custom_keycodes,
                        &self.layer_names,
                        &self.keycode_picker.macro_names,
                        &self.keycode_picker.tap_dance_names,
                        self.app_settings.key_legend_layout,
                    ),
                    self.app_settings.show_shifted_number_symbols,
                    self.app_settings.key_legend_layout,
                );
                draw_key_label(&painter, draw_rect, &label, dark, key.rotation.to_radians());
            }
        }

        let encoder_custom_keycodes = layout.custom_keycodes.clone();
        let encoder_layer_names = self.layer_names.clone();
        let encoder_macro_names = self.keycode_picker.macro_names.clone();
        let encoder_tap_dance_names = self.keycode_picker.tap_dance_names.clone();
        let encoder_key_legend_layout = self.app_settings.key_legend_layout;
        let encoder_label = |kc: u16| -> String {
            match kc {
                0x0000 => "✕".to_string(),
                0x0001 => "▽".to_string(),
                _ => keycode_label_with_macro_names(
                    kc,
                    &encoder_custom_keycodes,
                    &encoder_layer_names,
                    &encoder_macro_names,
                    &encoder_tap_dance_names,
                    encoder_key_legend_layout,
                )
                .replace('\n', " "),
            }
        };

        let draw_encoder_arrow = |painter: &egui::Painter,
                                  center: egui::Pos2,
                                  encoder_radius: f32,
                                  top: bool,
                                  color: Color32| {
            let (start_deg, end_deg) = if top {
                (240.0_f32, 300.0_f32)
            } else {
                (120.0_f32, 60.0_f32)
            };
            let r = encoder_radius * 1.22;
            let mut points = Vec::new();
            for step in 0..=12 {
                let t = step as f32 / 12.0;
                let deg = start_deg + (end_deg - start_deg) * t;
                let rad = deg.to_radians();
                points.push(egui::pos2(
                    center.x + rad.cos() * r,
                    center.y + rad.sin() * r,
                ));
            }
            painter.add(egui::Shape::line(points.clone(), Stroke::new(1.7, color)));
            if points.len() >= 2 {
                let end = points[points.len() - 1];
                let prev = points[points.len() - 2];
                let dir = egui::vec2(end.x - prev.x, end.y - prev.y).normalized();
                let left = egui::vec2(-dir.y, dir.x);
                painter.add(egui::Shape::convex_polygon(
                    vec![
                        end,
                        egui::pos2(
                            end.x - dir.x * 3.6 + left.x * 2.4,
                            end.y - dir.y * 3.6 + left.y * 2.4,
                        ),
                        egui::pos2(
                            end.x - dir.x * 3.6 - left.x * 2.4,
                            end.y - dir.y * 3.6 - left.y * 2.4,
                        ),
                    ],
                    color,
                    Stroke::NONE,
                ));
            }
        };

        const ENCODER_HOVER_SCALE: f32 = 1.5;
        let encoder_hover_enlarge = self.app_settings.encoder_hover_enlarge;
        for (_encoder_idx, rect, ccw, cw) in &encoder_groups {
            let center = rect.center();
            let base_radius = rect.width().min(rect.height()) * LAYOUT_ENCODER_RADIUS_FACTOR;
            let hover_radius = base_radius * ENCODER_HOVER_SCALE;
            let interactive_radius = if encoder_hover_enlarge {
                hover_radius
            } else {
                base_radius
            };
            let circle_bounds = egui::Rect::from_center_size(
                center,
                egui::vec2(interactive_radius * 2.0, interactive_radius * 2.0),
            );
            let press_slot = encoder_press_rects
                .iter()
                .find(|(_, press_rect)| press_rect.center().distance(center) < 1.0)
                .map(|(press_ki, press_rect)| (*press_ki, *press_rect));
            let (top_rect, middle_rect, bottom_rect) = if let Some((_, press_rect)) = press_slot {
                let divider_gap = base_radius * 0.06;
                let top_divider_y = press_rect.top() - divider_gap;
                let bottom_divider_y = press_rect.bottom() + divider_gap;
                (
                    egui::Rect::from_min_max(
                        circle_bounds.min,
                        egui::pos2(circle_bounds.max.x, top_divider_y),
                    ),
                    Some(egui::Rect::from_min_max(
                        egui::pos2(circle_bounds.min.x, top_divider_y),
                        egui::pos2(circle_bounds.max.x, bottom_divider_y),
                    )),
                    egui::Rect::from_min_max(
                        egui::pos2(circle_bounds.min.x, bottom_divider_y),
                        circle_bounds.max,
                    ),
                )
            } else {
                (
                    egui::Rect::from_min_max(
                        circle_bounds.min,
                        egui::pos2(circle_bounds.max.x, center.y),
                    ),
                    None,
                    egui::Rect::from_min_max(
                        egui::pos2(circle_bounds.min.x, center.y),
                        circle_bounds.max,
                    ),
                )
            };
            let top_resp = ui.allocate_rect(top_rect, Sense::click());
            let middle_resp =
                middle_rect.map(|middle_rect| ui.allocate_rect(middle_rect, Sense::click()));
            let bottom_resp = ui.allocate_rect(bottom_rect, Sense::click());
            let encoder_hovered = top_resp.hovered()
                || middle_resp.as_ref().map(|r| r.hovered()).unwrap_or(false)
                || bottom_resp.hovered();
            let radius = if encoder_hovered && encoder_hover_enlarge {
                hover_radius
            } else {
                base_radius
            };
            let font_scale = if encoder_hovered && encoder_hover_enlarge {
                ENCODER_HOVER_SCALE
            } else {
                1.0
            };
            if encoder_hovered {
                hovered_encoder = true;
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            let ctrl_held = ui.input(|i| i.modifiers.ctrl);
            if top_resp.hovered() {
                if let Some((_, kc)) = cw {
                    hovered_encoder_keycode = Some(*kc);
                    let tip = keycode_tooltip_with_macro_names(
                        *kc,
                        &layout.custom_keycodes,
                        &self.layer_names,
                        &self.keycode_picker.macro_names,
                        &self.keycode_picker.tap_dance_names,
                    );
                    let _ = top_resp
                        .clone()
                        .on_hover_text(crate::i18n::tr_text(self.app_settings.language, &tip));
                }
            }
            if top_resp.secondary_clicked() {
                if let Some((visual_idx, kc)) = cw {
                    self.handle_secondary_target(ctrl_held, *kc, None, Some(*visual_idx));
                }
            }
            if top_resp.clicked() {
                if let Some((visual_idx, _)) = cw {
                    self.selected_key = None;
                    self.selected_encoder = Some((layer, *visual_idx));
                    self.keycode_picker.open = true;
                    self.keycode_picker.result = None;
                }
            }
            if let (Some((press_ki, _)), Some(middle_resp)) = (press_slot, middle_resp.as_ref()) {
                if middle_resp.hovered() {
                    hovered_key = Some(press_ki);
                    let kc = layout.get_keycode(self.selected_layer, press_ki);
                    hovered_encoder_keycode = Some(kc);
                    let tip = keycode_tooltip_with_macro_names(
                        kc,
                        &layout.custom_keycodes,
                        &self.layer_names,
                        &self.keycode_picker.macro_names,
                        &self.keycode_picker.tap_dance_names,
                    );
                    let _ = middle_resp
                        .clone()
                        .on_hover_text(crate::i18n::tr_text(self.app_settings.language, &tip));
                }
                if middle_resp.secondary_clicked() {
                    let kc = layout.get_keycode(self.selected_layer, press_ki);
                    self.handle_secondary_target(ctrl_held, kc, Some(press_ki), None);
                }
                if middle_resp.clicked() {
                    self.open_picker_for_target(Some(press_ki), None);
                    self.selected_encoder = None;
                }
            }
            if bottom_resp.hovered() {
                if let Some((_, kc)) = ccw {
                    hovered_encoder_keycode = Some(*kc);
                    let tip = keycode_tooltip_with_macro_names(
                        *kc,
                        &layout.custom_keycodes,
                        &self.layer_names,
                        &self.keycode_picker.macro_names,
                        &self.keycode_picker.tap_dance_names,
                    );
                    let _ = bottom_resp
                        .clone()
                        .on_hover_text(crate::i18n::tr_text(self.app_settings.language, &tip));
                }
            }
            if bottom_resp.secondary_clicked() {
                if let Some((visual_idx, kc)) = ccw {
                    self.handle_secondary_target(ctrl_held, *kc, None, Some(*visual_idx));
                }
            }
            if bottom_resp.clicked() {
                if let Some((visual_idx, _)) = ccw {
                    self.selected_key = None;
                    self.selected_encoder = Some((self.selected_layer, *visual_idx));
                    self.keycode_picker.open = true;
                    self.keycode_picker.result = None;
                }
            }

            let top_selected = cw
                .map(|(visual_idx, _)| Some((layer, visual_idx)) == self.selected_encoder)
                .unwrap_or(false);
            let bottom_selected = ccw
                .map(|(visual_idx, _)| Some((layer, visual_idx)) == self.selected_encoder)
                .unwrap_or(false);
            let middle_selected = press_slot
                .map(|(press_ki, _)| self.selected_key == Some((layer, press_ki)))
                .unwrap_or(false);
            let middle_hovered = middle_resp.as_ref().map(|r| r.hovered()).unwrap_or(false);
            let visuals = &ui.visuals().widgets;
            let fill_radius = radius + LAYOUT_ENCODER_FILL_EXTRA;
            let top_fill = if top_selected {
                visuals.active.bg_fill
            } else if top_resp.hovered() {
                visuals.hovered.bg_fill
            } else {
                visuals.inactive.bg_fill
            };
            let bottom_fill = if bottom_selected {
                visuals.active.bg_fill
            } else if bottom_resp.hovered() {
                visuals.hovered.bg_fill
            } else {
                visuals.inactive.bg_fill
            };
            let middle_fill = if middle_selected {
                visuals.active.bg_fill
            } else if middle_hovered {
                visuals.hovered.bg_fill
            } else {
                visuals.inactive.bg_fill
            };
            let outline = if top_selected || bottom_selected || middle_selected {
                visuals.active.bg_stroke
            } else if top_resp.hovered() || middle_hovered || bottom_resp.hovered() {
                visuals.hovered.bg_stroke
            } else {
                visuals.inactive.bg_stroke
            };

            let painter = ui.painter();
            painter.circle_filled(center, fill_radius, visuals.inactive.bg_fill);
            painter
                .with_clip_rect(top_rect)
                .circle_filled(center, fill_radius, top_fill);
            if let Some(middle_rect) = middle_rect {
                painter
                    .with_clip_rect(middle_rect)
                    .circle_filled(center, fill_radius, middle_fill);
            }
            painter
                .with_clip_rect(bottom_rect)
                .circle_filled(center, fill_radius, bottom_fill);
            painter.circle_stroke(center, radius, outline);

            let has_press_button = encoder_press_rects
                .iter()
                .any(|(_, press_rect)| press_rect.center().distance(center) < 1.0);
            let top_label = encoder_label(cw.map(|(_, kc)| kc).unwrap_or(0x0000));
            let bottom_label = encoder_label(ccw.map(|(_, kc)| kc).unwrap_or(0x0000));
            let top_font = if has_press_button {
                egui::FontId::proportional(
                    if top_label.chars().count() > 9 {
                        6.6
                    } else {
                        7.4
                    } * font_scale,
                )
            } else {
                egui::FontId::proportional(
                    if top_label.chars().count() > 9 {
                        8.5
                    } else {
                        9.5
                    } * font_scale,
                )
            };
            let bottom_font = if has_press_button {
                egui::FontId::proportional(
                    if bottom_label.chars().count() > 9 {
                        6.6
                    } else {
                        7.4
                    } * font_scale,
                )
            } else {
                egui::FontId::proportional(
                    if bottom_label.chars().count() > 9 {
                        8.5
                    } else {
                        9.5
                    } * font_scale,
                )
            };
            let top_label_y = center.y - radius * if has_press_button { 0.52 } else { 0.30 };
            let bottom_label_y = center.y + radius * if has_press_button { 0.52 } else { 0.30 };
            let top_text_color = if top_selected {
                visuals.active.fg_stroke.color
            } else if top_resp.hovered() {
                visuals.hovered.fg_stroke.color
            } else {
                visuals.inactive.fg_stroke.color
            };
            let bottom_text_color = if bottom_selected {
                visuals.active.fg_stroke.color
            } else if bottom_resp.hovered() {
                visuals.hovered.fg_stroke.color
            } else {
                visuals.inactive.fg_stroke.color
            };
            painter.text(
                egui::pos2(center.x, top_label_y),
                egui::Align2::CENTER_CENTER,
                top_label,
                top_font,
                top_text_color,
            );
            painter.text(
                egui::pos2(center.x, bottom_label_y),
                egui::Align2::CENTER_CENTER,
                bottom_label,
                bottom_font,
                bottom_text_color,
            );

            let arrow_color_top = outline.color;
            let arrow_color_bottom = outline.color;
            draw_encoder_arrow(painter, center, radius, true, arrow_color_top);
            draw_encoder_arrow(painter, center, radius, false, arrow_color_bottom);

            if let Some((press_ki, _)) = press_slot {
                let middle_rect = middle_rect.unwrap();
                let top_divider_y = middle_rect.top();
                let bottom_divider_y = middle_rect.bottom();
                let divider_radius = (radius - 0.5).max(0.0);
                let top_divider_half_width = (divider_radius * divider_radius
                    - (top_divider_y - center.y) * (top_divider_y - center.y))
                    .max(0.0)
                    .sqrt();
                let bottom_divider_half_width = (divider_radius * divider_radius
                    - (bottom_divider_y - center.y) * (bottom_divider_y - center.y))
                    .max(0.0)
                    .sqrt();
                painter.line_segment(
                    [
                        egui::pos2(center.x - top_divider_half_width, top_divider_y),
                        egui::pos2(center.x + top_divider_half_width, top_divider_y),
                    ],
                    Stroke::new(1.0, outline.color),
                );
                painter.line_segment(
                    [
                        egui::pos2(center.x - bottom_divider_half_width, bottom_divider_y),
                        egui::pos2(center.x + bottom_divider_half_width, bottom_divider_y),
                    ],
                    Stroke::new(1.0, outline.color),
                );
                let is_hovering = hover_alpha > 0.05;
                let text_color = if middle_selected {
                    visuals.active.fg_stroke.color
                } else if middle_hovered {
                    visuals.hovered.fg_stroke.color
                } else {
                    visuals.inactive.fg_stroke.color
                };
                let press_text_rect = middle_rect.shrink2(egui::vec2(4.0, 2.0));

                let press_label = {
                    let kc = layout.get_keycode(layer, press_ki);
                    if kc == 0x0001 && !is_hovering {
                        let fallback_kc = (0..layer)
                            .rev()
                            .map(|l| layout.get_keycode(l, press_ki))
                            .find(|&k| k != 0x0001)
                            .unwrap_or(0x0000);
                        if fallback_kc == 0x0000 || fallback_kc == 0x0001 {
                            "▽".to_string()
                        } else {
                            keycode_label_with_macro_names(
                                fallback_kc,
                                &layout.custom_keycodes,
                                &self.layer_names,
                                &self.keycode_picker.macro_names,
                                &self.keycode_picker.tap_dance_names,
                                self.app_settings.key_legend_layout,
                            )
                        }
                    } else if kc == 0x0001 {
                        "▽".to_string()
                    } else if kc == 0x0000 {
                        "✕".to_string()
                    } else {
                        keycode_label_with_macro_names(
                            kc,
                            &layout.custom_keycodes,
                            &self.layer_names,
                            &self.keycode_picker.macro_names,
                            &self.keycode_picker.tap_dance_names,
                            self.app_settings.key_legend_layout,
                        )
                    }
                }
                .replace('\n', " ");
                let press_font = FontId::proportional(
                    if press_label.chars().count() > 8 {
                        7.2
                    } else {
                        8.2
                    } * font_scale,
                );
                painter.with_clip_rect(press_text_rect).text(
                    press_text_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    press_label,
                    press_font,
                    text_color,
                );
            } else {
                let divider_half_width = (radius - 0.5).max(0.0);
                painter.line_segment(
                    [
                        egui::pos2(center.x - divider_half_width, center.y),
                        egui::pos2(center.x + divider_half_width, center.y),
                    ],
                    Stroke::new(1.0, outline.color),
                );
            }
        }

        self.prev_hovered_key = hovered_key;
        self.prev_hovered_encoder = hovered_encoder;
        self.prev_hovered_encoder_keycode = hovered_encoder_keycode;

        if layout_h > avail.y {
            ui.allocate_space(Vec2::new(0.0, (layout_h - avail.y).max(0.0)));
        }
    }
}

fn layout_matrix_key_pressed(
    layout: &KeyboardLayout,
    matrix_pressed: &[bool],
    row: u8,
    col: u8,
) -> bool {
    matrix_pressed
        .get(row as usize * layout.cols + col as usize)
        .copied()
        .unwrap_or(false)
}

fn layout_effective_keycode(layout: &KeyboardLayout, layer: usize, key_idx: usize) -> u16 {
    let kc = layout.get_keycode(layer, key_idx);
    if kc != 0x0001 {
        return kc;
    }

    (0..layer)
        .rev()
        .map(|fallback_layer| layout.get_keycode(fallback_layer, key_idx))
        .find(|fallback| *fallback != 0x0001)
        .unwrap_or(0x0000)
}

fn sticky_layout_active_layer(
    layout: &KeyboardLayout,
    matrix_pressed: &[bool],
    toggled_layers: &[bool],
    base_layer: usize,
) -> usize {
    let layer_count = layout.layers.len().max(1);
    let mut active_layer = toggled_layers
        .iter()
        .enumerate()
        .rev()
        .find_map(|(layer, enabled)| (*enabled && layer < layer_count).then_some(layer))
        .unwrap_or_else(|| base_layer.min(layer_count - 1));

    for _ in 0..layer_count {
        let next_layer = layout.keys.iter().enumerate().find_map(|(key_idx, key)| {
            if !layout_matrix_key_pressed(layout, matrix_pressed, key.row, key.col) {
                return None;
            }
            sticky_momentary_layer_target(layout_effective_keycode(layout, active_layer, key_idx))
                .filter(|target| *target < layer_count)
        });

        match next_layer {
            Some(next_layer) if next_layer != active_layer => active_layer = next_layer,
            _ => break,
        }
    }

    active_layer
}

fn draw_sticky_encoder_arrow(
    painter: &egui::Painter,
    center: egui::Pos2,
    encoder_radius: f32,
    top: bool,
    color: Color32,
) {
    let (start_deg, end_deg) = if top {
        (240.0_f32, 300.0_f32)
    } else {
        (120.0_f32, 60.0_f32)
    };
    let r = encoder_radius * 1.22;
    let mut points = Vec::new();
    for step in 0..=12 {
        let t = step as f32 / 12.0;
        let deg = start_deg + (end_deg - start_deg) * t;
        let rad = deg.to_radians();
        points.push(egui::pos2(
            center.x + rad.cos() * r,
            center.y + rad.sin() * r,
        ));
    }
    painter.add(egui::Shape::line(points.clone(), Stroke::new(1.7, color)));
    if points.len() >= 2 {
        let end = points[points.len() - 1];
        let prev = points[points.len() - 2];
        let dir = egui::vec2(end.x - prev.x, end.y - prev.y).normalized();
        let left = egui::vec2(-dir.y, dir.x);
        painter.add(egui::Shape::convex_polygon(
            vec![
                end,
                egui::pos2(
                    end.x - dir.x * 3.6 + left.x * 2.4,
                    end.y - dir.y * 3.6 + left.y * 2.4,
                ),
                egui::pos2(
                    end.x - dir.x * 3.6 - left.x * 2.4,
                    end.y - dir.y * 3.6 - left.y * 2.4,
                ),
            ],
            color,
            Stroke::NONE,
        ));
    }
}

fn sticky_compact_label(label: &str, max_chars: usize) -> String {
    let label = label.trim();
    let count = label.chars().count();
    if count <= max_chars {
        return label.to_string();
    }
    let keep = max_chars.saturating_sub(1).max(1);
    format!("{}…", label.chars().take(keep).collect::<String>())
}

fn sticky_key_label_sizes(label: &str, rect: egui::Rect) -> (Option<f32>, f32) {
    let (top, bottom) = key_label_font_sizes(label);
    let longest = label
        .split(['\n', '/'])
        .map(|part| part.trim().chars().count())
        .max()
        .unwrap_or(1)
        .max(1) as f32;
    let available = (rect.width() - 8.0).max(8.0);
    let base = bottom.max(top.unwrap_or(bottom));
    let fit_scale = (available / (longest * base * 0.58)).clamp(0.58, 1.0);
    (top.map(|size| size * fit_scale), bottom * fit_scale)
}

fn draw_sticky_key_label(
    painter: &egui::Painter,
    rect: egui::Rect,
    label: &str,
    dark: bool,
    rotation: f32,
    dimmed: bool,
) {
    let clip_rect = rect.shrink2(egui::vec2(4.0, 3.0));
    let (top, bottom) = if label.contains('\n') {
        let mut parts = label.splitn(2, '\n');
        let t = parts.next().unwrap_or("");
        let b = parts.next().unwrap_or(label);
        (Some(t), b)
    } else if let Some(pos) = label.find('/') {
        (Some(&label[..pos]), &label[pos + 1..])
    } else {
        (None, label)
    };
    let (top_size, bottom_size) = sticky_key_label_sizes(label, rect);
    let (top_color, bottom_color) = if dimmed {
        if dark {
            (Color32::from_rgb(45, 45, 50), Color32::from_rgb(62, 56, 56))
        } else {
            (
                Color32::from_rgb(215, 215, 220),
                Color32::from_rgb(200, 200, 208),
            )
        }
    } else if dark {
        (
            Color32::from_rgb(130, 130, 145),
            Color32::from_rgb(239, 233, 232),
        )
    } else {
        (
            Color32::from_rgb(130, 130, 150),
            Color32::from_rgb(26, 26, 30),
        )
    };

    let clipped = painter.with_clip_rect(clip_rect);
    if let Some(top_str) = top {
        let center = rect.center();
        paint_centered_text_rotated(
            &clipped,
            center + rotated_offset(0.0, -7.0, rotation),
            top_str,
            FontId::proportional(top_size.unwrap_or(9.0)),
            top_color,
            rotation,
        );
        paint_centered_text_rotated(
            &clipped,
            center + rotated_offset(0.0, 6.0, rotation),
            bottom,
            FontId::proportional(bottom_size),
            bottom_color,
            rotation,
        );
    } else {
        let font_size = if bottom == "↵" {
            16.0_f32.min(bottom_size + 4.0)
        } else {
            bottom_size
        };
        paint_centered_text_rotated(
            &clipped,
            rect.center(),
            bottom,
            FontId::proportional(font_size),
            bottom_color,
            rotation,
        );
    }
}

fn rotated_offset(dx: f32, dy: f32, angle: f32) -> egui::Vec2 {
    egui::vec2(
        dx * angle.cos() - dy * angle.sin(),
        dx * angle.sin() + dy * angle.cos(),
    )
}

fn paint_centered_text_rotated(
    painter: &egui::Painter,
    center: egui::Pos2,
    text: &str,
    font_id: FontId,
    color: Color32,
    rotation: f32,
) {
    if rotation == 0.0 {
        painter.text(center, egui::Align2::CENTER_CENTER, text, font_id, color);
        return;
    }

    let galley = painter.layout_no_wrap(text.to_string(), font_id, color);
    let half = galley.size() * 0.5;
    let pos = center - rotated_offset(half.x, half.y, rotation);
    painter.add(egui::Shape::Text(
        egui::epaint::TextShape::new(pos, galley, color).with_angle(rotation),
    ));
}

fn draw_key_label_dimmed(
    painter: &egui::Painter,
    rect: egui::Rect,
    label: &str,
    dark: bool,
    rotation: f32,
) {
    let dim = if dark {
        Color32::from_rgb(62, 56, 56)
    } else {
        Color32::from_rgb(200, 200, 208)
    };
    let dim_top = if dark {
        Color32::from_rgb(45, 45, 50)
    } else {
        Color32::from_rgb(215, 215, 220)
    };
    let (top, bottom) = if label.contains('\n') {
        let mut parts = label.splitn(2, '\n');
        let t = parts.next().unwrap_or("");
        let b = parts.next().unwrap_or(label);
        (Some(t), b)
    } else if let Some(pos) = label.find('/') {
        (Some(&label[..pos]), &label[pos + 1..])
    } else {
        (None, label)
    };
    let (top_size, bottom_size) = key_label_font_sizes(label);

    if let Some(top_str) = top {
        let center = rect.center();
        paint_centered_text_rotated(
            painter,
            center + rotated_offset(0.0, -7.0, rotation),
            top_str,
            FontId::proportional(top_size.unwrap_or(9.0)),
            dim_top,
            rotation,
        );
        paint_centered_text_rotated(
            painter,
            center + rotated_offset(0.0, 6.0, rotation),
            bottom,
            FontId::proportional(bottom_size),
            dim,
            rotation,
        );
    } else {
        let font_size = if bottom == "↵" { 16.0 } else { bottom_size };
        paint_centered_text_rotated(
            painter,
            rect.center(),
            bottom,
            FontId::proportional(font_size),
            dim,
            rotation,
        );
    }
}

fn number_row_shifted_label(
    label: String,
    enabled: bool,
    key_legend_layout: KeyLegendLayout,
) -> String {
    if !enabled {
        return label;
    }

    let Some((digit, english, russian)) = (match label.as_str() {
        "1" => Some(("1", "!", "!")),
        "2" => Some(("2", "@", "\"")),
        "3" => Some(("3", "#", "№")),
        "4" => Some(("4", "$", ";")),
        "5" => Some(("5", "%", "%")),
        "6" => Some(("6", "^", ":")),
        "7" => Some(("7", "&", "?")),
        "8" => Some(("8", "*", "*")),
        "9" => Some(("9", "(", "(")),
        "0" => Some(("0", ")", ")")),
        _ => None,
    }) else {
        return label;
    };

    let shifted = match key_legend_layout {
        KeyLegendLayout::English => english.to_string(),
        KeyLegendLayout::Russian => {
            if english == russian {
                english.to_string()
            } else {
                format!("{}  {}", english, russian)
            }
        }
        KeyLegendLayout::RussianPrimary => {
            if english == russian {
                russian.to_string()
            } else {
                format!("{}  {}", russian, english)
            }
        }
    };
    format!("{}\n{}", shifted, digit)
}

fn with_alpha(color: Color32, alpha: f32) -> Color32 {
    let a = (color.a() as f32 * alpha.clamp(0.0, 1.0)) as u8;
    Color32::from_rgba_premultiplied(
        (color.r() as f32 * alpha.clamp(0.0, 1.0)) as u8,
        (color.g() as f32 * alpha.clamp(0.0, 1.0)) as u8,
        (color.b() as f32 * alpha.clamp(0.0, 1.0)) as u8,
        a,
    )
}

fn draw_key_label_alpha(
    painter: &egui::Painter,
    rect: egui::Rect,
    label: &str,
    dark: bool,
    alpha: f32,
) {
    let (top, bottom) = if label.contains('\n') {
        let mut parts = label.splitn(2, '\n');
        let t = parts.next().unwrap_or("");
        let b = parts.next().unwrap_or(label);
        (Some(t), b)
    } else if let Some(pos) = label.find('/') {
        (Some(&label[..pos]), &label[pos + 1..])
    } else {
        (None, label)
    };
    let (top_size, bottom_size) = key_label_font_sizes(label);
    let top_color = with_alpha(
        if dark {
            Color32::from_rgb(130, 130, 145)
        } else {
            Color32::from_rgb(130, 130, 150)
        },
        alpha,
    );
    let main_color = with_alpha(
        if dark {
            Color32::from_rgb(239, 233, 232)
        } else {
            Color32::from_rgb(26, 26, 30)
        },
        alpha,
    );
    if let Some(top_str) = top {
        let center = rect.center();
        painter.text(
            egui::pos2(center.x, center.y - 7.0),
            egui::Align2::CENTER_CENTER,
            top_str,
            FontId::proportional(top_size.unwrap_or(9.0)),
            top_color,
        );
        painter.text(
            egui::pos2(center.x, center.y + 6.0),
            egui::Align2::CENTER_CENTER,
            bottom,
            FontId::proportional(bottom_size),
            main_color,
        );
    } else {
        let font_size = if bottom == "↵" { 16.0 } else { bottom_size };
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            bottom,
            FontId::proportional(font_size),
            main_color,
        );
    }
}

fn draw_key_label_dimmed_alpha(
    painter: &egui::Painter,
    rect: egui::Rect,
    label: &str,
    dark: bool,
    alpha: f32,
) {
    let dim = with_alpha(
        if dark {
            Color32::from_rgb(80, 80, 90)
        } else {
            Color32::from_rgb(180, 180, 195)
        },
        alpha,
    );
    let dim_top = with_alpha(
        if dark {
            Color32::from_rgb(60, 60, 70)
        } else {
            Color32::from_rgb(190, 190, 205)
        },
        alpha,
    );
    let (top, bottom) = if label.contains('\n') {
        let mut parts = label.splitn(2, '\n');
        let t = parts.next().unwrap_or("");
        let b = parts.next().unwrap_or(label);
        (Some(t), b)
    } else if let Some(pos) = label.find('/') {
        (Some(&label[..pos]), &label[pos + 1..])
    } else {
        (None, label)
    };
    let (top_size, bottom_size) = key_label_font_sizes(label);
    if let Some(top_str) = top {
        let center = rect.center();
        painter.text(
            egui::pos2(center.x, center.y - 7.0),
            egui::Align2::CENTER_CENTER,
            top_str,
            FontId::proportional(top_size.unwrap_or(9.0)),
            dim_top,
        );
        painter.text(
            egui::pos2(center.x, center.y + 6.0),
            egui::Align2::CENTER_CENTER,
            bottom,
            FontId::proportional(bottom_size),
            dim,
        );
    } else {
        let font_size = if bottom == "↵" { 16.0 } else { bottom_size };
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            bottom,
            FontId::proportional(font_size),
            dim,
        );
    }
}

fn draw_key_label(
    painter: &egui::Painter,
    rect: egui::Rect,
    label: &str,
    dark: bool,
    rotation: f32,
) {
    // Split on "\n" first, then on "/" — show top part small+dim, bottom part normal
    let (top, bottom) = if label.contains('\n') {
        let mut parts = label.splitn(2, '\n');
        let t = parts.next().unwrap_or("");
        let b = parts.next().unwrap_or(label);
        (Some(t), b)
    } else if let Some(pos) = label.find('/') {
        let t = &label[..pos];
        let b = &label[pos + 1..];
        (Some(t), b)
    } else {
        (None, label)
    };
    let (top_size, bottom_size) = key_label_font_sizes(label);

    if let Some(top_str) = top {
        // Two-line layout
        let top_color = if dark {
            Color32::from_rgb(130, 130, 145)
        } else {
            Color32::from_rgb(130, 130, 150)
        };
        let main_color = if dark {
            Color32::from_rgb(239, 233, 232)
        } else {
            Color32::from_rgb(26, 26, 30)
        };
        paint_centered_text_rotated(
            painter,
            rect.center() + rotated_offset(0.0, -7.0, rotation),
            top_str,
            FontId::proportional(top_size.unwrap_or(9.0)),
            top_color,
            rotation,
        );
        paint_centered_text_rotated(
            painter,
            rect.center() + rotated_offset(0.0, 6.0, rotation),
            bottom,
            FontId::proportional(bottom_size),
            main_color,
            rotation,
        );
    } else {
        let font_size = if bottom == "↵" { 16.0 } else { bottom_size };
        paint_centered_text_rotated(
            painter,
            rect.center(),
            bottom,
            FontId::proportional(font_size),
            if dark {
                Color32::from_rgb(239, 233, 232)
            } else {
                Color32::from_rgb(26, 26, 30)
            },
            rotation,
        );
    }
}

impl EntropyApp {
    fn draw_placeholder(&mut self, ui: &mut egui::Ui) {
        let key_w = 52.0_f32;
        let key_h = 52.0_f32;
        let gap = 6.0_f32;
        let total_w = 6.0 * (key_w + gap);
        let start_x = (ui.available_width() - total_w * 2.0 - 40.0) / 2.0;
        let start_y = ui.min_rect().top() + 40.0;

        let mut keys: Vec<(usize, egui::Rect, egui::Response)> = vec![];
        for half in 0..2_usize {
            let half_offset = if half == 0 { 0.0 } else { total_w + 40.0 };
            for row in 0..4_usize {
                for col in 0..6_usize {
                    let key_idx = half * 24 + row * 6 + col;
                    let rect = egui::Rect::from_min_size(
                        egui::pos2(
                            start_x + half_offset + col as f32 * (key_w + gap),
                            start_y + row as f32 * (key_h + gap),
                        ),
                        Vec2::new(key_w, key_h),
                    );
                    let response = ui.allocate_rect(rect, Sense::click());
                    keys.push((key_idx, rect, response));
                }
            }
        }

        for (key_idx, _, response) in &keys {
            if response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if response.clicked() {
                self.selected_key = Some((self.selected_layer, *key_idx));
            }
        }

        let painter = ui.painter();
        for (key_idx, rect, _) in &keys {
            let is_selected = self.selected_key == Some((self.selected_layer, *key_idx));
            let bg = if is_selected {
                Color32::from_rgb(70, 110, 190)
            } else {
                Color32::from_gray(45)
            };
            painter.rect(
                *rect,
                6.0,
                bg,
                Stroke::new(1.0, Color32::from_gray(80)),
                egui::StrokeKind::Inside,
            );
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("K{key_idx}"),
                FontId::proportional(11.0),
                Color32::WHITE,
            );
        }
    }
}
