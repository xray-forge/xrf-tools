//! `all.spawn` files opened as sessions, read section by section, edited, and packed or unpacked.

pub mod commands;
pub(crate) mod conversion;
mod descriptor;
mod loading;
pub mod plugin;
pub mod registry;
pub(crate) mod request;
pub mod state;
#[cfg(test)]
mod tests;
pub(crate) use descriptor::SpawnSessionDescriptor;
