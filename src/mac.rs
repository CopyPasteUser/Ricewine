//! RiceWine Mojave (2018) UI: Finder-like list and inspector, compact utility
//! toolbar, original vector icons, and System Preferences-style groups.
//! All Linux titlebar/window controls remain compositor-owned.

use eframe::egui::{
    self, Align2, Color32, FontId, Painter, Pos2, Rect, Response, RichText, Sense, Stroke,
};

pub const SIDEBAR: Color32 = Color32::from_rgb(229, 228, 226);
pub const TOOLBAR: Color32 = Color32::from_rgb(235, 234, 232);
pub const CANVAS: Color32 = Color32::WHITE;
pub const INSPECTOR: Color32 = Color32::from_rgb(247, 247, 246);
pub const ACCENT: Color32 = Color32::from_rgb(44, 113, 210);
const TEXT: Color32 = Color32::from_rgb(45, 45, 47);
const SECONDARY: Color32 = Color32::from_rgb(111, 110, 112);
const HAIRLINE: Color32 = Color32::from_rgb(194, 193, 191);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Wine,
    Applications,
    Environments,
    Runners,
    Preferences,
    Install,
    Import,
    Preview,
    Back,
    Search,
}

/// Original icons drawn at vector scale. No SF Symbols, Apple artwork, icon
/// font, remote assets or images from an installed desktop are needed.
pub fn paint_icon(p: &Painter, top_left: Pos2, icon: Icon, size: f32, tint: Color32) {
    let k = size / 20.0;
    let point = |x: f32, y: f32| top_left + egui::vec2(x * k, y * k);
    let block = |x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color32| {
        p.rect_filled(
            Rect::from_min_size(point(x, y), egui::vec2(w * k, h * k)),
            radius * k,
            color,
        );
    };
    let line = |a: (f32, f32), b: (f32, f32), color: Color32, width: f32| {
        p.line_segment(
            [point(a.0, a.1), point(b.0, b.1)],
            Stroke::new(width * k, color),
        );
    };
    match icon {
        Icon::Wine => {
            block(1.0, 1.0, 18.0, 18.0, 4.0, Color32::from_rgb(76, 81, 129));
            block(2.0, 2.0, 16.0, 8.0, 3.0, Color32::from_rgb(140, 167, 218));
            block(4.0, 4.0, 12.0, 1.5, 0.7, Color32::from_rgb(214, 228, 251));
            line((5.0, 7.0), (8.0, 12.0), Color32::WHITE, 1.3);
            line((15.0, 7.0), (12.0, 12.0), Color32::WHITE, 1.3);
            line((5.0, 7.0), (15.0, 7.0), Color32::WHITE, 1.2);
            block(8.0, 11.0, 4.0, 3.0, 1.0, Color32::from_rgb(221, 231, 255));
            block(9.0, 14.0, 2.0, 3.0, 0.0, Color32::WHITE);
            block(6.0, 17.0, 8.0, 1.0, 0.5, Color32::WHITE);
        }
        Icon::Applications => {
            block(2.0, 2.0, 8.0, 8.0, 1.8, tint);
            block(11.0, 2.0, 7.0, 8.0, 1.8, tint);
            block(2.0, 11.0, 8.0, 7.0, 1.8, tint);
            block(11.0, 11.0, 7.0, 7.0, 1.8, tint);
            for (x, y) in [(4.2, 4.2), (13.2, 4.2), (4.2, 13.0), (13.2, 13.0)] {
                block(x, y, 2.3, 2.3, 0.5, Color32::from_rgb(235, 240, 247));
            }
        }
        Icon::Environments => {
            block(2.0, 5.0, 16.0, 12.0, 1.5, tint);
            block(2.0, 3.0, 7.0, 4.5, 1.1, tint);
            block(3.2, 8.0, 13.6, 7.3, 0.9, Color32::from_rgb(249, 249, 246));
            line((6.0, 10.3), (14.3, 10.3), tint, 1.2);
            line((6.0, 13.0), (12.0, 13.0), tint, 1.2);
        }
        Icon::Runners => {
            block(1.0, 3.0, 18.0, 14.0, 2.0, tint);
            block(2.5, 5.0, 15.0, 10.5, 1.0, Color32::from_rgb(239, 241, 242));
            line((5.0, 8.0), (7.7, 10.0), tint, 1.4);
            line((7.7, 10.0), (5.0, 12.0), tint, 1.4);
            line((10.0, 12.3), (14.0, 12.3), tint, 1.2);
        }
        Icon::Preferences => {
            for i in 0..8 {
                let angle = i as f32 * std::f32::consts::TAU / 8.0;
                let (dx, dy) = (angle.cos(), angle.sin());
                line(
                    (10.0 + dx * 5.0, 10.0 + dy * 5.0),
                    (10.0 + dx * 8.0, 10.0 + dy * 8.0),
                    tint,
                    2.6,
                );
            }
            p.circle_filled(point(10.0, 10.0), 5.4 * k, tint);
            p.circle_filled(point(10.0, 10.0), 2.4 * k, Color32::from_rgb(244, 244, 243));
        }
        Icon::Install => {
            block(2.0, 6.0, 15.0, 11.0, 2.0, Color32::from_rgb(122, 139, 162));
            block(3.5, 8.0, 12.0, 7.0, 1.0, Color32::from_rgb(235, 238, 241));
            p.circle_filled(point(15.5, 5.0), 4.3 * k, Color32::from_rgb(70, 161, 88));
            line((12.6, 5.0), (18.4, 5.0), Color32::WHITE, 1.6);
            line((15.5, 2.1), (15.5, 7.9), Color32::WHITE, 1.6);
        }
        Icon::Import => {
            block(2.0, 7.0, 16.0, 10.0, 2.0, Color32::from_rgb(184, 140, 73));
            block(3.0, 9.0, 14.0, 6.7, 1.0, Color32::from_rgb(240, 207, 147));
            line((10.0, 1.5), (10.0, 11.2), tint, 2.0);
            line((6.0, 8.0), (10.0, 12.0), tint, 2.0);
            line((14.0, 8.0), (10.0, 12.0), tint, 2.0);
        }
        Icon::Preview => {
            block(1.0, 2.5, 18.0, 15.0, 2.0, tint);
            block(2.5, 4.0, 7.3, 12.0, 0.7, Color32::from_rgb(244, 246, 247));
            block(11.0, 4.0, 6.5, 12.0, 0.7, Color32::from_rgb(221, 227, 235));
        }
        Icon::Back => {
            line((12.5, 3.5), (6.0, 10.0), tint, 2.2);
            line((6.0, 10.0), (12.5, 16.5), tint, 2.2);
        }
        Icon::Search => {
            p.circle_stroke(point(8.5, 8.5), 5.5 * k, Stroke::new(2.0 * k, tint));
            line((12.6, 12.6), (18.0, 18.0), tint, 2.0);
        }
    }
}

