#![doc = include_str!("../README.md")]

mod diagnostics;
pub mod ipc;
mod paths;
pub mod plugin;
mod ui_log;
mod window;

pub use diagnostics::{MachineProbeState, StartupLog};
