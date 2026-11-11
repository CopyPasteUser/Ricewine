//! Opt-in GitHub release checks and checksum-verified GE-Proton provisioning.
//! Network operations are launched only by explicit user actions, never on startup.

use crate::core::{self, OperationControl};
use serde_json::Value;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

const GE_REPOSITORY: &str = "GloriousEggroll/proton-ge-custom";
static STAGING_SERIAL: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub struct Release {
    pub tag: String,
    pub url: String,
    pub assets: Vec<Asset>,
}

#[derive(Clone, Debug)]
pub struct Asset {
    pub name: String,
    pub url: String,
}

pub fn parse_repository(value: &str) -> Result<(&str, &str), String> {
    let (owner, repo) = value
        .trim()
        .split_once('/')
        .ok_or("Use a GitHub repository as owner/repository")?;
    if owner.is_empty()
        || repo.is_empty()
        || owner.len() > 39
        || repo.len() > 100
        || !owner
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        || !repo
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(
            "GitHub repository must contain a valid owner/repository, without URLs or extra paths"
                .into(),
        );
    }
    Ok((owner, repo))
}

pub fn parse_release(body: &[u8], repository: &str) -> Result<Release, String> {
    let value: Value =
        serde_json::from_slice(body).map_err(|e| format!("Invalid release response: {e}"))?;
    let tag = value["tag_name"]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= 128)
        .ok_or("No published GitHub release was found for this repository")?;
    let url = value["html_url"].as_str().ok_or("Release URL missing")?;
    let prefix = format!("https://github.com/{repository}/releases/tag/");
    if !url.starts_with(&prefix) {
        return Err("Release URL is outside the selected GitHub repository".into());
    }
    let assets = value["assets"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let name = item["name"].as_str()?;
                    let url = item["browser_download_url"].as_str()?;
                    let asset_prefix =
                        format!("https://github.com/{repository}/releases/download/");
                    if !url.starts_with(&asset_prefix) || !safe_asset_name(name) {
                        return None;
                    }
                    Some(Asset {
                        name: name.into(),
                        url: url.into(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(Release {
        tag: tag.into(),
        url: url.into(),
        assets,
    })
}

fn safe_asset_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() < 180
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        && !name.starts_with('.')
}

fn curl_bytes(url: &str) -> Result<Vec<u8>, String> {
    let result = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--max-time",
            "20",
            "--max-filesize",
            "2000000",
            "--user-agent",
            "RiceWine/2.0",
            url,
        ])
        .output()
        .map_err(|e| format!("curl is needed to check releases: {e}"))?;
    if !result.status.success() {
        return Err(format!(
            "Could not retrieve GitHub release (curl {})",
            result.status
        ));
    }
    Ok(result.stdout)
}

pub fn latest_release(repository: &str) -> Result<Release, String> {
    let (owner, repo) = parse_repository(repository)?;
    let source = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");
    parse_release(&curl_bytes(&source)?, repository)
}

pub fn latest_ge_proton() -> Result<Release, String> {
    latest_release(GE_REPOSITORY)
}

fn asset_pair(release: &Release, architecture: &str) -> Result<(Asset, Asset), String> {
    let suffix = format!("-{architecture}.tar.gz");
    let tarball = release
        .assets
        .iter()
        .find(|asset| asset.name.ends_with(&suffix))
        .ok_or(format!(
            "No GE-Proton tarball for {architecture} in this release"
        ))?;
    let checksums = release
        .assets
        .iter()
        .find(|asset| asset.name == format!("{}.sha512sum", tarball.name))
        .or_else(|| {
            release.assets.iter().find(|asset| {
                asset.name.ends_with(".sha512sum") && asset.name.contains(architecture)
            })
        })
        .ok_or("Release contains no matching SHA512 checksum; refusing unverified download")?;
    Ok((tarball.clone(), checksums.clone()))
}

