use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::{Aabb, Sphere, Vector3d};
use xrf_utils::narrow_format_size;

use crate::tool::editable_wallmark_vertex::EditableWallmarkVertex;

/// One decal the wall mark tool placed, `ESceneWallmarkTool::wallmark`: a lit triangle fan cut out of the geometry it
/// was shot at.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableWallmark {
  /// `flSelected`, which a new decal is placed with and the editor toggles as it is picked.
  pub flags: u8,
  /// The box around the fan, grown by `EPS_L`: what the editor picks the decal by.
  pub bbox: Aabb,
  /// The sphere around the fan, which the export and the renderer cull it by.
  pub bounds: Sphere,
  /// The size and turn the decal was shot with, recorded only; an older part keeps none and reads as 1.
  pub width: f32,
  pub height: f32,
  pub rotation: f32,
  pub vertices: Vec<EditableWallmarkVertex>,
}

impl EditableWallmark {
  /// Bytes a decal of version 3 occupies before its vertices: flags, box, sphere and their count; version 4 adds three
  /// floats.
  pub const OLD_FIXED_SIZE: u64 = 1 + 24 + 16 + 4;

  /// `flSelected`, which `AddWallmark` places a new decal with.
  pub const SELECTED: u8 = 1;

  /// `MAX_WALLMARK_VERTEX_COUNT`: the editor drops a decal of more on load.
  pub const MAX_VERTICES: usize = 512;

  /// How far `AddWallmark` grows the box past the fan, `EPS_L`.
  pub const BOX_GROWTH: f32 = 0.001;

  /// The box and sphere `AddWallmark` gives a fan: the sphere about the fan's box, then the box grown by
  /// [`Self::BOX_GROWTH`]. None for a fan of no vertices.
  pub fn get_bounds(vertices: &[EditableWallmarkVertex]) -> Option<(Aabb, Sphere)> {
    let fan: Aabb = Aabb::from_points(vertices.iter().map(|vertex| vertex.position))?;
    let growth: Vector3d = Vector3d::new(Self::BOX_GROWTH, Self::BOX_GROWTH, Self::BOX_GROWTH);

    Some((
      Aabb {
        min: &fan.min - &growth,
        max: &fan.max + &growth,
      },
      fan.get_bounding_sphere(),
    ))
  }

  /// One decal of a slot, as `LoadStream` reads it; `recorded` is whether the size and turn are stored, which they are
  /// from version 4.
  pub(crate) fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>, recorded: bool) -> XrfResult<Self> {
    let flags: u8 = reader.read_u8()?;
    let bbox: Aabb = reader.read_xr::<T, _>()?;
    let bounds: Sphere = reader.read_xr::<T, _>()?;
    let [width, height, rotation] = if recorded {
      [
        reader.read_f32::<T>()?,
        reader.read_f32::<T>()?,
        reader.read_f32::<T>()?,
      ]
    } else {
      [1.0; 3]
    };
    let count: u32 = reader.read_u32::<T>()?;
    let mut vertices: Vec<EditableWallmarkVertex> = reader.new_bounded_vec(
      count.into(),
      EditableWallmarkVertex::SERIALIZED_SIZE,
      "wall mark vertices",
    )?;

    for _ in 0..count {
      vertices.push(EditableWallmarkVertex::read::<T, D>(reader)?);
    }

    Ok(Self {
      flags,
      bbox,
      bounds,
      width,
      height,
      rotation,
      vertices,
    })
  }

  /// As `SaveStream` writes a decal.
  pub(crate) fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u8(self.flags)?;
    writer.write_xr::<T, _>(&self.bbox)?;
    writer.write_xr::<T, _>(&self.bounds)?;
    writer.write_f32::<T>(self.width)?;
    writer.write_f32::<T>(self.height)?;
    writer.write_f32::<T>(self.rotation)?;
    writer.write_u32::<T>(narrow_format_size(self.vertices.len(), "wall mark vertices")?)?;

    for vertex in &self.vertices {
      vertex.write::<T>(writer)?;
    }

    Ok(())
  }
}
