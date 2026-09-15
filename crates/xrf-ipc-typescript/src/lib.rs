#![doc = include_str!("../README.md")]

mod bindings_surface;
mod command_module_writer;
mod command_typescript;
mod constants;
mod enumerations;
mod generated_output;
mod ipc_bindings_generator;
mod ipc_command_surface;
mod type_module_writer;
mod type_ownership;
mod typescript;

pub use bindings_surface::SurfaceDrift;
pub use ipc_bindings_generator::IpcBindingsGenerator;
pub use ipc_command_surface::{IpcCommandSurface, RawCommandDeclaration};
