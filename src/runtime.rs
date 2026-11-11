//! Per-prefix runner configuration and compatibility actions.
//!
//! The catalog is deliberately independent of this file: existing 0.1 libraries
//! stay readable and a missing profile means system Wine.

use crate::core::{self, Application, InstallMode, WineTools};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SAVE_ID: AtomicU64 = AtomicU64::new(0);
const PROFILE_VERSION: u32 = 1;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Runner {
    #[default]
    SystemWine,
    CustomWine {
        wine: PathBuf,
    },
    ProtonUmu {
        proton: PathBuf,
        game_id: String,
    },
}

impl Runner {
    pub fn label(&self) -> &'static str {
        match self {
            Self::SystemWine => "System Wine",
            Self::CustomWine { .. } => "Custom Wine",
            Self::ProtonUmu { .. } => "Proton / umu",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct PrefixProfile {
    pub version: u32,
    pub runner: Runner,
    pub template: Template,
    pub launch_arguments: Vec<String>,
    pub launch_environment: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Template {
    #[default]
    Standard,
    Games,
    Legacy32,
}

impl Template {
    pub fn label(self) -> &'static str {
        match self {
            Self::Standard => "Standard (64-bit)",
            Self::Games => "Games (64-bit)",
            Self::Legacy32 => "Legacy 32-bit",
        }
    }
}

impl Default for PrefixProfile {
    fn default() -> Self {
        Self {
            version: PROFILE_VERSION,
            runner: Runner::SystemWine,
            template: Template::Standard,
            launch_arguments: Vec::new(),
            launch_environment: BTreeMap::new(),
        }
    }
}

pub fn profile_path(prefix: &Path) -> PathBuf {
    prefix.join(".ricewine.toml")
}

pub fn load_profile(prefix: &Path) -> Result<PrefixProfile, String> {
    let path = profile_path(prefix);
    if !path.exists() {
        return Ok(PrefixProfile::default());
    }
    let contents = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let profile: PrefixProfile = toml::from_str(&contents)
        .map_err(|e| format!("Invalid runner profile {}: {e}", path.display()))?;
    if profile.version != PROFILE_VERSION {
        return Err(format!(
            "Unsupported profile version {} in {}; no files were changed",
            profile.version,
            path.display()
        ));
    }
    Ok(profile)
}

pub fn save_profile(prefix: &Path, profile: &PrefixProfile) -> Result<(), String> {
    if !prefix.is_dir() {
        return Err(format!("Prefix does not exist: {}", prefix.display()));
    }
    validate_profile(profile)?;
    let contents = toml::to_string_pretty(profile).map_err(|e| e.to_string())?;
    let final_path = profile_path(prefix);
    let temp_path = prefix.join(format!(
        ".ricewine-{}-{}.tmp",
        std::process::id(),
        SAVE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        fs::rename(&temp_path, &final_path)?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&temp_path);
        return Err(format!("Saving runner profile failed: {error}"));
    }
    Ok(())
}

/// Used before any potentially expensive or modifying Wine operation.
/// Otherwise an invalid profile can leave an orphaned prefix after setup.
pub fn validate_profile(profile: &PrefixProfile) -> Result<(), String> {
    if profile.version != PROFILE_VERSION {
        return Err("Unsupported runner profile version".into());
    }
    validate_runner(&profile.runner)?;
    validate_launch_options(profile)?;
    if profile.template == Template::Legacy32 && matches!(profile.runner, Runner::ProtonUmu { .. })
    {
        return Err("Legacy 32-bit prefixes require Wine, not Proton".into());
    }
    Ok(())
}

pub fn validate_runner(runner: &Runner) -> Result<(), String> {
    match runner {
        Runner::SystemWine => Ok(()),
        Runner::CustomWine { wine } => {
            if !wine.is_file() || !wine.with_file_name("wineboot").is_file() {
                return Err("Choose a Wine binary with wineboot in the same directory".into());
            }
            Ok(())
        }
        Runner::ProtonUmu { proton, game_id } => {
            if !proton.is_dir() || !proton.join("proton").is_file() {
                return Err("Choose an extracted Proton directory containing 'proton'".into());
            }
            if game_id.is_empty()
                || game_id.len() > 90
                || !game_id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
            {
                return Err(
                    "Game ID must use 1–90 letters, digits, dot, dash or underscore".into(),
                );
            }
            if executable_on_path("umu-run").is_none() {
                return Err("umu-run is not installed or not in PATH".into());
            }
            Ok(())
        }
    }
}

pub const ALLOWED_ENVIRONMENT: &[&str] = &[
    "WINEDEBUG",
    "WINEESYNC",
    "WINEFSYNC",
    "DXVK_HUD",
    "DXVK_LOG_LEVEL",
    "VKD3D_DEBUG",
    "MESA_GLTHREAD",
    "PULSE_LATENCY_MSEC",
];

pub fn validate_launch_options(profile: &PrefixProfile) -> Result<(), String> {
    if profile.launch_arguments.len() > 32
        || profile
            .launch_arguments
            .iter()
            .any(|arg| arg.len() > 1024 || arg.contains('\0') || arg.contains('\n'))
    {
        return Err("Use up to 32 launch arguments, each at most 1024 bytes".into());
    }
    if profile.launch_environment.len() > ALLOWED_ENVIRONMENT.len() {
        return Err("Too many environment overrides".into());
    }
    for (key, value) in &profile.launch_environment {
        if !ALLOWED_ENVIRONMENT.contains(&key.as_str())
            || value.len() > 2048
            || value.contains('\0')
        {
            return Err(format!("Invalid environment override: {key}"));
        }
    }
    Ok(())
}

pub fn parse_launch_configuration(
    arguments: &str,
    variables: &str,
    profile: &mut PrefixProfile,
) -> Result<(), String> {
    let args = arguments
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut env = BTreeMap::new();
    for line in variables.lines().filter(|line| !line.trim().is_empty()) {
        let (key, value) = line
            .split_once('=')
            .ok_or("Each environment line must be KEY=value")?;
        let key = key.trim();
        if env.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err(format!("Duplicate environment override: {key}"));
        }
    }
    let mut candidate = profile.clone();
    candidate.launch_arguments = args;
    candidate.launch_environment = env;
    validate_launch_options(&candidate)?;
    *profile = candidate;
    Ok(())
}

pub fn executable_on_path(name: &str) -> Option<PathBuf> {
    env::split_paths(&env::var_os("PATH")?).find_map(|dir| {
        let file = dir.join(name);
        file.is_file().then_some(file)
    })
}

pub fn runner_command(
    profile: &PrefixProfile,
    system: Option<&WineTools>,
    prefix: &Path,
    executable: &Path,
) -> Result<Command, String> {
    validate_runner(&profile.runner)?;
    let mut command = match &profile.runner {
        Runner::SystemWine => Command::new(
            &system
                .ok_or("System Wine is unavailable; choose a different runner")?
                .wine,
        ),
        Runner::CustomWine { wine } => Command::new(wine),
        Runner::ProtonUmu { proton, game_id } => {
            let mut command =
                Command::new(executable_on_path("umu-run").ok_or("umu-run is unavailable")?);
            command.env("PROTONPATH", proton).env("GAMEID", game_id);
            command
        }
    };
    if executable
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("msi"))
    {
        // umu expects a Windows executable; msiexec can be invoked via its
        // full Windows command only with plain Wine for now.
        if matches!(profile.runner, Runner::ProtonUmu { .. }) {
            return Err("MSI via Proton is not supported; use Wine for installation".into());
        }
        command.args(["msiexec", "/i"]);
    }
    command.arg(executable).env("WINEPREFIX", prefix);
    if let Some(parent) = executable.parent() {
        command.current_dir(parent);
    }
    Ok(command)
}

