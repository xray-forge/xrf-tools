use serde::{Deserialize, Serialize};

/// How the screen's ambient occlusion is searched.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderAmbientOcclusionMethod {
  /// GTAO as XeGTAO computes it: everything the depth shows taken to reach infinitely behind it.
  #[default]
  Gtao,
  /// VBAO, visibility-bitmask ambient occlusion: every occluder a thickness deep, so light passes behind thin things.
  Vbao,
}
