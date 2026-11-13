//! RiceBowl-inspired shells for the four additional maintained desktop families.
//! All actions are owned by RiceWineApp; this module only renders visual chrome.
use eframe::egui::{self, Color32, Response, RichText};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flavor {
    Classic98,
    Aero7,
    ChromeOS,
    FrutigerAero,
}

impl Flavor {
    pub fn title(self) -> &'static str {
        match self {
            Self::Classic98 => "Windows 98",
            Self::Aero7 => "Windows 7 Aero",
            Self::ChromeOS => "ChromeOS",
            Self::FrutigerAero => "Frutiger Aero",
        }
    }

    fn surface(self) -> Color32 {
        match self {
            Self::Classic98 => Color32::from_rgb(192, 192, 192),
            Self::Aero7 => Color32::from_rgb(210, 230, 245),
            Self::ChromeOS => Color32::from_rgb(247, 249, 253),
            Self::FrutigerAero => Color32::from_rgb(184, 232, 225),
        }
    }

    fn canvas(self) -> Color32 {
        match self {
            Self::Classic98 => Color32::from_rgb(224, 224, 224),
            Self::Aero7 => Color32::from_rgb(248, 252, 255),
            Self::ChromeOS => Color32::WHITE,
            Self::FrutigerAero => Color32::from_rgb(246, 255, 253),
        }
    }

    fn foreground(self) -> Color32 {
        match self {
            Self::Classic98 => Color32::from_rgb(28, 28, 28),
            Self::Aero7 => Color32::from_rgb(25, 48, 71),
            Self::ChromeOS => Color32::from_rgb(32, 43, 59),
            Self::FrutigerAero => Color32::from_rgb(6, 56, 64),
        }
    }

    fn accent(self) -> Color32 {
        match self {
            Self::Classic98 => Color32::from_rgb(0, 51, 153),
            Self::Aero7 => Color32::from_rgb(23, 92, 152),
            Self::ChromeOS => Color32::from_rgb(24, 86, 166),
            Self::FrutigerAero => Color32::from_rgb(9, 99, 119),
        }
    }

    fn radius(self) -> u8 {
        match self {
            Self::Classic98 => 0,
            Self::Aero7 => 6,
            Self::ChromeOS => 14,
            Self::FrutigerAero => 12,
        }
    }

    fn stroke(self) -> egui::Stroke {
        match self {
            Self::Classic98 => egui::Stroke::new(1.0, Color32::from_rgb(110, 110, 110)),
            Self::Aero7 => egui::Stroke::new(1.0, Color32::from_rgb(149, 187, 218)),
            Self::ChromeOS => egui::Stroke::new(1.0, Color32::from_rgb(213, 222, 233)),
            Self::FrutigerAero => egui::Stroke::new(1.0, Color32::from_rgb(126, 192, 190)),
        }
    }
}

/// Keep recognizable desktop language, not one generic header with four palettes.
/// Header text deliberately describes the current page; it does not impersonate
/// native window chrome (that stays the Linux compositor's responsibility).
pub fn masthead(ui: &mut egui::Ui, rice: Flavor, page: &str) {
    let margin = if rice == Flavor::Classic98 { 6 } else { 14 };
    let header_width = (ui.available_width() - (margin * 2 + 2) as f32).max(0.0);
    egui::Frame::new()
        .fill(rice.surface())
        .stroke(rice.stroke())
        .corner_radius(egui::CornerRadius::same(rice.radius()))
        .inner_margin(egui::Margin::symmetric(margin, 10))
        .show(ui, |ui| {
            ui.set_min_width(header_width);
            match rice {
                Flavor::Classic98 => {
                    // Compact, rectangular Win9x application header. The actual
                    // File/View/Help menu lives in the application shell above it.
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("▣").size(18.0).color(rice.accent()));
                        ui.label(
                            RichText::new("RiceWine")
                                .size(16.0)
                                .strong()
                                .color(rice.foreground()),
                        );
                        ui.separator();
                        ui.label(RichText::new(page).size(13.0).color(rice.foreground()));
                    });
                }
                Flavor::Aero7 => {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new(format!("Control Panel  ›  Programs  ›  {page}"))
                                .size(11.0)
                                .color(rice.accent()),
                        );
                    });
                    ui.add_space(7.0);
                    ui.label(
                        RichText::new("RiceWine")
                            .size(23.0)
                            .strong()
                            .color(rice.foreground()),
                    );
                    ui.small("Manage Windows programs and compatibility settings");
                }
                Flavor::ChromeOS => {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("●").size(20.0).color(rice.accent()));
                        ui.label(
                            RichText::new("RiceWine")
                                .size(20.0)
                                .strong()
                                .color(rice.foreground()),
                        );
                        ui.separator();
                        ui.label(RichText::new(page).size(14.0).color(rice.foreground()));
                    });
                }
                Flavor::FrutigerAero => {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new("◉").size(30.0).color(rice.accent()));
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new("RiceWine")
                                    .size(24.0)
                                    .strong()
                                    .color(rice.foreground()),
                            );
                            ui.label(
                                RichText::new("Windows on Linux • your personal workspace")
                                    .size(11.0)
                                    .color(rice.accent()),
                            );
                        });
                    });
                    ui.add_space(5.0);
                    ui.label(
                        RichText::new(page)
                            .size(13.0)
                            .strong()
                            .color(rice.foreground()),
                    );
                }
            }
        });
}

