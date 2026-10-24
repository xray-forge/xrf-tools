#![doc = include_str!("../README.md")]

mod commands;
mod conversion;
mod file;
pub mod ipc;
pub mod plugin;
mod request;
#[cfg(test)]
mod tests;

pub use conversion::{SpawnConversion, SpawnConversionResult};
pub use file::{SpawnChunk, SpawnChunkKind, SpawnFiles, SpawnSession, SpawnSessionDescriptor};
