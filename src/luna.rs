//! XP Luna Blue application surfaces. Window decorations belong to the compositor.

pub use crate::luna_icons::XpIcon;
use crate::luna_icons::paint_icon;

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
    let full_width = ui.available_width();
    egui::Frame::new()
        .fill(Color32::from_rgb(242, 240, 235))
        .stroke(Stroke::new(1.0, Color32::from_rgb(178, 179, 175)))
        .inner_margin(egui::Margin::symmetric(8, 2))
        .show(ui, |ui| {
            ui.set_min_width((full_width - 18.0).max(0.0));
            ui.scope(|ui| {
                // Old Windows menu bars use plain labels, not framed buttons.
                ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
                ui.style_mut().visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
                ui.style_mut().visuals.widgets.inactive.bg_stroke = Stroke::NONE;
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
        });
    action
}

/// Explorer-era menu and location strip, below the real Linux window titlebar.
/// On compact windows the breadcrumb wraps instead of forcing horizontal scroll.
pub fn application_toolbar(ui: &mut egui::Ui, section: &str) -> MenuAction {
    let mut action = MenuAction::None;
    let full_width = ui.available_width();
    egui::Frame::new()
        .fill(Color32::from_rgb(238, 237, 230))
        .stroke(Stroke::new(1.0, Color32::from_rgb(178, 177, 160)))
        .inner_margin(egui::Margin::symmetric(8, 5))
        .show(ui, |ui| {
            ui.set_min_width((full_width - 18.0).max(0.0));
            ui.horizontal_wrapped(|ui| {
                let back = ui.add_enabled(
                    section != "Library",
                    egui::Button::new("      Back").min_size(egui::vec2(85.0, 26.0)),
                );
                paint_icon(
                    ui.painter(),
                    back.rect.left_center() + egui::vec2(8.0, -9.0),
                    XpIcon::Back,
                    18.0,
                );
                if back.clicked() {
                    action = MenuAction::Library;
                }
                let home = ui.add(egui::Button::new("      Home").min_size(egui::vec2(88.0, 26.0)));
                paint_icon(
                    ui.painter(),
                    home.rect.left_center() + egui::vec2(8.0, -9.0),
                    XpIcon::Home,
                    18.0,
                );
                if home.clicked() {
                    action = MenuAction::Library;
                }
                ui.separator();
                ui.menu_button("Go to ▾", |ui| {
                    for (label, destination) in [
                        ("Installed programs", MenuAction::Library),
                        ("Wine prefixes", MenuAction::Prefixes),
                        ("Wine & Proton", MenuAction::Runners),
                        ("Preferences", MenuAction::Settings),
                    ] {
                        if ui.button(label).clicked() {
                            action = destination;
                            ui.close();
                        }
                    }
                });
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new("Address")
                        .size(11.5)
                        .color(Color32::from_rgb(66, 64, 58)),
                );
                egui::Frame::new()
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(1.0, Color32::from_rgb(136, 150, 169)))
                    .inner_margin(egui::Margin::symmetric(7, 3))
                    .show(ui, |ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(16.0, 16.0), Sense::hover());
                        paint_icon(ui.painter(), rect.min, XpIcon::Programs, 16.0);
                        ui.label(
                            RichText::new(format!("Control Panel  ›  RiceWine  ›  {section}"))
                                .size(11.5)
                                .color(Color32::from_rgb(43, 75, 138)),
                        );
                    });
            });
        });
    action
}

/// Reusable XP action control. Other Rices retain native egui controls.
pub fn action(ui: &mut egui::Ui, label: &str, enabled: bool, luna: bool) -> egui::Response {
    if luna {
        xp_button(ui, label, enabled)
    } else {
        ui.add_enabled(enabled, egui::Button::new(label))
    }
}

/// Control Panel-style section heading with original era-inspired icon.
pub fn section_heading(ui: &mut egui::Ui, title: &str, icon: XpIcon) {
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(20.0, 20.0), Sense::hover());
        paint_icon(ui.painter(), rect.min, icon, 18.0);
        ui.label(RichText::new(title).size(13.0).strong().color(NAV_BLUE));
    });
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 2.0), Sense::hover());
    ui.painter().line_segment(
        [rect.left_top(), rect.right_top()],
        Stroke::new(1.0, Color32::from_rgb(147, 176, 221)),
    );
    ui.add_space(3.0);
}

/// Classic system notification, keeps an accessible real dismiss control.
pub fn notice(ui: &mut egui::Ui, error: bool, message: &str) -> bool {
    let mut close = false;
    let fill = if error {
        Color32::from_rgb(255, 248, 224)
    } else {
        Color32::from_rgb(237, 247, 255)
    };
    egui::Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, Color32::from_rgb(152, 182, 219)))
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(19.0, 19.0), Sense::hover());
                paint_icon(
                    ui.painter(),
                    rect.min,
                    if error { XpIcon::Warning } else { XpIcon::Info },
                    18.0,
                );
                ui.label(
                    RichText::new(message)
                        .size(12.0)
                        .color(Color32::from_rgb(47, 58, 77)),
                );
                close = ui
                    .small_button("×")
                    .on_hover_text("Dismiss notification")
                    .clicked();
            });
        });
    close
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

