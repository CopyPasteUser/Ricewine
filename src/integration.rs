//! XDG desktop integration and safe catalog reads for launch shortcuts.

use crate::core::{self, Application};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn application_directory() -> Result<PathBuf, String> {
    if let Some(xdg) = env::var_os("XDG_DATA_HOME") {
        let path = PathBuf::from(xdg);
        if path.is_absolute() {
            return Ok(path.join("applications"));
        }
    }
    env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".local/share/applications"))
        .ok_or("Cannot find a user-local application directory".into())
}

pub fn shortcut_file_name(id: &str) -> Result<String, String> {
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("Invalid application ID".into());
    }
    Ok(format!("ricewine-{id}.desktop"))
}

fn escape_exec(path: &Path) -> String {
    let text = path.to_string_lossy();
    let mut result = String::from("\"");
    for character in text.chars() {
        match character {
            // Desktop Entry Exec strings need escaping for both the quoted
            // argument and the desktop-file reader itself.
            '\\' => result.push_str("\\\\\\\\"),
            '"' => result.push_str("\\\\\""),
            '$' => result.push_str("\\\\$"),
            '`' => result.push_str("\\\\`"),
            _ => result.push(character),
        }
    }
    result.push('"');
    result
}

pub fn desktop_entry(app: &Application, ricewine_exe: &Path) -> Result<String, String> {
    core::validate_name(&app.name)?;
    shortcut_file_name(&app.id)?;
    if !ricewine_exe.is_absolute() {
        return Err("The RiceWine executable path must be absolute".into());
    }
    let safe_name = app.name.replace('\\', "\\\\");
    Ok(format!(
        "[Desktop Entry]\nType=Application\nVersion=1.0\nName={safe_name}\nComment=Run with RiceWine\nExec={} --launch {}\nTerminal=false\nStartupNotify=true\nCategories=Utility;\nIcon=wine\nX-RiceWine-ID={}\n",
        escape_exec(ricewine_exe),
        app.id,
        app.id
    ))
}

pub fn install_shortcut(app: &Application, ricewine_exe: &Path) -> Result<PathBuf, String> {
    let directory = application_directory()?;
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let target = directory.join(shortcut_file_name(&app.id)?);
    let stage = directory.join(format!(".ricewine-{}-{}.tmp", app.id, std::process::id()));
    let content = desktop_entry(app, ricewine_exe)?;
    let result = (|| -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&stage)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        fs::rename(&stage, &target)?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&stage);
        return Err(format!("Could not create application shortcut: {error}"));
    }
    Ok(target)
}

/// Only remove shortcuts carrying our matching custom ID, never an unrelated file.
pub fn remove_shortcut(app_id: &str) -> Result<(), String> {
    let path = application_directory()?.join(shortcut_file_name(app_id)?);
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.to_string()),
    };
    if !contents
        .lines()
        .any(|line| line == format!("X-RiceWine-ID={app_id}"))
    {
        return Err(format!(
            "Refusing to remove an unrecognized shortcut: {}",
            path.display()
        ));
    }
    fs::remove_file(&path).map_err(|e| e.to_string())
}
