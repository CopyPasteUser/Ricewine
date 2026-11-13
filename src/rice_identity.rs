//! Presentation layer for the seven independently art-directed RiceWine Rices.
//! Only layout and iconography live here; Wine operations remain in RiceWineApp.
//! Do not emulate compositor-owned window decorations.

use crate::luna_icons::{self, XpIcon};
use crate::rice_icons::{self, Family, Glyph};
use crate::{Page, Theme};
use eframe::egui::{
    self, Color32, Frame, Painter, Rect, Response, RichText, Sense, Stroke, Ui, Vec2,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    Page(Page),
    Install,
    Import,
    Refresh,
    About,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyIntent {
    Install,
    Import,
}

fn family(rice: Theme) -> Family {
    match rice {
        Theme::Luna => Family::Luna,
        Theme::MacOS => Family::Cupertino,
        Theme::Windows10 => Family::Fluent,
        Theme::Windows98 => Family::Pixel,
        Theme::Windows7 => Family::Aero,
        Theme::ChromeOS => Family::Material,
        Theme::FrutigerAero => Family::Aqua,
    }
}

fn xp_glyph(glyph: Glyph) -> XpIcon {
    match glyph {
        Glyph::Brand => XpIcon::Wine,
        Glyph::Programs | Glyph::Play | Glyph::Search | Glyph::Favorite => XpIcon::Programs,
        Glyph::Tools | Glyph::Settings => XpIcon::Settings,
        Glyph::Install => XpIcon::Install,
        Glyph::Import => XpIcon::Import,
        Glyph::Prefix => XpIcon::Prefix,
        Glyph::Runner | Glyph::Refresh => XpIcon::Runner,
        Glyph::Backup => XpIcon::Disk,
        Glyph::Info => XpIcon::Info,
        Glyph::Warning => XpIcon::Warning,
    }
}

/// Small intentionally square bitmap-style pictograms for the 98 Rice.
/// Distinct from the XP shaded assets and the newer vector families.
fn pixel_98(p: &Painter, rect: Rect, glyph: Glyph) {
    let c = Color32::from_rgb(0, 32, 124);
    let gray = Color32::from_rgb(75, 75, 75);
    let gold = Color32::from_rgb(230, 180, 43);
    let k = rect.width().min(rect.height()) / 16.0;
    let xy = |x: f32, y: f32| rect.min + egui::vec2(x * k, y * k);
    let block = |x: f32, y: f32, w: f32, h: f32, color: Color32| {
        p.rect_filled(
            Rect::from_min_size(xy(x, y), egui::vec2(w * k, h * k)),
            0.0,
            color,
        );
    };
    match glyph {
        Glyph::Brand => {
            // The same recognizable goblet expressed as a square 16-pixel icon.
            block(2.0, 2.0, 12.0, 2.0, c);
            block(3.0, 4.0, 10.0, 4.0, Color32::from_rgb(78, 134, 211));
            block(5.0, 8.0, 6.0, 2.0, c);
            block(7.0, 10.0, 2.0, 4.0, gray);
            block(4.0, 14.0, 8.0, 2.0, gray);
        }
        Glyph::Prefix | Glyph::Import | Glyph::Backup => {
            block(1.0, 4.0, 14.0, 10.0, gray);
            block(2.0, 3.0, 7.0, 3.0, gold);
            block(2.0, 6.0, 12.0, 7.0, gold);
            block(3.0, 7.0, 10.0, 2.0, Color32::from_rgb(255, 225, 125));
        }
        Glyph::Settings | Glyph::Tools | Glyph::Runner | Glyph::Refresh => {
            block(2.0, 2.0, 12.0, 12.0, gray);
            block(4.0, 4.0, 8.0, 8.0, Color32::from_rgb(192, 192, 192));
            block(6.0, 6.0, 4.0, 4.0, c);
        }
        Glyph::Install | Glyph::Play => {
            block(6.0, 1.0, 4.0, 10.0, c);
            block(3.0, 7.0, 10.0, 4.0, c);
            block(5.0, 11.0, 6.0, 3.0, gold);
        }
        Glyph::Warning | Glyph::Info => {
            block(3.0, 2.0, 10.0, 12.0, gold);
            block(7.0, 5.0, 2.0, 5.0, gray);
            block(7.0, 12.0, 2.0, 1.0, gray);
        }
        _ => {
            block(1.0, 2.0, 14.0, 11.0, gray);
            block(2.0, 3.0, 12.0, 9.0, Color32::from_rgb(224, 232, 255));
            block(2.0, 3.0, 12.0, 3.0, c);
            block(5.0, 14.0, 6.0, 1.0, gray);
        }
    }
}

/// Icon family is a component of each Rice, not a shared icon skin.
/// XP uses its shaded native artwork, 98 uses pixel icons, macOS/Win10 use
/// crisp monochrome lines, 7/Aero add translucent icon badges, and ChromeOS
/// uses a circular material token.
pub fn icon(p: &Painter, rice: Theme, glyph: Glyph, rect: Rect, color: Color32) {
    match rice {
        Theme::Luna => luna_icons::paint_icon(p, rect.min, xp_glyph(glyph), rect.width()),
        Theme::Windows98 => pixel_98(p, rect, glyph),
        Theme::Windows7 | Theme::FrutigerAero => {
            let (bg, stroke) = if rice == Theme::Windows7 {
                (
                    Color32::from_rgb(222, 242, 253),
                    Color32::from_rgb(100, 166, 205),
                )
            } else {
                (
                    Color32::from_rgb(216, 253, 240),
                    Color32::from_rgb(36, 155, 149),
                )
            };
            p.circle_filled(rect.center(), rect.width() * 0.51, bg);
            p.circle_stroke(rect.center(), rect.width() * 0.50, Stroke::new(0.8, stroke));
            let inner = rect.shrink(rect.width() * 0.16);
            rice_icons::paint(p, inner, family(rice), glyph, color);
        }
        Theme::ChromeOS => {
            p.circle_filled(
                rect.center(),
                rect.width() * 0.52,
                Color32::from_rgb(230, 239, 255),
            );
            rice_icons::paint(
                p,
                rect.shrink(rect.width() * 0.18),
                family(rice),
                glyph,
                color,
            );
        }
        Theme::MacOS | Theme::Windows10 => rice_icons::paint(p, rect, family(rice), glyph, color),
    }
}

pub fn inline_icon(ui: &mut Ui, rice: Theme, glyph: Glyph, side: f32, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(side), Sense::hover());
    icon(ui.painter(), rice, glyph, rect, color);
}

