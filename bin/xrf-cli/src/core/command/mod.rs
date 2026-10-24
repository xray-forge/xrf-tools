//! The contract every command is written against: what it declares and runs, what it is handed, how it fails, and the
//! domain group it is registered in.

mod command_context;
mod command_error;
mod command_group;
#[cfg(test)]
pub mod command_testing;
mod generic_command;

pub use command_context::CommandContext;
pub use command_error::{CommandError, CommandResult};
pub use command_group::CommandGroup;
pub use generic_command::GenericCommand;
