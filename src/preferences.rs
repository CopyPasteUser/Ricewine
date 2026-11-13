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
    // Old experiments are retired. Deserialize them into the maintained Luna
    // Rice rather than rejecting existing preferences.json files at startup.
    #[default]
    #[serde(alias = "luna_olive", alias = "luna_silver")]
    Luna,
    #[serde(rename = "macos", alias = "mac_os")]
    MacOS,
    #[serde(
        alias = "windows10_dark",
        alias = "windows10_light",
        alias = "windows_10"
    )]
    Windows10,
    #[serde(alias = "win98", alias = "windows_98", alias = "winclassic")]
    Windows98,
    #[serde(alias = "win7", alias = "windows_7", alias = "aero")]
    Windows7,
    #[serde(rename = "chromeos", alias = "chrome_os", alias = "chrome")]
    ChromeOS,
    #[serde(rename = "frutiger", alias = "frutiger_aero", alias = "aqua")]
    FrutigerAero,
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

/// Recognize only actively maintained Rice families; unknown desktop themes
/// leave the current application appearance untouched.
pub fn appearance_for_theme(theme: &str) -> Option<Appearance> {
    let name = theme.to_ascii_lowercase();
    // Match specific RiceBowl family names before generic visual keywords:
    // "frutiger-aero" must not be confused with Windows 7 Aero.
    if name.contains("frutiger") {
        Some(Appearance::FrutigerAero)
    } else if name.contains("ricehub-classic")
        || name.contains("windows-98")
        || name.contains("windows98")
        || name.contains("winclassic")
        || name.contains("win98")
    {
        Some(Appearance::Windows98)
    } else if name.contains("ricehub-chrome")
        || name.contains("chromeos")
        || name.contains("chrome-os")
    {
        Some(Appearance::ChromeOS)
    } else if name.contains("ricehub-aero")
        || name.contains("windows-7")
        || name.contains("windows7")
        || name.contains("win7")
    {
        Some(Appearance::Windows7)
    } else if name.contains("luna") || name.contains("winxp") || name.contains("windows-xp") {
        Some(Appearance::Luna)
    } else if name.contains("ricehub-ten")
        || name.contains("windows-10")
        || name.contains("windows10")
        || name.contains("win10")
        || name.contains("fluent")
    {
        Some(Appearance::Windows10)
    } else if name.contains("whitesur")
        || name.contains("macrice")
        || name.contains("macos")
        || name.contains("mac-os")
        || name.contains("cupertino")
    {
        Some(Appearance::MacOS)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_appearances_are_migrated_without_losing_preferences() {
        for legacy in ["luna_olive", "luna_silver"] {
            let json = format!(r#"{{"appearance":"{legacy}","follow_desktop":true}}"#);
            let loaded: Preferences = serde_json::from_str(&json).unwrap();
            assert_eq!(loaded.appearance, Appearance::Luna, "{legacy}");
            assert!(loaded.follow_desktop);
        }
    }

    #[test]
    fn only_curated_rices_round_trip() {
        for appearance in [
            Appearance::Luna,
            Appearance::MacOS,
            Appearance::Windows10,
            Appearance::Windows98,
            Appearance::Windows7,
            Appearance::ChromeOS,
            Appearance::FrutigerAero,
        ] {
            let json = serde_json::to_string(&appearance).unwrap();
            let decoded: Appearance = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, appearance);
        }
        assert_eq!(
            serde_json::to_string(&Appearance::MacOS).unwrap(),
            r#""macos""#
        );
    }

    #[test]
    fn desktop_detection_supports_all_seven_ricebowl_families() {
        assert_eq!(
            appearance_for_theme("WhiteSur-Dark"),
            Some(Appearance::MacOS)
        );
        assert_eq!(appearance_for_theme("XP-Luna-Blue"), Some(Appearance::Luna));
        assert_eq!(
            appearance_for_theme("Windows-10-Dark"),
            Some(Appearance::Windows10)
        );
        assert_eq!(
            appearance_for_theme("Fluent-Light"),
            Some(Appearance::Windows10)
        );
        assert_eq!(
            serde_json::from_str::<Appearance>(r#""windows10_dark""#).unwrap(),
            Appearance::Windows10
        );
        for (name, expected) in [
            ("RiceHub-Luna", Appearance::Luna),
            ("RiceHub-Ten-Dark", Appearance::Windows10),
            ("RiceHub-Ten-Light", Appearance::Windows10),
            ("RiceHub-Classic", Appearance::Windows98),
            ("RiceHub-Aero", Appearance::Windows7),
            ("RiceHub-Chrome", Appearance::ChromeOS),
            ("RiceHub-Frutiger", Appearance::FrutigerAero),
            ("Frutiger-Aero", Appearance::FrutigerAero),
            ("MacRice", Appearance::MacOS),
        ] {
            assert_eq!(appearance_for_theme(name), Some(expected), "{name}");
        }
        assert_eq!(appearance_for_theme("Adwaita"), None);
        // Imported or unknown desktop themes must not silently reset the manual choice.
        assert_eq!(appearance_for_theme("Some-Unknown-Rice"), None);
    }

    #[test]
    fn old_preferences_still_deserialize_as_their_maintained_families() {
        for (old, current) in [
            ("winclassic", Appearance::Windows98),
            ("win7", Appearance::Windows7),
            ("chrome_os", Appearance::ChromeOS),
            ("frutiger", Appearance::FrutigerAero),
            ("windows10_light", Appearance::Windows10),
            ("luna_olive", Appearance::Luna),
        ] {
            let json = format!(r#"{{"appearance":"{old}","follow_desktop":true}}"#);
            let parsed: Preferences = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed.appearance, current, "{old}");
            assert!(parsed.follow_desktop);
        }
    }
}
