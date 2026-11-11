//! XP Luna Blue chrome. Visual-only helpers; application behavior stays in main.rs.

use eframe::egui::{self, Align2, Color32, FontId, Painter, Rect, RichText, Sense, Stroke};

// Small shared palette and hand-drawn icons keep Luna independent of system fonts.
pub const NAV_BLUE: Color32 = Color32::from_rgb(24, 70, 158);
const HEADER_LIGHT: Color32 = Color32::from_rgb(79, 153, 252);
const HEADER_MID: Color32 = Color32::from_rgb(30, 104, 218);
const HEADER_DARK: Color32 = Color32::from_rgb(10, 51, 145);
const INK: Color32 = Color32::from_rgb(39, 57, 91);
const MUTED: Color32 = Color32::from_rgb(87, 102, 131);
const HAIRLINE: Color32 = Color32::from_rgb(174, 191, 222);

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
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 68.0), Sense::hover());
    let painter = ui.painter_at(rect);
    let (top, lower) = rect.split_top_bottom_at_y(rect.top() + 31.0);
    gradient(&painter, top, HEADER_LIGHT, HEADER_MID);
    gradient(&painter, lower, HEADER_MID, HEADER_DARK);
    painter.line_segment(
        [
            rect.left_top() + egui::vec2(0.0, 2.0),
            rect.right_top() + egui::vec2(0.0, 2.0),
        ],
        Stroke::new(3.0, Color32::from_rgb(159, 212, 255)),
    );
    painter.line_segment(
        [
            rect.left_bottom() - egui::vec2(0.0, 2.0),
            rect.right_bottom() - egui::vec2(0.0, 2.0),
        ],
        Stroke::new(3.0, Color32::from_rgb(5, 37, 115)),
    );
    wine_icon(&painter, rect.left_top() + egui::vec2(22.0, 12.0));
    painter.text(
        rect.left_top() + egui::vec2(62.0, 10.5),
        Align2::LEFT_TOP,
        "RiceWine",
        FontId::proportional(28.0),
        Color32::from_rgb(15, 55, 137),
    );
    painter.text(
        rect.left_top() + egui::vec2(61.0, 9.0),
        Align2::LEFT_TOP,
        "RiceWine",
        FontId::proportional(28.0),
        Color32::WHITE,
    );
    painter.text(
        rect.left_top() + egui::vec2(62.0, 44.0),
        Align2::LEFT_TOP,
        "YOUR WINDOWS APPS, ONE PLACE",
        FontId::proportional(11.0),
        Color32::from_rgb(221, 234, 255),
    );
    if rect.width() >= 690.0 {
        let badge = Rect::from_min_size(
            rect.right_top() + egui::vec2(-158.0, 20.0),
            egui::vec2(135.0, 29.0),
        );
        painter.rect_filled(
            badge,
            egui::CornerRadius::same(5),
            Color32::from_rgba_unmultiplied(5, 39, 119, 100),
        );
        painter.rect_stroke(
            badge,
            egui::CornerRadius::same(5),
            Stroke::new(1.0, Color32::from_rgb(129, 185, 247)),
            egui::StrokeKind::Inside,
        );
        painter.text(
            badge.center(),
            Align2::CENTER_CENTER,
            "LUNA BLUE  ·  2.0",
            FontId::proportional(12.0),
            Color32::WHITE,
        );
    }
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
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 39.0), Sense::hover());
    let painter = ui.painter_at(rect);
    gradient(
        &painter,
        rect,
        Color32::from_rgb(84, 151, 240),
        Color32::from_rgb(22, 72, 177),
    );
    painter.line_segment(
        [
            rect.left_top() + egui::vec2(1.0, 1.0),
            rect.right_top() + egui::vec2(-1.0, 1.0),
        ],
        Stroke::new(1.0, Color32::from_rgb(193, 227, 255)),
    );
    painter.text(
        rect.left_center() + egui::vec2(13.0, 0.0),
        Align2::LEFT_CENTER,
        heading,
        FontId::proportional(15.5),
        Color32::WHITE,
    );
    painter.text(
        rect.right_center() + egui::vec2(-14.0, 0.0),
        Align2::RIGHT_CENTER,
        "⌃",
        FontId::proportional(18.0),
        Color32::from_rgb(223, 237, 255),
    );
}

