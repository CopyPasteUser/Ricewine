use ricewine::core::{InstallMode, Store};
use ricewine::runtime::{self, PrefixProfile, Runner};
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Temp(PathBuf);

impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ricewine-runtime-{}-{}",
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
fn missing_profile_is_backward_compatible() {
    let tmp = Temp::new();
    assert_eq!(
        runtime::load_profile(&tmp.0).unwrap(),
        PrefixProfile::default()
    );
    assert!(!runtime::profile_path(&tmp.0).exists());
}

#[test]
fn profile_round_trip_and_rejects_unknown_versions_without_overwrite() {
    let tmp = Temp::new();
    let wine = tmp.0.join("wine");
    File::create(&wine).unwrap();
    File::create(tmp.0.join("wineboot")).unwrap();
    let profile = PrefixProfile {
        runner: Runner::CustomWine { wine },
        ..Default::default()
    };
    runtime::save_profile(&tmp.0, &profile).unwrap();
    assert_eq!(runtime::load_profile(&tmp.0).unwrap(), profile);
    let original = fs::read(runtime::profile_path(&tmp.0)).unwrap();
    let unsupported = PrefixProfile {
        version: 42,
        ..profile
    };
    assert!(runtime::save_profile(&tmp.0, &unsupported).is_err());
    assert_eq!(fs::read(runtime::profile_path(&tmp.0)).unwrap(), original);
    fs::write(runtime::profile_path(&tmp.0), "version = 99\n").unwrap();
    assert!(runtime::load_profile(&tmp.0).is_err());
}

#[test]
fn custom_runner_preserves_exe_path_and_prefix() {
    let tmp = Temp::new();
    let wine = tmp.0.join("wine");
    File::create(&wine).unwrap();
    File::create(tmp.0.join("wineboot")).unwrap();
    let prefix = tmp.0.join("prefix with spaces");
    let exe = tmp.0.join("setup with spaces.msi");
    let profile = PrefixProfile {
        runner: Runner::CustomWine { wine: wine.clone() },
        ..Default::default()
    };
    let command = runtime::runner_command(&profile, None, &prefix, &exe).unwrap();
    assert_eq!(command.get_program(), wine);
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        ["msiexec".as_ref(), "/i".as_ref(), exe.as_os_str()]
    );
    assert!(
        command
            .get_envs()
            .any(|(key, val)| key == "WINEPREFIX" && val == Some(prefix.as_os_str()))
    );
}

#[test]
fn invalid_proton_profile_and_unknown_verbs_are_rejected() {
    let tmp = Temp::new();
    let proton = tmp.0.join("proton-directory");
    fs::create_dir(&proton).unwrap();
    let profile = PrefixProfile {
        runner: Runner::ProtonUmu {
            proton,
            game_id: "bad;id".into(),
        },
        ..Default::default()
    };
    assert!(runtime::save_profile(&tmp.0, &profile).is_err());
    assert!(runtime::log_tail(&tmp.0, "../../etc/passwd", 1024).is_err());
    let store = Store::open_at(tmp.0.join("data")).unwrap();
    let exe = tmp.0.join("app.exe");
    File::create(&exe).unwrap();
    let app = ricewine::core::Application {
        id: "safe-app".into(),
        name: "Safe".into(),
        prefix: tmp.0.clone(),
        executable: Some(exe),
        installer: None,
        portable: true,
    };
    assert!(runtime::run_winetricks(store.root(), &app, &PrefixProfile::default(), "-q").is_err());
}

#[test]
fn bounded_log_tail_does_not_load_entire_file() {
    let tmp = Temp::new();
    let root = tmp.0.join("data");
    fs::create_dir_all(root.join("logs")).unwrap();
    fs::write(root.join("logs/game-1.log"), "a".repeat(200_000) + "END").unwrap();
    assert_eq!(runtime::log_tail(&root, "game-1", 3).unwrap(), "END");
    assert_eq!(runtime::log_tail(&root, "absent", 3).unwrap(), "");
}

