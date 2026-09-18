//! Reads every drawable visual of a compiled level and accounts for what could not be drawn.

use std::time::{Duration, Instant};

use xrf_chunk::{ChunkDataSource, InMemoryChunkDataSource};
use xrf_level::{
  LevelFile, LevelGeomSource, LevelPortal, LevelSector, LevelShaderEntry, LevelVertex, LevelVisual, LevelVisualsChunk,
};
use xrf_ogf::OgfGeometryContainerChunk;
use xrf_report::{Finding, Report, RuleId};
use xrf_spawn::XRayByteOrder;

use crate::commands::level::level_assets::LevelAssets;
use crate::commands::level::verify::level_verification_census::LevelVerificationCensus;
use crate::commands::level::verify::level_verification_result::LevelVerificationResult;
use crate::commands::level::verify::level_verification_state::LevelVerificationState;

/// Reads a compiled level's geometry the way the renderer would address it.
/// Every check here is an invariant the engine relies on without testing.
pub struct LevelVerifier<'a> {
  assets: &'a LevelAssets<'a>,
}

impl<'a> LevelVerifier<'a> {
  /// The bundle every compiled level has.
  const LEVEL_FILE: &'static str = "level";

  /// The render geometry a visual's ordinary container addresses.
  const GEOMETRY_FILE: &'static str = "level.geom";

  /// The alternative buffers a visual's fast path addresses.
  const DETAIL_GEOMETRY_FILE: &'static str = "level.geomx";

  pub fn new(assets: &'a LevelAssets<'a>) -> Self {
    Self { assets }
  }

  /// Sweeps the level and reports every range the renderer could not draw.
  pub fn run(&self) -> LevelVerificationResult {
    let started: Instant = Instant::now();
    let mut state: LevelVerificationState = LevelVerificationState::default();

    let level: Option<LevelFile> = self.read_level(&mut state);
    let visuals: Option<LevelVisualsChunk> = self.read_visuals(&mut state);
    let mut geometry: Option<LevelGeomSource<_>> = self.open(Self::GEOMETRY_FILE, &mut state);
    let mut detail: Option<LevelGeomSource<_>> = self.open(Self::DETAIL_GEOMETRY_FILE, &mut state);

    if let Some(visuals) = &visuals {
      for (index, visual) in visuals.visuals.iter().enumerate() {
        state.census.record_visual(visual.header.model_type);

        if let (Some(container), Some(source)) = (&visual.geometry, geometry.as_mut()) {
          state.census.record_drawable(container);
          self.verify_container(index, Self::GEOMETRY_FILE, container, source, &mut state);
        }

        if let (Some(container), Some(source)) = (&visual.fastpath, detail.as_mut()) {
          state.census.fastpath_visuals += 1;
          self.verify_container(index, Self::DETAIL_GEOMETRY_FILE, container, source, &mut state);
        }

        self.verify_shader(index, visual, level.as_ref(), &mut state.ranges);
      }
    }

    self.verify_sectors(level.as_ref(), visuals.as_ref(), &mut state);

    let duration: Duration = started.elapsed();
    let (census, report): (LevelVerificationCensus, Report) = state.into_report(duration);

    LevelVerificationResult {
      census,
      duration,
      report,
    }
  }

  /// Reads the bundle, reporting an absent shader table as the defect the renderer asserts on.
  fn read_level(&self, state: &mut LevelVerificationState) -> Option<LevelFile> {
    let bytes: Vec<u8> = self.require_file(Self::LEVEL_FILE, "level.read", state)?;
    let level: LevelFile = match LevelFile::read_from_bytes::<XRayByteOrder>(bytes) {
      Ok(level) => level,
      Err(error) => {
        state
          .read
          .push(self.finding("level.read", Self::LEVEL_FILE, error.to_string()));

        return None;
      }
    };

    state.census.shader_entries = level.shaders.as_ref().map_or(0, |shaders| shaders.entries.len());

    if level.shaders.is_none() {
      state.read.push(self.finding(
        "level.shaders.absent",
        Self::LEVEL_FILE,
        String::from("The level carries no shader table, which the renderer asserts on"),
      ));
    }

    Some(level)
  }

  /// Reads the visuals run, which is the bulk of the bundle and a separate door for that reason.
  fn read_visuals(&self, state: &mut LevelVerificationState) -> Option<LevelVisualsChunk> {
    let bytes: Vec<u8> = self.require_file(Self::LEVEL_FILE, "level.visuals.read", state)?;

    match LevelFile::read_visuals_from_bytes::<XRayByteOrder>(bytes) {
      Ok(visuals) => visuals,
      Err(error) => {
        state
          .read
          .push(self.finding("level.visuals.read", Self::LEVEL_FILE, error.to_string()));

        None
      }
    }
  }

