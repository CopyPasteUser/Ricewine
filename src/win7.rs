//! Windows 7 Aero-era Control Panel / Programs and Features UI.
//! Original vector glyphs and native egui painting; the Linux compositor
//! owns window decorations. Wine actions remain in RiceWineApp.

use eframe::egui::{self, Align2, Color32, FontId, Rect, Response, RichText, Sense, Stroke, Vec2};

pub const TEXT: Color32 = Color32::from_rgb(27, 43, 62);
pub const MUTED: Color32 = Color32::from_rgb(86, 106, 125);
pub const LINK: Color32 = Color32::from_rgb(27, 88, 153);
pub const BORDER: Color32 = Color32::from_rgb(163, 194, 214);
pub const BACKDROP: Color32 = Color32::from_rgb(242, 248, 253);
pub const SIDE: Color32 = Color32::from_rgb(235, 244, 251);
pub const WHITE: Color32 = Color32::WHITE;
pub const SELECT: Color32 = Color32::from_rgb(206, 232, 247);
pub const SELECT_BORDER: Color32 = Color32::from_rgb(107, 174, 207);

pub fn shell_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(BACKDROP)
        .inner_margin(egui::Margin::same(0))
}

pub fn explorer_bar(
    ui: &mut egui::Ui,
    page: &str,
    search: &mut String,
    show_search: bool,
) -> Option<Response> {
    let mut result = None;
    egui::Frame::new()
        .fill(Color32::from_rgb(223, 238, 247))
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("◀  ▶").color(LINK).size(12.0));
                ui.separator();
                ui.label(RichText::new("Control Panel").color(LINK).size(12.0));
                ui.label(RichText::new("›").color(MUTED));
                ui.label(RichText::new("Programs").color(LINK).size(12.0));
                if ui.available_width() > 310.0 {
                    ui.label(RichText::new("›").color(MUTED));
                    ui.label(RichText::new(page).size(12.0).color(TEXT));
                }
                if show_search && ui.available_width() >= 160.0 {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        result = Some(
                            ui.add(
                                egui::TextEdit::singleline(search)
                                    .hint_text("Search programs")
                                    .desired_width(ui.available_width().min(170.0)),
                            ),
                        );
                    });
                }
            });
        });
    result.or_else(|| {
        show_search.then(|| {
            ui.add(
                egui::TextEdit::singleline(search)
                    .hint_text("Search programs")
                    .desired_width(ui.available_width().min(220.0)),
            )
        })
    })
}

pub fn content_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(WHITE)
        .inner_margin(egui::Margin::symmetric(12, 10))
}

pub fn task_pane() -> egui::Frame {
    egui::Frame::new()
        .fill(SIDE)
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(egui::Margin::symmetric(11, 14))
}

pub fn task_heading(ui: &mut egui::Ui, label: &str) {
    ui.label(RichText::new(label).size(12.0).strong().color(TEXT));
    ui.add_space(4.0);
}

pub fn task_link(ui: &mut egui::Ui, label: &str, selected: bool) -> Response {
    ui.add_sized(
        [ui.available_width().max(0.0), 25.0],
        egui::Button::new(RichText::new(label).size(12.0).color(LINK))
            .fill(if selected {
                Color32::from_rgb(215, 235, 248)
            } else {
                Color32::TRANSPARENT
            })
            .stroke(if selected {
                Stroke::new(1.0, SELECT_BORDER)
            } else {
                Stroke::NONE
            })
            .corner_radius(egui::CornerRadius::same(2)),
    )
}

pub fn page_heading(ui: &mut egui::Ui, title: &str, description: &str, compact: bool) {
    ui.label(
        RichText::new(title)
            .size(if compact { 16.0 } else { 21.0 })
            .color(Color32::from_rgb(19, 64, 113)),
    );
    if !compact {
        ui.add_space(3.0);
        ui.label(RichText::new(description).size(12.0).color(TEXT));
    }
}

