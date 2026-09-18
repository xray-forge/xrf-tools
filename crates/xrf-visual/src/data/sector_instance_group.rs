use serde::Serialize;

use crate::data::sector_geometry::SectorGeometry;
use crate::data::sector_surface::SectorSurface;
use crate::data::visual_section::VisualSection;

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
}

impl SectorInstanceGroup {
  /// Floats one instance's transform occupies.
  pub const FLOATS_PER_INSTANCE: usize = 16;
}
