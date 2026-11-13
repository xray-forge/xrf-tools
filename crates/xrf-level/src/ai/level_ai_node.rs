use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

use crate::ai::level_ai_node_layout::LevelAiNodeLayout;

/// One node of a level's AI map, `CLevelVertex`: a walkable cell, its plane, its cover and its four neighbours.
///
/// Decoded from the packed form the file keeps (`NodeCompressed*`); turning the packed cell and height into a world
/// position needs the map's header, [`crate::LevelAiHeader::get_node_position`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelAiNode {
  /// The neighbours across each of the cell's four edges, by node index. A link at or past the map's node count is no
  /// neighbour, which is how the engine tells them apart (`valid_vertex_id`).
  pub links: [u32; 4],
  /// Cover from standing height in each of the four directions, 0 to 15.
  pub high_cover: [u8; 4],
  /// Cover from crouching height, the same values as [`Self::high_cover`] for a version that stores one set.
  pub low_cover: [u8; 4],
  /// The plane normal, compressed as [`Vector3d::from_compressed_normal`] reads it.
  pub plane: u16,
  /// The cell, `x * row_length + z` over the map's grid.
  pub packed_xz: u32,
  /// The height across the map's vertical extent, 0 at the box floor and 65535 at its top.
  pub packed_y: u16,
  /// The light value the bits past the links hold, 0 to 15 (`NodeCompressed::light`), which no loader reads; always 0
  /// in a version with no such bits.
  pub light: u8,
}

impl LevelAiNode {
  /// Decodes a node from its packed bytes.
  ///
  /// `bytes` holds at least `layout.size` bytes, which the caller guarantees by slicing whole nodes.
  pub(crate) fn decode<T: ByteOrder>(bytes: &[u8], layout: &LevelAiNodeLayout) -> Self {
    let mask: u32 = (1 << layout.link_bits) - 1;
    let position: &[u8] = &bytes[layout.position..];
    let light_bits: u32 = layout.get_light_bits();

    Self {
      links: layout
        .links
        .map(|(offset, shift)| (T::read_u32(&bytes[offset..offset + 4]) >> shift) & mask),
      high_cover: Self::decode_cover::<T>(&bytes[layout.high_cover..]),
      low_cover: Self::decode_cover::<T>(&bytes[layout.low_cover..]),
      plane: T::read_u16(&bytes[layout.plane..]),
      packed_xz: if layout.is_wide_position {
        T::read_u32(position)
      } else {
        T::read_u24(position)
      },
      packed_y: T::read_u16(&position[if layout.is_wide_position { 4 } else { 3 }..]),
      light: if light_bits == 0 {
        0
      } else {
        bytes[layout.high_cover - 1] >> (8 - light_bits)
      },
    }
  }

  /// Packs the node into `bytes`, `layout.size` long and zeroed, the inverse of [`Self::decode`]. A version keeping one
  /// cover set stores [`Self::high_cover`].
  ///
  /// # Errors
  ///
  /// Returns an error when a link, the cell, a cover or the light does not fit the bits the layout gives it, which
  /// would write another node than this one.
  pub(crate) fn encode<T: ByteOrder>(&self, bytes: &mut [u8], layout: &LevelAiNodeLayout) -> XrfResult {
    let mask: u32 = (1 << layout.link_bits) - 1;
    let light_bits: u32 = layout.get_light_bits();
    let cell_bits: u32 = if layout.is_wide_position { 32 } else { 24 };

    if let Some(link) = self.links.iter().find(|link| **link > mask) {
      return Err(Self::new_overflow_error("link", *link, layout.link_bits));
    }

    if cell_bits < 32 && self.packed_xz >> cell_bits != 0 {
      return Err(Self::new_overflow_error("cell", self.packed_xz, cell_bits));
    }

    if let Some(cover) = self.high_cover.iter().chain(&self.low_cover).find(|cover| **cover > 15) {
      return Err(Self::new_overflow_error("cover", u32::from(*cover), 4));
    }

    if u32::from(self.light) >> light_bits != 0 {
      return Err(Self::new_overflow_error("light", u32::from(self.light), light_bits));
    }

    // The links share bytes, so each is merged into the words already written.
    for (link, (offset, shift)) in self.links.iter().zip(layout.links) {
      let word: u32 = T::read_u32(&bytes[offset..offset + 4]) | (link << shift);

      T::write_u32(&mut bytes[offset..offset + 4], word);
    }

    if light_bits > 0 {
      bytes[layout.high_cover - 1] |= self.light << (8 - light_bits);
    }

    T::write_u16(&mut bytes[layout.low_cover..], Self::encode_cover(&self.low_cover));
    T::write_u16(&mut bytes[layout.high_cover..], Self::encode_cover(&self.high_cover));
    T::write_u16(&mut bytes[layout.plane..], self.plane);

    let position: &mut [u8] = &mut bytes[layout.position..];

    if layout.is_wide_position {
      T::write_u32(position, self.packed_xz);
      T::write_u16(&mut position[4..], self.packed_y);
    } else {
      T::write_u24(position, self.packed_xz);
      T::write_u16(&mut position[3..], self.packed_y);
    }

    Ok(())
  }

  /// The plane's unit normal.
  pub fn get_normal(&self) -> Vector3d<f32> {
    Vector3d::from_compressed_normal(self.plane)
  }

  /// The neighbours that are nodes of a map of `node_count` nodes.
  pub fn get_neighbours(&self, node_count: u32) -> impl Iterator<Item = u32> + '_ {
    self.links.iter().copied().filter(move |link| *link < node_count)
  }

  /// A cover set, `NodeCover5`: four 4-bit values, the first in the lowest bits.
  fn decode_cover<T: ByteOrder>(bytes: &[u8]) -> [u8; 4] {
    let packed: u16 = T::read_u16(bytes);

    [0, 4, 8, 12].map(|shift| ((packed >> shift) & 0xF) as u8)
  }

  /// A cover set packed as [`Self::decode_cover`] reads it, each value already within 4 bits.
  fn encode_cover(cover: &[u8; 4]) -> u16 {
    cover
      .iter()
      .zip([0, 4, 8, 12])
      .fold(0, |packed, (value, shift)| packed | (u16::from(*value) << shift))
  }

  fn new_overflow_error(field: &str, value: u32, bits: u32) -> XrfError {
    XrfError::new_invalid_error(format!(
      "Level AI-map node {field} {value} does not fit the {bits} bits its version gives it"
    ))
  }
}
