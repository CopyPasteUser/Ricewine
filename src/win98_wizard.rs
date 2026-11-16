//! Windows 98-only, reversible multi-page setup/import UI.
//! The app owns the actual Wine/Proton operations and confirmation boundaries.
//! This module only edits drafts and returns an explicit intent on Finish.

use crate::win98;
use crate::{ImportDraft, InstallDraft, RiceWineApp};
use eframe::egui::{self, Color32, RichText};
use ricewine::core;
use ricewine::runtime::Template;
use std::path::Path;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum Step {
    #[default]
    Details,
    Compatibility,
    Review,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Action {
    None,
    Cancel,
    Finish,
}

pub(crate) fn install_details_ready(draft: &InstallDraft) -> bool {
    !draft.file.trim().is_empty() && !draft.name.trim().is_empty()
}

pub(crate) fn install_compatibility_ready(draft: &InstallDraft) -> bool {
    match draft.runner_kind {
        0 => true,
        1 => !draft.custom_wine.trim().is_empty(),
        2 => !draft.proton_dir.trim().is_empty(),
        _ => false,
    }
}

pub(crate) fn import_details_ready(draft: &ImportDraft) -> bool {
    !draft.name.trim().is_empty()
        && !draft.prefix.trim().is_empty()
        && !draft.executable.trim().is_empty()
}

fn classic_group(ui: &mut egui::Ui, title: &str, content: impl FnOnce(&mut egui::Ui)) {
    ui.label(RichText::new(title).strong().size(12.0).color(win98::INK));
    egui::Frame::new()
        .fill(win98::FACE)
        .stroke(egui::Stroke::new(1.0, win98::SHADOW))
        .inner_margin(egui::Margin::same(9))
        .show(ui, content);
}

fn header(ui: &mut egui::Ui, title: &str, step: Step, import: bool) {
    let (index, count, caption) = if import {
        match step {
            Step::Details => (1, 2, "Locate an existing Wine installation"),
            Step::Compatibility | Step::Review => (2, 2, "Review imported program"),
        }
    } else {
        match step {
            Step::Details => (1, 3, "Choose the Windows setup program"),
            Step::Compatibility => (2, 3, "Configure Wine compatibility"),
            Step::Review => (3, 3, "Confirm installation"),
        }
    };
    egui::Frame::new()
        .fill(Color32::WHITE)
        .stroke(egui::Stroke::new(1.0, win98::SHADOW))
        .inner_margin(egui::Margin::symmetric(10, 7))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                crate::rice_identity::symbol(
                    ui,
                    crate::rice_identity::Family::Win98,
                    if import {
                        crate::rice_identity::Icon::Import
                    } else {
                        crate::rice_identity::Icon::Install
                    },
                    win98::NAVY,
                    27.0,
                );
                ui.vertical(|ui| {
                    ui.label(RichText::new(title).strong().size(13.0).color(win98::INK));
                    ui.label(RichText::new(format!("Step {index} of {count}: {caption}"))
                        .size(11.0).color(win98::INK));
                });
            });
        });
}

fn field_width(ui: &egui::Ui) -> f32 {
    (ui.available_width() - 92.0).clamp(120.0, 370.0)
}

fn detail(ui: &mut egui::Ui, name: &str, value: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(format!("{name}:")).strong().color(win98::INK));
        ui.label(RichText::new(value).color(win98::INK));
    });
}

fn footer(ui: &mut egui::Ui, step: &mut Step, back: bool, next: bool,
          is_last: bool, finish: &str, finish_enabled: bool) -> Action {
    let mut action = Action::None;
    ui.separator();
    ui.horizontal_wrapped(|ui| {
        if win98::push_button(ui, "< Back", back).clicked() {
            *step = match *step {
                Step::Details => Step::Details,
                Step::Compatibility => Step::Details,
                Step::Review => if is_last && finish == "Import program" {
                    Step::Details
                } else {
                    Step::Compatibility
                },
            };
        }
        if !is_last {
            if win98::push_button(ui, "Next >", next).clicked() {
                *step = match *step {
                    Step::Details => if finish == "Import program" {
                        Step::Review
                    } else {
                        Step::Compatibility
                    },
                    Step::Compatibility => Step::Review,
                    Step::Review => Step::Review,
                };
            }
        } else if win98::push_button(ui, finish, finish_enabled).clicked() {
            action = Action::Finish;
        }
        if win98::push_button(ui, "Cancel", true).clicked() {
            action = Action::Cancel;
        }
    });
    action
}

