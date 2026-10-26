use serde::Serialize;
use xrf_ogf::OgfResidue;

/// Where a visual departs from a well-formed chunk tree without the engine's loader noticing: bytes it never reads, or
/// a size its payload contradicts, which counts no ignored bytes.
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
