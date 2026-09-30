use serde::Serialize;
use xrf_spawn::ClsId;
use xrf_visual::VisualTransform;

use crate::plugins::levels::state::selection::level_spawn_category::LevelSpawnCategory;

/// One spawned object the viewer draws: what it is, where it stands, and which visual it stands as.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSpawnObject {
  /// Its place among the level's spawned objects, which names it to the backend.
  pub index: u32,
  pub name: String,
  pub section: String,
  pub clsid: ClsId,
  /// Absent for an object without one (`INVALID_STORY_ID`).
  pub story_id: Option<u32>,
  pub category: LevelSpawnCategory,
  /// The visual, by its index among the description's visuals.
  pub visual: u32,
  /// The object's `XFORM`, in renderer space.
  pub transform: VisualTransform,
}