pub fn icon_button(
    ui: &mut Ui,
    rice: Theme,
    label: &str,
    glyph: Glyph,
    enabled: bool,
    emphasis: bool,
    width: f32,
) -> Response {
    let p = rice.palette();
    let fg = if emphasis { Color32::WHITE } else { p.text };
    let bg = if emphasis { p.accent } else { p.button };
    let stroke = match rice {
        Theme::Windows98 => Stroke::new(1.4, Color32::from_rgb(87, 87, 87)),
        Theme::Luna => Stroke::new(1.0, Color32::from_rgb(90, 129, 193)),
        Theme::Windows7 => Stroke::new(1.0, Color32::from_rgb(109, 163, 204)),
        Theme::MacOS => Stroke::new(0.7, Color32::from_rgb(193, 201, 212)),
        Theme::Windows10 => Stroke::new(if emphasis { 0.0 } else { 1.0 }, p.border),
        Theme::ChromeOS | Theme::FrutigerAero => Stroke::new(1.0, p.border),
    };
    let height: f32 = match rice {
        Theme::Windows98 => 29.0,
        Theme::Luna => 32.0,
        Theme::Windows10 => 34.0,
        Theme::Windows7 | Theme::MacOS => 35.0,
        Theme::ChromeOS | Theme::FrutigerAero => 39.0,
    };
    let height = if ui.ctx().content_rect().height() < 600.0 {
        height.min(31.0)
    } else {
        height
    };
    let button = egui::Button::new(RichText::new(format!("     {label}")).color(fg).strong())
        .fill(bg)
        .stroke(stroke)
        .corner_radius(egui::CornerRadius::same(p.corner_radius));
    let response = ui
        .add_enabled_ui(enabled, |ui| ui.add_sized([width, height], button))
        .inner;
    let center = egui::pos2(response.rect.left() + 17.0, response.rect.center().y);
    let rect = Rect::from_center_size(center, Vec2::splat(19.0));
    icon(
        ui.painter(),
        rice,
        glyph,
        rect,
        if emphasis { Color32::WHITE } else { p.accent },
    );
    if rice == Theme::Windows98 {
        // Raised 3D edge is fundamental to the Win98 language.
        let r = response.rect.shrink(1.0);
        ui.painter().line_segment(
            [r.left_bottom(), r.left_top()],
            Stroke::new(1.0, Color32::WHITE),
        );
        ui.painter().line_segment(
            [r.left_top(), r.right_top()],
            Stroke::new(1.0, Color32::WHITE),
        );
        ui.painter().line_segment(
            [r.right_top(), r.right_bottom()],
            Stroke::new(1.0, Color32::DARK_GRAY),
        );
        ui.painter().line_segment(
            [r.right_bottom(), r.left_bottom()],
            Stroke::new(1.0, Color32::DARK_GRAY),
        );
    }
    response
}

