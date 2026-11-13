//! Lightweight, original UI pictograms shared by all RiceWine styles.
//! All icons use egui geometry: no external icon font, image loader, or
//! proprietary operating-system artwork is required.

use eframe::egui::{self, Color32, Painter, Pos2, Rect, Stroke, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    Programs,
    Tools,
    Settings,
    Install,
    Import,
    Prefix,
    Runner,
    Backup,
    Play,
    Search,
    Info,
    Warning,
    Refresh,
    Favorite,
    Brand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Pixel,
    Luna,
    Aero,
    Cupertino,
    Fluent,
    Material,
    Aqua,
}

impl Family {
    #[cfg(test)]
    pub const ALL: [Self; 7] = [
        Self::Pixel,
        Self::Luna,
        Self::Aero,
        Self::Cupertino,
        Self::Fluent,
        Self::Material,
        Self::Aqua,
    ];

    fn stroke(self, size: f32) -> f32 {
        let weight = match self {
            Self::Pixel => 1.8,
            Self::Luna => 1.8,
            Self::Aero => 1.55,
            Self::Cupertino => 1.45,
            Self::Fluent => 1.65,
            Self::Material => 1.85,
            Self::Aqua => 1.7,
        };
        (weight * size / 20.0).max(0.9)
    }

    fn round(self, scale: f32) -> f32 {
        match self {
            Self::Pixel => 0.0,
            Self::Luna => 2.0 * scale,
            Self::Aero => 2.0 * scale,
            Self::Cupertino => 3.0 * scale,
            Self::Fluent => 1.0 * scale,
            Self::Material => 4.0 * scale,
            Self::Aqua => 3.0 * scale,
        }
    }
}

