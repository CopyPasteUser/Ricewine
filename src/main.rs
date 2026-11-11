use eframe::egui::{self, Color32, RichText};
use ricewine::core::{self, Application, InstallMode, Store, WineTools};
use ricewine::runtime::{self, PrefixProfile, Runner};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Theme {
    Windows10,
    Luna,
}

impl Theme {
    fn label(self) -> &'static str {
        match self {
            Self::Windows10 => "Windows 10",
            Self::Luna => "XP Luna",
        }
    }

    fn accent(self) -> Color32 {
        match self {
            Self::Windows10 => Color32::from_rgb(0, 120, 215),
            Self::Luna => Color32::from_rgb(32, 76, 184),
        }
    }

    fn sidebar(self) -> Color32 {
        match self {
            Self::Windows10 => Color32::from_rgb(242, 244, 247),
            Self::Luna => Color32::from_rgb(211, 225, 251),
        }
    }

    fn apply(self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::light();
        visuals.panel_fill = match self {
            Self::Windows10 => Color32::WHITE,
            Self::Luna => Color32::from_rgb(248, 247, 237),
        };
        visuals.selection.bg_fill = self.accent().gamma_multiply(0.35);
        visuals.selection.stroke.color = Color32::BLACK;
        visuals.widgets.active.bg_fill = self.accent();
        visuals.widgets.hovered.bg_fill = self.accent().gamma_multiply(0.25);
        visuals.widgets.inactive.bg_fill = match self {
            Self::Windows10 => Color32::from_rgb(237, 239, 243),
            Self::Luna => Color32::from_rgb(232, 234, 228),
        };
        ctx.set_visuals(visuals);
        ctx.global_style_mut(|style| {
            style.spacing.item_spacing = egui::vec2(8.0, 10.0);
            style.spacing.button_padding = egui::vec2(12.0, 7.0);
        });
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Library,
    Prefixes,
    Runners,
    Settings,
}

impl Page {
    fn label(self) -> &'static str {
        match self {
            Self::Library => "Library",
            Self::Prefixes => "Prefixes",
            Self::Runners => "Runners",
            Self::Settings => "Settings",
        }
    }
}

#[derive(Default)]
struct InstallDraft {
    name: String,
    file: String,
    portable: bool,
    runner_kind: usize,
    custom_wine: String,
    proton_dir: String,
    game_id: String,
}

impl InstallDraft {
    fn runner(&self) -> Runner {
        match self.runner_kind {
            1 => Runner::CustomWine {
                wine: PathBuf::from(self.custom_wine.trim()),
            },
            2 => Runner::ProtonUmu {
                proton: PathBuf::from(self.proton_dir.trim()),
                game_id: if self.game_id.trim().is_empty() {
                    "0".into()
                } else {
                    self.game_id.trim().into()
                },
            },
            _ => Runner::SystemWine,
        }
    }
}

enum Event {
    Installed(Result<Application, String>),
    Started(String, Result<u32, String>),
    Exited(String, Result<String, String>),
    CompatibilityFinished(String, String, Result<(), String>),
}

struct RiceWineApp {
    store: Option<Store>,
    wine: Option<WineTools>,
    theme: Theme,
    page: Page,
    selected: Option<String>,
    install: Option<InstallDraft>,
    remove_confirmation: Option<String>,
    busy: bool,
    running: HashSet<String>,
    compatibility_busy: HashSet<String>,
    runner_draft: InstallDraft,
    winetricks_verb: usize,
    log_preview: String,
    notice: Option<(bool, String)>,
    sender: Sender<Event>,
    receiver: Receiver<Event>,
}

impl RiceWineApp {
    fn new(ctx: &eframe::CreationContext<'_>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let store = Store::open_default();
        let wine = WineTools::detect();
        let mut notice = None;
        if let Err(error) = &store {
            notice = Some((true, error.clone()));
        } else if let Err(error) = &wine {
            notice = Some((
                true,
                format!("{error}. Install Wine to enable setup and launch."),
            ));
        }
        let theme = Theme::Windows10;
        theme.apply(&ctx.egui_ctx);
        let selected = store
            .as_ref()
            .ok()
            .and_then(|store| store.applications().first().map(|app| app.id.clone()));
        Self {
            store: store.ok(),
            wine: wine.ok(),
            theme,
            page: Page::Library,
            selected,
            install: None,
            remove_confirmation: None,
            busy: false,
            running: HashSet::new(),
            compatibility_busy: HashSet::new(),
            runner_draft: InstallDraft::default(),
            winetricks_verb: 0,
            log_preview: String::new(),
            notice,
            sender,
            receiver,
        }
    }

