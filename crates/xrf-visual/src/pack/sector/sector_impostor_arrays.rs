use xrf_level::LevelVisual;
use xrf_math::Vector3d;
use xrf_ogf::{OgfLodDefinitionChunk, OgfLodVertex};

use crate::data::sector::sector_impostor_group::SectorImpostorGroup;
use crate::data::sector::sector_impostors::SectorImpostors;
use crate::pack::sector::sector_surface_table::SectorSurfaceTable;
use crate::pack::visual_buffer_builder::VisualBufferBuilder;
use crate::pack::visual_conversion::convert_vector;

/// The impostors of a sector under construction, in the order their places name them.
#[derive(Debug, Default)]
pub(crate) struct SectorImpostorArrays {
  groups: Vec<SectorImpostorGroup>,
  spheres: Vec<f32>,
  factors: Vec<f32>,
  corners: Vec<f32>,
  normals: Vec<f32>,
}

impl SectorImpostorArrays {
  /// What a byte term is divided by, as `unpack_D3DCOLOR` reads one.
  const BYTE_SCALE: f32 = 255.0;

  /// Where `D3DCOLOR` keeps its alpha, which is where a corner's hemisphere term rides.
  const ALPHA_SHIFT: u32 = 24;

  /// Impostors gathered so far, which is the index the next one takes.
  pub(crate) fn len(&self) -> u32 {
    self.factors.len() as u32
  }

  /// Appends one `MT_LOD` visual's impostor, its surface joining the run of the one before where it is the same.
  pub(crate) fn push(
    &mut self,
    visual: &LevelVisual,
    definition: &OgfLodDefinitionChunk,
    surfaces: &SectorSurfaceTable,
  ) {
    let radius: f32 = visual.header.bounding_sphere.radius;
    let center: Vector3d = convert_vector(&visual.header.bounding_sphere.position);

    self.spheres.extend_from_slice(&[center.x, center.y, center.z, radius]);
    self.factors.push(OgfLodDefinitionChunk::get_lod_factor(
      &visual.header.bounding_box,
      radius,
    ));

    for (index, facet) in definition.facets.iter().enumerate() {
      let normal: Vector3d = convert_vector(&definition.get_facet_normal(index));

      self.normals.extend_from_slice(&[normal.x, normal.y, normal.z, 0.0]);

      for corner in &facet.vertices {
        self.push_corner(corner);
      }
    }

    match self.groups.last_mut() {
      Some(group) if group.surface.shader_id == visual.header.shader_id => group.count += 1,
      _ => self.groups.push(SectorImpostorGroup {
        count: 1,
        start: self.len() - 1,
        surface: surfaces.get(visual.header.shader_id),
      }),
    }
  }

  /// Writes the impostors into the buffer, or nothing for a sector that has none.
  pub(crate) fn write_into(self, builder: &mut VisualBufferBuilder) -> Option<SectorImpostors> {
    if self.factors.is_empty() {
      return None;
    }

    Some(SectorImpostors {
      count: self.len(),
      corners: builder.push_f32_section(&self.corners),
      factors: builder.push_f32_section(&self.factors),
      groups: self.groups,
      normals: builder.push_f32_section(&self.normals),
      spheres: builder.push_f32_section(&self.spheres),
    })
  }

  fn push_corner(&mut self, corner: &OgfLodVertex) {
    let position: Vector3d = convert_vector(&corner.position);

    self.corners.extend_from_slice(&[
      position.x,
      position.y,
      position.z,
      corner.texture_coordinate.0,
      corner.texture_coordinate.1,
      f32::from((corner.rgb_hemi >> Self::ALPHA_SHIFT) as u8) / Self::BYTE_SCALE,
      f32::from(corner.sun) / Self::BYTE_SCALE,
      0.0,
    ]);
  }
}
