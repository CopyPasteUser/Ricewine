#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Temp(PathBuf);

impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ricewine-install-test-{}-{}",
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

fn fixture(temp: &Temp) -> (PathBuf, PathBuf) {
    let project = temp.0.join("project");
    fs::create_dir_all(project.join("scripts")).unwrap();
    fs::create_dir_all(project.join("target/release")).unwrap();
    fs::create_dir_all(project.join("assets")).unwrap();
    fs::write(
        project.join("assets/ricewine.png"),
        include_bytes!("../assets/ricewine.png"),
    )
    .unwrap();
    fs::write(
        project.join("scripts/install-user.sh"),
        include_str!("../scripts/install-user.sh"),
    )
    .unwrap();
    fs::write(
        project.join("scripts/uninstall-user.sh"),
        include_str!("../scripts/uninstall-user.sh"),
    )
    .unwrap();
    let fake_bin = temp.0.join("fake-bin");
    fs::create_dir(&fake_bin).unwrap();
    let cargo = fake_bin.join("cargo");
    fs::write(&cargo, "#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&cargo, fs::Permissions::from_mode(0o700)).unwrap();
    (project, fake_bin)
}

fn run_script(script: &Path, temp: &Temp, fake_bin: &Path) -> std::process::Output {
    let path = format!("{}:/usr/bin:/bin", fake_bin.display());
    Command::new("sh")
        .arg(script)
        .env("HOME", &temp.0)
        .env("XDG_DATA_HOME", temp.0.join("share"))
        .env("PATH", path)
        .output()
        .unwrap()
}

#[test]
fn installer_and_uninstaller_preserve_application_data() {
    let temp = Temp::new();
    let (project, fake_bin) = fixture(&temp);
    fs::copy(
        env!("CARGO_BIN_EXE_ricewine"),
        project.join("target/release/ricewine"),
    )
    .unwrap();
    let installer = run_script(&project.join("scripts/install-user.sh"), &temp, &fake_bin);
    assert!(
        installer.status.success(),
        "{}",
        String::from_utf8_lossy(&installer.stderr)
    );
    let binary = temp.0.join(".local/bin/ricewine");
    let menu = temp.0.join("share/applications/ricewine.desktop");
    assert!(binary.is_file());
    let icon = temp.0.join("share/icons/hicolor/128x128/apps/ricewine.png");
    assert_eq!(
        fs::read(&icon).unwrap(),
        include_bytes!("../assets/ricewine.png")
    );
    let desktop = fs::read_to_string(&menu).unwrap();
    assert!(desktop.contains("X-RiceWine-Launcher=true"));
    assert!(desktop.contains("Icon=ricewine"));
    let shortcut_dir = menu.parent().unwrap();
    let owned_shortcut = shortcut_dir.join("ricewine-owned-id.desktop");
    let unrelated_shortcut = shortcut_dir.join("ricewine-unrelated.desktop");
    let linked_shortcut = shortcut_dir.join("ricewine-linked-id.desktop");
    let outside_shortcut = temp.0.join("outside.desktop");
    fs::write(&owned_shortcut, "[Desktop Entry]\nX-RiceWine-ID=owned-id\n").unwrap();
    fs::write(&unrelated_shortcut, "[Desktop Entry]\nName=Other app\n").unwrap();
    fs::write(
        &outside_shortcut,
        "[Desktop Entry]\nX-RiceWine-ID=linked-id\n",
    )
    .unwrap();
    std::os::unix::fs::symlink(&outside_shortcut, &linked_shortcut).unwrap();
    let data = temp.0.join("share/ricewine/keep-me.txt");
    fs::create_dir_all(data.parent().unwrap()).unwrap();
    fs::write(&data, "user data").unwrap();
    let uninstaller = run_script(&project.join("scripts/uninstall-user.sh"), &temp, &fake_bin);
    assert!(
        uninstaller.status.success(),
        "{}",
        String::from_utf8_lossy(&uninstaller.stderr)
    );
    assert!(!binary.exists());
    assert!(!menu.exists());
    assert!(!icon.exists());
    assert!(!owned_shortcut.exists());
    assert!(unrelated_shortcut.exists());
    assert!(
        linked_shortcut
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert!(outside_shortcut.exists());
    assert_eq!(fs::read_to_string(&data).unwrap(), "user data");
}

#[test]
fn installer_rolls_back_existing_binary_on_registration_failure() {
    let temp = Temp::new();
    let (project, fake_bin) = fixture(&temp);
    let binary = temp.0.join(".local/bin/ricewine");
    fs::create_dir_all(binary.parent().unwrap()).unwrap();
    let previous = "#!/bin/sh\nif [ \"$1\" = --version ]; then echo 'RiceWine 0.9.0'; fi\n";
    fs::write(&binary, previous).unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
    let icon = temp.0.join("share/icons/hicolor/128x128/apps/ricewine.png");
    fs::create_dir_all(icon.parent().unwrap()).unwrap();
    fs::write(&icon, "old icon data").unwrap();
    let failing = project.join("target/release/ricewine");
    fs::write(&failing, "#!/bin/sh\nexit 1\n").unwrap();
    fs::set_permissions(&failing, fs::Permissions::from_mode(0o700)).unwrap();
    let output = run_script(&project.join("scripts/install-user.sh"), &temp, &fake_bin);
    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(binary).unwrap(), previous);
    assert_eq!(fs::read_to_string(icon).unwrap(), "old icon data");
}

#[test]
fn installer_refuses_to_overwrite_unrelated_regular_file() {
    let temp = Temp::new();
    let (project, fake_bin) = fixture(&temp);
    let binary = temp.0.join(".local/bin/ricewine");
    fs::create_dir_all(binary.parent().unwrap()).unwrap();
    fs::write(&binary, "unrelated data").unwrap();
    let output = run_script(&project.join("scripts/install-user.sh"), &temp, &fake_bin);
    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(binary).unwrap(), "unrelated data");
}

