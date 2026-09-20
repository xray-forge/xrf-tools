use xrf_level::LevelVertex;
use xrf_math::Vector3d;

use crate::data::sector_attributes::SectorAttributes;
use crate::data::sector_geometry::SectorGeometry;
use crate::data::visual_bounds::VisualBounds;
use crate::data::visual_section::VisualSection;
use crate::pack::visual_buffer_builder::VisualBufferBuilder;
use crate::pack::visual_conversion::{convert_uvs, convert_vector};

/// The attribute arrays of a sector under construction, one entry per packed vertex.
#[derive(Debug)]
pub(crate) struct SectorVertexArrays {
  attributes: SectorAttributes,
  positions: Vec<f32>,
  normals: Vec<f32>,
  tangents: Vec<f32>,
  binormals: Vec<f32>,
  uvs: Vec<f32>,
  lightmap_uvs: Vec<f32>,
  colors: Vec<f32>,
  hemi: Vec<f32>,
}

impl SectorVertexArrays {
  /// A byte the engine packs a normalized value into, which is how a colour and the hemisphere term arrive.
  const BYTE_SCALE: f32 = 255.0;

  /// What a vertex carrying no baked colour contributes, which is nothing.
  const UNLIT_COLOR: (u8, u8, u8) = (255, 255, 255);

  pub fn new(attributes: SectorAttributes) -> Self {
    Self {
      attributes,
      binormals: Vec::new(),
      colors: Vec::new(),
      hemi: Vec::new(),
      lightmap_uvs: Vec::new(),
      normals: Vec::new(),
      positions: Vec::new(),
      tangents: Vec::new(),
      uvs: Vec::new(),
    }
  }

  /// What the sector declared, so a mesh packed on its own packs the same attributes as the sector around it.
  pub const fn get_attributes(&self) -> SectorAttributes {
    self.attributes
  }

  /// Vertices packed so far, which is the base the next range is rebased onto.
  pub fn get_vertex_count(&self) -> u32 {
    (self.positions.len() / 3) as u32
  }

  /// Appends one decoded vertex, converted into renderer space.
  ///
  /// Unplaced, and there is nothing to place it by: a sector's own geometry is already in the level's space, and a
  /// visual the level places is packed once in its own space and stood by the transforms of its instance group.
  pub fn push(&mut self, vertex: &LevelVertex) {
    Self::push_vector(&mut self.positions, &convert_vector(&vertex.position));

    if self.attributes.normals {
      Self::push_vector(&mut self.normals, &convert_vector(&vertex.normal));
    }

    if self.attributes.hemi {
      self.hemi.push(f32::from(vertex.hemi) / Self::BYTE_SCALE);
    }

    if self.attributes.tangents {
      Self::push_direction(&mut self.tangents, vertex.tangent.as_ref());
    }

    if self.attributes.binormals {
      Self::push_direction(&mut self.binormals, vertex.binormal.as_ref());
    }

    if self.attributes.uvs {
      let (u, v): (f32, f32) = convert_uvs(vertex.texture_coordinate.0, vertex.texture_coordinate.1);

      self.uvs.push(u);
      self.uvs.push(v);
    }

    if self.attributes.lightmap_uvs {
      let (u, v): (f32, f32) = vertex.lightmap_coordinate.unwrap_or((0.0, 0.0));

      self.lightmap_uvs.push(u);
      self.lightmap_uvs.push(v);
    }

    if self.attributes.colors {
      let (red, green, blue): (u8, u8, u8) = vertex.color.unwrap_or(Self::UNLIT_COLOR);

      for component in [red, green, blue] {
        self.colors.push(f32::from(component) / Self::BYTE_SCALE);
      }
    }
  }

  /// The extent the packed positions span, or `None` when nothing was packed.
  pub fn get_bounds(&self) -> Option<VisualBounds> {
    VisualBounds::from_positions(self.positions.as_chunks::<3>().0.iter().map(|[x, y, z]| Vector3d {
      x: *x,
      y: *y,
      z: *z,
    }))
  }

  /// Writes every declared array and the indices into the buffer, and says where each landed.
  pub fn write_into(&self, indices: &[u32], builder: &mut VisualBufferBuilder) -> SectorGeometry {
    SectorGeometry {
      vertex_count: self.get_vertex_count(),
      index_count: indices.len() as u32,
      positions: builder.push_f32_section(&self.positions),
      normals: self.push_declared(builder, self.attributes.normals, &self.normals),
      tangents: self.push_declared(builder, self.attributes.tangents, &self.tangents),
      binormals: self.push_declared(builder, self.attributes.binormals, &self.binormals),
      uvs: self.push_declared(builder, self.attributes.uvs, &self.uvs),
      lightmap_uvs: self.push_declared(builder, self.attributes.lightmap_uvs, &self.lightmap_uvs),
      colors: self.push_declared(builder, self.attributes.colors, &self.colors),
      hemi: self.push_declared(builder, self.attributes.hemi, &self.hemi),
      indices: builder.push_u32_section(indices),
    }
  }

  fn push_declared(&self, builder: &mut VisualBufferBuilder, declared: bool, values: &[f32]) -> Option<VisualSection> {
    declared.then(|| builder.push_f32_section(values))
  }

  /// A direction the range declared but this vertex may not carry, which packs as a zero rather than a gap.
  fn push_direction(values: &mut Vec<f32>, direction: Option<&Vector3d>) {
    match direction {
      Some(direction) => Self::push_vector(values, &convert_vector(direction)),
      None => values.extend_from_slice(&[0.0, 0.0, 0.0]),
    }
  }

  fn push_vector(values: &mut Vec<f32>, vector: &Vector3d) {
    values.push(vector.x);
    values.push(vector.y);
    values.push(vector.z);
  }
}
