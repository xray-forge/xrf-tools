use serde::Serialize;
use xrf_spawn::AlifeObjectInherited;

/// What a spawned object is, by the engine class its spawn stores: what the viewer groups and toggles it by.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelSpawnCategory {
  /// Physics objects, breakables, inventory boxes, vehicles, and zones drawn as a model.
  Props,
  /// Every inventory item but a weapon.
  Items,
  /// Firearms and grenades.
  Weapons,
  /// Hanging lamps the engine spawns on R2.
  Lamps,
}

impl LevelSpawnCategory {
  /// The category an object is drawn in, or `None` for a class the viewer draws nothing of.
  pub fn of(inherited: &AlifeObjectInherited) -> Option<Self> {
    match inherited {
      AlifeObjectInherited::CseAlifeObjectPhysic(_)
      | AlifeObjectInherited::CseAlifeObjectBreakable(_)
      | AlifeObjectInherited::CseAlifeInventoryBox(_)
      | AlifeObjectInherited::CseAlifeCar(_)
      | AlifeObjectInherited::CseAlifeHelicopter(_)
      | AlifeObjectInherited::CseAlifeZoneVisual(_) => Some(Self::Props),
      AlifeObjectInherited::CseAlifeItem(_)
      | AlifeObjectInherited::CseAlifeItemExplosive(_)
      | AlifeObjectInherited::CseAlifeItemPda(_)
      | AlifeObjectInherited::CseAlifeItemAmmo(_)
      | AlifeObjectInherited::CseAlifeItemArtefact(_)
      | AlifeObjectInherited::CseAlifeItemDetector(_)
      | AlifeObjectInherited::CseAlifeItemHelmet(_)
      | AlifeObjectInherited::CseAlifeItemCustomOutfit(_) => Some(Self::Items),
      AlifeObjectInherited::CseAlifeItemWeapon(_)
      | AlifeObjectInherited::CseAlifeItemWeaponShotgun(_)
      | AlifeObjectInherited::CseAlifeItemWeaponMagazined(_)
      | AlifeObjectInherited::CseAlifeItemWeaponMagazinedWGl(_)
      | AlifeObjectInherited::CseAlifeItemGrenade(_) => Some(Self::Weapons),
      AlifeObjectInherited::CseAlifeObjectHangingLamp(lamp) => lamp.is_spawned_on_r2().then_some(Self::Lamps),
      // Characters, which stand animated and are not in a level's spawn but for the actor, and the logic volumes.
      AlifeObjectInherited::SeActor(_)
      | AlifeObjectInherited::CseAlifeTrader(_)
      | AlifeObjectInherited::CseAlifeObjectClimable(_)
      | AlifeObjectInherited::CseAlifeGraphPoint(_)
      | AlifeObjectInherited::CseAlifeSpaceRestrictor(_)
      | AlifeObjectInherited::SeSmartCover(_)
      | AlifeObjectInherited::CseAlifeAnomalousZone(_)
      | AlifeObjectInherited::CseAlifeTorridZone(_)
      | AlifeObjectInherited::SeSmartTerrain(_)
      | AlifeObjectInherited::SeLevelChanger(_) => None,
    }
  }
}
