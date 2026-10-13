use xrf_visual::{SectorGeometry, VisualGeometry, VisualSection};

use crate::scene::section_bytes::{get_section_bytes, read_pods};
use crate::scene::static_scene::static_layout::StaticLayout;

/// Packs geometry into the static arenas' vertex words, and measures how far it sways.
pub struct StaticVertexWords;

impl StaticVertexWords {
  /// The normal a geometry carries none of: straight along `z`, packed as `pack_direction` packs one.
  pub const NEUTRAL_NORMAL: u32 = 0x0080_80FF;
  /// The tangent a geometry carries none of: along `x`, square to the neutral normal and binormal.
  pub const NEUTRAL_TANGENT: u32 = 0x00FF_8080;
  /// The binormal a geometry carries none of: along `y`, square to the neutral normal and tangent.
  pub const NEUTRAL_BINORMAL: u32 = 0x0080_FF80;

  /// A packed geometry's vertices as its arena holds them, `StaticLayout::STRIDE` words each.
  ///
  /// The shorts and bytes travel as the words they were packed into, decoded in the vertex shader; an attribute the
  /// pack left out is filled neutral, so every geometry of a layout shares one arena and one program.
  pub fn pack_vertex_words(layout: StaticLayout, geometry: &SectorGeometry, buffer: &[u8]) -> Vec<u32> {
    let count: usize = geometry.vertex_count as usize;
    let words = |section: &Option<VisualSection>| -> Option<&[u8]> {
      section.as_ref().map(|section| get_section_bytes(buffer, section))
    };
    let binormals: Option<&[u8]> = words(&geometry.binormals);
    let normals: Option<&[u8]> = words(&geometry.normals);
    let tangents: Option<&[u8]> = words(&geometry.tangents);
    let uv_words: usize = (geometry.uv_components as usize / 2).max(1);
    let uvs: Option<&[u8]> = words(&geometry.uvs);
    let lightmap_uvs: Option<&[u8]> = words(&geometry.lightmap_uvs);
    // A vertex lit geometry has no lightmap coordinate, so its baked light rides in that word instead.
    let colors: Option<&[u8]> = words(&geometry.colors);
    let positions: &[u8] = get_section_bytes(buffer, &geometry.positions);
    let mut packed: Vec<u32> = Vec::with_capacity(count * StaticLayout::STRIDE as usize);

    for vertex in 0..count {
      let mut words: [u32; StaticLayout::STRIDE as usize] = [0; StaticLayout::STRIDE as usize];

      for axis in 0..3 {
        words[StaticLayout::POSITION + axis] = Self::read_word(Some(positions), vertex, 3, axis).unwrap_or(0);
      }

      words[StaticLayout::BINORMAL] = Self::read_word(binormals, vertex, 1, 0).unwrap_or(Self::NEUTRAL_BINORMAL);
      words[StaticLayout::NORMAL] = Self::read_word(normals, vertex, 1, 0).unwrap_or(Self::NEUTRAL_NORMAL);
      words[StaticLayout::TANGENT] = Self::read_word(tangents, vertex, 1, 0).unwrap_or(Self::NEUTRAL_TANGENT);
      words[StaticLayout::UV] = Self::read_word(uvs, vertex, uv_words, 0).unwrap_or(0);
      // A sector's geometry is baked or a tree's; nothing of a sector is packed as a model.
      words[StaticLayout::EXTRA] = if layout == StaticLayout::Tree {
        Self::read_word(uvs, vertex, uv_words, 1).unwrap_or(0)
      } else {
        Self::read_word(lightmap_uvs, vertex, 1, 0)
          .or_else(|| Self::read_word(colors, vertex, 1, 0))
          .unwrap_or(0)
      };

      packed.extend_from_slice(&words);
    }

    packed
  }

