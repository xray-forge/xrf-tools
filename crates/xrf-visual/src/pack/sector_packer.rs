use std::collections::BTreeMap;

use byteorder::ByteOrder;
use xrf_chunk::ChunkDataSource;
use xrf_error::{XrfError, XrfResult};
use xrf_level::{
  LevelGeomSource, LevelSectorComposition, LevelShaderEntry, LevelShadersChunk, LevelVertex, LevelVertexLayout,
  LevelVisual, LevelVisualsChunk,
};
use xrf_math::Matrix4x4;
use xrf_ogf::OgfGeometryContainerChunk;

use crate::pack::sector_attributes::SectorAttributes;
use crate::data::sector_description::SectorDescription;
use crate::data::sector_geometry::SectorGeometry;
use crate::data::sector_instance_group::SectorInstanceGroup;
use crate::data::sector_section::SectorSection;
use crate::data::sector_skip::SectorSkip;
use crate::data::sector_surface::SectorSurface;
use crate::data::visual_bounds::VisualBounds;
use crate::data::visual_section::VisualDrawRange;
use crate::data::visual_submesh::VisualSkipCause;
use crate::pack::sector_package::SectorPackage;
use crate::pack::sector_vertex_arrays::SectorVertexArrays;
use crate::pack::visual_buffer_builder::VisualBufferBuilder;
use crate::pack::visual_conversion::{convert_placement, reverse_triangle_winding};

/// The range of `level.geom` one drawable names, which is the key two drawables share geometry by.
type VertexRange = (u32, u32, u32);

/// One instanced mesh: the geometry it draws and the surface it is dressed by, which is what makes two copies one.
type InstanceKey = (u32, u32, u32, u32, u32, u32, u16);

/// The drawables of one surface, gathered while the sector is walked.
#[derive(Default)]
struct SectionGathering {
  drawables: Vec<u32>,
  /// Their indices, already rebased onto where each range was packed.
  indices: Vec<u32>,
}

/// The instances of one mesh, gathered the same way.
#[derive(Default)]
struct InstanceGathering {
  drawables: Vec<u32>,
  placements: Vec<Matrix4x4>,
}

/// Packs one sector's drawables into the single buffer a renderer draws it from.
pub struct SectorPacker<'a, D: ChunkDataSource> {
  visuals: &'a LevelVisualsChunk,
  shaders: Option<&'a LevelShadersChunk>,
  source: &'a mut LevelGeomSource<D>,
}

impl<'a, D: ChunkDataSource> SectorPacker<'a, D> {
  pub fn new(
    visuals: &'a LevelVisualsChunk,
    shaders: Option<&'a LevelShadersChunk>,
    source: &'a mut LevelGeomSource<D>,
  ) -> Self {
    Self {
      shaders,
      source,
      visuals,
    }
  }

  /// Packs everything one sector reaches into one buffer of attribute arrays and one index array.
  pub fn pack<T: ByteOrder>(&mut self, sector: u32, composition: &LevelSectorComposition) -> SectorPackage {
    let mut arrays: SectorVertexArrays = SectorVertexArrays::new(self.widen_attributes(composition));
    let mut packed: BTreeMap<VertexRange, u32> = BTreeMap::new();
    let mut sections: BTreeMap<u16, SectionGathering> = BTreeMap::new();
    let mut gathered: BTreeMap<InstanceKey, InstanceGathering> = BTreeMap::new();
    let mut skipped: Vec<SectorSkip> = Vec::new();

    for drawable in &composition.drawables {
      let Some((visual, container)) = Self::get_drawable(self.visuals, *drawable) else {
        continue;
      };

      // A visual the level places is an instance of a mesh rather than geometry of its own: the mesh is packed once
      // below and stood in every place that names it.
      if let Some(placement) = visual.get_placement() {
        let gathering: &mut InstanceGathering = gathered
          .entry(Self::instance_key(container, visual.header.shader_id))
          .or_default();

        gathering.drawables.push(*drawable);
        gathering.placements.push(convert_placement(placement));

        continue;
      }

      let base: u32 = match self.pack_range::<T>(container, &mut arrays, &mut packed) {
        Ok(base) => base,
        Err(error) => {
          skipped.push(Self::skip(*drawable, &error));

          continue;
        }
      };

      match self.read_indices::<T>(container, base) {
        Ok(indices) => {
          let gathering: &mut SectionGathering = sections.entry(visual.header.shader_id).or_default();

          gathering.drawables.push(*drawable);
          gathering.indices.extend(indices);
        }
        Err(error) => skipped.push(Self::skip(*drawable, &error)),
      }
    }

    self.build::<T>(sector, arrays, sections, gathered, &mut skipped)
  }

