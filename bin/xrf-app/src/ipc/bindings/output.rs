use std::fs;
use std::io;
use std::path::Path;

use xrf_utils::format_path;

use crate::ipc::bindings::constants::TAURI_SPECTA_HEADER;

/// Rewrites a file Tauri Specta just wrote, dropping its header and normalizing its doc comments.
pub(super) fn normalize_generated_bindings(path: &Path) -> io::Result<()> {
  let bindings = fs::read_to_string(path)?;

  fs::write(path, normalized(&bindings.replacen(TAURI_SPECTA_HEADER, "", 1)))
}

/// Generated source with Specta's doc comment indentation reduced to one space.
fn normalized(contents: &str) -> String {
  contents.replace("/**  ", "/** ").replace(" *  ", " * ")
}

pub(super) fn write_generated(path: &Path, contents: &str) {
  fs::write(path, normalized(contents))
    .unwrap_or_else(|error| panic!("Failed to write {}: {error}", format_path(path)));
}

/// Empties a generated directory so the committed output is exactly what the current run produced.
pub(super) fn reset_directory(path: &Path) {
  fs::create_dir_all(path).unwrap_or_else(|error| panic!("Failed to create {}: {error}", format_path(path)));

  for entry in fs::read_dir(path).unwrap_or_else(|error| panic!("Failed to read {}: {error}", format_path(path))) {
    let entry = entry.unwrap_or_else(|error| panic!("Failed to read an entry of {}: {error}", format_path(path)));

    if entry.path().extension().is_some_and(|extension| extension == "ts") {
      fs::remove_file(entry.path())
        .unwrap_or_else(|error| panic!("Failed to remove {}: {error}", format_path(&entry.path())));
    }
  }
}
