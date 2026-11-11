//! Read-only library diagnostics and explicitly requested metadata snapshots.
//! A metadata snapshot is intentionally distinct from a full Wine prefix backup.

use crate::core::{self, Application};
use crate::runtime;
use serde::Serialize;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static SNAPSHOT_NUMBER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Default)]
pub struct HealthReport {
    pub checked: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl HealthReport {
    pub fn is_healthy(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn lines(&self) -> Vec<String> {
        let mut lines = vec![format!(
            "Checked {} application(s): {} error(s), {} warning(s)",
            self.checked,
            self.errors.len(),
            self.warnings.len()
        )];
        lines.extend(
            self.errors
                .iter()
                .map(|message| format!("ERROR: {message}")),
        );
        lines.extend(
            self.warnings
                .iter()
                .map(|message| format!("WARN: {message}")),
        );
        lines
    }
}

/// Inspects registered paths and profiles; never launches Wine or changes data.
pub fn check(apps: &[Application]) -> HealthReport {
    let mut report = HealthReport {
        checked: apps.len(),
        ..Default::default()
    };
    for app in apps {
        if !app.prefix.is_dir() {
            report.errors.push(format!(
                "{} ({}): prefix is missing ({})",
                app.name,
                app.id,
                app.prefix.display()
            ));
            continue;
        }
        if let Some(exe) = &app.executable {
            if !exe.is_file() {
                report.warnings.push(format!(
                    "{} ({}): launch executable is missing ({})",
                    app.name,
                    app.id,
                    exe.display()
                ));
            }
        } else {
            report.warnings.push(format!(
                "{} ({}): no launch executable chosen",
                app.name, app.id
            ));
        }
        match runtime::load_profile(&app.prefix) {
            Ok(profile) => {
                if let Err(error) = runtime::validate_launch_options(&profile) {
                    report.errors.push(format!("{}: {error}", app.name));
                }
                if let Err(error) = runtime::validate_runner(&profile.runner) {
                    report.warnings.push(format!("{}: {error}", app.name));
                }
            }
            Err(error) => report.errors.push(format!("{}: {error}", app.name)),
        }
    }
    report
}

#[derive(Serialize)]
struct SnapshotManifest<'a> {
    format_version: u32,
    created_unix: u64,
    scope: &'a str,
    files: Vec<&'a str>,
}

/// Creates a recoverable copy of the small library metadata files.
/// Wine prefixes, logs, and existing snapshots are deliberately not included.
pub fn snapshot_metadata(root: &Path) -> Result<PathBuf, String> {
    let catalog = root.join("applications.json");
    if !catalog.is_file() {
        return Err("No saved application catalog to snapshot".into());
    }
    // Validate the catalog and its bounds before capturing it.
    core::Store::read_only_at(root)?;
    let preferences = root.join("preferences.json");
    let mut files = vec![(catalog, "applications.json")];
    if preferences.is_file() {
        if fs::metadata(&preferences).map_err(|e| e.to_string())?.len() > 64 * 1024 {
            return Err("Preferences file is unexpectedly large".into());
        }
        files.push((preferences, "preferences.json"));
    }
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let name = format!(
        "{timestamp}-{}-{}",
        std::process::id(),
        SNAPSHOT_NUMBER.fetch_add(1, Ordering::Relaxed)
    );
    let snapshots = root.join("backups/library");
    fs::create_dir_all(&snapshots).map_err(|e| e.to_string())?;
    core::private_permissions(&snapshots)?;
    let stage = snapshots.join(format!(".{name}.partial"));
    let destination = snapshots.join(name);
    fs::create_dir(&stage).map_err(|e| e.to_string())?;
    core::private_permissions(&stage)?;
    let outcome = (|| -> Result<(), String> {
        for (source, filename) in &files {
            // Never overwrite a file in another snapshot; staging is unique.
            if fs::metadata(source).map_err(|e| e.to_string())?.len() > 4 * 1024 * 1024 {
                return Err(format!("File changed size during snapshot: {filename}"));
            }
            let mut to = File::create_new(stage.join(filename)).map_err(|e| e.to_string())?;
            let bytes = fs::read(source).map_err(|e| e.to_string())?;
            to.write_all(&bytes).map_err(|e| e.to_string())?;
            to.sync_all().map_err(|e| e.to_string())?;
        }
        // A concurrently replaced catalog must still parse in the snapshot.
        core::Store::read_only_at(&stage)?;
        let manifest = SnapshotManifest {
            format_version: 1,
            created_unix: timestamp,
            scope: "metadata_only",
            files: files.iter().map(|(_, filename)| *filename).collect(),
        };
        let mut to = File::create_new(stage.join("manifest.json")).map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(&mut to, &manifest).map_err(|e| e.to_string())?;
        to.write_all(b"\n").map_err(|e| e.to_string())?;
        to.sync_all().map_err(|e| e.to_string())?;
        fs::rename(&stage, &destination).map_err(|e| e.to_string())?;
        Ok(())
    })();
    if let Err(error) = outcome {
        let _ = fs::remove_dir_all(&stage);
        return Err(format!("Metadata snapshot failed: {error}"));
    }
    Ok(destination)
}
