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
  pub const ALL: [RenderSpawnCategory; 4] = [
    RenderSpawnCategory::Props,
    RenderSpawnCategory::Items,
    RenderSpawnCategory::Weapons,
    RenderSpawnCategory::Lamps,
  ];

  /// The spawn group its objects are shown and hidden by, from one.
  pub const fn get_group(self) -> u32 {
    match self {
      RenderSpawnCategory::Props => 1,
      RenderSpawnCategory::Items => 2,
      RenderSpawnCategory::Weapons => 3,
      RenderSpawnCategory::Lamps => 4,
    }
  }

  /// The spawn group its objects a new game releases are shown and hidden by, apart from the ones it keeps.
  pub const fn get_released_group(self) -> u32 {
    self.get_group() + 4
  }
}
