//! Non-destructive prefix snapshots. Restores always create a new application
//! and never replace a live prefix.

use crate::core::{self, Application};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_SNAPSHOT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Manifest {
    version: u32,
    created_unix: u64,
    application: Application,
}

#[derive(Clone, Debug)]
pub struct BackupEntry {
    pub folder: PathBuf,
    pub app_name: String,
    pub created_unix: u64,
}

fn backups_root(data_root: &Path) -> PathBuf {
    data_root.join("backups")
}

fn copy_tree(source: &Path, target: &Path) -> Result<(), String> {
    fs::create_dir(target).map_err(|e| format!("Creating {}: {e}", target.display()))?;
    let entries = fs::read_dir(source).map_err(|e| format!("Reading {}: {e}", source.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        let from = entry.path();
        let to = target.join(entry.file_name());
        if file_type.is_dir() {
            copy_tree(&from, &to)?;
        } else if file_type.is_file() {
            fs::copy(&from, &to).map_err(|e| format!("Copying {}: {e}", from.display()))?;
        } else if file_type.is_symlink() {
            #[cfg(unix)]
            {
                let destination = fs::read_link(&from).map_err(|e| e.to_string())?;
                std::os::unix::fs::symlink(&destination, &to)
                    .map_err(|e| format!("Copying symlink {}: {e}", from.display()))?;
            }
            #[cfg(not(unix))]
            return Err("Prefix symlinks require a Unix host".into());
        } else {
            return Err(format!(
                "Unsupported file type in prefix: {} (no snapshot created)",
                from.display()
            ));
        }
    }
    fs::set_permissions(
        target,
        fs::metadata(source)
            .map_err(|e| e.to_string())?
            .permissions(),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn create_backup(data_root: &Path, app: &Application) -> Result<BackupEntry, String> {
    if !app.prefix.is_dir() {
        return Err("Cannot back up a missing Wine prefix".into());
    }
    let directory = backups_root(data_root);
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let nonce = NEXT_SNAPSHOT.fetch_add(1, Ordering::Relaxed);
    let basename = format!("{}-{timestamp}-{}-{nonce}", app.id, std::process::id());
    let stage = directory.join(format!(".{basename}.partial"));
    let destination = directory.join(basename);
    fs::create_dir(&stage).map_err(|e| e.to_string())?;
    let result = (|| -> Result<(), String> {
        copy_tree(&app.prefix, &stage.join("prefix"))?;
        let manifest = Manifest {
            version: 1,
            created_unix: timestamp,
            application: app.clone(),
        };
        let mut file = File::create_new(stage.join("manifest.json")).map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(&mut file, &manifest).map_err(|e| e.to_string())?;
        file.write_all(b"\n").map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        fs::rename(&stage, &destination).map_err(|e| e.to_string())?;
        Ok(())
    })();
    if let Err(error) = result {
        // Only our own newly-created staging directory is cleaned up.
        let _ = fs::remove_dir_all(&stage);
        return Err(format!("Backup failed: {error}"));
    }
    Ok(BackupEntry {
        folder: destination,
        app_name: app.name.clone(),
        created_unix: timestamp,
    })
}

fn read_manifest(folder: &Path) -> Result<Manifest, String> {
    let file = folder.join("manifest.json");
    let contents = fs::read(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    let manifest: Manifest = serde_json::from_slice(&contents)
        .map_err(|e| format!("Invalid backup manifest {}: {e}", file.display()))?;
    if manifest.version != 1 {
        return Err(format!("Unsupported backup version {}", manifest.version));
    }
    core::validate_name(&manifest.application.name)?;
    Ok(manifest)
}

pub fn list_backups(data_root: &Path) -> Result<Vec<BackupEntry>, String> {
    let directory = backups_root(data_root);
    if !directory.is_dir() {
        return Ok(Vec::new());
    }
    let mut items = Vec::new();
    for item in fs::read_dir(&directory).map_err(|e| e.to_string())? {
        let item = item.map_err(|e| e.to_string())?;
        if !item.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        if let Ok(manifest) = read_manifest(&item.path()) {
            items.push(BackupEntry {
                folder: item.path(),
                app_name: manifest.application.name,
                created_unix: manifest.created_unix,
            });
        }
    }
    items.sort_unstable_by(|a, b| {
        b.created_unix
            .cmp(&a.created_unix)
            .then_with(|| b.folder.cmp(&a.folder))
    });
    Ok(items)
}

pub fn restore_backup(data_root: &Path, folder: &Path) -> Result<Application, String> {
    // Only application-managed snapshots may be restored; no arbitrary
    // user-controlled path traversal into the filesystem.
    let trusted_root = fs::canonicalize(backups_root(data_root))
        .map_err(|e| format!("Backup directory is unavailable: {e}"))?;
    let folder = fs::canonicalize(folder).map_err(|e| e.to_string())?;
    if folder.parent() != Some(trusted_root.as_path()) {
        return Err("Choose a RiceWine-managed snapshot under the backups directory".into());
    }
    let manifest = read_manifest(&folder)?;
    let source_prefix = folder.join("prefix");
    if !source_prefix.is_dir() {
        return Err("Backup prefix data is missing".into());
    }
    let id = core::new_id(&manifest.application.name);
    let new_prefix = data_root.join("prefixes").join(&id);
    let result = copy_tree(&source_prefix, &new_prefix);
    if let Err(error) = result {
        let _ = fs::remove_dir_all(&new_prefix);
        return Err(format!("Restore failed: {error}"));
    }
    let mut app = manifest.application;
    if let Some(old_exe) = &app.executable {
        if let Ok(relative) = old_exe.strip_prefix(&app.prefix) {
            app.executable = Some(new_prefix.join(relative));
        } else if !old_exe.is_file() {
            app.executable = None;
        }
    }
    app.id = id;
    app.name = format!("{} (restored)", app.name);
    // App names are limited to 80 characters, including the suffix.
    if app.name.chars().count() > 80 {
        app.name = app.name.chars().take(80).collect();
    }
    app.prefix = new_prefix;
    Ok(app)
}
