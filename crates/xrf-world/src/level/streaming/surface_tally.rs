use std::collections::HashMap;

use xrf_level::LevelVertexLayout;
use xrf_renderer::get_section_bytes;
use xrf_visual::{SectorGeometry, SectorPackage};

use crate::contract::world_surface_geometry::WorldSurfaceGeometry;
use crate::contract::world_surface_span::WorldSurfaceSpan;

/// Indices looked at per draw when measuring a coordinate's range.
const SPAN_SAMPLES: u32 = 2048;

/// How much each shader table entry of a level draws, counted sector by sector as the loader packs them.
#[derive(Default)]
pub struct SurfaceTally {
  entries: HashMap<u16, WorldSurfaceGeometry>,
}

impl SurfaceTally {
  /// Counts one packed sector: its sections, and each mesh once per place it stands in at whole detail.
  pub fn measure(package: &SectorPackage) -> Self {
    let mut tally: Self = Self::default();
    let description = &package.description;
    let buffer: &[u8] = &package.buffer;

    for section in &description.sections {
      tally.add(
        section.surface.shader_id,
        section.drawables.len() as u32,
        section.draw.count / 3,
        Self::measure_span(&description.geometry, buffer, section.draw.start, section.draw.count),
      );
    }

    // A mesh's coordinates are the one mesh's, so they are measured once however many places it stands in. A
    // progressive mesh packs every window's indices; at its whole detail it draws its first band.
    for group in &description.instances {
      let (start, count): (u32, u32) = group
        .progressive
        .as_ref()
        .and_then(|progressive| progressive.bands.first())
        .map_or((0, group.geometry.index_count), |band| (band.start, band.count));

      tally.add(
        group.surface.shader_id,
        group.drawables.len() as u32,
        (count / 3).saturating_mul(group.instance_count),
        Self::measure_span(&group.geometry, buffer, start, count),
      );
    }

    tally
  }

  pub fn merge(&mut self, other: Self) {
    for (shader_id, geometry) in other.entries {
      self
        .entries
        .entry(shader_id)
        .or_insert_with(|| WorldSurfaceGeometry::new(shader_id))
        .merge(&geometry);
    }
  }

  /// Every entry something draws, by shader id.
  pub fn list(&self) -> Vec<WorldSurfaceGeometry> {
    let mut listed: Vec<WorldSurfaceGeometry> = self.entries.values().copied().collect();

    listed.sort_unstable_by_key(|geometry| geometry.shader_id);

    listed
  }

  fn add(&mut self, shader_id: u16, drawables: u32, triangles: u32, drawn: Option<WorldSurfaceSpan>) {
    self
      .entries
      .entry(shader_id)
      .or_insert_with(|| WorldSurfaceGeometry::new(shader_id))
      .add(drawables, triangles, drawn);
  }

  /// The range one draw's base coordinate covers, sampled across its indices; none where the geometry carries none.
  fn measure_span(geometry: &SectorGeometry, buffer: &[u8], start: u32, count: u32) -> Option<WorldSurfaceSpan> {
    let uvs: &[u8] = get_section_bytes(buffer, geometry.uvs.as_ref()?);
    let indices: &[u8] = get_section_bytes(buffer, &geometry.indices);
    let tangents: &[u8] = geometry
      .tangents
      .as_ref()
      .map_or(&[], |it| get_section_bytes(buffer, it));
    let binormals: &[u8] = geometry
      .binormals
      .as_ref()
      .map_or(&[], |it| get_section_bytes(buffer, it));
    let end: u32 = start.saturating_add(count).min((indices.len() / 4) as u32);
    let stride: usize = (count / SPAN_SAMPLES).max(1) as usize;
    let mut span: Option<WorldSurfaceSpan> = None;

    for at in (start..end).step_by(stride) {
      let vertex: usize = Self::read_u32(indices, at as usize) as usize;
      let is_tree: bool = geometry.uv_components as usize == LevelVertexLayout::TREE_COORDINATE_SHORTS;
      let coordinate: Option<(f32, f32)> = Self::read_coordinate((uvs, tangents, binormals), vertex, is_tree);

      if let Some((u, v)) = coordinate.filter(|(u, v)| u.is_finite() && v.is_finite()) {
        let at: WorldSurfaceSpan = WorldSurfaceSpan::at(u, v);

        span = Some(span.map_or(at, |span| span.merge(at)));
      }
    }

    span
  }

  /// One vertex's base coordinate: a tree's two shorts over its quantisation, or a baked surface's with the low bytes
  /// that ride in the fourth byte of its tangent and binormal; none where its shorts lie past the array.
  fn read_coordinate(
    (uvs, tangents, binormals): (&[u8], &[u8], &[u8]),
    vertex: usize,
    is_tree: bool,
  ) -> Option<(f32, f32)> {
    if is_tree {
      let at: usize = vertex * LevelVertexLayout::TREE_COORDINATE_SHORTS;
      let (u, v) = Self::read_i16(uvs, at).zip(Self::read_i16(uvs, at + 1))?;

      return Some((
        f32::from(u) / LevelVertexLayout::TREE_QUANT,
        f32::from(v) / LevelVertexLayout::TREE_QUANT,
      ));
    }

    let at: usize = vertex * LevelVertexLayout::BASE_COORDINATE_SHORTS;
    let (u, v) = Self::read_i16(uvs, at).zip(Self::read_i16(uvs, at + 1))?;
    let fraction = |bytes: &[u8]| {
      bytes
        .get(vertex * 4 + 3)
        .map_or(0.0, |it| f32::from(*it) / LevelVertexLayout::BASE_FRACTION)
    };

    Some((
      (f32::from(u) + fraction(tangents)) / LevelVertexLayout::BASE_QUANT,
      (f32::from(v) + fraction(binormals)) / LevelVertexLayout::BASE_QUANT,
    ))
  }

  fn read_u32(bytes: &[u8], index: usize) -> u32 {
    bytes
      .get(index * 4..index * 4 + 4)
      .map_or(0, |it| u32::from_le_bytes([it[0], it[1], it[2], it[3]]))
  }

  fn read_i16(bytes: &[u8], index: usize) -> Option<i16> {
    bytes
      .get(index * 2..index * 2 + 2)
      .map(|it| i16::from_le_bytes([it[0], it[1]]))
  }
}
