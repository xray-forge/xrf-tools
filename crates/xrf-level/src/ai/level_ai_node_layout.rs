/// Where a node of one `level.ai` version keeps each field, in bytes from the node's start
/// (`NodeCompressed7` to `NodeCompressed13`, `xray-16/src/Common/LevelStructure.hpp`).
///
/// A link is read as the engine reads it: a `u32` at its offset, shifted right and masked to its width.
/// The link block leaves a few bits unused past the fourth link, which older compilers filled with a light value no
/// loader reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LevelAiNodeLayout {
  /// Bytes one node occupies.
  pub size: usize,
  /// Bits one link occupies.
  pub link_bits: u32,
  /// Per link, the byte its 32-bit word starts at and the shift into that word.
  pub links: [(usize, u32); 4],
  pub high_cover: usize,
  /// The same offset as `high_cover` where a version keeps one cover set for both.
  pub low_cover: usize,
  pub plane: usize,
  pub position: usize,
  /// Whether the packed cell is a whole `u32` (`NodePosition12`) rather than three bytes (`NodePosition4`).
  pub is_wide_position: bool,
}

impl LevelAiNodeLayout {
  const LINKS_23: [(usize, u32); 4] = [(0, 0), (2, 7), (5, 6), (8, 5)];
  const LINKS_25: [(usize, u32); 4] = [(0, 0), (3, 1), (6, 2), (9, 3)];
  const LINKS_26: [(usize, u32); 4] = [(0, 0), (3, 2), (6, 4), (9, 6)];

  /// `NodeCompressed7`: 12 link bytes, one cover, a plane and a 5-byte position.
  pub const NODE_7: Self = Self {
    size: 21,
    link_bits: 23,
    links: Self::LINKS_23,
    high_cover: 12,
    low_cover: 12,
    plane: 14,
    position: 16,
    is_wide_position: false,
  };

  /// `NodeCompressed10`: 12 link bytes, high and low cover, a plane and a 5-byte position.
  pub const NODE_10: Self = Self {
    size: 23,
    link_bits: 23,
    links: Self::LINKS_23,
    high_cover: 12,
    low_cover: 14,
    plane: 16,
    position: 18,
    is_wide_position: false,
  };

  /// `NodeCompressed11`: 13 link bytes, high and low cover, a plane and a 5-byte position.
  pub const NODE_11: Self = Self {
    size: 24,
    link_bits: 25,
    links: Self::LINKS_25,
    high_cover: 13,
    low_cover: 15,
    plane: 17,
    position: 19,
    is_wide_position: false,
  };

  /// `NodeCompressed12`: as [`Self::NODE_11`] with a 6-byte position.
  pub const NODE_12: Self = Self {
    size: 25,
    link_bits: 25,
    links: Self::LINKS_25,
    high_cover: 13,
    low_cover: 15,
    plane: 17,
    position: 19,
    is_wide_position: true,
  };

  /// `NodeCompressed13`: as [`Self::NODE_12`] with 26-bit links.
  pub const NODE_13: Self = Self {
    size: 25,
    link_bits: 26,
    links: Self::LINKS_26,
    high_cover: 13,
    low_cover: 15,
    plane: 17,
    position: 19,
    is_wide_position: true,
  };
}
