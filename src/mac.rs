//! macOS-inspired RiceWine surfaces. The compositor retains the native titlebar
//! and window controls; these widgets style only the application interior.

use eframe::egui::{self, Color32, RichText};

pub const SIDEBAR: Color32 = Color32::from_rgb(233, 236, 242);
pub const TOOLBAR: Color32 = Color32::from_rgb(247, 248, 250);
pub const CANVAS: Color32 = Color32::from_rgb(250, 251, 253);
const ACCENT: Color32 = Color32::from_rgb(0, 96, 198);
const TEXT: Color32 = Color32::from_rgb(30, 36, 44);
const SECONDARY: Color32 = Color32::from_rgb(88, 97, 110);
const HAIRLINE: Color32 = Color32::from_rgb(211, 216, 224);

pub fn brand(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        egui::Frame::new()
            .fill(ACCENT)
            .corner_radius(egui::CornerRadius::same(9))
            .inner_margin(egui::Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.label(RichText::new("W").size(19.0).strong().color(Color32::WHITE));
            });
        ui.add_space(5.0);
        ui.vertical(|ui| {
            ui.label(RichText::new("RiceWine").size(17.0).strong().color(TEXT));
            ui.label(
                RichText::new("Your Windows apps")
                    .size(10.5)
                    .color(SECONDARY),
            );
        });
    });
}

pub fn section_label(ui: &mut egui::Ui, label: &str) {
    ui.add_space(7.0);
    ui.label(RichText::new(label).size(10.0).strong().color(SECONDARY));
    ui.add_space(4.0);
}

pub fn navigation(ui: &mut egui::Ui, symbol: &str, label: &str, selected: bool) -> egui::Response {
    ui.add_sized(
        [ui.available_width(), 35.0],
        egui::Button::selectable(
            selected,
            RichText::new(format!("{symbol}   {label}"))
                .size(13.0)
                .strong()
                .color(if selected { Color32::WHITE } else { TEXT }),
        ),
    )
}

pub fn toolbar_title(ui: &mut egui::Ui, title: &str, description: &str) {
    ui.vertical(|ui| {
        ui.label(RichText::new(title).size(18.0).strong().color(TEXT));
        ui.label(RichText::new(description).size(11.0).color(SECONDARY));
    });
}

pub fn list_header(ui: &mut egui::Ui) {
    egui::Frame::new()
        .fill(Color32::from_rgb(240, 242, 246))
        .stroke(egui::Stroke::new(1.0, HAIRLINE))
        .inner_margin(egui::Margin::symmetric(12, 8))
        .corner_radius(egui::CornerRadius::same(6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("APPLICATION")
                        .size(10.0)
                        .strong()
                        .color(SECONDARY),
                );
                ui.add_space(75.0);
                ui.label(RichText::new("STATUS").size(10.0).strong().color(SECONDARY));
            });
        });
}

pub fn program_row(ui: &mut egui::Ui, name: &str, selected: bool, width: f32) -> egui::Response {
    ui.add_sized(
        [width, 40.0],
        egui::Button::selectable(selected, RichText::new(format!("▣  {name}")).size(13.0)),
    )
}

pub fn empty_library(ui: &mut egui::Ui, can_install: bool) -> bool {
    let mut install_clicked = false;
    ui.add_space(30.0);
    egui::Frame::new()
        .fill(Color32::WHITE)
        .stroke(egui::Stroke::new(1.0, HAIRLINE))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::same(24))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("▣").size(42.0).color(ACCENT));
                ui.add_space(6.0);
                ui.label(
                    RichText::new("No applications yet")
                        .size(18.0)
                        .strong()
                        .color(TEXT),
                );
                ui.label(
                    RichText::new("Install a Windows program or import an existing Wine prefix.")
                        .color(SECONDARY),
                );
                ui.add_space(14.0);
                if ui
                    .add_enabled(
                        can_install,
                        egui::Button::new("＋ Install your first program"),
                    )
                    .clicked()
                {
                    install_clicked = true;
                }
                ui.add_space(6.0);
                ui.small("Already have a Wine prefix? Use More → Import existing prefix.");
            });
        });
    install_clicked
}

pub fn status_bar(ui: &mut egui::Ui, status: &str) {
    egui::Frame::new()
        .fill(TOOLBAR)
        .stroke(egui::Stroke::new(1.0, HAIRLINE))
        .inner_margin(egui::Margin::symmetric(16, 5))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(status).size(11.0).color(SECONDARY));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("RiceWine").size(11.0).color(SECONDARY));
                });
            });
        });
}
