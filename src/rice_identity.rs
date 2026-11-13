//! Individual visual identities for the seven RiceWine presentations.
//! Each family has its own visual motif, icon treatment, spacing and header.
//! Shared here is only the accessibility and drawing plumbing; Wine operations
//! remain owned by RiceWineApp. No external assets or proprietary icons.

use eframe::egui::{
    self, Color32, Painter, Pos2, Rect, Response, RichText, Sense, Stroke, Ui, Vec2,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Luna,
    Mac,
    Win10,
    Win98,
    Aero7,
    Chrome,
    Frutiger,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Brand,
    Programs,
    Tools,
    Settings,
    Install,
    Import,
    Prefix,
    Runner,
    Backup,
    Play,
}

impl Family {
    pub const ALL: [Self; 7] = [
        Self::Luna,
        Self::Mac,
        Self::Win10,
        Self::Win98,
        Self::Aero7,
        Self::Chrome,
        Self::Frutiger,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Luna => "Luna",
            Self::Mac => "macOS",
            Self::Win10 => "Windows 10",
            Self::Win98 => "Windows 98",
            Self::Aero7 => "Aero 7",
            Self::Chrome => "ChromeOS",
            Self::Frutiger => "Frutiger Aero",
        }
    }

    fn stroke(self, size: f32) -> f32 {
        let weight: f32 = match self {
            Self::Win98 => 1.8,
            Self::Luna => 1.7,
            Self::Aero7 => 1.55,
            Self::Mac => 1.35,
            Self::Win10 => 1.55,
            Self::Chrome => 1.85,
            Self::Frutiger => 1.6,
        };
        (weight * size / 20.0).max(1.0)
    }

    fn radius(self) -> f32 {
        match self {
            Self::Win98 | Self::Win10 => 0.0,
            Self::Luna => 2.0,
            Self::Aero7 => 3.0,
            Self::Mac => 4.0,
            Self::Chrome => 6.0,
            Self::Frutiger => 7.0,
        }
    }
}

/// Seven independent editorial treatments. Their copy and title hierarchy
/// intentionally follow each historical interface rather than only swapping
/// palette variables.
pub fn library_heading(family: Family) -> &'static str {
    match family {
        Family::Luna => "Installed Programs",
        Family::Mac => "Applications",
        Family::Win10 => "Apps & features",
        Family::Win98 => "Programs",
        Family::Aero7 => "Programs and Features",
        Family::Chrome => "Your apps",
        Family::Frutiger => "Program Explorer",
    }
}

pub fn empty_copy(family: Family) -> (&'static str, &'static str) {
    match family {
        Family::Luna => (
            "Welcome to RiceWine",
            "Select a program task below to get started.",
        ),
        Family::Mac => (
            "No Applications",
            "Add a Windows app or import an existing environment.",
        ),
        Family::Win10 => (
            "No apps installed",
            "Install a Windows app to add it to your library.",
        ),
        Family::Win98 => ("No programs found", "Use Install to add a Windows program."),
        Family::Aero7 => (
            "Get started with programs",
            "Install a program or bring in an existing Wine prefix.",
        ),
        Family::Chrome => (
            "Your app shelf is empty",
            "Choose a Windows app to add to your collection.",
        ),
        Family::Frutiger => (
            "A fresh start",
            "Discover Windows apps on your Linux desktop.",
        ),
    }
}

