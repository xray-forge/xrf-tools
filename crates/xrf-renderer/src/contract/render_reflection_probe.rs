use serde::{Deserialize, Serialize};

/// How fine the surroundings cube is, the level around the camera a reflection reads where it meets nothing on the
/// screen: texels a side of each of its six faces. Finer is sharper far reflections for more memory and a larger face
/// drawn and resolved each frame.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderReflectionProbe {
  /// 256 texels a side: 12 MB, a soft probe.
  Coarse,
  /// 512 texels a side: 50 MB, a seventh of the frame's detail at the default field of view.
  #[default]
  Fine,
  /// 1,024 texels a side: 200 MB, a planar reflection's spend.
  Finest,
}

impl RenderReflectionProbe {
  /// Every size, coarsest first.
  pub const ALL: [Self; 3] = [Self::Coarse, Self::Fine, Self::Finest];

  /// Texels a side of a face.
  pub const fn get_size(self) -> u32 {
    match self {
      Self::Coarse => 256,
      Self::Fine => 512,
      Self::Finest => 1024,
    }
  }

  /// Mips the prefiltered cube holds, the first the faces as drawn, the last four texels a side at roughness one.
  pub const fn get_levels(self) -> u32 {
    match self {
      Self::Coarse => 7,
      Self::Fine => 8,
      Self::Finest => 9,
    }
  }
}
