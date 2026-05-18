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

#[path = "app_storage.rs"]
mod app_storage;
use app_storage::*;

#[path = "ui/alt_repeat_settings.rs"]
mod alt_repeat_settings_ui;
#[path = "ui/app_lifecycle.rs"]
mod app_lifecycle;
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
#[path = "ui/settings_shell.rs"]
mod settings_shell;
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
}
