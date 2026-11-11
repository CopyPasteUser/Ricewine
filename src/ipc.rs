//! Small, local-only single-instance handoff for Open With requests.
//! The GUI owns a private Unix socket; no daemon or network listener exists.

use crate::core;
use std::ffi::OsString;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

const MAX_PATH_BYTES: usize = 4096;

pub fn socket_path(root: &Path) -> PathBuf {
    root.join("ricewine.sock")
}

/// If a GUI already owns the socket, forward a path and exit successfully.
/// A missing/stale socket is not an error: the normal GUI can start instead.
pub fn try_forward_install(root: &Path, file: &Path) -> Result<bool, String> {
    let absolute = fs::canonicalize(file).map_err(|e| {
        format!(
            "Could not resolve installation file {}: {e}",
            file.display()
        )
    })?;
    core::validate_installer(&absolute, false)?;
    let bytes = absolute.as_os_str().as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_PATH_BYTES {
        return Err("Installation path is empty or exceeds 4096 bytes".into());
    }
    let mut stream = match UnixStream::connect(socket_path(root)) {
        Ok(stream) => stream,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
            ) =>
        {
            return Ok(false);
        }
        Err(error) => return Err(format!("Could not contact running RiceWine: {error}")),
    };
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|e| e.to_string())?;
    stream
        .write_all(&(bytes.len() as u32).to_be_bytes())
        .and_then(|_| stream.write_all(bytes))
        .map_err(|e| format!("Could not forward installation path: {e}"))?;
    Ok(true)
}

pub struct Server {
    listener: UnixListener,
    path: PathBuf,
    inode: u64,
}

impl Server {
    pub fn bind(root: &Path) -> Result<Self, String> {
        let path = socket_path(root);
        if let Ok(metadata) = fs::symlink_metadata(&path) {
            if !metadata.file_type().is_socket() {
                return Err(format!(
                    "Refusing to replace non-socket path {}",
                    path.display()
                ));
            }
            if UnixStream::connect(&path).is_ok() {
                return Err("Another RiceWine GUI is already listening".into());
            }
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        let listener = UnixListener::bind(&path).map_err(|e| e.to_string())?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        let inode = fs::symlink_metadata(&path)
            .map_err(|e| e.to_string())?
            .ino();
        Ok(Self {
            listener,
            path,
            inode,
        })
    }

    /// Poll without blocking the egui frame. A misbehaving peer cannot hold
    /// the UI indefinitely because accepted streams have a short timeout.
    pub fn receive(&self) -> Result<Option<PathBuf>, String> {
        let (mut stream, _) = match self.listener.accept() {
            Ok(connection) => connection,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(None),
            Err(error) => return Err(format!("Open With handoff failed: {error}")),
        };
        stream
            .set_read_timeout(Some(Duration::from_millis(200)))
            .map_err(|e| e.to_string())?;
        let mut length = [0u8; 4];
        stream.read_exact(&mut length).map_err(|e| e.to_string())?;
        let size = u32::from_be_bytes(length) as usize;
        if size == 0 || size > MAX_PATH_BYTES {
            return Err("Open With request contains an invalid path length".into());
        }
        let mut data = vec![0u8; size];
        stream.read_exact(&mut data).map_err(|e| e.to_string())?;
        let path = PathBuf::from(OsString::from_vec(data));
        // Only accept ordinary files that can later be validated by the
        // installer; never execute data received from the socket.
        if !path.is_file() {
            return Err(format!("Open With file does not exist: {}", path.display()));
        }
        core::validate_installer(&path, false)?;
        Ok(Some(path))
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        // Avoid unlinking a different socket that appeared at our old path.
        if let Ok(metadata) = fs::symlink_metadata(&self.path)
            && metadata.file_type().is_socket()
            && metadata.ino() == self.inode
        {
            let _ = fs::remove_file(&self.path);
        }
    }
}
