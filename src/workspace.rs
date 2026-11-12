//! Persistent, user-local application workspace; deliberately independent of
//! the Wine catalog and per-prefix profiles. No GUI or Wine dependencies.

use crate::core::{self, Application};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const VERSION: u32 = 1;
const MAX_BYTES: u64 = 1024 * 1024;
const MAX_ENTRIES: usize = 10_000;
static SAVE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LibraryOrder {
    #[default]
    FavoritesFirst,
    Name,
    Recent,
}

impl LibraryOrder {
    pub const ALL: [Self; 3] = [Self::FavoritesFirst, Self::Name, Self::Recent];

    pub fn label(self) -> &'static str {
        match self {
            Self::FavoritesFirst => "Favorites first",
            Self::Name => "Name A–Z",
            Self::Recent => "Recently launched",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LibraryFilter {
    #[default]
    All,
    Favorites,
}

impl LibraryFilter {
    pub const ALL: [Self; 2] = [Self::All, Self::Favorites];

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All programs",
            Self::Favorites => "Favorites",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Workspace {
    version: u32,
    pub favorites: BTreeSet<String>,
    pub last_launched: BTreeMap<String, u64>,
    pub order: LibraryOrder,
    pub filter: LibraryFilter,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            version: VERSION,
            favorites: BTreeSet::new(),
            last_launched: BTreeMap::new(),
            order: LibraryOrder::default(),
            filter: LibraryFilter::default(),
        }
    }
}

impl Workspace {
    fn validate(&self) -> Result<(), String> {
        if self.version != VERSION {
            return Err(format!(
                "Unsupported workspace version {} (expected {VERSION}); no files changed",
                self.version
            ));
        }
        if self.favorites.len() > MAX_ENTRIES || self.last_launched.len() > MAX_ENTRIES {
            return Err("Workspace contains too many applications".into());
        }
        for id in self.favorites.iter().chain(self.last_launched.keys()) {
            core::validate_id(id)?;
        }
        Ok(())
    }

    pub fn is_favorite(&self, id: &str) -> bool {
        self.favorites.contains(id)
    }

    pub fn toggle_favorite(&mut self, id: &str) {
        if !self.favorites.remove(id) {
            self.favorites.insert(id.to_owned());
        }
    }

    /// Record only successfully spawned processes, never failed launch attempts.
    pub fn mark_launched(&mut self, id: &str) {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .min(u128::from(u64::MAX)) as u64;
        let previous = self.last_launched.values().copied().max().unwrap_or(0);
        // Two programs launched within one millisecond must still have a clear order.
        self.last_launched
            .insert(id.to_owned(), millis.max(previous.saturating_add(1)));
    }

    pub fn forget(&mut self, id: &str) {
        self.favorites.remove(id);
        self.last_launched.remove(id);
    }

    pub fn ordered_indices(&self, apps: &[Application], search: &str) -> Vec<usize> {
        let term = search.trim().to_lowercase();
        // Decorate-sort-undecorate: normalize each name ONCE rather than for
        // every comparison (important when scanning a large installed library).
        let mut entries: Vec<(usize, String, bool, u64)> = apps
            .iter()
            .enumerate()
            .filter_map(|(index, app)| {
                let pinned = self.is_favorite(&app.id);
                if self.filter == LibraryFilter::Favorites && !pinned {
                    return None;
                }
                let name = app.name.to_lowercase();
                if !term.is_empty() && !name.contains(&term) {
                    return None;
                }
                Some((
                    index,
                    name,
                    pinned,
                    self.last_launched.get(&app.id).copied().unwrap_or(0),
                ))
            })
            .collect();
        entries.sort_unstable_by(|a, b| {
            let favorites = || b.2.cmp(&a.2);
            let recent = || b.3.cmp(&a.3);
            let name = || a.1.cmp(&b.1).then_with(|| apps[a.0].id.cmp(&apps[b.0].id));
            match self.order {
                LibraryOrder::FavoritesFirst => favorites().then_with(name),
                LibraryOrder::Name => name(),
                LibraryOrder::Recent => recent().then_with(favorites).then_with(name),
            }
        });
        entries.into_iter().map(|entry| entry.0).collect()
    }
}

fn workspace_path(root: &Path) -> std::path::PathBuf {
    root.join("workspace.json")
}

pub fn load(root: &Path) -> Result<Workspace, String> {
    let path = workspace_path(root);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Workspace::default());
        }
        Err(error) => return Err(format!("Cannot inspect workspace: {error}")),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("Workspace path is not a regular file (refusing symlink)".into());
    }
    if metadata.len() > MAX_BYTES {
        return Err("Workspace exceeds its 1 MiB safety limit".into());
    }
    let bytes = fs::read(&path).map_err(|error| format!("Reading workspace failed: {error}"))?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("Workspace exceeds its 1 MiB safety limit".into());
    }
    let state: Workspace = serde_json::from_slice(&bytes)
        .map_err(|error| format!("Invalid workspace {}: {error}", path.display()))?;
    state.validate()?;
    Ok(state)
}

