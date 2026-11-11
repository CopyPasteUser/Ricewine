use ricewine::core::{self, InstallMode, Store, WineTools};
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEST: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ricewine-test-{}-{}",
            std::process::id(),
            NEXT_TEST.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn catalog_round_trip_and_safe_removal() {
    let tmp = TempDir::new();
    let program = tmp.0.join("Game.EXE");
    File::create(&program).unwrap();
    let mut store = Store::open_at(tmp.0.join("data")).unwrap();
    let app = ricewine::core::Application {
        id: "game-123".to_owned(),
        name: "Game".to_owned(),
        prefix: store.root().join("prefixes/game-123"),
        executable: Some(program),
        installer: None,
        portable: true,
    };
    fs::create_dir(&app.prefix).unwrap();
    store.add(app.clone()).unwrap();
    assert_eq!(Store::read_only_at(store.root()).unwrap().len(), 1);
    store.rename(&app.id, "Renamed game").unwrap();
    assert_eq!(store.get(&app.id).unwrap().name, "Renamed game");
    assert!(Store::open_at(tmp.0.join("data")).is_err());
    drop(store);
    let probe = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(tmp.0.join("data/catalog.lock"))
        .unwrap();
    assert!(
        probe.try_lock().is_ok(),
        "store drop did not release the catalog lock"
    );
    drop(probe);
    let mut reloaded = Store::open_at(tmp.0.join("data")).unwrap();
    assert_eq!(reloaded.applications().len(), 1);
    assert_eq!(reloaded.applications()[0].name, "Renamed game");
    reloaded.remove_from_library(&app.id).unwrap();
    assert!(reloaded.applications().is_empty());
    assert!(
        app.prefix.exists(),
        "removing an entry must preserve prefix data"
    );
}

#[test]
fn exclusive_lock_releases_when_file_is_dropped() {
    let tmp = TempDir::new();
    let path = tmp.0.join("lock");
    let first = File::create(&path).unwrap();
    first.try_lock().unwrap();
    let second = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .unwrap();
    assert!(second.try_lock().is_err());
    drop(first);
    assert!(second.try_lock().is_ok());
}

#[test]
fn invalid_catalog_is_not_overwritten() {
    let tmp = TempDir::new();
    let root = tmp.0.join("data");
    fs::create_dir(&root).unwrap();
    let catalog = root.join("applications.json");
    fs::write(&catalog, b"broken-json").unwrap();
    assert!(Store::open_at(root).is_err());
    assert_eq!(fs::read(&catalog).unwrap(), b"broken-json");
}

#[test]
fn oversized_and_duplicate_catalogs_are_rejected_without_changes() {
    let tmp = TempDir::new();
    let root = tmp.0.join("data");
    fs::create_dir(&root).unwrap();
    let catalog = root.join("applications.json");
    let too_large = vec![b'0'; 4 * 1024 * 1024 + 1];
    fs::write(&catalog, &too_large).unwrap();
    assert!(Store::open_at(root.clone()).is_err());
    assert_eq!(
        fs::metadata(&catalog).unwrap().len(),
        too_large.len() as u64
    );

    let application = serde_json::json!({
        "id": "duplicate-1", "name": "Example", "prefix": "/tmp/prefix",
        "executable": null, "installer": null, "portable": false
    });
    let payload =
        serde_json::json!({"schema": 1, "applications": [application.clone(), application]});
    fs::write(&catalog, serde_json::to_vec(&payload).unwrap()).unwrap();
    assert!(Store::open_at(root).is_err());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&fs::read(catalog).unwrap()).unwrap(),
        payload
    );
}

#[cfg(unix)]
#[test]
fn new_private_data_root_and_prefix_are_not_world_readable() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = TempDir::new();
    let store = Store::open_at(tmp.0.join("private-data")).unwrap();
    let mode = fs::metadata(store.root()).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o700);
}

