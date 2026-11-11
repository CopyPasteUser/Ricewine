use ricewine::updates::{self, parse_release, parse_repository};

#[test]
fn repository_validator_prevents_foreign_hosts_and_path_injection() {
    for invalid in [
        "",
        "../bad",
        "https://github.com/foo/bar",
        "foo/bar/baz",
        "foo/%2e",
        "foo/repo?x=1",
        "/repo",
    ] {
        assert!(parse_repository(invalid).is_err(), "accepted {invalid}");
    }
    assert_eq!(
        parse_repository("Example/repo-name").unwrap(),
        ("Example", "repo-name")
    );
}

#[test]
fn release_parser_keeps_only_publisher_assets() {
    let fixture = br#"{"tag_name":"v2.0","html_url":"https://github.com/example/tool/releases/tag/v2.0","assets":[{"name":"tool-v2.exe","browser_download_url":"https://github.com/example/tool/releases/download/v2.0/tool-v2.exe"},{"name":"bad.exe","browser_download_url":"https://evil.invalid/bad.exe"}]}"#;
    let release = parse_release(fixture, "example/tool").unwrap();
    assert_eq!(release.tag, "v2.0");
    assert_eq!(release.assets.len(), 1);
    assert!(parse_release(fixture, "attacker/tool").is_err());
    assert!(updates::managed_proton(std::path::Path::new("/missing/data")).is_empty());
}

#[test]
fn only_managed_runner_can_be_removed() {
    use std::fs;
    let root = std::env::temp_dir().join(format!("ricewine-ge-manager-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let runner = root.join("runners/GE-Proton-Test");
    let other = root.join("runners/External-Test");
    fs::create_dir_all(&runner).unwrap();
    fs::create_dir_all(&other).unwrap();
    fs::write(runner.join("proton"), b"fake executable").unwrap();
    fs::write(runner.join(".ricewine-managed"), b"owned").unwrap();
    assert!(updates::remove_managed_runner(&root, &other).is_err());
    assert!(other.exists());
    assert!(updates::remove_managed_runner(&root, &runner).is_ok());
    assert!(!runner.exists());
    assert!(other.exists());
    fs::remove_dir_all(root).unwrap();
}
