use xrf_visual::{SectorGeometry, VisualSection};

use crate::scene::section_bytes::get_section_bytes;
use crate::scene::static_scene::static_layout::StaticLayout;

/// A direction a geometry carries none of: straight along the tangent frame's `z`, with nothing in its fourth byte.
const NEUTRAL_DIRECTION: u32 = 0x0080_8080;

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
  let positions: &[u8] = get_section_bytes(buffer, &geometry.positions);
  let mut packed: Vec<u32> = Vec::with_capacity(count * StaticLayout::STRIDE as usize);

  for vertex in 0..count {
    packed.push(read_word(binormals, vertex, 1, 0).unwrap_or(NEUTRAL_DIRECTION));
    packed.push(read_word(normals, vertex, 1, 0).unwrap_or(NEUTRAL_DIRECTION));
    packed.push(read_word(tangents, vertex, 1, 0).unwrap_or(NEUTRAL_DIRECTION));

    match layout {
      StaticLayout::Baked => {
        packed.push(read_word(uvs, vertex, uv_words, 0).unwrap_or(0));
        packed.push(read_word(lightmap_uvs, vertex, 1, 0).unwrap_or(0));
      }
      StaticLayout::Tree => {
        packed.push(read_word(uvs, vertex, uv_words, 0).unwrap_or(0));
        packed.push(read_word(uvs, vertex, uv_words, 1).unwrap_or(0));
      }
    }

    for axis in 0..3 {
      packed.push(read_word(Some(positions), vertex, 3, axis).unwrap_or(0));
    }
  }

  packed
}

fn read_word(bytes: Option<&[u8]>, vertex: usize, per_vertex: usize, word: usize) -> Option<u32> {
  let start: usize = (vertex * per_vertex + word) * 4;

  bytes
    .and_then(|bytes| bytes.get(start..start + 4))
    .map(|it| u32::from_le_bytes([it[0], it[1], it[2], it[3]]))
}