#[cfg(unix)]
#[test]
fn custom_runner_installs_without_system_wine() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = Temp::new();
    let wine = tmp.0.join("wine");
    fs::write(&wine, "#!/bin/sh\nprintf '%s\n' \"$WINEPREFIX\" \"$@\"\n").unwrap();
    fs::set_permissions(&wine, fs::Permissions::from_mode(0o700)).unwrap();
    let boot = tmp.0.join("wineboot");
    fs::write(&boot, "#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&boot, fs::Permissions::from_mode(0o700)).unwrap();
    let exe = tmp.0.join("hello.exe");
    File::create(&exe).unwrap();
    let store = Store::open_at(tmp.0.join("data")).unwrap();
    let profile = PrefixProfile {
        runner: Runner::CustomWine { wine },
        ..Default::default()
    };
    let app = runtime::install_with_profile(
        store.root(),
        None,
        &profile,
        "Custom",
        &exe,
        InstallMode::Installer,
    )
    .unwrap();
    assert_eq!(runtime::load_profile(&app.prefix).unwrap(), profile);
    assert!(app.executable.is_none());
    assert!(app.prefix.exists());
}

#[test]
fn launch_options_validate_without_partially_overwriting_profile() {
    let mut profile = PrefixProfile::default();
    runtime::parse_launch_configuration(
        "--windowed\n--title=Hello World",
        "WINEESYNC=1\nDXVK_HUD=fps",
        &mut profile,
    )
    .unwrap();
    assert_eq!(
        profile.launch_arguments,
        ["--windowed", "--title=Hello World"]
    );
    assert_eq!(profile.launch_environment.get("DXVK_HUD").unwrap(), "fps");
    let original = profile.clone();
    assert!(
        runtime::parse_launch_configuration("--other", "LD_PRELOAD=/tmp/bad.so", &mut profile)
            .is_err()
    );
    assert_eq!(profile, original);
    assert!(
        runtime::parse_launch_configuration("", "WINEPREFIX=/tmp/other", &mut profile).is_err()
    );
    assert_eq!(profile, original);
    assert!(
        runtime::parse_launch_configuration("", "DXVK_HUD=one\nDXVK_HUD=two", &mut profile)
            .is_err()
    );
    assert_eq!(profile, original);
}

#[test]
fn proton_discovery_checks_only_known_locations_and_deduplicates() {
    let tmp = Temp::new();
    let build = tmp
        .0
        .join(".steam/root/compatibilitytools.d/GE-Proton-test");
    fs::create_dir_all(&build).unwrap();
    File::create(build.join("proton")).unwrap();
    fs::create_dir_all(tmp.0.join("unknown/proton")).unwrap();
    let found = runtime::discover_proton(&tmp.0);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "GE-Proton-test");
}

#[cfg(unix)]
#[test]
fn launch_passes_arguments_and_environment_without_shell_evaluation() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = Temp::new();
    let store = Store::open_at(tmp.0.join("data")).unwrap();
    let wine = tmp.0.join("wine");
    fs::write(
        &wine,
        "#!/bin/sh\nprintf '%s\n' \"$WINEPREFIX\" \"$WINEESYNC\" \"$@\"\n",
    )
    .unwrap();
    fs::set_permissions(&wine, fs::Permissions::from_mode(0o700)).unwrap();
    File::create(tmp.0.join("wineboot")).unwrap();
    let prefix = store.root().join("prefixes/game-one");
    fs::create_dir_all(&prefix).unwrap();
    let exe = tmp.0.join("app with spaces.exe");
    File::create(&exe).unwrap();
    let app = ricewine::core::Application {
        id: "game-one".into(),
        name: "Test".into(),
        prefix: prefix.clone(),
        executable: Some(exe.clone()),
        installer: None,
        portable: true,
    };
    let mut profile = PrefixProfile {
        runner: Runner::CustomWine { wine },
        ..Default::default()
    };
    runtime::parse_launch_configuration("--caption=two words", "WINEESYNC=1", &mut profile)
        .unwrap();
    let mut child = runtime::launch(store.root(), None, &app, &profile).unwrap();
    assert!(child.wait().unwrap().success());
    let log = fs::read_to_string(store.root().join("logs/game-one.log")).unwrap();
    assert_eq!(
        log.lines().collect::<Vec<_>>(),
        [
            prefix.to_str().unwrap(),
            "1",
            exe.to_str().unwrap(),
            "--caption=two words"
        ]
    );
}

#[test]
fn invalid_profile_never_creates_an_orphan_prefix() {
    let tmp = Temp::new();
    let store = Store::open_at(tmp.0.join("data")).unwrap();
    let exe = tmp.0.join("installer.exe");
    File::create(&exe).unwrap();
    let invalid_profiles = [
        PrefixProfile {
            version: 99,
            ..Default::default()
        },
        PrefixProfile {
            launch_arguments: vec!["invalid\nargument".into()],
            ..Default::default()
        },
    ];
    for profile in invalid_profiles {
        assert!(
            runtime::install_with_profile(
                store.root(),
                None,
                &profile,
                "Invalid",
                &exe,
                InstallMode::Installer
            )
            .is_err()
        );
    }
    assert!(
        fs::read_dir(store.root().join("prefixes"))
            .unwrap()
            .next()
            .is_none(),
        "Invalid settings must be rejected before filesystem changes"
    );
}