pub fn command_strip() -> egui::Frame {
    egui::Frame::new()
        .fill(Color32::from_rgb(246, 250, 252))
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(egui::Margin::symmetric(6, 3))
}

pub fn command(ui: &mut egui::Ui, label: &str, enabled: bool) -> Response {
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(label).size(12.0).color(TEXT))
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE)
            .corner_radius(egui::CornerRadius::same(2))
            .min_size(Vec2::new(0.0, 23.0)),
    )
}

/// Explorer details view: name, active runtime and readiness, with columns
/// adapting to narrow windows instead of squeezing into unreadable fragments.
pub fn table_header(ui: &mut egui::Ui) {
    let width = ui.available_width().max(0.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 26.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::from_rgb(239, 247, 252));
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0, BORDER),
    );
    let (name_end, runtime_end) = columns(width);
    draw_text(
        &painter,
        Rect::from_min_max(rect.min, egui::pos2(rect.left() + name_end, rect.bottom())),
        "Name",
        10.0,
        TEXT,
        12.0,
    );
    if runtime_end > name_end {
        draw_text(
            &painter,
            Rect::from_min_max(
                egui::pos2(rect.left() + name_end, rect.top()),
                egui::pos2(rect.left() + runtime_end, rect.bottom()),
            ),
            "Runner",
            8.0,
            TEXT,
            12.0,
        );
    }
    draw_text(
        &painter,
        Rect::from_min_max(egui::pos2(rect.left() + runtime_end, rect.top()), rect.max),
        "Status",
        8.0,
        TEXT,
        12.0,
    );
    for x in [name_end, runtime_end] {
        if x < width - 1.0 {
            painter.line_segment(
                [
                    egui::pos2(rect.left() + x, rect.top() + 4.0),
                    egui::pos2(rect.left() + x, rect.bottom() - 4.0),
                ],
                Stroke::new(1.0, BORDER),
            );
        }
    }
}

/// Return (name end, runner end); on narrow screens omit runtime to prioritize
/// the actual program name and its state.
fn columns(width: f32) -> (f32, f32) {
    if width < 340.0 {
        ((width - 85.0).max(100.0), (width - 85.0).max(100.0))
    } else if width < 520.0 {
        ((width * 0.56).floor(), (width * 0.77).floor())
    } else {
        ((width * 0.54).floor(), (width * 0.78).floor())
    }
}

fn draw_text(
    painter: &egui::Painter,
    rect: Rect,
    value: &str,
    indent: f32,
    color: Color32,
    size: f32,
) {
    if rect.width() <= indent || rect.height() <= 0.0 {
        return;
    }
    painter
        .with_clip_rect(rect.shrink2(Vec2::new(1.0, 0.0)))
        .text(
            egui::pos2(rect.left() + indent, rect.center().y),
            Align2::LEFT_CENTER,
            value,
            FontId::proportional(size),
            color,
        );
}