pub fn install_with_profile(
    root: &Path,
    system: Option<&WineTools>,
    profile: &PrefixProfile,
    name: &str,
    file: &Path,
    mode: InstallMode,
) -> Result<Application, String> {
    validate_profile(profile)?;
    let app = match &profile.runner {
        Runner::SystemWine => core::install_with_arch(
            root,
            system.ok_or("System Wine is not installed")?,
            name,
            file,
            mode,
            profile.template == Template::Legacy32,
        )?,
        Runner::CustomWine { wine } => {
            let custom = WineTools {
                wine: wine.clone(),
                wineboot: wine.with_file_name("wineboot"),
                version: "Custom Wine".into(),
            };
            core::install_with_arch(
                root,
                &custom,
                name,
                file,
                mode,
                profile.template == Template::Legacy32,
            )?
        }
        Runner::ProtonUmu { .. } => {
            core::validate_name(name)?;
            core::validate_installer(file, matches!(mode, InstallMode::Portable))?;
            let file = fs::canonicalize(file).map_err(|e| e.to_string())?;
            let id = core::new_id(name);
            let prefix = root.join("prefixes").join(&id);
            fs::create_dir(&prefix).map_err(|e| format!("Creating prefix: {e}"))?;
            core::private_permissions(&prefix)?;
            let portable = matches!(mode, InstallMode::Portable);
            if !portable {
                let mut command = runner_command(profile, system, &prefix, &file)?;
                core::run_logged(
                    &mut command,
                    &root.join("logs").join(format!("{id}.log")),
                    None,
                )
                .map_err(|e| format!("Proton installation failed ({e}); prefix retained"))?;
            }
            Application {
                id,
                name: name.trim().into(),
                prefix,
                executable: portable.then_some(file.clone()),
                installer: (!portable).then_some(file),
                portable,
            }
        }
    };
    save_profile(&app.prefix, profile)?;
    Ok(app)
}

