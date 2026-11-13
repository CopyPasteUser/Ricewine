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

/// Header layout deliberately differs per rice, while the Linux window frame stays native.
pub fn masthead(ui: &mut egui::Ui, rice: Flavor, page: &str) {
    let margin = if rice == Flavor::Classic98 { 6 } else { 14 };
    egui::Frame::new()
        .fill(rice.surface())
        .stroke(rice.stroke())
        .corner_radius(egui::CornerRadius::same(rice.radius()))
        .inner_margin(egui::Margin::symmetric(margin, 10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let emblem = match rice {
                    Flavor::Classic98 => "▣",
                    Flavor::Aero7 => "◈",
                    Flavor::ChromeOS => "●",
                    Flavor::FrutigerAero => "◉",
                };
                ui.label(RichText::new(emblem).size(22.0).color(rice.accent()));
                ui.label(
                    RichText::new("RiceWine")
                        .size(if rice == Flavor::Classic98 {
                            16.0
                        } else {
                            21.0
                        })
                        .strong()
                        .color(rice.foreground()),
                );
                ui.separator();
                ui.label(RichText::new(page).color(rice.foreground()));
            });
            match rice {
                Flavor::Classic98 => {}
                Flavor::Aero7 => {
                    ui.small("Control Panel  ›  Programs  ›  Wine & Proton");
                }
                Flavor::ChromeOS => {
                    ui.small("Your Windows apps, neatly in one place");
                }
                Flavor::FrutigerAero => {
                    ui.label(
                        RichText::new("Your Windows applications, beautifully organized")
                            .italics()
                            .size(12.0)
                            .color(rice.foreground()),
                    );
                }
            }
        });
}

pub fn navigation(
    ui: &mut egui::Ui,
    rice: Flavor,
    label: &str,
    selected: bool,
    horizontal: bool,
) -> Response {
    let text = if selected {
        RichText::new(label).color(Color32::WHITE).strong()
    } else {
        RichText::new(label).color(rice.foreground())
    };
    let fill = if selected {
        rice.accent()
    } else if rice == Flavor::Classic98 {
        rice.surface()
    } else {
        rice.canvas()
    };
    let button = egui::Button::new(text)
        .fill(fill)
        .stroke(if selected {
            egui::Stroke::new(1.0, rice.accent())
        } else {
            rice.stroke()
        })
        .corner_radius(egui::CornerRadius::same(rice.radius()));
    ui.add_sized(
        if horizontal {
            [120.0, 33.0]
        } else {
            [180.0, 36.0]
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
