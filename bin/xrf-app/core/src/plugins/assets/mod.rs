//! The mounted asset roots over IPC: listing, probing and reading them, and hearing what changes in them.

pub(crate) mod commands;
mod plugin;
pub(crate) mod routes;

pub use plugin::AssetsPlugin;
