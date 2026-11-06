use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::vector_3d::Vector3d;

/// An `Fmatrix` stored without its fourth column, as `NET_Packet::w_matrix` writes one: the three axis rows `i`, `j`
/// and `k`, then the translation `c`. Reading one back fixes the dropped column at `0, 0, 0, 1`, so what it holds is
/// an affine transform.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Affine3d {
  /// `i`, where the transform takes the x axis.
  pub x_axis: Vector3d,
  /// `j`.
  pub y_axis: Vector3d,
  /// `k`.
  pub z_axis: Vector3d,
  /// `c`.
  pub translation: Vector3d,
}

impl Affine3d {
  /// Four vectors, twelve `f32`s: the 64-byte `Matrix4x4` less its fourth column.
  pub const SERIALIZED_SIZE: u64 = 48;

  /// Places a point, `Fmatrix::transform_tiny`.
  pub fn transform_point(&self, point: &Vector3d) -> Vector3d {
    let mut placed: Vector3d = self.translation;

    placed += &(&self.x_axis * point.x);
    placed += &(&self.y_axis * point.y);
    placed += &(&self.z_axis * point.z);

    placed
  }
}

impl ChunkReadWrite for Affine3d {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      x_axis: reader.read_xr::<T, _>()?,
      y_axis: reader.read_xr::<T, _>()?,
      z_axis: reader.read_xr::<T, _>()?,
      translation: reader.read_xr::<T, _>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.x_axis.write::<T>(writer)?;
    self.y_axis.write::<T>(writer)?;
    self.z_axis.write::<T>(writer)?;
    self.translation.write::<T>(writer)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;

  use crate::affine_3d::Affine3d;
  use crate::vector_3d::Vector3d;

  fn stretched() -> Affine3d {
    Affine3d {
      x_axis: Vector3d::new(2.0, 0.0, 0.0),
      y_axis: Vector3d::new(0.0, 0.0, 3.0),
      z_axis: Vector3d::new(0.0, -1.0, 0.0),
      translation: Vector3d::new(10.0, 20.0, 30.0),
    }
  }

  #[test]
  fn places_a_point_along_its_axes_from_its_translation() {
    assert_eq!(
      stretched().transform_point(&Vector3d::new(1.0, 1.0, 1.0)),
      Vector3d::new(12.0, 19.0, 33.0)
    );
  }

  #[test]
  fn reads_back_what_it_writes_in_the_engine_order() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    stretched().write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written() as u64, Affine3d::SERIALIZED_SIZE);
    assert_eq!(&writer.buffer[..4], &2.0_f32.to_le_bytes(), "i comes first");
    assert_eq!(
      Affine3d::read::<XRayByteOrder, _>(&mut ChunkReader::from_bytes(&writer.buffer)?)?,
      stretched()
    );

    Ok(())
  }
}