  /// Reads one visual's range the way the renderer addresses it, and reports what could not be read.
  fn verify_container<D: ChunkDataSource>(
    &self,
    index: usize,
    file: &str,
    container: &OgfGeometryContainerChunk,
    source: &mut LevelGeomSource<D>,
    state: &mut LevelVerificationState,
  ) {
    let has_normal: bool = source
      .get_file()
      .vertex_buffers
      .get(container.vertex_buffer_id as usize)
      .is_some_and(|buffer| state.census.record_layout(buffer));

    if !container.index_count.is_multiple_of(3) {
      state.ranges.push(self.finding(
        "level.visuals.indices.not_triangles",
        file,
        format!(
          "Visual {index} draws {} indices, which is not a whole number of triangles",
          container.index_count
        ),
      ));
    }

    let vertices: Vec<LevelVertex> = match source.read_vertices::<XRayByteOrder>(
      container.vertex_buffer_id,
      container.vertex_base,
      container.vertex_count,
    ) {
      Ok(vertices) => vertices,
      Err(error) => {
        state.ranges.push(self.finding(
          "level.visuals.vertices.unreadable",
          file,
          format!("Visual {index}: {error}"),
        ));

        return;
      }
    };

    let indices: Vec<u16> = match source.read_indices::<XRayByteOrder>(
      container.index_buffer_id,
      container.index_base,
      container.index_count,
    ) {
      Ok(indices) => indices,
      Err(error) => {
        state.ranges.push(self.finding(
          "level.visuals.indices.unreadable",
          file,
          format!("Visual {index}: {error}"),
        ));

        return;
      }
    };

    self.verify_geometry(index, file, container, &vertices, &indices, has_normal, state);
  }

  /// Judges what a decoded range holds, once it is known to be readable.
  #[allow(clippy::too_many_arguments)]
  fn verify_geometry(
    &self,
    index: usize,
    file: &str,
    container: &OgfGeometryContainerChunk,
    vertices: &[LevelVertex],
    indices: &[u16],
    has_normal: bool,
    state: &mut LevelVerificationState,
  ) {
    // The renderer passes the visual's vertex base as the base vertex index, so an index is relative to that visual's
    // own range and one past its end reads a neighbour's vertex (`xray-16/src/Layers/xrRender/FVisual.cpp`).
    if let Some(stray) = indices
      .iter()
      .find(|index| u32::from(**index) >= container.vertex_count)
    {
      state.geometry.push(self.finding(
        "level.visuals.indices.out_of_range",
        file,
        format!(
          "Visual {index} draws index {stray}, past the {} vertices it declares",
          container.vertex_count
        ),
      ));
    }

    for vertex in vertices {
      state.census.record_vertex(vertex, has_normal);
    }

    if let Some(position) = vertices.iter().position(|vertex| {
      !vertex.position.x.is_finite() || !vertex.position.y.is_finite() || !vertex.position.z.is_finite()
    }) {
      state.geometry.push(self.finding(
        "level.visuals.vertices.not_finite",
        file,
        format!("Visual {index} carries a vertex at {position} whose position is not a finite number"),
      ));
    }
  }

  /// Checks that every sector and portal names something that exists.
  fn verify_sectors(
    &self,
    level: Option<&LevelFile>,
    visuals: Option<&LevelVisualsChunk>,
    state: &mut LevelVerificationState,
  ) {
    let Some(level) = level else {
      return;
    };

    let sectors: &[LevelSector] = level.sectors.as_ref().map_or(&[], |chunk| &chunk.sectors);
    let portals: &[LevelPortal] = level.portals.as_ref().map_or(&[], |chunk| &chunk.portals);
    let visual_count: usize = visuals.map_or(0, |visuals| visuals.visuals.len());

    state.census.sectors = sectors.len();
    state.census.portals = portals.len();
    state.census.lights = level.lights.as_ref().map_or(0, |chunk| chunk.lights.len());
    state.census.has_sun = level.lights.as_ref().is_some_and(|chunk| chunk.get_sun().is_some());

    for (index, sector) in sectors.iter().enumerate() {
      state.census.sector_portal_references += sector.portals.len();

      if visuals.is_some() && sector.root as usize >= visual_count {
        state.ranges.push(self.finding(
          "level.sectors.root.out_of_range",
          Self::LEVEL_FILE,
          format!(
            "Sector {index} draws visual {}, past the {visual_count} the level holds",
            sector.root
          ),
        ));
      }

      if let Some(stray) = sector.portals.iter().find(|id| **id as usize >= portals.len()) {
        state.ranges.push(self.finding(
          "level.sectors.portal.out_of_range",
          Self::LEVEL_FILE,
          format!(
            "Sector {index} names portal {stray}, past the {} the level holds",
            portals.len()
          ),
        ));
      }
    }

    for (index, portal) in portals.iter().enumerate() {
      if !portal.is_polygon() {
        state.geometry.push(self.finding(
          "level.portals.not_a_polygon",
          Self::LEVEL_FILE,
          format!(
            "Portal {index} spans {} vertices, where a polygon takes three and the record holds six",
            portal.vertex_count
          ),
        ));
      }

      for (side, sector) in [("front", portal.sector_front), ("back", portal.sector_back)] {
        if sector as usize >= sectors.len() {
          state.ranges.push(self.finding(
            "level.portals.sector.out_of_range",
            Self::LEVEL_FILE,
            format!(
              "Portal {index} joins sector {sector} on its {side}, past the {} the level holds",
              sectors.len()
            ),
          ));
        }
      }
    }
  }

