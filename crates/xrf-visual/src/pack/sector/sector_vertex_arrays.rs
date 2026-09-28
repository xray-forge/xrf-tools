use byteorder::ByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelVertex, LevelVertexLayout, LevelVertexPayload};
use xrf_math::Vector3d;

use crate::data::sector::sector_attributes::SectorAttributes;
use crate::data::sector::sector_geometry::SectorGeometry;
use crate::data::visual::bounds::visual_bounds::VisualBounds;
use crate::data::visual::geometry::visual_section::VisualSection;
use crate::pack::visual_buffer_builder::VisualBufferBuilder;
use crate::pack::visual_cluster_table::VisualClusterTable;
use crate::pack::visual_conversion::convert_vector;

/// The attribute arrays of a sector under construction, one entry per packed vertex: positions decoded into renderer
/// space, everything else as xrLC wrote it.
#[derive(Debug)]
pub(crate) struct SectorVertexArrays {
  attributes: SectorAttributes,
  /// Shorts a base coordinate takes a vertex, which every declaration packed here has to share.
  uv_components: u32,
  positions: Vec<f32>,
  normals: Vec<u8>,
  tangents: Vec<u8>,
  binormals: Vec<u8>,
  uvs: Vec<i16>,
  lightmap_uvs: Vec<i16>,
  colors: Vec<u8>,
}

impl SectorVertexArrays {
  /// Shorts a baked base coordinate takes a vertex.
  pub const BAKED_UV_COMPONENTS: u32 = LevelVertexLayout::BASE_COORDINATE_SHORTS as u32;

  /// What a vertex whose declaration carries no such direction packs: as near no direction as a byte comes, and
  /// nothing riding in the fourth byte, which is exact for a coordinate's low byte.
  const NEUTRAL_DIRECTION: [u8; 4] = [128, 128, 128, 0];

  /// What a vertex whose declaration carries no colour packs: no baked light, and the sun unoccluded.
  const NEUTRAL_COLOR: [u8; 4] = [0, 0, 0, 255];

  /// What a vertex whose declaration carries no lightmap coordinate packs.
  const NEUTRAL_LIGHTMAP: [i16; 2] = [0, 0];

  pub fn new(attributes: SectorAttributes, uv_components: u32) -> Self {
    Self {
      attributes,
      binormals: Vec::new(),
      colors: Vec::new(),
      lightmap_uvs: Vec::new(),
      normals: Vec::new(),
      positions: Vec::new(),
      tangents: Vec::new(),
      uv_components,
      uvs: Vec::new(),
    }
  }

  /// Shorts a base coordinate of this declaration takes a vertex.
  pub const fn uv_components_of(layout: &LevelVertexLayout) -> u32 {
    layout.get_texture_coordinate_shorts() as u32
  }

  /// Vertices packed so far, which is the base the next range is rebased onto.
  pub fn get_vertex_count(&self) -> u32 {
    (self.positions.len() / 3) as u32
  }

  /// Appends a range of stored vertices: positions converted into renderer space, the rest copied as they are but
  /// for each direction's z, whose byte is negated with it. `255 - b` is exact: `(255 - b) / 127.5 - 1` is
  /// `-(b / 127.5 - 1)`.
  ///
  /// # Errors
  ///
  /// Returns an error, packing nothing, when the range stores its base coordinate in another width than the arrays
  /// do: a tree's four shorts cannot share one array with a baked surface's two.
  pub fn push<T: ByteOrder>(&mut self, payload: &LevelVertexPayload) -> XrfResult {
    let layout: &LevelVertexLayout = payload.get_layout();
    let uv_components: usize = self.uv_components as usize;

    if self.attributes.uvs
      && layout.get_texture_coordinate_offset().is_some()
      && Self::uv_components_of(layout) != self.uv_components
    {
      return Err(XrfError::new_not_implemented_error(format!(
        "stores its base coordinate as {} shorts beside geometry storing {}, which one mesh cannot draw both of",
        Self::uv_components_of(layout),
        self.uv_components
      )));
    }

    for vertex in payload.vertices() {
      Self::push_vector(&mut self.positions, &convert_vector(&vertex.get_position::<T>()));

      if self.attributes.normals {
        Self::push_direction(&mut self.normals, vertex.get_normal());
      }

      if self.attributes.tangents {
        Self::push_direction(&mut self.tangents, vertex.get_tangent());
      }

      if self.attributes.binormals {
        Self::push_direction(&mut self.binormals, vertex.get_binormal());
      }

      if self.attributes.uvs {
        let shorts: [i16; LevelVertexLayout::TREE_COORDINATE_SHORTS] =
          vertex.get_texture_coordinate::<T>().unwrap_or_default();

        self.uvs.extend_from_slice(&shorts[..uv_components]);
      }

      if self.attributes.lightmap_uvs {
        self
          .lightmap_uvs
          .extend_from_slice(&vertex.get_lightmap_coordinate::<T>().unwrap_or(Self::NEUTRAL_LIGHTMAP));
      }

      if self.attributes.colors {
        self
          .colors
          .extend_from_slice(&vertex.get_color().unwrap_or(Self::NEUTRAL_COLOR));
      }
    }

    Ok(())
  }