/// Distinct era-specific finish to the shell's chrome. Painted after
/// layout, these details stay within the existing header rectangle and do
/// not change hit testing, tab order or small-window layout.
pub fn header_finish(p: &Painter, r: Rect, family: Family, accent: Color32) {
    let l = r.left() + 2.0;
    let right = r.right() - 2.0;
    let top = r.top() + 2.0;
    let bottom = r.bottom() - 2.0;
    match family {
        Family::Luna => {
            p.line_segment(
                [Pos2::new(l, bottom - 1.0), Pos2::new(right, bottom - 1.0)],
                Stroke::new(2.0, Color32::from_rgb(52, 111, 216)),
            );
            p.line_segment(
                [
                    Pos2::new(l + 3.0, top + 1.0),
                    Pos2::new(right - 3.0, top + 1.0),
                ],
                Stroke::new(1.0, Color32::from_rgb(240, 249, 255)),
            );
        }
        Family::Mac => {
            p.line_segment(
                [Pos2::new(l, bottom), Pos2::new(right, bottom)],
                Stroke::new(1.0, Color32::from_rgb(180, 187, 199)),
            );
        }
        Family::Win10 => {
            p.line_segment(
                [Pos2::new(l, bottom), Pos2::new(right, bottom)],
                Stroke::new(1.0, Color32::from_rgb(208, 208, 208)),
            );
        }
        Family::Win98 => {
            let white = Color32::WHITE;
            let dark = Color32::from_rgb(100, 100, 100);
            p.line_segment(
                [Pos2::new(l, top), Pos2::new(right, top)],
                Stroke::new(1.0, white),
            );
            p.line_segment(
                [Pos2::new(l, top), Pos2::new(l, bottom)],
                Stroke::new(1.0, white),
            );
            p.line_segment(
                [Pos2::new(l, bottom), Pos2::new(right, bottom)],
                Stroke::new(1.0, dark),
            );
            p.line_segment(
                [Pos2::new(right, top), Pos2::new(right, bottom)],
                Stroke::new(1.0, dark),
            );
        }
        Family::Aero7 => {
            p.line_segment(
                [
                    Pos2::new(l + 5.0, top + 1.0),
                    Pos2::new(right - 5.0, top + 1.0),
                ],
                Stroke::new(1.0, Color32::from_rgb(255, 255, 255)),
            );
            p.line_segment(
                [Pos2::new(l, bottom), Pos2::new(right, bottom)],
                Stroke::new(2.0, Color32::from_rgb(131, 184, 221)),
            );
        }
        Family::Chrome => {
            p.circle_filled(Pos2::new(right - 7.0, top + 8.0), 2.0, accent);
        }
        Family::Frutiger => {
            p.line_segment(
                [
                    Pos2::new(l + 7.0, top + 1.0),
                    Pos2::new(right - 7.0, top + 1.0),
                ],
                Stroke::new(1.5, Color32::from_rgb(236, 255, 251)),
            );
            p.line_segment(
                [Pos2::new(l, bottom), Pos2::new(right, bottom)],
                Stroke::new(2.0, Color32::from_rgb(66, 187, 171)),
            );
        }
    }
}

#[derive(Default)]
pub struct EmptyActions {
    pub install: bool,
    pub import: bool,
}