    fn poll(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.receiver.try_recv() {
            match event {
                Event::Installed(result) => {
                    self.busy = false;
                    match result {
                        Ok(app) => {
                            self.selected = Some(app.id.clone());
                            let name = app.name.clone();
                            match self
                                .store
                                .as_mut()
                                .expect("store available during install")
                                .add(app)
                            {
                                Ok(()) => {
                                    self.sync_runner_draft();
                                    self.notice = Some((
                                        false,
                                        format!(
                                            "{name} added. Select its installed EXE to enable Play."
                                        ),
                                    ))
                                }
                                Err(error) => self.notice = Some((true, error)),
                            }
                        }
                        Err(error) => self.notice = Some((true, error)),
                    }
                }
                Event::Started(id, result) => match result {
                    Ok(pid) => {
                        self.running.insert(id);
                        self.notice = Some((false, format!("Program started (PID {pid})")));
                    }
                    Err(error) => {
                        self.running.remove(&id);
                        self.notice = Some((true, error));
                    }
                },
                Event::Exited(id, result) => {
                    self.running.remove(&id);
                    if let Err(error) = result {
                        self.notice = Some((true, error));
                    }
                }
                Event::CompatibilityFinished(id, verb, result) => {
                    self.compatibility_busy.remove(&id);
                    self.notice = Some(match result {
                        Ok(()) => (false, format!("Winetricks completed: {verb}")),
                        Err(error) => (true, format!("Winetricks {verb} failed: {error}")),
                    });
                }
            }
        }
        if self.busy || !self.running.is_empty() || !self.compatibility_busy.is_empty() {
            ctx.request_repaint_after(Duration::from_millis(200));
        }
    }

    fn install_async(&mut self, draft: InstallDraft) {
        let Some(store) = self.store.as_ref() else {
            return;
        };
        let profile = PrefixProfile {
            runner: draft.runner(),
            ..Default::default()
        };
        if let Err(error) = runtime::validate_runner(&profile.runner) {
            self.notice = Some((true, error));
            return;
        }
        if self.wine.is_none() && matches!(profile.runner, Runner::SystemWine) {
            self.notice = Some((true, "Install Wine or choose a different runner".into()));
            return;
        }
        let file = PathBuf::from(draft.file.trim());
        if let Err(error) = core::validate_name(&draft.name)
            .and_then(|_| core::validate_installer(&file, draft.portable))
        {
            self.notice = Some((true, error));
            return;
        }
        self.busy = true;
        self.notice = Some((
            false,
            format!(
                "Setting up {}. Complete the Windows installer when it opens…",
                draft.name
            ),
        ));
        let root = store.root().to_path_buf();
        let wine = self.wine.clone();
        let tx = self.sender.clone();
        thread::spawn(move || {
            let mode = if draft.portable {
                InstallMode::Portable
            } else {
                InstallMode::Installer
            };
            let result = runtime::install_with_profile(
                &root,
                wine.as_ref(),
                &profile,
                &draft.name,
                &file,
                mode,
            );
            let _ = tx.send(Event::Installed(result));
        });
    }

    fn play(&mut self, app: Application) {
        let Some(store) = &self.store else {
            return;
        };
        let profile = match runtime::load_profile(&app.prefix) {
            Ok(profile) => profile,
            Err(error) => {
                self.notice = Some((true, error));
                return;
            }
        };
        if self.running.contains(&app.id) {
            self.notice = Some((false, "Already running".into()));
            return;
        }
        let tx = self.sender.clone();
        let root = store.root().to_path_buf();
        let wine = self.wine.clone();
        self.running.insert(app.id.clone());
        thread::spawn(move || {
            let id = app.id.clone();
            match runtime::launch(&root, wine.as_ref(), &app, &profile) {
                Ok(mut child) => {
                    let _ = tx.send(Event::Started(id.clone(), Ok(child.id())));
                    let outcome = child
                        .wait()
                        .map(|status| format!("Exit status {status}"))
                        .map_err(|e| e.to_string());
                    let _ = tx.send(Event::Exited(id, outcome));
                }
                Err(error) => {
                    let _ = tx.send(Event::Started(id, Err(error)));
                }
            }
        });
    }

