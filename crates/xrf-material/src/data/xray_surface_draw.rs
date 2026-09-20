use serde::Serialize;

/// How the renderer draws a surface once its blender is compiled.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum XraySurfaceDraw {
  /// Alpha is not read: whatever the texture carries in its fourth channel is ignored, and every texel is drawn.
  Opaque,
  /// Texels below the reference are killed and the rest are drawn opaque, in the g-buffer pass.
  AlphaTested { reference: u8 },
  /// Drawn in a forward pass, source alpha over inverse source alpha, testing against the authored reference.
  Blended { reference: u8 },
  /// Added to what is behind it, which is how a glow lights the air rather than covering it.
  Added { reference: u8 },
  /// Multiplied into what is behind it, which is how a decal darkens the surface it is laid on rather than replacing
  /// it. `is_doubled` is `MUL_2X`, whose destination factor is the source colour rather than zero.
  Multiplied { is_doubled: bool },
}

impl XraySurfaceDraw {
  /// What a cut-out surface tests against: `def_aref`, the deferred pixel shader's own constant.
  pub const DEFERRED_ALPHA_REFERENCE: u8 = 200;
}
