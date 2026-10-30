#![doc = include_str!("../README.md")]

pub mod assets;
pub mod cache;
pub mod directories;
pub mod execution;
pub mod ipc;
pub mod jobs;
pub mod logging;
pub mod plugins;
pub mod session;
mod tauri_result;
pub mod transport;
pub mod window;

pub use tauri_result::TauriResult;
