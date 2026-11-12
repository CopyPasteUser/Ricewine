#![forbid(unsafe_code)]

mod luna;
mod luna_icons;

use eframe::egui::{self, Color32, RichText};
use ricewine::backup::{self, BackupEntry};
use ricewine::core::{self, Application, InstallMode, OperationControl, Store, WineTools};
use ricewine::discovery::{self, LaunchCandidate};
use ricewine::integration;
#[cfg(unix)]
use ricewine::ipc;
use ricewine::library;
use ricewine::preferences::{self, Appearance, Preferences};
use ricewine::runtime::{self, CompatibilityPreset, PrefixProfile, Runner, Template};
use ricewine::storage::{self, CleanupPreview, DiskUsage};
use ricewine::updates::{self, Release};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Theme {
    Windows10,
    Luna,
    Windows10Dark,
    LunaOlive,
    LunaSilver,
}

#[derive(Clone, Copy)]
struct RicePalette {
    accent: Color32,
    sidebar: Color32,
    panel: Color32,
    button: Color32,
    hover: Color32,
    input: Color32,
    text: Color32,
    muted: Color32,
    border: Color32,
    dark: bool,
    corner_radius: u8,
}

impl Theme {
    const ALL: [Self; 5] = [
        Self::Windows10,
        Self::Windows10Dark,
        Self::Luna,
        Self::LunaOlive,
        Self::LunaSilver,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Windows10 => "Windows 10 Light",
            Self::Windows10Dark => "Windows 10 Dark",
            Self::Luna => "XP Luna Blue",
            Self::LunaOlive => "XP Luna Olive",
            Self::LunaSilver => "XP Luna Silver",
        }
    }

    fn palette(self) -> RicePalette {
        let rgb = Color32::from_rgb;
        match self {
            Self::Windows10 => RicePalette {
                accent: rgb(0, 102, 190),
                sidebar: rgb(242, 244, 247),
                panel: rgb(255, 255, 255),
                button: rgb(232, 237, 244),
                hover: rgb(211, 229, 248),
                input: rgb(255, 255, 255),
                text: rgb(28, 37, 47),
                muted: rgb(82, 96, 112),
                border: rgb(176, 189, 202),
                dark: false,
                corner_radius: 3,
            },
            Self::Windows10Dark => RicePalette {
                accent: rgb(86, 175, 244),
                sidebar: rgb(32, 39, 48),
                panel: rgb(23, 29, 36),
                button: rgb(45, 55, 67),
                hover: rgb(60, 75, 90),
                input: rgb(35, 43, 53),
                text: rgb(234, 240, 246),
                muted: rgb(179, 193, 209),
                border: rgb(89, 104, 120),
                dark: true,
                corner_radius: 3,
            },
            Self::Luna => RicePalette {
                accent: rgb(25, 73, 171),
                sidebar: rgb(160, 191, 239),
                panel: rgb(255, 255, 255),
                button: rgb(236, 237, 233),
                hover: rgb(255, 245, 205),
                input: rgb(255, 255, 255),
                text: rgb(28, 46, 81),
                muted: rgb(75, 91, 118),
                border: rgb(119, 150, 195),
                dark: false,
                corner_radius: 3,
            },
            Self::LunaOlive => RicePalette {
                accent: rgb(77, 106, 43),
                sidebar: rgb(219, 229, 191),
                panel: rgb(248, 246, 229),
                button: rgb(227, 234, 209),
                hover: rgb(208, 224, 175),
                input: rgb(255, 254, 246),
                text: rgb(42, 55, 34),
                muted: rgb(84, 100, 69),
                border: rgb(140, 160, 105),
                dark: false,
                corner_radius: 6,
            },
            Self::LunaSilver => RicePalette {
                accent: rgb(72, 91, 145),
                sidebar: rgb(220, 225, 239),
                panel: rgb(247, 248, 252),
                button: rgb(227, 231, 241),
                hover: rgb(209, 218, 240),
                input: rgb(255, 255, 255),
                text: rgb(38, 47, 69),
                muted: rgb(84, 96, 124),
                border: rgb(144, 156, 183),
                dark: false,
                corner_radius: 6,
            },
        }
    }

    fn accent(self) -> Color32 {
        self.palette().accent
    }

    fn error(self) -> Color32 {
        if self.palette().dark {
            Color32::from_rgb(255, 137, 130)
        } else {
            Color32::DARK_RED
        }
    }

    fn sidebar(self) -> Color32 {
        self.palette().sidebar
    }

    fn apply(self, ctx: &egui::Context) {
        let palette = self.palette();
        let mut visuals = if palette.dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.panel_fill = palette.panel;
        visuals.window_fill = palette.panel;
        visuals.window_stroke = egui::Stroke::new(1.0, palette.border);
        visuals.extreme_bg_color = palette.input;
        visuals.text_edit_bg_color = Some(palette.input);
        visuals.faint_bg_color = palette.button;
        visuals.weak_text_color = Some(palette.muted);
        visuals.hyperlink_color = palette.accent;
        visuals.selection.bg_fill = palette.accent;
        visuals.selection.stroke.color = if palette.dark {
            palette.panel
        } else {
            Color32::WHITE
        };
        visuals.widgets.active.bg_fill = palette.accent;
        visuals.widgets.active.weak_bg_fill = palette.accent;
        visuals.widgets.active.fg_stroke.color = if palette.dark {
            palette.panel
        } else {
            Color32::WHITE
        };
        visuals.widgets.hovered.bg_fill = palette.hover;
        visuals.widgets.hovered.weak_bg_fill = palette.hover;
        visuals.widgets.hovered.fg_stroke.color = palette.text;
        visuals.widgets.inactive.bg_fill = palette.button;
        visuals.widgets.inactive.weak_bg_fill = palette.button;
        visuals.widgets.inactive.fg_stroke.color = palette.text;
        visuals.widgets.noninteractive.fg_stroke.color = palette.text;
        visuals.widgets.noninteractive.bg_fill = palette.panel;
        visuals.widgets.open.bg_fill = palette.hover;
        visuals.widgets.open.weak_bg_fill = palette.hover;
        visuals.widgets.open.fg_stroke.color = palette.text;
        for widget in [
            &mut visuals.widgets.noninteractive,
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.open,
        ] {
            widget.corner_radius = egui::CornerRadius::same(palette.corner_radius);
            widget.bg_stroke = egui::Stroke::new(1.0, palette.border);
        }
        if self == Self::Luna {
            // Classic Luna control surfaces, not the compositor title bar.
            visuals.widgets.inactive.bg_fill = Color32::from_rgb(238, 240, 246);
            visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(238, 240, 246);
            visuals.widgets.hovered.bg_fill = Color32::from_rgb(255, 245, 205);
            visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(255, 245, 205);
            visuals.widgets.active.bg_fill = Color32::from_rgb(212, 225, 246);
            visuals.widgets.active.weak_bg_fill = Color32::from_rgb(212, 225, 246);
            visuals.widgets.active.fg_stroke.color = Color32::from_rgb(25, 52, 108);
            let border = egui::Stroke::new(1.0, Color32::from_rgb(118, 148, 194));
            visuals.widgets.inactive.bg_stroke = border;
            visuals.widgets.hovered.bg_stroke =
                egui::Stroke::new(1.0, Color32::from_rgb(238, 150, 40));
            visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, palette.accent);
        }
        visuals.window_corner_radius = egui::CornerRadius::same(palette.corner_radius);
        visuals.menu_corner_radius = egui::CornerRadius::same(palette.corner_radius);
        ctx.set_visuals(visuals);
        ctx.global_style_mut(|style| {
            style.spacing.item_spacing = if self == Self::Luna {
                egui::vec2(6.0, 7.0)
            } else {
                egui::vec2(8.0, 10.0)
            };
            style.spacing.button_padding = if self == Self::Luna {
                egui::vec2(8.0, 4.0)
            } else {
                egui::vec2(12.0, 7.0)
            };
            style.spacing.interact_size = if self == Self::Luna {
                egui::vec2(36.0, 24.0)
            } else {
                egui::vec2(40.0, 28.0)
            };
            let (body, heading, button, small, monospace) = if self == Self::Luna {
                (12.0, 17.0, 12.0, 11.0, 11.0)
            } else {
                (14.0, 19.0, 14.0, 12.0, 13.0)
            };
            for (kind, size, family) in [
                (egui::TextStyle::Body, body, egui::FontFamily::Proportional),
                (egui::TextStyle::Heading, heading, egui::FontFamily::Proportional),
                (egui::TextStyle::Button, button, egui::FontFamily::Proportional),
                (egui::TextStyle::Small, small, egui::FontFamily::Proportional),
                (egui::TextStyle::Monospace, monospace, egui::FontFamily::Monospace),
            ] {
                style.text_styles.insert(kind, egui::FontId::new(size, family));
            }
        });
    }
}

impl From<Appearance> for Theme {
    fn from(value: Appearance) -> Self {
        match value {
            Appearance::Windows10 => Self::Windows10,
            Appearance::Luna => Self::Luna,
            Appearance::Windows10Dark => Self::Windows10Dark,
            Appearance::LunaOlive => Self::LunaOlive,
            Appearance::LunaSilver => Self::LunaSilver,
        }
    }
}

