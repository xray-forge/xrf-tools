#![doc = include_str!("../README.md")]

mod commands;
pub mod ipc;
pub mod plugin;
mod project;
mod request;
#[cfg(test)]
mod tests;

pub use project::{DialogProjects, OpenedDialogProject};
