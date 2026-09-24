use std::collections::BTreeMap;

use byteorder::ByteOrder;
use xrf_chunk::ChunkDataSource;
use xrf_error::{XrfError, XrfResult};
use xrf_level::{
  LevelGeomSource, LevelSectorComposition, LevelShadersChunk, LevelVertexLayout, LevelVertexPayload, LevelVisual,
  LevelVisualsChunk,
};
use xrf_ogf::OgfGeometryContainerChunk;

use crate::data::sector::sector_attributes::SectorAttributes;
use crate::data::sector::sector_description::SectorDescription;
use crate::data::sector::sector_geometry::SectorGeometry;
use crate::data::sector::sector_instance_group::SectorInstanceGroup;
use crate::data::sector::sector_section::SectorSection;
use crate::data::sector::sector_skip::SectorSkip;
use crate::data::visual::bounds::visual_bounds::VisualBounds;
use crate::data::visual::geometry::visual_draw_range::VisualDrawRange;
use crate::data::visual::geometry::visual_skip_cause::VisualSkipCause;
use crate::pack::sector::sector_instance_gathering::SectorInstanceGathering;
use crate::pack::sector::sector_instance_key::SectorInstanceKey;
use crate::pack::sector::sector_package::SectorPackage;
use crate::pack::sector::sector_section_gathering::SectorSectionGathering;
use crate::pack::sector::sector_surface_table::SectorSurfaceTable;
use crate::pack::sector::sector_vertex_arrays::SectorVertexArrays;
use crate::pack::sector::sector_vertex_range::SectorVertexRange;
use crate::pack::visual_buffer_builder::VisualBufferBuilder;
use crate::pack::visual_conversion::{convert_placement, convert_tree_hemi, reverse_triangle_winding};

/// Packs one sector's drawables into the single buffer a renderer draws it from.
pub struct SectorPacker<'a, D: ChunkDataSource> {
  visuals: &'a LevelVisualsChunk,
  surfaces: SectorSurfaceTable<'a>,
  source: &'a LevelGeomSource<D>,
}

impl<'a, D: ChunkDataSource> SectorPacker<'a, D> {
  pub fn new(
    visuals: &'a LevelVisualsChunk,
    shaders: Option<&'a LevelShadersChunk>,
    source: &'a LevelGeomSource<D>,
  ) -> Self {
    Self {
      source,
      surfaces: SectorSurfaceTable::of(shaders),
      visuals,
    }
  }

  /// Packs everything one sector reaches into one buffer of attribute arrays and one index array.
  pub fn pack<T: ByteOrder>(
    &self,
    sector: u32,
    composition: &LevelSectorComposition,
    wanted: SectorAttributes,
  ) -> SectorPackage {
    // Baked geometry stores its base coordinate as two shorts; a tree, which stores four, is packed as an instance.
    let mut arrays: SectorVertexArrays = SectorVertexArrays::new(
      self.widen_attributes(composition).intersect(wanted),
      SectorVertexArrays::BAKED_UV_COMPONENTS,
    );
    let mut packed: BTreeMap<SectorVertexRange, u32> = BTreeMap::new();
    let mut sections: BTreeMap<u16, SectorSectionGathering> = BTreeMap::new();
    let mut gathered: BTreeMap<SectorInstanceKey, SectorInstanceGathering> = BTreeMap::new();
    let mut skipped: Vec<SectorSkip> = Vec::new();

    for drawable in &composition.drawables {
      let Some((visual, container)) = Self::get_drawable(self.visuals, *drawable) else {
        continue;
      };

      // A visual the level places is an instance of a mesh rather than geometry of its own: the mesh is packed once
      // below and stood in every place that names it.
      if let Some(tree) = &visual.tree {
        let gathering: &mut SectorInstanceGathering = gathered
          .entry(SectorInstanceKey::of(container, visual.header.shader_id))
          .or_default();

        gathering.drawables.push(*drawable);
        gathering.placements.push(convert_placement(&tree.transform));
        gathering.hemi.push(convert_tree_hemi(tree));

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
          let gathering: &mut SectorSectionGathering = sections.entry(visual.header.shader_id).or_default();

          gathering.drawables.push(*drawable);
          gathering.indices.extend(indices);
        }
        Err(error) => skipped.push(Self::skip(*drawable, &error)),
      }
    }

