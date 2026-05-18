#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TourTarget {
    MainNavigation,
    DeviceSelector,
    LayerSwitcher,
    KeyboardArea,
    SettingsMenu,
    BottomHints,
}

#[derive(Clone, Copy)]
pub(crate) struct TourStep {
    pub(crate) target: Option<TourTarget>,
    pub(crate) title_key: &'static str,
    pub(crate) body_key: &'static str,
}

pub(crate) const ONBOARDING_TOUR_STEPS: [TourStep; 7] = [
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
pub(crate) struct TourState {
    pub(crate) active: bool,
    pub(crate) step: usize,
}
