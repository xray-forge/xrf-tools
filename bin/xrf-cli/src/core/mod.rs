//! Shared infrastructure the commands and the composition root build on, belonging to no command domain.
//!
//! - `arguments` - the named argument groups commands declare and read back.
//! - `command` - the contract every command is written against, and the groups commands are registered in.
//! - `host_walk` - the files of one kind below a host path, unwalkable directories kept as errors.
//! - `report` - a run's machine-readable account of itself, and a verify command's verdict.
//! - `reports` - report shapes more than one command answers with.
//! - `terminal` - what reaches a person at the terminal.

pub(crate) mod arguments;
pub(crate) mod command;
pub(crate) mod host_walk;
pub(crate) mod logging;
pub(crate) mod report;
pub(crate) mod reports;
pub(crate) mod terminal;
