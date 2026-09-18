use std::collections::BTreeMap;

use byteorder::ByteOrder;
use xrf_chunk::ChunkDataSource;
use xrf_error::{XrfError, XrfResult};
use xrf_level::{
  LevelGeomSource, LevelSectorComposition, LevelShaderEntry, LevelShadersChunk, LevelVertex, LevelVertexLayout,
  LevelVisual, LevelVisualsChunk,
};
use xrf_ogf::OgfGeometryContainerChunk;

use crate::data::sector_attributes::SectorAttributes;
use crate::data::sector_description::SectorDescription;
use crate::data::sector_section::SectorSection;
use crate::data::sector_skip::SectorSkip;
use crate::data::visual_section::{VisualDrawRange, VisualSection};
use crate::data::visual_submesh::VisualSkipCause;
use crate::pack::sector_package::SectorPackage;
use crate::pack::sector_vertex_arrays::SectorVertexArrays;
use crate::pack::sector_vertex_sections::SectorVertexSections;
use crate::pack::visual_buffer_builder::VisualBufferBuilder;
use crate::pack::visual_conversion::reverse_triangle_winding;

/// The range of `level.geom` one drawable names, which is the key two drawables share geometry by.
type VertexRange = (u32, u32, u32);

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
    let mut groups: BTreeMap<u16, (Vec<u32>, Vec<u32>)> = BTreeMap::new();
    let mut skipped: Vec<SectorSkip> = Vec::new();
    let mut shared_ranges: u32 = 0;
    let mut unshared_vertex_count: u32 = 0;

    for drawable in &composition.drawables {
      let Some((visual, container)) = Self::get_drawable(self.visuals, *drawable) else {
        continue;
      };

      unshared_vertex_count = unshared_vertex_count.saturating_add(container.vertex_count);

      let base: u32 = match self.pack_range::<T>(container, &mut arrays, &mut packed) {
        Ok((base, reused)) => {
          shared_ranges += u32::from(reused);

          base
        }
        Err(error) => {
          skipped.push(Self::skip(*drawable, &error));

          continue;
        }
      };

      match self.read_indices::<T>(container, base) {
        Ok(indices) => {
          let (drawables, packed_indices) = groups.entry(visual.header.shader_id).or_default();

          drawables.push(*drawable);
          packed_indices.extend(indices);
        }
        Err(error) => skipped.push(Self::skip(*drawable, &error)),
      }
    }

    self.build(sector, arrays, groups, skipped, shared_ranges, unshared_vertex_count)
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

  /// Packs the vertex range a drawable names, unless another drawable already did, and says which happened.
  fn pack_range<T: ByteOrder>(
    &mut self,
    container: &OgfGeometryContainerChunk,
    arrays: &mut SectorVertexArrays,
    packed: &mut BTreeMap<VertexRange, u32>,
  ) -> XrfResult<(u32, bool)> {
    let range: VertexRange = (
      container.vertex_buffer_id,
      container.vertex_base,
      container.vertex_count,
    );

    if let Some(base) = packed.get(&range) {
      return Ok((*base, true));
    }

    let vertices: Vec<LevelVertex> = self.source.read_vertices::<T>(
      container.vertex_buffer_id,
      container.vertex_base,
      container.vertex_count,
    )?;
    let base: u32 = arrays.count();

    for vertex in &vertices {
      arrays.push(vertex);
    }

    packed.insert(range, base);

    Ok((base, false))
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
  fn build(
    &self,
    sector: u32,
    arrays: SectorVertexArrays,
    groups: BTreeMap<u16, (Vec<u32>, Vec<u32>)>,
    skipped: Vec<SectorSkip>,
    shared_ranges: u32,
    unshared_vertex_count: u32,
  ) -> SectorPackage {
    let mut builder: VisualBufferBuilder = VisualBufferBuilder::new();
    let vertices: SectorVertexSections = arrays.write_into(&mut builder);

    let mut indices: Vec<u32> = Vec::new();
    let mut sections: Vec<SectorSection> = Vec::new();

    for (shader_id, (drawables, group)) in groups {
      sections.push(SectorSection {
        draw: VisualDrawRange {
          count: group.len() as u32,
          start: indices.len() as u32,
        },
        drawables,
        shader_id,
        shader_name: self.get_shader_name(shader_id),
        texture_name: self.get_texture_name(shader_id),
      });

      indices.extend(group);
    }

    let index_section: VisualSection = builder.push_u32_section(&indices);

    SectorPackage {
      description: SectorDescription {
        binormals: vertices.binormals,
        bounds: arrays.get_bounds(),
        buffer_length: builder.length(),
        colors: vertices.colors,
        hemi: vertices.hemi,
        index_count: indices.len() as u32,
        indices: index_section,
        lightmap_coordinates: vertices.lightmap_coordinates,
        normals: vertices.normals,
        positions: vertices.positions,
        sections,
        sector,
        shared_ranges,
        skipped,
        tangents: vertices.tangents,
        texture_coordinates: vertices.texture_coordinates,
        unshared_vertex_count,
        vertex_count: arrays.count(),
      },
      buffer: builder.into_buffer(),
    }
  }

  /// The shader a table entry names, when the level carries a table that resolves.
  fn get_shader_name(&self, shader_id: u16) -> Option<String> {
    match self.get_entry(shader_id)? {
      LevelShaderEntry::Reference(reference) => Some(reference.shader.clone()),
      _ => None,
    }
  }

  /// The first texture a table entry names, which is the one a surface is dressed with.
  fn get_texture_name(&self, shader_id: u16) -> Option<String> {
    match self.get_entry(shader_id)? {
      LevelShaderEntry::Reference(reference) => reference.textures.first().cloned(),
      _ => None,
    }
  }

  fn get_entry(&self, shader_id: u16) -> Option<&LevelShaderEntry> {
    self.shaders?.entries.get(shader_id as usize)
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
