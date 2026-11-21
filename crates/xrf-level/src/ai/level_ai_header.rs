use serde::{Deserialize, Serialize};
use uuid::Uuid;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;
use xrf_math::{Aabb, AiNodePosition, Vector3d};

use crate::ai::level_ai_grid::LevelAiGrid;
use crate::ai::level_ai_node::LevelAiNode;

/// `hdrNODES` in c++ codebase, stored raw at the very start of the `level.ai` file.
///
/// Unlike most xray formats `level.ai` is not chunked - `CLevelGraph` opens the file and casts its
/// pointer straight onto this structure, so the header occupies the first 56 bytes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelAiHeader {
  /// The node layout, [`crate::LevelAiVersion`] by number.
  pub version: u32,
  /// Nodes the file holds.
  pub count: u32,
  /// The side of one grid cell, `cell_size`.
  pub size: f32,
  /// The vertical extent a node's packed height spreads over, `factor_y`.
  pub size_y: f32,
  /// The box the grid covers, `aabb`: the first cell sits at its lowest corner.
  pub bounds: Aabb,
  /// The graph identity, which the spawn set built against this map carries too.
  pub guid: Uuid,
}

impl LevelAiHeader {
  /// Byte size of the header as laid out by the engine.
  pub const SIZE: u64 = 56;

  /// The grid the nodes are packed over, checked to be one `CLevelGraph` can unpack them from.
  ///
  /// # Errors
  ///
  /// Returns an error where [`LevelAiGrid::new`] refuses the header's box and cell size.
  pub fn get_grid(&self) -> XrfResult<LevelAiGrid> {
    LevelAiGrid::new(&self.bounds, self.size)
  }

  /// Cells along z, the stride a node's packed cell counts x in (`m_row_length`), unchecked unlike
  /// [`Self::get_grid`].
  pub fn get_row_length(&self) -> u32 {
    LevelAiGrid::covering(&self.bounds, self.size).get_row_length()
  }

  /// Cells along x (`m_column_length`), unchecked unlike [`Self::get_grid`].
  pub fn get_column_length(&self) -> u32 {
    LevelAiGrid::covering(&self.bounds, self.size).get_column_length()
  }

  /// Where a node sits in the world: the centre of its cell, at its packed height (`CLevelGraph::vertex_position`).
  ///
  /// The height is the node's own, not its plane's at another point: the plane passes through this position.
  pub fn get_node_position(&self, node: &LevelAiNode) -> Vector3d<f32> {
    let (x, z) = LevelAiGrid::covering(&self.bounds, self.size).unpack(node.packed_xz);

    Vector3d::new(
      x,
      AiNodePosition::unpack_height(node.packed_y, self.bounds.min.y, self.size_y),
      z,
    )
  }
}

impl ChunkReadWrite for LevelAiHeader {
  /// Read level AI-map header from the chunk reader.
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      version: reader.read_u32()?,
      count: reader.read_u32()?,
      size: reader.read_f32()?,
      size_y: reader.read_f32()?,
      bounds: reader.read_xr()?,
      guid: Uuid::from_u128(reader.read_u128()?),
    })
  }

  /// Write level AI-map header into the chunk writer.
  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32(self.version)?;
    writer.write_u32(self.count)?;
    writer.write_f32(self.size)?;
    writer.write_f32(self.size_y)?;
    writer.write_xr(&self.bounds)?;
    writer.write_u128(self.guid.as_u128())?;

    Ok(())
  }
}
