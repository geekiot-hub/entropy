use super::*;
use crate::text_expander::{TextExpansionConfig, TextExpansionEngine, TextExpansionRule};
use std::sync::{Mutex, OnceLock, RwLock};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextExpanderAppCandidate {
    pub exe: String,
    pub title: String,
}

static TEXT_EXPANDER_CONFIG: OnceLock<RwLock<TextExpansionConfig>> = OnceLock::new();
static TEXT_EXPANDER_ENGINE: OnceLock<Mutex<TextExpansionEngine>> = OnceLock::new();
static RECENT_FOREGROUND_APPS: OnceLock<Mutex<Vec<TextExpanderAppCandidate>>> = OnceLock::new();

fn text_expander_config() -> &'static RwLock<TextExpansionConfig> {
    TEXT_EXPANDER_CONFIG.get_or_init(|| RwLock::new(TextExpansionConfig::default()))
}

pub(super) fn text_expander_engine() -> &'static Mutex<TextExpansionEngine> {
    TEXT_EXPANDER_ENGINE.get_or_init(|| Mutex::new(TextExpansionEngine::default()))
}

fn recent_foreground_apps() -> &'static Mutex<Vec<TextExpanderAppCandidate>> {
    RECENT_FOREGROUND_APPS.get_or_init(|| Mutex::new(Vec::new()))
}

pub(super) fn current_process_name_lower() -> Option<String> {
    std::env::current_exe()
        .ok()
        .and_then(|path| path.file_name().map(|name| name.to_owned()))
        .and_then(|name| name.to_str().map(|name| name.to_ascii_lowercase()))
}

pub(super) fn remember_foreground_app(candidate: TextExpanderAppCandidate) {
    let exe = candidate.exe.trim().to_ascii_lowercase();
    if exe.is_empty() {
        return;
    }
    if current_process_name_lower().as_deref() == Some(exe.as_str()) {
        return;
    }
    let title = candidate.title.trim().to_owned();
    if let Ok(mut apps) = recent_foreground_apps().lock() {
        apps.retain(|app| app.exe != exe);
        apps.insert(0, TextExpanderAppCandidate { exe, title });
        apps.truncate(12);
    }
}

pub fn text_expander_app_candidates() -> Vec<TextExpanderAppCandidate> {
    let mut apps = platform_open_window_candidates();
    if let Ok(recent) = recent_foreground_apps().lock() {
        for candidate in recent.iter().rev() {
            if !apps.iter().any(|app| app.exe == candidate.exe) {
                apps.insert(0, candidate.clone());
            }
        }
    }
    apps.truncate(16);
    apps
}

pub fn set_text_expander_config(
    enabled: bool,
    rules: Vec<TextExpansionRule>,
    app_blacklist: Vec<String>,
) {
    let config = TextExpansionConfig {
        enabled,
        rules: rules.clone(),
        app_blacklist: app_blacklist
            .into_iter()
            .map(|name| name.trim().to_ascii_lowercase())
            .filter(|name| !name.is_empty())
            .collect(),
    };
    if let Ok(mut guard) = text_expander_config().write() {
        *guard = config;
    }
    if let Ok(mut engine) = text_expander_engine().lock() {
        engine.set_rules(rules);
    }
}

pub(super) fn text_expander_enabled() -> bool {
    text_expander_config()
        .read()
        .map(|config| config.enabled && config.rules.iter().any(|rule| rule.enabled))
        .unwrap_or(false)
}

pub(super) fn text_expander_suppressed_for_context() -> bool {
    text_expander_config()
        .read()
        .map(|config| foreground_app_blacklisted(&config.app_blacklist))
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
pub(super) fn remember_current_foreground_app() {
    if let Some(candidate) = foreground_app_candidate() {
        remember_foreground_app(candidate);
    }
}

#[cfg(not(target_os = "windows"))]
pub(super) fn remember_current_foreground_app() {}
