use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::ogf::ogf_cylinder::OgfCylinder;
use crate::data::ogf::ogf_obb::OgfObb;
use crate::data::ogf::ogf_sphere::OgfSphere;

/// Collision shape of one bone, `SBoneShape` in the engine.
///
/// All three primitives are always stored, whichever `shape_type` selects. The engine reads the whole
/// struct as one blob under `#pragma pack(1)`, so the layout is exactly 2 + 2 + 60 + 16 + 32 bytes with
/// no padding.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfBoneShape {
  /// 0 none, 1 box, 2 sphere, 3 cylinder.
  pub shape_type: u16,
  pub flags: u16,
  pub box_shape: OgfObb,
  pub sphere: OgfSphere,
  pub cylinder: OgfCylinder,
}

impl OgfBoneShape {
  pub const SIZE: usize = 2 + 2 + 60 + 16 + 32;
}

impl ChunkReadWrite for OgfBoneShape {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      shape_type: reader.read_u16::<T>()?,
      flags: reader.read_u16::<T>()?,
      box_shape: reader.read_xr::<T, _>()?,
      sphere: reader.read_xr::<T, _>()?,
      cylinder: reader.read_xr::<T, _>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u16::<T>(self.shape_type)?;
    writer.write_u16::<T>(self.flags)?;
    self.box_shape.write::<T>(writer)?;
    self.sphere.write::<T>(writer)?;
    self.cylinder.write::<T>(writer)?;

    Ok(())
  }
}
