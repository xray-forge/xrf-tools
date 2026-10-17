use serde::Serialize;
use xrf_ogf::OgfResidue;

/// Bytes a visual ends with that the engine's loader never reads.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfResidueReport {
  cause: String,
  /// The motion reference path the residue completes, for the split-reference shape only.
  discarded_reference: Option<String>,
  position: u64,
  /// Every ignored byte, counting those inside the motion refs chunk as well as those after it.
  size: usize,
}

impl OgfResidueReport {
  /// The residue, with `trailing` the ignored bytes inside the motion refs chunk.
  pub fn new(residue: &OgfResidue, trailing: usize) -> Self {
    Self {
      cause: String::from(residue.cause.as_str()),
      discarded_reference: residue.cause.get_discarded_path().map(String::from),
      position: residue.position,
      size: residue.bytes.len() + trailing,
    }
  }
}