  /// Checks that a visual's shader id names an entry of the level's table.
  fn verify_shader(&self, index: usize, visual: &LevelVisual, level: Option<&LevelFile>, findings: &mut Vec<Finding>) {
    // Zero means no shader; the engine only resolves one when it is non-zero.
    if visual.header.shader_id == 0 {
      return;
    }

    let Some(entries) = level.and_then(|level| level.shaders.as_ref()) else {
      return;
    };

    match entries.entries.get(visual.header.shader_id as usize) {
      Some(LevelShaderEntry::Reference(_)) => {}
      Some(LevelShaderEntry::Empty) => findings.push(self.finding(
        "level.visuals.shader.empty",
        Self::LEVEL_FILE,
        format!(
          "Visual {index} names shader table entry {}, which is empty and the renderer skips",
          visual.header.shader_id
        ),
      )),
      Some(LevelShaderEntry::Malformed(raw)) => findings.push(self.finding(
        "level.visuals.shader.malformed",
        Self::LEVEL_FILE,
        format!(
          "Visual {index} names shader table entry {} ({raw}), which carries no delimiter and the renderer dereferences the result without a null check",
          visual.header.shader_id
        ),
      )),
      None => findings.push(self.finding(
        "level.visuals.shader.out_of_range",
        Self::LEVEL_FILE,
        format!(
          "Visual {index} names shader table entry {}, past the {} the table holds",
          visual.header.shader_id,
          entries.entries.len()
        ),
      )),
    }
  }

  /// Opens one render geometry file, reporting a read failure rather than stopping the sweep.
  fn open(&self, name: &str, state: &mut LevelVerificationState) -> Option<LevelGeomSource<InMemoryChunkDataSource>> {
    let bytes: Vec<u8> = self.read_file(name, "level.geometry.read", state)?;

    match LevelGeomSource::open_from_bytes::<XRayByteOrder>(bytes) {
      Ok(source) => Some(source),
      Err(error) => {
        state
          .read
          .push(self.finding("level.geometry.read", name, error.to_string()));

        None
      }
    }
  }

  /// Reads one of the level's files, where the level not having it is a finding.
  ///
  /// The bundle is not optional: a level without one is what a path naming no level at all looks like, and reporting
  /// nothing would let it pass every check by having nothing to check.
  fn require_file(&self, name: &str, rule: &str, state: &mut LevelVerificationState) -> Option<Vec<u8>> {
    match self.assets.read(name) {
      Ok(Some(bytes)) => Some(bytes),
      Ok(None) => {
        state.read.push(self.finding(
          rule,
          name,
          format!("Level file was not found: {}", self.assets.describe_file(name)),
        ));

        None
      }
      Err(error) => {
        state.read.push(self.finding(rule, name, error.to_string()));

        None
      }
    }
  }

  /// Reads one of the level's files, where the level not having it is ordinary, as `level.geomx` is.
  fn read_file(&self, name: &str, rule: &str, state: &mut LevelVerificationState) -> Option<Vec<u8>> {
    match self.assets.read(name) {
      Ok(Some(bytes)) => Some(bytes),
      Ok(None) => None,
      Err(error) => {
        state.read.push(self.finding(rule, name, error.to_string()));

        None
      }
    }
  }

  fn finding(&self, rule: &str, file: &str, message: String) -> Finding {
    Finding::new(
      RuleId::new(rule).expect("Expected a non-empty rule id"),
      Some(self.assets.describe_file(file)),
      message,
    )
  }
}
