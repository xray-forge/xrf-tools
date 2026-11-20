use serde::{Deserialize, Serialize};

/// How a ray model face treats a light ray crossing it, as the SDK's lighters do (`getLastRP_Scale`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum ShadowClass {
  /// Its compiler shader casts no shadow: the ray passes.
  Passes,
  /// Its texture has alpha and an image: the light is dimmed by `1 - alpha` or `1 - alpha²` where the ray crosses,
  /// as the tracing's rules say.
  Alpha,
  /// Anything else: the ray is blocked.
  Opaque,
}