pub fn task_panel(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(Color32::from_rgb(248, 250, 255))
        .stroke(Stroke::new(1.0, Color32::from_rgb(97, 140, 215)))
        .corner_radius(egui::CornerRadius::same(6))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui);
        });
}

pub fn brand(ui: &mut egui::Ui) {
    ui.add_space(11.0);
    ui.label(
        RichText::new("RiceWine")
            .size(27.0)
            .strong()
            .color(NAV_BLUE),
    );
    ui.label(
        RichText::new("EXPLORER FOR WINDOWS APPS")
            .size(10.0)
            .color(Color32::from_rgb(49, 82, 137)),
    );
    ui.add_space(17.0);
}

pub fn content_heading(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 91.0), Sense::hover());
    let painter = ui.painter_at(rect);
    gradient(
        &painter,
        rect,
        Color32::from_rgb(255, 255, 251),
        Color32::from_rgb(241, 239, 221),
    );
    painter.rect_stroke(
        rect,
        egui::CornerRadius::same(2),
        Stroke::new(1.0, Color32::from_rgb(209, 195, 153)),
        egui::StrokeKind::Inside,
    );
    painter.rect_filled(
        Rect::from_min_max(rect.min, egui::pos2(rect.left() + 5.0, rect.bottom())),
        egui::CornerRadius::ZERO,
        Color32::from_rgb(71, 131, 222),
    );
    let icon = Rect::from_min_size(
        rect.left_top() + egui::vec2(23.0, 30.0),
        egui::vec2(32.0, 26.0),
    );
    painter.rect_filled(
        icon.expand(8.0),
        egui::CornerRadius::same(9),
        Color32::from_rgb(225, 236, 252),
    );
    if title.contains("Runners") {
        painter.circle_filled(icon.center(), 16.0, Color32::from_rgb(71, 146, 61));
        painter.circle_stroke(
            icon.center(),
            16.0,
            Stroke::new(1.5, Color32::from_rgb(37, 96, 36)),
        );
        painter.text(
            icon.center(),
            Align2::CENTER_CENTER,
            "▶",
            FontId::proportional(17.0),
            Color32::WHITE,
        );
    } else if title.contains("Settings") {
        painter.circle_filled(icon.center(), 16.0, Color32::from_rgb(86, 124, 186));
        painter.circle_stroke(
            icon.center(),
            16.0,
            Stroke::new(1.5, Color32::from_rgb(34, 77, 144)),
        );
        painter.circle_stroke(icon.center(), 7.0, Stroke::new(3.0, Color32::WHITE));
        painter.circle_filled(icon.center(), 2.0, Color32::from_rgb(51, 91, 163));
    } else {
        painter.rect_filled(
            Rect::from_min_size(
                icon.left_top() + egui::vec2(2.0, -6.0),
                egui::vec2(16.0, 10.0),
            ),
            egui::CornerRadius::same(3),
            Color32::from_rgb(233, 164, 30),
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
    }
    painter.text(
        rect.left_top() + egui::vec2(79.0, 18.0),
        Align2::LEFT_TOP,
        title,
        FontId::proportional(25.0),
        NAV_BLUE,
    );
    painter.text(
        rect.left_top() + egui::vec2(80.0, 55.0),
        Align2::LEFT_TOP,
        subtitle,
        FontId::proportional(13.0),
        MUTED,
    );
}

/// Explorer-style category divider shared by the application pages.
pub fn section_heading(ui: &mut egui::Ui, title: &str, detail: &str) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        let (mark, _) = ui.allocate_exact_size(egui::vec2(6.0, 35.0), Sense::hover());
        ui.painter().rect_filled(
            mark,
            egui::CornerRadius::same(2),
            Color32::from_rgb(51, 119, 220),
        );
        ui.add_space(5.0);
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(17.0).strong().color(NAV_BLUE));
            ui.label(RichText::new(detail).size(12.0).color(MUTED));
        });
    });
    ui.add_space(6.0);
}

