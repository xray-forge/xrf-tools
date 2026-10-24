//! Running work over IPC, whatever domain it belongs to: what is going on, and asking it to stop.

pub(crate) mod commands;
mod plugin;

pub use plugin::JobsPlugin;
