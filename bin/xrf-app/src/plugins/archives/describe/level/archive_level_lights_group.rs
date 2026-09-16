use serde::Serialize;
use xrf_db::{LevelLight, LevelLightsChunk};

/// One chunk of a level's compiled light list, as the viewer reads it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelLightsGroup {
  /// The chunk id the compiler wrote the run under.
  pub id: u32,
  pub lights: usize,
  /// Lights the runtime would make a light source of, which are the point ones.
  pub point: usize,
  /// Whether the engine opens this chunk at all: `CLight_DB::LoadHemi` takes `fsL_HEADER` and ignores the rest.
  pub is_read_by_engine: bool,
  /// Whether the payload was a run of lights at all, rather than something kept verbatim.
  pub is_lights: bool,
}

impl ArchiveLevelLightsGroup {
  /// Every chunk of a light list, in the order the file holds them.
  pub fn of_all(chunks: &[LevelLightsChunk]) -> Vec<Self> {
    chunks.iter().map(Self::of).collect()
  }

  /// One chunk, taken over the lights it holds.
  fn of(chunk: &LevelLightsChunk) -> Self {
    let lights: &[LevelLight] = chunk.get_lights();

    Self {
      id: chunk.get_id(),
      lights: lights.len(),
      point: lights.iter().filter(|light| light.is_point()).count(),
      is_read_by_engine: chunk.get_id() == LevelLightsChunk::HEMI_CHUNK_ID,
      is_lights: matches!(chunk, LevelLightsChunk::Lights { .. }),
    }
  }
}
