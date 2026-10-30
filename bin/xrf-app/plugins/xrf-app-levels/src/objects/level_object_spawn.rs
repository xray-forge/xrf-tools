use serde::Serialize;
use xrf_spawn::ClsId;

use crate::spawn::LevelSpawnRelease;

/// What the spawn records of a level object it places: its section, class and story id, why a new game releases it,
/// and the visual it is drawn as.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelObjectSpawn {
  pub section: String,
  pub clsid: ClsId,
  /// Absent for an object without one (`INVALID_STORY_ID`).
  pub story_id: Option<u32>,
  /// Why a new game releases it, absent for an object it keeps; a released one is drawn only when asked.
  pub release: Option<LevelSpawnRelease>,
  /// The visual it is drawn as, by its index among the description's visuals; none for one drawn as nothing.
  pub visual: Option<u32>,
}
