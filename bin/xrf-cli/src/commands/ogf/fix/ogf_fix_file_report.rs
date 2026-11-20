use std::path::Path;

use serde::Serialize;
use xrf_ogf::OgfResidue;

/// One visual whose bytes `ogf fix` changed, or would change on a dry run.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OgfFixFileReport {
  /// Why the residue was inert to the engine: `split-motion-ref`, `trailing-fragment` or `unread-trailing-chunk` for
  /// bytes dropped, `understated-motion-refs`, `overstated-motion-refs` or `overstated-lods` for a size corrected.
  /// Absent when the only discarded bytes sat inside the motion refs chunk, past its count.
  cause: Option<String>,
  destination: String,
  /// The motion reference path the residue completed, for the split-reference shape only.
  discarded_reference: Option<String>,
  normalized_size: usize,
  original_size: usize,
  source: String,
}

impl OgfFixFileReport {
  pub(crate) fn new(
    source: &Path,
    destination: &Path,
    original_size: usize,
    normalized_size: usize,
    residue: Option<&OgfResidue>,
  ) -> Self {
    Self {
      cause: residue.map(|residue| String::from(residue.cause.as_str())),
      destination: xrf_utils::format_portable_path(destination),
      discarded_reference: residue
        .and_then(|residue| residue.cause.get_discarded_path())
        .map(String::from),
      normalized_size,
      original_size,
      source: xrf_utils::format_portable_path(source),
    }
  }

  /// Bytes the source carried that the engine never read, inside the motion refs chunk and after the last chunk.
  pub(crate) fn discarded_size(&self) -> usize {
    self.original_size.saturating_sub(self.normalized_size)
  }
}
