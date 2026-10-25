//! What the caller hands a run: each plugin's surface, which crates share a types module, and how the generated
//! modules address one another and the runtime beside them.

mod ipc_command_surface;
mod mirror_imports;
mod type_modules;

pub use ipc_command_surface::{BulkRouteDeclaration, IpcCommandSurface, RawCommandDeclaration};
pub use mirror_imports::MirrorImports;
pub use type_modules::TypeModules;
