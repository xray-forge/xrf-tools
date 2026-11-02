use std::fs;
use std::path::Path;

use xrf_utils::format_path;

/// The generated tree on disk: its layout, its header, and the writes every writer makes through it.
pub(crate) struct GeneratedOutput;

impl GeneratedOutput {
  /// The first line of every generated module.
  pub const HEADER: &'static str = "// Auto-generated rust bindings. Do not edit it manually.\n";

  /// Directory holding one module per crate that declares an exported type.
  pub const TYPES_DIRECTORY: &'static str = "types";

  /// Directory holding one module per Tauri plugin, each exporting only that plugin's commands.
  pub const COMMANDS_DIRECTORY: &'static str = "commands";

  /// Source Specta rendered, with its doc comment indentation reduced to one space.
  pub fn normalize_doc_indentation(contents: &str) -> String {
    contents.replace("/**  ", "/** ").replace(" *  ", " * ")
  }

  /// Writes a generated module as given: what Specta rendered in it is normalized before, and nothing else is.
  pub fn write(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap_or_else(|error| panic!("Failed to write {}: {error}", format_path(path)));
  }

  /// Empties a generated directory so the committed output is exactly what the current run produced.
  pub fn reset_directory(path: &Path) {
    fs::create_dir_all(path).unwrap_or_else(|error| panic!("Failed to create {}: {error}", format_path(path)));

    for entry in fs::read_dir(path).unwrap_or_else(|error| panic!("Failed to read {}: {error}", format_path(path))) {
      let entry = entry.unwrap_or_else(|error| panic!("Failed to read an entry of {}: {error}", format_path(path)));

      if entry.path().extension().is_some_and(|extension| extension == "ts") {
        fs::remove_file(entry.path())
          .unwrap_or_else(|error| panic!("Failed to remove {}: {error}", format_path(&entry.path())));
      }
    }
  }
}
