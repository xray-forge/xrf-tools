//! Where the loopback transport listens, for the frontend to fetch bulk bytes from.

pub(crate) mod commands;
mod plugin;

pub use plugin::TransportPlugin;
