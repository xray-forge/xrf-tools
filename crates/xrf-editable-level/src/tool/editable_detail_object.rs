use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};

/// One detail object of the detail tool, `EDetail` as `EDetail::Save` writes it: the library object it plants, by
/// reference, and how it is planted.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableDetailObject {
  /// The library object, by its path under `$objects$` without the extension, such as `detail\trava_green`.
  pub reference: String,
  /// The range each planted copy is scaled within.
  pub min_scale: f32,
  pub max_scale: f32,
  /// How densely it is planted, against the tool's density.
  pub density_factor: f32,
  /// `DO_NO_WAVING` and the rest of `EDetail`'s flags.
  pub flags: u32,
}

impl EditableDetailObject {
  /// `DETOBJ_VERSION`, the only one the editor loads.
  pub const VERSION: u32 = 1;

  /// `DO_NO_WAVING`: the object stands still in the wind.
  pub const NO_WAVING_FLAG: u32 = 1;

  /// The density factor a new object takes, and one read without its chunk.
  pub const DEFAULT_DENSITY_FACTOR: f32 = 1.0;

  const VERSION_CHUNK_ID: u32 = 0x1000;
  const REFERENCE_CHUNK_ID: u32 = 0x0101;
  const SCALE_LIMITS_CHUNK_ID: u32 = 0x0102;
  const DENSITY_FACTOR_CHUNK_ID: u32 = 0x0103;
  const FLAGS_CHUNK_ID: u32 = 0x0104;

  /// An object planting `reference` scaled within `min_scale..max_scale`, at the default density, flagged `flags`.
  pub fn new(reference: impl Into<String>, min_scale: f32, max_scale: f32, flags: u32) -> Self {
    Self {
      reference: reference.into(),
      min_scale,
      max_scale,
      density_factor: Self::DEFAULT_DENSITY_FACTOR,
      flags,
    }
  }

  /// The object as `EDetail::Load` reads its chunk, but for the editor's fixes to a zero minimum scale and a maximum
  /// below it, which happen as it loads and are kept as stored here.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk the editor asserts on is missing or the version is another.
  pub(crate) fn read<T: ByteOrder>(reader: &mut ChunkReader<InMemoryChunkDataSource>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = reader.read_children()?;
    let version: u32 = ChunkReader::find_required_by_id(&chunks, Self::VERSION_CHUNK_ID)?.read_u32::<T>()?;

    if version != Self::VERSION {
      return Err(XrfError::new_parsing_error(format!(
        "Detail object version {version} is not {}",
        Self::VERSION
      )));
    }

    let reference: String = ChunkReader::find_required_by_id(&chunks, Self::REFERENCE_CHUNK_ID)?.read_w1251_string()?;
    let mut scale: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::find_required_by_id(&chunks, Self::SCALE_LIMITS_CHUNK_ID)?;
    let (min_scale, max_scale) = (scale.read_f32::<T>()?, scale.read_f32::<T>()?);
    let mut object: Self = Self::new(reference, min_scale, max_scale, 0);

    for mut chunk in chunks {
      match chunk.id {
        Self::DENSITY_FACTOR_CHUNK_ID => object.density_factor = chunk.read_f32::<T>()?,
        Self::FLAGS_CHUNK_ID => object.flags = chunk.read_u32::<T>()?,
        _ => {}
      }
    }

    Ok(object)
  }

  /// The object's chunks as `EDetail::Save` writes them.
  ///
  /// # Errors
  ///
  /// Returns an error when the reference does not encode.
  pub(crate) fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut chunk = |id: u32, write: &dyn Fn(&mut ChunkWriter) -> XrfResult| -> XrfResult {
      let mut child: ChunkWriter = ChunkWriter::new();

      write(&mut child)?;
      child.flush_chunk_into::<T>(&mut writer.buffer, id)?;

      Ok(())
    };

    chunk(Self::VERSION_CHUNK_ID, &|child| {
      Ok(child.write_u32::<T>(Self::VERSION)?)
    })?;
    chunk(Self::REFERENCE_CHUNK_ID, &|child| {
      child.write_w1251_string(&self.reference)?;
      Ok(())
    })?;
    chunk(Self::SCALE_LIMITS_CHUNK_ID, &|child| {
      child.write_f32::<T>(self.min_scale)?;
      Ok(child.write_f32::<T>(self.max_scale)?)
    })?;
    chunk(Self::DENSITY_FACTOR_CHUNK_ID, &|child| {
      Ok(child.write_f32::<T>(self.density_factor)?)
    })?;
    chunk(Self::FLAGS_CHUNK_ID, &|child| Ok(child.write_u32::<T>(self.flags)?))
  }
}