    fn sync_runner_draft(&mut self) {
        self.runner_draft = InstallDraft::default();
        let Some(app) = self
            .selected
            .as_ref()
            .and_then(|id| self.store.as_ref()?.get(id))
        else {
            return;
        };
        match runtime::load_profile(&app.prefix) {
            Ok(profile) => match profile.runner {
                Runner::SystemWine => {}
                Runner::CustomWine { wine } => {
                    self.runner_draft.runner_kind = 1;
                    self.runner_draft.custom_wine = core::readable_path(&wine);
                }
                Runner::ProtonUmu { proton, game_id } => {
                    self.runner_draft.runner_kind = 2;
                    self.runner_draft.proton_dir = core::readable_path(&proton);
                    self.runner_draft.game_id = game_id;
                }
            },
            Err(error) => self.notice = Some((true, error)),
        }
    }

    fn compatibility_async(&mut self, app: Application, verb: String) {
        let Some(store) = &self.store else {
            return;
        };
        let profile = match runtime::load_profile(&app.prefix) {
            Ok(profile) => profile,
            Err(error) => {
                self.notice = Some((true, error));
                return;
            }
        };
        let root = store.root().to_path_buf();
        let sender = self.sender.clone();
        self.compatibility_busy.insert(app.id.clone());
        self.notice = Some((false, format!("Applying {verb}; see Wine log for details…")));
        thread::spawn(move || {
            let result = runtime::run_winetricks(&root, &app, &profile, &verb);
            let _ = sender.send(Event::CompatibilityFinished(app.id, verb, result));
        });
    }

