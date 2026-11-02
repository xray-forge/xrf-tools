#![doc = include_str!("../README.md")]

mod collected;
mod enumerations;
mod input;
mod ipc_bindings_generator;
mod typescript;
mod verify;
mod write;

pub use input::{BulkRouteDeclaration, IpcCommandSurface, MirrorImports, RawCommandDeclaration, TypeModules};
pub use ipc_bindings_generator::IpcBindingsGenerator;
pub use verify::SurfaceDrift;