/// Seven separately composed empty library views, not one recolored card.
/// The caller owns the actions; this only supplies presentation and click
/// signals. These layouts stay compact enough for 640×480 screenshot tests.
#[expect(clippy::too_many_arguments, reason = "the colors are named explicitly for each Rice presentation")]
pub fn empty_library(
    ui: &mut Ui,
    family: Family,
    accent: Color32,
    text: Color32,
    muted: Color32,
    surface: Color32,
    border: Color32,
    enabled: bool,
) -> EmptyActions {
    let mut actions = EmptyActions::default();
    let (heading, description) = empty_copy(family);
    let primary = |ui: &mut Ui, actions: &mut EmptyActions| {
        actions.install |= button(
            ui,
            family,
            Icon::Install,
            "Install Windows program",
            Color32::WHITE,
            Color32::WHITE,
            accent,
            accent,
            enabled,
            235.0,
        )
        .clicked();
    };
    let secondary = |ui: &mut Ui, actions: &mut EmptyActions| {
        actions.import |= button(
            ui,
            family,
            Icon::Import,
            "Import existing prefix",
            accent,
            text,
            surface,
            border,
            enabled,
            235.0,
        )
        .clicked();
    };
    match family {
        Family::Luna => {
            // The cheerful XP task pane: blue header, warm task area.
            egui::Frame::new()
                .fill(Color32::from_rgb(246, 248, 254))
                .stroke(Stroke::new(1.0, border))
                .corner_radius(egui::CornerRadius::same(7))
                .inner_margin(egui::Margin::same(11))
                .show(ui, |ui| {
                    let w = ui.available_width();
                    egui::Frame::new()
                        .fill(Color32::from_rgb(53, 104, 199))
                        .corner_radius(egui::CornerRadius::same(5))
                        .inner_margin(egui::Margin::symmetric(11, 7))
                        .show(ui, |ui| {
                            ui.set_min_width((w - 46.0).max(0.0));
                            ui.horizontal(|ui| {
                                symbol(ui, family, Icon::Programs, Color32::WHITE, 22.0);
                                ui.label(RichText::new(heading).strong().color(Color32::WHITE));
                            });
                        });
                    ui.add_space(8.0);
                    ui.small(description);
                    ui.add_space(7.0);
                    ui.label(RichText::new("Program tasks").strong().color(accent));
                    ui.horizontal_wrapped(|ui| {
                        primary(ui, &mut actions);
                        secondary(ui, &mut actions);
                    });
                });
        }
        Family::Win98 => {
            // Sunken gray dialog and simple commands instead of a big hero.
            egui::Frame::new()
                .fill(Color32::from_rgb(192, 192, 192))
                .stroke(Stroke::new(2.0, Color32::from_rgb(105, 105, 105)))
                .inner_margin(egui::Margin::same(9))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        symbol(ui, family, Icon::Programs, accent, 28.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(heading).strong().color(text));
                            ui.small(description);
                        });
                    });
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        primary(ui, &mut actions);
                        secondary(ui, &mut actions);
                    });
                });
        }
        Family::Aero7 => {
            // Explorer/Control Panel: a pale blue pane with a small heading.
            egui::Frame::new()
                .fill(Color32::from_rgb(237, 247, 254))
                .stroke(Stroke::new(1.0, Color32::from_rgb(142, 186, 219)))
                .corner_radius(egui::CornerRadius::same(7))
                .inner_margin(egui::Margin::same(13))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        symbol(ui, family, Icon::Programs, accent, 34.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(heading).size(18.0).strong().color(text));
                            ui.label(RichText::new(description).color(muted));
                        });
                    });
                    ui.add_space(11.0);
                    ui.horizontal_wrapped(|ui| {
                        primary(ui, &mut actions);
                        secondary(ui, &mut actions);
                    });
                });
        }
        Family::Mac => {
            // Quiet centered Finder-like utility state, no decorative card.
            ui.vertical_centered(|ui| {
                ui.add_space(7.0);
                symbol(ui, family, Icon::Brand, accent, 45.0);
                ui.add_space(5.0);
                ui.label(RichText::new(heading).size(20.0).strong().color(text));
                ui.label(RichText::new(description).color(muted));
                ui.add_space(9.0);
                primary(ui, &mut actions);
                ui.add_space(3.0);
                secondary(ui, &mut actions);
            });
        }
        Family::Win10 => {
            // Flat settings-style command surface aligned with the page edge.
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    symbol(ui, family, Icon::Programs, accent, 36.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(heading).size(20.0).color(text));
                        ui.label(RichText::new(description).color(muted));
                    });
                });
                ui.add_space(12.0);
                ui.horizontal_wrapped(|ui| {
                    primary(ui, &mut actions);
                    secondary(ui, &mut actions);
                });
            });
        }
        Family::Chrome => {
            // Material-style app onboarding, light and centrally balanced.
            ui.vertical_centered(|ui| {
                ui.add_space(4.0);
                egui::Frame::new()
                    .fill(Color32::from_rgb(231, 240, 254))
                    .corner_radius(egui::CornerRadius::same(22))
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| symbol(ui, family, Icon::Programs, accent, 32.0));
                ui.add_space(7.0);
                ui.label(RichText::new(heading).size(21.0).strong().color(text));
                ui.label(RichText::new(description).color(muted));
                ui.add_space(9.0);
                primary(ui, &mut actions);
                secondary(ui, &mut actions);
            });
        }
        Family::Frutiger => {
            // Organic aqua panel and translucent-feeling mint surface.
            egui::Frame::new()
                .fill(Color32::from_rgb(222, 249, 242))
                .stroke(Stroke::new(1.0, Color32::from_rgb(106, 188, 177)))
                .corner_radius(egui::CornerRadius::same(14))
                .inner_margin(egui::Margin::same(14))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        symbol(ui, family, Icon::Brand, accent, 48.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(heading).size(21.0).strong().color(text));
                            ui.label(RichText::new(description).color(muted));
                        });
                    });
                    ui.add_space(11.0);
                    ui.horizontal_wrapped(|ui| {
                        primary(ui, &mut actions);
                        secondary(ui, &mut actions);
                    });
                });
        }
    }
    actions
}

