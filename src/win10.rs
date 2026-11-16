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

/// Original, narrow-line navigation glyphs from the Windows 10 Settings era.
#[derive(Clone, Copy)]
enum NavigationGlyph {
    Applications,
    Environments,
    Runners,
    Appearance,
}

fn navigation_glyph(label: &str) -> NavigationGlyph {
    match label {
        "Wine environments" => NavigationGlyph::Environments,
        "Wine & Proton" => NavigationGlyph::Runners,
        "Appearance" => NavigationGlyph::Appearance,
        _ => NavigationGlyph::Applications,
    }
}

fn paint_navigation_glyph(painter: &egui::Painter, rect: Rect, glyph: NavigationGlyph) {
    let stroke = Stroke::new(1.45, TEXT);
    let center = rect.center();
    match glyph {
        NavigationGlyph::Applications => {
            for x in [-5.0_f32, 3.0] {
                for y in [-5.0_f32, 3.0] {
                    let square = Rect::from_min_size(center + Vec2::new(x, y), Vec2::splat(5.5));
                    painter.rect_stroke(square, 0.0, stroke, egui::StrokeKind::Inside);
                }
            }
        }
        NavigationGlyph::Environments => {
            painter.rect_stroke(
                Rect::from_center_size(center, Vec2::new(15.0, 12.0)),
                0.0,
                stroke,
                egui::StrokeKind::Inside,
            );
            painter.line_segment(
                [
                    center + Vec2::new(-7.0, -2.0),
                    center + Vec2::new(7.0, -2.0),
                ],
                stroke,
            );
            painter.line_segment(
                [center + Vec2::new(-3.0, 9.0), center + Vec2::new(3.0, 9.0)],
                stroke,
            );
        }
        NavigationGlyph::Runners => {
            painter.rect_stroke(
                Rect::from_center_size(center, Vec2::new(16.0, 13.0)),
                0.0,
                stroke,
                egui::StrokeKind::Inside,
            );
            painter.line_segment(
                [
                    center + Vec2::new(-5.0, -3.0),
                    center + Vec2::new(-1.5, 0.0),
                ],
                stroke,
            );
            painter.line_segment(
                [center + Vec2::new(-1.5, 0.0), center + Vec2::new(-5.0, 3.0)],
                stroke,
            );
            painter.line_segment(
                [center + Vec2::new(1.0, 3.5), center + Vec2::new(5.0, 3.5)],
                stroke,
            );
        }
        NavigationGlyph::Appearance => {
            painter.circle_stroke(center, 6.8, stroke);
            painter.circle_filled(center, 2.3, TEXT);
            for offset in [
                Vec2::new(-10.0, 0.0),
                Vec2::new(10.0, 0.0),
                Vec2::new(0.0, -10.0),
                Vec2::new(0.0, 10.0),
            ] {
                painter.line_segment([center + offset * 0.83, center + offset], stroke);
            }
        }
    }
}

/// Settings navigation uses left-aligned text, period-appropriate line icons
/// and a blue 3px active rail, with a full-row pointer and keyboard target.
pub fn navigation(ui: &mut egui::Ui, label: &str, selected: bool) -> Response {
    let width = ui.available_width().max(0.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 39.0), Sense::click());
    let painter = ui.painter_at(rect);
    painter.rect_filled(
        rect,
        0.0,
        if selected {
            SELECTED
        } else if response.hovered() {
            HOVER
        } else {
            SIDEBAR
        },
    );
    if selected {
        painter.rect_filled(
            Rect::from_min_size(rect.min, Vec2::new(3.0, rect.height())),
            0.0,
            ACCENT,
        );
    }
    paint_navigation_glyph(
        &painter,
        Rect::from_center_size(rect.left_center() + Vec2::new(24.0, 0.0), Vec2::splat(18.0)),
        navigation_glyph(label),
    );
    painter.text(
        rect.left_center() + Vec2::new(42.0, 0.0),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.0),
        TEXT,
    );
    if response.has_focus() {
        painter.rect_stroke(
            rect.shrink(2.0),
            0.0,
            Stroke::new(1.0, ACCENT),
            egui::StrokeKind::Inside,
        );
    }
    response.on_hover_text(label)
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
    let width = ui.available_width().max(0.0);
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
    if selected {
        painter.rect_filled(
            Rect::from_min_size(rect.min, Vec2::new(3.0, rect.height())),
            0.0,
            ACCENT,
        );
    }
    if response.has_focus() {
        painter.rect_stroke(
            rect.shrink(2.0),
            0.0,
            Stroke::new(1.0, ACCENT),
            egui::StrokeKind::Inside,
        );
    }
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
    response.on_hover_text(format!("{name}\n{runtime} · {status}"))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_icons_follow_distinct_windows_10_categories() {
        assert!(matches!(
            navigation_glyph("Apps & features"),
            NavigationGlyph::Applications
        ));
        assert!(matches!(
            navigation_glyph("Wine environments"),
            NavigationGlyph::Environments
        ));
        assert!(matches!(
            navigation_glyph("Wine & Proton"),
            NavigationGlyph::Runners
        ));
        assert!(matches!(
            navigation_glyph("Appearance"),
            NavigationGlyph::Appearance
        ));
    }

    #[test]
    fn compact_settings_palette_remains_square_and_legible() {
        assert_eq!(ACCENT, Color32::from_rgb(0, 103, 184));
        assert_eq!(CANVAS, Color32::WHITE);
        assert_ne!(TEXT, MUTED);
        assert_ne!(SELECTED, HOVER);
    }
}
