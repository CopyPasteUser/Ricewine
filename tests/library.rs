use ricewine::core::{Application, Store};
use ricewine::library;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Temp(PathBuf);

impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ricewine-library-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
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
fn snapshot_does_not_touch_live_data_or_copy_prefixes() {
    let temp = Temp::new();
    let mut store = Store::open_at(temp.0.join("data")).unwrap();
    let prefix = store.root().join("prefixes/app-1");
    fs::create_dir(&prefix).unwrap();
    fs::write(prefix.join("large-win-file.bin"), b"untouched").unwrap();
    let app = Application {
        id: "app-1".into(),
        name: "Example".into(),
        prefix,
        executable: None,
        installer: None,
        portable: true,
    };
    store.add(app).unwrap();
    fs::write(
        store.root().join("preferences.json"),
        "{\"appearance\":\"luna\"}",
    )
    .unwrap();
    let old_catalog = fs::read(store.root().join("applications.json")).unwrap();
    let snapshot = library::snapshot_metadata(store.root()).unwrap();
    assert_eq!(
        fs::read(snapshot.join("applications.json")).unwrap(),
        old_catalog
    );
    assert!(snapshot.join("preferences.json").is_file());
    assert!(!snapshot.join("prefixes").exists());
    assert!(!snapshot.join("large-win-file.bin").exists());
    assert_eq!(
        fs::read(store.root().join("applications.json")).unwrap(),
        old_catalog
    );
    let second = library::snapshot_metadata(store.root()).unwrap();
    assert_ne!(snapshot, second);
}

#[test]
fn diagnostics_reveal_bad_profiles_and_missing_paths_without_changes() {
    let temp = Temp::new();
    let prefix = temp.0.join("prefix");
    fs::create_dir(&prefix).unwrap();
    fs::write(prefix.join(".ricewine.toml"), "version = 999\n").unwrap();
    let data = fs::read(prefix.join(".ricewine.toml")).unwrap();
    let apps = vec![Application {
        id: "broken-1".into(),
        name: "Broken app".into(),
        prefix,
        executable: Some(temp.0.join("missing.exe")),
        installer: None,
        portable: false,
    }];
    let report = library::check(&apps);
    assert!(!report.is_healthy());
    assert_eq!(report.checked, 1);
    assert_eq!(report.errors.len(), 1);
    assert_eq!(report.warnings.len(), 1);
    assert_eq!(
        fs::read(apps[0].prefix.join(".ricewine.toml")).unwrap(),
        data
    );
}

#[test]
fn snapshot_rejects_invalid_catalog_and_oversized_preferences() {
    let temp = Temp::new();
    let root = temp.0.join("data");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("applications.json"), b"invalid json").unwrap();
    assert!(library::snapshot_metadata(&root).is_err());
    assert!(!root.join("backups").exists());
    fs::write(
        root.join("applications.json"),
        br#"{"schema":1,"applications":[]}"#,
    )
    .unwrap();
    fs::write(root.join("preferences.json"), vec![0u8; 64 * 1024 + 1]).unwrap();
    assert!(library::snapshot_metadata(&root).is_err());
    assert!(!root.join("backups").exists());
}
