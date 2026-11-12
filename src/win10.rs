//! A Windows 10 (1809–1909)-era Settings-inspired Rice for RiceWine.
//! Only egui application content is styled: Linux window borders remain native.
//! All UI art is drawn with primitives; no Windows fonts, bitmaps or trademarks.

use eframe::egui::{self, Align2, Color32, FontId, Rect, Response, RichText, Sense, Stroke, Vec2};

pub const SIDEBAR: Color32 = Color32::from_rgb(242, 242, 242);
pub const CANVAS: Color32 = Color32::WHITE;
pub const ACCENT: Color32 = Color32::from_rgb(0, 103, 184);
pub const TEXT: Color32 = Color32::from_rgb(33, 33, 33);
pub const MUTED: Color32 = Color32::from_rgb(93, 93, 93);
pub const HAIRLINE: Color32 = Color32::from_rgb(224, 224, 224);
const HOVER: Color32 = Color32::from_rgb(229, 229, 229);
const SELECTED: Color32 = Color32::from_rgb(218, 235, 249);

pub fn sidebar_heading(ui: &mut egui::Ui) {
    ui.label(RichText::new("Settings").size(23.0).color(TEXT));
}

pub fn section_label(ui: &mut egui::Ui, label: &str) {
    ui.add_space(20.0);
    ui.label(RichText::new(label).size(12.0).strong().color(TEXT));
    ui.add_space(8.0);
}

/// Full-width Settings navigation: neutral hover fill + thin blue selection rail.
pub fn navigation(ui: &mut egui::Ui, label: &str, selected: bool) -> Response {
    let response = ui.add_sized(
        [ui.available_width(), 39.0],
        egui::Button::new(RichText::new(label).size(13.0).color(TEXT))
            .fill(if selected { SELECTED } else { SIDEBAR })
            .stroke(Stroke::NONE),
    );
    if selected {
        let rail = Rect::from_min_size(response.rect.min, Vec2::new(3.0, response.rect.height()));
        ui.painter().rect_filled(rail, 0.0, ACCENT);
    }
    response
}

pub fn page_heading(ui: &mut egui::Ui, title: &str) {
    ui.label(RichText::new(title).size(29.0).color(TEXT));
}

pub fn section_heading(ui: &mut egui::Ui, title: &str) {
    ui.add_space(14.0);
    ui.label(RichText::new(title).size(18.0).color(TEXT));
    ui.add_space(8.0);
}

pub fn link_action(ui: &mut egui::Ui, label: &str, enabled: bool) -> Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(label).size(13.0).color(ACCENT))
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE),
    )
}

/// Small rectangular Windows 10 pushbutton, not a large modern SaaS CTA.
pub fn primary_action(ui: &mut egui::Ui, label: &str, enabled: bool) -> Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(label).size(13.0).color(Color32::WHITE))
            .fill(ACCENT)
            .stroke(Stroke::new(1.0, ACCENT)),
    )
}

pub fn secondary_action(ui: &mut egui::Ui, label: &str, enabled: bool) -> Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(label).size(12.5).color(TEXT))
            .fill(Color32::from_rgb(236, 236, 236))
            .stroke(Stroke::new(1.0, Color32::from_rgb(173, 173, 173))),
    )
}

/// Draws a row with two independent text hierarchies and a locally rendered
/// app glyph. Its hit target is a single egui clickable region.
pub fn app_row(
    ui: &mut egui::Ui,
    name: &str,
    runtime: &str,
    status: &str,
    selected: bool,
) -> Response {
    let width = ui.available_width().max(260.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 63.0), Sense::click());
    let painter = ui.painter_at(rect);
    painter.rect_filled(
        rect,
        0.0,
        if selected {
            SELECTED
        } else if response.hovered() {
            HOVER
        } else {
            CANVAS
        },
    );
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, HAIRLINE),
    );

    // Flat 32×32 application glyph with a simple window motif.
    let icon_rect = Rect::from_min_size(rect.min + Vec2::new(12.0, 14.0), Vec2::splat(32.0));
    painter.rect_filled(icon_rect, 0.0, ACCENT);
    painter.rect_stroke(
        Rect::from_min_max(
            icon_rect.min + Vec2::splat(7.0),
            icon_rect.max - Vec2::splat(7.0),
        ),
        0.0,
        Stroke::new(1.6, Color32::WHITE),
        egui::StrokeKind::Inside,
    );
    painter.line_segment(
        [
            icon_rect.min + Vec2::new(7.0, 12.0),
            icon_rect.min + Vec2::new(25.0, 12.0),
        ],
        Stroke::new(1.2, Color32::WHITE),
    );

    let right_column = 96.0;
    let text_clip = Rect::from_min_max(
        rect.min + Vec2::new(57.0, 2.0),
        rect.max - Vec2::new(right_column, 2.0),
    );
    let text_painter = painter.with_clip_rect(text_clip);
    text_painter.text(
        rect.min + Vec2::new(58.0, 15.0),
        Align2::LEFT_TOP,
        name,
        FontId::proportional(14.0),
        TEXT,
    );
    text_painter.text(
        rect.min + Vec2::new(58.0, 37.0),
        Align2::LEFT_TOP,
        runtime,
        FontId::proportional(11.5),
        MUTED,
    );
    painter.text(
        rect.right_center() - Vec2::new(14.0, 0.0),
        Align2::RIGHT_CENTER,
        status,
        FontId::proportional(11.5),
        if status == "Ready" { ACCENT } else { MUTED },
    );
    response.on_hover_text(name)
}

pub fn empty_library(ui: &mut egui::Ui, enabled: bool) -> bool {
    let mut install = false;
    ui.add_space(18.0);
    ui.separator();
    ui.add_space(22.0);
    ui.label(
        RichText::new("No Windows programs are installed yet")
            .size(16.0)
            .color(TEXT),
    );
    ui.add_space(6.0);
    ui.label(
        RichText::new("Choose an EXE or MSI to install it using Wine.")
            .size(13.0)
            .color(MUTED),
    );
    ui.add_space(16.0);
    if primary_action(ui, "Install a program", enabled).clicked() {
        install = true;
    }
    ui.add_space(8.0);
    ui.label(
        RichText::new("Existing Wine prefix? Use Import existing prefix.")
            .size(11.5)
            .color(MUTED),
    );
    install
}

