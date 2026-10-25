//! What the generator gathers before it writes anything: the commands every plugin exported and the constants marked
//! for the bindings, with the types both carry.

mod collected_commands;
mod collected_constants;

pub(crate) use collected_commands::CollectedCommands;
pub(crate) use collected_constants::CollectedConstants;
