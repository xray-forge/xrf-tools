use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::{Cylinder, Obb, Sphere};

use crate::SkeletonBoneShapeKind;

/// A bone's collision shape, `SBoneShape`, as OGF visuals and editable objects both store it: which shape it is, how it
/// takes part in physics, and all three shapes, of which `kind` selects one; the engine keeps the other two as last
/// edited.
#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonBoneShape {
  pub kind: SkeletonBoneShapeKind,
  /// `EShapeFlags`, see the `*_FLAG` constants.
  pub flags: u16,
  pub obb: Obb,
  pub sphere: Sphere,
  pub cylinder: Cylinder,
}

impl SkeletonBoneShape {
  /// The kind, the flags and the three shapes.
  pub const SERIALIZED_SIZE: u64 = 2 + 2 + Obb::SERIALIZED_SIZE + Sphere::SERIALIZED_SIZE + Cylinder::SERIALIZED_SIZE;

  /// `sfNoPickable`: rays pass through it.
  pub const NO_PICKABLE_FLAG: u16 = 1 << 0;
  /// `sfRemoveAfterBreak`: the bone's part is removed once its joint breaks.
  pub const REMOVE_AFTER_BREAK_FLAG: u16 = 1 << 1;
  /// `sfNoPhysics`: the shape takes no part in physics.
  pub const NO_PHYSICS_FLAG: u16 = 1 << 2;
  /// `sfNoFogCollider`: fog volumes do not collide with it.
  pub const NO_FOG_COLLIDER_FLAG: u16 = 1 << 3;
}

impl ChunkReadWrite for SkeletonBoneShape {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      kind: SkeletonBoneShapeKind::from(reader.read_u16::<T>()?),
      flags: reader.read_u16::<T>()?,
      obb: reader.read_xr::<T, _>()?,
      sphere: reader.read_xr::<T, _>()?,
      cylinder: reader.read_xr::<T, _>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u16::<T>(u16::from(self.kind))?;
    writer.write_u16::<T>(self.flags)?;
    self.obb.write::<T>(writer)?;
    self.sphere.write::<T>(writer)?;
    self.cylinder.write::<T>(writer)?;

    Ok(())
  }
}
