use serde::{Deserialize, Serialize};

/// What turns the physical light's scene-referred frame into the display's colours, before it is encoded for the
/// display. The engine's light model always shows its frame by the engine's curve.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderDisplayTransform {
  /// The engine's curve, `x (1 + x / W²) / (1 + x)` a channel, white at 1.7: each channel clips on its own, so bright
  /// colours walk to yellow and white.
  Engine,
  /// AgX (Troy Sobotka; Blender's default view since 4.0): a log encoding and a sigmoid over a rotated, inset gamut,
  /// so highlights roll off towards white without their hues walking.
  Agx,
  /// Khronos PBR Neutral (2024): colours kept as they are up to a shoulder near white, then compressed and desaturated,
  /// for materials read as authored.
  #[default]
  PbrNeutral,
}
