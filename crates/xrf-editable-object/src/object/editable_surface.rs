use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// One surface of an editable object, `CSurface` as `EOBJ_CHUNK_SURFACES3` stores it: what a face's material is made
/// of, named so a mesh's faces can point at it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableSurface {
  /// The name a mesh's face lists name it by.
  pub name: String,
  /// The engine shader the game draws it with, `def_shaders\def_vertex` and the like.
  pub shader: String,
  /// The compiler shader xrLC lights it with when it is level geometry.
  pub compiler_shader: String,
  /// The game material that sets its sound, marks and physics.
  pub game_material: String,
  pub texture: String,
  /// The UV map of a mesh this surface's texture is laid out by.
  pub vmap: String,
  /// `CSurface::m_Flags`: bit 0 is two-sided.
  pub flags: u32,
  /// The vertex format, `D3DFVF_*` bits.
  pub fvf: u32,
  /// Texture coordinate sets: the SDK writes 1 and refuses less.
  pub texture_coordinate_count: u32,
}

impl EditableSurface {
  /// Six empty strings and three numbers, the least a surface takes.
  pub const MIN_SERIALIZED_SIZE: u64 = 6 + 3 * 4;

  /// `CSurface::sf2Sided`.
  pub const TWO_SIDED_FLAG: u32 = 1;

  /// `D3DFVF_XYZ | D3DFVF_NORMAL | D3DFVF_TEX1`: a position, a normal and one texture coordinate set, the vertex
  /// format the Actor Editor gives a model's surfaces.
  pub const POSITION_NORMAL_UV_FVF: u32 = 0x112;

  /// The name a surface's UV map goes by, `Texture`, which the editor matches to the mesh's `texture` map regardless
  /// of case.
  pub const UV_MAP_NAME: &'static str = "Texture";

  /// Whether both of its sides are drawn.
  pub const fn is_two_sided(&self) -> bool {
    self.flags & Self::TWO_SIDED_FLAG != 0
  }
}

impl ChunkReadWrite for EditableSurface {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      name: reader.read_w1251_string()?,
      shader: reader.read_w1251_string()?,
      compiler_shader: reader.read_w1251_string()?,
      game_material: reader.read_w1251_string()?,
      texture: reader.read_w1251_string()?,
      vmap: reader.read_w1251_string()?,
      flags: reader.read_u32::<T>()?,
      fvf: reader.read_u32::<T>()?,
      texture_coordinate_count: reader.read_u32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.name)?;
    writer.write_w1251_string(&self.shader)?;
    writer.write_w1251_string(&self.compiler_shader)?;
    writer.write_w1251_string(&self.game_material)?;
    writer.write_w1251_string(&self.texture)?;
    writer.write_w1251_string(&self.vmap)?;
    writer.write_u32::<T>(self.flags)?;
    writer.write_u32::<T>(self.fvf)?;
    writer.write_u32::<T>(self.texture_coordinate_count)?;

    Ok(())
  }
}
