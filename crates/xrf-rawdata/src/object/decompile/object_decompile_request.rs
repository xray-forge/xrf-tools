use std::path::{Path, PathBuf};

use xrf_utils::OutputFiles;

use crate::provenance::object_provenance::ObjectProvenance;

/// A compiled model's decompile into an `.object` on disk, as a front-end asks for it: the OGF or `.dm` read, the
/// `.object` written with its provenance sidecar beside it, and whether existing ones are replaced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectDecompileRequest {
  pub source: PathBuf,
  pub destination: PathBuf,
  /// Whether an existing object or sidecar is replaced rather than refused before the source is read.
  pub is_forced: bool,
}

impl ObjectDecompileRequest {
  /// Where the object's provenance sidecar is written.
  pub fn get_sidecar_path(&self) -> PathBuf {
    ObjectProvenance::sidecar_path(&self.destination)
  }

  /// What the decompile writes: the object and its sidecar.
  pub fn get_outputs(&self) -> OutputFiles {
    let names: Vec<String> = [self.destination.clone(), self.get_sidecar_path()]
      .iter()
      .filter_map(|path| path.file_name())
      .map(|name| name.to_string_lossy().into_owned())
      .collect();

    OutputFiles::new(self.destination.parent().unwrap_or_else(|| Path::new("")))
      .with_files(names.iter().map(String::as_str))
  }
}
