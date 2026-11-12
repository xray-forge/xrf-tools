use serde::{Deserialize, Serialize};

/// How a ray model face treats a light ray crossing it, as the SDK's lighters do (`getLastRP_Scale`,
/// `xrLC_Light/detail_slot_calculate.cpp:93`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShadowClass {
  /// Its compiler shader casts no shadow: the ray passes.
  Passes,
  /// Its texture has alpha and an image: the light is scaled by `1 - alpha` where the ray crosses it.
  Alpha,
  /// Anything else: the ray is blocked.
  Opaque,
}
