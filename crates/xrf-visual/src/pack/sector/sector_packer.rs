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
use crate::data::sector::sector_progressive::SectorProgressive;
use crate::data::sector::sector_section::SectorSection;
use crate::data::sector::sector_skip::SectorSkip;
use crate::data::visual::bounds::visual_bounds::VisualBounds;
use crate::data::visual::geometry::visual_draw_range::VisualDrawRange;
use crate::data::visual::geometry::visual_section::VisualSection;
use crate::data::visual::geometry::visual_skip_cause::VisualSkipCause;
use crate::pack::sector::sector_gathering::SectorGathering;
use crate::pack::sector::sector_impostor_arrays::SectorImpostorArrays;
use crate::pack::sector::sector_instance_gathering::SectorInstanceGathering;
use crate::pack::sector::sector_instance_key::SectorInstanceKey;
use crate::pack::sector::sector_package::SectorPackage;
use crate::pack::sector::sector_section_gathering::SectorSectionGathering;
use crate::pack::sector::sector_surface_table::SectorSurfaceTable;
use crate::pack::sector::sector_vertex_arrays::SectorVertexArrays;
use crate::pack::sector::sector_vertex_range::SectorVertexRange;
use crate::pack::sector::sector_window::SectorWindow;
use crate::pack::visual_buffer_builder::VisualBufferBuilder;
use crate::pack::visual_conversion::{convert_placement, convert_tree_hemi, reverse_triangle_winding};

/// Packs one sector's drawables into the single buffer a renderer draws it from.
pub struct SectorPacker<'a, D: ChunkDataSource> {
  visuals: &'a LevelVisualsChunk,
  surfaces: SectorSurfaceTable<'a>,
  source: &'a LevelGeomSource<D>,
}

impl<'a, D: ChunkDataSource> SectorPacker<'a, D> {
  /// What a place names for its impostor where no `MT_LOD` visual composes it.
  const NO_IMPOSTOR: i32 = -1;

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
    let (impostors, parents): (SectorImpostorArrays, BTreeMap<u32, u32>) = self.gather_impostors(composition);

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

        if gathering.drawables.is_empty() {
          gathering.windows = self.get_windows(visual);
        }

        gathering.drawables.push(*drawable);
        gathering.placements.push(convert_placement(&tree.transform));
        gathering.hemi.push(convert_tree_hemi(tree));
        gathering
          .impostors
          .push(parents.get(drawable).map_or(Self::NO_IMPOSTOR, |index| *index as i32));

