use serde::{Deserialize, Serialize};

use crate::contract::render_spawn_category::RenderSpawnCategory;

/// What of the level's world plays, rather than what is drawn of it: the weather's rain, bolts and wind, the campfires,
/// the ambient effects, and which groups of the spawned objects stream in.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderWorldToggles {
  /// Whether the weather's rain falls and wets surfaces.
  pub is_rainy: bool,
  /// Whether the weather's bolts strike.
  pub is_thundering: bool,
  /// Whether the weather's wind sways trees and grass.
  pub is_windy: bool,
  /// Whether its campfires burn, as `CZoneCampfire` starts, rather than smoulder out.
  pub is_campfire_lit: bool,
  /// Whether the weather's ambient effects play near the camera and bring their wind.
  pub is_ambient_played: bool,
  /// Which groups of the level's spawned objects are drawn.
  pub is_spawned_props: bool,
  pub is_spawned_items: bool,
  pub is_spawned_weapons: bool,
  pub is_spawned_lamps: bool,
  /// Whether the spawned objects a new game releases are drawn too, each with its group.
  pub is_spawned_released: bool,
}

impl Default for RenderWorldToggles {
  fn default() -> Self {
    Self {
      is_rainy: true,
      is_thundering: true,
      is_windy: true,
      is_campfire_lit: true,
      is_ambient_played: true,
      is_spawned_props: true,
      is_spawned_items: true,
      is_spawned_weapons: true,
      is_spawned_lamps: true,
      is_spawned_released: false,
    }
  }
}

impl RenderWorldToggles {
  /// Whether a group of spawned objects is drawn.
  pub fn is_spawned(&self, category: RenderSpawnCategory) -> bool {
    match category {
      RenderSpawnCategory::Props => self.is_spawned_props,
      RenderSpawnCategory::Items => self.is_spawned_items,
      RenderSpawnCategory::Weapons => self.is_spawned_weapons,
      RenderSpawnCategory::Lamps => self.is_spawned_lamps,
    }
  }
}
