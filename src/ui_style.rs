use egui::{Color32, Stroke, Vec2};

pub fn accent() -> Color32 {
    Color32::from_rgb(91, 104, 223)
}

pub fn panel_fill(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(30, 30, 30)
    } else {
        Color32::from_rgb(245, 245, 245)
    }
}

pub fn window_fill(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(37, 37, 38)
    } else {
        Color32::from_rgb(255, 255, 255)
    }
}

pub fn surface_fill(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(45, 45, 48)
    } else {
        Color32::from_rgb(255, 255, 255)
    }
}

pub fn hover_fill(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(60, 60, 65)
    } else {
        Color32::from_rgb(232, 232, 240)
    }
}

pub fn border_color(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(72, 72, 78)
    } else {
        Color32::from_rgb(220, 220, 228)
    }
}

pub fn muted_text(dark: bool) -> Color32 {
    if dark {
        Color32::from_gray(150)
    } else {
        Color32::from_gray(120)
    }
}

pub fn modal_outline_stroke(dark: bool) -> Stroke {
    if dark {
        Stroke::new(1.0, Color32::from_gray(110))
    } else {
        Stroke::new(1.0, Color32::from_gray(175))
    }
}

pub fn modal_action_button_size() -> Vec2 {
    Vec2::new(104.0, 32.0)
}

pub fn modal_window_frame(style: &egui::Style, dark: bool) -> egui::Frame {
    egui::Frame::window(style)
        .fill(window_fill(dark))
        .stroke(egui::Stroke::NONE)
        .inner_margin(egui::Margin::same(10))
}

pub fn modal_backdrop_alpha(dark: bool) -> u8 {
    if dark { 96 } else { 48 }
}
