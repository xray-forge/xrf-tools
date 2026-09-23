use serde::Serialize;

use crate::data::sector::sector_geometry::SectorGeometry;
use crate::data::sector::sector_surface::SectorSurface;
use crate::data::visual::geometry::visual_section::VisualSection;

/// One mesh a sector draws many times, packed once with the places it stands.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorInstanceGroup {
  pub surface: SectorSurface,
  /// The drawables this group stands in for, by their index in the visuals run.
  pub drawables: Vec<u32>,
  /// The mesh itself, in its own space, its indices counting from its own first vertex.
  pub geometry: SectorGeometry,
  pub instance_count: u32,
  /// Sixteen floats for each instance, exactly as the engine stores a matrix.
  pub transforms: VisualSection,
  /// Two floats for each instance: what scales and then offsets its vertices' hemisphere term.
  pub hemi: VisualSection,
}

impl SectorInstanceGroup {
  /// Floats one instance's transform occupies.
  pub const FLOATS_PER_INSTANCE: usize = 16;

  /// Floats one instance's hemisphere terms occupy.
  pub const HEMI_FLOATS_PER_INSTANCE: usize = 2;
}
