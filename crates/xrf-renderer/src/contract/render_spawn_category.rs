use serde::{Deserialize, Serialize};

/// Which group of a level's spawned objects one is drawn in, by the class its spawn stores.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderSpawnCategory {
  /// Physics objects, breakables, inventory boxes and vehicles.
  Props,
  /// Every inventory item but weapons.
  Items,
  /// Firearms and grenades.
  Weapons,
  Lamps,
}

impl RenderSpawnCategory {
  /// The visibility group its objects are culled by, from one.
  pub const fn get_group(self) -> u32 {
    match self {
      RenderSpawnCategory::Props => 1,
      RenderSpawnCategory::Items => 2,
      RenderSpawnCategory::Weapons => 3,
      RenderSpawnCategory::Lamps => 4,
    }
  }
}
