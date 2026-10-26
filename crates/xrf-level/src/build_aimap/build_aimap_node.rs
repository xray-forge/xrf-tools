use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

/// One node of the AI map the level editor hands xrAI: its four neighbours, its plane and its packed position.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildAimapNode {
  /// The neighbours, by node index; a link no node answers is the format's invalid one, the widest value a link of
  /// its version holds.
  pub links: [u32; 4],
  /// The plane normal, compressed as [`Vector3d::from_compressed_normal`] reads it.
  pub plane: u16,
  /// The cell along x, in patches from the world origin (`NodePosition::x`).
  pub x: i16,
  /// The height across the map's box, 0 at its floor and 65535 at its top.
  pub y: u16,
  /// The cell along z, in patches from the world origin.
  pub z: i16,
}

impl BuildAimapNode {
  /// Bytes a link occupies in a version-1 map, which the SDK writes: the low three of a `u32`.
  pub const NARROW_LINK_SIZE: u64 = 3;

  /// Bytes a link occupies from version 2 on, which IX-Ray's editor writes.
  pub const WIDE_LINK_SIZE: u64 = 4;

  /// Bytes a node occupies past its links: the plane and the position.
  pub const TAIL_SIZE: u64 = 2 + 6;

  /// The widest link a version-1 map holds, which is also its invalid one.
  pub const NARROW_LINK_MASK: u32 = 0x00FF_FFFF;

  /// Bytes a node occupies, with four-byte links or three-byte ones.
  pub const fn get_serialized_size(has_wide_links: bool) -> u64 {
    4 * if has_wide_links {
      Self::WIDE_LINK_SIZE
    } else {
      Self::NARROW_LINK_SIZE
    } + Self::TAIL_SIZE
  }

  /// Reads one node, with four-byte links or three-byte ones.
  ///
  /// # Errors
  ///
  /// Returns an error when the node is truncated.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>, has_wide_links: bool) -> XrfResult<Self> {
    let mut links: [u32; 4] = [0; 4];

    for link in &mut links {
      *link = if has_wide_links {
        reader.read_u32::<T>()?
      } else {
        reader.read_u24::<T>()?
      };
    }

    Ok(Self {
      links,
      plane: reader.read_u16::<T>()?,
      x: reader.read_i16::<T>()?,
      y: reader.read_u16::<T>()?,
      z: reader.read_i16::<T>()?,
    })
  }

  /// Writes one node, with four-byte links or three-byte ones.
  ///
  /// # Errors
  ///
  /// Returns an error when a link is too wide for three bytes, or the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter, has_wide_links: bool) -> XrfResult {
    for link in self.links {
      if has_wide_links {
        writer.write_u32::<T>(link)?;
      } else if link > Self::NARROW_LINK_MASK {
        return Err(XrfError::new_invalid_error(format!(
          "AI map link {link} does not fit the three bytes a version-1 map gives it"
        )));
      } else {
        writer.write_u24::<T>(link)?;
      }
    }

    writer.write_u16::<T>(self.plane)?;
    writer.write_i16::<T>(self.x)?;
    writer.write_u16::<T>(self.y)?;
    writer.write_i16::<T>(self.z)?;

    Ok(())
  }

  /// The plane's unit normal.
  pub fn get_normal(&self) -> Vector3d<f32> {
    Vector3d::from_compressed_normal(self.plane)
  }
}
