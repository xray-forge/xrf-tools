use serde::{Deserialize, Serialize};
use xrf_dds::DdsEncodeCandidate;

/// The format both halves of a bump pair are written in.
///
/// Only formats that keep alpha at full precision are offered, because the pair stores the two values that must stay
/// precise there: the normal's `x` in the first half and the height in the second. BC1 keeps one bit of alpha and BC2
/// four. The error plane of the second half is measured against whichever of these encoded the first half, so the two
/// halves always share one format.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GenerateBumpFormat {
  /// DXT5, the format `DXTCompressBump` writes, which DX9 and DX11 both load.
  #[default]
  Bc3,
  /// BC7, which loses less of the normals but which D3DX9, and so the DX9 renderer, cannot load.
  Bc7,
}

impl GenerateBumpFormat {
  pub const ALL: [Self; 2] = [Self::Bc3, Self::Bc7];

  /// The encoder candidate this format writes through, which also answers which renderers load it.
  pub const fn to_candidate(self) -> DdsEncodeCandidate {
    match self {
      Self::Bc3 => DdsEncodeCandidate::Bc3,
      Self::Bc7 => DdsEncodeCandidate::Bc7,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::GenerateBumpFormat;

  #[test]
  fn only_a_bc7_pair_is_refused_by_a_renderer() {
    // The DX9 path cannot load BC7, and a pair is no exception; DXT5 loads wherever the format was checked.
    assert!(GenerateBumpFormat::Bc7.to_candidate().is_refused_anywhere());
    assert!(!GenerateBumpFormat::Bc3.to_candidate().is_refused_anywhere());
  }
}