impl From<Theme> for Appearance {
    fn from(value: Theme) -> Self {
        match value {
            Theme::Windows10 => Self::Windows10,
            Theme::Luna => Self::Luna,
            Theme::Windows10Dark => Self::Windows10Dark,
            Theme::LunaOlive => Self::LunaOlive,
            Theme::LunaSilver => Self::LunaSilver,
        }
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

    /// Screenshot-only override for the initial page, not the persisted app state.
    fn capture_override() -> Self {
        match std::env::var("RICEWINE_CAPTURE_PAGE").as_deref() {
            Ok("prefixes") => Self::Prefixes,
            Ok("runners") => Self::Runners,
            Ok("settings") => Self::Settings,
            _ => Self::Library,
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
    template: Template,
}

#[derive(Default)]
struct ImportDraft {
    name: String,
    prefix: String,
    executable: String,
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
    Installed(Result<(Application, bool), String>),
    Started(String, Result<u32, String>),
    Exited(String, Result<String, String>),
    CompatibilityFinished(String, String, Result<(), String>),
    BackupCreated(Result<BackupEntry, String>),
    Restored(Result<Application, String>),
    UsageMeasured(String, Result<DiskUsage, String>),
    CleanupScanned(String, Result<CleanupPreview, String>),
    Cleaned(String, Result<DiskUsage, String>),
    GeReleaseChecked(Result<Release, String>),
    GeInstalled(Result<PathBuf, String>),
    GeRemoved(PathBuf, Result<(), String>),
    AppReleaseChecked(String, Result<Release, String>),
    AppUpdated(String, Result<(), String>),
}

struct RiceWineApp {
    store: Option<Store>,
    wine: Option<WineTools>,
    theme: Theme,
    preferences: Preferences,
    page: Page,
    about_open: bool,
    selected: Option<String>,
    search: String,
    name_edit: String,
    launch_args_text: String,
    launch_env_text: String,
    update_repo_text: String,
    installed_version_text: String,
    latest_app_releases: HashMap<String, Release>,
    update_install_confirmation: Option<(String, PathBuf)>,
    update_controls: HashMap<String, OperationControl>,
    detected_proton: Vec<runtime::DetectedProton>,
    ge_release: Option<Release>,
    ge_busy: bool,
    ge_control: Option<OperationControl>,
    ge_install_confirmation: bool,
    ge_remove_confirmation: Option<PathBuf>,
    install: Option<InstallDraft>,
    queued_install_files: VecDeque<PathBuf>,
    #[cfg(unix)]
    ipc: Option<ipc::Server>,
    import: Option<ImportDraft>,
    remove_confirmation: Option<String>,
    cancel_install_confirmation: bool,
    stop_confirmation: Option<String>,
    busy: bool,
    install_control: Option<OperationControl>,
    running: HashSet<String>,
    running_controls: HashMap<String, OperationControl>,
    exe_candidates: Vec<LaunchCandidate>,
    exe_candidate_for: Option<String>,
    compatibility_busy: HashSet<String>,
    backup_busy: bool,
    backups: Vec<BackupEntry>,
    disk_usage: HashMap<String, DiskUsage>,
    cleanup_preview: Option<(String, CleanupPreview)>,
    storage_busy: HashSet<String>,
    runner_draft: InstallDraft,
    winetricks_verb: usize,
    preset_choice: usize,
    log_preview: String,
    library_report: String,
    notice: Option<(bool, String)>,
    sender: Sender<Event>,
    receiver: Receiver<Event>,
}

impl RiceWineApp {
    fn install_draft_for_file(path: PathBuf) -> InstallDraft {
        InstallDraft {
            name: path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            file: core::readable_path(&path),
            ..Default::default()
        }
    }

    fn new(ctx: &eframe::CreationContext<'_>, initial_install: Option<PathBuf>) -> Self {
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
        let preferences_result = store
            .as_ref()
            .ok()
            .map(|s| preferences::load(s.root()))
            .unwrap_or(Ok(Preferences::default()));
        if let Err(error) = &preferences_result {
            notice = Some((true, error.clone()));
        }
        let preferences = preferences_result.unwrap_or_default();
        let theme: Theme = if preferences.follow_desktop {
            preferences::detect_desktop_appearance()
                .unwrap_or(preferences.appearance)
                .into()
        } else {
            preferences.appearance.into()
        };
        theme.apply(&ctx.egui_ctx);
        let selected = store
            .as_ref()
            .ok()
            .and_then(|store| store.applications().first().map(|app| app.id.clone()));
        let backups = store
            .as_ref()
            .ok()
            .and_then(|store| backup::list_backups(store.root()).ok())
            .unwrap_or_default();
        #[cfg(unix)]
        let ipc = store
            .as_ref()
            .ok()
            .and_then(|store| match ipc::Server::bind(store.root()) {
                Ok(server) => Some(server),
                Err(error) => {
                    notice = Some((true, format!("Open With handoff unavailable: {error}")));
                    None
                }
            });
        let mut app = Self {
            store: store.ok(),
            wine: wine.ok(),
            theme,
            preferences,
            page: Page::capture_override(),
            about_open: false,
            selected,
            search: String::new(),
            name_edit: String::new(),
            launch_args_text: String::new(),
            launch_env_text: String::new(),
            update_repo_text: String::new(),
            installed_version_text: String::new(),
            latest_app_releases: HashMap::new(),
            update_install_confirmation: None,
            update_controls: HashMap::new(),
            detected_proton: Vec::new(),
            ge_release: None,
            ge_busy: false,
            ge_control: None,
            ge_install_confirmation: false,
            ge_remove_confirmation: None,
            install: initial_install.map(Self::install_draft_for_file),
            queued_install_files: VecDeque::new(),
            #[cfg(unix)]
            ipc,
            import: None,
            remove_confirmation: None,
            cancel_install_confirmation: false,
            stop_confirmation: None,
            busy: false,
            install_control: None,
            running: HashSet::new(),
            running_controls: HashMap::new(),
            exe_candidates: Vec::new(),
            exe_candidate_for: None,
            compatibility_busy: HashSet::new(),
            backup_busy: false,
            backups,
            disk_usage: HashMap::new(),
            cleanup_preview: None,
            storage_busy: HashSet::new(),
            runner_draft: InstallDraft::default(),
            winetricks_verb: 0,
            preset_choice: 0,
            log_preview: String::new(),
            library_report: String::new(),
            notice,
            sender,
            receiver,
        };
        app.sync_runner_draft();
        app
    }

    fn poll(&mut self, ctx: &egui::Context) {
        #[cfg(unix)]
        if let Some(server) = &self.ipc {
            for _ in 0..8 {
                match server.receive() {
                    Ok(Some(path)) => self.queued_install_files.push_back(path),
                    Ok(None) => break,
                    Err(error) => self.notice = Some((true, error)),
                }
            }
        }
        while let Ok(event) = self.receiver.try_recv() {
            match event {
                Event::Installed(result) => {
                    self.busy = false;
                    self.install_control = None;
                    match result {
                        Ok((app, autodetected)) => {
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
                                        if autodetected {
                                            format!(
                                                "{name} added; launch EXE detected automatically. Check it before Play."
                                            )
                                        } else {
                                            format!(
                                                "{name} added. Scan or choose its launch EXE to enable Play."
                                            )
                                        },
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
                        self.running_controls.remove(&id);
                        self.notice = Some((true, error));
                    }
                },
                Event::Exited(id, result) => {
                    self.running.remove(&id);
                    self.running_controls.remove(&id);
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
                Event::BackupCreated(result) => {
                    self.backup_busy = false;
                    match result {
                        Ok(entry) => {
                            self.notice =
                                Some((false, format!("Backup created for {}", entry.app_name)));
                            self.backups.insert(0, entry);
                        }
                        Err(error) => self.notice = Some((true, error)),
                    }
                }
                Event::Restored(result) => {
                    self.backup_busy = false;
                    match result {
                        Ok(app) => {
                            let id = app.id.clone();
                            if let Some(store) = self.store.as_mut() {
                                match store.add(app) {
                                    Ok(()) => {
                                        self.selected = Some(id);
                                        self.sync_runner_draft();
                                        self.notice = Some((false, "Backup restored as a separate program; the original was preserved".into()));
                                    }
                                    Err(error) => self.notice = Some((true, error)),
                                }
                            }
                        }
                        Err(error) => self.notice = Some((true, error)),
                    }
                }
                Event::UsageMeasured(id, result) => {
                    self.storage_busy.remove(&id);
                    match result {
                        Ok(usage) => {
                            self.disk_usage.insert(id, usage);
                        }
                        Err(error) => self.notice = Some((true, error)),
                    }
                }
                Event::CleanupScanned(id, result) => {
                    self.storage_busy.remove(&id);
                    match result {
                        Ok(preview) => self.cleanup_preview = Some((id, preview)),
                        Err(error) => self.notice = Some((true, error)),
                    }
                }
                Event::Cleaned(id, result) => {
                    self.storage_busy.remove(&id);
                    self.disk_usage.remove(&id);
                    self.notice = Some(match result {
                        Ok(usage) => (
                            false,
                            format!(
                                "Removed {} old temporary files ({})",
                                usage.files,
                                storage::human_size(usage.bytes)
                            ),
                        ),
                        Err(error) => (true, error),
                    });
                }
                Event::GeReleaseChecked(result) => {
                    self.ge_busy = false;
                    match result {
                        Ok(release) => self.ge_release = Some(release),
                        Err(error) => {
                            self.notice = Some((true, format!("GE-Proton release check: {error}")))
                        }
                    }
                }
                Event::GeInstalled(result) => {
                    self.ge_busy = false;
                    self.ge_control = None;
                    match result {
                        Ok(path) => {
                            let name = path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into_owned();
                            self.detected_proton.push(runtime::DetectedProton {
                                path: path.clone(),
                                name,
                            });
                            self.detected_proton.sort_by(|a, b| a.name.cmp(&b.name));
                            self.notice = Some((
                                false,
                                format!("Verified GE-Proton installed at {}", path.display()),
                            ));
                        }
                        Err(error) => self.notice = Some((true, error)),
                    }
                }
                Event::GeRemoved(path, result) => {
                    self.ge_busy = false;
                    match result {
                        Ok(()) => {
                            self.detected_proton.retain(|item| item.path != path);
                            self.notice =
                                Some((false, "Managed runner removed; prefixes preserved".into()));
                        }
                        Err(error) => self.notice = Some((true, error)),
                    }
                }
                Event::AppReleaseChecked(id, result) => {
                    self.compatibility_busy.remove(&id);
                    match result {
                        Ok(release) => {
                            self.notice =
                                Some((false, format!("Latest publisher release: {}", release.tag)));
                            self.latest_app_releases.insert(id, release);
                        }
                        Err(error) => self.notice = Some((true, error)),
                    }
                }
                Event::AppUpdated(id, result) => {
                    self.compatibility_busy.remove(&id);
                    self.update_controls.remove(&id);
                    self.notice = Some(match result {
                        Ok(()) => (false, "Update installer completed. Verify the application and update its version label if needed.".into()),
                        Err(error) => (true, format!("Update installer: {error}")),
                    });
                }
            }
        }
        if self.install.is_none()
            && let Some(path) = self.queued_install_files.pop_front()
        {
            self.install = Some(Self::install_draft_for_file(path));
            self.notice = Some((
                false,
                "Open With request received; review before installing".into(),
            ));
        }
        #[cfg(unix)]
        if self.ipc.is_some() {
            ctx.request_repaint_after(Duration::from_millis(300));
        }
        if self.busy
            || self.backup_busy
            || !self.running.is_empty()
            || !self.compatibility_busy.is_empty()
            || !self.storage_busy.is_empty()
            || self.ge_busy
        {
            ctx.request_repaint_after(Duration::from_millis(200));
        }
    }

    fn persist_preferences(&mut self) {
        if let Some(store) = &self.store
            && let Err(error) = preferences::save(store.root(), &self.preferences)
        {
            self.notice = Some((true, format!("Could not save appearance: {error}")));
        }
    }

    fn set_theme(&mut self, theme: Theme, ctx: &egui::Context) {
        self.theme = theme;
        self.preferences.appearance = theme.into();
        self.preferences.follow_desktop = false;
        self.theme.apply(ctx);
        self.persist_preferences();
    }

    fn install_async(&mut self, draft: InstallDraft) {
        let Some(store) = self.store.as_ref() else {
            return;
        };
        let profile = PrefixProfile {
            runner: draft.runner(),
            template: draft.template,
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
        let control = OperationControl::default();
        self.install_control = Some(control.clone());
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
            let result = runtime::install_with_profile_control(
                &root,
                wine.as_ref(),
                &profile,
                &draft.name,
                &file,
                mode,
                &control,
            )
            .map(|mut app| {
                let autodetected = if app.executable.is_none() {
                    discovery::scan_executables(&app.prefix, &app.name)
                        .ok()
                        .and_then(|candidates| discovery::confident_executable(&candidates))
                        .map(|exe| {
                            app.executable = Some(exe);
                        })
                        .is_some()
                } else {
                    false
                };
                (app, autodetected)
            });
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
        let control = OperationControl::default();
        self.running_controls
            .insert(app.id.clone(), control.clone());
        thread::spawn(move || {
            let id = app.id.clone();
            match runtime::launch(&root, wine.as_ref(), &app, &profile) {
                Ok(mut child) => {
                    let _ = tx.send(Event::Started(id.clone(), Ok(child.id())));
                    let outcome = loop {
                        match child.try_wait() {
                            Ok(Some(status)) => break Ok(format!("Exit status {status}")),
                            Ok(None) if control.is_cancelled() => {
                                core::terminate_child(&mut child);
                                let _ = child.wait();
                                break Ok("Program stopped by user".into());
                            }
                            Ok(None) => thread::sleep(Duration::from_millis(100)),
                            Err(error) => break Err(error.to_string()),
                        }
                    };
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
        self.launch_args_text.clear();
        self.launch_env_text.clear();
        self.update_repo_text.clear();
        self.installed_version_text.clear();
        self.name_edit.clear();
        let Some(app) = self
            .selected
            .as_ref()
            .and_then(|id| self.store.as_ref()?.get(id))
        else {
            return;
        };
        self.name_edit = app.name.clone();
        match runtime::load_profile(&app.prefix) {
            Ok(profile) => {
                self.update_repo_text = profile.update_repository.clone().unwrap_or_default();
                self.installed_version_text = profile.installed_version.clone();
                self.launch_args_text = profile.launch_arguments.join("\n");
                self.launch_env_text = profile
                    .launch_environment
                    .iter()
                    .map(|(key, value)| format!("{key}={value}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                match profile.runner {
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
                }
            }
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

    fn backup_async(&mut self, app: Application) {
        let Some(store) = self.store.as_ref() else {
            return;
        };
        if self.backup_busy
            || self.busy
            || self.running.contains(&app.id)
            || self.compatibility_busy.contains(&app.id)
            || self.storage_busy.contains(&app.id)
        {
            return;
        }
        self.backup_busy = true;
        self.notice = Some((
            false,
            format!(
                "Backing up {}; close other processes using its prefix",
                app.name
            ),
        ));
        let root = store.root().to_path_buf();
        let sender = self.sender.clone();
        thread::spawn(move || {
            let _ = sender.send(Event::BackupCreated(backup::create_backup(&root, &app)));
        });
    }

    fn restore_async(&mut self, entry: BackupEntry) {
        let Some(store) = self.store.as_ref() else {
            return;
        };
        if self.backup_busy || self.busy || !self.running.is_empty() {
            return;
        }
        self.backup_busy = true;
        self.notice = Some((
            false,
            format!("Restoring {} into a new prefix…", entry.app_name),
        ));
        let root = store.root().to_path_buf();
        let sender = self.sender.clone();
        thread::spawn(move || {
            let _ = sender.send(Event::Restored(backup::restore_backup(
                &root,
                &entry.folder,
            )));
        });
    }

    fn measure_storage(&mut self, app: Application, cleanup: bool) {
        let Some(root) = self.store.as_ref().map(|store| store.root().to_path_buf()) else {
            return;
        };
        if self.storage_busy.contains(&app.id)
            || self.running.contains(&app.id)
            || self.compatibility_busy.contains(&app.id)
            || self.backup_busy
            || self.busy
        {
            return;
        }
        let id = app.id.clone();
        self.storage_busy.insert(id.clone());
        let sender = self.sender.clone();
        thread::spawn(move || {
            if cleanup {
                let _ = sender.send(Event::CleanupScanned(
                    id,
                    storage::preview_cleanup(&root, &app),
                ));
            } else {
                let _ = sender.send(Event::UsageMeasured(id, storage::disk_usage(&app.prefix)));
            }
        });
    }

    fn check_ge_release(&mut self) {
        if self.ge_busy {
            return;
        }
        self.ge_busy = true;
        let sender = self.sender.clone();
        thread::spawn(move || {
            let _ = sender.send(Event::GeReleaseChecked(updates::latest_ge_proton()));
        });
    }

    fn check_app_release(&mut self, app: &Application) {
        let profile = match runtime::load_profile(&app.prefix) {
            Ok(value) => value,
            Err(error) => {
                self.notice = Some((true, error));
                return;
            }
        };
        let Some(repository) = profile.update_repository else {
            self.notice = Some((
                true,
                "Save an owner/repo update source in Runners first".into(),
            ));
            return;
        };
        self.compatibility_busy.insert(app.id.clone());
        let sender = self.sender.clone();
        let id = app.id.clone();
        thread::spawn(move || {
            let _ = sender.send(Event::AppReleaseChecked(
                id,
                updates::latest_release(&repository),
            ));
        });
    }

    fn update_app_async(&mut self, app: Application, installer: PathBuf) {
        let Some(root) = self.store.as_ref().map(|store| store.root().to_path_buf()) else {
            return;
        };
        if self.compatibility_busy.contains(&app.id)
            || self.running.contains(&app.id)
            || self.backup_busy
            || self.busy
        {
            return;
        }
        let profile = match runtime::load_profile(&app.prefix) {
            Ok(value) => value,
            Err(error) => {
                self.notice = Some((true, error));
                return;
            }
        };
        let id = app.id.clone();
        self.compatibility_busy.insert(id.clone());
        let control = OperationControl::default();
        self.update_controls.insert(id.clone(), control.clone());
        let sender = self.sender.clone();
        let wine = self.wine.clone();
        self.notice = Some((
            false,
            format!("Updating {} in its existing Wine prefix…", app.name),
        ));
        thread::spawn(move || {
            let result = runtime::run_update_installer(
                &root,
                wine.as_ref(),
                &app,
                &profile,
                &installer,
                &control,
            );
            let _ = sender.send(Event::AppUpdated(id, result));
        });
    }

    fn install_ge_async(&mut self, release: Release) {
        let Some(root) = self.store.as_ref().map(|store| store.root().to_path_buf()) else {
            return;
        };
        if self.ge_busy {
            return;
        }
        self.ge_busy = true;
        let control = OperationControl::default();
        self.ge_control = Some(control.clone());
        self.notice = Some((false, format!("Downloading and verifying {}…", release.tag)));
        let sender = self.sender.clone();
        thread::spawn(move || {
            let _ = sender.send(Event::GeInstalled(updates::install_ge_proton(
                &root, &release, &control,
            )));
        });
    }

    fn runner_in_use(&self, path: &std::path::Path) -> bool {
        let canonical_runner = std::fs::canonicalize(path).ok();
        self.store.as_ref().is_some_and(|store| {
            store.applications().iter().any(|app| {
                match runtime::load_profile(&app.prefix) {
                    Ok(profile) => match profile.runner {
                        Runner::ProtonUmu { proton, .. } => {
                            std::fs::canonicalize(&proton).ok() == canonical_runner
                        }
                        Runner::CustomWine { wine } => {
                            std::fs::canonicalize(wine).ok().is_some_and(|binary| {
                                canonical_runner
                                    .as_ref()
                                    .is_some_and(|root| binary.starts_with(root))
                            })
                        }
                        _ => false,
                    },
                    // Corrupt profiles must not lead to deleting an in-use runner.
                    Err(_) => true,
                }
            })
        })
    }

    fn remove_managed_runner_async(&mut self, path: PathBuf) {
        let Some(root) = self.store.as_ref().map(|store| store.root().to_path_buf()) else {
            return;
        };
        if self.ge_busy || self.runner_in_use(&path) || !updates::is_managed_runner(&root, &path) {
            return;
        }
        self.ge_busy = true;
        let sender = self.sender.clone();
        thread::spawn(move || {
            let result = updates::remove_managed_runner(&root, &path);
            let _ = sender.send(Event::GeRemoved(path, result));
        });
    }

    fn cleanup_async(&mut self, app: Application) {
        let Some(root) = self.store.as_ref().map(|store| store.root().to_path_buf()) else {
            return;
        };
        if self.storage_busy.contains(&app.id)
            || self.running.contains(&app.id)
            || self.compatibility_busy.contains(&app.id)
            || self.backup_busy
            || self.busy
        {
            return;
        }
        let id = app.id.clone();
        self.storage_busy.insert(id.clone());
        let sender = self.sender.clone();
        thread::spawn(move || {
            let _ = sender.send(Event::Cleaned(id, storage::cleanup_old_temp(&root, &app)));
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
                        .color(self.theme.palette().muted),
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
                ui.label(
                    RichText::new("RICE")
                        .small()
                        .strong()
                        .color(self.theme.palette().muted),
                );
                egui::ComboBox::from_id_salt("sidebar_rice")
                    .width(ui.available_width())
                    .selected_text(self.theme.label())
                    .show_ui(ui, |ui| {
                        for theme in Theme::ALL {
                            if ui
                                .selectable_label(self.theme == theme, theme.label())
                                .clicked()
                            {
                                self.set_theme(theme, ui.ctx());
                            }
                        }
                    });
            });
    }

    fn luna_sidebar(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(self.theme.sidebar())
            .inner_margin(egui::Margin::same(10))
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(89, 131, 207)))
            .show(ui, |ui| {
                ui.set_min_width(194.0);
                ui.set_min_height((ui.available_height() - 22.0).max(300.0));
                luna::brand(ui);
                luna::task_panel(ui, |ui| {
                    luna::task_header(ui, "Program tasks");
                    ui.add_space(4.0);
                    if luna::task_link(
                        ui,
                        "Program library",
                        luna::XpIcon::Programs,
                        self.page == Page::Library,
                    )
                    .clicked()
                    {
                        self.page = Page::Library;
                    }
                    if ui
                        .add_enabled_ui(!self.busy && self.store.is_some(), |ui| {
                            luna::task_link(ui, "Install program", luna::XpIcon::Install, false)
                        })
                        .inner
                        .clicked()
                    {
                        self.install = Some(InstallDraft::default());
                    }
                    if ui
                        .add_enabled_ui(!self.busy && self.store.is_some(), |ui| {
                            luna::task_link(ui, "Import prefix", luna::XpIcon::Import, false)
                        })
                        .inner
                        .clicked()
                    {
                        self.import = Some(ImportDraft::default());
                    }
                    ui.add_space(5.0);
                });
                ui.add_space(11.0);
                luna::task_panel(ui, |ui| {
                    luna::task_header(ui, "Other places");
                    ui.add_space(4.0);
                    for (page, icon) in [
                        (Page::Prefixes, luna::XpIcon::Prefix),
                        (Page::Runners, luna::XpIcon::Runner),
                        (Page::Settings, luna::XpIcon::Settings),
                    ] {
                        if luna::task_link(ui, page.label(), icon, self.page == page).clicked() {
                            self.page = page;
                        }
                    }
                    ui.add_space(5.0);
                });
                ui.add_space(11.0);
                luna::task_panel(ui, |ui| {
                    luna::task_header(ui, "RiceWine");
                    ui.add_space(4.0);
                    if luna::task_link(ui, "About", luna::XpIcon::Info, false).clicked() {
                        self.about_open = true;
                    }
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        ui.label(RichText::new("Visual style").size(11.0));
                    });
                    ui.horizontal(|ui| {
                        ui.add_space(9.0);
                        egui::ComboBox::from_id_salt("luna_sidebar_rice")
                            .width(157.0)
                            .selected_text(self.theme.label())
                            .show_ui(ui, |ui| {
                                for theme in Theme::ALL {
                                    if ui
                                        .selectable_label(self.theme == theme, theme.label())
                                        .clicked()
                                    {
                                        self.set_theme(theme, ui.ctx());
                                    }
                                }
                            });
                    });
                    ui.add_space(8.0);
                });
                ui.add_space(16.0);
                ui.label(
                    RichText::new("Windows compatibility manager")
                        .size(10.0)
                        .color(Color32::from_rgb(41, 71, 119)),
                );
            });
    }

    fn page_body(&mut self, ui: &mut egui::Ui) {
        if let Some((is_error, message)) = self.notice.clone() {
            if self.theme == Theme::Luna {
                if luna::notice(ui, is_error, &message) {
                    self.notice = None;
                }
            } else {
                let color = if is_error {
                    self.theme.error()
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
        }
        match self.page {
            Page::Library => self.library(ui),
            Page::Prefixes => self.prefixes(ui),
            Page::Runners => {
                egui::ScrollArea::vertical().show(ui, |ui| self.runners(ui));
            }
            Page::Settings => self.settings(ui),
        }
    }

    fn luna_shell(&mut self, ui: &mut egui::Ui) {
        match luna::application_menu(ui) {
            luna::MenuAction::Install if !self.busy && self.store.is_some() => {
                self.install = Some(InstallDraft::default());
            }
            luna::MenuAction::Import if !self.busy && self.store.is_some() => {
                self.import = Some(ImportDraft::default());
            }
            luna::MenuAction::Library => self.page = Page::Library,
            luna::MenuAction::Prefixes => self.page = Page::Prefixes,
            luna::MenuAction::Runners => self.page = Page::Runners,
            luna::MenuAction::Settings => self.page = Page::Settings,
            luna::MenuAction::About => self.about_open = true,
            _ => {}
        }
        match luna::application_toolbar(ui, self.page.label()) {
            luna::MenuAction::Library => self.page = Page::Library,
            luna::MenuAction::Prefixes => self.page = Page::Prefixes,
            luna::MenuAction::Runners => self.page = Page::Runners,
            luna::MenuAction::Settings => self.page = Page::Settings,
            _ => {}
        }
        ui.add_space(5.0);
        let available_height = (ui.available_height() - 27.0).max(250.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(225.0, available_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| self.luna_sidebar(ui),
            );
            ui.add_space(6.0);
            let content_width = ui.available_width().max(390.0);
            ui.allocate_ui_with_layout(
                egui::vec2(content_width, available_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    egui::Frame::new()
                        .fill(Color32::WHITE)
                        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(117, 145, 191)))
                        .inner_margin(11.0)
                        .corner_radius(egui::CornerRadius::same(2))
                        .show(ui, |ui| {
                            ui.set_min_height((available_height - 28.0).max(380.0));
                            let (title, subtitle) = match self.page {
                                Page::Library => {
                                    ("Program Library", "Manage your Windows applications")
                                }
                                Page::Prefixes => {
                                    ("Wine Prefixes", "Back up and restore environments")
                                }
                                Page::Runners => {
                                    ("Compatibility Runners", "Wine and Proton installations")
                                }
                                Page::Settings => (
                                    "RiceWine Settings",
                                    "Choose the look and integration options",
                                ),
                            };
                            luna::content_heading(ui, title, subtitle);
                            ui.add_space(9.0);
                            self.page_body(ui);
                        });
                },
            );
        });
        let state = match self.page {
            Page::Library => format!(
                "{} installed program(s)",
                self.store
                    .as_ref()
                    .map_or(0, |store| store.applications().len())
            ),
            Page::Prefixes => "Wine prefix management".to_owned(),
            Page::Runners => "Runner configuration".to_owned(),
            Page::Settings => "RiceWine preferences".to_owned(),
        };
        luna::status_bar(ui, &state);
    }

    fn library(&mut self, ui: &mut egui::Ui) {
        let luna_mode = self.theme == Theme::Luna;
        if luna_mode {
            luna::section_heading(ui, "Installed Windows programs", luna::XpIcon::Programs);
        }
        ui.horizontal(|ui| {
            if self.theme != Theme::Luna {
                ui.heading("Library");
                ui.add_space(20.0);
            }
            if self.theme != Theme::Luna {
                if ui
                    .add_enabled(
                        !self.busy && self.store.is_some(),
                        egui::Button::new("＋ Install program"),
                    )
                    .clicked()
                {
                    self.install = Some(InstallDraft::default());
                }
                if ui
                    .add_enabled(
                        !self.busy && self.store.is_some(),
                        egui::Button::new("Import existing prefix…"),
                    )
                    .clicked()
                {
                    self.import = Some(ImportDraft::default());
                }
            }
            if self.busy {
                ui.spinner();
                ui.label("Installing…");
                if ui.button("Cancel installation…").clicked() {
                    self.cancel_install_confirmation = true;
                }
            }
        });
        ui.add_space(if luna_mode { 5.0 } else { 10.0 });
        ui.horizontal(|ui| {
            ui.label(if luna_mode { "Search for:" } else { "Search" });
            ui.add(
                egui::TextEdit::singleline(&mut self.search)
                    .hint_text("Filter installed programs…")
                    .desired_width(210.0),
            );
            if !self.search.is_empty()
                && luna::action(ui, "Clear", true, luna_mode).clicked()
            {
                self.search.clear();
            }
        });
        if self.store.is_none() {
            ui.add_space(28.0);
            ui.heading("Library unavailable");
            ui.label("RiceWine could not load the catalog. Review the error above and preserve the original data before attempting recovery.");
            return;
        }
        if self
            .store
            .as_ref()
            .is_none_or(|s| s.applications().is_empty())
        {
            if self.theme == Theme::Luna {
                ui.add_space(7.0);
                luna::empty_library(ui);
            } else {
                ui.add_space(45.0);
                ui.heading("Your library is empty");
                ui.label("Install a Windows program or add a portable EXE to get started.");
            }
            return;
        }
        let term = self.search.trim().to_lowercase();
        // Only lightweight indexes are allocated per frame. Full Application
        // objects (with multiple paths/strings) are not cloned for the list.
        let matching_indices: Vec<usize> = self
            .store
            .as_ref()
            .map(|store| {
                store
                    .applications()
                    .iter()
                    .enumerate()
                    .filter(|(_, app)| term.is_empty() || app.name.to_lowercase().contains(&term))
                    .map(|(index, _)| index)
                    .collect()
            })
            .unwrap_or_default();
        ui.small(format!("{} matching program(s)", matching_indices.len()));
        if matching_indices.is_empty() {
            ui.label("No matching programs. Clear the search to show the library.");
            return;
        }
        let width = (ui.available_width() - 280.0).max(260.0);
        let mut next_selected: Option<String> = None;
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(width, ui.available_height()),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    if self.theme == Theme::Luna {
                        luna::program_list_header(ui);
                    } else {
                        egui::Frame::new()
                            .fill(self.theme.palette().button)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("Program")
                                            .strong()
                                            .color(self.theme.palette().text),
                                    );
                                    ui.add_space(80.0);
                                    ui.label(
                                        RichText::new("Runner")
                                            .strong()
                                            .color(self.theme.palette().text),
                                    );
                                });
                            });
                    }
                    let row_height = if self.theme == Theme::Luna {
                        28.0
                    } else {
                        42.0
                    };
                    egui::ScrollArea::vertical().show_rows(
                        ui,
                        row_height,
                        matching_indices.len(),
                        |ui, range| {
                            for index in range {
                                let app = &self
                                    .store
                                    .as_ref()
                                    .expect("library is available while displaying rows")
                                    .applications()[matching_indices[index]];
                                let selected = self.selected.as_deref() == Some(&app.id);
                                ui.horizontal(|ui| {
                                    let response = if self.theme == Theme::Luna {
                                        luna::program_row(ui, &app.name, selected, width - 112.0)
                                    } else {
                                        ui.add_sized(
                                            [width - 112.0, 42.0],
                                            egui::Button::selectable(selected, &app.name),
                                        )
                                    };
                                    if response.clicked() {
                                        next_selected = Some(app.id.clone());
                                    }
                                    let label = runtime::load_profile(&app.prefix)
                                        .map(|profile| profile.runner.label())
                                        .unwrap_or("Invalid profile");
                                    ui.label(label);
                                });
                            }
                        },
                    );
                },
            );
            ui.separator();
            ui.vertical(|ui| {
                ui.set_min_width(235.0);
                let selected = self
                    .selected
                    .as_ref()
                    .and_then(|id| self.store.as_ref()?.get(id))
                    .cloned();
                if let Some(app) = selected {
                    ui.heading(&app.name);
                    ui.label("Display name");
                    ui.text_edit_singleline(&mut self.name_edit);
                    if ui
                        .add_enabled(
                            !self.busy && !self.running.contains(&app.id),
                            egui::Button::new("Save name"),
                        )
                        .clicked()
                        && let Some(store) = &mut self.store
                    {
                        match store.rename(&app.id, &self.name_edit) {
                            Ok(()) => {
                                self.notice = Some((false, "Program name updated".into()));
                                let updated = store.get(&app.id).cloned();
                                if let Some(updated) = updated
                                    && let Ok(path) = integration::application_directory()
                                    && path.join(format!("ricewine-{}.desktop", app.id)).exists()
                                    && let Ok(binary) = std::env::current_exe()
                                    && let Err(error) =
                                        integration::install_shortcut(&updated, &binary)
                                {
                                    self.notice = Some((
                                        true,
                                        format!("Name saved, shortcut update failed: {error}"),
                                    ));
                                }
                            }
                            Err(error) => self.notice = Some((true, error)),
                        }
                    }
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Runner")
                            .small()
                            .strong()
                            .color(self.theme.palette().text),
                    );
                    let selected_profile = runtime::load_profile(&app.prefix);
                    match &selected_profile {
                        Ok(profile) => ui.label(profile.runner.label()),
                        Err(error) => ui.colored_label(self.theme.error(), error),
                    };
                    ui.add_space(5.0);
                    ui.label(
                        RichText::new("Prefix")
                            .small()
                            .strong()
                            .color(self.theme.palette().text),
                    );
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
                        if ui.button("■ Stop program…").clicked() {
                            self.stop_confirmation = Some(app.id.clone());
                        }
                    }
                    if app.executable.is_none() {
                        ui.small("Find or choose the installed EXE to enable Play.");
                    }
                    if ui.button("Scan installed EXEs").clicked() {
                        match discovery::scan_executables(&app.prefix, &app.name) {
                            Ok(candidates) => {
                                self.exe_candidates = candidates;
                                self.exe_candidate_for = Some(app.id.clone());
                                if self.exe_candidates.is_empty() {
                                    self.notice = Some((
                                        false,
                                        "No program executables detected; choose a file manually"
                                            .into(),
                                    ));
                                }
                            }
                            Err(error) => self.notice = Some((true, error)),
                        }
                    }
                    if self.exe_candidate_for.as_deref() == Some(&app.id) {
                        for candidate in self.exe_candidates.iter().take(12) {
                            let chosen = candidate.path.clone();
                            let label = chosen.file_name().unwrap_or_default().to_string_lossy();
                            if ui
                                .button(format!("Use {label}"))
                                .on_hover_text(core::readable_path(&chosen))
                                .clicked()
                                && let Some(store) = self.store.as_mut()
                            {
                                self.notice = Some(match store.set_executable(&app.id, chosen) {
                                    Ok(()) => (false, "Launch EXE selected".into()),
                                    Err(error) => (true, error),
                                });
                            }
                        }
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
                    if ui.button("Add application-menu shortcut").clicked() {
                        let outcome = std::env::current_exe()
                            .map_err(|e| e.to_string())
                            .and_then(|bin| integration::install_shortcut(&app, &bin));
                        self.notice = Some(match outcome {
                            Ok(file) => (false, format!("Shortcut created: {}", file.display())),
                            Err(error) => (true, error),
                        });
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
        if let Some(id) = next_selected {
            self.selected = Some(id);
            self.exe_candidates.clear();
            self.exe_candidate_for = None;
            self.sync_runner_draft();
        }
    }

    fn prefixes(&mut self, ui: &mut egui::Ui) {
        let luna_mode = self.theme == Theme::Luna;
        if luna_mode {
            luna::section_heading(ui, "Wine prefix maintenance", luna::XpIcon::Prefix);
            ui.small(
                "Back up your Windows environments before changing runners or components.",
            );
        } else {
            ui.heading("Wine prefixes");
            ui.label(
                "Create a backup before runner changes. Restoring never overwrites a live prefix.",
            );
        }
        ui.add_space(9.0);
        let apps = self
            .store
            .as_ref()
            .map(|store| store.applications().to_vec())
            .unwrap_or_default();
        egui::ScrollArea::vertical().show(ui, |ui| {
            if apps.is_empty() && luna_mode {
                egui::Frame::new()
                    .fill(Color32::from_rgb(249, 250, 252))
                    .stroke(egui::Stroke::new(
                        1.0,
                        Color32::from_rgb(170, 185, 205),
                    ))
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        ui.label("No Wine prefixes are registered yet.");
                        ui.small("Install or import a program from the Program tasks pane.");
                    });
            }
            for app in apps {
                let app_name = app.name.clone();
                let render = |ui: &mut egui::Ui| {
                    if !luna_mode {
                        ui.label(
                            RichText::new(&app.name)
                                .strong()
                                .color(self.theme.palette().text),
                        );
                    }
                    ui.small(core::readable_path(&app.prefix));
                    if let Some(usage) = self.disk_usage.get(&app.id) {
                        ui.label(format!(
                            "Disk usage: {} across {} files",
                            storage::human_size(usage.bytes),
                            usage.files
                        ));
                    }
                    if self.storage_busy.contains(&app.id) {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.small("Checking files…");
                        });
                    }
                    let idle = !self.busy
                        && !self.backup_busy
                        && !self.running.contains(&app.id)
                        && !self.compatibility_busy.contains(&app.id)
                        && !self.storage_busy.contains(&app.id);
                    ui.horizontal_wrapped(|ui| {
                        if luna::action(ui, "Calculate size", idle, luna_mode).clicked() {
                            self.measure_storage(app.clone(), false);
                        }
                        if luna::action(ui, "Preview cleanup…", idle, luna_mode).clicked() {
                            self.measure_storage(app.clone(), true);
                        }
                        let can_backup = !self.backup_busy
                            && !self.running.contains(&app.id)
                            && !self.compatibility_busy.contains(&app.id)
                            && !self.busy;
                        if luna::action(ui, "Back up prefix", can_backup, luna_mode).clicked() {
                            self.backup_async(app.clone());
                        }
                    });
                };
                if luna_mode {
                    luna::option_group(ui, &app_name, luna::XpIcon::Prefix, render);
                } else {
                    ui.group(render);
                    ui.add_space(8.0);
                }
            }

            if luna_mode {
                luna::section_heading(ui, "Backup archives", luna::XpIcon::Disk);
            } else {
                ui.add_space(12.0);
                ui.heading("Backups");
            }
            if luna::action(ui, "Refresh backup list", true, luna_mode).clicked()
                && let Some(store) = &self.store
            {
                match backup::list_backups(store.root()) {
                    Ok(items) => self.backups = items,
                    Err(error) => self.notice = Some((true, error)),
                }
            }
            if self.backup_busy {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Copying prefix files…");
                });
            }
            if self.backups.is_empty() {
                ui.small("No backup archives found.");
            }
            for entry in self.backups.clone() {
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("{} — {}", entry.app_name, entry.created_unix));
                    if luna::action(
                        ui,
                        "Restore as new program",
                        !self.backup_busy && !self.busy && self.running.is_empty(),
                        luna_mode,
                    )
                    .clicked()
                    {
                        self.restore_async(entry.clone());
                    }
                });
            }
        });
    }

    fn runners(&mut self, ui: &mut egui::Ui) {
        let luna_mode = self.theme == Theme::Luna;
        if luna_mode {
            luna::section_heading(ui, "Installed runtime tools", luna::XpIcon::Runner);
        } else {
            ui.heading("Runners");
        }
        ui.small("Wine and Proton are configured independently for each Windows program.");
        ui.add_space(7.0);
        let diagnostics = runtime::diagnostics(self.wine.as_ref());
        if luna_mode {
            egui::Frame::new()
                .fill(Color32::from_rgb(248, 250, 254))
                .stroke(egui::Stroke::new(
                    1.0,
                    Color32::from_rgb(170, 187, 212),
                ))
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.set_min_width((ui.available_width() - 20.0).max(0.0));
                    for line in &diagnostics {
                        ui.label(RichText::new(line).size(11.0));
                    }
                });
        } else {
            for line in diagnostics {
                ui.small(line);
            }
        }
        ui.add_space(9.0);
        if luna_mode {
            luna::section_heading(ui, "GE-Proton manager", luna::XpIcon::Install);
        } else {
            ui.label(
                RichText::new("GE-Proton manager")
                    .strong()
                    .color(self.theme.palette().text),
            );
        }
        ui.small("Opt-in official GE-Proton releases. Downloads are SHA512-verified and installed only under RiceWine's data directory; no sudo or Steam changes.");
        ui.horizontal(|ui| {
            if luna::action(ui, "Check latest GE-Proton", !self.ge_busy, luna_mode)
                .clicked()
            {
                self.check_ge_release();
            }
            if let Some(release) = &self.ge_release {
                ui.label(format!("Latest: {}", release.tag));
                if luna::action(ui, "Install verified release…", !self.ge_busy, luna_mode)
                    .clicked()
                {
                    self.ge_install_confirmation = true;
                }
            }
            if self.ge_busy {
                ui.spinner();
                ui.label("Checking or downloading…");
                if let Some(control) = &self.ge_control
                    && ui.button("Cancel download").clicked()
                {
                    control.cancel();
                }
            }
        });
        ui.small("Requires curl, tar, sha512sum and (for launching) umu-run. Existing installations are not overwritten.");
        if luna::action(ui, "Find locally installed Proton versions", true, luna_mode)
            .clicked()
        {
            self.detected_proton = std::env::var_os("HOME")
                .map(|home| runtime::discover_proton(&PathBuf::from(home)))
                .unwrap_or_default();
            if let Some(store) = &self.store {
                for path in updates::managed_proton(store.root()) {
                    if !self.detected_proton.iter().any(|item| item.path == path) {
                        self.detected_proton.push(runtime::DetectedProton {
                            name: path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into_owned(),
                            path,
                        });
                    }
                }
            }
            if self.detected_proton.is_empty() {
                self.notice = Some((
                    false,
                    "No extracted Proton builds found in common Steam locations".into(),
                ));
            }
        }
        for candidate in &self.detected_proton {
            let managed = self
                .store
                .as_ref()
                .is_some_and(|store| updates::is_managed_runner(store.root(), &candidate.path));
            let removable = managed && !self.runner_in_use(&candidate.path);
            ui.horizontal(|ui| {
                ui.label(&candidate.name);
                if luna::action(ui, "Use for selected program", true, luna_mode).clicked() {
                    self.runner_draft.runner_kind = 2;
                    self.runner_draft.proton_dir = core::readable_path(&candidate.path);
                    if self.runner_draft.game_id.trim().is_empty() {
                        self.runner_draft.game_id = "0".into();
                    }
                }
                if managed
                    && luna::action(
                        ui,
                        "Remove managed runner…",
                        !self.ge_busy && removable,
                        luna_mode,
                    )
                    .clicked()
                {
                    self.ge_remove_confirmation = Some(candidate.path.clone());
                }
            });
        }
        ui.add_space(7.0);
        if luna_mode {
            luna::section_heading(ui, "Selected program configuration", luna::XpIcon::Programs);
        } else {
            ui.separator();
        }
        let Some(app) = self
            .selected
            .as_ref()
            .and_then(|id| self.store.as_ref()?.get(id))
            .cloned()
        else {
            if luna_mode {
                ui.label("No program selected.");
                ui.small("Open Program library, choose a Windows app, then return to Runners.");
                if luna::action(ui, "Open library", true, true).clicked() {
                    self.page = Page::Library;
                }
            } else {
                ui.label("Add and select a program in Library to configure a runner.");
            }
            return;
        };
        if luna_mode {
            ui.label(RichText::new(format!("Program: {}", app.name)).strong());
        } else {
            ui.heading(format!("Program: {}", app.name));
        }
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
        ui.label("Launch arguments (one complete argument per line)");
        ui.add(
            egui::TextEdit::multiline(&mut self.launch_args_text)
                .desired_rows(2)
                .desired_width(400.0),
        );
        ui.label("Optional environment overrides (KEY=value, one per line)");
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("compatibility_presets")
                .selected_text(CompatibilityPreset::ALL[self.preset_choice].label())
                .show_ui(ui, |ui| {
                    for (index, preset) in CompatibilityPreset::ALL.iter().enumerate() {
                        ui.selectable_value(&mut self.preset_choice, index, preset.label());
                    }
                });
            if ui
                .add_enabled(
                    !self.running.contains(&app.id) && !self.busy,
                    egui::Button::new("Apply preset to draft"),
                )
                .clicked()
            {
                let preset = CompatibilityPreset::ALL[self.preset_choice];
                self.launch_env_text = preset
                    .environment()
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                self.notice = Some((
                    false,
                    "Preset loaded into draft; click Save runner to persist".into(),
                ));
            }
        });
        ui.small(CompatibilityPreset::ALL[self.preset_choice].description());
        ui.add(
            egui::TextEdit::multiline(&mut self.launch_env_text)
                .desired_rows(2)
                .desired_width(400.0),
        );
        ui.small(format!(
            "Allowed keys: {}",
            runtime::ALLOWED_ENVIRONMENT.join(", ")
        ));
        ui.add_space(14.0);
        if luna_mode {
            luna::section_heading(ui, "Application updates", luna::XpIcon::Import);
        } else {
            ui.label(
                RichText::new("Application updates")
                    .strong()
                    .color(self.theme.palette().text),
            );
        }
        ui.small("Optional release notices for apps published on GitHub. RiceWine never downloads or runs a publisher EXE without your explicit selection.");
        ui.label("GitHub repository (owner/repo)");
        ui.add(
            egui::TextEdit::singleline(&mut self.update_repo_text)
                .desired_width(350.0)
                .hint_text("owner/repository (optional)"),
        );
        ui.label("Currently installed version (manual label)");
        ui.add(
            egui::TextEdit::singleline(&mut self.installed_version_text)
                .desired_width(250.0)
                .hint_text("e.g. v1.2.3"),
        );
        let working = self.running.contains(&app.id)
            || self.compatibility_busy.contains(&app.id)
            || self.storage_busy.contains(&app.id)
            || self.backup_busy
            || self.busy;
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!working, egui::Button::new("Save runner for program"))
                .clicked()
            {
                let result = runtime::load_profile(&app.prefix).and_then(|mut profile| {
                    profile.runner = self.runner_draft.runner();
                    profile.update_repository = (!self.update_repo_text.trim().is_empty())
                        .then(|| self.update_repo_text.trim().to_owned());
                    profile.installed_version = self.installed_version_text.trim().to_owned();
                    runtime::parse_launch_configuration(
                        &self.launch_args_text,
                        &self.launch_env_text,
                        &mut profile,
                    )?;
                    runtime::save_profile(&app.prefix, &profile)
                });
                match result {
                    Ok(()) => {
                        self.latest_app_releases.remove(&app.id);
                        self.notice = Some((false, format!("Runner saved for {}", app.name)));
                    }
                    Err(error) => self.notice = Some((true, error)),
                }
            }
            if luna::action(ui, "Reload saved runner", true, luna_mode).clicked() {
                self.sync_runner_draft();
            }
        });
        ui.add_space(12.0);
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(!working, egui::Button::new("Check saved GitHub release"))
                .clicked()
            {
                self.check_app_release(&app);
            }
            if let Some(release) = self.latest_app_releases.get(&app.id) {
                ui.label(format!("Latest: {}", release.tag));
                if ui.button("Open release page").clicked()
                    && let Err(error) = std::process::Command::new("xdg-open")
                        .arg(&release.url)
                        .spawn()
                {
                    self.notice = Some((true, format!("Opening release page failed: {error}")));
                }
            }
        });
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    !working,
                    egui::Button::new("Run update installer in this prefix…"),
                )
                .clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("Windows installer", &["exe", "msi"])
                    .pick_file()
            {
                self.update_install_confirmation = Some((app.id.clone(), path));
            }
            if let Some(control) = self.update_controls.get(&app.id)
                && ui.button("Cancel running update").clicked()
            {
                control.cancel();
            }
        });
        ui.small("A publisher update can overwrite installed app files. Back up the prefix first. Successful setup does not prove the app version changed.");
        ui.add_space(12.0);
        ui.separator();
        if luna_mode {
            luna::section_heading(ui, "Compatibility components", luna::XpIcon::Settings);
        } else {
            ui.label(
                RichText::new("Compatibility tools")
                    .strong()
                    .color(self.theme.palette().text),
            );
        }
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
        if luna_mode {
            luna::section_heading(ui, "Diagnostic log", luna::XpIcon::Disk);
        }
        if luna::action(ui, "Read recent Wine log", true, luna_mode).clicked() {
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

    /// XP Luna Control Panel-style settings; other Rices retain their layouts.
    fn luna_settings(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            luna::option_group(ui, "Appearance", luna::XpIcon::Settings, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Visual style:");
                    egui::ComboBox::from_id_salt("luna_settings_theme")
                        .selected_text(self.theme.label())
                        .width(195.0)
                        .show_ui(ui, |ui| {
                            for theme in Theme::ALL {
                                if ui
                                    .selectable_label(self.theme == theme, theme.label())
                                    .clicked()
                                {
                                    self.set_theme(theme, ui.ctx());
                                }
                            }
                        });
                });
                ui.add_space(5.0);
                ui.small(
                    "Changes affect RiceWine only. The desktop window frame is managed by RiceBowl.",
                );
                ui.add_space(8.0);
                let mut follow = self.preferences.follow_desktop;
                if ui.checkbox(&mut follow, "Follow a recognized RiceBowl desktop style").changed() {
                    self.preferences.follow_desktop = follow;
                    if follow {
                        if let Some(appearance) = preferences::detect_desktop_appearance() {
                            self.theme = appearance.into();
                            self.theme.apply(ui.ctx());
                        } else {
                            self.notice = Some((
                                false,
                                "No supported desktop style detected; appearance unchanged".into(),
                            ));
                        }
                    }
                    self.persist_preferences();
                }
                if self.preferences.follow_desktop
                    && luna::xp_button(ui, "Detect now", true).clicked()
                {
                    if let Some(appearance) = preferences::detect_desktop_appearance() {
                        self.theme = appearance.into();
                        self.theme.apply(ui.ctx());
                        self.notice = Some((false, "Desktop appearance refreshed".into()));
                    } else {
                        self.notice = Some((
                            false,
                            "Desktop style not recognized; appearance unchanged".into(),
                        ));
                    }
                }
            });

            luna::option_group(ui, "Windows program integration", luna::XpIcon::Install, |ui| {
                ui.label("Add RiceWine to the file manager's Open With menu.");
                ui.add_space(6.0);
                if luna::xp_button(ui, "Add Open With entry", true).clicked() {
                    let result = std::env::current_exe()
                        .map_err(|error| error.to_string())
                        .and_then(|binary| integration::install_open_with(&binary));
                    self.notice = Some(match result {
                        Ok(path) => (false, format!("Open With entry created: {}", path.display())),
                        Err(error) => (true, error),
                    });
                }
                ui.small("Existing system defaults and EXE associations are not replaced.");
            });

            luna::option_group(ui, "Library maintenance", luna::XpIcon::Disk, |ui| {
                let Some(store) = &self.store else {
                    ui.label("Catalog unavailable. See the error above.");
                    return;
                };
                ui.label("Application data directory:");
                ui.small(core::readable_path(store.root()));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if luna::xp_button(ui, "Verify library", true).clicked() {
                        self.library_report = library::check(store.applications())
                            .lines()
                            .join("\n");
                    }
                    if luna::xp_button(ui, "Save metadata snapshot", true).clicked() {
                        self.notice = Some(match library::snapshot_metadata(store.root()) {
                            Ok(path) => (false, format!("Metadata saved: {}", path.display())),
                            Err(error) => (true, error),
                        });
                    }
                });
                ui.small("Verification does not modify files. Snapshots exclude Wine prefixes.");
                if !self.library_report.is_empty() {
                    ui.add_space(5.0);
                    ui.add(
                        egui::TextEdit::multiline(&mut self.library_report)
                            .desired_rows(5)
                            .desired_width(f32::INFINITY)
                            .interactive(false),
                    );
                }
            });
        });
    }

    fn settings(&mut self, ui: &mut egui::Ui) {
        if self.theme == Theme::Luna {
            self.luna_settings(ui);
            return;
        }
        if self.theme != Theme::Luna {
            ui.heading("Settings");
        }
        ui.label(
            RichText::new("Rices")
                .strong()
                .color(self.theme.palette().text),
        );
        ui.small("Five lightweight built-in looks. Switching does not change Wine or your desktop theme.");
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            for theme in Theme::ALL {
                let palette = theme.palette();
                let selected = self.theme == theme;
                let button = egui::Button::new(RichText::new(theme.label()).color(palette.text))
                    .fill(palette.sidebar)
                    .stroke(egui::Stroke::new(
                        if selected { 2.0 } else { 1.0 },
                        if selected {
                            palette.accent
                        } else {
                            palette.border
                        },
                    ));
                if ui.add_sized([143.0, 42.0], button).clicked() {
                    self.set_theme(theme, ui.ctx());
                }
            }
        });
        ui.add_space(8.0);
        let mut follow = self.preferences.follow_desktop;
        if ui
            .checkbox(&mut follow, "Match recognized RiceBowl theme at startup")
            .changed()
        {
            self.preferences.follow_desktop = follow;
            if follow {
                if let Some(appearance) = preferences::detect_desktop_appearance() {
                    self.theme = appearance.into();
                    self.theme.apply(ui.ctx());
                } else {
                    self.notice = Some((
                        false,
                        "No supported RiceBowl theme detected; keeping current style".into(),
                    ));
                }
            }
            self.persist_preferences();
        }
        if self.preferences.follow_desktop && ui.button("Check desktop appearance again").clicked()
        {
            if let Some(appearance) = preferences::detect_desktop_appearance() {
                self.theme = appearance.into();
                self.theme.apply(ui.ctx());
                self.notice = Some((false, "Desktop appearance refreshed".into()));
            } else {
                self.notice = Some((
                    false,
                    "Desktop theme not recognized; appearance unchanged".into(),
                ));
            }
        }
        ui.add_space(16.0);
        ui.label("Desktop integration");
        if ui.button("Add RiceWine to Open With menu").clicked() {
            let result = std::env::current_exe()
                .map_err(|e| e.to_string())
                .and_then(|path| integration::install_open_with(&path));
            self.notice = Some(match result {
                Ok(path) => (
                    false,
                    format!("Open With entry created: {}", path.display()),
                ),
                Err(error) => (true, error),
            });
        }
        ui.small("Does not replace system file associations or change your default app.");
        ui.add_space(18.0);
        if let Some(store) = &self.store {
            ui.label("Application data");
            ui.small(core::readable_path(store.root()));
            ui.small("Catalog changes are saved atomically. Removing a program from the library preserves its files.");
            ui.add_space(12.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button("Verify library").clicked() {
                    self.library_report = library::check(store.applications()).lines().join("\n");
                }
                if ui.button("Snapshot library metadata").clicked() {
                    match library::snapshot_metadata(store.root()) {
                        Ok(path) => {
                            self.notice =
                                Some((false, format!("Metadata saved in {}", path.display())))
                        }
                        Err(error) => self.notice = Some((true, error)),
                    }
                }
            });
            ui.small("Snapshots preserve the catalog and appearance settings only; use Prefixes for full Wine backups. Verification never changes files.");
            if !self.library_report.is_empty() {
                ui.add(
                    egui::TextEdit::multiline(&mut self.library_report)
                        .desired_rows(8)
                        .desired_width(f32::INFINITY)
                        .interactive(false),
                );
            }
        }
    }

    fn install_dialog(&mut self, ctx: &egui::Context) {
        let mut submit = None;
        let mut keep_open = true;
        let luna_mode = self.theme == Theme::Luna;
        if let Some(draft) = &mut self.install {
            egui::Window::new("Install Windows program")
                .collapsible(false)
                .resizable(false)
                .open(&mut keep_open)
                .show(ctx, |ui| {
                    ui.set_min_width(460.0);
                    if luna_mode {
                        luna::dialog_intro(
                            ui,
                            luna::XpIcon::Install,
                            "Install a Windows program",
                            "Choose a setup file and its Wine environment.",
                        );
                    }
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
                    ui.label("Installation template");
                    egui::ComboBox::from_id_salt("installation_template")
                        .selected_text(draft.template.label())
                        .show_ui(ui, |ui| {
                            for template in [Template::Standard, Template::Games, Template::Legacy32] {
                                ui.selectable_value(&mut draft.template, template, template.label());
                            }
                        });
                    if draft.template == Template::Legacy32 {
                        ui.small("Creates a 32-bit Wine prefix. Requires 32-bit Wine support. Cannot be changed later.");
                    }
                    if draft.template == Template::Games {
                        ui.small("Games preset. DXVK/VKD3D can be added from Runners as needed.");
                    }
                    ui.label("Runner for this program");
                    ui.horizontal_wrapped(|ui| {
                        for (index, label) in ["System Wine", "Custom Wine", "Proton / umu"].iter().enumerate() {
                            ui.radio_value(&mut draft.runner_kind, index, *label);
                        }
                    });
                    Self::runner_fields(ui, draft);
                    ui.small("A fresh prefix does not sandbox Windows software. Only run applications you trust.");
                    ui.add_space(12.0);
                    let install_label = if draft.portable {
                        "Add portable program"
                    } else {
                        "Run installer"
                    };
                    let clicked = if luna_mode {
                        luna::xp_button(ui, install_label, true).clicked()
                    } else {
                        ui.button(install_label).clicked()
                    };
                    if clicked {
                        submit = Some(InstallDraft {
                            name: draft.name.clone(), file: draft.file.clone(), portable: draft.portable,
                            runner_kind: draft.runner_kind, custom_wine: draft.custom_wine.clone(),
                            proton_dir: draft.proton_dir.clone(), game_id: draft.game_id.clone(),
                            template: draft.template,
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

    fn import_dialog(&mut self, ctx: &egui::Context) {
        let mut keep_open = true;
        let mut imported = None;
        let luna_mode = self.theme == Theme::Luna;
        if let Some(draft) = &mut self.import {
            egui::Window::new("Import existing Wine prefix")
                .collapsible(false).resizable(false).open(&mut keep_open)
                .show(ctx, |ui| {
                    ui.set_min_width(450.0);
                    if luna_mode {
                        luna::dialog_intro(
                            ui,
                            luna::XpIcon::Import,
                            "Import an existing prefix",
                            "Register a program without moving any of its files.",
                        );
                    }
                    ui.label("Program name");
                    ui.text_edit_singleline(&mut draft.name);
                    ui.label("Existing Wine prefix directory");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut draft.prefix);
                        if ui.button("Browse…").clicked()
                            && let Some(path) = rfd::FileDialog::new().pick_folder()
                        {
                            draft.prefix = core::readable_path(&path);
                        }
                    });
                    ui.label("Existing launch EXE");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut draft.executable);
                        if ui.button("Browse…").clicked()
                            && let Some(path) = rfd::FileDialog::new().add_filter("Windows program", &["exe"]).pick_file()
                        {
                            draft.executable = core::readable_path(&path);
                        }
                    });
                    ui.small("Import only registers an existing prefix. Its files are never moved or deleted.");
                    let clicked = if luna_mode {
                        luna::xp_button(ui, "Import prefix", true).clicked()
                    } else {
                        ui.button("Import prefix").clicked()
                    };
                    if clicked {
                        imported = Some(core::imported_application(
                            &draft.name,
                            &PathBuf::from(draft.prefix.trim()),
                            &PathBuf::from(draft.executable.trim()),
                        ));
                    }
                });
        }
        if !keep_open {
            self.import = None;
        }
        if let Some(result) = imported {
            match result.and_then(|app| {
                let id = app.id.clone();
                self.store
                    .as_mut()
                    .ok_or("Catalog is unavailable")?
                    .add(app)?;
                Ok(id)
            }) {
                Ok(id) => {
                    self.selected = Some(id);
                    self.sync_runner_draft();
                    self.notice = Some((
                        false,
                        "Existing prefix imported. Original files were preserved.".into(),
                    ));
                    self.import = None;
                }
                Err(error) => self.notice = Some((true, error)),
            }
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
                                self.notice = Some(match integration::remove_shortcut(&id) {
                                    Ok(()) => (false, "Removed from library. Prefix preserved.".into()),
                                    Err(error) => (true, format!("Entry removed, but shortcut cleanup failed: {error}")),
                                });
                            }
                            Err(error) => self.notice = Some((true, error)),
                        }
                    }
                    self.remove_confirmation = None;
                }
            });
        });
    }

    fn process_dialogs(&mut self, ctx: &egui::Context) {
        if let Some(path) = self.ge_remove_confirmation.clone() {
            egui::Window::new("Remove managed GE-Proton runner?")
                .collapsible(false).resizable(false).show(ctx, |ui| {
                    ui.label(core::readable_path(&path));
                    ui.label("Only this RiceWine-downloaded runner will be deleted permanently. Prefixes stay intact, but removed runner files cannot be restored automatically.");
                    ui.horizontal(|ui| {
                        if ui.button("Keep runner").clicked() { self.ge_remove_confirmation = None; }
                        if ui.add_enabled(!self.runner_in_use(&path), egui::Button::new("Delete managed runner")).clicked() {
                            self.ge_remove_confirmation = None;
                            self.remove_managed_runner_async(path);
                        }
                    });
                });
        }
        if let Some((id, installer)) = self.update_install_confirmation.clone() {
            egui::Window::new("Run Windows update installer?").collapsible(false).resizable(false).show(ctx, |ui| {
                ui.label(core::readable_path(&installer));
                ui.label("The installer runs with the existing application's runner and prefix. Back up the prefix first; only install updates from trusted publishers.");
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() { self.update_install_confirmation = None; }
                    if ui.button("Run update now").clicked() {
                        self.update_install_confirmation = None;
                        if let Some(app) = self.store.as_ref().and_then(|store| store.get(&id)).cloned() {
                            self.update_app_async(app, installer);
                        }
                    }
                });
            });
        }
        if self.ge_install_confirmation {
            egui::Window::new("Download GE-Proton?").collapsible(false).resizable(false).show(ctx, |ui| {
                ui.label("The official release archive can be large. RiceWine verifies SHA512 before extraction and never overwrites an existing runner.");
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() { self.ge_install_confirmation = false; }
                    if ui.button("Download and install").clicked() {
                        self.ge_install_confirmation = false;
                        if let Some(release) = self.ge_release.clone() { self.install_ge_async(release); }
                    }
                });
            });
        }
        if let Some((id, preview)) = self.cleanup_preview.clone() {
            egui::Window::new("Clean old temporary files?").collapsible(false).resizable(false).show(ctx, |ui| {
                ui.label(format!("{} files / {} eligible (older than 7 days)", preview.files.len(), storage::human_size(preview.bytes)));
                ui.label("Only regular files inside temporary folders of a RiceWine-owned prefix. No program files, prefixes, directories or links are removed.");
                ui.horizontal(|ui| {
                    if ui.button("Keep files").clicked() { self.cleanup_preview = None; }
                    let app = self.store.as_ref().and_then(|store| store.get(&id)).cloned();
                    if ui.add_enabled(!preview.files.is_empty() && app.is_some() && !self.running.contains(&id), egui::Button::new("Delete listed temp files")).clicked() {
                        self.cleanup_preview = None;
                        if let Some(app) = app { self.cleanup_async(app); }
                    }
                });
            });
        }
        if self.cancel_install_confirmation {
            egui::Window::new("Cancel installation?").collapsible(false).resizable(false).show(ctx, |ui| {
                ui.label("Stop the installer and its process group. Partial Wine prefix data will be kept for recovery.");
                ui.horizontal(|ui| {
                    if ui.button("Keep installing").clicked() { self.cancel_install_confirmation = false; }
                    if ui.button("Stop installer").clicked() {
                        if let Some(control) = &self.install_control { control.cancel(); }
                        self.cancel_install_confirmation = false;
                        self.notice = Some((false, "Stopping installer…".into()));
                    }
                });
            });
        }
        if let Some(id) = self.stop_confirmation.clone() {
            egui::Window::new("Stop Windows program?")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Unsaved changes in the Windows program may be lost.");
                    ui.horizontal(|ui| {
                        if ui.button("Keep running").clicked() {
                            self.stop_confirmation = None;
                        }
                        if ui.button("Stop program").clicked() {
                            if let Some(control) = self.running_controls.get(&id) {
                                control.cancel();
                                self.notice = Some((false, "Stopping program…".into()));
                            }
                            self.stop_confirmation = None;
                        }
                    });
                });
        }
    }
}

