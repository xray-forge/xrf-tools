#![doc = include_str!("../README.md")]

mod commands;
pub mod ipc;
pub mod plugin;
mod request;
mod verify;

pub use verify::{GamedataCheckSummary, GamedataVerifySummary};