fn brand(ui: &mut Ui, rice: Theme, heading: &str, subline: Option<&str>, fg: Color32) {
    ui.horizontal(|ui| {
        inline_icon(
            ui,
            rice,
            Glyph::Brand,
            if rice == Theme::Windows98 { 20.0 } else { 25.0 },
            fg,
        );
        ui.vertical(|ui| {
            ui.label(
                RichText::new(heading)
                    .size(match rice {
                        Theme::Windows98 => 16.0,
                        Theme::Luna => 19.0,
                        Theme::Windows10 => 22.0,
                        Theme::FrutigerAero => 22.0,
                        _ => 20.0,
                    })
                    .strong()
                    .color(fg),
            );
            if let Some(note) = subline.filter(|_| {
                ui.ctx().content_rect().height() >= 600.0
                    || matches!(rice, Theme::Luna | Theme::Windows98 | Theme::Windows7)
            }) {
                ui.label(
                    RichText::new(note)
                        .size(10.5)
                        .color(fg.gamma_multiply(0.85)),
                );
            }
        });
    });
}

fn tools_menu(ui: &mut Ui, rice: Theme, can_install: bool) -> Option<Intent> {
    let mut action = None;
    let caption = if rice == Theme::Windows98 {
        "Tools"
    } else {
        "Tools ▾"
    };
    let result = ui.menu_button(format!("    {caption}"), |ui| {
        ui.set_min_width(225.0);
        for (label, icon, intent) in [
            (
                "Wine environments & backups",
                Glyph::Prefix,
                Intent::Page(Page::Prefixes),
            ),
            (
                "Wine / Proton runners",
                Glyph::Runner,
                Intent::Page(Page::Runners),
            ),
        ] {
            if icon_button(ui, rice, label, icon, true, false, 225.0).clicked() {
                action = Some(intent);
                ui.close();
            }
        }
        ui.separator();
        if icon_button(
            ui,
            rice,
            "Import existing prefix",
            Glyph::Import,
            can_install,
            false,
            225.0,
        )
        .clicked()
        {
            action = Some(Intent::Import);
            ui.close();
        }
        if icon_button(
            ui,
            rice,
            "Refresh runner status",
            Glyph::Refresh,
            true,
            false,
            225.0,
        )
        .clicked()
        {
            action = Some(Intent::Refresh);
            ui.close();
        }
        ui.separator();
        if icon_button(ui, rice, "About RiceWine", Glyph::Info, true, false, 225.0).clicked() {
            action = Some(Intent::About);
            ui.close();
        }
    });
    let rect = Rect::from_center_size(
        egui::pos2(
            result.response.rect.left() + 14.0,
            result.response.rect.center().y,
        ),
        Vec2::splat(16.0),
    );
    icon(
        ui.painter(),
        rice,
        Glyph::Tools,
        rect,
        rice.palette().accent,
    );
    action
}

fn navigation(ui: &mut Ui, rice: Theme, page: Page, can_install: bool) -> Option<Intent> {
    let mut action = None;
    let p = rice.palette();
    ui.horizontal_wrapped(|ui| {
        if icon_button(
            ui,
            rice,
            "Programs",
            Glyph::Programs,
            true,
            page == Page::Library,
            122.0,
        )
        .clicked()
        {
            action = Some(Intent::Page(Page::Library));
        }
        if let Some(intent) = tools_menu(ui, rice, can_install) {
            action = Some(intent);
        }
        if icon_button(
            ui,
            rice,
            "Settings",
            Glyph::Settings,
            true,
            page == Page::Settings,
            116.0,
        )
        .clicked()
        {
            action = Some(Intent::Page(Page::Settings));
        }
        ui.add_space(if rice == Theme::Windows98 { 3.0 } else { 9.0 });
        if icon_button(
            ui,
            rice,
            "Install",
            Glyph::Install,
            can_install,
            true,
            110.0,
        )
        .clicked()
        {
            action = Some(Intent::Install);
        }
    });
    // Active controls are never represented solely by color: the selected
    // theme's button also carries its own physical shape / border treatment.
    let _ = p;
    action
}

fn width_frame(ui: &mut Ui, width: f32, margin: i8, frame: Frame, content: impl FnOnce(&mut Ui)) {
    frame.show(ui, |ui| {
        ui.set_min_width((width - (margin as f32 * 2.0 + 2.0)).max(0.0));
        content(ui);
    });
}

fn luna_toolbar(ui: &mut Ui, page: Page, can_install: bool) -> Option<Intent> {
    let rice = Theme::Luna;
    let width = ui.available_width();
    width_frame(
        ui,
        width,
        10,
        Frame::new()
            .fill(Color32::from_rgb(35, 86, 181))
            .stroke(Stroke::new(1.0, Color32::from_rgb(21, 53, 133)))
            .corner_radius(4)
            .inner_margin(egui::Margin::symmetric(10, 5)),
        |ui| {
            brand(
                ui,
                rice,
                "RiceWine",
                Some("Windows application manager"),
                Color32::WHITE,
            )
        },
    );
    width_frame(
        ui,
        width,
        9,
        Frame::new()
            .fill(Color32::from_rgb(197, 220, 251))
            .stroke(Stroke::new(1.0, Color32::from_rgb(102, 145, 199)))
            .inner_margin(egui::Margin::symmetric(9, 5)),
        |ui| {
            ui.label(
                RichText::new("Program tasks")
                    .size(11.0)
                    .strong()
                    .color(Color32::from_rgb(21, 63, 149)),
            );
        },
    );
    navigation(ui, rice, page, can_install)
}

