//! Bounded, read-only executable discovery inside a Wine prefix.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_ENTRIES: usize = 20_000;
const MAX_DEPTH: usize = 7;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchCandidate {
    pub path: PathBuf,
    pub score: i32,
}

fn normalized(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

pub fn scan_executables(
    prefix: &Path,
    application_name: &str,
) -> Result<Vec<LaunchCandidate>, String> {
    let drive = prefix.join("drive_c");
    if !drive.is_dir() {
        return Ok(Vec::new());
    }
    let mut pending = vec![(drive, 0)];
    let mut seen = HashSet::new();
    let mut results = Vec::new();
    let mut visited = 0;
    let wanted = normalized(application_name);
    while let Some((dir, depth)) = pending.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => continue,
            Err(error) => return Err(format!("Scanning {} failed: {error}", dir.display())),
        };
        for entry in entries {
            visited += 1;
            if visited > MAX_ENTRIES {
                return Err(
                    "Executable scan reached its 20,000-entry safety limit; choose EXE manually"
                        .into(),
                );
            }
            let entry = entry.map_err(|error| error.to_string())?;
            let file_type = entry.file_type().map_err(|error| error.to_string())?;
            if file_type.is_symlink() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            let lower = name.to_ascii_lowercase();
            let path = entry.path();
            if file_type.is_dir() {
                if depth < MAX_DEPTH
                    && !matches!(
                        lower.as_str(),
                        "windows"
                            | "winsxs"
                            | "system32"
                            | "syswow64"
                            | "temp"
                            | "cache"
                            | "redist"
                            | "redists"
                    )
                {
                    pending.push((path, depth + 1));
                }
                continue;
            }
            if !file_type.is_file() || !lower.ends_with(".exe") || !seen.insert(path.clone()) {
                continue;
            }
            let stem = lower.trim_end_matches(".exe");
            if [
                "unins",
                "setup",
                "install",
                "uninstall",
                "unwise",
                "vcredist",
                "dxsetup",
                "crash",
                "updater",
                "update",
                "repair",
                "wineboot",
                "rundll",
                "regsvr",
                "helper",
            ]
            .iter()
            .any(|value| stem.starts_with(value))
            {
                continue;
            }
            let current = normalized(stem);
            let mut score = 10;
            if !wanted.is_empty() {
                if current == wanted {
                    score += 120;
                } else if current.contains(&wanted)
                    || wanted.contains(&current) && current.len() >= 5
                {
                    score += 85;
                } else if current.len() >= 4
                    && wanted.len() >= 4
                    && current.chars().take(4).eq(wanted.chars().take(4))
                {
                    score += 32;
                }
            }
            if path.components().any(|part| {
                part.as_os_str()
                    .to_string_lossy()
                    .eq_ignore_ascii_case("Program Files")
            }) {
                score += 10;
            }
            if path.components().any(|part| {
                part.as_os_str()
                    .to_string_lossy()
                    .eq_ignore_ascii_case("Program Files (x86)")
            }) {
                score += 10;
            }
            if stem.contains("launcher") {
                score -= 8;
            }
            results.push(LaunchCandidate { path, score });
        }
    }
    results.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.path.cmp(&b.path)));
    Ok(results)
}

/// Auto-select only when the result is unambiguous. The UI can display the
/// remaining candidates, allowing the user to override the chosen executable.
pub fn confident_executable(candidates: &[LaunchCandidate]) -> Option<PathBuf> {
    let first = candidates.first()?;
    if candidates.len() == 1 || (first.score >= 85 && first.score - candidates[1].score >= 25) {
        Some(first.path.clone())
    } else {
        None
    }
}
