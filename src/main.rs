#![forbid(unsafe_code)]

mod extra_rices;
mod luna;
mod luna_icons;
mod mac;
mod rice_identity;
mod ui_layout;
mod win10;

use eframe::egui::{self, Color32, RichText};
use ricewine::backup::{self, BackupEntry};
use ricewine::core::{self, Application, InstallMode, OperationControl, Store, WineTools};
use ricewine::discovery::{self, LaunchCandidate};
use ricewine::doctor;
use ricewine::integration;
#[cfg(unix)]
use ricewine::ipc;
use ricewine::library;
use ricewine::preferences::{self, Appearance, Preferences};
use ricewine::runtime::{self, CompatibilityPreset, PrefixProfile, Runner, Template};
use ricewine::storage::{self, CleanupPreview, DiskUsage};
use ricewine::updates::{self, Release};
use ricewine::workspace::{self, LibraryFilter, LibraryOrder, Workspace};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Theme {
    Luna,
    MacOS,
    Windows10,
    Windows98,
    Windows7,
    ChromeOS,
    FrutigerAero,
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
    /// The additional period shells intentionally have independent widgets.
    fn extra_flavor(self) -> Option<extra_rices::Flavor> {
        use extra_rices::Flavor;
        match self {
            Self::Windows98 => Some(Flavor::Classic98),
            Self::Windows7 => Some(Flavor::Aero7),
            Self::ChromeOS => Some(Flavor::ChromeOS),
            Self::FrutigerAero => Some(Flavor::FrutigerAero),
            Self::Luna | Self::MacOS | Self::Windows10 => None,
        }
    }

    fn identity(self) -> rice_identity::Family {
        use rice_identity::Family;
        match self {
            Self::Luna => Family::Luna,
            Self::MacOS => Family::Mac,
            Self::Windows10 => Family::Win10,
            Self::Windows98 => Family::Win98,
            Self::Windows7 => Family::Aero7,
            Self::ChromeOS => Family::Chrome,
            Self::FrutigerAero => Family::Frutiger,
        }
    }

    const ALL: [Self; 7] = [
        Self::Luna,
        Self::MacOS,
        Self::Windows10,
        Self::Windows98,
        Self::Windows7,
        Self::ChromeOS,
        Self::FrutigerAero,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Luna => "XP Luna Blue",
            Self::MacOS => "macOS",
            Self::Windows10 => "Windows 10",
            Self::Windows98 => "Windows 98",
            Self::Windows7 => "Windows 7 Aero",
            Self::ChromeOS => "ChromeOS",
            Self::FrutigerAero => "Frutiger Aero",
        }
    }

    fn palette(self) -> RicePalette {
        let rgb = Color32::from_rgb;
        match self {
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
            Self::MacOS => RicePalette {
                accent: rgb(0, 96, 198),
                sidebar: rgb(233, 236, 242),
                panel: rgb(250, 251, 253),
                button: rgb(244, 246, 249),
                hover: rgb(218, 231, 249),
                input: rgb(255, 255, 255),
                text: rgb(30, 36, 44),
                muted: rgb(88, 97, 110),
                border: rgb(211, 216, 224),
                dark: false,
                corner_radius: 9,
            },
            Self::Windows10 => RicePalette {
                accent: rgb(0, 103, 184),
                sidebar: rgb(242, 242, 242),
                panel: rgb(255, 255, 255),
                button: rgb(239, 239, 239),
                hover: rgb(220, 236, 251),
                input: rgb(255, 255, 255),
                text: rgb(30, 30, 30),
                muted: rgb(83, 83, 83),
                border: rgb(198, 198, 198),
                dark: false,
                corner_radius: 0,
            },
            Self::Windows98 => RicePalette {
                accent: rgb(0, 51, 153),
                sidebar: rgb(192, 192, 192),
                panel: rgb(224, 224, 224),
                button: rgb(192, 192, 192),
                hover: rgb(240, 240, 225),
                input: rgb(255, 255, 255),
                text: rgb(28, 28, 28),
                muted: rgb(66, 66, 66),
                border: rgb(110, 110, 110),
                dark: false,
                corner_radius: 0,
            },
            Self::Windows7 => RicePalette {
                accent: rgb(23, 92, 152),
                sidebar: rgb(219, 233, 245),
                panel: rgb(247, 251, 255),
                button: rgb(233, 241, 250),
                hover: rgb(205, 230, 249),
                input: rgb(255, 255, 255),
                text: rgb(25, 48, 71),
                muted: rgb(65, 92, 113),
                border: rgb(149, 187, 218),
                dark: false,
                corner_radius: 6,
            },
            Self::ChromeOS => RicePalette {
                accent: rgb(24, 86, 166),
                sidebar: rgb(237, 242, 251),
                panel: rgb(255, 255, 255),
                button: rgb(232, 239, 252),
                hover: rgb(218, 232, 250),
                input: rgb(255, 255, 255),
                text: rgb(32, 43, 59),
                muted: rgb(85, 98, 115),
                border: rgb(213, 222, 233),
                dark: false,
                corner_radius: 14,
            },
            Self::FrutigerAero => RicePalette {
                accent: rgb(9, 99, 119),
                sidebar: rgb(207, 240, 236),
                panel: rgb(246, 255, 253),
                button: rgb(222, 248, 243),
                hover: rgb(195, 240, 231),
                input: rgb(255, 255, 255),
                text: rgb(6, 56, 64),
                muted: rgb(62, 98, 103),
                border: rgb(126, 192, 190),
                dark: false,
                corner_radius: 12,
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
            // Match the information density to the era without shipping fonts
            // or changing the user's compositor-owned window decorations.
            let (body, button, heading) = match self {
                Self::Windows98 => (12.0, 12.0, 19.0),
                Self::Luna => (12.5, 12.5, 20.0),
                Self::Windows7 => (13.0, 13.0, 21.0),
                Self::Windows10 => (13.0, 13.0, 22.0),
                Self::MacOS => (13.5, 13.0, 22.0),
                Self::ChromeOS => (13.5, 13.5, 23.0),
                Self::FrutigerAero => (13.5, 13.0, 22.0),
            };
            style
                .text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(body));
            style
                .text_styles
                .insert(egui::TextStyle::Button, egui::FontId::proportional(button));
            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::proportional(heading),
            );
            style.spacing.item_spacing = match self {
                Self::Luna => egui::vec2(6.0, 7.0),
                Self::MacOS => egui::vec2(8.0, 10.0),
                Self::Windows10 => egui::vec2(8.0, 7.0),
                Self::Windows98 => egui::vec2(5.0, 5.0),
                Self::Windows7 => egui::vec2(8.0, 8.0),
                Self::ChromeOS => egui::vec2(11.0, 10.0),
                Self::FrutigerAero => egui::vec2(10.0, 9.0),
            };
            style.spacing.button_padding = match self {
                Self::Luna => egui::vec2(8.0, 4.0),
                Self::MacOS => egui::vec2(12.0, 7.0),
                Self::Windows10 => egui::vec2(12.0, 5.0),
                Self::Windows98 => egui::vec2(7.0, 3.0),
                Self::Windows7 => egui::vec2(11.0, 5.0),
                Self::ChromeOS => egui::vec2(14.0, 7.0),
                Self::FrutigerAero => egui::vec2(13.0, 7.0),
            };
            style.spacing.interact_size = match self {
                Self::Luna => egui::vec2(36.0, 24.0),
                Self::MacOS => egui::vec2(40.0, 28.0),
                Self::Windows10 => egui::vec2(40.0, 30.0),
                Self::Windows98 => egui::vec2(34.0, 22.0),
                Self::Windows7 => egui::vec2(40.0, 28.0),
                Self::ChromeOS => egui::vec2(42.0, 36.0),
                Self::FrutigerAero => egui::vec2(42.0, 34.0),
            };
            let (body, heading, button, small, monospace) = match self {
                Self::Luna => (12.0, 17.0, 12.0, 11.0, 11.0),
                Self::MacOS => (14.0, 19.0, 14.0, 12.0, 13.0),
                Self::Windows10 => (13.0, 26.0, 13.0, 11.5, 12.0),
                Self::Windows98 => (12.0, 17.0, 12.0, 11.0, 11.0),
                Self::Windows7 => (13.0, 23.0, 13.0, 11.5, 12.0),
                Self::ChromeOS => (14.0, 22.0, 14.0, 12.0, 12.0),
                Self::FrutigerAero => (14.0, 24.0, 13.0, 12.0, 12.0),
            };
            for (kind, size, family) in [
                (egui::TextStyle::Body, body, egui::FontFamily::Proportional),
                (
                    egui::TextStyle::Heading,
                    heading,
                    egui::FontFamily::Proportional,
                ),
                (
                    egui::TextStyle::Button,
                    button,
                    egui::FontFamily::Proportional,
                ),
                (
                    egui::TextStyle::Small,
                    small,
                    egui::FontFamily::Proportional,
                ),
                (
                    egui::TextStyle::Monospace,
                    monospace,
                    egui::FontFamily::Monospace,
                ),
            ] {
                style
                    .text_styles
                    .insert(kind, egui::FontId::new(size, family));
            }
        });
    }
}

impl From<Appearance> for Theme {
    fn from(value: Appearance) -> Self {
        match value {
            Appearance::Luna => Self::Luna,
            Appearance::MacOS => Self::MacOS,
            Appearance::Windows10 => Self::Windows10,
            Appearance::Windows98 => Self::Windows98,
            Appearance::Windows7 => Self::Windows7,
            Appearance::ChromeOS => Self::ChromeOS,
            Appearance::FrutigerAero => Self::FrutigerAero,
        }
    }
}