fn win98_toolbar(ui: &mut Ui, page: Page, can_install: bool) -> Option<Intent> {
    let rice = Theme::Windows98;
    let width = ui.available_width();
    width_frame(
        ui,
        width,
        5,
        Frame::new()
            .fill(Color32::from_rgb(0, 0, 128))
            .inner_margin(egui::Margin::symmetric(5, 3)),
        |ui| {
            brand(
                ui,
                rice,
                "RiceWine",
                Some("File   Edit   View   Help"),
                Color32::WHITE,
            )
        },
    );
    width_frame(
        ui,
        width,
        6,
        Frame::new()
            .fill(Color32::from_rgb(192, 192, 192))
            .stroke(Stroke::new(1.0, Color32::from_rgb(101, 101, 101)))
            .inner_margin(egui::Margin::symmetric(6, 4)),
        |ui| {
            ui.label(
                RichText::new("Standard Buttons")
                    .size(10.0)
                    .color(Color32::from_rgb(65, 65, 65)),
            );
        },
    );
    navigation(ui, rice, page, can_install)
}

fn win7_toolbar(ui: &mut Ui, page: Page, can_install: bool) -> Option<Intent> {
    let rice = Theme::Windows7;
    let width = ui.available_width();
    width_frame(
        ui,
        width,
        15,
        Frame::new()
            .fill(Color32::from_rgb(219, 238, 252))
            .stroke(Stroke::new(1.0, Color32::from_rgb(112, 163, 203)))
            .corner_radius(7)
            .inner_margin(egui::Margin::symmetric(15, 7)),
        |ui| {
            brand(
                ui,
                rice,
                "RiceWine",
                Some("Control Panel  ›  Programs  ›  Wine compatibility"),
                rice.palette().text,
            );
            ui.add_space(5.0);
            let r = ui.available_rect_before_wrap();
            ui.painter().line_segment(
                [r.left_top(), r.right_top()],
                Stroke::new(1.0, Color32::WHITE),
            );
        },
    );
    navigation(ui, rice, page, can_install)
}

fn mac_toolbar(ui: &mut Ui, page: Page, can_install: bool) -> Option<Intent> {
    let rice = Theme::MacOS;
    let width = ui.available_width();
    width_frame(
        ui,
        width,
        16,
        Frame::new()
            .fill(Color32::from_rgb(244, 246, 249))
            .stroke(Stroke::new(0.7, Color32::from_rgb(198, 204, 215)))
            .corner_radius(10)
            .inner_margin(egui::Margin::symmetric(16, 8)),
        |ui| {
            brand(
                ui,
                rice,
                "RiceWine",
                Some("Your Windows apps"),
                rice.palette().text,
            )
        },
    );
    ui.add_space(5.0);
    navigation(ui, rice, page, can_install)
}

fn win10_toolbar(ui: &mut Ui, page: Page, can_install: bool) -> Option<Intent> {
    let rice = Theme::Windows10;
    let width = ui.available_width();
    width_frame(
        ui,
        width,
        13,
        Frame::new()
            .fill(Color32::WHITE)
            .stroke(Stroke::new(1.0, Color32::from_rgb(226, 226, 226)))
            .inner_margin(egui::Margin::symmetric(13, 6)),
        |ui| {
            brand(
                ui,
                rice,
                "RiceWine",
                Some("Apps & compatibility"),
                rice.palette().text,
            )
        },
    );
    let line = ui.available_rect_before_wrap();
    ui.painter().line_segment(
        [line.left_top(), line.right_top()],
        Stroke::new(2.0, rice.palette().accent),
    );
    navigation(ui, rice, page, can_install)
}

fn chrome_toolbar(ui: &mut Ui, page: Page, can_install: bool) -> Option<Intent> {
    let rice = Theme::ChromeOS;
    let width = ui.available_width();
    width_frame(
        ui,
        width,
        20,
        Frame::new()
            .fill(Color32::from_rgb(250, 252, 255))
            .stroke(Stroke::new(1.0, Color32::from_rgb(219, 228, 239)))
            .corner_radius(20)
            .inner_margin(egui::Margin::symmetric(20, 10)),
        |ui| brand(ui, rice, "RiceWine", Some("Apps"), rice.palette().text),
    );
    ui.add_space(7.0);
    navigation(ui, rice, page, can_install)
}