  /// What every declaration in the sector together carries, which decides the arrays it packs.
  fn widen_attributes(&self, composition: &LevelSectorComposition) -> SectorAttributes {
    let mut attributes: SectorAttributes = SectorAttributes::default();

    for drawable in &composition.drawables {
      let Some((_, container)) = Self::get_drawable(self.visuals, *drawable) else {
        continue;
      };

      let Some(declared) = self
        .source
        .get_file()
        .vertex_buffers
        .get(container.vertex_buffer_id as usize)
      else {
        continue;
      };

      if let Ok(layout) = LevelVertexLayout::of(declared) {
        attributes.widen(&layout);
      }
    }

    attributes
  }

  /// One drawable of the run, with the container that makes it drawable.
  fn get_drawable(
    visuals: &'a LevelVisualsChunk,
    drawable: u32,
  ) -> Option<(&'a LevelVisual, &'a OgfGeometryContainerChunk)> {
    let visual: &'a LevelVisual = visuals.visuals.get(drawable as usize)?;

    visual.geometry.as_ref().map(|container| (visual, container))
  }

  /// Packs the vertex range a drawable names, unless another drawable already did, and says where it sits.
  fn pack_range<T: ByteOrder>(
    &mut self,
    container: &OgfGeometryContainerChunk,
    arrays: &mut SectorVertexArrays,
    packed: &mut BTreeMap<VertexRange, u32>,
  ) -> XrfResult<u32> {
    let range: VertexRange = (
      container.vertex_buffer_id,
      container.vertex_base,
      container.vertex_count,
    );

    if let Some(base) = packed.get(&range) {
      return Ok(*base);
    }

    let vertices: Vec<LevelVertex> = self.source.read_vertices::<T>(
      container.vertex_buffer_id,
      container.vertex_base,
      container.vertex_count,
    )?;
    let base: u32 = arrays.get_vertex_count();

    for vertex in &vertices {
      arrays.push(vertex, None);
    }

    packed.insert(range, base);

    Ok(base)
  }

  /// Reads a drawable's indices and moves them onto the vertices it was packed at.
  fn read_indices<T: ByteOrder>(&mut self, container: &OgfGeometryContainerChunk, base: u32) -> XrfResult<Vec<u32>> {
    let indices: Vec<u16> =
      self
        .source
        .read_indices::<T>(container.index_buffer_id, container.index_base, container.index_count)?;

    if let Some(stray) = indices
      .iter()
      .find(|index| u32::from(**index) >= container.vertex_count)
    {
      return Err(XrfError::new_invalid_error(format!(
        "draws index {stray}, past the {} vertices it declares",
        container.vertex_count
      )));
    }

    let mut rebased: Vec<u32> = indices.iter().map(|index| base + u32::from(*index)).collect();

    reverse_triangle_winding(&mut rebased);

    Ok(rebased)
  }

  /// Writes the arrays and the grouped indices into one buffer and describes what landed where.
  fn build<T: ByteOrder>(
    &mut self,
    sector: u32,
    arrays: SectorVertexArrays,
    gathered_sections: BTreeMap<u16, SectionGathering>,
    gathered: BTreeMap<InstanceKey, InstanceGathering>,
    skipped: &mut Vec<SectorSkip>,
  ) -> SectorPackage {
    let mut builder: VisualBufferBuilder = VisualBufferBuilder::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut sections: Vec<SectorSection> = Vec::new();

    for (shader_id, gathering) in gathered_sections {
      sections.push(SectorSection {
        draw: VisualDrawRange {
          count: gathering.indices.len() as u32,
          start: indices.len() as u32,
        },
        drawables: gathering.drawables,
        surface: self.get_surface(shader_id),
      });

      indices.extend(gathering.indices);
    }

    let bounds: Option<VisualBounds> = arrays.get_bounds();
    let attributes: SectorAttributes = arrays.get_attributes();
    let geometry: SectorGeometry = arrays.write_into(&indices, &mut builder);
    let instances: Vec<SectorInstanceGroup> = self.pack_instances::<T>(gathered, attributes, &mut builder, skipped);

    SectorPackage {
      description: SectorDescription {
        bounds,
        buffer_length: builder.length(),
        geometry,
        instances,
        sections,
        sector,
        skipped: std::mem::take(skipped),
      },
      buffer: builder.into_buffer(),
    }
  }

