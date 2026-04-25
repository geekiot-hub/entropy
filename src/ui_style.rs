use egui::{Color32, RichText, Stroke, Ui, Vec2};

pub fn accent() -> Color32 {
    Color32::from_rgb(196, 132, 144)
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
        Color32::from_rgb(64, 54, 58)
    } else {
        Color32::from_rgb(244, 232, 234)
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

pub fn modal_tab_button_size() -> Vec2 {
    Vec2::new(52.0, 28.0)
}

pub fn modal_tab_add_button_size() -> Vec2 {
    Vec2::new(28.0, 28.0)
}

pub fn modal_field_button_height() -> f32 {
    34.0
}

pub fn modal_field_button_size(width: f32) -> Vec2 {
    Vec2::new(width, modal_field_button_height())
}

pub fn modal_small_button_size(width: f32) -> Vec2 {
    Vec2::new(width, 32.0)
}

pub fn modal_space_xs() -> f32 {
    4.0
}

pub fn modal_space_sm() -> f32 {
    8.0
}

pub fn modal_space_md() -> f32 {
    12.0
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

#[derive(Debug, Clone, Copy)]
pub struct ModalLayout {
    pub content_width: f32,
    pub top_padding: f32,
}

impl ModalLayout {
    pub fn new(content_width: f32) -> Self {
        Self {
            content_width,
            top_padding: 6.0,
        }
    }

    pub fn with_top_padding(mut self, top_padding: f32) -> Self {
        self.top_padding = top_padding;
        self
    }
}

pub fn centered_modal_window<'a>(
    ctx: &egui::Context,
    title: &'a str,
    id: egui::Id,
    open: &'a mut bool,
    size: Vec2,
) -> egui::Window<'a> {
    egui::Window::new(title)
        .id(id)
        .open(open)
        .collapsible(false)
        .resizable(false)
        .movable(true)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .fixed_size(size)
        .frame(modal_window_frame(ctx.style().as_ref(), ctx.style().visuals.dark_mode))
}

pub fn modal_content(ui: &mut Ui, layout: ModalLayout, add_contents: impl FnOnce(&mut Ui)) {
    ui.vertical_centered(|ui| {
        if layout.top_padding > 0.0 {
            ui.add_space(layout.top_padding);
        }
        ui.allocate_ui_with_layout(
            Vec2::new(layout.content_width, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            add_contents,
        );
    });
}

pub fn modal_section_title(ui: &mut Ui, title: &str) {
    ui.label(RichText::new(title).size(12.5).strong());
}

pub fn modal_intro(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .size(11.0)
            .color(Color32::from_gray(if ui.visuals().dark_mode { 140 } else { 140 })),
    );
}

pub fn modal_hint(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text)
            .size(11.0)
            .color(muted_text(ui.visuals().dark_mode)),
    );
}

pub fn modal_empty_state(ui: &mut Ui, title: &str, detail: Option<&str>) {
    let dark = ui.visuals().dark_mode;
    ui.vertical_centered(|ui| {
        ui.add_space(72.0);
        ui.label(
            RichText::new(title)
                .size(13.0)
                .color(muted_text(dark)),
        );
        if let Some(detail) = detail {
            ui.add_space(6.0);
            ui.label(
                RichText::new(detail)
                    .size(11.5)
                    .color(muted_text(dark)),
            );
        }
    });
}

pub fn modal_action_row(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    ui.add_space(18.0);
    ui.horizontal_centered(add_contents);
}

pub fn modal_centered_text_block(
    ui: &mut Ui,
    width: f32,
    add_contents: impl FnOnce(&mut Ui),
) {
    ui.horizontal_centered(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(width, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            add_contents,
        );
    });
}

pub fn modal_checkbox_label_row(
    ui: &mut Ui,
    content_width: f32,
    row_height: f32,
    checked: &mut bool,
    label: &str,
    checkbox_label_gap: f32,
) -> bool {
    let mut changed = false;
    ui.allocate_ui_with_layout(
        egui::vec2(content_width, row_height),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.horizontal_centered(|ui| {
                let resp = ui.add(egui::Checkbox::without_text(checked));
                if resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if resp.changed() {
                    changed = true;
                }
                if checkbox_label_gap > 0.0 {
                    ui.add_space(checkbox_label_gap);
                }
                ui.label(label);
            });
        },
    );
    changed
}

pub fn modal_labeled_row(
    ui: &mut Ui,
    content_width: f32,
    label_width: f32,
    row_height: f32,
    add_label: impl FnOnce(&mut Ui),
    add_control: impl FnOnce(&mut Ui),
) {
    let control_width = (content_width - label_width).max(0.0);
    ui.allocate_ui_with_layout(
        egui::vec2(content_width, row_height),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(label_width, row_height),
                egui::Layout::left_to_right(egui::Align::Center),
                add_label,
            );
            ui.allocate_ui_with_layout(
                egui::vec2(control_width, row_height),
                egui::Layout::left_to_right(egui::Align::Center),
                add_control,
            );
        },
    );
}
