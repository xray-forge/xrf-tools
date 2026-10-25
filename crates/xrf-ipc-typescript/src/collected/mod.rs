//! What the generator gathers before it writes anything: the commands every plugin exported, through the exporter
//! that records them and renders their wrappers, and the constants marked for the bindings, with the types both carry.

mod collected_commands;
mod collected_constants;
mod command_typescript;
mod command_wrappers;
mod plugin_commands;
mod specta_command;

pub(crate) use collected_commands::CollectedCommands;
pub(crate) use collected_constants::CollectedConstants;
pub(crate) use command_typescript::CommandTypescript;
pub(crate) use command_wrappers::CommandWrappers;
pub(crate) use plugin_commands::PluginCommands;
pub(crate) use specta_command::SpectaCommand;
