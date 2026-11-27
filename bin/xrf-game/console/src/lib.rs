#![doc = include_str!("../README.md")]

pub(crate) mod console_answer;
pub(crate) mod console_choice;
pub(crate) mod console_command;
pub(crate) mod console_completion;
pub(crate) mod console_context;
pub(crate) mod console_family;
pub(crate) mod console_line;
pub(crate) mod console_registry;
pub(crate) mod console_variable;
#[cfg(test)]
mod tests;

pub use crate::console_answer::ConsoleAnswer;
pub use crate::console_choice::ConsoleChoice;
pub use crate::console_command::{ConsoleCommand, ConsoleRun};
pub use crate::console_completion::ConsoleCompletion;
pub use crate::console_context::ConsoleContext;
pub use crate::console_family::ConsoleFamily;
pub use crate::console_line::ConsoleLine;
pub use crate::console_registry::ConsoleRegistry;
pub use crate::console_variable::ConsoleVariable;
