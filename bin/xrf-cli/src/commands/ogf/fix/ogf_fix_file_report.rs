use std::path::Path;

use serde::Serialize;
use xrf_ogf::OgfResidue;

/// One visual whose bytes `ogf fix` changed, or would change on a dry run.
///
/// The bytes discarded are the two sizes' difference, and whether the file was written is the report's `isDryRun`, so
/// neither is carried again here.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfFixFileReport {
  /// Why the residue was inert to the engine: `split-motion-ref` or `trailing-fragment`. Absent when the only discarded
  /// bytes sat inside the motion refs chunk, past its count.
  cause: Option<String>,
  destination: String,
  /// The motion reference path the residue completed, for the split-reference shape only.
  discarded_reference: Option<String>,
  normalized_size: usize,
  original_size: usize,
  source: String,
}

impl OgfFixFileReport {
  pub fn new(
    source: &Path,
    destination: &Path,
    original_size: usize,
    normalized_size: usize,
    residue: Option<&OgfResidue>,
  ) -> Self {
    Self {
      cause: residue.map(|residue| String::from(residue.cause.as_str())),
      destination: xrf_utils::to_portable_path_string(destination),
      discarded_reference: residue
        .and_then(|residue| residue.cause.get_discarded_path())
        .map(String::from),
      normalized_size,
      original_size,
      source: xrf_utils::to_portable_path_string(source),
    }
  }

  /// Bytes the source carried that the engine never read, inside the motion refs chunk and after it.
  pub fn discarded_size(&self) -> usize {
    self.original_size.saturating_sub(self.normalized_size)
  }
}
