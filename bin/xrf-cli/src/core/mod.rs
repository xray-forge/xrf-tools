//! Shared infrastructure the commands and the composition root build on, belonging to no command domain.

pub mod collisions;
pub mod command_context;
pub mod command_error;
#[cfg(test)]
pub mod command_testing;
pub mod execution;
pub mod generic_command;
pub mod logging;
pub mod output;
pub mod progress;
pub mod reporting;
pub mod reports;
pub mod staged_write;
