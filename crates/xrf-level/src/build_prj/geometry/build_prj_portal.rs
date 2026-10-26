use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

/// One portal between two sectors, `b_portal` (`communicate.h`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPrjPortal {
  pub sector_front: u16,
  pub sector_back: u16,
  /// Every slot of the `svector`, the ones past [`Self::vertex_count`] holding whatever the editor's memory did; kept
  /// so a rewrite gives back what was read.
  pub vertex_slots: [Vector3d<f32>; Self::MAXIMUM_VERTICES],
  pub vertex_count: u32,
}

impl BuildPrjPortal {
  /// `XR_MAX_PORTAL_VERTS`.
  pub const MAXIMUM_VERTICES: usize = 6;

  /// Bytes one portal occupies.
  pub const SERIALIZED_SIZE: u64 = 2 + 2 + Self::MAXIMUM_VERTICES as u64 * 12 + 4;

  /// The portal's polygon, its slots up to its count.
  pub fn get_vertices(&self) -> &[Vector3d<f32>] {
    &self.vertex_slots[..(self.vertex_count as usize).min(Self::MAXIMUM_VERTICES)]
  }
}

impl ChunkReadWrite for BuildPrjPortal {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let sector_front: u16 = reader.read_u16::<T>()?;
    let sector_back: u16 = reader.read_u16::<T>()?;
    let mut vertex_slots: [Vector3d<f32>; Self::MAXIMUM_VERTICES] = [const { Vector3d::ZERO }; Self::MAXIMUM_VERTICES];

    for slot in &mut vertex_slots {
      *slot = reader.read_xr::<T, _>()?;
    }

    Ok(Self {
      sector_front,
      sector_back,
      vertex_slots,
      vertex_count: reader.read_u32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u16::<T>(self.sector_front)?;
    writer.write_u16::<T>(self.sector_back)?;

    for slot in &self.vertex_slots {
      writer.write_xr::<T, _>(slot)?;
    }

    writer.write_u32::<T>(self.vertex_count)?;

    Ok(())
  }
}
