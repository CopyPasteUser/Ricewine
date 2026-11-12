#![forbid(unsafe_code)]

pub mod backup;
pub mod core;
pub mod discovery;
pub mod doctor;
pub mod integration;
#[cfg(unix)]
pub mod ipc;
pub mod library;
pub mod preferences;
pub mod runtime;
pub mod storage;
pub mod updates;
pub mod workspace;
