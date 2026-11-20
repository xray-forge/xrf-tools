use xrf_enum_derive::EnumDisplay;

use crate::data::alife::script_dialect::ScriptDialect;
use crate::data::meta::cls_id::ClsId;
use crate::data::meta::map::class_of;

/// The server class a spawned object is read as: Call of Pripyat's spawnable classes (`object_factory_register.cpp`,
/// `class_registrator.script`), and Clear Sky's script classes that save other fields.
#[derive(Clone, Debug, PartialEq, Eq, EnumDisplay)]
pub enum AlifeClass {
  CseAlifeAnomalousZone,
  CseAlifeCar,
  CseAlifeCreatureCrow,
  CseAlifeCreaturePhantom,
  CseAlifeDynamicObjectVisual,
  CseAlifeGraphPoint,
  CseAlifeHelicopter,
  CseAlifeInventoryBox,
  CseAlifeItem,
  CseAlifeItemAmmo,
  CseAlifeItemArtefact,
  CseAlifeItemBolt,
  CseAlifeItemCustomOutfit,
  CseAlifeItemDetector,
  CseAlifeItemDocument,
  CseAlifeItemExplosive,
  CseAlifeItemGrenade,
  CseAlifeItemHelmet,
  CseAlifeItemPda,
  CseAlifeItemTorch,
  CseAlifeItemWeapon,
  CseAlifeItemWeaponMagazined,
  CseAlifeItemWeaponMagazinedWGl,
  CseAlifeItemWeaponShotgun,
  CseAlifeObjectBreakable,
  CseAlifeObjectClimable,
  CseAlifeObjectHangingLamp,
  CseAlifeObjectPhysic,
  CseAlifeObjectProjector,
  CseAlifePhSkeletonObject,
  CseAlifeSpaceRestrictor,
  CseAlifeStationaryMGun,
  CseAlifeTeamBaseZone,
  CseAlifeTorridZone,
  CseAlifeTrader,
  CseAlifeZoneVisual,
  SeActor,
  SeLevelChanger,
  SeMonster,
  SeSmartCover,
  SeSmartTerrain,
  SeStalker,
  SimSquadScripted,
  /// Clear Sky's `se_actor`.
  ClearSkySeActor,
  /// Clear Sky's `se_monster`.
  ClearSkySeMonster,
  /// Clear Sky's `se_sim_faction`, which Call of Pripyat has no class for.
  ClearSkySeSimFaction,
  /// Clear Sky's `se_smart_terrain`.
  ClearSkySeSmartTerrain,
  /// Clear Sky's `se_stalker`.
  ClearSkySeStalker,
  /// A class no spawn can hold, such as a rocket's `CSE_Temporary`, which is no alife object.
  Unknown,
}

impl AlifeClass {
  /// The class Call of Pripyat reads an object of `cls_id` as.
  pub fn from_cls_id(cls_id: &ClsId) -> Self {
    class_of(cls_id)
  }

  /// The class an object of `cls_id` is read as under `dialect`: Clear Sky's own script class where it saves other
  /// fields than Call of Pripyat's.
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

  // The script classes the two games disagree on resolve by dialect; the engine classes they share do not.
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