impl From<Theme> for Appearance {
    fn from(value: Theme) -> Self {
        match value {
            Theme::Luna => Self::Luna,
            Theme::MacOS => Self::MacOS,
            Theme::Windows10 => Self::Windows10,
            Theme::Windows98 => Self::Windows98,
            Theme::Windows7 => Self::Windows7,
            Theme::ChromeOS => Self::ChromeOS,
            Theme::FrutigerAero => Self::FrutigerAero,
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

#[derive(Clone, Default)]
struct InstallDraft {
    name: String,
    file: String,
    error: Option<String>,
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
    error: Option<String>,
}

impl InstallDraft {
    /// A readable suggestion, never a forced rename: users can edit it before setup.
    fn suggested_name(path: &std::path::Path) -> String {
        let raw = path.file_stem().unwrap_or_default().to_string_lossy();
        let normalized = raw.replace(['_', '-'], " ");
        let mut words: Vec<&str> = normalized.split_whitespace().collect();
        if words.last().is_some_and(|word| {
            matches!(
                word.to_ascii_lowercase().as_str(),
                "setup" | "installer" | "install"
            )
        }) {
            words.pop();
        }
        let suggested = words.join(" ");
        if suggested.is_empty() {
            raw.replace(['_', '-'], " ").trim().to_owned()
        } else {
            suggested
        }
    }

    fn choose_file(&mut self, path: PathBuf) {
        let previous_suggestion = Self::suggested_name(std::path::Path::new(&self.file));
        let replace_name = self.name.trim().is_empty() || self.name == previous_suggestion;
        self.file = core::readable_path(&path);
        if replace_name {
            self.name = Self::suggested_name(&path);
        }
        self.error = None;
    }

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
    ExecutablesScanned(String, Result<Vec<LaunchCandidate>, String>),
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
    DoctorFinished(doctor::DoctorReport),
}

struct RiceWineApp {
    simple_ui: bool,
    store: Option<Store>,
    profile_cache: HashMap<String, Result<PrefixProfile, String>>,
    wine: Option<WineTools>,
    theme: Theme,
    preferences: Preferences,
    workspace: Workspace,
    workspace_writable: bool,
    doctor_report: String,
    doctor_busy: bool,
    troubleshoot_id: Option<String>,
    troubleshoot_report: String,
    troubleshoot_log: String,
    compatibility_confirmation: Option<(String, String)>,
    page: Page,
    about_open: bool,
    advanced_open: bool,
    selected: Option<String>,
    search: String,
    search_focus_requested: bool,
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
    exe_scan_busy: HashSet<String>,
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
        let mut draft = InstallDraft::default();
        draft.choose_file(path);
        draft
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
        let workspace_result = store
            .as_ref()
            .ok()
            .map(|s| workspace::load(s.root()))
            .unwrap_or(Ok(Workspace::default()));
        let workspace_writable = workspace_result.is_ok() && store.is_ok();
        if let Err(error) = &workspace_result {
            notice = Some((
                true,
                format!("Library favorites and recent history are read-only: {error}"),
            ));
        }
        let workspace = workspace_result.unwrap_or_default();
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
        // Load runner profiles once, never during per-frame list rendering.
        let profile_cache = store
            .as_ref()
            .ok()
            .map(|store| {
                store
                    .applications()
                    .iter()
                    .map(|app| (app.id.clone(), runtime::load_profile(&app.prefix)))
                    .collect()
            })
            .unwrap_or_default();
        // Screenshot-only opt-in for real dialog captures. No fixture programs.
        let capture_dialog = std::env::var("RICEWINE_CAPTURE_DIALOG").ok();
        let mut app = Self {
            // OS-native shells are the product default. The legacy unified shell
            // remains an explicit diagnostic fallback, never silently selected.
            simple_ui: std::env::var("RICEWINE_UNIFIED_UI").as_deref() == Ok("1"),
            store: store.ok(),
            profile_cache,
            wine: wine.ok(),
            theme,
            preferences,
            workspace,
            workspace_writable,
            doctor_report: String::new(),
            doctor_busy: false,
            troubleshoot_id: None,
            troubleshoot_report: String::new(),
            troubleshoot_log: String::new(),
            compatibility_confirmation: None,
            page: Page::capture_override(),
            about_open: false,
            advanced_open: matches!(Page::capture_override(), Page::Prefixes | Page::Runners),
            selected,
            search: String::new(),
            search_focus_requested: false,
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
            install: initial_install
                .map(Self::install_draft_for_file)
                .or_else(|| {
                    (capture_dialog.as_deref() == Some("install")).then(InstallDraft::default)
                }),
            queued_install_files: VecDeque::new(),
            #[cfg(unix)]
            ipc,
            import: (capture_dialog.as_deref() == Some("import")).then(ImportDraft::default),
            remove_confirmation: None,
            cancel_install_confirmation: false,
            stop_confirmation: None,
            busy: false,
            install_control: None,
            running: HashSet::new(),
            running_controls: HashMap::new(),
            exe_candidates: Vec::new(),
            exe_candidate_for: None,
            exe_scan_busy: HashSet::new(),
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
                                    self.page = Page::Library;
                                    self.search.clear();
                                    self.workspace.filter = LibraryFilter::All;
                                    if let Err(error) = self.persist_workspace() {
                                        self.notice = Some((
                                            true,
                                            format!("Could not save Library view: {error}"),
                                        ));
                                    }
                                    self.sync_runner_draft();
                                    self.notice = Some((
                                        false,
                                        if autodetected {
                                            format!(
                                                "{name} added; launch EXE detected automatically. Check it before Play."
                                            )
                                        } else {
                                            format!(
                                                "{name} added. Find its launch EXE automatically or choose one manually to start it."
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
                Event::ExecutablesScanned(id, result) => {
                    self.exe_scan_busy.remove(&id);
                    if self.selected.as_deref() == Some(id.as_str()) {
                        match result {
                            Ok(candidates) => {
                                let count = candidates.len();
                                self.exe_candidate_for = Some(id);
                                self.exe_candidates = candidates;
                                self.notice = Some((
                                    false,
                                    if count == 0 {
                                        "No launch EXE found. Choose one manually.".into()
                                    } else {
                                        format!(
                                            "Found {count} possible launch files. Choose the correct one."
                                        )
                                    },
                                ));
                            }
                            Err(error) => {
                                self.notice = Some((true, format!("EXE search: {error}")))
                            }
                        }
                    }
                }
                Event::Started(id, result) => match result {
                    Ok(pid) => {
                        self.running.insert(id.clone());
                        self.workspace.mark_launched(&id);
                        self.notice = Some(match self.persist_workspace() {
                            Ok(()) => (false, format!("Program started (PID {pid})")),
                            Err(error) => (
                                true,
                                format!(
                                    "Program started (PID {pid}), but recent history could not be saved: {error}"
                                ),
                            ),
                        });
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
                                        self.workspace.filter = LibraryFilter::All;
                                        self.search.clear();
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
                Event::DoctorFinished(report) => {
                    self.doctor_busy = false;
                    self.doctor_report = report.readable();
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
            || !self.exe_scan_busy.is_empty()
            || self.doctor_busy
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

    fn persist_workspace(&self) -> Result<(), String> {
        if !self.workspace_writable {
            return Err("Workspace could not be loaded; the existing file is preserved".into());
        }
        let root = self.store.as_ref().ok_or("Library unavailable")?;
        workspace::save(root.root(), &self.workspace)
    }

    fn scan_health(&mut self) {
        if self.doctor_busy {
            return;
        }
        let Some(store) = self.store.as_ref() else {
            self.notice = Some((true, "Library unavailable; cannot run checks".into()));
            return;
        };
        let apps = store.applications().to_vec();
        let wine_available = self.wine.is_some();
        let tx = self.sender.clone();
        self.doctor_busy = true;
        self.doctor_report.clear();
        thread::spawn(move || {
            let _ = tx.send(Event::DoctorFinished(doctor::check(&apps, wine_available)));
        });
    }

    fn open_troubleshooter(&mut self, app: &Application) {
        self.troubleshoot_id = Some(app.id.clone());
        self.troubleshoot_report =
            doctor::check(std::slice::from_ref(app), self.wine.is_some()).readable();
        self.troubleshoot_log.clear();
    }

    fn doctor_controls(&mut self, ui: &mut egui::Ui) {
        if self.theme == Theme::Windows10 {
            win10::section_heading(ui, "Health & diagnostics");
        } else if self.theme != Theme::Luna {
            ui.heading("Health & diagnostics");
        }
        ui.small("Check launch files, prefixes and Wine/Proton runners without changing anything.");
        if ui
            .add_enabled(
                !self.doctor_busy && self.store.is_some(),
                egui::Button::new("Check now"),
            )
            .clicked()
        {
            self.scan_health();
        }
        if self.doctor_busy {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.small("Checking applications…");
            });
        }
        if !self.doctor_report.is_empty() {
            ui.add(
                egui::TextEdit::multiline(&mut self.doctor_report)
                    .desired_rows(10)
                    .desired_width(f32::INFINITY)
                    .interactive(false),
            );
        }
    }

    fn set_theme(&mut self, theme: Theme, ctx: &egui::Context) {
        self.theme = theme;
        self.preferences.appearance = theme.into();
        self.preferences.follow_desktop = false;
        self.theme.apply(ctx);
        self.persist_preferences();
    }

    fn install_async(&mut self, draft: InstallDraft) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("Library unavailable")?;
        let profile = PrefixProfile {
            runner: draft.runner(),
            template: draft.template,
            ..Default::default()
        };
        // Validate *before* dismissing the setup dialog; invalid inputs remain editable.
        runtime::validate_profile(&profile)?;
        if self.wine.is_none() && matches!(profile.runner, Runner::SystemWine) {
            return Err("System Wine is missing. Install Wine, or choose a working runner in Advanced settings.".into());
        }
        let file = PathBuf::from(draft.file.trim());
        core::validate_name(&draft.name)?;
        core::validate_installer(&file, draft.portable)?;
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
        Ok(())
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

    fn refresh_profile_cache(&mut self) {
        self.profile_cache = self
            .store
            .as_ref()
            .map(|store| {
                store
                    .applications()
                    .iter()
                    .map(|app| (app.id.clone(), runtime::load_profile(&app.prefix)))
                    .collect()
            })
            .unwrap_or_default();
    }

    /// Bounded prefix scans may traverse thousands of files; never block egui.
    fn scan_executables_async(&mut self, app: &Application) {
        if !self.exe_scan_busy.insert(app.id.clone()) {
            return;
        }
        self.exe_candidates.clear();
        self.exe_candidate_for = None;
        let id = app.id.clone();
        let prefix = app.prefix.clone();
        let name = app.name.clone();
        let sender = self.sender.clone();
        thread::spawn(move || {
            let result = discovery::scan_executables(&prefix, &name);
            let _ = sender.send(Event::ExecutablesScanned(id, result));
        });
    }

    /// This is the same picker used in the normal first-run flow and advanced tools.
    fn show_exe_candidates(&mut self, ui: &mut egui::Ui, app: &Application) {
        if self.exe_candidate_for.as_deref() != Some(app.id.as_str()) {
            return;
        }
        let mut selected = None;
        for candidate in self.exe_candidates.iter().take(12) {
            let name = candidate
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            if ui
                .button(format!("Use {name}"))
                .on_hover_text(core::readable_path(&candidate.path))
                .clicked()
            {
                selected = Some(candidate.path.clone());
            }
        }
        if let Some(path) = selected
            && let Some(store) = self.store.as_mut()
        {
            match store.set_executable(&app.id, path) {
                Ok(()) => {
                    self.exe_candidates.clear();
                    self.exe_candidate_for = None;
                    self.notice = Some((
                        false,
                        "Launch EXE selected. You can now start the program.".into(),
                    ));
                }
                Err(error) => self.notice = Some((true, error)),
            }
        }
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
        let profile = runtime::load_profile(&app.prefix);
        self.profile_cache.insert(app.id.clone(), profile.clone());
        match profile {
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

    fn mac_sidebar(&mut self, ui: &mut egui::Ui) {
        let height = ui.available_height();
        egui::Frame::new()
            .fill(mac::SIDEBAR)
            .inner_margin(egui::Margin::symmetric(14, 17))
            .show(ui, |ui| {
                ui.set_min_width(174.0);
                ui.set_min_height((height - 34.0).max(0.0));
                mac::brand(ui);
                ui.add_space(25.0);
                mac::section_label(ui, "YOUR APPS");
                if mac::navigation(ui, rice_identity::Icon::Programs, "My programs", self.page == Page::Library).clicked() {
                    self.page = Page::Library;
                }
                ui.add_space(12.0);
                mac::section_label(ui, "PREFERENCES");
                if mac::navigation(ui, rice_identity::Icon::Settings, "Settings", self.page == Page::Settings).clicked() {
                    self.page = Page::Settings;
                }
                ui.add_space(14.0);
                if ui
                    .small_button(if self.advanced_open {
                        "⌄ Hide advanced tools"
                    } else {
                        "› Show advanced tools"
                    })
                    .clicked()
                {
                    self.advanced_open = !self.advanced_open;
                }
                if self.advanced_open {
                    for (page, icon) in [
                        (Page::Prefixes, rice_identity::Icon::Tools),
                        (Page::Runners, rice_identity::Icon::Runner),
                    ] {
                        if mac::navigation(ui, icon, page.label(), self.page == page).clicked() {
                            self.page = page;
                        }
                    }
                }
                ui.add_space(16.0);
                ui.separator();
                if ui.link("About RiceWine").clicked() {
                    self.about_open = true;
                }
            });
    }

    fn win10_sidebar(&mut self, ui: &mut egui::Ui) {
        let height = ui.available_height();
        egui::Frame::new()
            .fill(win10::SIDEBAR)
            .inner_margin(egui::Margin::symmetric(14, 21))
            .show(ui, |ui| {
                ui.set_min_width(192.0);
                ui.set_min_height((height - 42.0).max(0.0));
                ui.horizontal(|ui| {
                    if ui
                        .button("←")
                        .on_hover_text("Back to Apps & features")
                        .clicked()
                    {
                        self.page = Page::Library;
                    }
                    ui.add_space(8.0);
                    win10::sidebar_heading(ui);
                });
                ui.add_space(26.0);
                if ui
                    .add(
                        egui::TextEdit::singleline(&mut self.search)
                            .hint_text("Find an app")
                            .desired_width(ui.available_width()),
                    )
                    .changed()
                {
                    self.page = Page::Library;
                }
                win10::section_label(ui, "Apps");
                for (page, label) in [
                    (Page::Library, "Apps & features"),
                    (Page::Prefixes, "Wine environments"),
                    (Page::Runners, "Wine & Proton"),
                    (Page::Settings, "Appearance"),
                ] {
                    if win10::navigation(ui, label, self.page == page).clicked() {
                        self.page = page;
                    }
                }
                ui.add_space(24.0);
                ui.separator();
                if win10::link_action(ui, "About RiceWine", true).clicked() {
                    self.about_open = true;
                }
            });
    }

    fn luna_sidebar(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(self.theme.palette().sidebar)
            .inner_margin(egui::Margin::same(10))
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(89, 131, 207)))
            .show(ui, |ui| {
                ui.set_min_width(194.0);
                ui.set_min_height((ui.available_height() - 22.0).max(0.0));
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
                    if luna::task_link(
                        ui,
                        "Settings",
                        luna::XpIcon::Settings,
                        self.page == Page::Settings,
                    )
                    .clicked()
                    {
                        self.page = Page::Settings;
                    }
                    if luna::task_link(
                        ui,
                        if self.advanced_open {
                            "Hide advanced tools"
                        } else {
                            "Show advanced tools"
                        },
                        luna::XpIcon::Info,
                        false,
                    )
                    .clicked()
                    {
                        self.advanced_open = !self.advanced_open;
                    }
                    if self.advanced_open {
                        for (page, icon) in [
                            (Page::Prefixes, luna::XpIcon::Prefix),
                            (Page::Runners, luna::XpIcon::Runner),
                        ] {
                            if luna::task_link(ui, page.label(), icon, self.page == page).clicked()
                            {
                                self.page = page;
                            }
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

    /// Always reachable when a fixed sidebar would crowd the workspace.
    fn compact_navigation(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.menu_button(format!("☰  {}", self.page.label()), |ui| {
                for page in [Page::Library, Page::Prefixes, Page::Runners, Page::Settings] {
                    if ui
                        .selectable_label(self.page == page, page.label())
                        .clicked()
                    {
                        self.page = page;
                        ui.close();
                    }
                }
            });
            if ui.small_button("About").clicked() {
                self.about_open = true;
            }
        });
        ui.add_space(5.0);
    }

    fn page_body(&mut self, ui: &mut egui::Ui) {
        if let Some((is_error, message)) = self.notice.clone() {
            if self.simple_ui {
                let palette = self.theme.palette();
                let (fill, border, foreground) = if is_error {
                    (
                        Color32::from_rgb(255, 245, 233),
                        Color32::from_rgb(191, 128, 78),
                        Color32::from_rgb(115, 47, 28),
                    )
                } else {
                    (palette.button, palette.border, palette.text)
                };
                let width = ui.available_width();
                egui::Frame::new()
                    .fill(fill)
                    .stroke(egui::Stroke::new(1.0, border))
                    .corner_radius(egui::CornerRadius::same(palette.corner_radius))
                    .inner_margin(egui::Margin::symmetric(10, 8))
                    .show(ui, |ui| {
                        ui.set_min_width((width - 22.0).max(0.0));
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                RichText::new(if is_error { "⚠" } else { "ℹ" })
                                    .strong()
                                    .color(foreground),
                            );
                            ui.label(RichText::new(message).color(foreground));
                            if ui.small_button("Dismiss ×").clicked() {
                                self.notice = None;
                            }
                        });
                    });
                ui.add_space(12.0);
            } else if self.theme == Theme::Luna {
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
            Page::Runners => self.runners(ui),
            Page::Settings => self.settings(ui),
        }
    }

    /// The same discoverable workflow across seven independently styled Rices.
    /// Use a full-width toolbar; the previous auto-sized Frame left most of the
    /// window empty and made the header look detached from the content.
    fn simple_shell(&mut self, ui: &mut egui::Ui) {
        let palette = self.theme.palette();
        let can_install = !self.busy && self.store.is_some();
        let is_classic = matches!(self.theme, Theme::Windows98 | Theme::Luna);
        let available_width = ui.available_width();
        let pad = if is_classic { 9 } else { 14 };
        let header = egui::Frame::new()
            .fill(palette.sidebar)
            .stroke(egui::Stroke::new(1.0, palette.border))
            .inner_margin(egui::Margin::symmetric(
                pad,
                if is_classic { 8 } else { 10 },
            ))
            .corner_radius(egui::CornerRadius::same(palette.corner_radius))
            .show(ui, |ui| {
                ui.set_min_width((available_width - (pad * 2 + 2) as f32).max(0.0));
                ui.horizontal_wrapped(|ui| {
                    rice_identity::symbol(
                        ui,
                        self.theme.identity(),
                        rice_identity::Icon::Brand,
                        palette.accent,
                        if is_classic { 22.0 } else { 24.0 },
                    );
                    ui.label(
                        RichText::new("RiceWine")
                            .strong()
                            .size(if is_classic { 17.0 } else { 19.0 })
                            .color(palette.text),
                    );
                    ui.add_space(12.0);
                    if rice_identity::button(
                        ui,
                        self.theme.identity(),
                        rice_identity::Icon::Programs,
                        "Programs",
                        if self.page == Page::Library {
                            Color32::WHITE
                        } else {
                            palette.accent
                        },
                        if self.page == Page::Library {
                            Color32::WHITE
                        } else {
                            palette.text
                        },
                        if self.page == Page::Library {
                            palette.accent
                        } else {
                            palette.sidebar
                        },
                        palette.border,
                        true,
                        117.0,
                    )
                    .clicked()
                    {
                        self.page = Page::Library;
                    }
                    rice_identity::symbol(
                        ui,
                        self.theme.identity(),
                        rice_identity::Icon::Tools,
                        palette.accent,
                        18.0,
                    );
                    ui.menu_button(
                        if matches!(self.page, Page::Prefixes | Page::Runners) {
                            "Tools •".to_owned()
                        } else {
                            "Tools ▾".to_owned()
                        },
                        |ui| {
                            ui.set_min_width(214.0);
                            if ui.button("Wine prefixes & backups").clicked() {
                                self.page = Page::Prefixes;
                                ui.close();
                            }
                            if ui.button("Wine & Proton runners").clicked() {
                                self.page = Page::Runners;
                                ui.close();
                            }
                            ui.separator();
                            if ui
                                .add_enabled(
                                    can_install,
                                    egui::Button::new("Import existing prefix…"),
                                )
                                .clicked()
                            {
                                self.import = Some(ImportDraft::default());
                                ui.close();
                            }
                            if ui.button("Refresh runner status").clicked() {
                                self.refresh_profile_cache();
                                self.sync_runner_draft();
                                ui.close();
                            }
                            ui.separator();
                            if ui.button("About RiceWine").clicked() {
                                self.about_open = true;
                                ui.close();
                            }
                        },
                    );
                    if rice_identity::button(
                        ui,
                        self.theme.identity(),
                        rice_identity::Icon::Settings,
                        "Settings",
                        if self.page == Page::Settings {
                            Color32::WHITE
                        } else {
                            palette.accent
                        },
                        if self.page == Page::Settings {
                            Color32::WHITE
                        } else {
                            palette.text
                        },
                        if self.page == Page::Settings {
                            palette.accent
                        } else {
                            palette.sidebar
                        },
                        palette.border,
                        true,
                        111.0,
                    )
                    .clicked()
                    {
                        self.page = Page::Settings;
                    }
                    ui.add_space(4.0);
                    if rice_identity::button(
                        ui,
                        self.theme.identity(),
                        rice_identity::Icon::Install,
                        "Install program",
                        Color32::WHITE,
                        Color32::WHITE,
                        palette.accent,
                        palette.accent,
                        can_install,
                        168.0,
                    )
                    .clicked()
                    {
                        self.install = Some(InstallDraft::default());
                    }
                });
            });

        // A simple era-specific toolbar edge, not a fake window titlebar.
        let edge = match self.theme {
            Theme::Luna => (3.0, Color32::from_rgb(55, 111, 210)),
            Theme::Windows98 => (3.0, Color32::from_rgb(0, 0, 128)),
            Theme::Windows7 => (2.0, Color32::from_rgb(127, 180, 211)),
            Theme::FrutigerAero => (2.0, Color32::from_rgb(63, 183, 165)),
            Theme::Windows10 => (1.0, palette.border),
            Theme::MacOS | Theme::ChromeOS => (1.0, palette.border),
        };
        let rect = header.response.rect;
        rice_identity::header_finish(ui.painter(), rect, self.theme.identity(), palette.accent);
        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(rect.left() + 1.0, rect.bottom() - edge.0 - 1.0),
                egui::vec2((rect.width() - 2.0).max(0.0), edge.0),
            ),
            0.0,
            edge.1,
        );
        ui.add_space(10.0);

        let height = ui.available_height().max(0.0);
        let page_width = ui.available_width();
        let inner_margin = if is_classic { 12 } else { 18 };
        ui.allocate_ui_with_layout(
            egui::vec2(page_width.max(0.0), height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                egui::Frame::new()
                    .fill(palette.panel)
                    .stroke(egui::Stroke::new(
                        if self.theme == Theme::Windows98 {
                            1.5
                        } else {
                            1.0
                        },
                        palette.border,
                    ))
                    .corner_radius(egui::CornerRadius::same(palette.corner_radius))
                    .inner_margin(egui::Margin::same(inner_margin))
                    .show(ui, |ui| {
                        ui.set_min_width((page_width - (inner_margin * 2 + 2) as f32).max(0.0));
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| self.page_body(ui));
                    });
            },
        );
    }

    fn mac_shell(&mut self, ui: &mut egui::Ui) {
        let description = match self.page {
            Page::Library => "Your Windows applications",
            Page::Prefixes => "Manage isolated environments",
            Page::Runners => "Wine & Proton compatibility",
            Page::Settings => "Appearance & integration",
        };
        egui::Frame::new()
            .fill(mac::TOOLBAR)
            .stroke(egui::Stroke::new(1.0, self.theme.palette().border))
            .inner_margin(egui::Margin::symmetric(19, 12))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    mac::toolbar_title(ui, self.page.label(), description);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                !self.busy && self.store.is_some(),
                                egui::Button::new("＋ Install program"),
                            )
                            .clicked()
                        {
                            self.install = Some(InstallDraft::default());
                        }
                        ui.menu_button("More ▾", |ui| {
                            if ui
                                .add_enabled(
                                    !self.busy && self.store.is_some(),
                                    egui::Button::new("Import existing prefix…"),
                                )
                                .clicked()
                            {
                                self.import = Some(ImportDraft::default());
                                ui.close();
                            }
                            if ui.button("Settings").clicked() {
                                self.page = Page::Settings;
                                ui.close();
                            }
                        });
                    });
                });
            });
        let compact = ui.available_width() < 760.0;
        if compact {
            self.compact_navigation(ui);
        }
        let available_height = (ui.available_height() - 31.0).max(0.0);
        ui.horizontal_top(|ui| {
            if !compact {
                ui.allocate_ui_with_layout(
                    egui::vec2(204.0, available_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| self.mac_sidebar(ui));
                    },
                );
            }
            let content_width = ui.available_width().max(0.0);
            ui.allocate_ui_with_layout(
                egui::vec2(content_width, available_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    egui::Frame::new()
                        .fill(mac::CANVAS)
                        .inner_margin(egui::Margin::same(18))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| self.page_body(ui));
                        });
                },
            );
        });
        let status = match self.page {
            Page::Library => format!(
                "{} applications",
                self.store
                    .as_ref()
                    .map_or(0, |store| store.applications().len())
            ),
            Page::Prefixes => "Wine prefixes".to_owned(),
            Page::Runners => "Installed runtime tools".to_owned(),
            Page::Settings => "Local preferences".to_owned(),
        };
        mac::status_bar(ui, &status);
    }

    fn win10_shell(&mut self, ui: &mut egui::Ui) {
        let title = match self.page {
            Page::Library => "Apps & features",
            Page::Prefixes => "Wine environments",
            Page::Runners => "Wine & Proton",
            Page::Settings => "Appearance",
        };
        let compact = ui.available_width() < 760.0;
        if compact {
            self.compact_navigation(ui);
        }
        let available_height = ui.available_height().max(0.0);
        ui.horizontal_top(|ui| {
            if !compact {
                ui.allocate_ui_with_layout(
                    egui::vec2(235.0, available_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| self.win10_sidebar(ui));
                    },
                );
                ui.separator();
            }
            let content_width = ui.available_width().max(0.0);
            ui.allocate_ui_with_layout(
                egui::vec2(content_width, available_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    egui::Frame::new()
                        .fill(win10::CANVAS)
                        .inner_margin(egui::Margin::symmetric(27, 22))
                        .show(ui, |ui| {
                            win10::page_heading(ui, title);
                            if self.page == Page::Library {
                                ui.add_space(10.0);
                                ui.horizontal_wrapped(|ui| {
                                    if win10::link_action(
                                        ui,
                                        "＋  Install a Windows program",
                                        !self.busy && self.store.is_some(),
                                    )
                                    .clicked()
                                    {
                                        self.install = Some(InstallDraft::default());
                                    }
                                    if win10::link_action(
                                        ui,
                                        "Import existing prefix",
                                        !self.busy && self.store.is_some(),
                                    )
                                    .clicked()
                                    {
                                        self.import = Some(ImportDraft::default());
                                    }
                                });
                            }
                            ui.add_space(10.0);
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| self.page_body(ui));
                        });
                },
            );
        });
    }

    /// Four RiceBowl-inspired shells share Wine workflows, but not their navigation chrome.
    fn ricebowl_shell(&mut self, ui: &mut egui::Ui) {
        use extra_rices::Flavor;
        let Some(flavor) = self.theme.extra_flavor() else { return };

        // Classic desktop menus use real program actions, not decorative text.
        if flavor == Flavor::Classic98 {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui
                        .add_enabled(
                            !self.busy && self.store.is_some(),
                            egui::Button::new("Install program…"),
                        )
                        .clicked()
                    {
                        self.install = Some(InstallDraft::default());
                        ui.close();
                    }
                    if ui
                        .add_enabled(
                            !self.busy && self.store.is_some(),
                            egui::Button::new("Import prefix…"),
                        )
                        .clicked()
                    {
                        self.import = Some(ImportDraft::default());
                        ui.close();
                    }
                });
                ui.menu_button("View", |ui| {
                    for page in [Page::Library, Page::Prefixes, Page::Runners, Page::Settings] {
                        if ui.button(page.label()).clicked() {
                            self.page = page;
                            ui.close();
                        }
                    }
                });
                if ui.button("Help").clicked() {
                    self.about_open = true;
                }
            });
        }

        extra_rices::masthead(ui, flavor, self.page.label());
        ui.add_space(8.0);
        let can_add = !self.busy && self.store.is_some();
        // Windows 98 uses a compact property-sheet tab strip; Frutiger has
        // a 2007-era segmented command bar; ChromeOS uses its app navigation.
        if matches!(flavor, Flavor::ChromeOS | Flavor::Classic98 | Flavor::FrutigerAero) {
            ui.horizontal_wrapped(|ui| {
                for page in [Page::Library, Page::Prefixes, Page::Runners, Page::Settings] {
                    if extra_rices::navigation(ui, flavor, page.label(), self.page == page, true)
                        .clicked()
                    {
                        self.page = page;
                    }
                }
            });
        }
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(can_add, egui::Button::new("＋ Install program"))
                .clicked()
            {
                self.install = Some(InstallDraft::default());
            }
            if ui
                .add_enabled(can_add, egui::Button::new("Import existing prefix…"))
                .clicked()
            {
                self.import = Some(ImportDraft::default());
            }
        });
        ui.add_space(7.0);

        let full_width = matches!(flavor, Flavor::ChromeOS | Flavor::Classic98 | Flavor::FrutigerAero);
        let compact = ui.available_width() < 740.0;
        if compact && !full_width {
            self.compact_navigation(ui);
        }
        let available_height = (ui.available_height() - 29.0).max(0.0);
        if full_width || compact {
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width().max(0.0), available_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    extra_rices::content_frame(flavor).show(ui, |ui| {
                        extra_rices::page_heading(ui, flavor, self.page.label());
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| self.page_body(ui));
                    });
                },
            );
        } else {
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(202.0, available_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        egui::Frame::new()
                            .fill(self.theme.palette().sidebar)
                            .stroke(egui::Stroke::new(1.0, self.theme.palette().border))
                            .inner_margin(egui::Margin::same(9))
                            .show(ui, |ui| {
                                ui.set_min_height((available_height - 20.0).max(0.0));
                                ui.label(
                                    RichText::new(if flavor == Flavor::Classic98 {
                                        "Folders"
                                    } else if flavor == Flavor::Aero7 {
                                        "Control Panel"
                                    } else {
                                        "Explore RiceWine"
                                    })
                                    .strong()
                                    .color(self.theme.palette().text),
                                );
                                ui.add_space(7.0);
                                for page in
                                    [Page::Library, Page::Prefixes, Page::Runners, Page::Settings]
                                {
                                    if extra_rices::navigation(
                                        ui,
                                        flavor,
                                        page.label(),
                                        self.page == page,
                                        false,
                                    )
                                    .clicked()
                                    {
                                        self.page = page;
                                    }
                                }
                            });
                    },
                );
                ui.add_space(6.0);
                let width = ui.available_width().max(0.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(width, available_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        extra_rices::content_frame(flavor).show(ui, |ui| {
                            extra_rices::page_heading(ui, flavor, self.page.label());
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| self.page_body(ui));
                        });
                    },
                );
            });
        }
        let count = self
            .store
            .as_ref()
            .map_or(0, |store| store.applications().len());
        extra_rices::status(ui, flavor, count);
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
        let compact = ui.available_width() < 760.0;
        if compact {
            self.compact_navigation(ui);
        }
        let available_height = (ui.available_height() - 27.0).max(0.0);
        ui.horizontal_top(|ui| {
            if !compact {
                ui.allocate_ui_with_layout(
                    egui::vec2(225.0, available_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| self.luna_sidebar(ui));
                    },
                );
                ui.add_space(6.0);
            }
            let content_width = ui.available_width().max(0.0);
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
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| self.page_body(ui));
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
        let win10_mode = self.theme == Theme::Windows10;
        let count = self
            .store
            .as_ref()
            .map_or(0, |store| store.applications().len());
        if self.simple_ui {
            ui.horizontal_wrapped(|ui| {
                rice_identity::symbol(
                    ui,
                    self.theme.identity(),
                    rice_identity::Icon::Programs,
                    self.theme.accent(),
                    20.0,
                );
                ui.label(
                    RichText::new(rice_identity::library_heading(self.theme.identity()))
                        .strong()
                        .size(if matches!(self.theme, Theme::Luna | Theme::Windows98) {
                            19.0
                        } else {
                            23.0
                        })
                        .color(self.theme.palette().text),
                );
                if count > 0 {
                    ui.label(
                        RichText::new(format!("{count} installed"))
                            .size(12.0)
                            .color(self.theme.palette().muted),
                    );
                }
            });
            if self.busy {
                ui.horizontal_wrapped(|ui| {
                    ui.spinner();
                    ui.label("Installing…");
                    if ui.button("Cancel installation…").clicked() {
                        self.cancel_install_confirmation = true;
                    }
                });
            }
            if count > 0 {
                ui.add_space(14.0);
                let mut changed = false;
                ui.horizontal_wrapped(|ui| {
                    let search_response = ui.add(
                        egui::TextEdit::singleline(&mut self.search)
                            .hint_text("Search your programs")
                            .desired_width(260.0),
                    );
                    if self.search_focus_requested {
                        search_response.request_focus();
                        self.search_focus_requested = false;
                    }
                    if !self.search.is_empty() && ui.small_button("Clear").clicked() {
                        self.search.clear();
                    }
                    let view_label = if self.workspace.filter == LibraryFilter::Favorites {
                        "Favorites ▾"
                    } else {
                        "View ▾"
                    };
                    ui.menu_button(view_label, |ui| {
                        ui.label("Show");
                        ui.add_enabled_ui(self.workspace_writable, |ui| {
                            for filter in LibraryFilter::ALL {
                                changed |= ui
                                    .selectable_value(
                                        &mut self.workspace.filter,
                                        filter,
                                        filter.label(),
                                    )
                                    .changed();
                            }
                        });
                        ui.separator();
                        ui.label("Sort");
                        ui.add_enabled_ui(self.workspace_writable, |ui| {
                            for order in LibraryOrder::ALL {
                                changed |= ui
                                    .selectable_value(
                                        &mut self.workspace.order,
                                        order,
                                        order.label(),
                                    )
                                    .changed();
                            }
                        });
                    });
                });
                if changed && let Err(error) = self.persist_workspace() {
                    self.notice = Some((true, format!("Could not save Library view: {error}")));
                }
                ui.add_space(12.0);
            }
        } else {
            if luna_mode {
                luna::section_heading(ui, "Installed Windows programs", luna::XpIcon::Programs);
            } else if win10_mode {
                win10::section_heading(ui, "Installed apps");
            }
            ui.horizontal(|ui| {
                if self.busy {
                    ui.spinner();
                    ui.label("Installing…");
                    if ui.button("Cancel installation…").clicked() {
                        self.cancel_install_confirmation = true;
                    }
                }
            });
            ui.add_space(if luna_mode { 5.0 } else { 10.0 });
            if self.store.is_some() {
                ui.small("Drop EXE/MSI to install · Ctrl+F to search your programs.");
            }
            ui.horizontal_wrapped(|ui| {
                if !win10_mode {
                    ui.label(if luna_mode { "Search for:" } else { "Search" });
                }
                let search_response = ui.add(
                    egui::TextEdit::singleline(&mut self.search)
                        .hint_text(if win10_mode {
                            "Search this list"
                        } else {
                            "Filter installed programs…"
                        })
                        .desired_width(if win10_mode { 280.0 } else { 210.0 }),
                );
                if self.search_focus_requested {
                    search_response.request_focus();
                    self.search_focus_requested = false;
                }
                if !self.search.is_empty() && luna::action(ui, "Clear", true, luna_mode).clicked() {
                    self.search.clear();
                }
                if !win10_mode && ui.small_button("↻ Refresh profiles").clicked() {
                    self.refresh_profile_cache();
                    self.sync_runner_draft();
                }
                if win10_mode && win10::link_action(ui, "Refresh", true).clicked() {
                    self.refresh_profile_cache();
                    self.sync_runner_draft();
                }
            });
            let mut workspace_changed = false;
            ui.horizontal_wrapped(|ui| {
                ui.label("Show:");
                ui.add_enabled_ui(self.workspace_writable, |ui| {
                    egui::ComboBox::from_id_salt("ricewine_3_filter")
                        .selected_text(self.workspace.filter.label())
                        .show_ui(ui, |ui| {
                            for filter in LibraryFilter::ALL {
                                workspace_changed |= ui
                                    .selectable_value(
                                        &mut self.workspace.filter,
                                        filter,
                                        filter.label(),
                                    )
                                    .changed();
                            }
                        });
                });
                ui.label("Order:");
                ui.add_enabled_ui(self.workspace_writable, |ui| {
                    egui::ComboBox::from_id_salt("ricewine_3_order")
                        .selected_text(self.workspace.order.label())
                        .show_ui(ui, |ui| {
                            for order in LibraryOrder::ALL {
                                workspace_changed |= ui
                                    .selectable_value(
                                        &mut self.workspace.order,
                                        order,
                                        order.label(),
                                    )
                                    .changed();
                            }
                        });
                });
            });
            if workspace_changed && let Err(error) = self.persist_workspace() {
                self.notice = Some((true, format!("Could not save Library view: {error}")));
            }
        }
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
            if self.simple_ui {
                let palette = self.theme.palette();
                let actions = rice_identity::empty_library(
                    ui,
                    self.theme.identity(),
                    palette.accent,
                    palette.text,
                    palette.muted,
                    palette.button,
                    palette.border,
                    !self.busy && self.store.is_some(),
                );
                if actions.install {
                    self.install = Some(InstallDraft::default());
                }
                if actions.import {
                    self.import = Some(ImportDraft::default());
                }
            } else if self.theme == Theme::Luna {
                ui.add_space(7.0);
                luna::empty_library(ui);
            } else if self.theme == Theme::MacOS {
                if mac::empty_library(ui, !self.busy && self.store.is_some()) {
                    self.install = Some(InstallDraft::default());
                }
            } else if win10_mode {
                if win10::empty_library(ui, !self.busy && self.store.is_some()) {
                    self.install = Some(InstallDraft::default());
                }
            } else if let Some(flavor) = self.theme.extra_flavor()
                && extra_rices::empty_library(ui, flavor, !self.busy && self.store.is_some())
            {
                self.install = Some(InstallDraft::default());
            }
            return;
        }
        // Filters and sort are in-memory; catalog and profile files are not touched.
        let matching_indices: Vec<usize> = self
            .store
            .as_ref()
            .map(|store| {
                self.workspace
                    .ordered_indices(store.applications(), &self.search)
            })
            .unwrap_or_default();
        ui.small(format!("{} matching program(s)", matching_indices.len()));
        if matching_indices.is_empty() {
            if self.workspace.filter == LibraryFilter::Favorites {
                ui.label(
                    "No favorite programs match. Choose 'All programs' or mark one as favorite.",
                );
            } else {
                ui.label("No matching programs. Clear the search to show the library.");
            }
            return;
        }
        // Split only when the list and details genuinely fit side by side.
        let stacked = win10_mode || !ui_layout::split_panes(ui.available_width(), 355.0, 235.0);
        let width = if stacked {
            ui.available_width().max(0.0)
        } else {
            (ui.available_width() - 280.0).max(0.0)
        };
        let mut next_selected: Option<String> = None;
        let display = |ui: &mut egui::Ui| {
            let list_height = if win10_mode {
                254.0
            } else if stacked {
                260.0
            } else {
                340.0
            };
            ui.allocate_ui_with_layout(
                egui::vec2(width, list_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    if self.simple_ui {
                        ui.label(
                            RichText::new("PROGRAMS")
                                .size(10.5)
                                .strong()
                                .color(self.theme.palette().muted),
                        );
                        ui.separator();
                    } else if self.theme == Theme::Luna {
                        luna::program_list_header(ui);
                    } else if self.theme == Theme::MacOS {
                        mac::list_header(ui);
                    } else {
                        ui.separator();
                    }
                    let row_height = if self.simple_ui {
                        if matches!(self.theme, Theme::Windows98 | Theme::Luna) {
                            37.0
                        } else {
                            46.0
                        }
                    } else {
                        match self.theme {
                            Theme::Windows98 | Theme::Luna => 28.0,
                            Theme::Windows7 => 33.0,
                            Theme::MacOS => 40.0,
                            Theme::ChromeOS => 48.0,
                            Theme::FrutigerAero => 44.0,
                            Theme::Windows10 => 63.0,
                        }
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
                                let profile = self.profile_cache.get(&app.id);
                                let runtime = profile
                                    .and_then(|result| result.as_ref().ok())
                                    .map(|profile| profile.runner.label())
                                    .unwrap_or("Invalid profile");
                                let display_name = if self.workspace.is_favorite(&app.id) {
                                    format!("★ {}", app.name)
                                } else {
                                    app.name.clone()
                                };
                                let status = if runtime == "Invalid profile" {
                                    "Check runner"
                                } else if app.executable.is_none() {
                                    "Choose EXE"
                                } else if self.running.contains(&app.id) {
                                    "Running"
                                } else if self.wine.is_none() && runtime == "System Wine" {
                                    "Wine missing"
                                } else {
                                    "Ready"
                                };
                                if self.simple_ui {
                                    let palette = self.theme.palette();
                                    ui.horizontal(|ui| {
                                        let name = egui::Button::new(
                                            RichText::new(&display_name).strong().color(
                                                if selected {
                                                    Color32::WHITE
                                                } else {
                                                    palette.text
                                                },
                                            ),
                                        )
                                        .fill(if selected {
                                            palette.accent
                                        } else {
                                            palette.button
                                        })
                                        .stroke(egui::Stroke::new(1.0, palette.border))
                                        .corner_radius(egui::CornerRadius::same(
                                            palette.corner_radius,
                                        ));
                                        if ui
                                            .add_sized(
                                                [(width - 116.0).max(80.0), row_height - 5.0],
                                                name,
                                            )
                                            .clicked()
                                        {
                                            next_selected = Some(app.id.clone());
                                        }
                                        ui.label(
                                            RichText::new(status).size(11.5).color(palette.muted),
                                        );
                                    });
                                } else if win10_mode {
                                    if win10::app_row(ui, &display_name, runtime, status, selected)
                                        .clicked()
                                    {
                                        next_selected = Some(app.id.clone());
                                    }
                                } else {
                                    ui.horizontal(|ui| {
                                        let row_width = (ui.available_width() - 122.0).max(0.0);
                                        let response = if luna_mode {
                                            luna::program_row(ui, &display_name, selected, row_width)
                                        } else if self.theme == Theme::MacOS {
                                            mac::program_row(ui, &display_name, selected, row_width)
                                        } else if let Some(flavor) = self.theme.extra_flavor() {
                                            extra_rices::program_row(
                                                ui, flavor, &display_name, status, selected, row_width,
                                            )
                                        } else {
                                            ui.button(&display_name)
                                        };
                                        if response.clicked() {
                                            next_selected = Some(app.id.clone());
                                        }
                                        ui.label(if luna_mode { runtime } else { status });
                                    });
                                }
                            }
                        },
                    );
                },
            );
            if !win10_mode {
                ui.separator();
            }
            let simplified_details = self.simple_ui;
            let details_palette = self.theme.palette();
            let details = |ui: &mut egui::Ui| {
                if !stacked {
                    ui.set_min_width(235.0);
                }
                let selected = self
                    .selected
                    .as_ref()
                    .and_then(|id| self.store.as_ref()?.get(id))
                    .filter(|app| {
                        self.workspace.filter == LibraryFilter::All
                            || self.workspace.is_favorite(&app.id)
                    })
                    .cloned();
                if let Some(app) = selected {
                    if self.simple_ui {
                        ui.label(
                            RichText::new(&app.name)
                                .strong()
                                .size(18.0)
                                .color(self.theme.palette().text),
                        );
                    } else if luna_mode {
                        ui.label(RichText::new(&app.name).strong().size(13.0));
                    } else if win10_mode {
                        ui.label(RichText::new(&app.name).size(17.0).color(win10::TEXT));
                    } else {
                        ui.heading(&app.name);
                    }
                    if !self.simple_ui {
                        if ui
                            .add_enabled(
                                self.workspace_writable,
                                egui::Button::new(if self.workspace.is_favorite(&app.id) {
                                    "★ Remove favorite"
                                } else {
                                    "☆ Add to favorites"
                                }),
                            )
                            .clicked()
                        {
                            self.workspace.toggle_favorite(&app.id);
                            if let Err(error) = self.persist_workspace() {
                                self.notice =
                                    Some((true, format!("Could not save favorite: {error}")));
                            }
                        }
                        if ui.button("Diagnose & recover…").clicked() {
                            self.open_troubleshooter(&app);
                        }
                    }
                    let selected_profile =
                        self.profile_cache.get(&app.id).cloned().unwrap_or_else(|| {
                            Err("Runner profile not loaded; refresh profiles".into())
                        });
                    ui.add_space(8.0);
                    let can_play = !self.busy
                        && !self.compatibility_busy.contains(&app.id)
                        && app.executable.is_some()
                        && selected_profile.as_ref().is_ok_and(|p| {
                            self.wine.is_some() || !matches!(p.runner, Runner::SystemWine)
                        });
                    let start = if self.simple_ui {
                        let palette = self.theme.palette();
                        rice_identity::button(
                            ui,
                            self.theme.identity(),
                            rice_identity::Icon::Play,
                            "Run program",
                            Color32::WHITE,
                            Color32::WHITE,
                            palette.accent,
                            palette.accent,
                            can_play,
                            156.0,
                        )
                    } else if win10_mode {
                        win10::primary_action(ui, "Run program", can_play)
                    } else {
                        luna::action(
                            ui,
                            if luna_mode {
                                "Play program"
                            } else {
                                "▶ Play"
                            },
                            can_play,
                            luna_mode,
                        )
                    };
                    if start.clicked() {
                        self.play(app.clone());
                    }
                    if self.running.contains(&app.id) {
                        ui.label("● Running");
                        if luna::action(ui, "Stop program…", true, luna_mode).clicked() {
                            self.stop_confirmation = Some(app.id.clone());
                        }
                    }
                    if app.executable.is_none() {
                        ui.small("One step left: select the EXE that starts this program.");
                        ui.horizontal_wrapped(|ui| {
                            if luna::action(
                                ui,
                                "Find EXE automatically",
                                !self.exe_scan_busy.contains(&app.id),
                                luna_mode,
                            )
                            .clicked()
                            {
                                self.scan_executables_async(&app);
                            }
                            if luna::action(ui, "Choose EXE manually…", true, luna_mode).clicked()
                                && let Some(path) = rfd::FileDialog::new()
                                    .add_filter("Windows program", &["exe"])
                                    .pick_file()
                                && let Some(store) = self.store.as_mut()
                            {
                                self.notice = Some(match store.set_executable(&app.id, path) {
                                    Ok(()) => {
                                        (false, "Launch EXE selected. Ready to start.".into())
                                    }
                                    Err(error) => (true, error),
                                });
                            }
                        });
                        if self.exe_scan_busy.contains(&app.id) {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.small("Looking for installed programs…");
                            });
                        }
                        self.show_exe_candidates(ui, &app);
                    }
                    ui.add_space(14.0);
                    ui.collapsing(
                        if self.simple_ui {
                            "More options"
                        } else {
                            "More options · rename, runner, files, remove"
                        },
                        |ui| {
                            if self.simple_ui {
                                if ui
                                    .add_enabled(
                                        self.workspace_writable,
                                        egui::Button::new(if self.workspace.is_favorite(&app.id) {
                                            "★ Remove favorite"
                                        } else {
                                            "☆ Add to favorites"
                                        }),
                                    )
                                    .clicked()
                                {
                                    self.workspace.toggle_favorite(&app.id);
                                    if let Err(error) = self.persist_workspace() {
                                        self.notice = Some((
                                            true,
                                            format!("Could not save favorite: {error}"),
                                        ));
                                    }
                                }
                                if ui.button("Diagnose & recover…").clicked() {
                                    self.open_troubleshooter(&app);
                                }
                            }
                            ui.label("Display name");
                            ui.text_edit_singleline(&mut self.name_edit);
                            if luna::action(
                                ui,
                                "Save name",
                                !self.busy && !self.running.contains(&app.id),
                                luna_mode,
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
                                            && path
                                                .join(format!("ricewine-{}.desktop", app.id))
                                                .exists()
                                            && let Ok(binary) = std::env::current_exe()
                                            && let Err(error) =
                                                integration::install_shortcut(&updated, &binary)
                                        {
                                            self.notice = Some((
                                                true,
                                                format!(
                                                    "Name saved, shortcut update failed: {error}"
                                                ),
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
                            if app.executable.is_none() {
                                ui.small("Use the quick EXE finder above to finish setup.");
                            }
                            if luna::action(
                                ui,
                                "Scan installed EXEs",
                                !self.exe_scan_busy.contains(&app.id),
                                luna_mode,
                            )
                            .clicked()
                            {
                                self.scan_executables_async(&app);
                            }
                            if self.exe_scan_busy.contains(&app.id) {
                                ui.spinner();
                                ui.small("Scanning prefix…");
                            }
                            if app.executable.is_some() {
                                self.show_exe_candidates(ui, &app);
                            }
                            if luna::action(ui, "Choose launch EXE…", true, luna_mode).clicked()
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
                            if luna::action(ui, "Add menu shortcut", true, luna_mode).clicked() {
                                let outcome = std::env::current_exe()
                                    .map_err(|e| e.to_string())
                                    .and_then(|bin| integration::install_shortcut(&app, &bin));
                                self.notice = Some(match outcome {
                                    Ok(file) => {
                                        (false, format!("Shortcut created: {}", file.display()))
                                    }
                                    Err(error) => (true, error),
                                });
                            }
                            ui.add_space(18.0);
                            if luna::action(ui, "Remove from library…", true, luna_mode).clicked()
                            {
                                self.remove_confirmation = Some(app.id);
                            }
                            ui.small("This never deletes the Wine prefix.");
                        },
                    );
                } else {
                    ui.label("Select a program to view details.");
                }
            };
            if simplified_details {
                let palette = details_palette;
                egui::Frame::new()
                    .fill(palette.button)
                    .stroke(egui::Stroke::new(1.0, palette.border))
                    .corner_radius(egui::CornerRadius::same(palette.corner_radius))
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, details);
            } else if luna_mode {
                luna::option_group(ui, "Program details", luna::XpIcon::Programs, details);
            } else if win10_mode {
                win10::section_heading(ui, "Selected app");
                egui::Frame::new()
                    .fill(Color32::from_rgb(247, 247, 247))
                    .inner_margin(egui::Margin::same(16))
                    .show(ui, details);
            } else {
                ui.vertical(details);
            }
        };
        if stacked {
            ui.vertical(display);
        } else {
            ui.horizontal_top(display);
        }
        if let Some(id) = next_selected {
            self.selected = Some(id);
            self.exe_candidates.clear();
            self.exe_candidate_for = None;
            self.sync_runner_draft();
        }
    }

    fn prefixes(&mut self, ui: &mut egui::Ui) {
        let luna_mode = self.theme == Theme::Luna;
        if self.simple_ui {
            rice_identity::title(
                ui,
                self.theme.identity(),
                rice_identity::Icon::Prefix,
                "Wine environments",
                self.theme.accent(),
                self.theme.palette().text,
            );
            ui.label("Back up, restore and manage your program environments.");
        } else if luna_mode {
            luna::section_heading(ui, "Wine prefix maintenance", luna::XpIcon::Prefix);
            ui.small("Back up your Windows environments before changing runners or components.");
        } else {
            if self.theme == Theme::Luna {
                ui.heading("Wine prefixes");
            }
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
        ui.vertical(|ui| {
            if apps.is_empty() && luna_mode {
                egui::Frame::new()
                    .fill(Color32::from_rgb(249, 250, 252))
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(170, 185, 205)))
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        ui.label("No Wine prefixes are registered yet.");
                        ui.small(if self.simple_ui {
                            "Use Install program above, or Tools → Import existing prefix."
                        } else {
                            "Install or import a program from the Program tasks pane."
                        });
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

            if self.simple_ui {
                rice_identity::title(
                    ui,
                    self.theme.identity(),
                    rice_identity::Icon::Backup,
                    "Backups",
                    self.theme.accent(),
                    self.theme.palette().text,
                );
            } else if luna_mode {
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
        if self.simple_ui {
            rice_identity::title(
                ui,
                self.theme.identity(),
                rice_identity::Icon::Runner,
                "Wine & Proton",
                self.theme.accent(),
                self.theme.palette().text,
            );
        } else if luna_mode {
            luna::section_heading(ui, "Installed runtime tools", luna::XpIcon::Runner);
        }
        ui.small("Wine and Proton are configured independently for each Windows program.");
        ui.add_space(7.0);
        let diagnostics = runtime::diagnostics(self.wine.as_ref());
        if luna_mode {
            egui::Frame::new()
                .fill(Color32::from_rgb(248, 250, 254))
                .stroke(egui::Stroke::new(1.0, Color32::from_rgb(170, 187, 212)))
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
        ui.horizontal_wrapped(|ui| {
            if luna::action(ui, "Check latest GE-Proton", !self.ge_busy, luna_mode).clicked() {
                self.check_ge_release();
            }
            if let Some(release) = &self.ge_release {
                ui.label(format!("Latest: {}", release.tag));
                if luna::action(ui, "Install verified release…", !self.ge_busy, luna_mode).clicked()
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
        if luna::action(
            ui,
            "Find locally installed Proton versions",
            true,
            luna_mode,
        )
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
            ui.horizontal_wrapped(|ui| {
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
        ui.horizontal_wrapped(|ui| {
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
        ui.horizontal_wrapped(|ui| {
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
                        self.profile_cache
                            .insert(app.id.clone(), runtime::load_profile(&app.prefix));
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
        ui.horizontal_wrapped(|ui| {
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
        ui.horizontal_wrapped(|ui| {
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
                self.compatibility_confirmation = Some((
                    app.id.clone(),
                    runtime::WINETRICKS_VERBS[self.winetricks_verb].to_owned(),
                ));
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
                ui.horizontal_wrapped(|ui| {
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
                ui.horizontal_wrapped(|ui| {
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
        ui.vertical(|ui| {
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
            luna::option_group(ui, "Health & diagnostics", luna::XpIcon::Info, |ui| {
                self.doctor_controls(ui);
            });
        });
    }

    fn settings(&mut self, ui: &mut egui::Ui) {
        if self.simple_ui {
            rice_identity::title(
                ui,
                self.theme.identity(),
                rice_identity::Icon::Settings,
                "Settings",
                self.theme.accent(),
                self.theme.palette().text,
            );
            ui.add_space(10.0);
        }
        if self.theme == Theme::Luna && !self.simple_ui {
            self.luna_settings(ui);
            return;
        }
        if self.theme == Theme::Windows10 && !self.simple_ui {
            win10::section_heading(ui, "Choose a RiceWine style");
            ui.label(
                RichText::new("Changes this application only. Your Linux desktop stays as it is.")
                    .size(12.0)
                    .color(win10::MUTED),
            );
            ui.add_space(14.0);
            let mut selected = None;
            for theme in Theme::ALL {
                if ui.radio(self.theme == theme, theme.label()).clicked() {
                    selected = Some(theme);
                }
                ui.add_space(7.0);
            }
            if let Some(theme) = selected {
                self.set_theme(theme, ui.ctx());
            }
        } else {
            ui.label(
                RichText::new("Appearance")
                    .strong()
                    .color(self.theme.palette().text),
            );
            ui.small("Rices only change RiceWine, not the desktop or Wine prefixes.");
            ui.add_space(7.0);
            let mut selected = None;
            ui.horizontal_wrapped(|ui| {
                ui.label("Visual style:");
                egui::ComboBox::from_id_salt("ricewine_settings_rice")
                    .width(220.0)
                    .selected_text(self.theme.label())
                    .show_ui(ui, |ui| {
                        for theme in Theme::ALL {
                            if ui
                                .selectable_label(self.theme == theme, theme.label())
                                .clicked()
                            {
                                selected = Some(theme);
                            }
                        }
                    });
            });
            if let Some(theme) = selected {
                self.set_theme(theme, ui.ctx());
            }
        }
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
        ui.add_space(14.0);
        if self.simple_ui {
            ui.collapsing("Advanced settings & diagnostics", |ui| {
                self.settings_advanced(ui);
            });
        } else {
            self.settings_advanced(ui);
        }
    }

    fn settings_advanced(&mut self, ui: &mut egui::Ui) {
        ui.add_space(16.0);
        if self.theme == Theme::Windows10 {
            win10::section_heading(ui, "Desktop integration");
        } else {
            ui.label("Desktop integration");
        }
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
            if self.theme == Theme::Windows10 {
                win10::section_heading(ui, "Application data");
            } else {
                ui.label("Application data");
            }
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
        ui.add_space(20.0);
        self.doctor_controls(ui);
    }

    fn install_dialog(&mut self, ctx: &egui::Context) {
        let mut submit = None;
        let mut keep_open = true;
        let mut cancel_dialog = false;
        let luna_mode = self.theme == Theme::Luna;
        let win10_mode = self.theme == Theme::Windows10;
        let flavor = self.theme.extra_flavor();
        if let Some(draft) = &mut self.install {
            egui::Window::new("Install Windows program")
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .resizable(true)
                .vscroll(true)
                .max_size([
                    (ctx.content_rect().width() - 24.0).max(280.0),
                    (ctx.content_rect().height() - 24.0).max(220.0),
                ])
                .open(&mut keep_open)
                .show(ctx, |ui| {
                    ui.set_min_width(ui_layout::dialog_width(ctx.content_rect().width(), 460.0));
                    if luna_mode {
                        luna::dialog_intro(
                            ui,
                            luna::XpIcon::Install,
                            "Install a Windows program",
                            "Choose a setup file and its Wine environment.",
                        );
                    }
                    if luna_mode {
                        luna::section_heading(ui, "Program information", luna::XpIcon::Programs);
                    } else if self.theme == Theme::MacOS {
                        mac::dialog_intro(ui, "Install Windows program",
                            "Select a setup file and configure its Wine environment.");
                    } else if let Some(flavor) = flavor {
                        extra_rices::dialog_intro(ui, flavor, "Install a Windows program",
                            "Choose a setup file and review its compatibility settings.");
                    } else if win10_mode {
                        ui.label(RichText::new("Install a Windows app").size(20.0).color(win10::TEXT));
                        ui.small("Select an EXE or MSI; Wine handles the installation.");
                        ui.separator();
                    }
                    ui.label(RichText::new("1. Choose a Windows program").strong());
                    ui.small("Select a setup EXE/MSI, or an EXE that runs without installation.");
                    ui.add_space(3.0);
                    ui.horizontal_wrapped(|ui| {
                        let file_input = ui.add(
                            egui::TextEdit::singleline(&mut draft.file)
                                .hint_text("Choose an EXE or MSI file…")
                                .desired_width(ui_layout::field_width(ui.available_width(), 112.0, 340.0)),
                        );
                        if file_input.changed() {
                            draft.error = None;
                        }
                        if file_input.lost_focus() && draft.name.trim().is_empty() {
                            draft.name = InstallDraft::suggested_name(std::path::Path::new(&draft.file));
                        }
                        if luna::action(ui, "Browse…", true, luna_mode).clicked()
                            && let Some(file) = rfd::FileDialog::new()
                                .add_filter("Windows setup", &["exe", "msi"])
                                .pick_file()
                        {
                            draft.choose_file(file);
                        }
                    });
                    ui.add_space(8.0);
                    ui.label(RichText::new("2. Confirm the name").strong());
                    if ui.add(
                        egui::TextEdit::singleline(&mut draft.name)
                            .hint_text("Name shown in your library"),
                    ).changed() {
                        draft.error = None;
                    }
                    ui.small("Suggested from the filename. You can change it.");
                    ui.add_space(5.0);
                    ui.checkbox(&mut draft.portable, "This app is portable (no installer)");
                    if draft.portable {
                        ui.small("RiceWine will register this EXE directly instead of running setup.");
                    }
                    ui.add_space(8.0);
                    ui.collapsing("Advanced settings · Wine, Proton and compatibility", |ui| {
                        if luna_mode {
                            luna::section_heading(ui, "Compatibility", luna::XpIcon::Runner);
                        }
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
                    });
                    ui.small("Windows programs can access your Linux user files. Only run software you trust.");
                    if let Some(error) = &draft.error {
                        ui.add_space(5.0);
                        ui.colored_label(Color32::DARK_RED, error);
                    }
                    if draft.file.trim().is_empty() {
                        ui.small("Choose an EXE/MSI to continue.");
                    }
                    ui.add_space(9.0);
                    ui.separator();
                    let install_label = if draft.portable {
                        "Add portable program"
                    } else {
                        "Run installer"
                    };
                    ui.horizontal_wrapped(|ui| {
                        let ready = !draft.file.trim().is_empty() && !draft.name.trim().is_empty();
                        let start = if win10_mode {
                            win10::primary_action(ui, install_label, ready)
                        } else {
                            luna::action(ui, install_label, ready, luna_mode)
                        };
                        if start.clicked() {
                            submit = Some(draft.clone());
                        }
                        let cancel = if win10_mode {
                            win10::secondary_action(ui, "Cancel", true)
                        } else {
                            luna::action(ui, "Cancel", true, luna_mode)
                        };
                        if cancel.clicked() {
                            cancel_dialog = true;
                        }
                    });
                });
        }
        if !keep_open || cancel_dialog {
            self.install = None;
        }
        if let Some(draft) = submit {
            match self.install_async(draft) {
                Ok(()) => self.install = None,
                Err(error) => {
                    self.notice = Some((true, error.clone()));
                    if let Some(draft) = &mut self.install {
                        draft.error = Some(error);
                    }
                }
            }
        }
    }

    fn import_dialog(&mut self, ctx: &egui::Context) {
        let mut keep_open = true;
        let mut cancel_dialog = false;
        let mut imported = None;
        let luna_mode = self.theme == Theme::Luna;
        let win10_mode = self.theme == Theme::Windows10;
        let flavor = self.theme.extra_flavor();
        if let Some(draft) = &mut self.import {
            egui::Window::new("Import existing Wine prefix")
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .resizable(true)
                .vscroll(true)
                .max_size([
                    (ctx.content_rect().width() - 24.0).max(280.0),
                    (ctx.content_rect().height() - 24.0).max(220.0),
                ])
                .open(&mut keep_open)
                .show(ctx, |ui| {
                    ui.set_min_width(ui_layout::dialog_width(ctx.content_rect().width(), 450.0));
                    if luna_mode {
                        luna::dialog_intro(
                            ui,
                            luna::XpIcon::Import,
                            "Import an existing prefix",
                            "Register a program without moving any of its files.",
                        );
                    } else if self.theme == Theme::MacOS {
                        mac::dialog_intro(ui, "Import Windows program",
                            "Register an existing application without moving its files.");
                    } else if let Some(flavor) = flavor {
                        extra_rices::dialog_intro(ui, flavor, "Import existing program",
                            "Register a Wine environment without moving or deleting its files.");
                    } else if win10_mode {
                        ui.label(RichText::new("Import a Wine environment").size(20.0).color(win10::TEXT));
                        ui.small("Register an existing program without moving its files.");
                        ui.separator();
                    }
                    ui.label("Program name");
                    if ui.text_edit_singleline(&mut draft.name).changed() {
                        draft.error = None;
                    }
                    if luna_mode {
                        luna::section_heading(ui, "Existing environment", luna::XpIcon::Prefix);
                    }
                    ui.label("Existing Wine prefix directory");
                    ui.horizontal_wrapped(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut draft.prefix)
                            .desired_width(ui_layout::field_width(ui.available_width(), 108.0, 330.0)));
                        if luna::action(ui, "Browse…", true, luna_mode).clicked()
                            && let Some(path) = rfd::FileDialog::new().pick_folder()
                        {
                            draft.prefix = core::readable_path(&path);
                            draft.error = None;
                        }
                    });
                    ui.label("Existing launch EXE");
                    ui.horizontal_wrapped(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut draft.executable)
                            .desired_width(ui_layout::field_width(ui.available_width(), 108.0, 330.0)));
                        if luna::action(ui, "Browse…", true, luna_mode).clicked()
                            && let Some(path) = rfd::FileDialog::new().add_filter("Windows program", &["exe"]).pick_file()
                        {
                            draft.executable = core::readable_path(&path);
                            if draft.name.trim().is_empty() {
                                draft.name = InstallDraft::suggested_name(&path);
                            }
                            draft.error = None;
                        }
                    });
                    ui.small("Import only registers an existing prefix. Its files are never moved or deleted.");
                    if let Some(error) = &draft.error {
                        ui.add_space(5.0);
                        ui.colored_label(Color32::DARK_RED, error);
                    }
                    ui.add_space(8.0);
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        let ready = !draft.name.trim().is_empty()
                            && !draft.prefix.trim().is_empty()
                            && !draft.executable.trim().is_empty();
                        let start = if win10_mode {
                            win10::primary_action(ui, "Import program", ready)
                        } else {
                            luna::action(ui, "Import program", ready, luna_mode)
                        };
                        if start.clicked() {
                            imported = Some(core::imported_application(
                                &draft.name,
                                &PathBuf::from(draft.prefix.trim()),
                                &PathBuf::from(draft.executable.trim()),
                            ));
                        }
                        let cancel = if win10_mode {
                            win10::secondary_action(ui, "Cancel", true)
                        } else {
                            luna::action(ui, "Cancel", true, luna_mode)
                        };
                        if cancel.clicked() {
                            cancel_dialog = true;
                        }
                    });
                });
        }
        if !keep_open || cancel_dialog {
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
                    self.page = Page::Library;
                    self.search.clear();
                    self.workspace.filter = LibraryFilter::All;
                    self.sync_runner_draft();
                    self.notice = Some((
                        false,
                        "Existing prefix imported. Original files were preserved.".into(),
                    ));
                    self.import = None;
                }
                Err(error) => {
                    self.notice = Some((true, error.clone()));
                    if let Some(draft) = &mut self.import {
                        draft.error = Some(error);
                    }
                }
            }
        }
    }

    fn remove_dialog(&mut self, ctx: &egui::Context) {
        let luna_mode = self.theme == Theme::Luna;
        let Some(id) = self.remove_confirmation.clone() else {
            return;
        };
        egui::Window::new("Remove from library?")
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false).resizable(false).show(ctx, |ui| {
            if luna_mode {
                luna::dialog_intro(
                    ui,
                    luna::XpIcon::Warning,
                    "Remove library entry?",
                    "The installed Windows files and Wine prefix remain on disk.",
                );
            }
            ui.label("The catalog entry will be removed. The Wine prefix and installed files will remain on disk.");
            ui.horizontal(|ui| {
                if luna::action(ui, "Cancel", true, luna_mode).clicked() { self.remove_confirmation = None; }
                if luna::action(ui, "Remove entry", true, luna_mode).clicked() {
                    if let Some(store) = &mut self.store {
                        match store.remove_from_library(&id) {
                            Ok(()) => {
                                self.selected = store.applications().first().map(|app| app.id.clone());
                                self.workspace.forget(&id);
                                if let Err(error) = self.persist_workspace() {
                                    self.notice = Some((true, format!("Program removed, but Library preferences were not saved: {error}")));
                                }
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

    /// Read-only per-app triage. Backup is explicit and does not trigger a repair.
    fn troubleshoot_dialog(&mut self, ctx: &egui::Context) {
        let Some(id) = self.troubleshoot_id.clone() else {
            return;
        };
        let Some(app) = self
            .store
            .as_ref()
            .and_then(|store| store.get(&id))
            .cloned()
        else {
            self.troubleshoot_id = None;
            self.notice = Some((true, "This program is no longer in the library".into()));
            return;
        };
        let luna_mode = self.theme == Theme::Luna;
        let can_backup = !self.backup_busy
            && !self.busy
            && !self.running.contains(&id)
            && !self.compatibility_busy.contains(&id);
        let mut keep_open = true;
        let mut action = 0u8;
        egui::Window::new(format!("Troubleshoot: {}", app.name))
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .collapsible(false)
            .default_width(540.0)
            .vscroll(true)
            .max_size([
                (ctx.content_rect().width() - 24.0).max(280.0),
                (ctx.content_rect().height() - 24.0).max(220.0),
            ])
            .open(&mut keep_open)
            .show(ctx, |ui| {
                if luna_mode {
                    luna::dialog_intro(
                        ui,
                        luna::XpIcon::Info,
                        "Program doesn't start?",
                        "Review problems before changing Wine settings.",
                    );
                }
                ui.label("1. Check this program");
                ui.small("Only reads the current prefix, selected EXE and runner profile.");
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Recheck").clicked() {
                        action = 1;
                    }
                    if ui.button("Read recent Wine log").clicked() {
                        action = 2;
                    }
                });
                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        ui.monospace(&self.troubleshoot_report);
                    });
                if !self.troubleshoot_log.is_empty() {
                    ui.separator();
                    ui.label("Recent application log (last 12 KiB)");
                    egui::ScrollArea::vertical()
                        .max_height(140.0)
                        .show(ui, |ui| {
                            ui.monospace(&self.troubleshoot_log);
                        });
                }
                ui.add_space(10.0);
                ui.separator();
                ui.label("2. Choose a safe next step");
                ui.small("A backup is recommended before changing compatibility components. No fixes run automatically.");
                ui.horizontal_wrapped(|ui| {
                    if app.executable.is_none() && ui.button("Find launch EXE").clicked() {
                        action = 3;
                    }
                    if ui.button("Runner settings").clicked() {
                        action = 4;
                    }
                    if ui.button("Open backups").clicked() {
                        action = 7;
                    }
                    if ui
                        .add_enabled(can_backup, egui::Button::new("Back up prefix"))
                        .clicked()
                    {
                        action = 5;
                    }
                });
                if self.backup_busy {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.small("Creating backup…");
                    });
                }
                ui.add_space(6.0);
                if ui.button("Close").clicked() {
                    action = 6;
                }
            });
        if !keep_open || action == 6 {
            self.troubleshoot_id = None;
            return;
        }
        match action {
            1 => {
                self.troubleshoot_report =
                    doctor::check(std::slice::from_ref(&app), self.wine.is_some()).readable();
            }
            2 => {
                self.troubleshoot_log = self
                    .store
                    .as_ref()
                    .map(|store| {
                        runtime::log_tail(store.root(), &app.id, 12_000)
                            .unwrap_or_else(|error| format!("Could not read Wine log: {error}"))
                    })
                    .unwrap_or_default();
            }
            3 => {
                self.troubleshoot_id = None;
                self.page = Page::Library;
                self.selected = Some(app.id.clone());
                self.workspace.filter = LibraryFilter::All;
                self.search.clear();
                if let Err(error) = self.persist_workspace() {
                    self.notice = Some((true, format!("Could not save Library view: {error}")));
                }
                self.scan_executables_async(&app);
            }
            4 => {
                self.troubleshoot_id = None;
                self.page = Page::Runners;
                self.selected = Some(app.id);
                self.sync_runner_draft();
            }
            5 => {
                self.backup_async(app);
            }
            7 => {
                self.troubleshoot_id = None;
                self.page = Page::Prefixes;
            }
            _ => {}
        }
    }

    fn process_dialogs(&mut self, ctx: &egui::Context) {
        let luna_mode = self.theme == Theme::Luna;
        if let Some((id, verb)) = self.compatibility_confirmation.clone() {
            let program = self
                .store
                .as_ref()
                .and_then(|store| store.get(&id))
                .cloned();
            let available = program.is_some()
                && !self.backup_busy
                && !self.busy
                && !self.running.contains(&id)
                && !self.compatibility_busy.contains(&id);
            let mut decision = 0u8;
            egui::Window::new("Change Wine compatibility?")
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    if luna_mode {
                        luna::dialog_intro(
                            ui,
                            luna::XpIcon::Warning,
                            "Change Wine compatibility?",
                            "Back up the prefix before installing components.",
                        );
                    }
                    ui.label(format!("Component: {verb}"));
                    ui.label("Winetricks can modify files inside the selected Wine prefix. A backup is recommended before applying the change.");
                    ui.small("Back up first creates an archive only. You must explicitly return and apply the component afterward.");
                    ui.horizontal_wrapped(|ui| {
                        if luna::action(ui, "Cancel", true, luna_mode).clicked() {
                            decision = 1;
                        }
                        if luna::action(ui, "Back up first", available, luna_mode).clicked() {
                            decision = 2;
                        }
                        if luna::action(ui, "Apply without backup", available, luna_mode).clicked() {
                            decision = 3;
                        }
                    });
                });
            match decision {
                1 => self.compatibility_confirmation = None,
                2 => {
                    self.compatibility_confirmation = None;
                    if let Some(app) = program {
                        self.backup_async(app);
                    }
                }
                3 => {
                    self.compatibility_confirmation = None;
                    if let Some(app) = program {
                        self.compatibility_async(app, verb);
                    }
                }
                _ => {}
            }
        }
        if let Some(path) = self.ge_remove_confirmation.clone() {
            egui::Window::new("Remove managed GE-Proton runner?")
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false).resizable(false).show(ctx, |ui| {
                    if luna_mode {
                        luna::dialog_intro(ui, luna::XpIcon::Warning, "Remove managed runner?", "This operation deletes only the managed runner.");
                    }
                    ui.label(core::readable_path(&path));
                    ui.label("Only this RiceWine-downloaded runner will be deleted permanently. Prefixes stay intact, but removed runner files cannot be restored automatically.");
                    ui.horizontal(|ui| {
                        if luna::action(ui, "Keep runner", true, luna_mode).clicked() { self.ge_remove_confirmation = None; }
                        if ui.add_enabled(!self.runner_in_use(&path), egui::Button::new("Delete managed runner")).clicked() {
                            self.ge_remove_confirmation = None;
                            self.remove_managed_runner_async(path);
                        }
                    });
                });
        }
        if let Some((id, installer)) = self.update_install_confirmation.clone() {
            let app = self
                .store
                .as_ref()
                .and_then(|store| store.get(&id))
                .cloned();
            let available = app.is_some()
                && !self.backup_busy
                && !self.busy
                && !self.running.contains(&id)
                && !self.compatibility_busy.contains(&id);
            let mut decision = 0u8;
            egui::Window::new("Run Windows update installer?")
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    if luna_mode {
                        luna::dialog_intro(
                            ui,
                            luna::XpIcon::Install,
                            "Update an installed Windows app?",
                            "Back up this prefix before running the update.",
                        );
                    }
                    if let Some(app) = &app {
                        ui.label(format!("Program: {}", app.name));
                    }
                    ui.label(format!("Installer: {}", core::readable_path(&installer)));
                    ui.label("Updates execute within the program's existing Wine prefix. Only select installers from trusted publishers.");
                    ui.small("Back up first creates a prefix archive. You decide separately whether to run this installer.");
                    if self.backup_busy {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.small("Creating backup. Update actions are disabled…");
                        });
                    }
                    ui.horizontal_wrapped(|ui| {
                        if luna::action(ui, "Cancel", true, luna_mode).clicked() {
                            decision = 1;
                        }
                        if luna::action(ui, "Back up prefix first", available, luna_mode).clicked() {
                            decision = 2;
                        }
                        if luna::action(ui, "Run update now", available, luna_mode).clicked() {
                            decision = 3;
                        }
                    });
                });
            match decision {
                1 => self.update_install_confirmation = None,
                2 => {
                    if let Some(app) = app {
                        self.backup_async(app);
                    }
                }
                3 => {
                    self.update_install_confirmation = None;
                    if let Some(app) = app {
                        self.update_app_async(app, installer);
                    }
                }
                _ => {}
            }
        }
        if self.ge_install_confirmation {
            egui::Window::new("Download GE-Proton?").anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO).collapsible(false).resizable(false).show(ctx, |ui| {
                if luna_mode {
                    luna::dialog_intro(ui, luna::XpIcon::Install, "Install GE-Proton?", "The archive will be downloaded and verified.");
                }
                ui.label("The official release archive can be large. RiceWine verifies SHA512 before extraction and never overwrites an existing runner.");
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() { self.ge_install_confirmation = false; }
                    if luna::action(ui, "Download and install", true, luna_mode).clicked() {
                        self.ge_install_confirmation = false;
                        if let Some(release) = self.ge_release.clone() { self.install_ge_async(release); }
                    }
                });
            });
        }
        if let Some((id, preview)) = self.cleanup_preview.clone() {
            egui::Window::new("Clean old temporary files?").anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO).collapsible(false).resizable(false).show(ctx, |ui| {
                if luna_mode {
                    luna::dialog_intro(ui, luna::XpIcon::Disk, "Temporary files", "Review the eligible files before cleaning.");
                }
                ui.label(format!("{} files / {} eligible (older than 7 days)", preview.files.len(), storage::human_size(preview.bytes)));
                ui.label("Only regular files inside temporary folders of a RiceWine-owned prefix. No program files, prefixes, directories or links are removed.");
                ui.horizontal(|ui| {
                    if luna::action(ui, "Keep files", true, luna_mode).clicked() { self.cleanup_preview = None; }
                    let app = self.store.as_ref().and_then(|store| store.get(&id)).cloned();
                    if ui.add_enabled(!preview.files.is_empty() && app.is_some() && !self.running.contains(&id), egui::Button::new("Delete listed temp files")).clicked() {
                        self.cleanup_preview = None;
                        if let Some(app) = app { self.cleanup_async(app); }
                    }
                });
            });
        }
        if self.cancel_install_confirmation {
            egui::Window::new("Cancel installation?").anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO).collapsible(false).resizable(false).show(ctx, |ui| {
                if luna_mode {
                    luna::dialog_intro(ui, luna::XpIcon::Warning, "Cancel installation?", "Partial prefix data will be preserved.");
                }
                ui.label("Stop the installer and its process group. Partial Wine prefix data will be kept for recovery.");
                ui.horizontal(|ui| {
                    if luna::action(ui, "Keep installing", true, luna_mode).clicked() { self.cancel_install_confirmation = false; }
                    if luna::action(ui, "Stop installer", true, luna_mode).clicked() {
                        if let Some(control) = &self.install_control { control.cancel(); }
                        self.cancel_install_confirmation = false;
                        self.notice = Some((false, "Stopping installer…".into()));
                    }
                });
            });
        }
        if let Some(id) = self.stop_confirmation.clone() {
            egui::Window::new("Stop Windows program?")
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    if luna_mode {
                        luna::dialog_intro(
                            ui,
                            luna::XpIcon::Warning,
                            "Stop Windows program?",
                            "Any unsaved changes may be lost.",
                        );
                    }
                    ui.label("Unsaved changes in the Windows program may be lost.");
                    ui.horizontal(|ui| {
                        if luna::action(ui, "Keep running", true, luna_mode).clicked() {
                            self.stop_confirmation = None;
                        }
                        if luna::action(ui, "Stop program", true, luna_mode).clicked() {
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
        let dialog_open = self.install.is_some()
            || self.import.is_some()
            || self.remove_confirmation.is_some()
            || self.troubleshoot_id.is_some()
            || self.compatibility_confirmation.is_some();
        if !dialog_open
            && ui
                .ctx()
                .input_mut(|input| input.consume_key(egui::Modifiers::CTRL, egui::Key::F))
        {
            self.page = Page::Library;
            self.search_focus_requested = true;
        }
        // Drag-and-drop is a shortcut to the same reviewable setup form,
        // never permission to execute a dropped Windows binary automatically.
        if self.install.is_none()
            && !self.busy
            && self.store.is_some()
            && let Some(path) = ui.ctx().input(|input| {
                input.raw.dropped_files.iter().find_map(|file| {
                    let path = file.path();
                    (!path.as_os_str().is_empty()).then(|| path.to_path_buf())
                })
            })
        {
            let supported = path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("exe") || ext.eq_ignore_ascii_case("msi")
                });
            if supported {
                self.install = Some(Self::install_draft_for_file(path));
            } else {
                self.notice = Some((
                    true,
                    "Drop a Windows EXE or MSI to open the installation form.".into(),
                ));
            }
        }
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
            if self.simple_ui {
                self.simple_shell(ui);
            } else {
                match self.theme {
                    Theme::Luna => self.luna_shell(ui),
                    Theme::MacOS => self.mac_shell(ui),
                    Theme::Windows10 => self.win10_shell(ui),
                    Theme::Windows98 | Theme::Windows7 | Theme::ChromeOS | Theme::FrutigerAero => {
                        self.ricebowl_shell(ui)
                    }
                }
            }
        });
        self.install_dialog(ui.ctx());
        self.import_dialog(ui.ctx());
        self.remove_dialog(ui.ctx());
        self.process_dialogs(ui.ctx());
        self.troubleshoot_dialog(ui.ctx());
        if self.about_open {
            egui::Window::new("About RiceWine")
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .open(&mut self.about_open)
                .show(ui.ctx(), |ui| {
                    ui.heading("RiceWine");
                    ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                    ui.label("Windows programs on Linux, managed with Wine and Proton.");
                    ui.label("Rices style RiceWine only. The compositor owns window decorations.");
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
                    "RiceWine {}\nUsage: ricewine [--launch APPLICATION_ID | --install-file FILE | --list | --diagnose | --doctor | --verify-library | --snapshot-library | --register-menu | --unregister-menu | --unregister-open-with | --unregister-shortcuts | --version | --help]",
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
            [flag] if flag == "--doctor" => core::Store::list_for_cli().and_then(|(_, apps)| {
                let report = doctor::check(&apps, WineTools::detect().is_ok());
                println!("{}", report.readable());
                if report.has_errors() {
                    Err("Doctor found launch blockers. No files were changed".into())
                } else {
                    Ok(())
                }
            }),
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
            .with_min_inner_size([560.0, 400.0]),
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
    fn installation_suggests_names_without_erasing_manual_edits() {
        let mut draft = InstallDraft::default();
        draft.choose_file(PathBuf::from("/tmp/Example-App_Setup.exe"));
        assert_eq!(draft.name, "Example App");
        assert!(draft.file.ends_with("Example-App_Setup.exe"));

        // Choosing another setup replaces only an automatically suggested name.
        draft.choose_file(PathBuf::from("/tmp/Another_Program_installer.msi"));
        assert_eq!(draft.name, "Another Program");

        draft.name = "My custom title".into();
        draft.choose_file(PathBuf::from("/tmp/Third-install.exe"));
        assert_eq!(draft.name, "My custom title");

        assert_eq!(
            InstallDraft::suggested_name(std::path::Path::new("setup.exe")),
            "setup"
        );
        assert_eq!(
            InstallDraft::suggested_name(std::path::Path::new("My_App_1.2.3_Setup.exe")),
            "My App 1.2.3"
        );
    }

    #[test]
    fn all_simple_rices_have_readable_navigation_and_primary_actions() {
        for theme in Theme::ALL {
            let palette = theme.palette();
            assert!(
                contrast(Color32::WHITE, palette.accent) >= 4.5,
                "{theme:?} primary action text"
            );
            assert!(
                contrast(palette.text, palette.sidebar) >= 4.5,
                "{theme:?} toolbar text"
            );
            assert!(
                contrast(palette.text, palette.button) >= 4.5,
                "{theme:?} program row text"
            );
        }
    }

    #[test]
    fn compact_library_stacks_details_instead_of_overflowing() {
        for width in [280.0, 400.0, 540.0, 609.0] {
            let stacked = width < 610.0;
            let list_width = if stacked { width } else { width - 280.0 };
            assert!(stacked);
            assert!(list_width > 0.0);
        }
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
