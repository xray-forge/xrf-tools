#![doc = include_str!("../README.md")]

pub mod browse;
pub mod describe;
pub mod edit;
pub mod encoding;
pub mod ipc;
mod lease;
pub mod plugin;
pub mod surface;
#[cfg(test)]
mod tests;
