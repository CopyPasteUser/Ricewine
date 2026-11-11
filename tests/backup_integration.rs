use ricewine::backup;
use ricewine::core::{self, Application, Store};
use ricewine::integration;
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Temp(PathBuf);

impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ricewine-backup-test-{}-{}",
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

fn sample_app(root: &std::path::Path) -> Application {
    let prefix = root.join("prefixes/app-123");
    let program_dir = prefix.join("drive_c/Program Files/Example");
    fs::create_dir_all(&program_dir).unwrap();
    File::create(prefix.join("system.reg")).unwrap();
    let exe = program_dir.join("program.exe");
    fs::write(&exe, b"windows executable placeholder").unwrap();
    Application {
        id: "app-123".into(),
        name: "Example".into(),
        prefix,
        executable: Some(exe),
        installer: None,
        portable: false,
    }
}

#[test]
fn snapshot_restore_never_overwrites_original_and_remaps_launch_exe() {
    let temp = Temp::new();
    let store = Store::open_at(temp.0.join("data")).unwrap();
    let app = sample_app(store.root());
    let original_exe = app.executable.as_ref().unwrap();
    let created = backup::create_backup(store.root(), &app).unwrap();
    fs::write(original_exe, b"changed after snapshot").unwrap();

    let listing = backup::list_backups(store.root()).unwrap();
    assert_eq!(listing.len(), 1);
    assert_eq!(listing[0].app_name, "Example");
    let restored = backup::restore_backup(store.root(), &created.folder).unwrap();
    assert_ne!(restored.id, app.id);
    assert_ne!(restored.prefix, app.prefix);
    assert!(restored.prefix.starts_with(store.root().join("prefixes")));
    assert_eq!(fs::read(original_exe).unwrap(), b"changed after snapshot");
    assert_eq!(
        fs::read(restored.executable.unwrap()).unwrap(),
        b"windows executable placeholder"
    );
    assert!(app.prefix.exists());
}

#[test]
fn cannot_restore_arbitrary_directory_outside_managed_backups() {
    let temp = Temp::new();
    let store = Store::open_at(temp.0.join("data")).unwrap();
    assert!(backup::restore_backup(store.root(), &temp.0).is_err());
}

#[cfg(unix)]
#[test]
fn snapshot_preserves_symlink_instead_of_dereferencing_external_content() {
    let temp = Temp::new();
    let store = Store::open_at(temp.0.join("data")).unwrap();
    let app = sample_app(store.root());
    std::os::unix::fs::symlink("/etc/passwd", app.prefix.join("outside-link")).unwrap();
    let entry = backup::create_backup(store.root(), &app).unwrap();
    let saved_link = entry.folder.join("prefix/outside-link");
    assert!(
        saved_link
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(
        fs::read_link(saved_link).unwrap(),
        PathBuf::from("/etc/passwd")
    );
}

#[test]
fn import_requires_real_prefix_markers_and_does_not_change_files() {
    let temp = Temp::new();
    let app = sample_app(&temp.0);
    let exe = app.executable.as_ref().unwrap();
    let original = fs::read(exe).unwrap();
    let imported = core::imported_application("Imported", &app.prefix, exe).unwrap();
    assert_eq!(imported.prefix, app.prefix);
    assert_eq!(imported.executable, app.executable);
    assert_eq!(fs::read(exe).unwrap(), original);
    fs::remove_file(app.prefix.join("system.reg")).unwrap();
    assert!(core::imported_application("Bad", &app.prefix, exe).is_err());
}

#[test]
fn desktop_launcher_quotes_program_paths_and_rejects_unsafe_ids() {
    let temp = Temp::new();
    let app = sample_app(&temp.0);
    let entry =
        integration::desktop_entry(&app, &PathBuf::from("/home/user/My App/ricewine")).unwrap();
    assert!(entry.contains("Exec=\"/home/user/My App/ricewine\" --launch app-123"));
    assert!(entry.contains("X-RiceWine-ID=app-123"));
    assert!(integration::shortcut_file_name("../../oops").is_err());
    let open_with =
        integration::open_with_entry(&PathBuf::from("/home/user/My App/ricewine")).unwrap();
    assert!(open_with.contains(" --install-file %f"));
    assert!(open_with.contains("MimeType=application/x-ms-dos-executable;"));
    assert!(!open_with.contains("x-scheme-handler"));
}
