//! Small original pixel/vector icons inspired by the Windows XP application era.
//! No proprietary Windows icons or external image assets are bundled.

use eframe::egui::{self, Color32, Painter, Pos2, Rect, Stroke};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XpIcon {
    Programs,
    Install,
    Import,
    Prefix,
    Runner,
    Settings,
    Info,
    Warning,
    Disk,
}

fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

fn block(p: &Painter, at: Pos2, k: f32, bounds: [f32; 4], fill: Color32) {
    let [x, y, w, h] = bounds;
    p.rect_filled(
        Rect::from_min_size(at + egui::vec2(x * k, y * k), egui::vec2(w * k, h * k)),
        0.0,
        fill,
    );
}

fn segment(
    p: &Painter,
    at: Pos2,
    k: f32,
    from: [f32; 2],
    to: [f32; 2],
    color: Color32,
    width: f32,
) {
    p.line_segment(
        [
            at + egui::vec2(from[0] * k, from[1] * k),
            at + egui::vec2(to[0] * k, to[1] * k),
        ],
        Stroke::new((width * k).max(0.8), color),
    );
}

fn disc(p: &Painter, at: Pos2, k: f32, cx: f32, cy: f32, radius: f32, color: Color32) {
    p.circle_filled(at + egui::vec2(cx * k, cy * k), radius * k, color);
}

