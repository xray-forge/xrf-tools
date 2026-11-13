use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::XrfResult;

use crate::math_constants::EPS_L;
use crate::shape::aabb::Aabb;
use crate::vector_3d::Vector3d;

/// Where an AI map node stands as the level editor packs it, `SNodePositionOld` (`xrLevel.h`): its cell along x and z,
/// counted in patches from the world origin, and its height spread across a box's height. The AI map tool's part and
/// the `build.aimap` it exports hold their nodes this way.
///
/// The packed height is one rule for every AI map file: `level.ai` packs its own height across its header's extent the
/// same way ([`Self::unpack_height`], [`Self::pack_height`]).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiNodePosition {
  pub x: i16,
  /// The height across the box, 0 at its floor and 65535 at its top.
  pub y: u16,
  pub z: i16,
}

impl AiNodePosition {
  /// Three 16-bit fields, x, y and z.
  pub const SERIALIZED_SIZE: u64 = 6;

  /// The packed height's top, `u16::MAX`.
  const HEIGHT_STEPS: f32 = 65535.0;

  /// A point packed against `bounds` with cells of `patch_size`, `ESceneAIMapTool::PackPosition`: each cell
  /// `iFloor(coordinate / patch + EPS_L)`, the height [`Self::pack_height`] over the box's height.
  pub fn pack(position: &Vector3d, patch_size: f32, bounds: &Aabb) -> Self {
    let cell = |coordinate: f32| (coordinate / patch_size + EPS_L).floor().clamp(-32767.0, 32767.0) as i16;

    Self {
      x: cell(position.x),
      y: Self::pack_height(position.y, bounds.min.y, bounds.max.y - bounds.min.y),
      z: cell(position.z),
    }
  }

  /// The point it names, `ESceneAIMapTool::UnpackPosition`: its cell scaled by `patch_size`, its height unpacked across
  /// the box's height.
  pub fn unpack(&self, patch_size: f32, bounds: &Aabb) -> Vector3d {
    Vector3d::new(
      f32::from(self.x) * patch_size,
      Self::unpack_height(self.y, bounds.min.y, bounds.max.y - bounds.min.y),
      f32::from(self.z) * patch_size,
    )
  }

  /// A height packed across `extent` above `floor`: `iFloor(65535 * (y - floor) / extent + EPS_L)`, clamped to 16 bits,
  /// as the editor's export and xrAI's save both compute it. The floor itself for an extent of none, where the engine
  /// would divide by zero.
  pub fn pack_height(y: f32, floor: f32, extent: f32) -> u16 {
    if extent <= 0.0 {
      return 0;
    }

    (Self::HEIGHT_STEPS * (y - floor) / extent + EPS_L)
      .floor()
      .clamp(0.0, Self::HEIGHT_STEPS) as u16
  }

  /// The height a packed one names, `packed / 65535 * extent + floor`.
  pub fn unpack_height(packed: u16, floor: f32, extent: f32) -> f32 {
    f32::from(packed) / Self::HEIGHT_STEPS * extent + floor
  }
}

/// x, y, z as the editor writes them: `s16`, `u16`, `s16`.
impl ChunkFieldReadWrite for AiNodePosition {
  #[inline]
  fn read_fields<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    Ok(Self {
      x: reader.read_u16::<T>()?.cast_signed(),
      y: reader.read_u16::<T>()?,
      z: reader.read_u16::<T>()?.cast_signed(),
    })
  }

  #[inline]
  fn write_fields<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    writer.write_u16::<T>(self.x.cast_unsigned())?;
    writer.write_u16::<T>(self.y)?;
    writer.write_u16::<T>(self.z.cast_unsigned())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;

  use crate::ai_node_position::AiNodePosition;
  use crate::shape::aabb::Aabb;
  use crate::vector_3d::Vector3d;

  #[test]
  fn packs_and_unpacks_a_node_against_its_box() {
    let bounds: Aabb = Aabb {
      min: Vector3d::new(-10.0, 2.0, -10.0),
      max: Vector3d::new(10.0, 12.0, 10.0),
    };
    let position: AiNodePosition = AiNodePosition::pack(&Vector3d::new(-5.6, 7.0, 2.1), 0.7, &bounds);

    assert_eq!((position.x, position.z), (-8, 3));
    assert_eq!(position.y, 32767);

    let unpacked: Vector3d = position.unpack(0.7, &bounds);

    assert!(unpacked.distance(&Vector3d::new(-5.6, 7.0, 2.1)) < 1e-3, "{unpacked}");
  }

  #[test]
  fn packs_a_height_into_the_box_and_the_floor_for_a_flat_one() {
    assert_eq!(AiNodePosition::pack_height(12.0, 2.0, 10.0), 65535);
    assert_eq!(AiNodePosition::pack_height(40.0, 2.0, 10.0), 65535);
    assert_eq!(AiNodePosition::pack_height(-1.0, 2.0, 10.0), 0);
    assert_eq!(AiNodePosition::pack_height(5.0, 2.0, 0.0), 0);
    assert_eq!(AiNodePosition::unpack_height(65535, 2.0, 10.0), 12.0);
  }

  #[test]
  fn reads_and_writes_six_bytes() -> XrfResult {
    let position: AiNodePosition = AiNodePosition { x: -3, y: 40000, z: 7 };
    let mut writer: ChunkWriter = ChunkWriter::new();

    position.write::<XRayByteOrder>(&mut writer)?;
    assert_eq!(writer.buffer.len() as u64, AiNodePosition::SERIALIZED_SIZE);

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;

    assert_eq!(AiNodePosition::read::<XRayByteOrder, _>(&mut reader)?, position);

    Ok(())
  }
}