/// Paint an original icon whose silhouette, surface treatment and accent are
/// selected for a particular Rice. The names are semantic; no Unicode icons
/// or external font must render correctly.
pub fn paint(p: &Painter, bounds: Rect, family: Family, icon: Icon, accent: Color32) {
    let side = bounds.width().min(bounds.height()).max(1.0);
    let k = side / 20.0;
    let at = bounds.center() - Vec2::splat(side * 0.5);
    let pos = |x: f32, y: f32| at + egui::vec2(x * k, y * k);
    let stroke = Stroke::new(family.stroke(side), accent);
    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment([pos(a.0, a.1), pos(b.0, b.1)], stroke);
    };
    let polygon = |coordinates: &[(f32, f32)]| {
        p.add(egui::Shape::line(
            coordinates.iter().map(|&(x, y)| pos(x, y)).collect(),
            stroke,
        ));
    };
    let rect = |x: f32, y: f32, w: f32, h: f32, color: Color32| {
        p.rect_filled(
            Rect::from_min_size(pos(x, y), egui::vec2(w * k, h * k)),
            family.radius() * k,
            color,
        );
    };
    let circle = |x: f32, y: f32, r: f32| {
        p.circle_stroke(pos(x, y), r * k, stroke);
    };
    // The brand mark genuinely differs by family rather than sharing an
    // identical logo drawn seven times with a different primary color.
    if icon == Icon::Brand {
        match family {
            Family::Luna => {
                rect(1.0, 2.0, 18.0, 16.0, Color32::from_rgb(55, 116, 218));
                rect(3.0, 4.0, 14.0, 5.0, Color32::from_rgb(155, 209, 255));
                line((5.0, 14.0), (15.0, 14.0));
                line((5.0, 16.0), (12.0, 16.0));
            }
            Family::Mac => {
                p.circle_filled(pos(10.0, 10.0), 8.0 * k, Color32::from_rgb(230, 235, 244));
                circle(10.0, 10.0, 7.5);
                polygon(&[
                    (7.0, 4.0),
                    (7.0, 15.0),
                    (12.0, 15.0),
                    (14.0, 12.5),
                    (14.0, 4.0),
                ]);
            }
            Family::Win10 => {
                for x in 0..2 {
                    for y in 0..2 {
                        rect(2.0 + x as f32 * 8.5, 2.0 + y as f32 * 8.5, 7.0, 7.0, accent);
                    }
                }
            }
            Family::Win98 => {
                rect(1.0, 1.0, 18.0, 18.0, Color32::from_rgb(128, 128, 128));
                rect(2.0, 2.0, 16.0, 16.0, Color32::from_rgb(220, 220, 220));
                rect(4.0, 4.0, 12.0, 4.0, Color32::from_rgb(0, 0, 128));
                rect(4.0, 10.0, 12.0, 5.0, Color32::from_rgb(255, 255, 255));
            }
            Family::Aero7 => {
                p.circle_filled(pos(10.0, 10.0), 9.0 * k, Color32::from_rgb(163, 223, 250));
                p.circle_filled(pos(10.0, 10.0), 6.6 * k, accent);
                rect(5.0, 4.0, 10.0, 3.0, Color32::from_rgb(233, 251, 255));
                line((6.0, 13.0), (14.0, 13.0));
            }
            Family::Chrome => {
                p.circle_filled(pos(10.0, 10.0), 9.0 * k, accent);
                p.circle_filled(pos(10.0, 10.0), 5.7 * k, Color32::WHITE);
                p.circle_filled(pos(10.0, 10.0), 2.8 * k, accent);
            }
            Family::Frutiger => {
                p.circle_filled(pos(10.0, 10.0), 9.0 * k, Color32::from_rgb(83, 196, 173));
                p.circle_filled(pos(7.0, 7.0), 5.0 * k, Color32::from_rgb(196, 250, 242));
                polygon(&[(4.0, 13.0), (9.0, 16.0), (16.0, 8.0)]);
            }
        }
        return;
    }
    match icon {
        Icon::Programs => {
            rect(2.0, 3.0, 16.0, 12.0, accent);
            rect(3.7, 5.0, 12.6, 8.0, Color32::WHITE);
            line((10.0, 15.0), (10.0, 18.0));
            line((6.0, 18.0), (14.0, 18.0));
        }
        Icon::Tools => {
            polygon(&[
                (2.0, 8.0),
                (18.0, 8.0),
                (18.0, 17.0),
                (2.0, 17.0),
                (2.0, 8.0),
            ]);
            polygon(&[(6.0, 8.0), (6.0, 4.0), (14.0, 4.0), (14.0, 8.0)]);
            rect(8.0, 10.0, 4.0, 3.0, accent);
        }
        Icon::Settings => {
            circle(10.0, 10.0, 6.0);
            circle(10.0, 10.0, 2.0);
            for i in 0..8 {
                let angle = i as f32 * std::f32::consts::TAU / 8.0;
                let (s, c) = angle.sin_cos();
                line(
                    (10.0 + 6.0 * c, 10.0 + 6.0 * s),
                    (10.0 + 8.0 * c, 10.0 + 8.0 * s),
                );
            }
        }
        Icon::Install => {
            line((10.0, 2.0), (10.0, 13.0));
            polygon(&[(5.5, 9.0), (10.0, 13.5), (14.5, 9.0)]);
            polygon(&[(3.0, 12.0), (3.0, 18.0), (17.0, 18.0), (17.0, 12.0)]);
        }
        Icon::Import | Icon::Prefix => {
            polygon(&[
                (2.0, 5.0),
                (8.0, 5.0),
                (10.0, 7.0),
                (18.0, 7.0),
                (18.0, 17.0),
                (2.0, 17.0),
                (2.0, 5.0),
            ]);
            if icon == Icon::Import {
                line((10.0, 2.0), (10.0, 13.0));
                polygon(&[(7.0, 10.0), (10.0, 13.0), (13.0, 10.0)]);
            } else {
                rect(11.0, 10.0, 4.0, 4.0, accent);
            }
        }
        Icon::Runner => {
            circle(10.0, 10.0, 8.0);
            circle(10.0, 10.0, 2.0);
            line((10.0, 10.0), (15.0, 5.0));
        }
        Icon::Backup => {
            rect(3.0, 2.0, 14.0, 16.0, accent);
            rect(6.0, 3.0, 8.0, 6.0, Color32::WHITE);
            rect(6.0, 12.0, 8.0, 5.0, Color32::WHITE);
        }
        Icon::Play => {
            p.add(egui::Shape::convex_polygon(
                vec![pos(5.0, 2.0), pos(17.0, 10.0), pos(5.0, 18.0)],
                accent,
                Stroke::NONE,
            ));
        }
        Icon::Brand => unreachable!(),
    }
    // Distinct surface families: classic bevel, Luna highlight, Aero reflection,
    // thin mac strokes, Win10 flat, material discs and Frutiger sheen.
    match family {
        Family::Win98 => {
            line((1.0, 1.0), (1.0, 17.0));
            line((1.0, 1.0), (16.0, 1.0));
        }
        Family::Luna => {
            p.circle_filled(pos(3.0, 3.0), 1.4 * k, Color32::from_rgb(255, 198, 65));
        }
        Family::Aero7 => {
            line((4.0, 2.0), (14.0, 2.0));
        }
        Family::Mac => {}
        Family::Win10 => {}
        Family::Chrome => {
            p.circle_filled(pos(17.0, 3.0), 1.1 * k, accent);
        }
        Family::Frutiger => {
            p.circle_filled(pos(3.0, 3.0), 1.6 * k, Color32::from_rgb(138, 232, 216));
        }
    }
}

