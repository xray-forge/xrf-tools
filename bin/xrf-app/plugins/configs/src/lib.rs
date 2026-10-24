//! LTX configuration projects: a project opened as a session of the roots it reads, its resolved sections and the
//! findings of its verification, and the formatting and verification of a directory.

pub mod commands;
pub mod configs_project;
pub mod descriptor;
mod opened_configs_project;
pub mod plugin;
pub mod registry;
pub mod request;
pub mod resolved_root;
pub mod state;
#[cfg(test)]
mod tests;