  /// The key two copies of one mesh are the same instance by: the geometry, and the surface drawing it.
  fn instance_key(container: &OgfGeometryContainerChunk, shader_id: u16) -> InstanceKey {
    (
      container.vertex_buffer_id,
      container.vertex_base,
      container.vertex_count,
      container.index_buffer_id,
      container.index_base,
      container.index_count,
      shader_id,
    )
  }

  /// Packs each gathered mesh once and writes the places it stands beside it.
  fn pack_instances<T: ByteOrder>(
    &mut self,
    gathered: BTreeMap<InstanceKey, InstanceGathering>,
    attributes: SectorAttributes,
    builder: &mut VisualBufferBuilder,
    skipped: &mut Vec<SectorSkip>,
  ) -> Vec<SectorInstanceGroup> {
    let mut instances: Vec<SectorInstanceGroup> = Vec::new();

    for (key, gathering) in gathered {
      match self.pack_instance::<T>(key, &gathering, attributes, builder) {
        Ok(group) => instances.push(group),
        Err(error) => Self::skip_all(&gathering, &error, skipped),
      }
    }

    instances
  }

  /// Packs one mesh, in its own space, beside the transforms that stand it.
  ///
  /// # Errors
  ///
  /// Returns an error when the mesh's own range cannot be read, which leaves every instance of it undrawable.
  fn pack_instance<T: ByteOrder>(
    &mut self,
    key: InstanceKey,
    gathering: &InstanceGathering,
    attributes: SectorAttributes,
    builder: &mut VisualBufferBuilder,
  ) -> XrfResult<SectorInstanceGroup> {
    let (vertex_buffer, vertex_base, vertex_count, index_buffer, index_base, index_count, shader_id) = key;
    let mut arrays: SectorVertexArrays = SectorVertexArrays::new(attributes);

    // Packed unplaced: the mesh is in its own space, and each instance's transform stands a copy of it.
    for vertex in &self
      .source
      .read_vertices::<T>(vertex_buffer, vertex_base, vertex_count)?
    {
      arrays.push(vertex, None);
    }

    let mut indices: Vec<u32> = self
      .source
      .read_indices::<T>(index_buffer, index_base, index_count)?
      .iter()
      .map(|index| u32::from(*index))
      .collect();

    reverse_triangle_winding(&mut indices);

    let transforms: Vec<f32> = gathering
      .placements
      .iter()
      .flat_map(|placement| placement.values)
      .collect();

    Ok(SectorInstanceGroup {
      drawables: gathering.drawables.clone(),
      geometry: arrays.write_into(&indices, builder),
      instance_count: gathering.placements.len() as u32,
      surface: self.get_surface(shader_id),
      transforms: builder.push_f32_section(&transforms),
    })
  }

  /// Records every instance of a mesh that could not be read, since none of them can be drawn without it.
  fn skip_all(gathering: &InstanceGathering, error: &XrfError, skipped: &mut Vec<SectorSkip>) {
    for drawable in &gathering.drawables {
      skipped.push(Self::skip(*drawable, error));
    }
  }

  /// How one table entry dresses a surface: its shader, its base texture, and the lightmaps after it.
  fn get_surface(&self, shader_id: u16) -> SectorSurface {
    let Some(LevelShaderEntry::Reference(reference)) = self.shaders.and_then(|it| it.entries.get(shader_id as usize))
    else {
      return SectorSurface {
        shader_id,
        ..SectorSurface::default()
      };
    };

    SectorSurface {
      lightmaps: reference.textures.iter().skip(1).cloned().collect(),
      shader_id,
      shader_name: Some(reference.shader.clone()),
      texture_name: reference.textures.first().cloned(),
    }
  }

  /// Grades a drawable that produced nothing by whether the geometry is stored in a form the reader handles.
  fn skip(drawable: u32, error: &XrfError) -> SectorSkip {
    SectorSkip {
      cause: match error {
        XrfError::NotImplemented { .. } => VisualSkipCause::Unsupported,
        _ => VisualSkipCause::Malformed,
      },
      drawable,
      reason: error.to_string(),
    }
  }
}
