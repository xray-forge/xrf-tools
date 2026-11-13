use std::ops::Index;

use derive_more::{Display, FromStr};
use enum_map::Enum;

use crate::data::alife::script_dialect::ScriptDialect;
use crate::data::meta::cls_id::ClsId;
use crate::data::meta::map::CLS_ID_TO_CLASS;

/// The server class a spawned object is read as: those Call of Pripyat creates from a spawn, by the engine's
/// `object_factory_register.cpp` and the scripts' `class_registrator.script`, and Clear Sky's script classes where its
/// own `class_registrator.script` registers one that saves other fields.
#[derive(Clone, Debug, Enum, PartialEq, Eq, FromStr, Display)]
pub enum AlifeClass {
  #[display("CseAlifeAnomalousZone")]
  CseAlifeAnomalousZone,
  #[display("CseAlifeCar")]
  CseAlifeCar,
  #[display("CseAlifeCreatureCrow")]
  CseAlifeCreatureCrow,
  #[display("CseAlifeCreaturePhantom")]
  CseAlifeCreaturePhantom,
  #[display("CseAlifeDynamicObjectVisual")]
  CseAlifeDynamicObjectVisual,
  #[display("CseAlifeGraphPoint")]
  CseAlifeGraphPoint,
  #[display("CseAlifeHelicopter")]
  CseAlifeHelicopter,
  #[display("CseAlifeInventoryBox")]
  CseAlifeInventoryBox,
  #[display("CseAlifeItem")]
  CseAlifeItem,
  #[display("CseAlifeItemAmmo")]
  CseAlifeItemAmmo,
  #[display("CseAlifeItemArtefact")]
  CseAlifeItemArtefact,
  #[display("CseAlifeItemBolt")]
  CseAlifeItemBolt,
  #[display("CseAlifeItemCustomOutfit")]
  CseAlifeItemCustomOutfit,
  #[display("CseAlifeItemDetector")]
  CseAlifeItemDetector,
  #[display("CseAlifeItemDocument")]
  CseAlifeItemDocument,
  #[display("CseAlifeItemExplosive")]
  CseAlifeItemExplosive,
  #[display("CseAlifeItemGrenade")]
  CseAlifeItemGrenade,
  #[display("CseAlifeItemHelmet")]
  CseAlifeItemHelmet,
  #[display("CseAlifeItemPda")]
  CseAlifeItemPda,
  #[display("CseAlifeItemTorch")]
  CseAlifeItemTorch,
  #[display("CseAlifeItemWeapon")]
  CseAlifeItemWeapon,
  #[display("CseAlifeItemWeaponMagazined")]
  CseAlifeItemWeaponMagazined,
  #[display("CseAlifeItemWeaponMagazinedWGl")]
  CseAlifeItemWeaponMagazinedWGl,
  #[display("CseAlifeItemWeaponShotgun")]
  CseAlifeItemWeaponShotgun,
  #[display("CseAlifeObjectBreakable")]
  CseAlifeObjectBreakable,
  #[display("CseAlifeObjectClimable")]
  CseAlifeObjectClimable,
  #[display("CseAlifeObjectHangingLamp")]
  CseAlifeObjectHangingLamp,
  #[display("CseAlifeObjectPhysic")]
  CseAlifeObjectPhysic,
  #[display("CseAlifeObjectProjector")]
  CseAlifeObjectProjector,
  #[display("CseAlifePhSkeletonObject")]
  CseAlifePhSkeletonObject,
  #[display("CseAlifeSpaceRestrictor")]
  CseAlifeSpaceRestrictor,
  #[display("CseAlifeStationaryMGun")]
  CseAlifeStationaryMGun,
  #[display("CseAlifeTeamBaseZone")]
  CseAlifeTeamBaseZone,
  #[display("CseAlifeTorridZone")]
  CseAlifeTorridZone,
  #[display("CseAlifeTrader")]
  CseAlifeTrader,
  #[display("CseAlifeZoneVisual")]
  CseAlifeZoneVisual,
  #[display("SeActor")]
  SeActor,
  #[display("SeLevelChanger")]
  SeLevelChanger,
  #[display("SeMonster")]
  SeMonster,
  #[display("SeSmartCover")]
  SeSmartCover,
  #[display("SeSmartTerrain")]
  SeSmartTerrain,
  #[display("SeStalker")]
  SeStalker,
  #[display("SimSquadScripted")]
  SimSquadScripted,
  /// Clear Sky's `se_actor`.
  #[display("ClearSkySeActor")]
  ClearSkySeActor,
  /// Clear Sky's `se_monster`.
  #[display("ClearSkySeMonster")]
  ClearSkySeMonster,
  /// Clear Sky's `se_sim_faction`, which Call of Pripyat has no class for.
  #[display("ClearSkySeSimFaction")]
  ClearSkySeSimFaction,
  /// Clear Sky's `se_smart_terrain`.
  #[display("ClearSkySeSmartTerrain")]
  ClearSkySeSmartTerrain,
  /// Clear Sky's `se_stalker`.
  #[display("ClearSkySeStalker")]
  ClearSkySeStalker,
  /// A class no spawn can hold, such as a rocket's `CSE_Temporary`, which is no alife object.
  #[display("Unknown")]
  Unknown,
}

