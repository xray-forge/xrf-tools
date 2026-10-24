//! The infrastructure every plugin of the desktop application builds on, belonging to no domain: the mounted asset roots
//! and their changes, documents opened as sessions in owner scopes, the worker pool and its jobs, the loopback transport,
//! where the application writes, its log, and the windows plugins draw into. Its own plugins expose the asset roots,
//! jobs, scopes and transport over IPC; `registry` holds the macro every plugin crate expands its command domains
//! through, and expands core's own.

pub mod app_directories;
pub mod assets;
pub mod directories;
pub mod execution;
pub mod file_stamp;
pub mod jobs;
pub mod logging;
pub mod plugins;
pub mod process;
pub mod registry;
pub mod session;
pub mod transport;
pub mod types;
pub mod window;
