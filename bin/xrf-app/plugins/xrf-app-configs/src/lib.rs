#![doc = include_str!("../README.md")]

mod commands;
mod format;
pub mod ipc;
pub mod plugin;
mod project;
mod request;
#[cfg(test)]
mod tests;

pub use format::ConfigsFormat;
pub use project::{
  ConfigsDocument, ConfigsProject, ConfigsProjectDescriptor, ConfigsProjects, ConfigsResolvedRoot, OpenedConfigsProject,
};