impl AlifeClass {
  /// The class Call of Pripyat reads an object of `cls_id` as.
  pub fn from_cls_id(cls_id: &ClsId) -> Self {
    // todo: Implement with From<T> trait?
    CLS_ID_TO_CLASS.index(cls_id.clone()).clone()
  }

  /// The class the game whose scripts the object follows reads an object of `cls_id` as: Clear Sky's own script class
  /// where its `class_registrator.script` registers one that saves other fields than Call of Pripyat's.
  pub fn from_dialect_cls_id(cls_id: &ClsId, dialect: ScriptDialect) -> Self {
    let class: Self = Self::from_cls_id(cls_id);

    match dialect {
      ScriptDialect::CallOfPripyat => class,
      ScriptDialect::ClearSky => match (cls_id, class) {
        (ClsId::SFaction, _) => Self::ClearSkySeSimFaction,
        (_, Self::SeActor) => Self::ClearSkySeActor,
        (_, Self::SeMonster) => Self::ClearSkySeMonster,
        (_, Self::SeSmartTerrain) => Self::ClearSkySeSmartTerrain,
        (_, Self::SeStalker) => Self::ClearSkySeStalker,
        (_, class) => class,
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::data::alife::script_dialect::ScriptDialect;
  use crate::data::meta::alife_class::AlifeClass;
  use crate::data::meta::cls_id::ClsId;

  /// The script classes the two games disagree on resolve by dialect; the engine classes they share do not.
  #[test]
  fn resolves_the_script_classes_the_games_disagree_on() {
    for (cls_id, call_of_pripyat, clear_sky) in [
      (ClsId::SActor, AlifeClass::SeActor, AlifeClass::ClearSkySeActor),
      (ClsId::AiStlS, AlifeClass::SeStalker, AlifeClass::ClearSkySeStalker),
      (ClsId::SmDogS, AlifeClass::SeMonster, AlifeClass::ClearSkySeMonster),
      (ClsId::SmPoltr, AlifeClass::SeMonster, AlifeClass::ClearSkySeMonster),
      (
        ClsId::SmrtTrrn,
        AlifeClass::SeSmartTerrain,
        AlifeClass::ClearSkySeSmartTerrain,
      ),
      (ClsId::SFaction, AlifeClass::Unknown, AlifeClass::ClearSkySeSimFaction),
      (
        ClsId::ScriptZn,
        AlifeClass::CseAlifeSpaceRestrictor,
        AlifeClass::CseAlifeSpaceRestrictor,
      ),
      (ClsId::AiTrdS, AlifeClass::CseAlifeTrader, AlifeClass::CseAlifeTrader),
      (ClsId::SmrtCS, AlifeClass::SeSmartCover, AlifeClass::SeSmartCover),
    ] {
      assert_eq!(
        AlifeClass::from_dialect_cls_id(&cls_id, ScriptDialect::CallOfPripyat),
        call_of_pripyat,
        "{cls_id:?}"
      );
      assert_eq!(
        AlifeClass::from_dialect_cls_id(&cls_id, ScriptDialect::ClearSky),
        clear_sky,
        "{cls_id:?}"
      );
    }
  }
}
