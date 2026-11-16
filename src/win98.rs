//! Windows 98 SE Add/Remove Programs–style RiceWine interior.
//! Original primitive-drawn controls; Linux owns the actual window decoration.
//! All Wine operations and dialog confirmations stay in RiceWineApp.

use eframe::egui::{self, Align2, Color32, FontId, Rect, Response, RichText, Sense, Stroke, Vec2};

pub const FACE: Color32 = Color32::from_rgb(192, 192, 192);
pub const PAPER: Color32 = Color32::WHITE;
pub const LIGHT: Color32 = Color32::WHITE;
pub const SHADOW: Color32 = Color32::from_rgb(128, 128, 128);
pub const DARK: Color32 = Color32::from_rgb(64, 64, 64);
pub const INK: Color32 = Color32::from_rgb(0, 0, 0);
pub const NAVY: Color32 = Color32::from_rgb(0, 0, 128);

/// The selected tab is visually joined to the property sheet rather than
/// looking like a modern navigation button.
pub fn tab(ui: &mut egui::Ui, text: &str, selected: bool) -> Response {
    let button = egui::Button::new(RichText::new(text).size(12.0).color(INK))
        .fill(if selected {
            FACE
        } else {
            Color32::from_rgb(212, 212, 212)
        })
        .stroke(Stroke::new(1.0, if selected { SHADOW } else { DARK }))
        .corner_radius(egui::CornerRadius::ZERO);
    let width = (text.len() as f32 * 6.8 + 24.0).max(78.0);
    let response = ui.add_sized([width, if selected { 31.0 } else { 28.0 }], button);
    if selected {
        let painter = ui.painter();
        painter.line_segment(
            [response.rect.left_top(), response.rect.right_top()],
            Stroke::new(1.0, LIGHT),
        );
        painter.line_segment(
            [response.rect.left_top(), response.rect.left_bottom()],
            Stroke::new(1.0, LIGHT),
        );
        // Hide the bottom edge on the selected tab to connect it to the sheet.
        painter.line_segment(
            [response.rect.left_bottom(), response.rect.right_bottom()],
            Stroke::new(2.0, FACE),
        );
    }
    response
}

pub fn property_sheet() -> egui::Frame {
    egui::Frame::new()
        .fill(FACE)
        .stroke(Stroke::new(1.0, DARK))
        .inner_margin(egui::Margin::same(9))
}

pub fn sunken_list() -> egui::Frame {
    egui::Frame::new()
        .fill(PAPER)
        .stroke(Stroke::new(1.0, SHADOW))
        .inner_margin(egui::Margin::same(2))
}

pub fn push_button(ui: &mut egui::Ui, label: &str, enabled: bool) -> Response {
    let response = ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(label).size(12.0).color(INK))
            .fill(FACE)
            .stroke(Stroke::new(1.0, DARK))
            .corner_radius(egui::CornerRadius::ZERO)
            .min_size(Vec2::new(72.0, 24.0)),
    );
    let rect = response.rect;
    if enabled && !response.is_pointer_button_down_on() {
        ui.painter().line_segment(
            [
                rect.left_top() + Vec2::splat(1.0),
                rect.right_top() - Vec2::new(1.0, -1.0),
            ],
            Stroke::new(1.0, LIGHT),
        );
        ui.painter().line_segment(
            [
                rect.left_top() + Vec2::splat(1.0),
                rect.left_bottom() - Vec2::new(-1.0, 1.0),
            ],
            Stroke::new(1.0, LIGHT),
        );
    }
    response
}

pub fn caption(ui: &mut egui::Ui, title: &str, explanation: &str) {
    ui.horizontal(|ui| {
        crate::rice_identity::symbol(
            ui,
            crate::rice_identity::Family::Win98,
            crate::rice_identity::Icon::Programs,
            NAVY,
            26.0,
        );
        ui.vertical(|ui| {
            ui.label(RichText::new(title).strong().size(12.5).color(INK));
            ui.label(RichText::new(explanation).size(11.5).color(INK));
        });
    });
}