pub fn save(root: &Path, state: &Workspace) -> Result<(), String> {
    state.validate()?;
    if !root.is_dir() {
        return Err("RiceWine workspace data directory is missing".into());
    }
    let dest = workspace_path(root);
    match fs::symlink_metadata(&dest) {
        Ok(metadata) if !metadata.is_file() || metadata.file_type().is_symlink() => {
            return Err("Refusing to overwrite non-regular workspace file".into());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("Cannot inspect workspace destination: {error}")),
        _ => {}
    }
    let bytes = serde_json::to_vec_pretty(state).map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("Workspace exceeds its 1 MiB safety limit".into());
    }
    let temp = root.join(format!(
        ".workspace-{}-{}.tmp",
        std::process::id(),
        SAVE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| -> std::io::Result<()> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temp)?;
        file.write_all(&bytes)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        fs::rename(&temp, &dest)?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(temp);
        return Err(format!("Saving workspace failed: {error}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn app(id: &str, name: &str) -> Application {
        Application {
            id: id.into(),
            name: name.into(),
            prefix: "/tmp/unused".into(),
            executable: None,
            installer: None,
            portable: false,
        }
    }

    #[test]
    fn favorite_and_recent_orders_are_stable() {
        let apps = [app("b", "Zulu"), app("a", "alpha"), app("c", "Beta")];
        let mut state = Workspace::default();
        assert_eq!(state.ordered_indices(&apps, ""), vec![1, 2, 0]);
        state.toggle_favorite("b");
        assert_eq!(state.ordered_indices(&apps, ""), vec![0, 1, 2]);
        state.filter = LibraryFilter::Favorites;
        assert_eq!(state.ordered_indices(&apps, ""), vec![0]);
        state.filter = LibraryFilter::All;
        state.mark_launched("c");
        state.order = LibraryOrder::Recent;
        assert_eq!(state.ordered_indices(&apps, ""), vec![2, 0, 1]);
        state.mark_launched("a");
        // The previous launch timestamp is always strictly older.
        assert!(state.last_launched["a"] > state.last_launched["c"]);
        assert_eq!(state.ordered_indices(&apps, ""), vec![1, 2, 0]);
        assert_eq!(state.ordered_indices(&apps, "ALP"), vec![1]);
        state.forget("b");
        assert!(!state.is_favorite("b"));
    }

    #[test]
    fn workspace_io_is_atomic_and_malformed_file_is_preserved() {
        let root = std::env::temp_dir().join(format!(
            "ricewine-workspace-test-{}-{}",
            std::process::id(),
            SAVE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        assert_eq!(load(&root).unwrap(), Workspace::default());
        let mut state = Workspace::default();
        state.toggle_favorite("app-1");
        state.mark_launched("app-1");
        save(&root, &state).unwrap();
        assert_eq!(load(&root).unwrap(), state);
        let path = root.join("workspace.json");
        fs::write(&path, "not-json").unwrap();
        assert!(load(&root).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "not-json");
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn symlink_workspace_is_rejected() {
        use std::os::unix::fs::symlink;
        let root = std::env::temp_dir().join(format!(
            "ricewine-workspace-link-{}-{}",
            std::process::id(),
            SAVE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("outside"), "{}").unwrap();
        symlink(root.join("outside"), root.join("workspace.json")).unwrap();
        assert!(load(&root).is_err());
        assert!(save(&root, &Workspace::default()).is_err());
        assert_eq!(fs::read_to_string(root.join("outside")).unwrap(), "{}");
        fs::remove_dir_all(root).unwrap();
    }
}