fn frutiger_toolbar(ui: &mut Ui, page: Page, can_install: bool) -> Option<Intent> {
    let rice = Theme::FrutigerAero;
    let width = ui.available_width();
    width_frame(
        ui,
        width,
        16,
        Frame::new()
            .fill(Color32::from_rgb(185, 238, 227))
            .stroke(Stroke::new(1.0, Color32::from_rgb(82, 161, 167)))
            .corner_radius(14)
            .inner_margin(egui::Margin::symmetric(16, 10)),
        |ui| {
            brand(
                ui,
                rice,
                "RiceWine",
                Some("Your windows into new worlds"),
                rice.palette().text,
            );
            // A restrained glass glint above the Aqua content.
            let r = ui.available_rect_before_wrap();
            ui.painter().line_segment(
                [r.left_top(), r.right_top()],
                Stroke::new(1.0, Color32::from_rgb(248, 255, 251)),
            );
        },
    );
    ui.add_space(5.0);
    navigation(ui, rice, page, can_install)
}

pub fn toolbar(ui: &mut Ui, rice: Theme, page: Page, can_install: bool) -> Option<Intent> {
    match rice {
        Theme::Luna => luna_toolbar(ui, page, can_install),
        Theme::Windows98 => win98_toolbar(ui, page, can_install),
        Theme::Windows7 => win7_toolbar(ui, page, can_install),
        Theme::MacOS => mac_toolbar(ui, page, can_install),
        Theme::Windows10 => win10_toolbar(ui, page, can_install),
        Theme::ChromeOS => chrome_toolbar(ui, page, can_install),
        Theme::FrutigerAero => frutiger_toolbar(ui, page, can_install),
    }
}

pub fn page_panel(ui: &mut Ui, rice: Theme, content: impl FnOnce(&mut Ui)) {
    let p = rice.palette();
    let width = ui.available_width();
    let compact = ui.ctx().content_rect().height() < 600.0;
    let margin = match rice {
        Theme::Windows98 => 7,
        Theme::Luna => 11,
        Theme::Windows10 => 16,
        Theme::Windows7 => 17,
        Theme::MacOS => 19,
        Theme::ChromeOS => 21,
        Theme::FrutigerAero => 17,
    };
    let margin = if compact { margin.min(10) } else { margin };
    let (fill, border, thickness) = match rice {
        Theme::Windows98 => (
            Color32::from_rgb(224, 224, 224),
            Color32::from_rgb(93, 93, 93),
            1.5,
        ),
        Theme::Luna => (Color32::WHITE, Color32::from_rgb(112, 148, 194), 1.0),
        Theme::Windows7 => (
            Color32::from_rgb(251, 254, 255),
            Color32::from_rgb(164, 201, 231),
            1.0,
        ),
        Theme::MacOS => (Color32::from_rgb(252, 253, 254), p.border, 0.7),
        Theme::Windows10 => (Color32::WHITE, Color32::from_rgb(232, 232, 232), 1.0),
        Theme::ChromeOS => (Color32::WHITE, Color32::from_rgb(229, 236, 244), 1.0),
        Theme::FrutigerAero => (
            Color32::from_rgb(248, 255, 253),
            Color32::from_rgb(111, 194, 186),
            1.0,
        ),
    };
    width_frame(
        ui,
        width,
        margin,
        Frame::new()
            .fill(fill)
            .stroke(Stroke::new(thickness, border))
            .corner_radius(p.corner_radius)
            .inner_margin(egui::Margin::same(margin)),
        |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, content);
        },
    );
}