/// Navigation retains genuine button hit targets and a clearly visible selected
/// state. Win7 uses a light selection rail rather than the dark buttons of 98.
pub fn navigation(
    ui: &mut egui::Ui,
    rice: Flavor,
    label: &str,
    selected: bool,
    horizontal: bool,
) -> Response {
    let text_color = if selected && rice != Flavor::Aero7 {
        Color32::WHITE
    } else {
        rice.foreground()
    };
    let fill = match (rice, selected) {
        (Flavor::Classic98, true) => rice.accent(),
        (Flavor::Classic98, false) => rice.surface(),
        (Flavor::Aero7, true) => Color32::from_rgb(202, 230, 251),
        (Flavor::Aero7, false) => Color32::from_rgb(237, 247, 253),
        (Flavor::ChromeOS, true) | (Flavor::FrutigerAero, true) => rice.accent(),
        (Flavor::ChromeOS, false) => Color32::from_rgb(239, 244, 252),
        (Flavor::FrutigerAero, false) => Color32::from_rgb(224, 249, 244),
    };
    let corner_radius = if rice == Flavor::Aero7 { 3 } else { rice.radius() };
    let button = egui::Button::new(
        RichText::new(label)
            .color(text_color)
            .strong(),
    )
    .fill(fill)
    .stroke(if selected {
        egui::Stroke::new(1.0, rice.accent())
    } else {
        rice.stroke()
    })
    .corner_radius(egui::CornerRadius::same(corner_radius));
    ui.add_sized(
        if horizontal {
            [ui.available_width().min(120.0), 33.0]
        } else {
            [ui.available_width().min(180.0), 36.0]
        },
        button,
    )
}

/// A consistent content frame around the existing functional page, without a second app state.
pub fn content_frame(rice: Flavor) -> egui::Frame {
    egui::Frame::new()
        .fill(rice.canvas())
        .stroke(rice.stroke())
        .inner_margin(if rice == Flavor::Classic98 {
            egui::Margin::same(12)
        } else {
            egui::Margin::same(18)
        })
        .corner_radius(egui::CornerRadius::same(rice.radius()))
}

pub fn page_heading(ui: &mut egui::Ui, rice: Flavor, title: &str) {
    ui.label(
        RichText::new(title)
            .size(match rice {
                Flavor::Classic98 => 17.0,
                Flavor::Aero7 => 23.0,
                Flavor::ChromeOS => 22.0,
                Flavor::FrutigerAero => 24.0,
            })
            .strong()
            .color(rice.foreground()),
    );
    if rice == Flavor::Classic98 {
        ui.separator();
    } else {
        ui.add_space(4.0);
    }
}

pub fn status(ui: &mut egui::Ui, rice: Flavor, count: usize) {
    if rice == Flavor::Classic98 {
        egui::Frame::new()
            .fill(rice.surface())
            .stroke(rice.stroke())
            .inner_margin(egui::Margin::symmetric(8, 3))
            .show(ui, |ui| {
                ui.small(format!("{count} program(s)  |  RiceWine"));
            });
    } else {
        ui.horizontal(|ui| {
            ui.small(format!("{count} registered Windows programs"));
            ui.separator();
            ui.small(rice.title());
        });
    }
}
