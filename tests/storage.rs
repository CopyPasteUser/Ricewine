use ricewine::core::Application;
use ricewine::storage::{self, human_size};
use std::fs;

fn app(prefix: std::path::PathBuf) -> Application {
    Application {
        id: "demo-123".into(),
        name: "Demo".into(),
        prefix,
        executable: None,
        installer: None,
        portable: false,
    }
}

#[test]
fn usage_counts_regular_files_and_reports_human_sizes() {
    let root = std::env::temp_dir().join(format!("ricewine-usage-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("folder")).unwrap();
    fs::write(root.join("folder/example"), vec![0_u8; 2048]).unwrap();
    assert_eq!(storage::disk_usage(&root).unwrap().bytes, 2048);
    assert_eq!(human_size(2048), "2.0 KiB");
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn cleanup_refuses_imported_prefix_and_ignores_links() {
    let root = std::env::temp_dir().join(format!("ricewine-clean-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let managed = root.join("prefixes/owned");
    let temp = managed.join("drive_c/windows/temp");
    fs::create_dir_all(&temp).unwrap();
    let outside = root.join("outside-important-data");
    fs::write(&outside, b"keep").unwrap();
    std::os::unix::fs::symlink(&outside, temp.join("linked.log")).unwrap();
    let preview = storage::preview_cleanup(&root, &app(managed)).unwrap();
    assert_eq!(preview.files.len(), 0);
    assert!(storage::preview_cleanup(&root, &app(root.clone())).is_err());
    assert_eq!(
        storage::cleanup_old_temp(&root, &app(root.join("prefixes/owned")))
            .unwrap()
            .files,
        0
    );
    assert_eq!(fs::read(outside).unwrap(), b"keep");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn only_old_temp_files_are_removed_after_explicit_cleanup() {
    use std::fs::FileTimes;
    use std::time::{Duration, SystemTime};
    let root = std::env::temp_dir().join(format!("ricewine-clean-age-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let prefix = root.join("prefixes/owned");
    let temporary = prefix.join("drive_c/windows/temp");
    fs::create_dir_all(&temporary).unwrap();
    let old = temporary.join("old-cache.tmp");
    let fresh = temporary.join("working.tmp");
    fs::write(&old, b"old").unwrap();
    fs::write(&fresh, b"fresh").unwrap();
    let handle = fs::OpenOptions::new().write(true).open(&old).unwrap();
    handle
        .set_times(
            FileTimes::new().set_modified(SystemTime::now() - Duration::from_secs(10 * 24 * 3600)),
        )
        .unwrap();
    let preview = storage::preview_cleanup(&root, &app(prefix.clone())).unwrap();
    assert_eq!(preview.files.as_slice(), std::slice::from_ref(&old));
    assert_eq!(preview.bytes, 3);
    let removed = storage::cleanup_old_temp(&root, &app(prefix)).unwrap();
    assert_eq!(removed.files, 1);
    assert!(!old.exists());
    assert_eq!(fs::read(fresh).unwrap(), b"fresh");
    fs::remove_dir_all(root).unwrap();
}