pub fn list_header(ui: &mut egui::Ui) {
    let width = ui.available_width().max(0.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 24.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, FACE);
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, SHADOW),
    );
    let status_width = if width >= 250.0 { 110.0 } else { 72.0 };
    let divider_x = rect.right() - status_width;
    painter.line_segment(
        [
            egui::pos2(divider_x, rect.top()),
            egui::pos2(divider_x, rect.bottom()),
        ],
        Stroke::new(1.0, SHADOW),
    );
    painter.text(
        rect.left_center() + Vec2::new(8.0, 0.0),
        Align2::LEFT_CENTER,
        "Program",
        FontId::proportional(12.0),
        INK,
    );
    painter.text(
        egui::pos2(divider_x + 8.0, rect.center().y),
        Align2::LEFT_CENTER,
        "Status",
        FontId::proportional(12.0),
        INK,
    );
}

/// One full-width details-list row. Selection includes status and icon;
/// text is clipped per column so long file names cannot move the status.
pub fn program_row(ui: &mut egui::Ui, name: &str, status: &str, selected: bool) -> Response {
    let width = ui.available_width().max(0.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 27.0), Sense::click());
    let fg = if selected { PAPER } else { INK };
    let bg = if selected { NAVY } else { PAPER };
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, bg);
    let icon_rect =
        Rect::from_center_size(rect.left_center() + Vec2::new(13.0, 0.0), Vec2::splat(16.0));
    crate::rice_identity::paint(
        &painter,
        icon_rect,
        crate::rice_identity::Family::Win98,
        crate::rice_identity::Icon::Programs,
        if selected { PAPER } else { NAVY },
    );
    let status_width = if width >= 250.0 { 110.0 } else { 72.0 };
    let divider_x = rect.right() - status_width;
    let name_clip = Rect::from_min_max(
        rect.min + Vec2::new(28.0, 0.0),
        egui::pos2((divider_x - 5.0).max(rect.left() + 28.0), rect.bottom()),
    );
    painter.with_clip_rect(name_clip).text(
        rect.left_center() + Vec2::new(29.0, 0.0),
        Align2::LEFT_CENTER,
        name,
        FontId::proportional(12.0),
        fg,
    );
    painter
        .with_clip_rect(Rect::from_min_max(
            egui::pos2(divider_x + 4.0, rect.top()),
            rect.right_bottom(),
        ))
        .text(
            egui::pos2(divider_x + 6.0, rect.center().y),
            Align2::LEFT_CENTER,
            status,
            FontId::proportional(11.0),
            fg,
        );
    if response.has_focus() {
        painter.rect_stroke(
            rect.shrink(2.0),
            0.0,
            Stroke::new(1.0, fg),
            egui::StrokeKind::Inside,
        );
    }
    response.widget_info(|| {
        let mut info = egui::WidgetInfo::labeled(
            egui::Role::Button,
            ui.is_enabled(),
            format!("{name}, {status}"),
        );
        info.selected = Some(selected);
        info
    });
    response.on_hover_text(format!("{name} · {status}"))
}

pub fn status_bar(ui: &mut egui::Ui, count: usize) {
    ui.horizontal(|ui| {
        egui::Frame::new()
            .fill(FACE)
            .stroke(Stroke::new(1.0, SHADOW))
            .inner_margin(egui::Margin::symmetric(6, 3))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!("{count} program(s)"))
                        .size(11.0)
                        .color(INK),
                );
            });
        egui::Frame::new()
            .fill(FACE)
            .stroke(Stroke::new(1.0, SHADOW))
            .inner_margin(egui::Margin::symmetric(6, 3))
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Wine prefixes are preserved on removal")
                        .size(11.0)
                        .color(INK),
                );
            });
    });
}

pub fn dialog_intro(ui: &mut egui::Ui, title: &str, description: &str) {
    caption(ui, title, description);
    ui.add_space(6.0);
    ui.separator();
    ui.add_space(4.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classic_palette_has_high_contrast_and_no_round_corners() {
        assert_eq!(FACE, Color32::from_rgb(192, 192, 192));
        assert_eq!(NAVY, Color32::from_rgb(0, 0, 128));
        assert_eq!(PAPER, Color32::WHITE);
        assert_ne!(INK, FACE);
    }
}