pub fn program_row(
    ui: &mut egui::Ui,
    name: &str,
    runner: &str,
    status: &str,
    selected: bool,
    striped: bool,
) -> Response {
    let width = ui.available_width().max(0.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, 31.0), Sense::click());
    let painter = ui.painter_at(rect);
    let fill = if selected {
        SELECT
    } else if response.hovered() {
        Color32::from_rgb(232, 244, 251)
    } else if striped {
        Color32::from_rgb(250, 253, 255)
    } else {
        WHITE
    };
    painter.rect_filled(rect, egui::CornerRadius::same(1), fill);
    if selected || response.hovered() {
        painter.rect_stroke(
            rect.shrink(0.5),
            egui::CornerRadius::same(1),
            Stroke::new(1.0, SELECT_BORDER),
            egui::StrokeKind::Inside,
        );
    }
    let (name_end, runtime_end) = columns(width);
    let icon = Rect::from_center_size(
        egui::pos2(rect.left() + 16.0, rect.center().y),
        Vec2::splat(19.0),
    );
    crate::rice_identity::paint(
        &painter,
        icon,
        crate::rice_identity::Family::Aero7,
        crate::rice_identity::Icon::Programs,
        LINK,
    );
    draw_text(
        &painter,
        Rect::from_min_max(rect.min, egui::pos2(rect.left() + name_end, rect.bottom())),
        name,
        32.0,
        TEXT,
        12.0,
    );
    if runtime_end > name_end {
        draw_text(
            &painter,
            Rect::from_min_max(
                egui::pos2(rect.left() + name_end, rect.top()),
                egui::pos2(rect.left() + runtime_end, rect.bottom()),
            ),
            runner,
            8.0,
            MUTED,
            11.5,
        );
    }
    draw_text(
        &painter,
        Rect::from_min_max(egui::pos2(rect.left() + runtime_end, rect.top()), rect.max),
        status,
        8.0,
        TEXT,
        11.5,
    );
    if response.has_focus() {
        painter.rect_stroke(
            rect.shrink(2.0),
            egui::CornerRadius::same(1),
            Stroke::new(1.0, LINK),
            egui::StrokeKind::Inside,
        );
    }
    response.widget_info(|| {
        let mut info = egui::WidgetInfo::labeled(
            egui::Role::Button,
            ui.is_enabled(),
            format!("{name}, {runner}, {status}"),
        );
        info.selected = Some(selected);
        info
    });
    response.on_hover_text(format!("{name} • {runner} • {status}"))
}

pub fn status_bar(ui: &mut egui::Ui, count: usize, selected: Option<&str>, compact: bool) {
    let full_width = ui.available_width().max(0.0);
    egui::Frame::new()
        .fill(Color32::from_rgb(223, 239, 251))
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(egui::Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.set_min_width((full_width - 22.0).max(0.0));
            if compact {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("{count} registered"))
                            .size(11.0)
                            .strong()
                            .color(TEXT),
                    );
                    ui.separator();
                    ui.label(
                        RichText::new("Wine prefixes preserved")
                            .size(11.0)
                            .color(MUTED),
                    );
                });
            } else {
                ui.horizontal(|ui| {
                    crate::rice_identity::symbol(
                        ui,
                        crate::rice_identity::Family::Aero7,
                        crate::rice_identity::Icon::Programs,
                        LINK,
                        23.0,
                    );
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(selected.unwrap_or("Currently installed programs"))
                                .size(12.0)
                                .strong()
                                .color(TEXT),
                        );
                        ui.label(
                            RichText::new(format!(
                                "{count} registered  •  Wine prefixes are not removed"
                            ))
                            .size(11.0)
                            .color(MUTED),
                        );
                    });
                });
            }
        });
}

pub fn empty_library(ui: &mut egui::Ui) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        crate::rice_identity::symbol(
            ui,
            crate::rice_identity::Family::Aero7,
            crate::rice_identity::Icon::Programs,
            LINK,
            32.0,
        );
        ui.vertical(|ui| {
            ui.label(
                RichText::new("No programs are currently registered")
                    .size(13.0)
                    .color(TEXT),
            );
            ui.label(
                RichText::new("Choose Install a program or Import existing above.")
                    .size(12.0)
                    .color(MUTED),
            );
        });
    });
}

pub fn dialog_intro(ui: &mut egui::Ui, title: &str, description: &str) {
    ui.label(RichText::new(title).size(18.0).color(LINK));
    ui.label(RichText::new(description).size(12.0).color(MUTED));
    ui.separator();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn details_columns_preserve_minimum_name_and_status_space() {
        for width in [320.0, 460.0, 680.0] {
            let (name_end, runner_end) = columns(width);
            assert!(name_end >= 100.0);
            assert!(runner_end >= name_end);
            assert!(runner_end < width);
            assert!(width - runner_end >= 60.0);
        }
        let (n, r) = columns(320.0);
        assert_eq!(n, r, "runtime column must collapse on small windows");
    }

    #[test]
    fn explorer_text_colors_are_legible() {
        assert_ne!(TEXT, WHITE);
        assert_ne!(LINK, BACKDROP);
        assert_ne!(SELECT, WHITE);
    }
}