/// Visual container only: callers retain ownership of all actions and state.
pub fn inset_panel(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new()
        .fill(Color32::from_rgb(249, 251, 255))
        .stroke(Stroke::new(1.0, HAIRLINE))
        .corner_radius(egui::CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(15, 12))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui);
        });
}

pub fn diagnostic_row(ui: &mut egui::Ui, line: &str) {
    let missing = line.contains("unavailable") || line.contains("not detected");
    let dot = if missing {
        Color32::from_rgb(174, 101, 34)
    } else {
        Color32::from_rgb(52, 133, 48)
    };
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(13.0, 20.0), Sense::hover());
        ui.painter().circle_filled(rect.center(), 4.0, dot);
        ui.label(RichText::new(line).size(13.0).color(INK));
    });
}

pub fn nav_button(ui: &mut egui::Ui, label: &str, selected: bool) -> bool {
    let (fill, text, stroke) = if selected {
        (
            Color32::from_rgb(30, 79, 172),
            Color32::WHITE,
            Color32::from_rgb(15, 57, 145),
        )
    } else {
        (
            Color32::from_rgb(248, 250, 255),
            NAV_BLUE,
            Color32::from_rgb(248, 250, 255),
        )
    };
    ui.add_sized(
        [ui.available_width() - 12.0, 38.0],
        egui::Button::new(RichText::new(format!("  {label}")).size(14.0).color(text))
            .fill(fill)
            .stroke(Stroke::new(1.0, stroke)),
    )
    .clicked()
}

/// Draws an Explorer-style program list even before the first installation.
/// The empty state exposes the real install/import actions, never sample data.
pub fn empty_library(ui: &mut egui::Ui, enabled: bool) -> EmptyLibraryAction {
    let mut action = EmptyLibraryAction::None;
    egui::Frame::new()
        .fill(Color32::from_rgb(254, 254, 255))
        .stroke(Stroke::new(1.0, HAIRLINE))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.set_min_height(ui.available_height().max(290.0));
            program_list_header(ui);
            ui.add_space(34.0);
            ui.vertical_centered(|ui| {
                folder_illustration(ui);
                ui.add_space(9.0);
                ui.label(
                    RichText::new("Ready for your first Windows app")
                        .size(22.0)
                        .strong()
                        .color(NAV_BLUE),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Install an EXE or MSI — RiceWine keeps each app in its own Wine environment.")
                        .size(13.5)
                        .color(MUTED),
                );
                ui.add_space(17.0);
                if ui
                    .add_enabled(
                        enabled,
                        egui::Button::new(
                            RichText::new("＋  Install a program")
                                .size(15.0)
                                .color(Color32::WHITE)
                                .strong(),
                        )
                        .fill(Color32::from_rgb(58, 143, 45))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(37, 101, 29)))
                        .min_size(egui::vec2(220.0, 42.0)),
                    )
                    .clicked()
                {
                    action = EmptyLibraryAction::Install;
                }
                ui.add_space(7.0);
                if ui
                    .add_enabled(
                        enabled,
                        egui::Button::new("Import an existing Wine prefix…")
                            .min_size(egui::vec2(220.0, 34.0)),
                    )
                    .clicked()
                {
                    action = EmptyLibraryAction::Import;
                }
                ui.add_space(12.0);
                ui.label(
                    RichText::new("Your library remains available even without Wine installed.")
                        .size(11.0)
                        .color(MUTED),
                );
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
            INK,
        );
    }
}

/// Column header used for populated libraries; columns match the compact list.
pub fn populated_list_header(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 34.0), Sense::hover());
    let painter = ui.painter_at(rect);
    gradient(
        &painter,
        rect,
        Color32::from_rgb(254, 254, 252),
        Color32::from_rgb(226, 232, 243),
    );
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, HAIRLINE),
    );
    painter.text(
        rect.left_center() + egui::vec2(12.0, 0.0),
        Align2::LEFT_CENTER,
        "Program",
        FontId::proportional(13.0),
        INK,
    );
    painter.text(
        rect.right_center() + egui::vec2(-98.0, 0.0),
        Align2::LEFT_CENTER,
        "Runner",
        FontId::proportional(13.0),
        INK,
    );
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
