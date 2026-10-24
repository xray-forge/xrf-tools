use serde::{Deserialize, Serialize};
use xrf_material::XraySurfaceDraw;

/// Which of the engine's three readings of a texture's alpha the body is drawn with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TextureSurfaceAlpha {
  /// Not read at all, which is what the plain deferred base shader does.
  Ignored,
  /// Clipped against `def_aref`, which is what every `_aref` shader does.
  CutOut,
  /// Composited over what is behind it, which the deferred pass never does.
  Blended,
}

impl TextureSurfaceAlpha {
  /// How a surface reading its alpha this way is drawn.
  pub fn to_draw(self) -> XraySurfaceDraw {
    match self {
      Self::Ignored => XraySurfaceDraw::Opaque,
      Self::CutOut => XraySurfaceDraw::AlphaTested {
        reference: XraySurfaceDraw::DEFERRED_ALPHA_REFERENCE,
      },
      Self::Blended => XraySurfaceDraw::Blended { reference: 0 },
    }
  }
}
