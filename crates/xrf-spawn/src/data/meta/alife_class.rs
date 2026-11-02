use std::ops::Index;

use derive_more::{Display, FromStr};
use enum_map::Enum;

use crate::data::meta::cls_id::ClsId;
use crate::data::meta::map::CLS_ID_TO_CLASS;

/// The server class a spawned object is read as: those Call of Pripyat creates from a spawn, by the engine's
/// `object_factory_register.cpp` and the scripts' `class_registrator.script`.
#[derive(Clone, Debug, Enum, PartialEq, FromStr, Display)]
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
  /// A class no spawn can hold, such as a rocket's `CSE_Temporary`, which is no alife object.
  #[display("Unknown")]
  Unknown,
}

impl AlifeClass {
  pub fn from_cls_id(cls_id: &ClsId) -> AlifeClass {
    // todo: Implement with From<T> trait?
    CLS_ID_TO_CLASS.index(cls_id.clone()).clone()
  }
}
