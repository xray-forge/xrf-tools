#![doc = include_str!("../README.md")]

pub(crate) mod console_answer;
pub(crate) mod console_command;
pub(crate) mod console_context;
pub(crate) mod console_line;
pub(crate) mod console_registry;
pub(crate) mod console_variable;
#[cfg(test)]
mod tests;

pub use crate::console_answer::ConsoleAnswer;
pub use crate::console_command::ConsoleCommand;
pub use crate::console_context::ConsoleContext;
pub use crate::console_line::ConsoleLine;
pub use crate::console_registry::ConsoleRegistry;
pub use crate::console_variable::ConsoleVariable;