/// XP-branded applet plaque, inside the application rather than a fake OS titlebar.
pub fn brand(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().max(0.0), 58.0),
        Sense::hover(),
    );
    let painter = ui.painter_at(rect);
    gradient(
        &painter,
        rect,
        Color32::from_rgb(73, 132, 218),
        Color32::from_rgb(21, 69, 163),
    );
    painter.line_segment(
        [
            rect.left_top() + egui::vec2(1.0, 1.0),
            rect.right_top() + egui::vec2(-1.0, 1.0),
        ],
        Stroke::new(1.0, Color32::from_rgb(170, 211, 255)),
    );
    paint_icon(
        &painter,
        rect.left_top() + egui::vec2(10.0, 12.0),
        XpIcon::Wine,
        34.0,
    );
    painter.text(
        rect.left_top() + egui::vec2(52.0, 10.0),
        Align2::LEFT_TOP,
        "RiceWine",
        FontId::proportional(20.0),
        Color32::WHITE,
    );
    painter.text(
        rect.left_top() + egui::vec2(53.0, 38.0),
        Align2::LEFT_TOP,
        "WINDOWS COMPATIBILITY",
        FontId::proportional(9.0),
        Color32::from_rgb(222, 238, 255),
    );
}

/// An XP Control Panel-style rail. Navigation buttons keep focus and keyboard
/// activation; the artwork itself does not intercept pointer input.
pub fn applet_nav(ui: &mut egui::Ui, label: &str, icon: XpIcon, selected: bool) -> egui::Response {
    let response = ui.add_sized(
        [ui.available_width().max(0.0), 46.0],
        egui::Button::new(
            RichText::new(format!("       {label}"))
                .size(12.0)
                .color(if selected {
                    Color32::from_rgb(16, 56, 137)
                } else {
                    Color32::from_rgb(48, 56, 68)
                })
                .strong(),
        )
        .fill(if selected {
            Color32::WHITE
        } else {
            Color32::from_rgb(238, 236, 223)
        })
        .stroke(Stroke::new(
            1.0,
            if selected {
                Color32::from_rgb(115, 153, 218)
            } else {
                Color32::from_rgb(207, 204, 187)
            },
        ))
        .corner_radius(egui::CornerRadius::same(2)),
    );
    paint_icon(
        ui.painter(),
        response.rect.left_center() + egui::vec2(8.0, -14.0),
        icon,
        28.0,
    );
    if selected {
        let r = response.rect;
        ui.painter().line_segment(
            [
                r.right_top() + egui::vec2(-2.0, 2.0),
                r.right_bottom() + egui::vec2(-2.0, -2.0),
            ],
            Stroke::new(3.0, Color32::from_rgb(44, 102, 203)),
        );
    }
    response
}

pub fn rail_heading(ui: &mut egui::Ui, title: &str) {
    ui.label(
        RichText::new(title)
            .size(10.0)
            .strong()
            .color(Color32::from_rgb(85, 83, 73)),
    );
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().max(0.0), 2.0),
        Sense::hover(),
    );
    ui.painter().line_segment(
        [rect.left_top(), rect.right_top()],
        Stroke::new(1.0, Color32::from_rgb(184, 181, 163)),
    );
    ui.add_space(5.0);
}

/// A period-correct blue selected-program strip above the applet's program list.
/// The actual program actions remain in the safe, reviewable details section.
pub fn programs_bar(ui: &mut egui::Ui, count: usize, selected_name: Option<&str>) {
    let selected = selected_name.is_some();
    let full_width = ui.available_width();
    egui::Frame::new()
        .fill(if selected {
            Color32::from_rgb(51, 104, 194)
        } else {
            Color32::from_rgb(246, 244, 232)
        })
        .stroke(Stroke::new(
            1.0,
            if selected {
                Color32::from_rgb(25, 76, 160)
            } else {
                Color32::from_rgb(174, 180, 185)
            },
        ))
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.set_min_width((full_width - 26.0).max(0.0));
            if let Some(name) = selected_name {
                ui.label(
                    RichText::new(name)
                        .size(14.0)
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.label(
                    RichText::new("Selected program · Review actions and compatibility below")
                        .size(11.0)
                        .color(Color32::from_rgb(233, 241, 255)),
                );
            } else {
                ui.label(
                    RichText::new(format!("Currently installed programs: {count}"))
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(49, 58, 75)),
                );
                ui.small("Select a program to see available actions.");
            }
        });
    ui.add_space(6.0);
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
            [
                r.left_top() + egui::vec2(2.0, 1.0),
                r.right_top() - egui::vec2(2.0, -1.0),
            ],
            Stroke::new(1.0, light),
        );
        ui.painter().line_segment(
            [
                r.left_bottom() + egui::vec2(2.0, -1.0),
                r.right_bottom() - egui::vec2(2.0, 1.0),
            ],
            Stroke::new(1.0, dark),
        );
    }
    response
}

/// XP-like bordered option group. Unlike a card, this follows the traditional
/// Control Panel fieldset hierarchy and leaves all children interactive.
pub fn option_group(ui: &mut egui::Ui, title: &str, icon: XpIcon, add: impl FnOnce(&mut egui::Ui)) {
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
        "Program Library" | "Add or Remove Programs" => XpIcon::Programs,
        "Wine Prefixes" => XpIcon::Prefix,
        "Compatibility Runners" => XpIcon::Runner,
        _ => XpIcon::Settings,
    };
    paint_icon(
        &painter,
        rect.left_top() + egui::vec2(15.0, 16.0),
        icon,
        33.0,
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

/// Explorer details pane empty state. The actionable install/import links live
/// in the task pane and File menu; the list itself stays visually quiet.
pub fn empty_library(ui: &mut egui::Ui) {
    egui::Frame::new()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0, Color32::from_rgb(146, 164, 183)))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.set_min_height(ui.available_height().max(190.0));
            program_list_header(ui);
            ui.add_space(24.0);
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                let (rect, _) = ui.allocate_exact_size(egui::vec2(48.0, 48.0), Sense::hover());
                paint_icon(ui.painter(), rect.min, XpIcon::Install, 44.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("No Windows programs are installed.")
                            .size(13.0)
                            .strong()
                            .color(Color32::from_rgb(42, 65, 109)),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Use Install a program in the task pane to get started.")
                            .size(11.5)
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
