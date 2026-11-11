//! XP Luna Blue application surfaces. Window decorations belong to the compositor.

use eframe::egui::{self, Align2, Color32, FontId, Painter, Rect, RichText, Sense, Stroke};
use crate::luna_icons::paint_icon;
pub use crate::luna_icons::XpIcon;

const NAV_BLUE: Color32 = Color32::from_rgb(18, 66, 163);

fn gradient(painter: &Painter, rect: Rect, top: Color32, bottom: Color32) {
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), top);
    mesh.colored_vertex(rect.right_top(), top);
    mesh.colored_vertex(rect.right_bottom(), bottom);
    mesh.colored_vertex(rect.left_bottom(), bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(egui::Shape::mesh(mesh));
}

/// Actions returned by the classic XP menu bar. The caller owns navigation
/// and installation state so the visual layer cannot mutate the Wine catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuAction {
    None,
    Install,
    Import,
    Library,
    Prefixes,
    Runners,
    Settings,
    About,
}

/// A genuine application menu bar, below the native window decoration.
pub fn application_menu(ui: &mut egui::Ui) -> MenuAction {
    let mut action = MenuAction::None;
    egui::Frame::new()
        .fill(Color32::from_rgb(242, 240, 235))
        .stroke(Stroke::new(1.0, Color32::from_rgb(178, 179, 175)))
        .inner_margin(egui::Margin::symmetric(8, 2))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Install program…").clicked() {
                        action = MenuAction::Install;
                        ui.close();
                    }
                    if ui.button("Import existing prefix…").clicked() {
                        action = MenuAction::Import;
                        ui.close();
                    }
                });
                ui.menu_button("View", |ui| {
                    for (label, next) in [
                        ("Library", MenuAction::Library),
                        ("Prefixes", MenuAction::Prefixes),
                        ("Runners", MenuAction::Runners),
                        ("Settings", MenuAction::Settings),
                    ] {
                        if ui.button(label).clicked() {
                            action = next;
                            ui.close();
                        }
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About RiceWine…").clicked() {
                        action = MenuAction::About;
                        ui.close();
                    }
                });
            });
        });
    action
}

/// Explorer-era address toolbar. This is an application control, never a
/// simulated title bar: the compositor and RiceBowl own window decorations.
pub fn application_toolbar(ui: &mut egui::Ui, section: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 35.0), Sense::hover());
    let painter = ui.painter_at(rect);
    gradient(
        &painter,
        rect,
        Color32::from_rgb(255, 255, 255),
        Color32::from_rgb(226, 232, 243),
    );
    painter.line_segment(
        [
            rect.left_bottom() - egui::vec2(0.0, 1.0),
            rect.right_bottom() - egui::vec2(0.0, 1.0),
        ],
        Stroke::new(1.0, Color32::from_rgb(153, 167, 189)),
    );
    painter.text(
        rect.left_center() + egui::vec2(13.0, 0.0),
        Align2::LEFT_CENTER,
        "Address",
        FontId::proportional(12.0),
        Color32::from_rgb(55, 61, 72),
    );
    let field = Rect::from_min_max(
        rect.left_top() + egui::vec2(72.0, 5.0),
        rect.right_bottom() - egui::vec2(12.0, 5.0),
    );
    painter.rect_filled(field, egui::CornerRadius::same(1), Color32::WHITE);
    painter.rect_stroke(
        field,
        egui::CornerRadius::same(1),
        Stroke::new(1.0, Color32::from_rgb(132, 150, 176)),
        egui::StrokeKind::Inside,
    );
    painter.text(
        field.left_center() + egui::vec2(9.0, 0.0),
        Align2::LEFT_CENTER,
        format!("RiceWine  ›  {section}"),
        FontId::proportional(12.0),
        Color32::from_rgb(36, 50, 82),
    );
}