    fn sidebar(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(self.theme.sidebar())
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_width(168.0);
                ui.set_min_height((ui.available_height() - 32.0).max(300.0));
                ui.add_space(5.0);
                ui.label(
                    RichText::new("◈ RiceWine")
                        .size(24.0)
                        .strong()
                        .color(self.theme.accent()),
                );
                ui.label(
                    RichText::new("Wine manager")
                        .small()
                        .color(Color32::DARK_GRAY),
                );
                ui.add_space(24.0);
                for page in [Page::Library, Page::Prefixes, Page::Runners, Page::Settings] {
                    let selected = page == self.page;
                    if ui
                        .add_sized(
                            [ui.available_width(), 37.0],
                            egui::Button::selectable(selected, page.label()),
                        )
                        .clicked()
                    {
                        self.page = page;
                    }
                }
                ui.add_space(20.0);
                ui.separator();
                ui.label(RichText::new("APPEARANCE").small().strong());
                for theme in [Theme::Windows10, Theme::Luna] {
                    if ui.radio(self.theme == theme, theme.label()).clicked() {
                        self.theme = theme;
                        theme.apply(ui.ctx());
                    }
                }
            });
    }

    fn library(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Library");
            ui.add_space(20.0);
            if ui
                .add_enabled(
                    !self.busy && self.store.is_some(),
                    egui::Button::new("＋ Install program"),
                )
                .clicked()
            {
                self.install = Some(InstallDraft::default());
            }
            if self.busy {
                ui.spinner();
                ui.label("Installing…");
            }
        });
        ui.add_space(10.0);
        let applications = self
            .store
            .as_ref()
            .map(|s| s.applications().to_vec())
            .unwrap_or_default();
        if applications.is_empty() {
            ui.add_space(45.0);
            ui.heading("Your library is empty");
            ui.label("Install a Windows program or add a portable EXE to get started.");
            return;
        }
        let width = (ui.available_width() - 280.0).max(260.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(width, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    egui::Frame::new()
                        .fill(Color32::from_gray(238))
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Program").strong());
                                ui.add_space(80.0);
                                ui.label(RichText::new("Runner").strong());
                            });
                        });
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for app in &applications {
                            let selected = self.selected.as_deref() == Some(&app.id);
                            ui.horizontal(|ui| {
                                if ui
                                    .add_sized(
                                        [width - 112.0, 36.0],
                                        egui::Button::selectable(selected, &app.name),
                                    )
                                    .clicked()
                                {
                                    self.selected = Some(app.id.clone());
                                    self.sync_runner_draft();
                                }
                                let label = runtime::load_profile(&app.prefix)
                                    .map(|profile| profile.runner.label())
                                    .unwrap_or("Invalid profile");
                                ui.label(label);
                            });
                            ui.separator();
                        }
                    });
                },
            );
            ui.separator();
            ui.vertical(|ui| {
                ui.set_min_width(235.0);
                let selected = self
                    .selected
                    .as_ref()
                    .and_then(|id| applications.iter().find(|app| &app.id == id))
                    .cloned();
                if let Some(app) = selected {
                    ui.heading(&app.name);
                    ui.add_space(8.0);
                    ui.label(RichText::new("Runner").small().strong());
                    let selected_profile = runtime::load_profile(&app.prefix);
                    match &selected_profile {
                        Ok(profile) => ui.label(profile.runner.label()),
                        Err(error) => ui.colored_label(Color32::DARK_RED, error),
                    };
                    ui.add_space(5.0);
                    ui.label(RichText::new("Prefix").small().strong());
                    ui.small(core::readable_path(&app.prefix));
                    ui.add_space(12.0);
                    if ui
                        .add_enabled(
                            !self.busy
                                && !self.compatibility_busy.contains(&app.id)
                                && app.executable.is_some()
                                && selected_profile.as_ref().is_ok_and(|p| {
                                    self.wine.is_some() || !matches!(p.runner, Runner::SystemWine)
                                }),
                            egui::Button::new("▶ Play"),
                        )
                        .clicked()
                    {
                        self.play(app.clone());
                    }
                    if self.running.contains(&app.id) {
                        ui.label("● Running");
                    }
                    if app.executable.is_none() {
                        ui.small("Choose the installed EXE to enable Play.");
                    }
                    if ui.button("Choose launch EXE…").clicked()
                        && let Some(path) = rfd::FileDialog::new()
                            .add_filter("Windows program", &["exe"])
                            .pick_file()
                        && let Some(store) = self.store.as_mut()
                    {
                        if let Err(error) = store.set_executable(&app.id, path) {
                            self.notice = Some((true, error));
                        } else {
                            self.notice = Some((false, "Launch executable saved".into()));
                        }
                    }
                    ui.add_space(18.0);
                    if ui.button("Remove from library…").clicked() {
                        self.remove_confirmation = Some(app.id);
                    }
                    ui.small("This never deletes the Wine prefix.");
                } else {
                    ui.label("Select a program to view details.");
                }
            });
        });
    }

    fn prefixes(&self, ui: &mut egui::Ui) {
        ui.heading("Wine prefixes");
        ui.label("Every program has an isolated Windows environment.");
        ui.add_space(12.0);
        if let Some(store) = &self.store {
            for app in store.applications() {
                ui.group(|ui| {
                    ui.label(RichText::new(&app.name).strong());
                    ui.small(core::readable_path(&app.prefix));
                });
            }
        }
    }

    fn runners(&mut self, ui: &mut egui::Ui) {
        ui.heading("Runners");
        ui.label("Choose a runner per program. Proton uses the external umu launcher.");
        ui.add_space(14.0);
        for line in runtime::diagnostics(self.wine.as_ref()) {
            ui.small(line);
        }
        ui.separator();
        let Some(app) = self
            .selected
            .as_ref()
            .and_then(|id| self.store.as_ref()?.get(id))
            .cloned()
        else {
            ui.label("Add and select a program in Library to configure a runner.");
            return;
        };
        ui.heading(format!("Program: {}", app.name));
        ui.horizontal_wrapped(|ui| {
            ui.label("Runner");
            for (index, label) in ["System Wine", "Custom Wine", "Proton / umu"]
                .iter()
                .enumerate()
            {
                ui.radio_value(&mut self.runner_draft.runner_kind, index, *label);
            }
        });
        Self::runner_fields(ui, &mut self.runner_draft);
        let working = self.running.contains(&app.id)
            || self.compatibility_busy.contains(&app.id)
            || self.busy;
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!working, egui::Button::new("Save runner for program"))
                .clicked()
            {
                let profile = PrefixProfile {
                    runner: self.runner_draft.runner(),
                    ..Default::default()
                };
                match runtime::save_profile(&app.prefix, &profile) {
                    Ok(()) => self.notice = Some((false, format!("Runner saved for {}", app.name))),
                    Err(error) => self.notice = Some((true, error)),
                }
            }
            if ui.button("Reload saved runner").clicked() {
                self.sync_runner_draft();
            }
        });
        ui.add_space(12.0);
        ui.separator();
        ui.label(RichText::new("Compatibility tools").strong());
        ui.small("Winetricks modifies the selected prefix. Apply components only when the program requires them. DXVK and VKD3D are provided by the installed Winetricks version.");
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("winetricks_component")
                .selected_text(runtime::WINETRICKS_VERBS[self.winetricks_verb])
                .show_ui(ui, |ui| {
                    for (index, name) in runtime::WINETRICKS_VERBS.iter().enumerate() {
                        ui.selectable_value(&mut self.winetricks_verb, index, *name);
                    }
                });
            if ui
                .add_enabled(!working, egui::Button::new("Apply with Winetricks"))
                .clicked()
            {
                self.compatibility_async(
                    app.clone(),
                    runtime::WINETRICKS_VERBS[self.winetricks_verb].to_owned(),
                );
            }
            if self.compatibility_busy.contains(&app.id) {
                ui.spinner();
                ui.label("Applying…");
            }
        });
        ui.add_space(12.0);
        ui.separator();
        if ui.button("Read recent Wine log").clicked() {
            self.log_preview = self
                .store
                .as_ref()
                .map(|store| {
                    runtime::log_tail(store.root(), &app.id, 12_000)
                        .unwrap_or_else(|e| format!("Could not read log: {e}"))
                })
                .unwrap_or_default();
        }
        if !self.log_preview.is_empty() {
            egui::ScrollArea::vertical()
                .max_height(180.0)
                .show(ui, |ui| {
                    ui.monospace(&self.log_preview);
                });
        }
    }

    fn runner_fields(ui: &mut egui::Ui, draft: &mut InstallDraft) {
        match draft.runner_kind {
            1 => {
                ui.label("Wine executable (wineboot must be alongside it)");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut draft.custom_wine);
                    if ui.button("Browse…").clicked()
                        && let Some(path) = rfd::FileDialog::new().pick_file()
                    {
                        draft.custom_wine = core::readable_path(&path);
                    }
                });
            }
            2 => {
                ui.label("Extracted Proton directory");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut draft.proton_dir);
                    if ui.button("Browse…").clicked()
                        && let Some(path) = rfd::FileDialog::new().pick_folder()
                    {
                        draft.proton_dir = core::readable_path(&path);
                    }
                });
                ui.label("Game ID (0 disables game-specific protonfixes)");
                ui.text_edit_singleline(&mut draft.game_id);
                ui.small("Requires umu-run in PATH. Wine prefixes are not permission sandboxes.");
            }
            _ => {}
        }
    }

    fn settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.label("Appearance");
        egui::ComboBox::from_id_salt("theme")
            .selected_text(self.theme.label())
            .show_ui(ui, |ui| {
                for theme in [Theme::Windows10, Theme::Luna] {
                    if ui
                        .selectable_value(&mut self.theme, theme, theme.label())
                        .clicked()
                    {
                        theme.apply(ui.ctx());
                    }
                }
            });
        ui.add_space(18.0);
        if let Some(store) = &self.store {
            ui.label("Application data");
            ui.small(core::readable_path(store.root()));
            ui.small("Catalog changes are saved atomically. Removing a program from the library preserves its files.");
        }
    }

    fn install_dialog(&mut self, ctx: &egui::Context) {
        let mut submit = None;
        let mut keep_open = true;
        if let Some(draft) = &mut self.install {
            egui::Window::new("Install Windows program")
                .collapsible(false)
                .resizable(false)
                .open(&mut keep_open)
                .show(ctx, |ui| {
                    ui.set_min_width(460.0);
                    ui.label("Program name");
                    ui.text_edit_singleline(&mut draft.name);
                    ui.label("EXE or MSI file");
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut draft.file).desired_width(340.0));
                        if ui.button("Browse…").clicked()
                            && let Some(file) = rfd::FileDialog::new().add_filter("Windows setup", &["exe", "msi"]).pick_file()
                        {
                            draft.file = file.to_string_lossy().into_owned();
                            if draft.name.trim().is_empty() {
                                draft.name = file.file_stem().unwrap_or_default().to_string_lossy().into_owned();
                            }
                        }
                    });
                    ui.checkbox(&mut draft.portable, "Portable EXE (no installer)");
                    ui.add_space(10.0);
                    ui.label("Runner for this program");
                    ui.horizontal_wrapped(|ui| {
                        for (index, label) in ["System Wine", "Custom Wine", "Proton / umu"].iter().enumerate() {
                            ui.radio_value(&mut draft.runner_kind, index, *label);
                        }
                    });
                    Self::runner_fields(ui, draft);
                    ui.small("A fresh prefix does not sandbox Windows software. Only run applications you trust.");
                    ui.add_space(12.0);
                    if ui.button(if draft.portable { "Add portable program" } else { "Run installer" }).clicked() {
                        submit = Some(InstallDraft {
                            name: draft.name.clone(), file: draft.file.clone(), portable: draft.portable,
                            runner_kind: draft.runner_kind, custom_wine: draft.custom_wine.clone(),
                            proton_dir: draft.proton_dir.clone(), game_id: draft.game_id.clone(),
                        });
                    }
                });
        }
        if !keep_open {
            self.install = None;
        }
        if let Some(draft) = submit {
            self.install = None;
            self.install_async(draft);
        }
    }

    fn remove_dialog(&mut self, ctx: &egui::Context) {
        let Some(id) = self.remove_confirmation.clone() else {
            return;
        };
        egui::Window::new("Remove from library?").collapsible(false).resizable(false).show(ctx, |ui| {
            ui.label("The catalog entry will be removed. The Wine prefix and installed files will remain on disk.");
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() { self.remove_confirmation = None; }
                if ui.button("Remove entry").clicked() {
                    if let Some(store) = &mut self.store {
                        match store.remove_from_library(&id) {
                            Ok(()) => {
                                self.selected = store.applications().first().map(|app| app.id.clone());
                                self.notice = Some((false, "Removed from library. Prefix preserved.".into()));
                            }
                            Err(error) => self.notice = Some((true, error)),
                        }
                    }
                    self.remove_confirmation = None;
                }
            });
        });
    }
}

impl eframe::App for RiceWineApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll(ui.ctx());
        let height = ui.available_height();
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(200.0, height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| self.sidebar(ui),
            );
            ui.separator();
            ui.vertical(|ui| {
                ui.add_space(10.0);
                if let Some((is_error, message)) = self.notice.clone() {
                    let color = if is_error {
                        Color32::DARK_RED
                    } else {
                        self.theme.accent()
                    };
                    ui.horizontal(|ui| {
                        ui.colored_label(color, message);
                        if ui.small_button("×").clicked() {
                            self.notice = None;
                        }
                    });
                    ui.separator();
                }
                match self.page {
                    Page::Library => self.library(ui),
                    Page::Prefixes => self.prefixes(ui),
                    Page::Runners => self.runners(ui),
                    Page::Settings => self.settings(ui),
                }
            });
        });
        self.install_dialog(ui.ctx());
        self.remove_dialog(ui.ctx());
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1080.0, 700.0])
            .with_min_inner_size([780.0, 500.0]),
        ..Default::default()
    };
    eframe::run_native(
        "RiceWine",
        options,
        Box::new(|ctx| Ok(Box::new(RiceWineApp::new(ctx)))),
    )
}
