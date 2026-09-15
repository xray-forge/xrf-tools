#![doc = include_str!("../README.md")]
mod command_module;
mod constants;
mod enumerations;
mod exporter;
mod normalization;
mod output;
mod ownership;
mod references;
mod surface;
mod types_module;

pub use command_module::{export_raw_commands, finalize_command_module};
pub use constants::{COMMANDS_DIRECTORY, TYPES_DIRECTORY};
pub use enumerations::Enumerations;
pub use exporter::{CommandTypescript, command_exporter};
pub use output::reset_directory;
pub use ownership::TypeOwnership;
pub use surface::{SurfaceDrift, compare_surfaces, read_surface};
pub use types_module::export_type_modules;
