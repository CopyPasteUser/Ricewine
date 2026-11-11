//! Read-only disk usage and conservative, explicitly confirmed Wine temp cleanup.
//! Never deletes a prefix, an executable, or any directory tree.

use crate::core::Application;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const MAX_SCAN_ENTRIES: usize = 200_000;
const MAX_CLEANUP_FILES: usize = 10_000;
const TEMP_MIN_AGE: Duration = Duration::from_secs(7 * 24 * 3600);

#[derive(Clone, Debug, Default)]
pub struct DiskUsage {
    pub bytes: u64,
    pub files: usize,
}

#[derive(Clone, Debug, Default)]
pub struct CleanupPreview {
    pub files: Vec<PathBuf>,
    pub bytes: u64,
}

pub fn disk_usage(path: &Path) -> Result<DiskUsage, String> {
    let mut result = DiskUsage::default();
    let mut pending = vec![path.to_path_buf()];
    let mut visited = 0;
    while let Some(directory) = pending.pop() {
        for entry in
            fs::read_dir(&directory).map_err(|e| format!("{}: {e}", directory.display()))?
        {
            visited += 1;
            if visited > MAX_SCAN_ENTRIES {
                return Err("Disk usage scan exceeded 200,000 entries".into());
            }
            let entry = entry.map_err(|e| e.to_string())?;
            let file_type = entry.file_type().map_err(|e| e.to_string())?;
            if file_type.is_dir() {
                pending.push(entry.path());
            } else if file_type.is_file() {
                result.files += 1;
                result.bytes += entry.metadata().map_err(|e| e.to_string())?.len();
            }
            // Do not count or dereference symlinks and special files.
        }
    }
    Ok(result)
}

fn managed_prefix(data_root: &Path, app: &Application) -> Result<PathBuf, String> {
    let root = fs::canonicalize(data_root.join("prefixes")).map_err(|e| e.to_string())?;
    let prefix = fs::canonicalize(&app.prefix).map_err(|e| e.to_string())?;
    if !prefix.starts_with(&root) || prefix == root {
        return Err("Cleanup is only available for RiceWine-managed prefixes; imported external prefixes are preserved".into());
    }
    Ok(prefix)
}

fn temp_folders(prefix: &Path) -> Result<Vec<PathBuf>, String> {
    let mut folders = vec![prefix.join("drive_c/windows/temp")];
    let users = prefix.join("drive_c/users");
    if users.is_dir() {
        for entry in fs::read_dir(users).map_err(|e| e.to_string())?.take(200) {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
                folders.push(entry.path().join("Temp"));
            }
        }
    }
    Ok(folders)
}

pub fn preview_cleanup(data_root: &Path, app: &Application) -> Result<CleanupPreview, String> {
    let prefix = managed_prefix(data_root, app)?;
    let mut preview = CleanupPreview::default();
    let now = SystemTime::now();
    for folder in temp_folders(&prefix)? {
        if !folder.exists() {
            continue;
        }
        let canonical = fs::canonicalize(&folder).map_err(|e| e.to_string())?;
        if !canonical.starts_with(&prefix) {
            continue;
        }
        let mut pending = vec![folder];
        while let Some(directory) = pending.pop() {
            for entry in fs::read_dir(&directory).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let file_type = entry.file_type().map_err(|e| e.to_string())?;
                if file_type.is_symlink() {
                    continue;
                }
                if file_type.is_dir() {
                    if fs::canonicalize(entry.path()).is_ok_and(|path| path.starts_with(&canonical))
                    {
                        pending.push(entry.path());
                    }
                    continue;
                }
                if !file_type.is_file() {
                    continue;
                }
                if preview.files.len() >= MAX_CLEANUP_FILES {
                    return Err("Cleanup scan reached 10,000 files; no files were removed".into());
                }
                let meta = entry.metadata().map_err(|e| e.to_string())?;
                let old_enough = meta
                    .modified()
                    .ok()
                    .and_then(|then| now.duration_since(then).ok())
                    .is_some_and(|age| age >= TEMP_MIN_AGE);
                if old_enough {
                    preview.bytes += meta.len();
                    preview.files.push(entry.path());
                }
            }
        }
    }
    preview.files.sort();
    preview.files.dedup();
    Ok(preview)
}

/// Revalidates every file at deletion time; never follows links or removes dirs.
pub fn cleanup_old_temp(data_root: &Path, app: &Application) -> Result<DiskUsage, String> {
    let preview = preview_cleanup(data_root, app)?;
    let prefix = managed_prefix(data_root, app)?;
    let mut result = DiskUsage::default();
    for file in preview.files {
        let meta = fs::symlink_metadata(&file).map_err(|e| e.to_string())?;
        if !meta.is_file() {
            continue;
        }
        let canonical = fs::canonicalize(&file).map_err(|e| e.to_string())?;
        if !canonical.starts_with(&prefix) {
            return Err("Cleanup target escaped the managed prefix".into());
        }
        fs::remove_file(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        result.files += 1;
        result.bytes += meta.len();
    }
    Ok(result)
}

pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}
