use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::{EPS_L, Vector3d};

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
  pub aabb_min: Vector3d<f32>,
  pub aabb_max: Vector3d<f32>,
  /// The graph identity, which the spawn set built against this map carries too.
  pub guid: Uuid,
}

impl LevelAiHeader {
  /// Byte size of the header as laid out by the engine.
  pub const SIZE: u64 = 56;

  /// Cells along z, the stride a node's packed cell counts x in (`m_row_length`).
  pub fn get_row_length(&self) -> u32 {
    Self::get_cells_across(self.aabb_min.z, self.aabb_max.z, self.size)
  }

  /// Cells along x (`m_column_length`).
  pub fn get_column_length(&self) -> u32 {
    Self::get_cells_across(self.aabb_min.x, self.aabb_max.x, self.size)
  }

  /// Where a node sits in the world: the centre of its cell, at its packed height (`CLevelGraph::vertex_position`).
  ///
  /// The height is the node's own, not its plane's at another point: the plane passes through this position.
  pub fn get_node_position(&self, node: &LevelAiNode) -> Vector3d<f32> {
    let row_length: u32 = self.get_row_length().max(1);

    Vector3d::new(
      (node.packed_xz / row_length) as f32 * self.size + self.aabb_min.x,
      f32::from(node.packed_y) / 65535.0 * self.size_y + self.aabb_min.y,
      (node.packed_xz % row_length) as f32 * self.size + self.aabb_min.z,
    )
  }

  /// `iFloor(extent / cell_size + EPS_L + 1.5f)`, as `CLevelGraph::Initialize` computes both grid sides.
  fn get_cells_across(minimum: f32, maximum: f32, cell_size: f32) -> u32 {
    ((maximum - minimum) / cell_size + EPS_L + 1.5).floor().max(0.0) as u32
  }
}

impl ChunkReadWrite for LevelAiHeader {
  /// Read level AI-map header from the chunk reader.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      version: reader.read_u32::<T>()?,
      count: reader.read_u32::<T>()?,
      size: reader.read_f32::<T>()?,
      size_y: reader.read_f32::<T>()?,
      aabb_min: reader.read_xr::<T, _>()?,
      aabb_max: reader.read_xr::<T, _>()?,
      guid: Uuid::from_u128(reader.read_u128::<T>()?),
    })
  }

  /// Write level AI-map header into the chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.version)?;
    writer.write_u32::<T>(self.count)?;
    writer.write_f32::<T>(self.size)?;
    writer.write_f32::<T>(self.size_y)?;
    writer.write_xr::<T, _>(&self.aabb_min)?;
    writer.write_xr::<T, _>(&self.aabb_max)?;
    writer.write_u128::<T>(self.guid.as_u128())?;

    Ok(())
  }
}