pub fn task_header(ui: &mut egui::Ui, heading: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 34.0), Sense::hover());
    let painter = ui.painter_at(rect);
    gradient(&painter, rect, Color32::from_rgb(55, 117, 219), NAV_BLUE);
    painter.line_segment(
        [
            rect.left_top() + egui::vec2(2.0, 2.0),
            rect.right_top() - egui::vec2(2.0, -2.0),
        ],
        Stroke::new(1.0, Color32::from_rgb(139, 185, 255)),
    );
    painter.text(
        rect.left_center() + egui::vec2(11.0, 0.0),
        Align2::LEFT_CENTER,
        heading,
        FontId::proportional(13.0),
        Color32::WHITE,
    );
    let chevron = rect.right_center() - egui::vec2(16.0, 0.0);
    painter.circle_filled(chevron, 9.0, Color32::from_rgb(218, 232, 255));
    painter.circle_stroke(
        chevron,
        9.0,
        Stroke::new(1.0, Color32::from_rgb(118, 159, 228)),
    );
    painter.line_segment(
        [
            chevron + egui::vec2(-3.0, 1.0),
            chevron + egui::vec2(0.0, -2.0),
        ],
        Stroke::new(1.5, NAV_BLUE),
    );
    painter.line_segment(
        [
            chevron + egui::vec2(0.0, -2.0),
            chevron + egui::vec2(3.0, 1.0),
        ],
        Stroke::new(1.5, NAV_BLUE),
    );
}

pub fn task_panel(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(Color32::from_rgb(246, 249, 255))
        .stroke(Stroke::new(1.0, Color32::from_rgb(113, 153, 216)))
        .corner_radius(egui::CornerRadius::same(5))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui);
        });
}

pub fn brand(ui: &mut egui::Ui) {
    ui.add_space(3.0);
    ui.label(RichText::new("Tasks").size(12.0).strong().color(NAV_BLUE));
    ui.add_space(7.0);
}

/// XP Explorer task links are compact blue text, not modern filled navigation
/// buttons. Retaining egui::Button preserves focus and keyboard activation.
pub fn task_link(ui: &mut egui::Ui, label: &str, icon: XpIcon, selected: bool) -> egui::Response {
    let text = RichText::new(format!("        {label}"))
        .size(12.0)
        .color(if selected {
            Color32::from_rgb(22, 54, 117)
        } else {
            NAV_BLUE
        });
    let text = if selected { text.strong() } else { text };
    let response = ui.add_sized(
        [ui.available_width() - 12.0, 26.0],
        egui::Button::new(text).frame(false),
    );
    let origin = response.rect.left_center() + egui::vec2(8.0, -8.0);
    paint_icon(ui.painter(), origin, icon, 16.0);
    if response.hovered() {
        ui.painter().line_segment(
            [
                response.rect.left_bottom() + egui::vec2(28.0, -5.0),
                response.rect.left_bottom() + egui::vec2(response.rect.width() - 8.0, -5.0),
            ],
            Stroke::new(0.8, Color32::from_rgb(87, 130, 203)),
        );
    }
    response
}

/// A classic beveled push-button with native egui focus and keyboard behavior.
/// The highlight/shadow edges are decorative; button interaction stays native.
pub fn xp_button(ui: &mut egui::Ui, text: &str, enabled: bool) -> egui::Response {
    let response = ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(text).size(12.0))
            .min_size(egui::vec2(98.0, 27.0))
            .corner_radius(egui::CornerRadius::same(3)),
    );
    let r = response.rect.shrink(1.0);
    if enabled {
        let pressed = response.is_pointer_button_down_on();
        let light = if pressed {
            Color32::from_rgb(126, 150, 190)
        } else {
            Color32::WHITE
        };
        let dark = if pressed {
            Color32::WHITE
        } else {
            Color32::from_rgb(125, 140, 164)
        };
        ui.painter().line_segment(
            [r.left_top() + egui::vec2(2.0, 1.0), r.right_top() - egui::vec2(2.0, -1.0)],
            Stroke::new(1.0, light),
        );
        ui.painter().line_segment(
            [r.left_bottom() + egui::vec2(2.0, -1.0), r.right_bottom() - egui::vec2(2.0, 1.0)],
            Stroke::new(1.0, dark),
        );
    }
    response
}

/// XP-like bordered option group. Unlike a card, this follows the traditional
/// Control Panel fieldset hierarchy and leaves all children interactive.
pub fn option_group(
    ui: &mut egui::Ui,
    title: &str,
    icon: XpIcon,
    add: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(19.0, 19.0), Sense::hover());
        paint_icon(ui.painter(), rect.min, icon, 18.0);
        ui.label(RichText::new(title).size(13.0).strong().color(NAV_BLUE));
    });
    // Keep a consistent fieldset width without overflowing its 12px margins.
    let inner_width = (ui.available_width() - 24.0).max(0.0);
    egui::Frame::new()
        .stroke(Stroke::new(1.0, Color32::from_rgb(166, 179, 199)))
        .fill(Color32::from_rgb(255, 255, 254))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.set_min_width(inner_width);
            add(ui);
        });
    ui.add_space(12.0);
}

