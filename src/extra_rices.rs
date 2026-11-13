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
pub fn masthead(ui: &mut egui::Ui, rice: Flavor, page: &str) -> bool {
    let mut go_home = false;
    let margin = if rice == Flavor::Classic98 { 6 } else { 14 };
    let header_width = (ui.available_width() - (margin * 2 + 2) as f32).max(0.0);
    egui::Frame::new()
        .fill(rice.surface())
        .stroke(rice.stroke())
        .corner_radius(egui::CornerRadius::same(rice.radius()))
        .inner_margin(egui::Margin::symmetric(margin, 10))
        .show(ui, |ui| {
            ui.set_min_width(header_width);
            if rice == Flavor::FrutigerAero {
                // Small, original 2007-era aqua highlight behind the masthead text.
                let area = ui.available_rect_before_wrap();
                let highlight = egui::Rect::from_min_size(area.min, egui::vec2(area.width(), 18.0));
                ui.painter().rect_filled(
                    highlight,
                    egui::CornerRadius::same(5),
                    Color32::from_rgb(222, 255, 250),
                );
                ui.painter().line_segment(
                    [highlight.left_bottom(), highlight.right_bottom()],
                    egui::Stroke::new(1.0, Color32::from_rgb(128, 205, 202)),
                );
            }
            match rice {
                Flavor::Classic98 => {
                    // Compact, rectangular Win9x application header. The actual
                    // File/View/Help menu lives in the application shell above it.
                    ui.horizontal_wrapped(|ui| {
                        crate::rice_identity::symbol(
                            ui,
                            crate::rice_identity::Family::Win98,
                            crate::rice_identity::Icon::Brand,
                            rice.accent(),
                            16.0,
                        );
                        ui.label(
                            RichText::new("RiceWine")
                                .size(12.0)
                                .strong()
                                .color(rice.foreground()),
                        );
                        ui.separator();
                        ui.label(
                            RichText::new(format!("{page} Properties"))
                                .size(12.0)
                                .color(rice.foreground()),
                        );
                    });
                }
                Flavor::Aero7 => {
                    ui.horizontal_wrapped(|ui| {
                        if ui.link("Control Panel").clicked() {
                            go_home = true;
                        }
                        ui.label(
                            RichText::new(format!(" ›  Programs  ›  {page}"))
                                .size(11.0)
                                .color(rice.accent()),
                        );
                    });
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Programs and Features")
                            .size(21.0)
                            .color(rice.foreground()),
                    );
                }
                Flavor::ChromeOS => {
                    ui.horizontal_wrapped(|ui| {
                        crate::rice_identity::symbol(
                            ui,
                            crate::rice_identity::Family::Chrome,
                            crate::rice_identity::Icon::Brand,
                            rice.accent(),
                            22.0,
                        );
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
                        crate::rice_identity::symbol(
                            ui,
                            crate::rice_identity::Family::Frutiger,
                            crate::rice_identity::Icon::Brand,
                            rice.accent(),
                            33.0,
                        );
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
    go_home
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
    if rice == Flavor::Aero7 {
        let response = ui.add_sized(
            if horizontal {
                [120.0, 30.0]
            } else {
                [ui.available_width().min(180.0), 30.0]
            },
            egui::Button::new(RichText::new(label).size(12.5).color(rice.accent()))
                .fill(if selected {
                    Color32::from_rgb(211, 233, 249)
                } else {
                    Color32::TRANSPARENT
                })
                .stroke(egui::Stroke::NONE)
                .corner_radius(egui::CornerRadius::same(2)),
        );
        if selected {
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    response.rect.min,
                    egui::vec2(3.0, response.rect.height()),
                ),
                0.0,
                rice.accent(),
            );
        }
        return response;
    }
    let text_color = if selected && matches!(rice, Flavor::ChromeOS | Flavor::FrutigerAero) {
        Color32::WHITE
    } else {
        rice.foreground()
    };
    let fill = match (rice, selected) {
        (Flavor::Classic98, true) => Color32::from_rgb(240, 240, 240),
        (Flavor::Classic98, false) => rice.surface(),
        (Flavor::Aero7, true) => Color32::from_rgb(202, 230, 251),
        (Flavor::Aero7, false) => Color32::from_rgb(237, 247, 253),
        (Flavor::ChromeOS, true) | (Flavor::FrutigerAero, true) => rice.accent(),
        (Flavor::ChromeOS, false) => Color32::from_rgb(239, 244, 252),
        (Flavor::FrutigerAero, false) => Color32::from_rgb(224, 249, 244),
    };
    let corner_radius = if rice == Flavor::Aero7 {
        3
    } else {
        rice.radius()
    };
    let button = egui::Button::new(RichText::new(label).color(text_color).strong())
        .fill(fill)
        .stroke(if selected && rice != Flavor::Classic98 {
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
                Flavor::Classic98 => 13.0,
                Flavor::Aero7 => 16.0,
                Flavor::ChromeOS => 20.0,
                Flavor::FrutigerAero => 21.0,
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

/// Four OS-specific library row styles. The Wine operation is owned by the
/// caller; rows are selectable, keyboard-focusable controls rather than art.
pub fn program_row(
    ui: &mut egui::Ui,
    rice: Flavor,
    name: &str,
    status: &str,
    selected: bool,
    width: f32,
) -> Response {
    let height = match rice {
        Flavor::Classic98 => 26.0,
        Flavor::Aero7 => 31.0,
        Flavor::ChromeOS => 46.0,
        Flavor::FrutigerAero => 42.0,
    };
    let fill = match (rice, selected) {
        (Flavor::Classic98, true) => Color32::from_rgb(0, 0, 128),
        (Flavor::Classic98, false) => Color32::WHITE,
        (Flavor::Aero7, true) => Color32::from_rgb(199, 229, 249),
        (Flavor::Aero7, false) => Color32::WHITE,
        (Flavor::ChromeOS, true) => Color32::from_rgb(221, 235, 251),
        (Flavor::ChromeOS, false) => Color32::WHITE,
        (Flavor::FrutigerAero, true) => Color32::from_rgb(182, 234, 222),
        (Flavor::FrutigerAero, false) => Color32::from_rgb(250, 255, 254),
    };
    let fg = if rice == Flavor::Classic98 && selected {
        Color32::WHITE
    } else {
        rice.foreground()
    };
    let button = egui::Button::new(
        RichText::new(format!("      {name}"))
            .size(if rice == Flavor::Classic98 {
                12.0
            } else {
                13.0
            })
            .color(fg),
    )
    .fill(fill)
    .stroke(if rice == Flavor::Classic98 {
        egui::Stroke::NONE
    } else {
        rice.stroke()
    })
    .corner_radius(egui::CornerRadius::same(if rice == Flavor::Classic98 {
        0
    } else if rice == Flavor::Aero7 {
        2
    } else {
        rice.radius()
    }));
    let response = ui.add_sized([width.max(0.0), height], button);
    let icon_size = if rice == Flavor::Classic98 {
        16.0
    } else {
        22.0
    };
    let rect = egui::Rect::from_center_size(
        response.rect.left_center() + egui::vec2(15.0, 0.0),
        egui::vec2(icon_size, icon_size),
    );
    let family = match rice {
        Flavor::Classic98 => crate::rice_identity::Family::Win98,
        Flavor::Aero7 => crate::rice_identity::Family::Aero7,
        Flavor::ChromeOS => crate::rice_identity::Family::Chrome,
        Flavor::FrutigerAero => crate::rice_identity::Family::Frutiger,
    };
    crate::rice_identity::paint(
        ui.painter(),
        rect,
        family,
        crate::rice_identity::Icon::Programs,
        rice.accent(),
    );
    response.on_hover_text(format!("{name} · {status}"))
}

/// Era-specific empty library layout; installation never starts without user confirmation.
pub fn empty_library(ui: &mut egui::Ui, rice: Flavor, enabled: bool) -> bool {
    let mut install = false;
    let (title, body, action) = match rice {
        Flavor::Classic98 => (
            "No installed programs",
            "Choose Install to add a Windows program to this computer.",
            "Install…",
        ),
        Flavor::Aero7 => (
            "No programs are registered",
            "You can install a Windows program or import an existing Wine environment.",
            "Install a program",
        ),
        Flavor::ChromeOS => (
            "Your apps will appear here",
            "Choose an EXE or MSI to add your first Windows app.",
            "Add app",
        ),
        Flavor::FrutigerAero => (
            "Welcome to your programs",
            "Bring your Windows applications together in one place.",
            "Add a program",
        ),
    };
    egui::Frame::new()
        .fill(rice.canvas())
        .stroke(rice.stroke())
        .corner_radius(egui::CornerRadius::same(if rice == Flavor::Classic98 {
            0
        } else {
            rice.radius()
        }))
        .inner_margin(if rice == Flavor::Classic98 {
            egui::Margin::same(12)
        } else {
            egui::Margin::same(20)
        })
        .show(ui, |ui| {
            ui.label(
                RichText::new(title)
                    .strong()
                    .size(if rice == Flavor::Classic98 {
                        13.0
                    } else {
                        18.0
                    })
                    .color(rice.foreground()),
            );
            ui.add_space(8.0);
            ui.label(body);
            ui.add_space(14.0);
            install = ui
                .add_enabled(
                    enabled,
                    egui::Button::new(RichText::new(action).color(if rice == Flavor::Classic98 {
                        rice.foreground()
                    } else {
                        Color32::WHITE
                    }))
                    .fill(if rice == Flavor::Classic98 {
                        rice.surface()
                    } else {
                        rice.accent()
                    })
                    .corner_radius(egui::CornerRadius::same(rice.radius())),
                )
                .clicked();
        });
    install
}

/// Compact application-managed dialog header, leaving the real window frame native.
pub fn dialog_intro(ui: &mut egui::Ui, rice: Flavor, title: &str, description: &str) {
    if rice == Flavor::Classic98 {
        ui.label(RichText::new(title).size(13.0).strong());
        ui.separator();
        ui.small(description);
    } else {
        egui::Frame::new()
            .fill(rice.surface())
            .stroke(rice.stroke())
            .corner_radius(egui::CornerRadius::same(rice.radius()))
            .inner_margin(egui::Margin::symmetric(10, 9))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(title)
                            .size(17.0)
                            .strong()
                            .color(rice.foreground()),
                    );
                    ui.small(description);
                });
            });
    }
    ui.add_space(8.0);
}

/// Period-specific command bar control; action ownership remains in RiceWineApp.
pub fn toolbar_action(
    ui: &mut egui::Ui,
    rice: Flavor,
    title: &str,
    enabled: bool,
    primary: bool,
) -> Response {
    let text = if rice == Flavor::Classic98 {
        RichText::new(title).size(12.0).color(rice.foreground())
    } else if rice == Flavor::Aero7 {
        RichText::new(title).size(12.5).color(rice.accent())
    } else if primary {
        RichText::new(title).size(13.0).color(Color32::WHITE)
    } else {
        RichText::new(title).size(13.0).color(rice.foreground())
    };
    let fill = match rice {
        Flavor::Classic98 => rice.surface(),
        Flavor::Aero7 => Color32::from_rgb(237, 247, 253),
        Flavor::ChromeOS | Flavor::FrutigerAero if primary => rice.accent(),
        _ => rice.surface(),
    };
    ui.add_enabled(
        enabled,
        egui::Button::new(text)
            .fill(fill)
            .stroke(rice.stroke())
            .corner_radius(egui::CornerRadius::same(if rice == Flavor::Aero7 {
                2
            } else {
                rice.radius()
            }))
            .min_size(egui::vec2(
                0.0,
                if rice == Flavor::Classic98 {
                    25.0
                } else {
                    31.0
                },
            )),
    )
}
