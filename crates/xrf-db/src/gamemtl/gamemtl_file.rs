use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, find_optional_chunk_by_id, find_required_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::gamemtl::gamemtl_material::GameMtlMaterial;
use crate::gamemtl::gamemtl_pair::GameMtlPair;

/// The game material library, `gamemtl.xr`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameMtlFile {
  pub version: u16,
  /// The next number the editor would hand out, which is one past the highest ever used rather than a count.
  pub material_index: u32,
  pub material_pair_index: u32,
  pub materials: Vec<GameMtlMaterial>,
  pub pairs: Vec<GameMtlPair>,
}

impl GameMtlFile {
  /// `GAMEMTL_VERSION_COP`, the only version `CGameMtlLibrary::Load` accepts; it logs and gives up on any other.
  pub const CURRENT_VERSION: u16 = 1;

  pub const VERSION_CHUNK_ID: u32 = 0x1000;
  pub const AUTOINCREMENT_CHUNK_ID: u32 = 0x1001;
  pub const MATERIALS_CHUNK_ID: u32 = 0x1002;
  pub const PAIRS_CHUNK_ID: u32 = 0x1003;

  /// Reads a material library from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a material library this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Game material library was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads a material library from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a material library this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived library arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a material library this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk is absent, the version is one the engine refuses, or a material cannot be read.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let version: u16 = find_required_chunk_by_id(&chunks, Self::VERSION_CHUNK_ID)?.read_u16::<T>()?;

    if version != Self::CURRENT_VERSION {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected game material library version {version} on read, only version {} is implemented",
        Self::CURRENT_VERSION
      )));
    }

    let mut autoincrement: ChunkReader<D> = find_required_chunk_by_id(&chunks, Self::AUTOINCREMENT_CHUNK_ID)?;

    Ok(Self {
      version,
      material_index: autoincrement.read_u32::<T>()?,
      material_pair_index: autoincrement.read_u32::<T>()?,
      materials: match find_optional_chunk_by_id(&chunks, Self::MATERIALS_CHUNK_ID) {
        Some(mut chunk) => Self::read_materials::<T, D>(&mut chunk)?,
        None => Vec::new(),
      },
      pairs: match find_optional_chunk_by_id(&chunks, Self::PAIRS_CHUNK_ID) {
        Some(mut chunk) => Self::read_pairs::<T, D>(&mut chunk)?,
        None => Vec::new(),
      },
    })
  }

  /// Reads every material, one per child chunk, in the order the library numbers them.
  fn read_materials<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<GameMtlMaterial>> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut materials: Vec<GameMtlMaterial> = Vec::with_capacity(chunks.len());

    for mut chunk in chunks {
      materials.push(GameMtlMaterial::read::<T, D>(&mut chunk)?);
    }

    Ok(materials)
  }

  /// Reads every pair, one per child chunk.
  fn read_pairs<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<GameMtlPair>> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut pairs: Vec<GameMtlPair> = Vec::with_capacity(chunks.len());

    for mut chunk in chunks {
      pairs.push(GameMtlPair::read::<T, D>(&mut chunk)?);
    }

    Ok(pairs)
  }
}

impl GameMtlFile {
  /// The material a library number names, or `None` for a number nothing defines.
  pub fn find_material(&self, id: u32) -> Option<&GameMtlMaterial> {
    self.materials.iter().find(|material| material.id == id)
  }

  /// Pairs that declare nothing of their own, which take everything from the pair they name as parent.
  pub fn get_inheriting_pairs_count(&self) -> usize {
    self.pairs.iter().filter(|pair| pair.has_parent()).count()
  }
}