pub fn symbol(ui: &mut Ui, family: Family, icon: Icon, accent: Color32, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    paint(ui.painter(), rect, family, icon, accent);
}

/// Draw a family-specific button but keep the label as real button text for
/// accessibility and keyboard navigation.
#[expect(clippy::too_many_arguments, reason = "the call site specifies text, icon and independent accessible colors")]
pub fn button(
    ui: &mut Ui,
    family: Family,
    icon: Icon,
    title: &str,
    accent: Color32,
    foreground: Color32,
    fill: Color32,
    border: Color32,
    enabled: bool,
    width: f32,
) -> Response {
    let corner = match family {
        Family::Win98 | Family::Win10 => 0,
        Family::Luna => 4,
        Family::Aero7 => 5,
        Family::Mac => 9,
        Family::Chrome => 13,
        Family::Frutiger => 12,
    };
    let caption = format!("    {title}");
    let button = egui::Button::new(RichText::new(caption).strong().color(foreground))
        .fill(fill)
        .stroke(Stroke::new(1.0, border))
        .corner_radius(egui::CornerRadius::same(corner));
    let response = ui.add_enabled(enabled, button.min_size(egui::vec2(width, 35.0)));
    let icon_rect = Rect::from_center_size(
        Pos2::new(response.rect.left() + 14.0, response.rect.center().y),
        Vec2::splat(16.0),
    );
    paint(
        ui.painter(),
        icon_rect,
        family,
        icon,
        if enabled {
            accent
        } else {
            accent.gamma_multiply(0.4)
        },
    );
    response
}