pub fn install_ge_proton(
    data_root: &Path,
    release: &Release,
    control: &OperationControl,
) -> Result<PathBuf, String> {
    if control.is_cancelled() {
        return Err("GE-Proton download cancelled".into());
    }
    let architecture = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => return Err("GE-Proton is only supported on x86_64/aarch64".into()),
    };
    let (tarball, checksums) = asset_pair(release, architecture)?;
    if !release
        .url
        .starts_with("https://github.com/GloriousEggroll/proton-ge-custom/releases/tag/")
    {
        return Err("Not an official GE-Proton release".into());
    }
    let version = tarball.name.trim_end_matches(".tar.gz");
    if !safe_asset_name(version) {
        return Err("Unsafe GE-Proton release name".into());
    }
    let destination_root = data_root.join("runners");
    fs::create_dir_all(&destination_root).map_err(|e| e.to_string())?;
    core::private_permissions(&destination_root)?;
    let destination = destination_root.join(version);
    if destination.exists() {
        return Err(format!(
            "{version} is already installed; existing files were not changed"
        ));
    }
    let stage = destination_root.join(format!(
        ".ricewine-{}-{}.partial",
        std::process::id(),
        STAGING_SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&stage).map_err(|e| e.to_string())?;
    core::private_permissions(&stage)?;
    let result = (|| -> Result<PathBuf, String> {
        let archive = stage.join(&tarball.name);
        download(&tarball.url, &archive, control)?;
        if control.is_cancelled() {
            return Err("GE-Proton download cancelled".into());
        }
        let checks = curl_bytes(&checksums.url)?;
        let expected_hash = checksum_for(&checks, &tarball.name)?;
        let actual = Command::new("sha512sum")
            .arg(&archive)
            .output()
            .map_err(|e| format!("sha512sum is needed for download verification: {e}"))?;
        if !actual.status.success()
            || !String::from_utf8_lossy(&actual.stdout).starts_with(&expected_hash)
        {
            return Err(
                "Downloaded GE-Proton tarball does not match the publisher SHA512 checksum".into(),
            );
        }
        let listing = Command::new("tar")
            .arg("-tzf")
            .arg(&archive)
            .output()
            .map_err(|e| format!("tar is needed to extract GE-Proton: {e}"))?;
        if !listing.status.success() {
            return Err("GE-Proton tarball failed archive inspection".into());
        }
        validate_archive_listing(&listing.stdout)?;
        if control.is_cancelled() {
            return Err("GE-Proton download cancelled".into());
        }
        let extract = stage.join("extracted");
        fs::create_dir(&extract).map_err(|e| e.to_string())?;
        let mut unpack = Command::new("tar");
        unpack
            .args(["-xzf"])
            .arg(&archive)
            .args(["-C"])
            .arg(&extract)
            .args([
                "--no-same-owner",
                "--no-same-permissions",
                "--strip-components=1",
            ]);
        let status = run_controllable(&mut unpack, control)?;
        if !status.success() {
            return Err(format!("Could not extract GE-Proton ({status})"));
        }
        let proton = fs::canonicalize(extract.join("proton"))
            .map_err(|_| "Extracted release has no Proton launcher")?;
        if !proton.starts_with(fs::canonicalize(&extract).map_err(|e| e.to_string())?) {
            return Err("Extracted Proton launcher escaped the staging directory".into());
        }
        if control.is_cancelled() {
            return Err("GE-Proton download cancelled".into());
        }
        fs::write(
            extract.join(".ricewine-managed"),
            b"RiceWine managed GE-Proton\n",
        )
        .map_err(|e| e.to_string())?;
        fs::rename(&extract, &destination)
            .map_err(|e| format!("Publishing verified runner: {e}"))?;
        Ok(destination.clone())
    })();
    let _ = fs::remove_dir_all(stage);
    result
}

fn download(url: &str, file: &Path, control: &OperationControl) -> Result<(), String> {
    let mut command = Command::new("curl");
    command
        .args([
            "--fail",
            "--show-error",
            "--silent",
            "--location",
            "--connect-timeout",
            "15",
            "--max-time",
            "1800",
            "--max-filesize",
            "2147483648",
            "--output",
        ])
        .arg(file)
        .arg(url);
    let status = run_controllable(&mut command, control)?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("GE-Proton download failed (curl {status})"))
    }
}