/// Distinct visual hierarchy for older-style modal form content.
pub fn dialog_intro(ui: &mut egui::Ui, icon: XpIcon, title: &str, subtitle: &str) {
    egui::Frame::new()
        .fill(Color32::WHITE)
        .inner_margin(egui::Margin::symmetric(10, 10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(36.0, 36.0), Sense::hover());
                paint_icon(ui.painter(), rect.min, icon, 34.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(title).size(15.0).strong().color(NAV_BLUE));
                    ui.small(subtitle);
                });
            });
        });
    ui.separator();
    ui.add_space(9.0);
}


pub fn content_heading(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 65.0), Sense::hover());
    let painter = ui.painter_at(rect);
    gradient(
        &painter,
        rect,
        Color32::from_rgb(255, 255, 252),
        Color32::from_rgb(246, 241, 220),
    );
    painter.rect_stroke(
        rect,
        egui::CornerRadius::same(2),
        Stroke::new(1.0, Color32::from_rgb(209, 195, 153)),
        egui::StrokeKind::Inside,
    );
    let icon = match title {
        "Program Library" => XpIcon::Programs,
        "Wine Prefixes" => XpIcon::Prefix,
        "Compatibility Runners" => XpIcon::Runner,
        _ => XpIcon::Settings,
    };
    paint_icon(&painter, rect.left_top() + egui::vec2(15.0, 16.0), icon, 33.0);
    painter.text(
        rect.left_top() + egui::vec2(57.0, 8.0),
        Align2::LEFT_TOP,
        title,
        FontId::proportional(21.0),
        NAV_BLUE,
    );
    painter.text(
        rect.left_top() + egui::vec2(58.0, 39.0),
        Align2::LEFT_TOP,
        subtitle,
        FontId::proportional(12.0),
        Color32::from_rgb(84, 91, 107),
    );
}

/// Explorer details pane empty state. The actionable install/import links live
/// in the task pane and File menu; the list itself stays visually quiet.
pub fn empty_library(ui: &mut egui::Ui) {
    egui::Frame::new()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0, Color32::from_rgb(146, 164, 183)))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.set_min_height(ui.available_height().max(310.0));
            program_list_header(ui);
            ui.add_space(18.0);
            ui.horizontal(|ui| {
                ui.add_space(14.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("There are no Windows programs to display.")
                            .size(12.0)
                            .color(Color32::from_rgb(52, 64, 84)),
                    );
                    ui.add_space(3.0);
                    ui.label(
                        RichText::new(
                            "To get started, choose Install a program in the Tasks pane.",
                        )
                        .size(11.0)
                        .color(Color32::from_rgb(85, 97, 117)),
                    );
                });
            });
        });
}

/// A compact icon-bearing Explorer list row. The selectable widget retains
/// keyboard navigation and selection semantics.
pub fn program_row(ui: &mut egui::Ui, name: &str, selected: bool, width: f32) -> egui::Response {
    let response = ui.add_sized(
        [width, 28.0],
        egui::Button::selectable(selected, format!("       {name}")),
    );
    paint_icon(
        ui.painter(),
        response.rect.left_center() + egui::vec2(7.0, -8.0),
        XpIcon::Programs,
        16.0,
    );
    response
}

pub fn program_list_header(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 35.0), Sense::hover());
    let painter = ui.painter_at(rect);
    gradient(
        &painter,
        rect,
        Color32::from_rgb(255, 255, 250),
        Color32::from_rgb(228, 225, 211),
    );
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, Color32::from_rgb(153, 162, 176)),
    );
    for (x, heading) in [
        (rect.left() + 14.0, "Name"),
        (rect.right() - 105.0, "Runner"),
    ] {
        painter.text(
            egui::pos2(x, rect.center().y),
            Align2::LEFT_CENTER,
            heading,
            FontId::proportional(13.0),
            Color32::from_rgb(50, 61, 82),
        );
    }
}

/// An XP-style status strip, part of the application surface.
pub fn status_bar(ui: &mut egui::Ui, text: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 23.0), Sense::hover());
    let painter = ui.painter_at(rect);
    gradient(
        &painter,
        rect,
        Color32::from_rgb(242, 242, 238),
        Color32::from_rgb(222, 226, 231),
    );
    painter.line_segment(
        [rect.left_top(), rect.right_top()],
        Stroke::new(1.0, Color32::from_rgb(148, 155, 169)),
    );
    painter.text(
        rect.left_center() + egui::vec2(12.0, 0.0),
        Align2::LEFT_CENTER,
        text,
        FontId::proportional(11.0),
        Color32::from_rgb(53, 63, 78),
    );
}