#[cfg(unix)]
#[test]
fn winetricks_component_command_uses_exact_runner_and_prefix() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = Temp::new();
    let wine = tmp.0.join("wine");
    File::create(&wine).unwrap();
    File::create(tmp.0.join("wineboot")).unwrap();
    let component_tool = tmp.0.join("fake-winetricks");
    fs::write(
        &component_tool,
        "#!/bin/sh\nprintf '%s\\n' \"$WINEPREFIX\" \"$WINE\" \"$1\" \"$2\"\n",
    )
    .unwrap();
    fs::set_permissions(&component_tool, fs::Permissions::from_mode(0o700)).unwrap();
    let prefix = tmp.0.join("prefix with spaces");
    fs::create_dir(&prefix).unwrap();
    let app = ricewine::core::Application {
        id: "test-app".into(),
        name: "Test".into(),
        prefix: prefix.clone(),
        executable: None,
        installer: None,
        portable: false,
    };
    let profile = PrefixProfile {
        runner: Runner::CustomWine { wine: wine.clone() },
        ..Default::default()
    };
    for component in ["dxvk", "vkd3d", "corefonts"] {
        let output = runtime::winetricks_command(&component_tool, &app, &profile, component)
            .unwrap()
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8(output.stdout)
                .unwrap()
                .lines()
                .collect::<Vec<_>>(),
            [
                prefix.to_str().unwrap(),
                wine.to_str().unwrap(),
                "-q",
                component
            ]
        );
    }
    assert!(runtime::winetricks_command(&component_tool, &app, &profile, "--help").is_err());
    let proton_profile = PrefixProfile {
        runner: Runner::ProtonUmu {
            proton: tmp.0.join("proton"),
            game_id: "0".into(),
        },
        ..Default::default()
    };
    assert!(runtime::winetricks_command(&component_tool, &app, &proton_profile, "dxvk").is_err());
}

#[cfg(unix)]
#[test]
fn proton_cli_launch_passes_exact_umu_environment_and_arguments() {
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;

    let tmp = Temp::new();
    let bin = tmp.0.join("bin");
    fs::create_dir(&bin).unwrap();
    let umu = bin.join("umu-run");
    fs::write(
        &umu,
        "#!/bin/sh\nprintf '%s\\n' \"$PROTONPATH\" \"$GAMEID\" \"$WINEPREFIX\" \"$1\" \"$2\"\n",
    )
    .unwrap();
    fs::set_permissions(&umu, fs::Permissions::from_mode(0o700)).unwrap();
    let proton = tmp.0.join("GE Proton with spaces");
    fs::create_dir(&proton).unwrap();
    File::create(proton.join("proton")).unwrap();

    let xdg_data = tmp.0.join("share");
    let mut store = Store::open_at(xdg_data.join("ricewine")).unwrap();
    let prefix = store.root().join("prefixes/game-1");
    fs::create_dir(&prefix).unwrap();
    let exe = tmp.0.join("My Game.exe");
    File::create(&exe).unwrap();
    let app = ricewine::core::Application {
        id: "game-1".into(),
        name: "Game".into(),
        prefix: prefix.clone(),
        executable: Some(exe.clone()),
        installer: None,
        portable: true,
    };
    store.add(app).unwrap();

    let profile = PrefixProfile {
        runner: Runner::ProtonUmu {
            proton: proton.clone(),
            game_id: "custom-game-42".into(),
        },
        launch_arguments: vec!["--windowed".into()],
        ..Default::default()
    };
    // The child, not the test runner, gets the fake umu-run in PATH.
    // Serialize directly so unrelated parallel tests never mutate the host PATH.
    fs::write(
        prefix.join(".ricewine.toml"),
        toml::to_string_pretty(&profile).unwrap(),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ricewine"))
        .args(["--launch", "game-1"])
        .env("XDG_DATA_HOME", xdg_data)
        .env("PATH", bin)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let log = fs::read_to_string(store.root().join("logs/game-1.log")).unwrap();
    assert_eq!(
        log.lines().collect::<Vec<_>>(),
        [
            proton.to_str().unwrap(),
            "custom-game-42",
            prefix.to_str().unwrap(),
            exe.to_str().unwrap(),
            "--windowed"
        ]
    );
}