/// Windows 98 controls only. No setup process starts until Finish is returned
/// and the caller executes the existing validated install_async operation.
pub(crate) fn install(ui: &mut egui::Ui, draft: &mut InstallDraft) -> Action {
    header(ui, "Add New Programs", draft.win98_step, false);
    ui.add_space(5.0);
    let area_height = (ui.ctx().content_rect().height() - 188.0).max(100.0);
    egui::ScrollArea::vertical()
        .id_salt("win98-install-steps")
        .max_height(area_height)
        .auto_shrink([false, false])
        .show(ui, |ui| match draft.win98_step {
            Step::Details => {
                classic_group(ui, "Setup program", |ui| {
                    ui.label("Installation file (EXE or MSI)");
                    ui.horizontal_wrapped(|ui| {
                        if ui.add(egui::TextEdit::singleline(&mut draft.file)
                            .hint_text("Select a Windows program…")
                            .desired_width(field_width(ui))).changed() {
                            draft.error = None;
                        }
                        if win98::push_button(ui, "Browse…", true).clicked()
                            && let Some(file) = rfd::FileDialog::new()
                                .add_filter("Windows setup", &["exe", "msi"])
                                .pick_file()
                        {
                            draft.choose_file(file);
                        }
                    });
                    ui.add_space(6.0);
                    ui.label("Program name");
                    if ui.add(egui::TextEdit::singleline(&mut draft.name)
                        .hint_text("Name in your program list")).changed() {
                        draft.error = None;
                    }
                    if draft.name.trim().is_empty() && !draft.file.trim().is_empty() {
                        if win98::push_button(ui, "Suggest name", true).clicked() {
                            draft.name = InstallDraft::suggested_name(Path::new(&draft.file));
                        }
                    }
                    ui.checkbox(&mut draft.portable,
                        "This is a portable program (do not run setup)");
                });
                ui.small("No program is executed while selecting files.");
            }
            Step::Compatibility => {
                classic_group(ui, "Wine environment", |ui| {
                    ui.label("Compatibility template");
                    egui::ComboBox::from_id_salt("win98-install-template")
                        .selected_text(draft.template.label())
                        .show_ui(ui, |ui| {
                            for template in [Template::Standard, Template::Games,
                                Template::Legacy32] {
                                ui.selectable_value(&mut draft.template, template,
                                    template.label());
                            }
                        });
                    if draft.template == Template::Legacy32 {
                        ui.small("32-bit Wine support is required. The prefix architecture cannot be changed later.");
                    }
                    ui.separator();
                    ui.label("Run this program with");
                    ui.radio_value(&mut draft.runner_kind, 0, "System Wine");
                    ui.radio_value(&mut draft.runner_kind, 1, "Custom Wine");
                    ui.radio_value(&mut draft.runner_kind, 2, "Proton / umu");
                    RiceWineApp::runner_fields(ui, draft);
                });
            }
            Step::Review => {
                classic_group(ui, "Confirm program settings", |ui| {
                    detail(ui, "Name", &draft.name);
                    detail(ui, "Setup file", &draft.file);
                    detail(ui, "Mode", if draft.portable {
                        "Portable application (register without setup)"
                    } else {
                        "Run setup using Wine"
                    });
                    detail(ui, "Template", draft.template.label());
                    detail(ui, "Runner", match draft.runner_kind {
                        0 => "System Wine",
                        1 => "Custom Wine",
                        _ => "Proton / umu",
                    });
                });
                ui.label(RichText::new(
                    "Windows applications can access files owned by your Linux user. Only run trusted software."
                ).color(win98::INK));
                ui.small("Finish runs the existing RiceWine validation and setup flow.");
            }
        });
    if let Some(error) = &draft.error {
        ui.colored_label(Color32::DARK_RED, error);
    }
    let step = draft.win98_step;
    let is_last = step == Step::Review;
    let ready = install_details_ready(draft) && install_compatibility_ready(draft);
    let next = if step == Step::Details {
        install_details_ready(draft)
    } else {
        ready
    };
    let finish_label = if draft.portable {
        "Add portable program"
    } else {
        "Run installer"
    };
    footer(
        ui,
        &mut draft.win98_step,
        step != Step::Details,
        next,
        is_last,
        finish_label,
        ready,
    )
}

