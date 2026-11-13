//! The AI map compiler: `build.aimap` into `level.ai`, as SDK 0.7's `xrAI -f` writes it.

pub(crate) mod ai_grid;
pub(crate) mod ai_map;
pub(crate) mod ai_map_compilation;
pub(crate) mod ai_map_compiler;
pub(crate) mod ai_map_compiler_load;
pub(crate) mod ai_map_compiler_save;
pub(crate) mod ai_map_covers;
pub(crate) mod ai_map_guid;
pub(crate) mod ai_map_request;
pub(crate) mod ai_vertex;
pub(crate) mod cover;