impl eframe::App for RiceWineApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll(ui.ctx());
        let panel = if self.theme == Theme::Luna {
            egui::CentralPanel::default().frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(222, 234, 255))
                    .inner_margin(0.0),
            )
        } else {
            egui::CentralPanel::default()
        };
        panel.show(ui, |ui| {
            if self.theme == Theme::Luna {
                self.luna_shell(ui);
            } else {
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
                        self.page_body(ui);
                    });
                });
            }
        });
        self.install_dialog(ui.ctx());
        self.import_dialog(ui.ctx());
        self.remove_dialog(ui.ctx());
        self.process_dialogs(ui.ctx());
        if self.about_open {
            egui::Window::new("About RiceWine")
                .collapsible(false)
                .resizable(false)
                .open(&mut self.about_open)
                .show(ui.ctx(), |ui| {
                    ui.heading("RiceWine");
                    ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                    ui.label("Windows programs on Linux, managed with Wine and Proton.");
                    ui.label(
                        "XP Luna styles the application only. RiceBowl owns window decorations.",
                    );
                });
        }
    }
}

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let initial_install = if let [flag, file] = args.as_slice()
        && flag == "--install-file"
    {
        Some(PathBuf::from(file))
    } else {
        None
    };
    #[cfg(unix)]
    if let Some(file) = &initial_install {
        let handoff = core::data_directory().and_then(|root| ipc::try_forward_install(&root, file));
        match handoff {
            Ok(true) => return Ok(()),
            Ok(false) => {}
            Err(error) => {
                eprintln!("RiceWine Open With: {error}");
                std::process::exit(1);
            }
        }
    }
    if !args.is_empty() && initial_install.is_none() {
        let outcome = match args.as_slice() {
            [flag] if flag == "--help" => {
                println!(
                    "RiceWine {}\nUsage: ricewine [--launch APPLICATION_ID | --install-file FILE | --list | --diagnose | --verify-library | --snapshot-library | --register-menu | --unregister-menu | --unregister-open-with | --unregister-shortcuts | --version | --help]",
                    env!("CARGO_PKG_VERSION")
                );
                Ok(())
            }
            [flag] if flag == "--version" => {
                println!("RiceWine {}", env!("CARGO_PKG_VERSION"));
                Ok(())
            }
            [flag] if flag == "--register-menu" => std::env::current_exe()
                .map_err(|e| e.to_string())
                .and_then(|path| integration::install_main_menu(&path))
                .map(|path| println!("Application menu entry created: {}", path.display())),
            [flag] if flag == "--unregister-menu" => integration::remove_main_menu(),
            [flag] if flag == "--unregister-open-with" => integration::remove_open_with(),
            [flag] if flag == "--unregister-shortcuts" => {
                integration::remove_all_shortcuts().map(|count| {
                    println!("Removed {count} RiceWine program shortcuts");
                })
            }
            [flag] if flag == "--list" => core::Store::list_for_cli().map(|(_, apps)| {
                for app in apps {
                    println!("{}\t{}", app.id, app.name);
                }
            }),
            [flag] if flag == "--snapshot-library" => {
                core::Store::list_for_cli().and_then(|(root, _)| {
                    library::snapshot_metadata(&root).map(|path| {
                        println!("Library metadata snapshot: {}", path.display());
                        println!("Wine prefixes are NOT included; use per-prefix backups.");
                    })
                })
            }
            [flag] if flag == "--verify-library" => {
                core::Store::list_for_cli().and_then(|(_, apps)| {
                    let report = library::check(&apps);
                    for row in report.lines() {
                        println!("{row}");
                    }
                    if report.is_healthy() {
                        Ok(())
                    } else {
                        Err("Library verification found errors".into())
                    }
                })
            }
            [flag] if flag == "--diagnose" => {
                let system = WineTools::detect().ok();
                for row in runtime::diagnostics(system.as_ref()) {
                    println!("{row}");
                }
                core::Store::list_for_cli().map(|(_, apps)| {
                    println!("Library: {} program(s)", apps.len());
                    for app in &apps {
                        match runtime::load_profile(&app.prefix) {
                            Ok(profile) => println!("{}: {}", app.name, profile.runner.label()),
                            Err(error) => println!("{}: PROFILE ERROR: {error}", app.name),
                        }
                    }
                    for row in library::check(&apps).lines() {
                        println!("{row}");
                    }
                })
            }
            [flag, id] if flag == "--launch" => {
                core::Store::load_for_launch(id).and_then(|(root, app)| {
                    let profile = runtime::load_profile(&app.prefix)?;
                    let system = WineTools::detect().ok();
                    let mut child = runtime::launch(&root, system.as_ref(), &app, &profile)?;
                    child.wait().map_err(|e| e.to_string()).and_then(|status| {
                        if status.success() {
                            Ok(())
                        } else {
                            Err(format!("Program exited with {status}"))
                        }
                    })
                })
            }
            _ => Err("Unsupported arguments. Run ricewine --help".into()),
        };
        if let Err(error) = outcome {
            eprintln!("RiceWine: {error}");
            std::process::exit(1);
        }
        return Ok(());
    }
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
        Box::new(move |ctx| Ok(Box::new(RiceWineApp::new(ctx, initial_install.clone())))),
    )
}

