use ricewine::preferences::{self, Appearance, Preferences};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Temp(PathBuf);

impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ricewine-preferences-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn preferences_round_trip_and_malformed_data_is_not_replaced() {
    let temp = Temp::new();
    assert_eq!(preferences::load(&temp.0).unwrap(), Preferences::default());
    let saved = Preferences {
        appearance: Appearance::Luna,
        follow_desktop: true,
    };
    preferences::save(&temp.0, &saved).unwrap();
    assert_eq!(preferences::load(&temp.0).unwrap(), saved);
    let path = temp.0.join("preferences.json");
    fs::write(&path, b"not json").unwrap();
    assert!(preferences::load(&temp.0).is_err());
    assert_eq!(fs::read(path).unwrap(), b"not json");
}

#[test]
fn ricebowl_theme_detection_is_explicit_and_does_not_guess() {
    assert_eq!(
        preferences::appearance_for_theme("RiceHub-Luna"),
        Some(Appearance::Luna)
    );
    assert_eq!(
        preferences::appearance_for_theme("winxp"),
        Some(Appearance::Luna)
    );
    assert_eq!(
        preferences::appearance_for_theme("WhiteSur-Dark"),
        Some(Appearance::MacOS)
    );
    assert_eq!(preferences::appearance_for_theme("RiceHub-Ten"), None);
    assert_eq!(
        preferences::appearance_for_theme("Windows-10-Dark"),
        Some(Appearance::Windows10)
    );
    assert_eq!(
        preferences::appearance_for_theme("Windows10-Fluent"),
        Some(Appearance::Windows10)
    );
    for (desktop, expected) in [
        ("RiceHub-Classic", Appearance::Windows98),
        ("RiceHub-Aero", Appearance::Windows7),
        ("RiceHub-Chrome", Appearance::ChromeOS),
        ("RiceHub-Frutiger", Appearance::FrutigerAero),
        ("RiceHub-Ten-Dark", Appearance::Windows10),
        ("RiceHub-Ten-Light", Appearance::Windows10),
    ] {
        assert_eq!(
            preferences::appearance_for_theme(desktop),
            Some(expected),
            "{desktop}"
        );
    }
    assert_eq!(preferences::appearance_for_theme("Adwaita"), None);
}

#[test]
fn maintained_rices_round_trip_and_legacy_values_remain_readable() {
    for appearance in [
        Appearance::Luna,
        Appearance::MacOS,
        Appearance::Windows10,
        Appearance::Windows98,
        Appearance::Windows7,
        Appearance::ChromeOS,
        Appearance::FrutigerAero,
    ] {
        let preferences = Preferences {
            appearance,
            follow_desktop: false,
        };
        let json = serde_json::to_string(&preferences).unwrap();
        assert_eq!(
            serde_json::from_str::<Preferences>(&json).unwrap(),
            preferences
        );
    }

    for old_name in ["luna_olive", "luna_silver"] {
        let json = format!(r#"{{"appearance":"{old_name}","follow_desktop":true}}"#);
        let result = serde_json::from_str::<Preferences>(&json).unwrap();
        assert_eq!(result.appearance, Appearance::Luna);
        assert!(result.follow_desktop);
    }
    for legacy in [
        "windows10",
        "windows10_dark",
        "windows10_light",
        "windows_10",
    ] {
        let json = format!(r#"{{"appearance":"{legacy}","follow_desktop":true}}"#);
        let result = serde_json::from_str::<Preferences>(&json).unwrap();
        assert_eq!(result.appearance, Appearance::Windows10);
        assert!(result.follow_desktop);
    }
}
