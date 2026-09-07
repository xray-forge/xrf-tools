use serde::Serialize;

/// How the renderer draws a surface once its blender is compiled: opaque, cut out, or blended.
///
/// The three cases a viewer has to reproduce, and the only three a mesh surface reaches. Which one a blender comes to
/// is [`crate::XraySurfaceResolver`]'s answer; what each one means is here.
///
/// Modelled for the deferred renderer, R2 and above, because that is what the game runs and what a preview is compared
/// against. R1 differs in one place and the descriptor carries the knobs to say so: there the switch alone selects an
/// alpha blended pass and the authored reference is the test, where the deferred path tests against a constant.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum XraySurfaceDraw {
  /// Alpha is not read: whatever the texture carries in its fourth channel is ignored, and every texel is drawn.
  Opaque,
  /// Texels below the reference are killed and the rest are drawn opaque, in the g-buffer pass.
  AlphaTested { reference: u8 },
  /// Drawn in a forward pass, source alpha over inverse source alpha, testing against the authored reference.
  ///
  /// Reached when the author asked for something the g-buffer cannot hold - a partly transparent surface, or one it
  /// wants sorted - so the surface leaves the deferred path entirely. Depth is tested and not written
  /// (`Layers/xrRender/blenders/blender_deffer_model.cpp`), which is what lets one blended surface show through
  /// another.
  Blended { reference: u8 },
}

impl XraySurfaceDraw {
  /// Published beside the variant it fills because it is the number that surprises: the authored `Alpha ref` is not
  /// it, and a consumer comparing the two needs both.
  pub const DEFERRED_ALPHA_REFERENCE: u8 = 200;

  /// The reference a surface tests against, or `None` for one that reads no alpha.
  pub fn reference(self) -> Option<u8> {
    match self {
      Self::Opaque => None,
      Self::AlphaTested { reference } | Self::Blended { reference } => Some(reference),
    }
  }

  /// Whether the surface reads the texture's alpha channel at all.
  pub fn is_alpha_read(self) -> bool {
    !matches!(self, Self::Opaque)
  }

  /// Whether the surface is drawn in a blended pass rather than written into the g-buffer.
  pub fn is_blended(self) -> bool {
    matches!(self, Self::Blended { .. })
  }
}