pub fn title(
    ui: &mut Ui,
    family: Family,
    icon: Icon,
    text: &str,
    accent: Color32,
    foreground: Color32,
) {
    ui.horizontal(|ui| {
        symbol(ui, family, icon, accent, 22.0);
        ui.add_space(5.0);
        ui.label(
            RichText::new(text)
                .strong()
                .size(match family {
                    Family::Win98 => 18.0,
                    Family::Luna => 19.0,
                    Family::Mac => 22.0,
                    Family::Win10 => 22.0,
                    Family::Aero7 => 21.0,
                    Family::Chrome => 23.0,
                    Family::Frutiger => 22.0,
                })
                .color(foreground),
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn each_family_defines_independent_identity_and_semantic_icons() {
        let icons = [
            Icon::Brand,
            Icon::Programs,
            Icon::Tools,
            Icon::Settings,
            Icon::Install,
            Icon::Import,
            Icon::Prefix,
            Icon::Runner,
            Icon::Backup,
            Icon::Play,
        ];
        assert_eq!(Family::ALL.len(), 7);
        assert_eq!(icons.len(), 10);
        for (index, family) in Family::ALL.iter().enumerate() {
            assert!(!family.name().is_empty());
            for next in Family::ALL.iter().skip(index + 1) {
                assert_ne!(family.name(), next.name());
            }
        }
    }
}
