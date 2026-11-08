/// Which group of a level's spawned objects one is drawn in, by the class its spawn stores.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
  pub const ALL: [Self; 4] = [Self::Props, Self::Items, Self::Weapons, Self::Lamps];

  /// The spawn group its objects are shown and hidden by, from one.
  pub const fn get_group(self) -> u32 {
    match self {
      Self::Props => 1,
      Self::Items => 2,
      Self::Weapons => 3,
      Self::Lamps => 4,
    }
  }

  /// The spawn group its objects a new game releases are shown and hidden by, apart from the ones it keeps.
  pub const fn get_released_group(self) -> u32 {
    self.get_group() + 4
  }
}
