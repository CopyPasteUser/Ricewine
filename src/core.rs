//! Wine operations and the on-disk application catalog. No GUI dependencies.

use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SCHEMA: u32 = 1;
static NEXT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Application {
    pub id: String,
    pub name: String,
    pub prefix: PathBuf,
    pub executable: Option<PathBuf>,
    pub installer: Option<PathBuf>,
    pub portable: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Catalog {
    schema: u32,
    applications: Vec<Application>,
}

impl Default for Catalog {
    fn default() -> Self {
        Self {
            schema: SCHEMA,
            applications: Vec::new(),
        }
    }
}

/// One-user catalog. Managed prefixes are never removed by catalog operations.
pub struct Store {
    root: PathBuf,
    catalog: Catalog,
    _lock: File,
}

impl Drop for Store {
    fn drop(&mut self) {
        let _ = self._lock.unlock();
    }
}

impl Store {
    pub fn open_default() -> Result<Self, String> {
        let root = data_directory()?;
        Self::open_at(root)
    }

    pub fn open_at(root: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(root.join("prefixes")).map_err(|e| e.to_string())?;
        fs::create_dir_all(root.join("logs")).map_err(|e| e.to_string())?;
        let lock = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(root.join("catalog.lock"))
            .map_err(|e| e.to_string())?;
        lock.try_lock()
            .map_err(|e| format!("Another RiceWine instance may be using this library: {e}"))?;
        let path = root.join("applications.json");
        let catalog: Catalog = if path.exists() {
            let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            serde_json::from_slice(&bytes)
                .map_err(|e| format!("Invalid catalog {}: {e}", path.display()))?
        } else {
            Catalog::default()
        };
        if catalog.schema != SCHEMA {
            return Err(format!(
                "Catalog schema {} is unsupported (expected {SCHEMA}); no data was changed",
                catalog.schema
            ));
        }
        Ok(Self {
            root,
            catalog,
            _lock: lock,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn applications(&self) -> &[Application] {
        &self.catalog.applications
    }

    pub fn get(&self, id: &str) -> Option<&Application> {
        self.catalog.applications.iter().find(|app| app.id == id)
    }

    pub fn add(&mut self, app: Application) -> Result<(), String> {
        if self.get(&app.id).is_some() {
            return Err("Application ID already exists".into());
        }
        let mut next = self.catalog.clone();
        next.applications.push(app);
        self.commit(next)
    }

    pub fn set_executable(&mut self, id: &str, path: PathBuf) -> Result<(), String> {
        validate_executable(&path)?;
        let path = fs::canonicalize(path).map_err(|e| e.to_string())?;
        let mut next = self.catalog.clone();
        let app = next
            .applications
            .iter_mut()
            .find(|app| app.id == id)
            .ok_or("Application not found")?;
        app.executable = Some(path);
        self.commit(next)
    }

    /// Only removes a catalog entry. Prefixes, installed files and logs are preserved.
    pub fn remove_from_library(&mut self, id: &str) -> Result<(), String> {
        let mut next = self.catalog.clone();
        next.applications.retain(|app| app.id != id);
        if next.applications.len() == self.catalog.applications.len() {
            return Err("Application not found".into());
        }
        self.commit(next)
    }

    fn commit(&mut self, next: Catalog) -> Result<(), String> {
        let path = self.root.join("applications.json");
        let tmp = self
            .root
            .join(format!(".applications-{}.tmp", new_id("save")));
        let result = (|| -> io::Result<()> {
            let mut file = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
            serde_json::to_writer_pretty(&mut file, &next).map_err(io::Error::other)?;
            file.write_all(b"\n")?;
            file.sync_all()?;
            fs::rename(&tmp, &path)?;
            Ok(())
        })();
        if let Err(error) = result {
            let _ = fs::remove_file(&tmp);
            return Err(format!("Saving catalog failed: {error}"));
        }
        self.catalog = next;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct WineTools {
    pub wine: PathBuf,
    pub wineboot: PathBuf,
    pub version: String,
}

impl WineTools {
    pub fn detect() -> Result<Self, String> {
        let wine = find_in_path("wine").ok_or("Wine not found in PATH")?;
        let wineboot = find_in_path("wineboot").ok_or("wineboot not found in PATH")?;
        let output = Command::new(&wine)
            .arg("--version")
            .output()
            .map_err(|e| format!("Could not inspect Wine: {e}"))?;
        if !output.status.success() {
            return Err("Wine --version failed".into());
        }
        Ok(Self {
            wine,
            wineboot,
            version: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        })
    }
}

pub enum InstallMode {
    Installer,
    Portable,
}

/// Runs in a worker thread: wineboot can take time, and Windows setup is interactive.
pub fn install(
    root: &Path,
    wine: &WineTools,
    name: &str,
    file: &Path,
    mode: InstallMode,
) -> Result<Application, String> {
    validate_name(name)?;
    validate_installer(file, matches!(mode, InstallMode::Portable))?;
    let file = fs::canonicalize(file).map_err(|e| e.to_string())?;
    let id = new_id(name);
    let prefix = root.join("prefixes").join(&id);
    fs::create_dir(&prefix).map_err(|e| format!("Could not create prefix: {e}"))?;
    let log = root.join("logs").join(format!("{id}.log"));

    let mut boot = Command::new(&wine.wineboot);
    boot.arg("-u").env("WINEPREFIX", &prefix);
    run_logged(&mut boot, &log, Some(Duration::from_secs(120))).map_err(|e| {
        format!(
            "Wine prefix setup failed ({e}); retained at {}",
            prefix.display()
        )
    })?;

    let portable = matches!(mode, InstallMode::Portable);
    if !portable {
        let mut command = install_command(wine, &prefix, &file);
        run_logged(&mut command, &log, None).map_err(|e| {
            format!(
                "Installer failed ({e}); prefix retained at {}",
                prefix.display()
            )
        })?;
    }

    Ok(Application {
        id,
        name: name.trim().to_owned(),
        prefix,
        executable: portable.then(|| file.clone()),
        installer: (!portable).then_some(file),
        portable,
    })
}

pub fn install_command(wine: &WineTools, prefix: &Path, installer: &Path) -> Command {
    let mut command = Command::new(&wine.wine);
    if extension_is(installer, "msi") {
        command.args(["msiexec", "/i"]);
    }
    command.arg(installer).env("WINEPREFIX", prefix);
    command
}

/// Returns a child process so the caller can reap it in a worker thread.
pub fn launch(root: &Path, wine: &WineTools, app: &Application) -> Result<Child, String> {
    let executable = app
        .executable
        .as_ref()
        .ok_or("No launch executable selected. Choose the program's EXE after installation.")?;
    validate_executable(executable)?;
    if !app.prefix.is_dir() {
        return Err("Wine prefix no longer exists".into());
    }
    let log_path = root.join("logs").join(format!("{}.log", app.id));
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| e.to_string())?;
    let mut command = Command::new(&wine.wine);
    command.arg(executable).env("WINEPREFIX", &app.prefix);
    if let Some(parent) = executable.parent() {
        command.current_dir(parent);
    }
    command
        .stdout(Stdio::from(log.try_clone().map_err(|e| e.to_string())?))
        .stderr(Stdio::from(log))
        .spawn()
        .map_err(|e| format!("Launching {} failed: {e}", app.name))
}

pub fn validate_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 80 || trimmed.chars().any(char::is_control) {
        return Err("Enter a name between 1 and 80 characters".into());
    }
    Ok(())
}

pub fn validate_executable(path: &Path) -> Result<(), String> {
    if !path.is_file() || !extension_is(path, "exe") {
        return Err("Choose an existing .exe file".into());
    }
    Ok(())
}

pub fn validate_installer(path: &Path, portable: bool) -> Result<(), String> {
    if !path.is_file() || !(extension_is(path, "exe") || (!portable && extension_is(path, "msi"))) {
        return Err(if portable {
            "Portable programs must be existing .exe files"
        } else {
            "Choose an existing .exe or .msi file"
        }
        .into());
    }
    Ok(())
}

fn extension_is(path: &Path, ext: &str) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(ext))
}

pub(crate) fn new_id(name: &str) -> String {
    let slug: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .take(30)
        .collect();
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!(
        "{}-{millis:x}-{}-{}",
        if slug.is_empty() { "app" } else { &slug },
        std::process::id(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    )
}

fn data_directory() -> Result<PathBuf, String> {
    if let Some(dir) = env::var_os("XDG_DATA_HOME") {
        let path = PathBuf::from(dir);
        if path.is_absolute() {
            return Ok(path.join("ricewine"));
        }
    }
    env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".local/share/ricewine"))
        .ok_or("Neither HOME nor an absolute XDG_DATA_HOME is available".into())
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    env::split_paths(&env::var_os("PATH")?).find_map(|dir| {
        let path = dir.join(name);
        path.is_file().then_some(path)
    })
}

pub(crate) fn run_logged(
    command: &mut Command,
    log_path: &Path,
    timeout: Option<Duration>,
) -> Result<(), String> {
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .map_err(|e| e.to_string())?;
    command
        .stdout(Stdio::from(log.try_clone().map_err(|e| e.to_string())?))
        .stderr(Stdio::from(log));
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let status = if let Some(timeout) = timeout {
        let start = std::time::Instant::now();
        loop {
            match child.try_wait().map_err(|e| e.to_string())? {
                Some(status) => break status,
                None if start.elapsed() < timeout => thread::sleep(Duration::from_millis(100)),
                None => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("timed out after {} seconds", timeout.as_secs()));
                }
            }
        }
    } else {
        child.wait().map_err(|e| e.to_string())?
    };
    if status.success() {
        Ok(())
    } else {
        Err(format!("exit status {status}; see {}", log_path.display()))
    }
}

/// Display-only path, never passed to a shell.
pub fn readable_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
