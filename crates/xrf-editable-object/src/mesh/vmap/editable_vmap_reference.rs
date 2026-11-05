use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_utils::to_format_size;

use crate::EditableVMapPoint;

/// The vertex map entries one face corner reads, `st_VMapPtLst`: its UV, then a weight per bone that moves it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditableVMapReference {
  /// At most 255, which is what the count byte holds.
  pub points: Vec<EditableVMapPoint>,
}

impl EditableVMapReference {
  /// The count byte of an empty list.
  pub const MIN_SERIALIZED_SIZE: u64 = 1;
}

impl ChunkReadWrite for EditableVMapReference {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let count: u8 = reader.read_u8()?;
    let mut points: Vec<EditableVMapPoint> =
      reader.new_bounded_vec(count.into(), EditableVMapPoint::SIZE, "vertex map points")?;

    for _ in 0..count {
      points.push(reader.read_xr::<T, _>()?);
    }

    Ok(Self { points })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u8(to_format_size(self.points.len(), "Vertex map points")?)?;

    for point in &self.points {
      point.write::<T>(writer)?;
    }

    Ok(())
  }
}