/// Paint a meaningful, scalable pictogram within the provided box.
/// All glyphs fit a 20×20 drawing grid and retain contrast from the caller.
pub fn paint(p: &Painter, bounds: Rect, family: Family, glyph: Glyph, color: Color32) {
    let side = bounds.width().min(bounds.height()).max(1.0);
    let scale = side / 20.0;
    let origin = bounds.center() - Vec2::splat(side * 0.5);
    let xy = |x: f32, y: f32| origin + egui::vec2(x * scale, y * scale);
    let stroke = Stroke::new(family.stroke(side), color);
    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment([xy(a.0, a.1), xy(b.0, b.1)], stroke);
    };
    let path = |points: &[(f32, f32)]| {
        p.add(egui::Shape::line(
            points.iter().map(|&(x, y)| xy(x, y)).collect(),
            stroke,
        ));
    };
    let box_line = |x: f32, y: f32, w: f32, h: f32| {
        path(&[(x, y), (x + w, y), (x + w, y + h), (x, y + h), (x, y)]);
    };
    let dot = |x: f32, y: f32, radius: f32| {
        p.circle_filled(xy(x, y), radius * scale, color);
    };
    let rounded = family.round(scale);
    match glyph {
        Glyph::Brand => {
            // The RiceWine goblet: recognisable at 16px without an image
            // decoding dependency. Each Rice supplies its own surface style.
            path(&[
                (4.0, 2.0),
                (16.0, 2.0),
                (14.5, 8.0),
                (12.0, 11.0),
                (8.0, 11.0),
                (5.5, 8.0),
                (4.0, 2.0),
            ]);
            line((10.0, 11.0), (10.0, 17.0));
            line((5.0, 18.0), (15.0, 18.0));
            path(&[(5.5, 5.5), (14.5, 5.5), (13.0, 8.0), (7.0, 8.0)]);
        }
        Glyph::Programs => {
            box_line(2.0, 3.0, 16.0, 11.0);
            line((2.0, 6.0), (18.0, 6.0));
            dot(4.0, 4.5, 0.6);
            dot(6.2, 4.5, 0.6);
            line((10.0, 14.0), (10.0, 17.0));
            line((6.0, 17.0), (14.0, 17.0));
        }
        Glyph::Tools => {
            // A compact toolbox shape, not an ambiguous unicode character.
            box_line(2.0, 7.0, 16.0, 10.0);
            path(&[(6.0, 7.0), (6.0, 4.0), (14.0, 4.0), (14.0, 7.0)]);
            line((2.0, 11.0), (18.0, 11.0));
            box_line(8.0, 10.0, 4.0, 3.0);
        }
        Glyph::Settings => {
            for i in 0..8 {
                let angle = i as f32 * std::f32::consts::TAU / 8.0;
                let (sn, cs) = angle.sin_cos();
                line(
                    (10.0 + cs * 6.0, 10.0 + sn * 6.0),
                    (10.0 + cs * 8.0, 10.0 + sn * 8.0),
                );
            }
            p.circle_stroke(xy(10.0, 10.0), 6.0 * scale, stroke);
            p.circle_stroke(xy(10.0, 10.0), 2.4 * scale, stroke);
        }
        Glyph::Install => {
            path(&[(3.0, 12.0), (3.0, 17.0), (17.0, 17.0), (17.0, 12.0)]);
            line((10.0, 2.0), (10.0, 12.0));
            path(&[(5.5, 9.0), (10.0, 13.5), (14.5, 9.0)]);
        }
        Glyph::Import => {
            path(&[
                (2.0, 6.0),
                (2.0, 17.0),
                (18.0, 17.0),
                (18.0, 8.0),
                (9.0, 8.0),
                (7.0, 6.0),
                (2.0, 6.0),
            ]);
            line((10.0, 3.0), (10.0, 12.0));
            path(&[(7.0, 9.0), (10.0, 12.0), (13.0, 9.0)]);
        }
        Glyph::Prefix => {
            path(&[
                (2.0, 6.0),
                (2.0, 17.0),
                (18.0, 17.0),
                (18.0, 8.0),
                (9.0, 8.0),
                (7.0, 6.0),
                (2.0, 6.0),
            ]);
            box_line(10.0, 10.0, 6.0, 5.0);
        }
        Glyph::Runner => {
            p.circle_stroke(xy(10.0, 10.0), 8.0 * scale, stroke);
            p.circle_stroke(xy(10.0, 10.0), 2.0 * scale, stroke);
            line((10.0, 10.0), (14.0, 5.0));
            line((5.0, 15.0), (15.0, 15.0));
        }
        Glyph::Backup => {
            box_line(3.0, 2.0, 14.0, 16.0);
            box_line(6.0, 3.0, 8.0, 6.0);
            box_line(6.0, 12.0, 8.0, 6.0);
            line((11.0, 4.0), (11.0, 8.0));
        }
        Glyph::Play => {
            p.add(egui::Shape::convex_polygon(
                vec![xy(5.0, 2.0), xy(17.0, 10.0), xy(5.0, 18.0)],
                color,
                Stroke::NONE,
            ));
        }
        Glyph::Search => {
            p.circle_stroke(xy(8.5, 8.5), 5.5 * scale, stroke);
            line((12.5, 12.5), (18.0, 18.0));
        }
        Glyph::Info => {
            p.circle_stroke(xy(10.0, 10.0), 8.0 * scale, stroke);
            dot(10.0, 5.6, 1.0);
            line((10.0, 9.0), (10.0, 14.5));
        }
        Glyph::Warning => {
            path(&[(10.0, 2.0), (19.0, 18.0), (1.0, 18.0), (10.0, 2.0)]);
            line((10.0, 7.0), (10.0, 12.0));
            dot(10.0, 15.0, 0.8);
        }
        Glyph::Refresh => {
            p.circle_stroke(xy(10.0, 10.0), 6.8 * scale, stroke);
            path(&[(10.0, 2.0), (15.0, 3.0), (13.0, 8.0)]);
        }
        Glyph::Favorite => {
            let mut pts = Vec::with_capacity(10);
            for index in 0..10 {
                let angle =
                    -std::f32::consts::FRAC_PI_2 + index as f32 * std::f32::consts::PI / 5.0;
                let radius = if index % 2 == 0 { 8.0 } else { 3.5 };
                pts.push(xy(10.0 + radius * angle.cos(), 10.0 + radius * angle.sin()));
            }
            p.add(egui::Shape::convex_polygon(pts, color, Stroke::NONE));
        }
    }
    // A restrained highlight identifies older glossy styles without decorating
    // the modern flat families. Keep this within the icon footprint.
    if matches!(family, Family::Aero | Family::Aqua)
        && matches!(glyph, Glyph::Programs | Glyph::Tools | Glyph::Prefix)
    {
        p.line_segment(
            [xy(3.0, 6.0), xy(8.0, 6.0)],
            Stroke::new((0.8 * scale).max(0.7), Color32::WHITE),
        );
    }
    if family == Family::Pixel && matches!(glyph, Glyph::Programs | Glyph::Tools) {
        let r = Rect::from_min_size(xy(1.0, 1.0), Vec2::splat(2.0 * scale));
        p.rect_filled(r, rounded, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_family_and_glyph_render_without_panic() {
        let glyphs = [
            Glyph::Programs,
            Glyph::Tools,
            Glyph::Settings,
            Glyph::Install,
            Glyph::Import,
            Glyph::Prefix,
            Glyph::Runner,
            Glyph::Backup,
            Glyph::Play,
            Glyph::Search,
            Glyph::Info,
            Glyph::Warning,
            Glyph::Refresh,
            Glyph::Favorite,
            Glyph::Brand,
        ];
        let ctx = egui::Context::default();
        for family in Family::ALL {
            for &glyph in &glyphs {
                let output = ctx.run(Default::default(), |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(20.0, 20.0), egui::Sense::hover());
                        paint(ui.painter(), rect, family, glyph, Color32::WHITE);
                    });
                });
                assert!(!output.shapes.is_empty(), "{family:?} {glyph:?}");
            }
        }
    }
}
