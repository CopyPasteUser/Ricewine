#![cfg(unix)]

use ricewine::ipc::{self, Server};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static SERIAL: AtomicU64 = AtomicU64::new(0);

struct Temp(PathBuf);

impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ricewine-ipc-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn forwards_open_with_without_starting_another_instance() {
    use std::os::unix::fs::PermissionsExt;
    let temp = Temp::new();
    let file = temp.0.join("Install a game.EXE");
    fs::write(&file, b"fake installer").unwrap();
    let no_server = ipc::try_forward_install(&temp.0, &file);
    if std::env::var_os("RICEWINE_SKIP_BLOCKED_UNIX_IPC").is_some()
        && no_server
            .as_ref()
            .is_err_and(|error| error.contains("Operation not permitted"))
    {
        eprintln!("SKIPPED Unix socket handoff: execution sandbox forbids connect() entirely");
        return;
    }
    assert!(!no_server.unwrap());
    let server = Server::bind(&temp.0).unwrap();
    let mode = fs::metadata(ipc::socket_path(&temp.0))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
    let forwarded = ipc::try_forward_install(&temp.0, &file);
    if std::env::var_os("RICEWINE_SKIP_BLOCKED_UNIX_IPC").is_some()
        && forwarded
            .as_ref()
            .is_err_and(|error| error.contains("Operation not permitted"))
    {
        eprintln!("SKIPPED Unix socket handoff: execution sandbox forbids socket connections");
        return;
    }
    assert!(forwarded.unwrap());
    assert_eq!(server.receive().unwrap(), Some(file));
    assert!(server.receive().unwrap().is_none());
    assert!(Server::bind(&temp.0).is_err());
    drop(server);
    assert!(!ipc::socket_path(&temp.0).exists());
}

#[test]
fn socket_bind_refuses_to_remove_non_socket_files() {
    let temp = Temp::new();
    let path = ipc::socket_path(&temp.0);
    fs::write(&path, "do not remove").unwrap();
    assert!(Server::bind(&temp.0).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), "do not remove");
}

#[test]
fn handoff_rejects_an_untrusted_or_nonexistent_path() {
    let temp = Temp::new();
    assert!(ipc::try_forward_install(&temp.0, &temp.0.join("missing.exe")).is_err());
    let file = temp.0.join("note.txt");
    fs::write(&file, "no").unwrap();
    assert!(ipc::try_forward_install(&temp.0, &file).is_err());
}
