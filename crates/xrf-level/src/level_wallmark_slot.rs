use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_utils::to_format_size;

use crate::level_wallmark::LevelWallmark;

/// Every decal of a level that shares one material, `ESceneWallmarkTool::wm_slot`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelWallmarkSlot {
  /// The blender the decals draw through, which names a definition inside `shaders.xr` rather than a file. Empty for
  /// a slot holding nothing, because the exporter writes no names for one.
  pub shader: String,
  /// The texture the decals draw with, which does name a file. Empty for the same reason.
  pub texture: String,
  pub marks: Vec<LevelWallmark>,
}

impl LevelWallmarkSlot {
  /// Bytes a slot occupies before its names and decals.
  pub const FIXED_SIZE: u64 = 4;

  /// Whether the slot names a material at all, which an empty one does not.
  pub fn is_used(&self) -> bool {
    !self.marks.is_empty()
  }
}

impl ChunkReadWrite for LevelWallmarkSlot {
  /// Reads one slot.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let count: u32 = reader.read_u32::<T>()?;

    if count == 0 {
      return Ok(Self {
        shader: String::new(),
        texture: String::new(),
        marks: Vec::new(),
      });
    }

    let shader: String = reader.read_w1251_string()?;
    let texture: String = reader.read_w1251_string()?;
    let mut marks: Vec<LevelWallmark> =
      reader.new_bounded_vec(count.into(), LevelWallmark::FIXED_SIZE, "level wallmarks")?;

    for _ in 0..count {
      marks.push(reader.read_xr::<T, _>()?);
    }

    Ok(Self { shader, texture, marks })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(to_format_size(self.marks.len(), "level wallmarks")?)?;

    if self.marks.is_empty() {
      return Ok(());
    }

    writer.write_w1251_string(&self.shader)?;
    writer.write_w1251_string(&self.texture)?;

    for mark in &self.marks {
      writer.write_xr::<T, _>(mark)?;
    }

    Ok(())
  }
}
