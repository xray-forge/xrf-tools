/// What kind of a level's spawned objects one is, by the class its spawn stores: the part of its group
/// (`RenderSpawnGroup`) a view switches by kind.
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
}
