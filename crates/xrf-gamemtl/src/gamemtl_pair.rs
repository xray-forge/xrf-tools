use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// What happens when two materials meet, `SGameMtlPair` (`xrMaterialSystem/GameMtlLib_Engine.cpp`).
///
/// The lists are stored resolved: the editor copies what a pair inherits into its own strings on save, so the engine
/// plays them as they stand and never walks `parent_id`, which only the editor reads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameMtlPair {
  /// The two materials, by their library numbers rather than their names.
  pub material_a: u32,
  pub material_b: u32,
  pub id: u32,
  /// The pair the editor takes unset properties from; `NO_PARENT_ID` is no parent.
  pub parent_id: u32,
  /// The `PROPERTY_*` bits the pair sets itself rather than inherits.
  pub own_properties: u32,
  pub breaking_sounds: String,
  pub step_sounds: String,
  pub collide_sounds: String,
  pub collide_particles: String,
  pub collide_marks: String,
}

impl GameMtlPair {
  pub const PAIR_CHUNK_ID: u32 = 0x1000;
  pub const BREAKING_CHUNK_ID: u32 = 0x1002;
  pub const STEP_CHUNK_ID: u32 = 0x1003;
  pub const COLLIDE_CHUNK_ID: u32 = 0x1005;

  /// `GAMEMTL_NONE_ID`, which a pair inheriting from nothing carries.
  pub const NO_PARENT_ID: u32 = u32::MAX;

  pub const PROPERTY_BREAKING_SOUNDS: u32 = 1 << 1;
  pub const PROPERTY_STEP_SOUNDS: u32 = 1 << 2;
  pub const PROPERTY_COLLIDE_SOUNDS: u32 = 1 << 4;
  pub const PROPERTY_COLLIDE_PARTICLES: u32 = 1 << 5;
  pub const PROPERTY_COLLIDE_MARKS: u32 = 1 << 6;

  /// Every property a pair can set itself, in bit order. Bits 0 and 3 are retired flotation and collide-sound bits.
  pub const PROPERTY_NAMES: [(u32, &'static str); 5] = [
    (Self::PROPERTY_BREAKING_SOUNDS, "breaking sounds"),
    (Self::PROPERTY_STEP_SOUNDS, "step sounds"),
    (Self::PROPERTY_COLLIDE_SOUNDS, "collide sounds"),
    (Self::PROPERTY_COLLIDE_PARTICLES, "collide particles"),
    (Self::PROPERTY_COLLIDE_MARKS, "collide marks"),
  ];

  /// Entries a sound list may hold before the engine's load asserts, `GAMEMTL_SUBITEM_COUNT + 2`.
  pub const MAX_SOUNDS: usize = 6;
  /// Entries a particle or mark list may hold before the engine's load asserts, `GAMEMTL_SUBITEM_COUNT`.
  pub const MAX_EFFECTS: usize = 4;

  /// Reads one pair from the chunk holding it.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk the engine asserts on is absent, or a chunk holds more or less than it should.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    let mut head: ChunkReader<D> = ChunkReader::find_required_by_id(&chunks, Self::PAIR_CHUNK_ID)?;
    let mut collide: ChunkReader<D> = ChunkReader::find_required_by_id(&chunks, Self::COLLIDE_CHUNK_ID)?;

    let pair: Self = Self {
      material_a: head.read_u32::<T>()?,
      material_b: head.read_u32::<T>()?,
      id: head.read_u32::<T>()?,
      parent_id: head.read_u32::<T>()?,
      own_properties: head.read_u32::<T>()?,
      breaking_sounds: ChunkReader::find_required_by_id(&chunks, Self::BREAKING_CHUNK_ID)?
        .read_w1251_string_payload()?,
      step_sounds: ChunkReader::find_required_by_id(&chunks, Self::STEP_CHUNK_ID)?.read_w1251_string_payload()?,
      collide_sounds: collide.read_w1251_string()?,
      collide_particles: collide.read_w1251_string()?,
      collide_marks: collide.read_w1251_string()?,
    };

    head.assert_read("All data should be read in game material pair chunk")?;
    collide.assert_read("All data should be read in game material pair collide chunk")?;

    Ok(pair)
  }

  /// Writes the pair back in the chunk order the editor wrote it in.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut pair: ChunkWriter = ChunkWriter::new();

    pair.write_u32::<T>(self.material_a)?;
    pair.write_u32::<T>(self.material_b)?;
    pair.write_u32::<T>(self.id)?;
    pair.write_u32::<T>(self.parent_id)?;
    pair.write_u32::<T>(self.own_properties)?;
    pair.flush_chunk_into::<T>(&mut writer.buffer, Self::PAIR_CHUNK_ID)?;

    let mut breaking: ChunkWriter = ChunkWriter::new();

    breaking.write_w1251_string(&self.breaking_sounds)?;
    breaking.flush_chunk_into::<T>(&mut writer.buffer, Self::BREAKING_CHUNK_ID)?;

    let mut step: ChunkWriter = ChunkWriter::new();

    step.write_w1251_string(&self.step_sounds)?;
    step.flush_chunk_into::<T>(&mut writer.buffer, Self::STEP_CHUNK_ID)?;

    let mut collide: ChunkWriter = ChunkWriter::new();

    collide.write_w1251_string(&self.collide_sounds)?;
    collide.write_w1251_string(&self.collide_particles)?;
    collide.write_w1251_string(&self.collide_marks)?;
    collide.flush_chunk_into::<T>(&mut writer.buffer, Self::COLLIDE_CHUNK_ID)?;

    Ok(())
  }

  /// Whether the pair is the meeting of the two materials, in either order, as `SGameMtlPair::IsPair` asks.
  pub const fn is_pair(&self, material_a: u32, material_b: u32) -> bool {
    (self.material_a == material_a && self.material_b == material_b)
      || (self.material_a == material_b && self.material_b == material_a)
  }

  /// Whether the editor takes the pair's unset properties from another pair, rather than from nothing.
  pub const fn has_parent(&self) -> bool {
    self.parent_id != Self::NO_PARENT_ID
  }

  /// What the pair sets itself, named.
  pub fn get_own_properties(&self) -> Vec<&'static str> {
    Self::PROPERTY_NAMES
      .into_iter()
      .filter_map(|(mask, name)| (self.own_properties & mask != 0).then_some(name))
      .collect()
  }

  /// The entries the engine loads from one of a pair's lists, trimmed as `_GetItem` trims them.
  ///
  /// Counted as `_GetItemCount` counts, which stops at the first empty entry between two commas: `a,,b` is `a` and an
  /// empty entry, and `b` is never loaded.
  pub fn split_list(list: &str) -> Vec<&str> {
    list
      .split(',')
      .take(Self::count_list(list))
      .map(|item| item.trim_matches(|character: char| character <= ' '))
      .collect()
  }

  /// `_GetItemCount` over a comma-separated list.
  fn count_list(list: &str) -> usize {
    if list.is_empty() {
      return 0;
    }

    let mut count: usize = 0;
    let mut rest: &str = list;

    while let Some(position) = rest.find(',') {
      rest = &rest[position + 1..];
      count += 1;

      if rest.starts_with(',') {
        break;
      }
    }

    count + usize::from(!rest.is_empty())
  }
}
