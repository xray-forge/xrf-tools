use serde::{Deserialize, Serialize};

/// The body a texture is laid on to be looked at.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextureSurfaceShape {
  /// Flat and face on, where the decode is read most directly and tiling is judged.
  Plane,
  /// Curved, so the normal sweeps every grazing angle a wrong tangent sign shows up at.
  Sphere,
  /// Edged, where a seam and the wrap of a tiling texture meet.
  Cube,
}
