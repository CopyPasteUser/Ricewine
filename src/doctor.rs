//! Read-only preflight for existing library entries and their Wine runners.
//! This module never launches executables, writes profiles, or repairs prefixes.

use crate::core::Application;
use crate::runtime::{self, Runner};
use std::path::Path;

const MAX_VISIBLE_ISSUES: usize = 80;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Warning => "WARN",
            Self::Error => "ERROR",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub severity: Severity,
    pub program: String,
    pub code: &'static str,
    pub description: String,
    pub suggestion: &'static str,
}

#[derive(Clone, Debug, Default)]
pub struct DoctorReport {
    pub programs_checked: usize,
    pub findings: Vec<Finding>,
    pub omitted: usize,
}

impl DoctorReport {
    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(|item| item.severity == Severity::Error)
    }

    pub fn readable(&self) -> String {
        let mut result = format!(
            "RiceWine health check — {} program(s) checked, {} finding(s)\n",
            self.programs_checked,
            self.findings.len() + self.omitted
        );
        if self.findings.is_empty() && self.omitted == 0 {
            result.push_str("All checked applications look ready. This is not a guarantee of Wine compatibility.\n");
        }
        for item in &self.findings {
            result.push_str(&format!(
                "\n[{}] {} / {}: {}\nNext step: {}\n",
                item.severity.label(),
                item.program,
                item.code,
                item.description,
                item.suggestion
            ));
        }
        if self.omitted > 0 {
            result.push_str(&format!(
                "\n{} additional findings omitted from this bounded report.\n",
                self.omitted
            ));
        }
        result.push_str("\nRead-only inspection. No Windows application or installer was launched; no files were changed.");
        result
    }
}

fn append(report: &mut DoctorReport, item: Finding) {
    if report.findings.len() < MAX_VISIBLE_ISSUES {
        report.findings.push(item);
    } else {
        report.omitted += 1;
    }
}

fn issue(
    report: &mut DoctorReport,
    severity: Severity,
    app: &str,
    code: &'static str,
    description: String,
    suggestion: &'static str,
) {
    append(
        report,
        Finding {
            severity,
            program: app.to_owned(),
            code,
            description,
            suggestion,
        },
    );
}

pub fn check(apps: &[Application], wine_available: bool) -> DoctorReport {
    let mut report = DoctorReport {
        programs_checked: apps.len(),
        ..Default::default()
    };
    let mut needs_system_wine = false;
    for app in apps {
        let name = app.name.as_str();
        if !app.prefix.is_dir() {
            issue(
                &mut report,
                Severity::Error,
                name,
                "PREFIX_MISSING",
                format!("Missing Wine environment: {}", app.prefix.display()),
                "Restore the prefix from a backup or re-import an existing installation.",
            );
        } else if !app.prefix.join("drive_c").is_dir()
            || !app.prefix.join("system.reg").is_file()
        {
            issue(
                &mut report,
                Severity::Warning,
                name,
                "PREFIX_INCOMPLETE",
                format!("Environment is not fully initialized: {}", app.prefix.display()),
                "Verify the existing prefix or restore a known-good backup before starting this app.",
            );
        }
        match &app.executable {
            None => issue(
                &mut report,
                Severity::Warning,
                name,
                "LAUNCH_EXE_UNSET",
                "No executable selected for launching.".into(),
                "Open this program in Library and use Find EXE automatically or Choose EXE manually.",
            ),
            Some(path) if !path.is_file() => issue(
                &mut report,
                Severity::Error,
                name,
                "LAUNCH_EXE_MISSING",
                format!("Launch executable is unavailable: {}", path.display()),
                "Check the drive/mount and select a valid EXE from the program's Library details.",
            ),
            Some(path) if !has_exe_extension(path) => issue(
                &mut report,
                Severity::Warning,
                name,
                "LAUNCH_EXE_TYPE",
                format!("Launch target does not have an EXE extension: {}", path.display()),
                "Review and choose the program's actual .exe launch file.",
            ),
            _ => {}
        }

        match runtime::load_profile(&app.prefix) {
            Err(error) => issue(
                &mut report,
                Severity::Error,
                name,
                "PROFILE_INVALID",
                error,
                "Review this prefix's .ricewine.toml with a backup; automatic repair is disabled.",
            ),
            Ok(profile) => {
                needs_system_wine |= matches!(profile.runner, Runner::SystemWine);
                // Proton and custom Wine are verified independently of system Wine.
                if let Err(error) = runtime::validate_profile(&profile) {
                    issue(
                        &mut report,
                        Severity::Error,
                        name,
                        "RUNNER_INVALID",
                        error,
                        "Open Wine & Proton / runner settings and choose an available runner; keep a prefix backup.",
                    );
                }
            }
        }
    }
    if !wine_available {
        issue(
            &mut report,
            if needs_system_wine {
                Severity::Error
            } else {
                Severity::Warning
            },
            "System",
            "SYSTEM_WINE_MISSING",
            "System Wine and wineboot are not both available.".into(),
            if needs_system_wine {
                "Install working Wine/wineboot or explicitly select another compatible runner for affected programs."
            } else {
                "System-Wine installations require Wine/wineboot; existing Proton apps can still use their own runner."
            },
        );
    }
    report
}

fn has_exe_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Temp(std::path::PathBuf);

    impl Temp {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "ricewine-doctor-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn healthy_app(&self) -> Application {
            fs::create_dir(self.0.join("drive_c")).unwrap();
            fs::write(self.0.join("system.reg"), "").unwrap();
            let exe = self.0.join("program.exe");
            fs::write(&exe, "").unwrap();
            Application {
                id: "app-1".into(),
                name: "Example".into(),
                prefix: self.0.clone(),
                executable: Some(exe),
                installer: None,
                portable: false,
            }
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn healthy_environment_has_no_findings() {
        let temp = Temp::new();
        let app = temp.healthy_app();
        let report = check(&[app], true);
        assert_eq!(report.programs_checked, 1);
        assert!(report.findings.is_empty());
        assert!(!report.has_errors());
    }

    #[test]
    fn missing_exe_and_bad_profile_have_actionable_diagnostics() {
        let temp = Temp::new();
        let mut app = temp.healthy_app();
        app.executable = None;
        fs::write(temp.0.join(".ricewine.toml"), "not toml [").unwrap();
        let report = check(&[app], false);
        assert!(report.findings.iter().any(|f| f.code == "LAUNCH_EXE_UNSET"));
        assert!(report.findings.iter().any(|f| f.code == "PROFILE_INVALID"));
        assert!(report.findings.iter().any(|f| f.code == "SYSTEM_WINE_MISSING"));
        assert!(report.has_errors());
        assert!(report.readable().contains("Next step"));
        assert_eq!(fs::read_to_string(temp.0.join(".ricewine.toml")).unwrap(), "not toml [");
    }

    #[test]
    fn missing_prefix_does_not_get_repaired() {
        let temp = Temp::new();
        let app = Application {
            id: "app-2".into(),
            name: "Lost".into(),
            prefix: temp.0.join("does-not-exist"),
            executable: None,
            installer: None,
            portable: false,
        };
        let report = check(&[app], true);
        assert!(report.findings.iter().any(|f| f.code == "PREFIX_MISSING"));
        assert!(!temp.0.join("does-not-exist").exists());
    }
}