pub fn page_title(ui: &mut Ui, rice: Theme, title: &str, description: &str, glyph: Glyph) {
    let p = rice.palette();
    let compact = ui.ctx().content_rect().height() < 600.0;
    match rice {
        Theme::Luna => {
            Frame::new()
                .fill(Color32::from_rgb(227, 239, 255))
                .stroke(Stroke::new(1.0, Color32::from_rgb(152, 183, 228)))
                .corner_radius(5)
                .inner_margin(egui::Margin::symmetric(9, 7))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        inline_icon(ui, rice, glyph, 28.0, p.accent);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(title).strong().size(18.0).color(p.accent));
                            if !description.is_empty() {
                                ui.small(description);
                            }
                        });
                    });
                });
        }
        Theme::Windows98 => {
            Frame::new()
                .fill(Color32::from_rgb(192, 192, 192))
                .stroke(Stroke::new(1.0, Color32::from_rgb(104, 104, 104)))
                .inner_margin(egui::Margin::symmetric(7, 4))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        inline_icon(ui, rice, glyph, 18.0, p.accent);
                        ui.label(RichText::new(title).strong().size(17.0));
                    });
                });
            if !description.is_empty() {
                ui.small(description);
            }
        }
        Theme::Windows7 => {
            Frame::new()
                .fill(Color32::from_rgb(232, 245, 255))
                .stroke(Stroke::new(1.0, Color32::from_rgb(158, 198, 224)))
                .corner_radius(7)
                .inner_margin(egui::Margin::symmetric(13, 9))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        inline_icon(ui, rice, glyph, if compact { 23.0 } else { 30.0 }, p.accent);
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(title)
                                    .size(if compact { 19.0 } else { 22.0 })
                                    .strong()
                                    .color(p.text),
                            );
                            if !description.is_empty() {
                                ui.label(RichText::new(description).size(11.5).color(p.muted));
                            }
                        });
                    });
                });
        }
        Theme::MacOS => {
            ui.horizontal(|ui| {
                inline_icon(ui, rice, glyph, if compact { 20.0 } else { 23.0 }, p.muted);
                ui.label(
                    RichText::new(title)
                        .size(if compact { 19.0 } else { 21.0 })
                        .strong()
                        .color(p.text),
                );
            });
            if !description.is_empty() {
                ui.small(description);
            }
            ui.separator();
        }
        Theme::Windows10 => {
            ui.horizontal(|ui| {
                inline_icon(ui, rice, glyph, if compact { 20.0 } else { 23.0 }, p.accent);
                ui.label(
                    RichText::new(title)
                        .size(if compact { 21.0 } else { 25.0 })
                        .color(p.text),
                );
            });
            if !description.is_empty() {
                if !description.is_empty() {
                    ui.label(RichText::new(description).color(p.muted));
                }
            }
            ui.add_space(3.0);
        }
        Theme::ChromeOS => {
            ui.horizontal(|ui| {
                inline_icon(ui, rice, glyph, if compact { 24.0 } else { 35.0 }, p.accent);
                ui.vertical(|ui| {
                    ui.label(RichText::new(title).size(22.0).strong().color(p.text));
                    ui.label(RichText::new(description).color(p.muted));
                });
            });
        }
        Theme::FrutigerAero => {
            Frame::new()
                .fill(Color32::from_rgb(226, 250, 244))
                .stroke(Stroke::new(1.0, p.border))
                .corner_radius(12)
                .inner_margin(egui::Margin::symmetric(14, 10))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        inline_icon(ui, rice, glyph, if compact { 24.0 } else { 34.0 }, p.accent);
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(title)
                                    .size(if compact { 20.0 } else { 23.0 })
                                    .strong()
                                    .color(p.text),
                            );
                            if !description.is_empty() {
                                ui.label(RichText::new(description).size(12.0).color(p.muted));
                            }
                        });
                    });
                });
        }
    }
    ui.add_space(if compact { 3.0 } else { 8.0 });
}

fn empty_panel(
    ui: &mut Ui,
    rice: Theme,
    enabled: bool,
    subtitle: &str,
    title: &str,
    icon_glyph: Glyph,
    centered: bool,
) -> Option<EmptyIntent> {
    let p = rice.palette();
    let mut action = None;
    let compact = ui.ctx().content_rect().height() < 600.0 || ui.available_height() < 250.0;
    let max_width = ui.available_width();
    let fill = match rice {
        Theme::Windows98 => Color32::from_rgb(192, 192, 192),
        Theme::Luna => Color32::from_rgb(255, 252, 229),
        Theme::Windows7 => Color32::from_rgb(234, 246, 255),
        Theme::MacOS => Color32::from_rgb(249, 250, 252),
        Theme::Windows10 => Color32::WHITE,
        Theme::ChromeOS => Color32::from_rgb(247, 250, 255),
        Theme::FrutigerAero => Color32::from_rgb(224, 251, 243),
    };
    let border = Stroke::new(if rice == Theme::Windows98 { 2.0 } else { 1.0 }, p.border);
    let indent = if rice == Theme::Windows98 {
        8
    } else if compact {
        7
    } else {
        14
    };
    let mut body = |ui: &mut Ui| {
        let mut body = |ui: &mut Ui| {
            inline_icon(
                ui,
                rice,
                icon_glyph,
                if compact { 22.0 } else { 40.0 },
                p.accent,
            );
            ui.add_space(if compact { 1.0 } else { 3.0 });
            ui.label(
                RichText::new(title)
                    .size(if compact {
                        18.0
                    } else {
                        match rice {
                            Theme::Windows98 => 17.0,
                            Theme::Luna => 19.0,
                            Theme::ChromeOS | Theme::FrutigerAero => 22.0,
                            _ => 20.0,
                        }
                    })
                    .strong()
                    .color(p.text),
            );
            ui.label(RichText::new(subtitle).size(12.0).color(p.muted));
            ui.add_space(if compact { 3.0 } else { 14.0 });
            ui.horizontal_wrapped(|ui| {
                if icon_button(
                    ui,
                    rice,
                    "Install a program",
                    Glyph::Install,
                    enabled,
                    true,
                    188.0,
                )
                .clicked()
                {
                    action = Some(EmptyIntent::Install);
                }
                if icon_button(
                    ui,
                    rice,
                    "Import prefix",
                    Glyph::Import,
                    enabled,
                    false,
                    152.0,
                )
                .clicked()
                {
                    action = Some(EmptyIntent::Import);
                }
            });
        };
        if centered {
            ui.vertical_centered(body);
        } else {
            body(ui);
        }
    };
    match rice {
        Theme::Windows98 => {
            ui.add_space(8.0);
            Frame::new()
                .fill(fill)
                .stroke(border)
                .inner_margin(egui::Margin::same(indent))
                .show(ui, body);
        }
        Theme::Luna => {
            Frame::new()
                .fill(fill)
                .stroke(border)
                .corner_radius(7)
                .inner_margin(egui::Margin::same(indent))
                .show(ui, body);
        }
        Theme::Windows10 => {
            ui.add_space(12.0);
            body(ui);
        }
        Theme::MacOS | Theme::Windows7 | Theme::ChromeOS | Theme::FrutigerAero => {
            ui.add_space(if compact { 3.0 } else { 10.0 });
            Frame::new()
                .fill(fill)
                .stroke(border)
                .corner_radius(p.corner_radius)
                .inner_margin(egui::Margin::same(indent))
                .show(ui, |ui| {
                    ui.set_min_width((max_width - (indent * 2 + 2) as f32).max(0.0));
                    body(ui);
                });
        }
    }
    action
}