#[cfg(test)]
mod rice_tests {
    use super::*;

    fn contrast(a: Color32, b: Color32) -> f32 {
        fn luminance(color: Color32) -> f32 {
            let channel = |value: u8| {
                let value = value as f32 / 255.0;
                if value <= 0.04045 {
                    value / 12.92
                } else {
                    ((value + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
        }
        let a = luminance(a);
        let b = luminance(b);
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    #[test]
    fn every_rice_has_readable_text_and_persists() {
        for theme in Theme::ALL {
            let palette = theme.palette();
            assert!(
                contrast(palette.text, palette.panel) >= 4.5,
                "{:?} panel",
                theme
            );
            assert!(
                contrast(palette.text, palette.sidebar) >= 4.5,
                "{:?} sidebar",
                theme
            );
            assert!(
                contrast(palette.text, palette.button) >= 4.5,
                "{:?} button",
                theme
            );
            let selected_text = if palette.dark {
                palette.panel
            } else {
                Color32::WHITE
            };
            assert!(
                contrast(selected_text, palette.accent) >= 4.5,
                "{:?} selection",
                theme
            );
            assert!(
                contrast(palette.accent, palette.sidebar) >= 3.0,
                "{:?} accent",
                theme
            );
            assert_eq!(Theme::from(Appearance::from(theme)), theme);
        }
    }
}