#[test]
fn validation_rejects_invalid_files_and_names() {
    let tmp = TempDir::new();
    let exe = tmp.0.join("game.ExE");
    let msi = tmp.0.join("setup.MSI");
    File::create(&exe).unwrap();
    File::create(&msi).unwrap();
    assert!(core::validate_executable(&exe).is_ok());
    assert!(core::validate_installer(&msi, false).is_ok());
    assert!(core::validate_installer(&msi, true).is_err());
    assert!(core::validate_name("   ").is_err());
    assert!(core::validate_name("name\nwith-newline").is_err());
    assert!(core::validate_installer(&tmp.0.join("missing.exe"), false).is_err());
}

#[test]
fn msi_uses_msiexec_with_an_explicit_prefix() {
    let tmp = TempDir::new();
    let tools = WineTools {
        wine: PathBuf::from("/usr/bin/wine"),
        wineboot: PathBuf::from("/usr/bin/wineboot"),
        version: "wine-test".into(),
    };
    let prefix = tmp.0.join("prefix");
    let installer = tmp.0.join("app.msi");
    let command = core::install_command(&tools, &prefix, &installer);
    let args: Vec<_> = command
        .get_args()
        .map(|v| v.to_string_lossy().into_owned())
        .collect();
    assert_eq!(args, ["msiexec", "/i", installer.to_str().unwrap()]);
    assert!(
        command
            .get_envs()
            .any(|(key, value)| key == "WINEPREFIX" && value == Some(prefix.as_os_str()))
    );
}

#[cfg(unix)]
#[test]
fn portable_program_initializes_its_own_prefix() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new();
    let boot = tmp.0.join("wineboot");
    fs::write(&boot, "#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&boot, fs::Permissions::from_mode(0o700)).unwrap();
    let exe = tmp.0.join("portable.exe");
    File::create(&exe).unwrap();
    let root = tmp.0.join("data");
    fs::create_dir_all(root.join("prefixes")).unwrap();
    fs::create_dir_all(root.join("logs")).unwrap();
    let tools = WineTools {
        wine: boot.clone(),
        wineboot: boot,
        version: "fake-wine".into(),
    };
    let app = core::install(&root, &tools, "Portable game", &exe, InstallMode::Portable).unwrap();
    assert!(app.prefix.is_dir());
    assert!(app.prefix.starts_with(root.join("prefixes")));
    assert_eq!(app.executable, Some(exe));
}

#[cfg(unix)]
#[test]
fn msi_installer_executes_with_prefix_and_preserves_argument_boundaries() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new();
    let fake_wine = tmp.0.join("wine");
    fs::write(
        &fake_wine,
        "#!/bin/sh\nprintf '%s\\n' \"$WINEPREFIX\" \"$@\"\n",
    )
    .unwrap();
    fs::set_permissions(&fake_wine, fs::Permissions::from_mode(0o700)).unwrap();
    let fake_boot = tmp.0.join("wineboot");
    fs::write(&fake_boot, "#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&fake_boot, fs::Permissions::from_mode(0o700)).unwrap();
    let installer = tmp.0.join("setup with spaces.MSI");
    File::create(&installer).unwrap();
    let store = Store::open_at(tmp.0.join("data")).unwrap();
    let tools = WineTools {
        wine: fake_wine,
        wineboot: fake_boot,
        version: "fake".into(),
    };

    let app = core::install(
        store.root(),
        &tools,
        "Test app",
        &installer,
        InstallMode::Installer,
    )
    .unwrap();
    assert!(app.prefix.is_dir());
    assert_eq!(app.executable, None);
    let log =
        fs::read_to_string(store.root().join("logs").join(format!("{}.log", app.id))).unwrap();
    let lines: Vec<_> = log.lines().collect();
    assert_eq!(
        lines,
        [
            app.prefix.to_str().unwrap(),
            "msiexec",
            "/i",
            installer.to_str().unwrap()
        ]
    );
}
