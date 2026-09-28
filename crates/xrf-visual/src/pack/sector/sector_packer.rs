use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

use byteorder::ByteOrder;
use xrf_chunk::ChunkDataSource;
use xrf_error::{XrfError, XrfResult};
use xrf_level::{
  LevelGeomSource, LevelSectorComposition, LevelShadersChunk, LevelVertexLayout, LevelVertexPayload, LevelVisual,
  LevelVisualsChunk,
};
use xrf_ogf::{OgfGeometryContainerChunk, OgfLodDefinitionChunk, OgfSwiContainerChunk, OgfSwiDataChunk};

use crate::data::sector::instance::sector_instance_group::SectorInstanceGroup;
use crate::data::sector::instance::sector_progressive::SectorProgressive;
use crate::data::sector::sector_attributes::SectorAttributes;
use crate::data::sector::sector_description::SectorDescription;
use crate::data::sector::sector_geometry::SectorGeometry;
use crate::data::sector::sector_section::SectorSection;
use crate::data::sector::sector_skip::SectorSkip;
use crate::data::visual::bounds::visual_bounds::VisualBounds;
use crate::data::visual::geometry::visual_clusters::VisualClusters;
use crate::data::visual::geometry::visual_draw_range::VisualDrawRange;
use crate::data::visual::geometry::visual_section::VisualSection;
use crate::data::visual::geometry::visual_skip_cause::VisualSkipCause;
use crate::pack::sector::gathering::sector_gathering::SectorGathering;
use crate::pack::sector::gathering::sector_index_range::SectorIndexRange;
use crate::pack::sector::gathering::sector_instance_gathering::SectorInstanceGathering;
use crate::pack::sector::gathering::sector_instance_key::SectorInstanceKey;
use crate::pack::sector::gathering::sector_section_gathering::SectorSectionGathering;
use crate::pack::sector::gathering::sector_vertex_range::SectorVertexRange;
use crate::pack::sector::sector_impostor_arrays::SectorImpostorArrays;
use crate::pack::sector::sector_package::SectorPackage;
use crate::pack::sector::sector_surface_table::SectorSurfaceTable;
use crate::pack::sector::sector_vertex_arrays::SectorVertexArrays;
use crate::pack::visual_buffer_builder::VisualBufferBuilder;
use crate::pack::visual_cluster_table::VisualClusterTable;
use crate::pack::visual_conversion::{convert_placement, convert_tree_hemi};
use crate::pack::visual_index_window::VisualIndexWindow;

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
    let (impostors, parents): (SectorImpostorArrays, BTreeMap<u32, u32>) =
      self.gather_impostors(composition, &mut skipped);

    for drawable in &composition.drawables {
      let Some((visual, container)) = Self::get_drawable(self.visuals, *drawable) else {
        continue;
      };

      // A visual the level places is an instance of a mesh rather than geometry of its own: the mesh is packed once
      // below and stood in every place that names it.
      if let Some(tree) = &visual.tree {
        // Every place of one mesh draws the windows of the first to name it.
        let gathering: &mut SectorInstanceGathering =
          match gathered.entry(SectorInstanceKey::of(container, visual.header.shader_id)) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => match self.get_windows(visual) {
              Ok(windows) => entry.insert(SectorInstanceGathering {
                windows,
                ..SectorInstanceGathering::default()
              }),
              Err(error) => {
                skipped.push(Self::skip(*drawable, &error));

                continue;
              }
            },
          };

        gathering.drawables.push(*drawable);
        gathering.placements.push(convert_placement(&tree.transform));
        gathering.hemi.push(convert_tree_hemi(tree));
        gathering
          .impostors
          .push(parents.get(drawable).map_or(Self::NO_IMPOSTOR, |index| *index as i32));

        continue;
      }

      // A progressive mesh's indices are every window laid end to end; baked into a section it draws the whole detail,
      // its first window, since a section of many visuals has one range.
      let packed_window: XrfResult<(VisualIndexWindow, u32)> = self.get_first_window(visual).and_then(|window| {
        let base: u32 = self.pack_range::<T>(container, &mut arrays, &mut packed)?;

        Ok((window.unwrap_or(VisualIndexWindow::whole(container.index_count)), base))
      });
      let (window, base): (VisualIndexWindow, u32) = match packed_window {
        Ok(packed) => packed,
        Err(error) => {
          skipped.push(Self::skip(*drawable, &error));

          continue;
        }
      };

      match SectorIndexRange::of(container).read_window::<T, D>(self.source, window, container.vertex_count, base) {
        Ok(indices) => {
          let gathering: &mut SectorSectionGathering = sections.entry(visual.header.shader_id).or_default();

          gathering.drawables.push(*drawable);
          gathering.runs.push(indices.len() as u32);
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
  /// belongs to, by the tree's index in the visuals run. A visual whose impostor cannot be read is left out, and its
  /// trees stand without one.
  fn gather_impostors(
    &self,
    composition: &LevelSectorComposition,
    skipped: &mut Vec<SectorSkip>,
  ) -> (SectorImpostorArrays, BTreeMap<u32, u32>) {
    let mut lods: Vec<(u16, u32, &LevelVisual, &OgfLodDefinitionChunk)> = Vec::new();

    for index in &composition.hierarchies {
      let Some(visual) = self.visuals.visuals.get(*index as usize) else {
        continue;
      };

      match &visual.lod {
        Some(Ok(definition)) => lods.push((visual.header.shader_id, *index, visual, definition)),
        Some(Err(error)) => skipped.push(Self::skip(*index, &Self::unreadable("impostor", error))),
        None => {}
      }
    }

    let mut arrays: SectorImpostorArrays = SectorImpostorArrays::default();
    let mut parents: BTreeMap<u32, u32> = BTreeMap::new();

    lods.sort_unstable_by_key(|(shader_id, index, _, _)| (*shader_id, *index));

    for (_, _, visual, definition) in lods {
      for child in &visual.children {
        parents.insert(*child, arrays.len());
      }

      arrays.push(visual, definition, &self.surfaces);
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
    let mut runs: Vec<(u32, Vec<(u32, u32)>)> = Vec::new();
    let mut sections: Vec<SectorSection> = Vec::new();

    for (shader_id, gathering) in gathered_sections {
      let start: u32 = indices.len() as u32;

      runs.push((start, gathering.drawables.iter().copied().zip(gathering.runs).collect()));
      sections.push(SectorSection {
        bounds: arrays.get_indexed_bounds(&gathering.indices),
        clusters: VisualDrawRange::default(),
        draw: VisualDrawRange {
          count: gathering.indices.len() as u32,
          start,
        },
        drawables: gathering.drawables,
        surface: self.surfaces.get(shader_id),
      });

      indices.extend(gathering.indices);
    }

    let mut clusters: VisualClusterTable = VisualClusterTable::default();

    // A section's clusters are its drawables' own, so none spans two of them.
    for (section, (start, drawables)) in sections.iter_mut().zip(runs) {
      let first: u32 = clusters.get_count();
      let mut at: u32 = start;

      for (drawable, count) in drawables {
        clusters.push_run(&indices, arrays.get_positions(), at, count, drawable);
        at += count;
      }

      section.clusters = VisualDrawRange {
        count: clusters.get_count() - first,
        start: first,
      };
    }

    let bounds: Option<VisualBounds> = arrays.get_bounds();
    let geometry: SectorGeometry = arrays.write_into(&indices, &clusters, &mut builder);
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
      SectorAttributes::of(payload.get_layout()).intersect(wanted),
      SectorVertexArrays::uv_components_of(payload.get_layout()),
    );

    // Packed unplaced: the mesh is in its own space, and each instance's transform stands a copy of it.
    arrays.push::<T>(&payload)?;

    let (window, mut progressive): (VisualIndexWindow, Option<SectorProgressive>) =
      Self::get_instance_detail(key.indices.count, gathering.windows.as_deref())?;
    // Onto nought: the mesh is packed alone.
    let indices: Vec<u32> = key
      .indices
      .read_window::<T, D>(self.source, window, key.vertices.count, 0)?;

    let mut table: VisualClusterTable = VisualClusterTable::default();
    // A place draws its band's window, or the whole mesh: the whole detail is band nought.
    let clusters: VisualDrawRange = match &mut progressive {
      Some(progressive) => {
        progressive.clusters = progressive
          .bands
          .iter()
          .map(|band| {
            table.push_run(
              &indices,
              arrays.get_positions(),
              band.start,
              band.count,
              VisualClusters::NO_DRAWABLE,
            )
          })
          .collect();

        progressive.clusters[0]
      }
      None => table.push_run(
        &indices,
        arrays.get_positions(),
        0,
        indices.len() as u32,
        VisualClusters::NO_DRAWABLE,
      ),
    };

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
      clusters,
      drawables: gathering.drawables.clone(),
      geometry: arrays.write_into(&indices, &table, builder),
      hemi: builder.push_f32_section(&hemi),
      impostors,
      instance_count: gathering.placements.len() as u32,
      progressive,
      surface: self.surfaces.get(key.shader_id),
      transforms: builder.push_f32_section(&transforms),
    })
  }

  /// A progressive visual's slide windows, the whole detail first, or `None` for a visual of one detail.
  ///
  /// # Errors
  ///
  /// Returns an error when the visual's windows, or which table of the level's it draws them from, cannot be read.
  fn get_windows(&self, visual: &LevelVisual) -> XrfResult<Option<Vec<VisualIndexWindow>>> {
    Ok(
      self
        .with_windows(visual, |windows| windows.collect::<Vec<VisualIndexWindow>>())?
        .filter(|windows| !windows.is_empty()),
    )
  }

  /// A progressive visual's first window, its whole detail, without collecting the rest.
  ///
  /// # Errors
  ///
  /// Returns an error when the visual's windows cannot be read.
  fn get_first_window(&self, visual: &LevelVisual) -> XrfResult<Option<VisualIndexWindow>> {
    Ok(self.with_windows(visual, |windows| windows.next())?.flatten())
  }

  /// Hands a visual's slide windows to `take`: its own, or those of the level's table it names; `None` for a visual
  /// naming none.
  fn with_windows<R>(
    &self,
    visual: &LevelVisual,
    take: impl FnOnce(&mut dyn Iterator<Item = VisualIndexWindow>) -> R,
  ) -> XrfResult<Option<R>> {
    if let Some(swi) = &visual.swi {
      let swi: &OgfSwiDataChunk = swi.as_ref().map_err(|error| Self::unreadable("slide windows", error))?;

      return Ok(Some(take(
        &mut swi
          .windows
          .iter()
          .map(|window| VisualIndexWindow::of_triangles(window.offset, window.num_tris)),
      )));
    }

    let Some(container) = &visual.swi_container else {
      return Ok(None);
    };
    let container: &OgfSwiContainerChunk = container
      .as_ref()
      .map_err(|error| Self::unreadable("slide window table", error))?;

    Ok(
      self
        .source
        .get_file()
        .slide_windows
        .get(container.ext_swib_index as usize)
        .map(|table| {
          take(
            &mut table
              .windows
              .iter()
              .map(|window| VisualIndexWindow::of_triangles(window.offset, window.triangles)),
          )
        }),
    )
  }

  /// What of a tree mesh's indices is packed, and the bands its places pick among: the whole run of its windows with
  /// bands where it has more than one, its one window where it has one, and all of its indices where it has none.
  ///
  /// # Errors
  ///
  /// Returns an error when a window reaches past the mesh's indices or does not start on a triangle.
  fn get_instance_detail(
    index_count: u32,
    windows: Option<&[VisualIndexWindow]>,
  ) -> XrfResult<(VisualIndexWindow, Option<SectorProgressive>)> {
    let whole: VisualIndexWindow = VisualIndexWindow::whole(index_count);

    let Some(windows) = windows else {
      return Ok((whole, None));
    };

    for window in windows {
      window.check(index_count)?;
    }

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
            let window: VisualIndexWindow = windows[SectorProgressive::get_band_window(band, bands, count) as usize];

            VisualDrawRange {
              count: window.count,
              start: window.offset,
            }
          })
          .collect(),
        clusters: Vec::new(),
        windows: count,
      }),
    ))
  }

  /// Records every instance of a mesh that could not be read, since none of them can be drawn without it.
  fn skip_all(gathering: &SectorInstanceGathering, error: &XrfError, skipped: &mut Vec<SectorSkip>) {
    for drawable in &gathering.drawables {
      skipped.push(Self::skip(*drawable, error));
    }
  }

  /// Why a part of a visual the file carries, which the level was read regardless of, cannot be drawn.
  fn unreadable(part: &str, error: &XrfError) -> XrfError {
    XrfError::new_invalid_error(format!("carries {part} that could not be read: {error}"))
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
