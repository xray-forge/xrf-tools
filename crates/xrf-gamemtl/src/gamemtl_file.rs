use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{format_path, to_format_size};

use crate::gamemtl_material::GameMtlMaterial;
use crate::gamemtl_pair::GameMtlPair;
use crate::gamemtl_version::GameMtlVersion;

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
  /// `GAMEMTL_FILENAME`, the library's path under `$game_data$`, which the engine reads and no other.
  pub const FILE_NAME: &'static str = "gamemtl.xr";

  /// `GAMEMTL_VERSION_COP`, the only version `CGameMtlLibrary::Load` accepts; it logs and gives up on any other.
  pub const CURRENT_VERSION: u16 = 1;

  pub const VERSION_CHUNK_ID: u32 = 0x1000;
  pub const AUTOINCREMENT_CHUNK_ID: u32 = 0x1001;
  pub const MATERIALS_CHUNK_ID: u32 = 0x1002;
  pub const PAIRS_CHUNK_ID: u32 = 0x1003;

  /// A library of the given materials and pairs, numbering on from the highest number either holds as the editor would.
  pub fn new(materials: Vec<GameMtlMaterial>, pairs: Vec<GameMtlPair>) -> Self {
    Self {
      version: Self::CURRENT_VERSION,
      material_index: Self::get_next_number(materials.iter().map(|material| material.id)),
      material_pair_index: Self::get_next_number(pairs.iter().map(|pair| pair.id)),
      materials,
      pairs,
    }
  }

  /// One past the highest of the numbers, the first a library handing them out in order would give next.
  fn get_next_number(numbers: impl Iterator<Item = u32>) -> u32 {
    numbers.max().map_or(0, |number| number.saturating_add(1))
  }

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
  /// Returns an error when a chunk is absent or holds more or less than it should, the version is one the engine
  /// refuses, or a material or pair cannot be read.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let version: u16 = ChunkReader::find_required_by_id(&chunks, Self::VERSION_CHUNK_ID)?.read_u16_payload::<T>()?;

    if version != Self::CURRENT_VERSION {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected game material library version {version} on read, only version {} is implemented",
        Self::CURRENT_VERSION
      )));
    }

    let mut autoincrement: ChunkReader<D> = ChunkReader::find_required_by_id(&chunks, Self::AUTOINCREMENT_CHUNK_ID)?;
    let material_index: u32 = autoincrement.read_u32::<T>()?;
    let material_pair_index: u32 = autoincrement.read_u32::<T>()?;

    autoincrement.assert_read("All data should be read in game material numbering chunk")?;

    Ok(Self {
      version,
      material_index,
      material_pair_index,
      materials: match ChunkReader::find_optional_by_id(&chunks, Self::MATERIALS_CHUNK_ID) {
        Some(mut chunk) => Self::read_materials::<T, D>(&mut chunk)?,
        None => Vec::new(),
      },
      pairs: match ChunkReader::find_optional_by_id(&chunks, Self::PAIRS_CHUNK_ID) {
        Some(mut chunk) => Self::read_pairs::<T, D>(&mut chunk)?,
        None => Vec::new(),
      },
    })
  }

  /// Writes the library back in the chunk order `CGameMtlLibrary::Load` reads it in.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes, or a list is too long to number its chunks.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut version: ChunkWriter = ChunkWriter::new();

    version.write_u16::<T>(self.version)?;
    version.flush_chunk_into::<T>(&mut writer.buffer, Self::VERSION_CHUNK_ID)?;

    let mut autoincrement: ChunkWriter = ChunkWriter::new();

    autoincrement.write_u32::<T>(self.material_index)?;
    autoincrement.write_u32::<T>(self.material_pair_index)?;
    autoincrement.flush_chunk_into::<T>(&mut writer.buffer, Self::AUTOINCREMENT_CHUNK_ID)?;

    let mut materials: ChunkWriter = ChunkWriter::new();

    for (index, material) in self.materials.iter().enumerate() {
      let mut chunk: ChunkWriter = ChunkWriter::new();

      material.write::<T>(&mut chunk)?;
      chunk.flush_chunk_into::<T>(&mut materials.buffer, to_format_size(index, "game material chunk")?)?;
    }

    materials.flush_chunk_into::<T>(&mut writer.buffer, Self::MATERIALS_CHUNK_ID)?;

    let mut pairs: ChunkWriter = ChunkWriter::new();

    for (index, pair) in self.pairs.iter().enumerate() {
      let mut chunk: ChunkWriter = ChunkWriter::new();

      pair.write::<T>(&mut chunk)?;
      chunk.flush_chunk_into::<T>(&mut pairs.buffer, to_format_size(index, "game material pair chunk")?)?;
    }

    pairs.flush_chunk_into::<T>(&mut writer.buffer, Self::PAIRS_CHUNK_ID)?;

    Ok(())
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

  /// The material a name names regardless of ASCII case, as `CGameMtlLibrary::GetMaterial` matches a bone's or a
  /// section's material, or `None` for a name nothing defines.
  pub fn find_material_by_name(&self, name: &str) -> Option<&GameMtlMaterial> {
    self
      .materials
      .iter()
      .find(|material| material.name.eq_ignore_ascii_case(name))
  }

  /// The pair two materials make in either order, or `None` where they have none, as happens for two static materials.
  pub fn find_pair(&self, material_a: u32, material_b: u32) -> Option<&GameMtlPair> {
    self.pairs.iter().find(|pair| pair.is_pair(material_a, material_b))
  }

  /// A pair by its materials' names, `a - b` as `SGameMtlPair::dbg_Name` gives it, a number standing in for a material
  /// the library does not define.
  pub fn get_pair_label(&self, pair: &GameMtlPair) -> String {
    let name = |id: u32| {
      self
        .find_material(id)
        .map_or_else(|| id.to_string(), |material| material.name.clone())
    };

    format!("{} - {}", name(pair.material_a), name(pair.material_b))
  }

  /// Every entry one of the pairs' lists names, as the engine splits it, with how many pairs name it; empty entries are
  /// left out.
  pub fn get_list_entries<'a>(&'a self, list: impl Fn(&'a GameMtlPair) -> &'a str) -> BTreeMap<&'a str, usize> {
    let mut entries: BTreeMap<&str, usize> = BTreeMap::new();

    for pair in &self.pairs {
      for entry in GameMtlPair::split_list(list(pair)) {
        if !entry.is_empty() {
          *entries.entry(entry).or_default() += 1;
        }
      }
    }

    entries
  }

  /// Materials flagged dynamic, which moving objects and their bones are made of and pairs are made for.
  pub fn get_dynamic_materials_count(&self) -> usize {
    self.materials.iter().filter(|material| material.is_dynamic()).count()
  }

  /// Pairs naming a parent pair the editor takes their unset properties from.
  pub fn get_inheriting_pairs_count(&self) -> usize {
    self.pairs.iter().filter(|pair| pair.has_parent()).count()
  }

  /// The game `CGameMtlLibrary::Load` takes the library for, by its newest material; Shadow of Chernobyl when it has
  /// none.
  pub fn get_game_version(&self) -> GameMtlVersion {
    self
      .materials
      .iter()
      .map(GameMtlMaterial::get_game_version)
      .max()
      .unwrap_or(GameMtlVersion::ShadowOfChernobyl)
  }
}
