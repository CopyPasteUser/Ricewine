//! XP Luna Blue chrome. Visual-only helpers; application behavior stays in main.rs.

use eframe::egui::{self, Align2, Color32, FontId, Painter, Rect, RichText, Sense, Stroke};

const NAV_BLUE: Color32 = Color32::from_rgb(18, 66, 163);
const HEADER_LIGHT: Color32 = Color32::from_rgb(75, 151, 249);
const HEADER_DARK: Color32 = Color32::from_rgb(8, 62, 181);

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

pub fn title_bar(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 58.0), Sense::hover());
    let painter = ui.painter_at(rect);
    gradient(&painter, rect, HEADER_LIGHT, HEADER_DARK);
    painter.line_segment(
        [
            rect.left_top() + egui::vec2(0.0, 1.0),
            rect.right_top() + egui::vec2(0.0, 1.0),
        ],
        Stroke::new(2.0, Color32::from_rgb(162, 207, 255)),
    );
    painter.line_segment(
        [
            rect.left_bottom() - egui::vec2(0.0, 2.0),
            rect.right_bottom() - egui::vec2(0.0, 2.0),
        ],
        Stroke::new(2.0, Color32::from_rgb(7, 40, 128)),
    );
    wine_icon(&painter, rect.left_top() + egui::vec2(20.0, 10.0));
    painter.text(
        rect.left_top() + egui::vec2(56.0, 10.0),
        Align2::LEFT_TOP,
        "RiceWine",
        FontId::proportional(25.0),
        Color32::WHITE,
    );
    painter.text(
        rect.left_top() + egui::vec2(57.0, 39.0),
        Align2::LEFT_TOP,
        "WINDOWS APPLICATION MANAGER",
        FontId::proportional(10.0),
        Color32::from_rgb(221, 234, 255),
    );
    painter.text(
        rect.right_top() + egui::vec2(-22.0, 20.0),
        Align2::RIGHT_TOP,
        "XP LUNA  •  BLUE",
        FontId::proportional(12.0),
        Color32::WHITE,
    );
}

fn wine_icon(painter: &Painter, origin: egui::Pos2) {
    let bowl = Rect::from_min_size(origin + egui::vec2(4.0, 2.0), egui::vec2(23.0, 20.0));
    painter.rect_filled(
        bowl,
        egui::CornerRadius::same(7),
        Color32::from_rgb(245, 249, 255),
    );
    painter.rect_filled(
        Rect::from_min_size(origin + egui::vec2(6.0, 10.0), egui::vec2(19.0, 10.0)),
        egui::CornerRadius::same(5),
        Color32::from_rgb(161, 42, 73),
    );
    painter.rect_stroke(
        bowl,
        egui::CornerRadius::same(7),
        Stroke::new(1.5, Color32::from_rgb(218, 231, 255)),
        egui::StrokeKind::Inside,
    );
    painter.line_segment(
        [
            origin + egui::vec2(15.5, 22.0),
            origin + egui::vec2(15.5, 34.0),
        ],
        Stroke::new(2.0, Color32::WHITE),
    );
    painter.line_segment(
        [
            origin + egui::vec2(8.0, 34.0),
            origin + egui::vec2(23.0, 34.0),
        ],
        Stroke::new(2.0, Color32::WHITE),
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
        FontId::proportional(15.0),
        Color32::WHITE,
    );
}

pub fn task_panel(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(Color32::from_rgb(246, 249, 255))
        .stroke(Stroke::new(1.0, Color32::from_rgb(113, 153, 216)))
        .corner_radius(egui::CornerRadius::same(7))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui);
        });
}

pub fn brand(ui: &mut egui::Ui) {
    ui.add_space(9.0);
    ui.label(
        RichText::new("RiceWine")
            .size(29.0)
            .strong()
            .color(NAV_BLUE),
    );
    ui.label(
        RichText::new("YOUR WINDOWS PROGRAMS")
            .size(10.0)
            .color(Color32::from_rgb(57, 86, 139)),
    );
    ui.add_space(14.0);
}

pub fn content_heading(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 76.0), Sense::hover());
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
        rect.left_top() + egui::vec2(16.0, 23.0),
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
        rect.left_top() + egui::vec2(57.0, 13.0),
        Align2::LEFT_TOP,
        title,
        FontId::proportional(24.0),
        NAV_BLUE,
    );
    painter.text(
        rect.left_top() + egui::vec2(58.0, 47.0),
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

fn program_list_header(ui: &mut egui::Ui) {
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
    for (offset, heading) in [(14.0, "Name"), (0.50, "Environment"), (0.77, "Runner")] {
        let x = if offset > 1.0 {
            rect.left() + offset
        } else {
            rect.left() + offset * rect.width()
        };
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