pub fn launch(
    root: &Path,
    system: Option<&WineTools>,
    app: &Application,
    profile: &PrefixProfile,
) -> Result<Child, String> {
    let exe = app
        .executable
        .as_ref()
        .ok_or("Choose a launch EXE before starting the program")?;
    core::validate_executable(exe)?;
    if !app.prefix.is_dir() {
        return Err("The program's prefix is missing".into());
    }
    let mut command = runner_command(profile, system, &app.prefix, exe)?;
    validate_launch_options(profile)?;
    command.args(&profile.launch_arguments);
    command.envs(&profile.launch_environment);
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join("logs").join(format!("{}.log", app.id)))
        .map_err(|e| e.to_string())?;
    command
        .stdout(Stdio::from(log.try_clone().map_err(|e| e.to_string())?))
        .stderr(Stdio::from(log))
        .spawn()
        .map_err(|e| format!("Launching {} failed: {e}", app.name))
}

pub const WINETRICKS_VERBS: &[&str] = &[
    "corefonts",
    "vcrun2022",
    "dotnet48",
    "d3dx9",
    "dxvk",
    "vkd3d",
    "xact",
];

pub fn run_winetricks(
    root: &Path,
    app: &Application,
    profile: &PrefixProfile,
    verb: &str,
) -> Result<(), String> {
    let executable = executable_on_path("winetricks").ok_or("Install winetricks first")?;
    let mut command = winetricks_command(&executable, app, profile, verb)?;
    core::run_logged(
        &mut command,
        &root.join("logs").join(format!("{}.log", app.id)),
        None,
    )
}

/// Construct a component operation separately from spawning it. This lets
/// tests verify exact arguments and environment without invoking real Wine.
pub fn winetricks_command(
    executable: &Path,
    app: &Application,
    profile: &PrefixProfile,
    verb: &str,
) -> Result<Command, String> {
    if !WINETRICKS_VERBS.contains(&verb) {
        return Err("Unsupported Winetricks component".into());
    }
    if !app.prefix.is_dir() {
        return Err("Wine prefix is missing".into());
    }
    let mut command = Command::new(executable);
    command.args(["-q", verb]).env("WINEPREFIX", &app.prefix);
    match &profile.runner {
        Runner::SystemWine => {}
        Runner::CustomWine { wine } => {
            validate_runner(&profile.runner)?;
            command.env("WINE", wine);
        }
        Runner::ProtonUmu { .. } => {
            return Err("Winetricks for Proton prefixes is not supported yet; use Wine".into());
        }
    }
    Ok(command)
}

/// Read only a bounded log tail: one misbehaving installer must not exhaust RAM.
pub fn log_tail(root: &Path, id: &str, limit: usize) -> Result<String, String> {
    if !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("Invalid application ID".into());
    }
    let path = root.join("logs").join(format!("{id}.log"));
    let mut file = match fs::File::open(&path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(String::new()),
        Err(e) => return Err(e.to_string()),
    };
    let end = file.metadata().map_err(|e| e.to_string())?.len();
    let start = end.saturating_sub(limit.min(65536) as u64);
    file.seek(SeekFrom::Start(start))
        .map_err(|e| e.to_string())?;
    let mut buffer = Vec::with_capacity((end - start) as usize);
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&buffer).into_owned())
}

pub fn diagnostics(system: Option<&WineTools>) -> Vec<String> {
    let mut rows = Vec::new();
    rows.push(match system {
        Some(wine) => format!("System Wine: {} ({})", wine.version, wine.wine.display()),
        None => "System Wine: not detected".into(),
    });
    for executable in ["umu-run", "winetricks", "vulkaninfo", "xdg-open"] {
        rows.push(match executable_on_path(executable) {
            Some(path) => format!("{executable}: {}", path.display()),
            None => format!("{executable}: unavailable"),
        });
    }
    rows
}

#[derive(Clone, Debug)]
pub struct DetectedProton {
    pub path: PathBuf,
    pub name: String,
}

pub fn discover_proton(home: &Path) -> Vec<DetectedProton> {
    let parents = [
        home.join(".steam/root/compatibilitytools.d"),
        home.join(".steam/debian-installation/compatibilitytools.d"),
        home.join(".local/share/Steam/compatibilitytools.d"),
        home.join(".steam/steam/steamapps/common"),
        home.join(".local/share/Steam/steamapps/common"),
        home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam/compatibilitytools.d"),
        home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/common"),
    ];
    let mut candidates = Vec::new();
    let mut unique = std::collections::HashSet::new();
    for directory in parents {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten().take(200) {
            let path = entry.path();
            if !path.join("proton").is_file() {
                continue;
            }
            let Ok(canonical) = fs::canonicalize(&path) else {
                continue;
            };
            if !unique.insert(canonical.clone()) {
                continue;
            }
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            candidates.push(DetectedProton {
                path: canonical,
                name,
            });
        }
    }
    candidates.sort_by_key(|candidate| candidate.name.to_lowercase());
    candidates
}

pub fn timestamp_slug() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