pub fn brand(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(38.0, 38.0), Sense::hover());
        paint_icon(ui.painter(), rect.min, Icon::Wine, 36.0, ACCENT);
        ui.vertical(|ui| {
            ui.label(RichText::new("RiceWine").size(15.0).strong().color(TEXT));
            ui.label(
                RichText::new("Windows on Linux")
                    .size(10.5)
                    .color(SECONDARY),
            );
        });
    });
}

pub fn section_label(ui: &mut egui::Ui, label: &str) {
    ui.add_space(9.0);
    ui.label(
        RichText::new(label)
            .size(10.5)
            .strong()
            .color(Color32::from_rgb(119, 117, 115)),
    );
    ui.add_space(3.0);
}

pub fn navigation(ui: &mut egui::Ui, icon: Icon, label: &str, selected: bool) -> Response {
    let response = ui.add_sized(
        [ui.available_width().max(0.0), 30.0],
        egui::Button::selectable(selected, RichText::new(label).color(Color32::TRANSPARENT))
            .fill(if selected {
                ACCENT
            } else {
                Color32::TRANSPARENT
            })
            .stroke(Stroke::NONE)
            .corner_radius(egui::CornerRadius::same(5)),
    );
    let painter = ui.painter().with_clip_rect(response.rect);
    let color = if selected {
        Color32::WHITE
    } else {
        Color32::from_rgb(119, 117, 118)
    };
    paint_icon(
        &painter,
        response.rect.left_center() + egui::vec2(9.0, -9.0),
        icon,
        18.0,
        color,
    );
    painter.text(
        response.rect.left_center() + egui::vec2(35.0, 0.0),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.5),
        if selected { Color32::WHITE } else { TEXT },
    );
    response.on_hover_text(label)
}

/// A compact Finder utility button with its own pictogram and readable label.
pub fn toolbar_button(ui: &mut egui::Ui, icon: Icon, label: &str, enabled: bool) -> Response {
    let width = if label.is_empty() {
        32.0
    } else {
        30.0 + label.chars().count() as f32 * 6.5
    };
    let response = ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(label).color(Color32::TRANSPARENT))
            .min_size(egui::vec2(width, 29.0))
            .fill(Color32::from_rgb(247, 247, 246))
            .stroke(Stroke::new(1.0, Color32::from_rgb(174, 172, 169)))
            .corner_radius(egui::CornerRadius::same(4)),
    );
    let color = if enabled {
        Color32::from_rgb(86, 87, 88)
    } else {
        Color32::from_rgb(173, 174, 176)
    };
    paint_icon(
        ui.painter(),
        response.rect.left_center() + egui::vec2(6.0, -8.0),
        icon,
        17.0,
        color,
    );
    if !label.is_empty() {
        ui.painter().text(
            response.rect.left_center() + egui::vec2(27.0, 0.0),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(12.0),
            color,
        );
    }
    response.on_hover_text(label)
}

pub fn toolbar_title(ui: &mut egui::Ui, title: &str, description: &str) {
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(title).size(13.5).strong().color(TEXT));
        ui.label(RichText::new(description).size(10.5).color(SECONDARY));
    });
}

