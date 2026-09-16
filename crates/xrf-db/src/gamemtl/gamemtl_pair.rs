use byteorder::{ByteOrder, ReadBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, find_required_chunk_by_id};
use xrf_error::XrfResult;

/// What a pair declares for itself rather than inheriting, by the bit `SGameMtlPair` names it under.
pub const GAMEMTL_PAIR_PROPERTIES: [(u32, &str); 5] = [
  (1 << 1, "breaking sounds"),
  (1 << 2, "step sounds"),
  (1 << 4, "collide sounds"),
  (1 << 5, "collide particles"),
  (1 << 6, "collide marks"),
];

/// What happens when two materials meet, `SGameMtlPair` (`xrMaterialSystem/GameMtlLib_Engine.cpp`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameMtlPair {
  /// The two materials, by their library numbers rather than their names.
  pub material_a: u32,
  pub material_b: u32,
  pub id: u32,
  /// The pair this one takes its unset properties from; `0xFFFFFFFF` is no parent.
  pub parent_id: u32,
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

  /// Reads one pair from the chunk holding it.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk the engine asserts on is absent, or a chunk does not hold what it should.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    let mut pair: ChunkReader<D> = find_required_chunk_by_id(&chunks, Self::PAIR_CHUNK_ID)?;
    let mut collide: ChunkReader<D> = find_required_chunk_by_id(&chunks, Self::COLLIDE_CHUNK_ID)?;

    Ok(Self {
      material_a: pair.read_u32::<T>()?,
      material_b: pair.read_u32::<T>()?,
      id: pair.read_u32::<T>()?,
      parent_id: pair.read_u32::<T>()?,
      own_properties: pair.read_u32::<T>()?,
      breaking_sounds: find_required_chunk_by_id(&chunks, Self::BREAKING_CHUNK_ID)?.read_w1251_string()?,
      step_sounds: find_required_chunk_by_id(&chunks, Self::STEP_CHUNK_ID)?.read_w1251_string()?,
      collide_sounds: collide.read_w1251_string()?,
      collide_particles: collide.read_w1251_string()?,
      collide_marks: collide.read_w1251_string()?,
    })
  }

  /// Whether the pair declares its own behaviour rather than taking another pair's.
  pub const fn has_parent(&self) -> bool {
    self.parent_id != Self::NO_PARENT_ID
  }

  /// What the pair declares for itself, named.
  pub fn get_own_properties(&self) -> Vec<&'static str> {
    GAMEMTL_PAIR_PROPERTIES
      .into_iter()
      .filter_map(|(mask, name)| (self.own_properties & mask != 0).then_some(name))
      .collect()
  }
}
