//! The contract every command is written against: what it declares and runs, what it is handed, how it fails, and the
//! domain group it is registered in.

mod command_context;
mod command_error;
mod command_group;
#[cfg(test)]
pub(crate) mod command_testing;
mod generic_command;

pub(crate) use command_context::CommandContext;
pub(crate) use command_error::{CommandError, CommandResult};
pub(crate) use command_group::CommandGroup;
pub(crate) use generic_command::GenericCommand;
