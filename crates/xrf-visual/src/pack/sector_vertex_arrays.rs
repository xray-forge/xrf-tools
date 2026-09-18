use xrf_level::LevelVertex;
use xrf_math::Matrix4x4;
use xrf_math::Vector3d;

use crate::data::sector_attributes::SectorAttributes;
use crate::data::visual_bounds::VisualBounds;
use crate::data::visual_section::VisualSection;
use crate::pack::sector_vertex_sections::SectorVertexSections;
use crate::pack::visual_buffer_builder::VisualBufferBuilder;
use crate::pack::visual_conversion::{convert_texture_coordinates, convert_vector};

/// The attribute arrays of a sector under construction, one entry per packed vertex.
#[derive(Debug)]
pub(crate) struct SectorVertexArrays {
  attributes: SectorAttributes,
  positions: Vec<f32>,
  normals: Vec<f32>,
  tangents: Vec<f32>,
  binormals: Vec<f32>,
  texture_coordinates: Vec<f32>,
  lightmap_coordinates: Vec<f32>,
  colors: Vec<f32>,
  hemi: Vec<f32>,
}

impl SectorVertexArrays {
  /// A byte the engine packs a normalized value into, which is how a colour and the hemisphere term arrive.
  const BYTE_SCALE: f32 = 255.0;

  pub fn new(attributes: SectorAttributes) -> Self {
    Self {
      attributes,
      binormals: Vec::new(),
      colors: Vec::new(),
      hemi: Vec::new(),
      lightmap_coordinates: Vec::new(),
      normals: Vec::new(),
      positions: Vec::new(),
      tangents: Vec::new(),
      texture_coordinates: Vec::new(),
    }
  }

  /// What the sector declared, so a mesh packed on its own packs the same attributes as the sector around it.
  pub const fn get_attributes(&self) -> SectorAttributes {
    self.attributes
  }

  /// Vertices packed so far, which is the base the next range is rebased onto.
  pub fn count(&self) -> u32 {
    (self.positions.len() / 3) as u32
  }

  /// Appends one decoded vertex, placed and converted into renderer space.
  pub fn push(&mut self, vertex: &LevelVertex, placement: Option<&Matrix4x4>) {
    Self::push_vector(
      &mut self.positions,
      &convert_vector(&Self::place(&vertex.position, placement)),
    );

    if self.attributes.normals {
      Self::push_vector(
        &mut self.normals,
        &convert_vector(&Self::turn(&vertex.normal, placement)),
      );
      self.hemi.push(f32::from(vertex.hemi) / Self::BYTE_SCALE);
    }

    if self.attributes.tangents {
      Self::push_direction(&mut self.tangents, vertex.tangent.as_ref(), placement);
    }

    if self.attributes.binormals {
      Self::push_direction(&mut self.binormals, vertex.binormal.as_ref(), placement);
    }

    if self.attributes.texture_coordinates {
      let (u, v): (f32, f32) = convert_texture_coordinates(vertex.texture_coordinate.0, vertex.texture_coordinate.1);

      self.texture_coordinates.push(u);
      self.texture_coordinates.push(v);
    }

    if self.attributes.lightmap_coordinates {
      let (u, v): (f32, f32) = vertex.lightmap_coordinate.unwrap_or((0.0, 0.0));

      self.lightmap_coordinates.push(u);
      self.lightmap_coordinates.push(v);
    }

    if self.attributes.colors {
      let (red, green, blue): (u8, u8, u8) = vertex.color.unwrap_or((0, 0, 0));

      for component in [red, green, blue] {
        self.colors.push(f32::from(component) / Self::BYTE_SCALE);
      }
    }
  }

  /// The extent the packed positions span, or `None` when nothing was packed.
  pub fn get_bounds(&self) -> Option<VisualBounds> {
    let positions: Vec<Vector3d> = self
      .positions
      .as_chunks::<3>()
      .0
      .iter()
      .map(|[x, y, z]| Vector3d { x: *x, y: *y, z: *z })
      .collect();

    VisualBounds::from_positions(&positions)
  }

  /// Writes every declared array into the buffer and says where each landed.
  pub fn write_into(&self, builder: &mut VisualBufferBuilder) -> SectorVertexSections {
    SectorVertexSections {
      positions: builder.push_f32_section(&self.positions),
      normals: self.push_declared(builder, self.attributes.normals, &self.normals),
      tangents: self.push_declared(builder, self.attributes.tangents, &self.tangents),
      binormals: self.push_declared(builder, self.attributes.binormals, &self.binormals),
      texture_coordinates: self.push_declared(builder, self.attributes.texture_coordinates, &self.texture_coordinates),
      lightmap_coordinates: self.push_declared(
        builder,
        self.attributes.lightmap_coordinates,
        &self.lightmap_coordinates,
      ),
      colors: self.push_declared(builder, self.attributes.colors, &self.colors),
      hemi: self.push_declared(builder, self.attributes.normals, &self.hemi),
    }
  }

  fn push_declared(&self, builder: &mut VisualBufferBuilder, declared: bool, values: &[f32]) -> Option<VisualSection> {
    declared.then(|| builder.push_f32_section(values))
  }

  /// A direction the range declared but this vertex may not carry, which packs as a zero rather than a gap.
  fn push_direction(values: &mut Vec<f32>, direction: Option<&Vector3d>, placement: Option<&Matrix4x4>) {
    match direction {
      Some(direction) => Self::push_vector(values, &convert_vector(&Self::turn(direction, placement))),
      None => values.extend_from_slice(&[0.0, 0.0, 0.0]),
    }
  }

  /// One position, put where the level puts it.
  fn place(position: &Vector3d, placement: Option<&Matrix4x4>) -> Vector3d {
    match placement {
      Some(transform) => transform.transform_point(position),
      None => position.clone(),
    }
  }

  /// One direction, turned the way the placement turns it, translation left out of it.
  fn turn(direction: &Vector3d, placement: Option<&Matrix4x4>) -> Vector3d {
    match placement {
      Some(transform) => transform.transform_direction(direction),
      None => direction.clone(),
    }
  }

  fn push_vector(values: &mut Vec<f32>, vector: &Vector3d) {
    values.push(vector.x);
    values.push(vector.y);
    values.push(vector.z);
  }
}