  /// Every packed position, three floats in renderer space.
  pub fn get_positions(&self) -> &[[f32; 3]] {
    self.positions.as_chunks::<3>().0
  }

  /// The extent the packed positions span, or `None` when nothing was packed.
  pub fn get_bounds(&self) -> Option<VisualBounds> {
    VisualBounds::from_positions(self.positions.as_chunks::<3>().0.iter().map(|[x, y, z]| Vector3d {
      x: *x,
      y: *y,
      z: *z,
    }))
  }

  /// The extent the positions a run of indices reaches, or `None` when it reaches none.
  pub fn get_indexed_bounds(&self, indices: &[u32]) -> Option<VisualBounds> {
    let positions: &[[f32; 3]] = self.positions.as_chunks::<3>().0;

    VisualBounds::from_positions(
      indices
        .iter()
        .filter_map(move |index| positions.get(*index as usize))
        .map(|[x, y, z]| Vector3d { x: *x, y: *y, z: *z }),
    )
  }

  /// Writes every declared array and the indices into the buffer, and says where each landed.
  pub fn write_into(
    &self,
    indices: &[u32],
    clusters: &VisualClusterTable,
    builder: &mut VisualBufferBuilder,
  ) -> SectorGeometry {
    SectorGeometry {
      vertex_count: self.get_vertex_count(),
      index_count: indices.len() as u32,
      positions: builder.push_f32_section(&self.positions),
      normals: Self::push_bytes(builder, self.attributes.normals, &self.normals),
      tangents: Self::push_bytes(builder, self.attributes.tangents, &self.tangents),
      binormals: Self::push_bytes(builder, self.attributes.binormals, &self.binormals),
      uvs: self.attributes.uvs.then(|| builder.push_i16_section(&self.uvs)),
      uv_components: if self.attributes.uvs { self.uv_components } else { 0 },
      lightmap_uvs: self
        .attributes
        .lightmap_uvs
        .then(|| builder.push_i16_section(&self.lightmap_uvs)),
      colors: Self::push_bytes(builder, self.attributes.colors, &self.colors),
      indices: builder.push_u32_section(indices),
      clusters: clusters.write_into(builder),
    }
  }

  fn push_bytes(builder: &mut VisualBufferBuilder, declared: bool, values: &[u8]) -> Option<VisualSection> {
    declared.then(|| builder.push_u8_section(values))
  }

  /// A direction's four bytes, its z negated into renderer space, or the neutral one where the vertex carries none.
  fn push_direction(values: &mut Vec<u8>, direction: Option<[u8; 4]>) {
    let Some(mut bytes) = direction else {
      values.extend_from_slice(&Self::NEUTRAL_DIRECTION);

      return;
    };

    bytes[LevelVertex::DIRECTION_Z] = u8::MAX - bytes[LevelVertex::DIRECTION_Z];
    values.extend_from_slice(&bytes);
  }

  fn push_vector(values: &mut Vec<f32>, vector: &Vector3d) {
    values.push(vector.x);
    values.push(vector.y);
    values.push(vector.z);
  }
}
