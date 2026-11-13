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