#[test]
fn installer_rejects_empty_home_before_creating_files() {
    let temp = Temp::new();
    let (project, fake_bin) = fixture(&temp);
    let output = Command::new("sh")
        .arg(project.join("scripts/install-user.sh"))
        .env("HOME", "")
        .env("PATH", format!("{}:/usr/bin:/bin", fake_bin.display()))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("HOME must"));
    assert!(!temp.0.join(".local").exists());
}

#[test]
fn uninstaller_rejects_binary_not_identified_as_ricewine() {
    let temp = Temp::new();
    let (project, fake_bin) = fixture(&temp);
    let binary = temp.0.join(".local/bin/ricewine");
    fs::create_dir_all(binary.parent().unwrap()).unwrap();
    fs::write(&binary, "#!/bin/sh\necho unrecognized\n").unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
    let output = run_script(&project.join("scripts/uninstall-user.sh"), &temp, &fake_bin);
    assert!(!output.status.success());
    assert!(binary.exists());
}

#[test]
fn installer_does_not_replace_a_symlink_to_another_binary() {
    let temp = Temp::new();
    let (project, fake_bin) = fixture(&temp);
    let destination = temp.0.join(".local/bin/ricewine");
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    let outside = temp.0.join("unrelated-binary");
    fs::write(&outside, "do not replace").unwrap();
    std::os::unix::fs::symlink(&outside, &destination).unwrap();
    let output = run_script(&project.join("scripts/install-user.sh"), &temp, &fake_bin);
    assert!(!output.status.success());
    assert!(
        destination
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(fs::read_to_string(outside).unwrap(), "do not replace");
}

#[test]
fn uninstall_preserves_a_user_customized_icon() {
    let temp = Temp::new();
    let (project, fake_bin) = fixture(&temp);
    fs::copy(
        env!("CARGO_BIN_EXE_ricewine"),
        project.join("target/release/ricewine"),
    )
    .unwrap();
    let installer = run_script(&project.join("scripts/install-user.sh"), &temp, &fake_bin);
    assert!(installer.status.success());
    let icon = temp.0.join("share/icons/hicolor/128x128/apps/ricewine.png");
    fs::write(&icon, b"personal artwork").unwrap();
    let removal = run_script(&project.join("scripts/uninstall-user.sh"), &temp, &fake_bin);
    assert!(removal.status.success());
    assert_eq!(fs::read(&icon).unwrap(), b"personal artwork");
}
