use std::collections::HashMap;

use xrf_renderer::get_section_bytes;
use xrf_visual::{SectorGeometry, SectorPackage};

use crate::contract::world_surface_geometry::WorldSurfaceGeometry;
use crate::contract::world_surface_span::WorldSurfaceSpan;

/// Indices looked at per draw when measuring a coordinate's range.
const SPAN_SAMPLES: u32 = 2048;

/// What a baked coordinate's shorts are divided by: `unpack_tc_base` scales by `32 / 32768`.
const BASE_QUANT: f32 = 1024.0;

/// The same for a tree's: `FTreeVisual_quant`, `32768 / 16`.
const TREE_QUANT: f32 = 2048.0;

/// Shorts a tree's coordinate takes a vertex, its last two the wind terms.
const TREE_COMPONENTS: u32 = 4;

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
        measure_span(&description.geometry, buffer, section.draw.start, section.draw.count),
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
        measure_span(&group.geometry, buffer, start, count),
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
    let vertex: usize = read_u32(indices, at as usize) as usize;
    let coordinate: Option<(f32, f32)> = if geometry.uv_components == TREE_COMPONENTS {
      read_i16(uvs, vertex * 4)
        .zip(read_i16(uvs, vertex * 4 + 1))
        .map(|(u, v)| (u as f32 / TREE_QUANT, v as f32 / TREE_QUANT))
    } else {
      // A baked coordinate's low bytes ride in the fourth byte of its tangent and binormal.
      let fraction_u: f32 = tangents.get(vertex * 4 + 3).map_or(0.0, |it| *it as f32 / 255.0);
      let fraction_v: f32 = binormals.get(vertex * 4 + 3).map_or(0.0, |it| *it as f32 / 255.0);

      read_i16(uvs, vertex * 2)
        .zip(read_i16(uvs, vertex * 2 + 1))
        .map(|(u, v)| {
          (
            (u as f32 + fraction_u) / BASE_QUANT,
            (v as f32 + fraction_v) / BASE_QUANT,
          )
        })
    };

    if let Some((u, v)) = coordinate.filter(|(u, v)| u.is_finite() && v.is_finite()) {
      let at: WorldSurfaceSpan = WorldSurfaceSpan::at(u, v);

      span = Some(span.map_or(at, |span| span.merge(at)));
    }
  }

  span
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
