use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
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
}

impl LevelAiNode {
  /// Decodes a node from its packed bytes.
  ///
  /// `bytes` holds at least `layout.size` bytes, which the caller guarantees by slicing whole nodes.
  pub(crate) fn decode<T: ByteOrder>(bytes: &[u8], layout: &LevelAiNodeLayout) -> Self {
    let mask: u32 = (1 << layout.link_bits) - 1;
    let position: &[u8] = &bytes[layout.position..];

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
    }
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
}