/// Import is non-destructive: Finish merely asks the app's original import
/// path to register an existing prefix, never moves or deletes source files.
pub(crate) fn import(ui: &mut egui::Ui, draft: &mut ImportDraft) -> Action {
    header(ui, "Import Existing Program", draft.win98_step, true);
    ui.add_space(5.0);
    let area_height = (ui.ctx().content_rect().height() - 185.0).max(100.0);
    egui::ScrollArea::vertical()
        .id_salt("win98-import-steps")
        .max_height(area_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if draft.win98_step == Step::Details {
                classic_group(ui, "Existing Wine installation", |ui| {
                    ui.label("Program name");
                    if ui.text_edit_singleline(&mut draft.name).changed() {
                        draft.error = None;
                    }
                    ui.add_space(7.0);
                    ui.label("Existing Wine prefix directory");
                    ui.horizontal_wrapped(|ui| {
                        if ui.add(egui::TextEdit::singleline(&mut draft.prefix)
                            .desired_width(field_width(ui))).changed() {
                            draft.error = None;
                        }
                        if win98::push_button(ui, "Browse folder…", true).clicked()
                            && let Some(path) = rfd::FileDialog::new().pick_folder()
                        {
                            draft.prefix = core::readable_path(&path);
                            draft.error = None;
                        }
                    });
                    ui.add_space(7.0);
                    ui.label("Existing launch EXE");
                    ui.horizontal_wrapped(|ui| {
                        if ui.add(egui::TextEdit::singleline(&mut draft.executable)
                            .desired_width(field_width(ui))).changed() {
                            draft.error = None;
                        }
                        if win98::push_button(ui, "Browse file…", true).clicked()
                            && let Some(path) = rfd::FileDialog::new()
                                .add_filter("Windows program", &["exe"]).pick_file()
                        {
                            draft.executable = core::readable_path(&path);
                            if draft.name.trim().is_empty() {
                                draft.name = InstallDraft::suggested_name(&path);
                            }
                            draft.error = None;
                        }
                    });
                });
                ui.small("Existing files and Wine environments will not be moved.");
            } else {
                classic_group(ui, "Review imported program", |ui| {
                    detail(ui, "Name", &draft.name);
                    detail(ui, "Wine prefix", &draft.prefix);
                    detail(ui, "Launch EXE", &draft.executable);
                });
                ui.small("Finish only registers the program in RiceWine. The existing prefix stays where it is.");
            }
        });
    if let Some(error) = &draft.error {
        ui.colored_label(Color32::DARK_RED, error);
    }
    let step = draft.win98_step;
    let ready = import_details_ready(draft);
    footer(
        ui,
        &mut draft.win98_step,
        step != Step::Details,
        ready,
        step == Step::Review,
        "Import program",
        ready,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_wizard_blocks_missing_fields_and_invalid_runners() {
        let mut draft = InstallDraft::default();
        assert!(!install_details_ready(&draft));
        draft.file = "/tmp/setup.exe".into();
        draft.name = "Example".into();
        assert!(install_details_ready(&draft));
        assert!(install_compatibility_ready(&draft));
        draft.runner_kind = 1;
        assert!(!install_compatibility_ready(&draft));
        draft.custom_wine = "/usr/bin/wine".into();
        assert!(install_compatibility_ready(&draft));
        draft.runner_kind = 2;
        assert!(!install_compatibility_ready(&draft));
        draft.proton_dir = "/tmp/proton".into();
        assert!(install_compatibility_ready(&draft));
        draft.runner_kind = 9;
        assert!(!install_compatibility_ready(&draft));
    }

    #[test]
    fn import_wizard_requires_complete_existing_source() {
        let mut draft = ImportDraft::default();
        assert!(!import_details_ready(&draft));
        draft.name = "Example".into();
        draft.prefix = "/tmp/prefix".into();
        draft.executable = "/tmp/prefix/drive_c/Program.exe".into();
        assert!(import_details_ready(&draft));
        draft.prefix.clear();
        assert!(!import_details_ready(&draft));
    }
}
