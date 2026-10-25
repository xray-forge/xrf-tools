//! Writing the mirror: `types/` and `commands/`, the ownership deciding which module declares what, and the names a
//! plugin's modules declare.

mod command_module_writer;
mod generated_output;
mod plugin_modules;
mod plugin_symbols;
mod type_module_writer;
mod type_ownership;
mod wrapper_module;

pub(crate) use command_module_writer::CommandModuleWriter;
pub(crate) use generated_output::GeneratedOutput;
pub(crate) use plugin_modules::PluginModules;
pub(crate) use plugin_symbols::PluginSymbols;
pub(crate) use type_module_writer::TypeModuleWriter;
pub(crate) use type_ownership::TypeOwnership;
