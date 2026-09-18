use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;
use xrf_utils::to_format_size;

use crate::wallmarks::level_wallmark_vertex::LevelWallmarkVertex;

/// One baked decal of a level, `ESceneWallmarkTool::wallmark`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelWallmark {
  /// Where the decal sits and how far it reaches, which is what the renderer culls it by.
  pub center: Vector3d<f32>,
  pub radius: f32,
  /// The triangle fan the decal draws, already lit.
  pub vertices: Vec<LevelWallmarkVertex>,
}

impl LevelWallmark {
  /// Bytes a decal occupies before its vertices: a bounding sphere and their count.
  pub const FIXED_SIZE: u64 = 12 + 4 + 4;

  /// Triangles the decal draws, which a fan makes two fewer than its vertices.
  pub const fn get_triangles_count(&self) -> usize {
    self.vertices.len().saturating_sub(2)
  }
}

impl ChunkReadWrite for LevelWallmark {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let center: Vector3d<f32> = reader.read_xr::<T, _>()?;
    let radius: f32 = reader.read_f32::<T>()?;
    let count: u32 = reader.read_u32::<T>()?;
    let mut vertices: Vec<LevelWallmarkVertex> =
      reader.new_bounded_vec(count.into(), LevelWallmarkVertex::SERIALIZED_SIZE, "wallmark vertices")?;

    for _ in 0..count {
      vertices.push(reader.read_xr::<T, _>()?);
    }

    Ok(Self {
      center,
      radius,
      vertices,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.center)?;
    writer.write_f32::<T>(self.radius)?;
    writer.write_u32::<T>(to_format_size(self.vertices.len(), "wallmark vertices")?)?;

    for vertex in &self.vertices {
      writer.write_xr::<T, _>(vertex)?;
    }

    Ok(())
  }
}
