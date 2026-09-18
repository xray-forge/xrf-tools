use std::ops::Index;

use derive_more::{Display, FromStr};
use enum_map::Enum;

use crate::data::meta::cls_id::ClsId;
use crate::data::meta::map::CLS_ID_TO_CLASS;

#[derive(Clone, Debug, Enum, PartialEq, FromStr, Display)]
pub enum AlifeClass {
  #[display("CseAlifeAnomalousZone")]
  CseAlifeAnomalousZone,
  #[display("CseAlifeCar")]
  CseAlifeCar,
  #[display("CseAlifeCreatureCrow")]
  CseAlifeCreatureCrow,
  #[display("CseAlifeDynamicObjectVisual")]
  CseAlifeDynamicObjectVisual,
  #[display("CseAlifeFleshGroup")]
  CseAlifeFleshGroup,
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
  #[display("CseAlifeMonsterBase")]
  CseAlifeMonsterBase,
  #[display("CseAlifeMountedWeapon")]
  CseAlifeMountedWeapon,
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
  #[display("CseAlifeRatGroup")]
  CseAlifeRatGroup,
  #[display("CseAlifeSpaceRestrictor")]
  CseAlifeSpaceRestrictor,
  #[display("CseAlifeSpawnGroup")]
  CseAlifeSpawnGroup,
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
  #[display("CseSpectator")]
  CseSpectator,
  #[display("SeActor")]
  SeActor,
  #[display("SeLevelChanger")]
  SeLevelChanger,
  #[display("SeMonster")]
  SeMonster,
  #[display("SeSimFaction")]
  SeSimFaction,
  #[display("SeSmartCover")]
  SeSmartCover,
  #[display("SeSmartTerrain")]
  SeSmartTerrain,
  #[display("SeStalker")]
  SeStalker,
  #[display("SimSquadScripted")]
  SimSquadScripted,
  #[display("Unknown")]
  Unknown,
}

impl AlifeClass {
  pub fn from_cls_id(cls_id: &ClsId) -> AlifeClass {
    // todo: Implement with From<T> trait?
    CLS_ID_TO_CLASS.index(cls_id.clone()).clone()
  }
}
