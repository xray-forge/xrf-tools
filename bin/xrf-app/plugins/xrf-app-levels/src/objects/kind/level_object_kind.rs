use serde::{Deserialize, Serialize};
use xrf_spawn::{AlifeObject, AlifeObjectInherited};

use crate::objects::{LEVEL_OBJECT_KINDS, LevelObjectGroup, LevelObjectKindEntry};
use crate::spawn::LevelSpawnCategory;

/// What a level object is, which the viewer lists, shows and draws helpers by.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelObjectKind {
  Props,
  Items,
  Weapons,
  Lamps,
  /// Where the actor starts.
  Actor,
  Stalkers,
  /// Monsters, crows and phantoms.
  Creatures,
  Traders,
  Squads,
  SmartTerrains,
  Restrictors,
  /// Anomalous and torrid zones.
  Anomalies,
  LevelChangers,
  /// Ladders.
  Climbables,
  SmartCovers,
  /// Patrol paths, and the points along them.
  Patrols,
  /// The game graph's vertices on the level.
  GraphPoints,
  /// Particle systems `level.ps_static` plants for a single player game.
  StaticParticles,
  /// Sounds `level.snd_static` plays where they stand.
  StaticSounds,
  /// The reverb environments' boxes of `level.snd_env`.
  SoundEnvironments,
  /// The volumetric fog's boxes of `level.fog_vol`.
  FogVolumes,
  /// The spheres of `level.env_mod` that move the weather toward their own.
  EnvModifiers,
  /// Where a multiplayer game respawns actors, artefacts and items (`level.game`); a single player game reads none.
  RespawnPoints,
  /// The level's static lights, the sun aside.
  Lights,
}

impl LevelObjectKind {
  /// The kind a spawned object is listed as, or `None` for one the level holds nothing of: a lamp the engine spawns
  /// on R1 alone, an object drawn as a visual it does not name, or a graph point, which the game graph keeps.
  pub fn of(object: &AlifeObject) -> Option<Self> {
    let inherited: &AlifeObjectInherited = &object.inherited;

    if let Some(category) = LevelSpawnCategory::of(inherited) {
      return inherited.get_visual().map(|_| category.into());
    }

    match inherited {
      AlifeObjectInherited::SeActor(_) => Some(Self::Actor),
      AlifeObjectInherited::SeStalker(_) => Some(Self::Stalkers),
      AlifeObjectInherited::SeMonster(_)
      | AlifeObjectInherited::CseAlifeCreatureCrow(_)
      | AlifeObjectInherited::CseAlifeCreaturePhantom(_) => Some(Self::Creatures),
      AlifeObjectInherited::CseAlifeTrader(_) => Some(Self::Traders),
      AlifeObjectInherited::SimSquadScripted(_) => Some(Self::Squads),
      AlifeObjectInherited::SeSmartTerrain(_) => Some(Self::SmartTerrains),
      AlifeObjectInherited::CseAlifeSpaceRestrictor(_) => Some(Self::Restrictors),
      AlifeObjectInherited::CseAlifeAnomalousZone(_) | AlifeObjectInherited::CseAlifeTorridZone(_) => {
        Some(Self::Anomalies)
      }
      AlifeObjectInherited::SeLevelChanger(_) => Some(Self::LevelChangers),
      AlifeObjectInherited::CseAlifeObjectClimable(_) => Some(Self::Climbables),
      AlifeObjectInherited::SeSmartCover(_) => Some(Self::SmartCovers),
      // The game graph lists the vertices a graph point was compiled into, which the level holds instead.
      AlifeObjectInherited::CseAlifeGraphPoint(_) => None,
      // The drawn classes reach here only where their category leaves them out: a lamp spawned on R1 alone.
      AlifeObjectInherited::CseAlifeObjectBreakable(_)
      | AlifeObjectInherited::CseAlifeZoneVisual(_)
      | AlifeObjectInherited::CseAlifeCar(_)
      | AlifeObjectInherited::CseAlifeObjectPhysic(_)
      | AlifeObjectInherited::CseAlifeHelicopter(_)
      | AlifeObjectInherited::CseAlifeInventoryBox(_)
      | AlifeObjectInherited::CseAlifeObjectHangingLamp(_)
      | AlifeObjectInherited::CseAlifeItem(_)
      | AlifeObjectInherited::CseAlifeItemExplosive(_)
      | AlifeObjectInherited::CseAlifeItemPda(_)
      | AlifeObjectInherited::CseAlifeItemAmmo(_)
      | AlifeObjectInherited::CseAlifeItemGrenade(_)
      | AlifeObjectInherited::CseAlifeItemArtefact(_)
      | AlifeObjectInherited::CseAlifeItemWeapon(_)
      | AlifeObjectInherited::CseAlifeItemDetector(_)
      | AlifeObjectInherited::CseAlifeItemHelmet(_)
      | AlifeObjectInherited::CseAlifeItemCustomOutfit(_)
      | AlifeObjectInherited::CseAlifeItemWeaponShotgun(_)
      | AlifeObjectInherited::CseAlifeItemWeaponMagazined(_)
      | AlifeObjectInherited::CseAlifeItemWeaponMagazinedWGl(_)
      | AlifeObjectInherited::CseAlifeItemBolt(_)
      | AlifeObjectInherited::CseAlifeItemTorch(_)
      | AlifeObjectInherited::CseAlifeItemDocument(_)
      | AlifeObjectInherited::CseAlifeDynamicObjectVisual(_)
      | AlifeObjectInherited::CseAlifeObjectProjector(_)
      | AlifeObjectInherited::CseAlifePhSkeletonObject(_)
      | AlifeObjectInherited::CseAlifeStationaryMGun(_) => None,
    }
  }

  /// Its row of the kind table, which lists every kind in the order they are declared.
  pub fn get_entry(self) -> &'static LevelObjectKindEntry {
    &LEVEL_OBJECT_KINDS[self as usize]
  }

  /// The family it is listed under.
  pub fn get_group(self) -> LevelObjectGroup {
    self.get_entry().group
  }
}

impl From<LevelSpawnCategory> for LevelObjectKind {
  fn from(category: LevelSpawnCategory) -> Self {
    match category {
      LevelSpawnCategory::Props => Self::Props,
      LevelSpawnCategory::Items => Self::Items,
      LevelSpawnCategory::Weapons => Self::Weapons,
      LevelSpawnCategory::Lamps => Self::Lamps,
    }
  }
}
