use std::fs;
use std::io::Error as IoError;
use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

/// The generated tree on disk: its layout, its header, and the writes every writer makes through it.
pub(crate) struct GeneratedOutput;

impl GeneratedOutput {
  /// The first line of every generated module.
  pub(crate) const HEADER: &'static str = "// Auto-generated rust bindings. Do not edit it manually.\n";

  /// Directory holding one module per crate that declares an exported type.
  pub(crate) const TYPES_DIRECTORY: &'static str = "types";

  /// Directory holding one module per Tauri plugin, each exporting only that plugin's commands.
  pub(crate) const COMMANDS_DIRECTORY: &'static str = "commands";

  /// Source Specta rendered, with its doc comment indentation reduced to one space.
  pub(crate) fn normalize_doc_indentation(contents: &str) -> String {
    contents.replace("/**  ", "/** ").replace(" *  ", " * ")
  }

  /// Writes a generated module as given: what Specta rendered in it is normalized before, and nothing else is.
  pub(crate) fn write(path: &Path, contents: &str) -> XrfResult {
    fs::write(path, contents).map_err(|error| Self::describe_io("write", path, &error))
  }

  /// Empties a generated directory so the committed output is exactly what the current run produced.
  pub(crate) fn reset_directory(path: &Path) -> XrfResult {
    fs::create_dir_all(path).map_err(|error| Self::describe_io("create", path, &error))?;

    for entry in fs::read_dir(path).map_err(|error| Self::describe_io("read", path, &error))? {
      let entry = entry.map_err(|error| Self::describe_io("read an entry of", path, &error))?;

      if entry.path().extension().is_some_and(|extension| extension == "ts") {
        fs::remove_file(entry.path()).map_err(|error| Self::describe_io("remove", &entry.path(), &error))?;
      }
    }

    Ok(())
  }

  /// A failed filesystem step on the generated tree, naming the action and the path it was taken on.
  pub(crate) fn describe_io(action: &str, path: &Path, error: &IoError) -> XrfError {
    XrfError::new_io_error(
      format!("Failed to {action} {}: {error}", format_path(path)),
      error.kind(),
    )
  }
}