pub fn empty_library(ui: &mut Ui, rice: Theme, enabled: bool) -> Option<EmptyIntent> {
    match rice {
        Theme::Luna => empty_panel(
            ui,
            rice,
            enabled,
            "Choose a Windows setup file or import an existing environment.",
            "What do you want to do?",
            Glyph::Install,
            false,
        ),
        Theme::Windows98 => empty_panel(
            ui,
            rice,
            enabled,
            "Choose a setup program to add an item to this list.",
            "This folder is empty",
            Glyph::Programs,
            false,
        ),
        Theme::Windows7 => empty_panel(
            ui,
            rice,
            enabled,
            "Add a Windows program to your library.",
            "Get started with RiceWine",
            Glyph::Programs,
            true,
        ),
        Theme::MacOS => empty_panel(
            ui,
            rice,
            enabled,
            "Your Windows apps appear here once installed.",
            "No applications",
            Glyph::Programs,
            true,
        ),
        Theme::Windows10 => empty_panel(
            ui,
            rice,
            enabled,
            "Install an EXE or MSI to see it in your library.",
            "Your app list is empty",
            Glyph::Programs,
            false,
        ),
        Theme::ChromeOS => empty_panel(
            ui,
            rice,
            enabled,
            "Install or import a Windows app to get started.",
            "All your Windows apps, here",
            Glyph::Programs,
            true,
        ),
        Theme::FrutigerAero => empty_panel(
            ui,
            rice,
            enabled,
            "Bring your Windows applications into this workspace.",
            "Explore a new world of apps",
            Glyph::Brand,
            true,
        ),
    }
}

/// Era-specific grouping for the Prefix/Backup tools. The visual shell owns
/// its surfaces, while the caller owns the potentially destructive actions.
pub fn group_box(
    ui: &mut Ui,
    rice: Theme,
    title: &str,
    glyph: Glyph,
    content: impl FnOnce(&mut Ui),
) {
    let p = rice.palette();
    let (fill, border, radius) = match rice {
        Theme::Windows98 => (
            Color32::from_rgb(192, 192, 192),
            Color32::from_rgb(100, 100, 100),
            0,
        ),
        Theme::Luna => (
            Color32::from_rgb(248, 250, 255),
            Color32::from_rgb(141, 172, 219),
            6,
        ),
        Theme::Windows7 => (
            Color32::from_rgb(241, 249, 255),
            Color32::from_rgb(162, 203, 233),
            8,
        ),
        Theme::MacOS => (
            Color32::from_rgb(247, 248, 250),
            Color32::from_rgb(216, 219, 227),
            9,
        ),
        Theme::Windows10 => (
            Color32::from_rgb(248, 248, 248),
            Color32::from_rgb(220, 220, 220),
            0,
        ),
        Theme::ChromeOS => (
            Color32::from_rgb(244, 248, 254),
            Color32::from_rgb(221, 230, 242),
            16,
        ),
        Theme::FrutigerAero => (
            Color32::from_rgb(221, 249, 242),
            Color32::from_rgb(120, 196, 187),
            12,
        ),
    };
    Frame::new()
        .fill(fill)
        .stroke(Stroke::new(1.0, border))
        .corner_radius(radius)
        .inner_margin(egui::Margin::same(if rice == Theme::Windows98 {
            7
        } else {
            12
        }))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                inline_icon(
                    ui,
                    rice,
                    glyph,
                    if rice == Theme::Windows98 { 18.0 } else { 24.0 },
                    p.accent,
                );
                ui.label(RichText::new(title).strong().size(15.0).color(p.text));
            });
            ui.add_space(6.0);
            content(ui);
        });
}