fn run_controllable(
    command: &mut Command,
    control: &OperationControl,
) -> Result<std::process::ExitStatus, String> {
    if control.is_cancelled() {
        return Err("GE-Proton operation cancelled".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            return Ok(status);
        }
        if control.is_cancelled() {
            core::terminate_child(&mut child);
            let _ = child.wait();
            return Err(
                "GE-Proton download cancelled; incomplete staging files were removed".into(),
            );
        }
        thread::sleep(Duration::from_millis(100));
    }
}

pub fn is_managed_runner(data_root: &Path, path: &Path) -> bool {
    let Ok(root) = fs::canonicalize(data_root.join("runners")) else {
        return false;
    };
    let Ok(meta) = fs::symlink_metadata(path) else {
        return false;
    };
    if !meta.is_dir() {
        return false;
    }
    let Ok(path) = fs::canonicalize(path) else {
        return false;
    };
    path.parent() == Some(root.as_path()) && path.join(".ricewine-managed").is_file()
}

/// Destructive only after the caller has shown a confirmation and checked that
/// no application profile references this exact managed runner.
pub fn remove_managed_runner(data_root: &Path, path: &Path) -> Result<(), String> {
    if !is_managed_runner(data_root, path) {
        return Err("Not a RiceWine-managed runner; refusing removal".into());
    }
    fs::remove_dir_all(path).map_err(|e| format!("Removing managed runner: {e}"))
}

fn checksum_for(contents: &[u8], filename: &str) -> Result<String, String> {
    let string = std::str::from_utf8(contents).map_err(|e| e.to_string())?;
    for line in string.lines() {
        let mut parts = line.split_whitespace();
        let hash = parts.next().unwrap_or("");
        let name = parts.next().unwrap_or("").trim_start_matches('*');
        if name == filename && hash.len() == 128 && hash.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Ok(hash.to_ascii_lowercase());
        }
    }
    Err("GE-Proton checksum file does not identify the downloaded tarball".into())
}

fn validate_archive_listing(listing: &[u8]) -> Result<(), String> {
    let listing = std::str::from_utf8(listing).map_err(|e| e.to_string())?;
    let mut top: Option<&str> = None;
    for line in listing.lines() {
        let path = Path::new(line);
        if path.is_absolute()
            || path
                .components()
                .any(|part| !matches!(part, Component::Normal(_) | Component::CurDir))
        {
            return Err("GE-Proton archive contains an unsafe path".into());
        }
        let first = line.split('/').next().unwrap_or("");
        if first.is_empty() || first == "." || top.is_some_and(|name| name != first) {
            return Err(
                "GE-Proton archive must contain exactly one top-level release directory".into(),
            );
        }
        top = Some(first);
    }
    if top.is_none() {
        return Err("GE-Proton archive is empty".into());
    }
    Ok(())
}

pub fn managed_proton(data_root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(data_root.join("runners")) else {
        return Vec::new();
    };
    let mut results = entries
        .flatten()
        .take(200)
        .filter_map(|entry| {
            let path = entry.path();
            if !entry.file_type().ok()?.is_dir() || !path.join("proton").is_file() {
                return None;
            }
            Some(path)
        })
        .collect::<Vec<_>>();
    results.sort();
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_listing_rejects_traversal_and_multiple_roots() {
        assert!(
            validate_archive_listing(b"GE-Proton42/proton\nGE-Proton42/files/bin/wine\n").is_ok()
        );
        for unsafe_listing in [
            b"../../etc/passwd\n".as_slice(),
            b"/usr/lib/malicious\n".as_slice(),
            b"GE-Proton42/proton\nOther/bin/wine\n".as_slice(),
        ] {
            assert!(validate_archive_listing(unsafe_listing).is_err());
        }
    }

    #[test]
    fn checksum_requires_exact_filename_and_sha512_digest() {
        let hash = "a".repeat(128);
        let content = format!("{hash}  GE-Proton42-x86_64.tar.gz\n");
        assert_eq!(
            checksum_for(content.as_bytes(), "GE-Proton42-x86_64.tar.gz").unwrap(),
            hash
        );
        assert!(checksum_for(content.as_bytes(), "other.tar.gz").is_err());
    }
}
