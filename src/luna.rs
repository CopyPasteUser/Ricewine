//! XP Luna Blue application surfaces. Window decorations belong to the compositor.

use eframe::egui::{self, Align2, Color32, FontId, Painter, Rect, RichText, Sense, Stroke};

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
    painter.text(
        rect.left_center() + egui::vec2(11.0, 0.0),
        Align2::LEFT_CENTER,
        heading,
        FontId::proportional(13.0),
        Color32::WHITE,
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
    let icon = Rect::from_min_size(
        rect.left_top() + egui::vec2(16.0, 18.0),
        egui::vec2(27.0, 22.0),
    );
    painter.rect_filled(
        icon,
        egui::CornerRadius::same(3),
        Color32::from_rgb(250, 188, 54),
    );
    painter.rect_stroke(
        icon,
        egui::CornerRadius::same(3),
        Stroke::new(1.5, Color32::from_rgb(174, 112, 18)),
        egui::StrokeKind::Inside,
    );
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

/// Draws an Explorer-style program list even before the first installation.
/// The empty state exposes the real install/import actions, never sample data.
pub fn empty_library(ui: &mut egui::Ui, enabled: bool) -> EmptyLibraryAction {
    let mut action = EmptyLibraryAction::None;
    egui::Frame::new()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0, Color32::from_rgb(146, 164, 183)))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.set_min_height(ui.available_height().max(310.0));
            program_list_header(ui);
            ui.add_space(70.0);
            ui.vertical_centered(|ui| {
                folder_illustration(ui);
                ui.add_space(12.0);
                ui.label(
                    RichText::new("Your library is ready")
                        .size(21.0)
                        .strong()
                        .color(NAV_BLUE),
                );
                ui.add_space(3.0);
                ui.label(
                    RichText::new("Install your first Windows app to see it here.")
                        .size(13.0)
                        .color(Color32::from_rgb(74, 82, 101)),
                );
                ui.add_space(19.0);
                if ui
                    .add_enabled(
                        enabled,
                        egui::Button::new(
                            RichText::new("＋  Install a program")
                                .color(Color32::WHITE)
                                .strong(),
                        )
                        .fill(Color32::from_rgb(64, 148, 41))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(36, 104, 25)))
                        .min_size(egui::vec2(200.0, 35.0)),
                    )
                    .clicked()
                {
                    action = EmptyLibraryAction::Install;
                }
                ui.add_space(4.0);
                if ui
                    .add_enabled(
                        enabled,
                        egui::Button::new("Import an existing Wine prefix…"),
                    )
                    .clicked()
                {
                    action = EmptyLibraryAction::Import;
                }
            });
        });
    action
}

#[derive(Debug, PartialEq, Eq)]
pub enum EmptyLibraryAction {
    None,
    Install,
    Import,
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

fn folder_illustration(ui: &mut egui::Ui) {
    let available = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(available, 64.0), Sense::hover());
    let center = rect.center();
    let painter = ui.painter_at(rect);
    let tab = Rect::from_min_size(center + egui::vec2(-30.0, -23.0), egui::vec2(29.0, 13.0));
    painter.rect_filled(
        tab,
        egui::CornerRadius::same(4),
        Color32::from_rgb(240, 176, 44),
    );
    let folder = Rect::from_min_size(center + egui::vec2(-33.0, -14.0), egui::vec2(67.0, 45.0));
    painter.rect_filled(
        folder,
        egui::CornerRadius::same(5),
        Color32::from_rgb(255, 206, 78),
    );
    painter.rect_stroke(
        folder,
        egui::CornerRadius::same(5),
        Stroke::new(2.0, Color32::from_rgb(185, 129, 30)),
        egui::StrokeKind::Inside,
    );
    painter.line_segment(
        [
            folder.left_top() + egui::vec2(5.0, 8.0),
            folder.right_top() + egui::vec2(-5.0, 8.0),
        ],
        Stroke::new(2.0, Color32::from_rgb(255, 242, 178)),
    );
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
