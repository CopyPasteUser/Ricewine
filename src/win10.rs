//! Windows 10-inspired RiceWine application interior.
//! Native title bars and window controls remain owned by the compositor.
//! All visuals are drawn with egui; no Microsoft assets or font files.

use eframe::egui::{self, Color32, RichText};

pub const SIDEBAR: Color32 = Color32::from_rgb(242, 242, 242);
pub const TOOLBAR: Color32 = Color32::from_rgb(255, 255, 255);
pub const CANVAS: Color32 = Color32::from_rgb(255, 255, 255);
const SELECTED: Color32 = Color32::from_rgb(224, 239, 253);
const BORDER: Color32 = Color32::from_rgb(210, 210, 210);
const TEXT: Color32 = Color32::from_rgb(30, 30, 30);
const MUTED: Color32 = Color32::from_rgb(83, 83, 83);
const BLUE: Color32 = Color32::from_rgb(0, 103, 184);

pub fn brand(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        // Simple four-tile glyph, independently drawn rather than bundled artwork.
        let (rect, _) = ui.allocate_exact_size(egui::vec2(26.0, 26.0), egui::Sense::hover());
        let w = 11.0;
        let space = 3.0;
        for row in 0..2 {
            for col in 0..2 {
                let origin = rect.min + egui::vec2(col as f32 * (w + space), row as f32 * (w + space));
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(origin, egui::vec2(w, w)),
                    0.0,
                    BLUE,
                );
            }
        }
        ui.add_space(6.0);
        ui.vertical(|ui| {
            ui.label(RichText::new("RiceWine").size(19.0).strong().color(TEXT));
            ui.label(RichText::new("Windows apps on Linux").size(11.0).color(MUTED));
        });
    });
}

pub fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(14.0);
    ui.label(RichText::new(title.to_uppercase()).size(10.0).strong().color(MUTED));
    ui.add_space(6.0);
}

pub fn navigation(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
    ui.add_sized(
        [ui.available_width(), 39.0],
        egui::Button::selectable(selected, RichText::new(label).size(13.5).color(TEXT))
            .fill(if selected { SELECTED } else { SIDEBAR })
            .stroke(if selected {
                egui::Stroke::new(1.0, BLUE)
            } else {
                egui::Stroke::NONE
            }),
    )
}

pub fn heading(ui: &mut egui::Ui, title: &str, description: &str) {
    ui.vertical(|ui| {
        ui.label(RichText::new(title).size(23.0).color(TEXT));
        ui.label(RichText::new(description).size(12.0).color(MUTED));
    });
}

pub fn primary_action(ui: &mut egui::Ui, label: &str, enabled: bool) -> egui::Response {
    let button = egui::Button::new(
        RichText::new(label)
            .size(13.0)
            .color(if enabled { Color32::WHITE } else { MUTED }),
    )
    .fill(if enabled { BLUE } else { SIDEBAR })
    .stroke(egui::Stroke::NONE);
    ui.add_enabled(enabled, button)
}

pub fn list_header(ui: &mut egui::Ui) {
    egui::Frame::new()
        .fill(SIDEBAR)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .inner_margin(egui::Margin::symmetric(12, 9))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("NAME").size(10.5).strong().color(MUTED));
                ui.add_space(100.0);
                ui.label(RichText::new("STATUS").size(10.5).strong().color(MUTED));
            });
        });
}

pub fn program_row(ui: &mut egui::Ui, name: &str, selected: bool, width: f32) -> egui::Response {
    ui.add_sized(
        [width, 39.0],
        egui::Button::selectable(selected, RichText::new(format!("▣   {name}")).size(13.0))
            .fill(if selected { SELECTED } else { Color32::WHITE }),
    )
}

pub fn empty_library(ui: &mut egui::Ui, can_install: bool) -> bool {
    let mut clicked = false;
    ui.add_space(42.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new("Your Windows apps").size(25.0).color(TEXT));
        ui.add_space(10.0);
        ui.label(
            RichText::new("Install an EXE or MSI to keep your programs in one place.")
                .size(13.0)
                .color(MUTED),
        );
        ui.add_space(18.0);
        if primary_action(ui, "＋  Install a program", can_install).clicked() {
            clicked = true;
        }
        ui.add_space(10.0);
        ui.label(
            RichText::new("Already have a Wine prefix? Choose More → Import.")
                .size(11.0)
                .color(MUTED),
        );
    });
    clicked
}

pub fn status_bar(ui: &mut egui::Ui, status: &str) {
    egui::Frame::new()
        .fill(SIDEBAR)
        .stroke(egui::Stroke::new(1.0, BORDER))
        .inner_margin(egui::Margin::symmetric(16, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(status).size(11.0).color(MUTED));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("RiceWine · Windows 10").size(11.0).color(MUTED));
                });
            });
        });
}
