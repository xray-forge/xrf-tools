use std::path::Path;

use serde::Serialize;
use xrf_ogf::OgfResidue;

/// One visual whose bytes `ogf fix` changed, or would change on a dry run.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfFixFileReport {
  /// Why the residue was inert to the engine: `split-motion-ref` or `trailing-fragment`. Absent when the only discarded
  /// bytes sat inside the motion refs chunk, past its count.
  cause: Option<String>,
  destination: String,
  /// The motion reference path the residue completed, for the split-reference shape only.
  discarded_reference: Option<String>,
  /// Bytes the source carried that the engine never read, inside the motion refs chunk and after it.
  pub discarded_size: usize,
  is_written: bool,
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
    is_written: bool,
  ) -> Self {
    Self {
      cause: residue.map(|residue| String::from(residue.cause.as_str())),
      destination: xrf_utils::to_portable_path_string(destination),
      discarded_reference: residue
        .and_then(|residue| residue.cause.get_discarded_path())
        .map(String::from),
      discarded_size: original_size.saturating_sub(normalized_size),
      is_written,
      normalized_size,
      original_size,
      source: xrf_utils::to_portable_path_string(source),
    }
  }
}