/// Draw an original, shaded 16-pixel-era icon at any application scale.
/// Uses only egui primitives; no font-glyph placeholders or bundled icons.
pub fn paint_icon(p: &Painter, at: Pos2, icon: XpIcon, size: f32) {
    let k = size / 16.0;
    let navy = rgb(29, 63, 120);
    let shadow = rgb(80, 91, 102);
    let white = Color32::WHITE;
    match icon {
        XpIcon::Programs => {
            block(p, at, k, [1.0, 1.0, 14.0, 11.0], navy);
            block(p, at, k, [2.0, 2.0, 12.0, 9.0], rgb(139, 200, 240));
            block(p, at, k, [3.0, 3.0, 10.0, 2.0], rgb(201, 239, 255));
            block(p, at, k, [3.0, 6.0, 10.0, 4.0], rgb(54, 131, 208));
            segment(p, at, k, [8.0, 12.0], [8.0, 14.0], shadow, 2.0);
            block(p, at, k, [4.0, 14.0, 8.0, 1.0], shadow);
        }
        XpIcon::Install => {
            disc(p, at, k, 6.5, 7.5, 6.4, rgb(94, 100, 144));
            disc(p, at, k, 6.5, 7.5, 5.3, rgb(231, 235, 244));
            segment(p, at, k, [2.0, 5.5], [10.0, 2.0], rgb(167, 224, 250), 2.0);
            segment(p, at, k, [3.0, 11.5], [10.0, 7.0], rgb(223, 186, 233), 2.0);
            disc(p, at, k, 6.5, 7.5, 1.6, rgb(118, 122, 147));
            block(p, at, k, [9.0, 11.0, 5.0, 3.0], rgb(38, 116, 25));
            block(p, at, k, [11.0, 8.0, 3.0, 6.0], rgb(79, 181, 44));
            segment(p, at, k, [9.0, 12.0], [12.0, 15.0], rgb(46, 127, 26), 2.0);
            segment(p, at, k, [15.0, 12.0], [12.0, 15.0], rgb(46, 127, 26), 2.0);
        }
        XpIcon::Import | XpIcon::Prefix => {
            block(p, at, k, [1.0, 3.0, 8.0, 3.0], rgb(163, 100, 17));
            block(p, at, k, [2.0, 3.0, 7.0, 2.0], rgb(255, 211, 108));
            block(p, at, k, [1.0, 5.0, 14.0, 9.0], rgb(177, 110, 18));
            block(p, at, k, [2.0, 6.0, 12.0, 7.0], rgb(248, 187, 58));
            block(p, at, k, [3.0, 7.0, 10.0, 2.0], rgb(255, 226, 121));
            if icon == XpIcon::Prefix {
                block(p, at, k, [7.0, 7.0, 5.0, 7.0], rgb(238, 245, 255));
                block(p, at, k, [8.0, 9.0, 3.0, 1.0], rgb(64, 130, 213));
                block(p, at, k, [8.0, 11.0, 3.0, 1.0], rgb(145, 167, 199));
            } else {
                block(p, at, k, [10.0, 1.0, 2.0, 6.0], rgb(20, 97, 164));
                segment(p, at, k, [8.0, 5.0], [11.0, 8.0], rgb(20, 97, 164), 2.0);
                segment(p, at, k, [14.0, 5.0], [11.0, 8.0], rgb(20, 97, 164), 2.0);
            }
        }
        XpIcon::Runner => {
            disc(p, at, k, 7.8, 7.8, 6.8, rgb(77, 84, 123));
            disc(p, at, k, 7.8, 7.8, 5.7, rgb(198, 217, 241));
            disc(p, at, k, 7.8, 7.8, 4.7, rgb(235, 240, 249));
            segment(p, at, k, [4.0, 5.0], [12.0, 11.0], rgb(147, 169, 230), 1.8);
            segment(p, at, k, [3.0, 10.0], [12.0, 4.0], rgb(192, 173, 226), 1.8);
            disc(p, at, k, 7.8, 7.8, 2.1, rgb(100, 114, 148));
            disc(p, at, k, 7.8, 7.8, 1.2, white);
        }
        XpIcon::Settings => {
            for n in 0..8 {
                let angle = n as f32 * std::f32::consts::TAU / 8.0;
                let dx = angle.cos();
                let dy = angle.sin();
                segment(
                    p,
                    at,
                    k,
                    [8.0 + dx * 4.2, 8.0 + dy * 4.2],
                    [8.0 + dx * 6.5, 8.0 + dy * 6.5],
                    rgb(72, 87, 114),
                    3.0,
                );
            }
            disc(p, at, k, 8.0, 8.0, 5.0, rgb(77, 107, 157));
            disc(p, at, k, 7.3, 7.1, 3.8, rgb(156, 198, 235));
            disc(p, at, k, 8.0, 8.0, 2.1, rgb(54, 79, 124));
            disc(p, at, k, 8.0, 8.0, 1.1, rgb(238, 247, 255));
        }
        XpIcon::Info => {
            disc(p, at, k, 8.0, 8.0, 7.0, rgb(45, 87, 174));
            disc(p, at, k, 8.0, 7.7, 5.7, rgb(94, 155, 233));
            block(p, at, k, [7.2, 7.0, 1.7, 5.0], white);
            disc(p, at, k, 8.0, 4.7, 1.1, white);
        }
        XpIcon::Warning => {
            p.add(egui::Shape::convex_polygon(
                vec![
                    at + egui::vec2(8.0 * k, 0.0),
                    at + egui::vec2(16.0 * k, 14.0 * k),
                    at + egui::vec2(0.0, 14.0 * k),
                ],
                rgb(246, 201, 71),
                Stroke::new(k.max(0.8), rgb(148, 110, 26)),
            ));
            block(p, at, k, [7.0, 5.0, 2.0, 5.0], rgb(84, 64, 22));
            disc(p, at, k, 8.0, 12.0, 1.1, rgb(84, 64, 22));
        }
        XpIcon::Disk => {
            block(p, at, k, [1.0, 3.0, 14.0, 11.0], rgb(75, 89, 115));
            block(p, at, k, [2.0, 3.0, 12.0, 9.0], rgb(219, 229, 244));
            block(p, at, k, [4.0, 4.0, 7.0, 4.0], rgb(96, 151, 210));
            block(p, at, k, [5.0, 4.0, 3.0, 4.0], rgb(226, 246, 255));
            block(p, at, k, [4.0, 9.0, 8.0, 4.0], rgb(255, 255, 252));
            block(p, at, k, [5.0, 10.0, 6.0, 2.0], rgb(153, 187, 219));
        }
    }
}