  /// A spawned model's posed vertices as the model arena holds them: binormal, normal and tangent packed as a sector's
  /// are, `z` then `y` then `x` a byte each with nothing in the fourth, after the position, then the base coordinate as
  /// two floats. Its attributes are already in renderer space.
  pub fn pack_model_words(geometry: &VisualGeometry, buffer: &[u8]) -> Vec<u32> {
    let floats = |section: &VisualSection| -> Vec<f32> { read_pods(buffer, section) };
    let (positions, normals, tangents, binormals, uvs) = (
      floats(&geometry.positions),
      floats(&geometry.normals),
      floats(&geometry.tangents),
      floats(&geometry.binormals),
      floats(&geometry.uvs),
    );
    let direction = |values: &[f32], vertex: usize, neutral: u32| -> u32 {
      values
        .get(vertex * 3..vertex * 3 + 3)
        .map_or(neutral, |it| Self::pack_direction([it[0], it[1], it[2]]))
    };
    let count: usize = (geometry.vertex_count as usize).min(positions.len() / 3);
    let mut packed: Vec<u32> = Vec::with_capacity(count * StaticLayout::STRIDE as usize);

    for vertex in 0..count {
      let mut words: [u32; StaticLayout::STRIDE as usize] = [0; StaticLayout::STRIDE as usize];

      for axis in 0..3 {
        words[StaticLayout::POSITION + axis] = positions[vertex * 3 + axis].to_bits();
      }

      words[StaticLayout::BINORMAL] = direction(&binormals, vertex, Self::NEUTRAL_BINORMAL);
      words[StaticLayout::NORMAL] = direction(&normals, vertex, Self::NEUTRAL_NORMAL);
      words[StaticLayout::TANGENT] = direction(&tangents, vertex, Self::NEUTRAL_TANGENT);
      words[StaticLayout::UV] = uvs.get(vertex * 2).copied().unwrap_or(0.0).to_bits();
      words[StaticLayout::EXTRA] = uvs.get(vertex * 2 + 1).copied().unwrap_or(0.0).to_bits();
      packed.extend_from_slice(&words);
    }

    packed
  }

  /// A unit direction as a `D3DCOLOR` packs one: `z`, `y` and `x` from `-1..1` to a byte each, nothing in the fourth.
  pub fn pack_direction([x, y, z]: [f32; 3]) -> u32 {
    let byte = |value: f32| -> u32 { ((value.clamp(-1.0, 1.0) * 0.5 + 0.5) * 255.0).round() as u32 };

    byte(z) | (byte(y) << 8) | (byte(x) << 16)
  }

  /// How far a tree geometry's vertices reach from its foot, each weighted by its rigidity: what the wind's amplitude
  /// is multiplied by to give the farthest any of them leans, before the place's scale.
  pub fn measure_sway_reach(geometry: &SectorGeometry, buffer: &[u8]) -> f32 {
    let uv_words: usize = (geometry.uv_components as usize / 2).max(1);
    let uvs: Option<&[u8]> = geometry.uvs.as_ref().map(|section| get_section_bytes(buffer, section));
    let positions: &[u8] = get_section_bytes(buffer, &geometry.positions);

    (0..geometry.vertex_count as usize)
      .map(|vertex| {
        let rigidity: f32 =
          Self::read_word(uvs, vertex, uv_words, 1).map_or(0.0, |word| (word as u16 as i16) as f32 / 2048.0);
        let position: [f32; 3] =
          [0, 1, 2].map(|axis| f32::from_bits(Self::read_word(Some(positions), vertex, 3, axis).unwrap_or(0)));
        let length: f32 = position.iter().map(|it| it * it).sum::<f32>().sqrt();

        length * rigidity.abs()
      })
      .filter(|reach| reach.is_finite())
      .fold(0.0, f32::max)
  }

  fn read_word(bytes: Option<&[u8]>, vertex: usize, per_vertex: usize, word: usize) -> Option<u32> {
    let start: usize = (vertex * per_vertex + word) * 4;

    bytes
      .and_then(|bytes| bytes.get(start..start + 4))
      .map(|it| u32::from_le_bytes([it[0], it[1], it[2], it[3]]))
  }
}