pub fn diagnostics_panel(ui: &mut Ui, rice: Theme, lines: &[String]) {
    group_box(ui, rice, "Runtime status", Glyph::Info, |ui| {
        if lines.is_empty() {
            ui.small("No runtime information available.");
        }
        for line in lines {
            ui.label(RichText::new(line).size(11.5).color(rice.palette().muted));
        }
    });
}

pub fn program_row(
    ui: &mut Ui,
    rice: Theme,
    name: &str,
    status: &str,
    selected: bool,
    width: f32,
    row_height: f32,
) -> Response {
    let p = rice.palette();
    let response = ui.add_sized(
        [width.max(90.0), (row_height - 4.0).max(26.0)],
        egui::Button::new(
            RichText::new(format!("     {name}"))
                .color(if selected { Color32::WHITE } else { p.text })
                .strong(),
        )
        .fill(if selected {
            p.accent
        } else {
            match rice {
                Theme::Windows98 => Color32::from_rgb(192, 192, 192),
                Theme::Luna => Color32::from_rgb(234, 243, 255),
                Theme::Windows7 => Color32::from_rgb(240, 248, 254),
                Theme::MacOS => Color32::from_rgb(249, 250, 252),
                Theme::Windows10 => Color32::WHITE,
                Theme::ChromeOS => Color32::from_rgb(240, 245, 255),
                Theme::FrutigerAero => Color32::from_rgb(229, 250, 244),
            }
        })
        .stroke(Stroke::new(
            if selected || matches!(rice, Theme::Windows98 | Theme::Luna) {
                1.0
            } else {
                0.6
            },
            p.border,
        ))
        .corner_radius(p.corner_radius),
    );
    let symbol = if status == "Running" {
        Glyph::Play
    } else if status == "Check runner" {
        Glyph::Warning
    } else if name.starts_with('★') {
        Glyph::Favorite
    } else {
        Glyph::Programs
    };
    let rect = Rect::from_center_size(
        egui::pos2(response.rect.left() + 19.0, response.rect.center().y),
        Vec2::splat(19.0),
    );
    icon(
        ui.painter(),
        rice,
        symbol,
        rect,
        if selected { Color32::WHITE } else { p.accent },
    );
    response
}

pub fn dialog_banner(ui: &mut Ui, rice: Theme, title: &str, subtitle: &str, glyph: Glyph) {
    page_title(ui, rice, title, subtitle, glyph);
}

pub fn setting_preview(ui: &mut Ui, rice: Theme) {
    let p = rice.palette();
    let compact = ui.ctx().content_rect().height() < 600.0;
    ui.add_space(6.0);
    Frame::new()
        .fill(p.sidebar)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(p.corner_radius)
        .inner_margin(egui::Margin::same(9))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                inline_icon(ui, rice, Glyph::Brand, 24.0, p.accent);
                ui.label(RichText::new(rice.label()).strong().color(p.text));
                if !compact {
                    ui.label(
                        RichText::new(match rice {
                            Theme::Luna => "Task panes • bright XP controls",
                            Theme::Windows98 => "Square buttons • compact Win9x",
                            Theme::Windows7 => "Aero glass • blue explorer details",
                            Theme::MacOS => "Quiet utility toolbar • precise lines",
                            Theme::Windows10 => "Flat Fluent • crisp geometry",
                            Theme::ChromeOS => "Material shapes • open spacing",
                            Theme::FrutigerAero => "Aqua glass • organic highlights",
                        })
                        .size(11.0)
                        .color(p.muted),
                    );
                }
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_rice_has_distinct_icon_family_and_palette() {
        let themes = [
            Theme::Luna,
            Theme::Windows98,
            Theme::Windows7,
            Theme::MacOS,
            Theme::Windows10,
            Theme::ChromeOS,
            Theme::FrutigerAero,
        ];
        for (i, first) in themes.iter().enumerate() {
            for second in themes.iter().skip(i + 1) {
                assert_ne!(family(*first), family(*second));
                assert_ne!(first.palette().sidebar, second.palette().sidebar);
            }
        }
    }

    #[test]
    fn all_rices_paint_icons_and_empty_states_without_panics() {
        let ctx = egui::Context::default();
        for rice in [
            Theme::Luna,
            Theme::Windows98,
            Theme::Windows7,
            Theme::MacOS,
            Theme::Windows10,
            Theme::ChromeOS,
            Theme::FrutigerAero,
        ] {
            let mut out = ctx.run_ui(Default::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    inline_icon(ui, rice, Glyph::Brand, 28.0, rice.palette().accent);
                    let _ = empty_library(ui, rice, false);
                });
            });
            assert!(!out.shapes.is_empty(), "{rice:?}");
            out.textures_delta.clear();
        }
    }
}
