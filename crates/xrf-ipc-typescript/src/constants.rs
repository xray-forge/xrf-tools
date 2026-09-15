//! Spellings several of the writers share, which belong to none of them alone.

pub(crate) const GENERATED_HEADER: &str = "// Auto-generated rust bindings. Do not edit it manually.\n";

/// Bindings import specifier root, matching the `@/*` path alias of the frontend.
pub(crate) const BINDINGS_ROOT: &str = "@/core/ipc";

/// Directory holding one module per crate that declares an exported type.
pub(crate) const TYPES_DIRECTORY: &str = "types";

/// Directory holding one module per Tauri plugin, each exporting only that plugin's commands.
pub(crate) const COMMANDS_DIRECTORY: &str = "commands";
