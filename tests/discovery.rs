use ricewine::discovery::{confident_executable, scan_executables};
use std::fs;

#[test]
fn detects_matching_program_without_selecting_uninstallers() {
    let root = std::env::temp_dir().join(format!("ricewine-discovery-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let base = root.join("drive_c/Program Files/Cool App");
    fs::create_dir_all(&base).unwrap();
    fs::write(base.join("CoolApp.exe"), b"test").unwrap();
    fs::write(base.join("unins000.exe"), b"test").unwrap();
    fs::write(base.join("helper.exe"), b"test").unwrap();
    let result = scan_executables(&root, "Cool App").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(
        confident_executable(&result),
        Some(base.join("CoolApp.exe"))
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn ambiguous_candidates_do_not_auto_launch() {
    let root = std::env::temp_dir().join(format!(
        "ricewine-discovery-ambiguity-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let base = root.join("drive_c/Program Files/App");
    fs::create_dir_all(&base).unwrap();
    fs::write(base.join("launcher1.exe"), b"test").unwrap();
    fs::write(base.join("launcher2.exe"), b"test").unwrap();
    let result = scan_executables(&root, "Something Else").unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(confident_executable(&result), None);
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn refuses_to_follow_prefix_symlinks() {
    let root = std::env::temp_dir().join(format!("ricewine-discovery-link-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("drive_c")).unwrap();
    std::os::unix::fs::symlink("/usr/bin", root.join("drive_c/outside")).unwrap();
    assert!(scan_executables(&root, "outside").unwrap().is_empty());
    fs::remove_dir_all(root).unwrap();
}