pub fn list_header(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 28.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::from_rgb(237, 237, 236));
    painter.line_segment(
        [rect.left_top(), rect.right_top()],
        Stroke::new(1.0, Color32::WHITE),
    );
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, HAIRLINE),
    );
    painter.text(
        rect.left_center() + egui::vec2(37.0, 0.0),
        Align2::LEFT_CENTER,
        "Name",
        FontId::proportional(11.5),
        TEXT,
    );
    painter.text(
        rect.right_center() - egui::vec2(103.0, 0.0),
        Align2::LEFT_CENTER,
        "Status",
        FontId::proportional(11.5),
        TEXT,
    );
    painter.line_segment(
        [
            rect.right_top() - egui::vec2(121.0, -2.0),
            rect.right_bottom() - egui::vec2(121.0, 2.0),
        ],
        Stroke::new(1.0, Color32::from_rgb(209, 209, 208)),
    );
}

/// Full-width Finder rows are left-aligned and subtly striped, never
/// center-justified pill buttons. The selected button still owns keyboard focus.
pub fn program_row(
    ui: &mut egui::Ui,
    name: &str,
    selected: bool,
    width: f32,
    index: usize,
) -> Response {
    let fill = if selected {
        ACCENT
    } else if index % 2 == 0 {
        Color32::WHITE
    } else {
        Color32::from_rgb(245, 247, 249)
    };
    let response = ui.add_sized(
        [width.max(0.0), 31.0],
        egui::Button::selectable(selected, RichText::new(name).color(Color32::TRANSPARENT))
            .fill(fill)
            .stroke(Stroke::NONE)
            .corner_radius(egui::CornerRadius::ZERO),
    );
    let painter = ui.painter().with_clip_rect(response.rect.shrink(1.0));
    paint_icon(
        &painter,
        response.rect.left_center() + egui::vec2(8.0, -10.0),
        Icon::Wine,
        20.0,
        ACCENT,
    );
    painter.text(
        response.rect.left_center() + egui::vec2(36.0, 0.0),
        Align2::LEFT_CENTER,
        name,
        FontId::proportional(12.5),
        if selected { Color32::WHITE } else { TEXT },
    );
    response.on_hover_text(name)
}

pub fn empty_library(ui: &mut egui::Ui, can_install: bool) -> bool {
    let mut install_clicked = false;
    // Plain Finder content area rather than the generic centered SaaS card.
    egui::Frame::new()
        .fill(CANVAS)
        .inner_margin(egui::Margin::ZERO)
        .show(ui, |ui| {
            list_header(ui);
            ui.add_space(33.0);
            ui.vertical_centered(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(56.0, 56.0), Sense::hover());
                paint_icon(ui.painter(), rect.min, Icon::Wine, 56.0, ACCENT);
                ui.add_space(9.0);
                ui.label(
                    RichText::new("No Applications")
                        .size(17.0)
                        .strong()
                        .color(TEXT),
                );
                ui.add_space(5.0);
                ui.label(
                    RichText::new("This library is empty. Add a Windows application to begin.")
                        .size(12.0)
                        .color(SECONDARY),
                );
                ui.add_space(11.0);
                if toolbar_button(ui, Icon::Install, "Add Application…", can_install).clicked() {
                    install_clicked = true;
                }
            });
        });
    install_clicked
}

/// Shown in the optional Finder preview column; program actions below are
/// still the existing guarded Wine operations and confirmations.
pub fn inspector_intro(ui: &mut egui::Ui, title: &str, runtime: &str) {
    ui.vertical_centered(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(64.0, 64.0), Sense::hover());
        paint_icon(ui.painter(), rect.min, Icon::Wine, 62.0, ACCENT);
        ui.add_space(8.0);
        ui.label(RichText::new(title).size(15.0).strong().color(TEXT));
        ui.label(RichText::new(runtime).size(11.0).color(SECONDARY));
    });
    ui.add_space(11.0);
    ui.separator();
    ui.add_space(8.0);
}

pub fn page_heading(ui: &mut egui::Ui, icon: Icon, title: &str, subtitle: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(42.0, 42.0), Sense::hover());
        paint_icon(
            ui.painter(),
            rect.min,
            icon,
            38.0,
            Color32::from_rgb(112, 120, 131),
        );
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(17.0).strong().color(TEXT));
            ui.label(RichText::new(subtitle).size(11.5).color(SECONDARY));
        });
    });
    ui.add_space(9.0);
    ui.separator();
    ui.add_space(9.0);
}

pub fn status_bar(ui: &mut egui::Ui, status: &str) {
    let width = ui.available_width();
    egui::Frame::new()
        .fill(Color32::from_rgb(241, 240, 238))
        .stroke(Stroke::new(1.0, HAIRLINE))
        .inner_margin(egui::Margin::symmetric(12, 5))
        .show(ui, |ui| {
            ui.set_min_width((width - 26.0).max(0.0));
            ui.horizontal(|ui| {
                ui.label(RichText::new(status).size(11.0).color(SECONDARY));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("RiceWine • local library")
                            .size(10.5)
                            .color(SECONDARY),
                    );
                });
            });
        });
}

/// A restrained installer-sheet header; never draw fake traffic lights.
pub fn dialog_intro(ui: &mut egui::Ui, title: &str, description: &str) {
    page_heading(ui, Icon::Install, title, description);
}
