//! Optional RiceBowl appearance detection and persisted GUI preferences.
//! No desktop settings are changed or background services launched.

use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SAVE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Appearance {
    #[default]
    Windows10,
    Luna,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Preferences {
    pub appearance: Appearance,
    pub follow_desktop: bool,
}

fn file_path(data_root: &Path) -> PathBuf {
    data_root.join("preferences.json")
}

pub fn load(data_root: &Path) -> Result<Preferences, String> {
    let path = file_path(data_root);
    match fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map_err(|e| format!("Preferences are invalid ({}): {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Preferences::default()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn save(data_root: &Path, preferences: &Preferences) -> Result<(), String> {
    if !data_root.is_dir() {
        return Err("RiceWine data directory does not exist".into());
    }
    let stage = data_root.join(format!(
        ".preferences-{}-{}.tmp",
        std::process::id(),
        NEXT_SAVE.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| -> std::io::Result<()> {
        let mut file = File::create_new(&stage)?;
        serde_json::to_writer_pretty(&mut file, preferences).map_err(std::io::Error::other)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        fs::rename(&stage, file_path(data_root))?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(stage);
        return Err(error.to_string());
    }
    Ok(())
}

/// Only recognize supported RiceBowl families; do not guess at unknown themes.
pub fn appearance_for_theme(theme: &str) -> Option<Appearance> {
    let name = theme.to_ascii_lowercase();
    if name.contains("luna") || name.contains("winxp") || name.contains("windows-xp") {
        Some(Appearance::Luna)
    } else if name.contains("ricehub-ten") || name.contains("win10") || name.contains("windows-10")
    {
        Some(Appearance::Windows10)
    } else {
        None
    }
}

pub fn detect_desktop_appearance() -> Option<Appearance> {
    if let Some(theme) = env::var_os("GTK_THEME")
        && let Some(appearance) = appearance_for_theme(&theme.to_string_lossy())
    {
        return Some(appearance);
    }
    // XFCE / RiceBowl's configured GTK theme is the source of truth when it
    // exists. The command is read-only and run only at startup/explicit refresh.
    let output = Command::new("xfconf-query")
        .args(["-c", "xsettings", "-p", "/Net/ThemeName"])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| appearance_for_theme(&String::from_utf8_lossy(&output.stdout)))
        .flatten()
}