    self.build::<T>(sector, arrays, sections, gathered, wanted, &mut skipped)
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
    &self,
    container: &OgfGeometryContainerChunk,
    arrays: &mut SectorVertexArrays,
    packed: &mut BTreeMap<SectorVertexRange, u32>,
  ) -> XrfResult<u32> {
    let range: SectorVertexRange = SectorVertexRange::of(container);

    if let Some(base) = packed.get(&range) {
      return Ok(*base);
    }

    let payload: LevelVertexPayload = self.source.read_vertex_payload(
      container.vertex_buffer_id,
      container.vertex_base,
      container.vertex_count,
    )?;
    let base: u32 = arrays.get_vertex_count();

    arrays.push::<T>(&payload)?;
    packed.insert(range, base);

    Ok(base)
  }

  /// Reads a drawable's indices and moves them onto the vertices it was packed at.
  fn read_indices<T: ByteOrder>(&self, container: &OgfGeometryContainerChunk, base: u32) -> XrfResult<Vec<u32>> {
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
    &self,
    sector: u32,
    arrays: SectorVertexArrays,
    gathered_sections: BTreeMap<u16, SectorSectionGathering>,
    gathered: BTreeMap<SectorInstanceKey, SectorInstanceGathering>,
    wanted: SectorAttributes,
    skipped: &mut Vec<SectorSkip>,
  ) -> SectorPackage {
    let mut builder: VisualBufferBuilder = VisualBufferBuilder::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut sections: Vec<SectorSection> = Vec::new();

    for (shader_id, gathering) in gathered_sections {
      sections.push(SectorSection {
        bounds: arrays.get_indexed_bounds(&gathering.indices),
        draw: VisualDrawRange {
          count: gathering.indices.len() as u32,
          start: indices.len() as u32,
        },
        drawables: gathering.drawables,
        surface: self.surfaces.get(shader_id),
      });

      indices.extend(gathering.indices);
    }

    let bounds: Option<VisualBounds> = arrays.get_bounds();
    let geometry: SectorGeometry = arrays.write_into(&indices, &mut builder);
    let instances: Vec<SectorInstanceGroup> = self.pack_instances::<T>(gathered, wanted, &mut builder, skipped);

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

  /// Packs each gathered mesh once and writes the places it stands beside it.
  fn pack_instances<T: ByteOrder>(
    &self,
    gathered: BTreeMap<SectorInstanceKey, SectorInstanceGathering>,
    wanted: SectorAttributes,
    builder: &mut VisualBufferBuilder,
    skipped: &mut Vec<SectorSkip>,
  ) -> Vec<SectorInstanceGroup> {
    let mut instances: Vec<SectorInstanceGroup> = Vec::new();

    for (key, gathering) in gathered {
      match self.pack_instance::<T>(key, &gathering, wanted, builder) {
        Ok(group) => instances.push(group),
        Err(error) => Self::skip_all(&gathering, &error, skipped),
      }
    }

    instances
  }

  /// Packs one mesh, in its own space and its own declaration, beside the transforms that stand it.
  ///
  /// # Errors
  ///
  /// Returns an error when the mesh's own range cannot be read, which leaves every instance of it undrawable.
  fn pack_instance<T: ByteOrder>(
    &self,
    key: SectorInstanceKey,
    gathering: &SectorInstanceGathering,
    wanted: SectorAttributes,
    builder: &mut VisualBufferBuilder,
  ) -> XrfResult<SectorInstanceGroup> {
    let payload: LevelVertexPayload =
      self
        .source
        .read_vertex_payload(key.vertices.buffer, key.vertices.base, key.vertices.count)?;
    let mut arrays: SectorVertexArrays = SectorVertexArrays::new(
      SectorAttributes::of(&payload.layout).intersect(wanted),
      SectorVertexArrays::uv_components_of(&payload.layout),
    );

    // Packed unplaced: the mesh is in its own space, and each instance's transform stands a copy of it.
    arrays.push::<T>(&payload)?;

    let mut indices: Vec<u32> = self
      .source
      .read_indices::<T>(key.index_buffer, key.index_base, key.index_count)?
      .iter()
      .map(|index| u32::from(*index))
      .collect();

    reverse_triangle_winding(&mut indices);

    let transforms: Vec<f32> = gathering
      .placements
      .iter()
      .flat_map(|placement| placement.values)
      .collect();

    let hemi: Vec<f32> = gathering.hemi.iter().flatten().copied().collect();

    Ok(SectorInstanceGroup {
      drawables: gathering.drawables.clone(),
      geometry: arrays.write_into(&indices, builder),
      hemi: builder.push_f32_section(&hemi),
      instance_count: gathering.placements.len() as u32,
      surface: self.surfaces.get(key.shader_id),
      transforms: builder.push_f32_section(&transforms),
    })
  }

  /// Records every instance of a mesh that could not be read, since none of them can be drawn without it.
  fn skip_all(gathering: &SectorInstanceGathering, error: &XrfError, skipped: &mut Vec<SectorSkip>) {
    for drawable in &gathering.drawables {
      skipped.push(Self::skip(*drawable, error));
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