        continue;
      }

      let base: u32 = match self.pack_range::<T>(container, &mut arrays, &mut packed) {
        Ok(base) => base,
        Err(error) => {
          skipped.push(Self::skip(*drawable, &error));

          continue;
        }
      };

      // A progressive mesh's indices are every window laid end to end; baked into a section it draws the whole detail,
      // its first window, since a section of many visuals has one range.
      let window: SectorWindow = self
        .get_windows(visual)
        .and_then(|windows| windows.first().copied())
        .unwrap_or(SectorWindow {
          offset: 0,
          triangles: container.index_count / 3,
        });

      match self.read_indices::<T>(container, base, window) {
        Ok(indices) => {
          let gathering: &mut SectorSectionGathering = sections.entry(visual.header.shader_id).or_default();

          gathering.drawables.push(*drawable);
          gathering.indices.extend(indices);
        }
        Err(error) => skipped.push(Self::skip(*drawable, &error)),
      }
    }

    let gathering: SectorGathering = SectorGathering {
      arrays,
      impostors,
      instances: gathered,
      sections,
      skipped,
    };

    self.build::<T>(sector, gathering, wanted)
  }

  /// The impostors of the sector's `MT_LOD` visuals, a surface's contiguous, and which impostor each tree they compose
  /// belongs to, by the tree's index in the visuals run.
  fn gather_impostors(&self, composition: &LevelSectorComposition) -> (SectorImpostorArrays, BTreeMap<u32, u32>) {
    let mut lods: Vec<(u16, u32)> = composition
      .hierarchies
      .iter()
      .filter_map(|index| {
        let visual: &LevelVisual = self.visuals.visuals.get(*index as usize)?;

        visual.lod.as_ref().map(|_| (visual.header.shader_id, *index))
      })
      .collect();
    let mut arrays: SectorImpostorArrays = SectorImpostorArrays::default();
    let mut parents: BTreeMap<u32, u32> = BTreeMap::new();

    lods.sort_unstable();

    for (_, index) in lods {
      let visual: &LevelVisual = &self.visuals.visuals[index as usize];

      for child in &visual.children {
        parents.insert(*child, arrays.len());
      }

      arrays.push(
        visual,
        visual.lod.as_ref().expect("gathered for its impostor"),
        &self.surfaces,
      );
    }

    (arrays, parents)
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

  /// Reads the window of a drawable's indices it draws and moves them onto the vertices it was packed at.
  fn read_indices<T: ByteOrder>(
    &self,
    container: &OgfGeometryContainerChunk,
    base: u32,
    window: SectorWindow,
  ) -> XrfResult<Vec<u32>> {
    Self::check_windows(container.index_count, &[window])?;

    let indices: Vec<u16> = self.source.read_indices::<T>(
      container.index_buffer_id,
      container.index_base + window.offset,
      window.get_index_count(),
    )?;

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
  fn build<T: ByteOrder>(&self, sector: u32, gathering: SectorGathering, wanted: SectorAttributes) -> SectorPackage {
    let SectorGathering {
      arrays,
      sections: gathered_sections,
      instances: gathered,
      impostors,
      mut skipped,
    } = gathering;
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
    let instances: Vec<SectorInstanceGroup> = self.pack_instances::<T>(gathered, wanted, &mut builder, &mut skipped);
    let impostors = impostors.write_into(&mut builder);

    SectorPackage {
      description: SectorDescription {
        bounds,
        buffer_length: builder.length(),
        geometry,
        impostors,
        instances,
        sections,
        sector,
        skipped,
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

    let (window, progressive): (SectorWindow, Option<SectorProgressive>) =
      Self::get_instance_detail(key.index_count, gathering.windows.as_deref())?;
    let mut indices: Vec<u32> = self
      .source
      .read_indices::<T>(
        key.index_buffer,
        key.index_base + window.offset,
        window.get_index_count(),
      )?
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
    let impostors: Option<VisualSection> = gathering
      .impostors
      .iter()
      .any(|impostor| *impostor != Self::NO_IMPOSTOR)
      .then(|| builder.push_i32_section(&gathering.impostors));

    Ok(SectorInstanceGroup {
      drawables: gathering.drawables.clone(),
      geometry: arrays.write_into(&indices, builder),
      hemi: builder.push_f32_section(&hemi),
      impostors,
      instance_count: gathering.placements.len() as u32,
      progressive,
      surface: self.surfaces.get(key.shader_id),
      transforms: builder.push_f32_section(&transforms),
    })
  }

  /// A progressive visual's slide windows, the whole detail first: its own, or the level's table it names.
  fn get_windows(&self, visual: &LevelVisual) -> Option<Vec<SectorWindow>> {
    let windows: Vec<SectorWindow> = if let Some(swi) = &visual.swi {
      swi
        .windows
        .iter()
        .map(|window| SectorWindow {
          offset: window.offset,
          triangles: u32::from(window.num_tris),
        })
        .collect()
    } else {
      self
        .source
        .get_file()
        .slide_windows
        .get(visual.swi_container.as_ref()?.ext_swib_index as usize)?
        .windows
        .iter()
        .map(|window| SectorWindow {
          offset: window.offset,
          triangles: u32::from(window.triangles),
        })
        .collect()
    };

    (!windows.is_empty()).then_some(windows)
  }

  /// What of a tree mesh's indices is packed, and the bands its places pick among: the whole run of its windows with
  /// bands where it has more than one, its one window where it has one, and all of its indices where it has none.
  ///
  /// # Errors
  ///
  /// Returns an error when a window reaches past the mesh's indices or does not start on a triangle.
  fn get_instance_detail(
    index_count: u32,
    windows: Option<&[SectorWindow]>,
  ) -> XrfResult<(SectorWindow, Option<SectorProgressive>)> {
    let whole: SectorWindow = SectorWindow {
      offset: 0,
      triangles: index_count / 3,
    };

    let Some(windows) = windows else {
      return Ok((whole, None));
    };

    Self::check_windows(index_count, windows)?;

    if windows.len() == 1 {
      return Ok((windows[0], None));
    }

    let count: u32 = windows.len() as u32;
    let bands: u32 = count.min(SectorProgressive::MAX_BANDS);

    Ok((
      whole,
      Some(SectorProgressive {
        bands: (0..bands)
          .map(|band| {
            let window: SectorWindow = windows[SectorProgressive::get_band_window(band, bands, count) as usize];

            VisualDrawRange {
              count: window.get_index_count(),
              start: window.offset,
            }
          })
          .collect(),
        windows: count,
      }),
    ))
  }

  /// Checks that every window lies within the indices it slides over, on whole triangles.
  ///
  /// # Errors
  ///
  /// Returns an error naming the first window that does not.
  fn check_windows(index_count: u32, windows: &[SectorWindow]) -> XrfResult {
    match windows
      .iter()
      .find(|window| window.offset % 3 != 0 || window.offset + window.get_index_count() > index_count)
    {
      Some(window) => Err(XrfError::new_invalid_error(format!(
        "draws a window of {} triangles from index {}, outside the {index_count} indices it declares",
        window.triangles, window.offset
      ))),
      None => Ok(()),
    }
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
