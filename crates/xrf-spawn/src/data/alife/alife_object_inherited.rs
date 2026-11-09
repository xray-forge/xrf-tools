use std::ops::Deref;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxImportExport, LtxRead, LtxWrite};
use xrf_math::Vector3d;

use crate::data::alife::alife_built_fields::{AlifeBuiltDestination, AlifeBuiltFields};
use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_actor::AlifeActor;
use crate::data::alife::inherited::alife_anomalous_zone::AlifeAnomalousZone;
use crate::data::alife::inherited::alife_graph_point::AlifeGraphPoint;
use crate::data::alife::inherited::alife_level_changer::AlifeLevelChanger;
use crate::data::alife::inherited::alife_monster::AlifeMonster;
use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
use crate::data::alife::inherited::alife_object_breakable::AlifeObjectBreakable;
use crate::data::alife::inherited::alife_object_car::AlifeObjectCar;
use crate::data::alife::inherited::alife_object_climable::AlifeObjectClimable;
use crate::data::alife::inherited::alife_object_creature::AlifeObjectCreature;
use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
use crate::data::alife::inherited::alife_object_hanging_lamp::AlifeObjectHangingLamp;
use crate::data::alife::inherited::alife_object_helicopter::AlifeObjectHelicopter;
use crate::data::alife::inherited::alife_object_inventory_box::AlifeObjectInventoryBox;
use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;
use crate::data::alife::inherited::alife_object_item_ammo::AlifeObjectItemAmmo;
use crate::data::alife::inherited::alife_object_item_artefact::AlifeObjectItemArtefact;
use crate::data::alife::inherited::alife_object_item_custom_outfit::AlifeObjectItemCustomOutfit;
use crate::data::alife::inherited::alife_object_item_detector::AlifeObjectItemDetector;
use crate::data::alife::inherited::alife_object_item_document::AlifeObjectItemDocument;
use crate::data::alife::inherited::alife_object_item_explosive::AlifeObjectItemExplosive;
use crate::data::alife::inherited::alife_object_item_grenade::AlifeObjectItemGrenade;
use crate::data::alife::inherited::alife_object_item_helmet::AlifeObjectItemHelmet;
use crate::data::alife::inherited::alife_object_item_pda::AlifeObjectItemPda;
use crate::data::alife::inherited::alife_object_item_torch::AlifeObjectItemTorch;
use crate::data::alife::inherited::alife_object_item_weapon::AlifeObjectItemWeapon;
use crate::data::alife::inherited::alife_object_item_weapon_magazined::AlifeObjectItemWeaponMagazined;
use crate::data::alife::inherited::alife_object_item_weapon_magazined_wgl::AlifeObjectItemWeaponMagazinedWgl;
use crate::data::alife::inherited::alife_object_item_weapon_shotgun::AlifeObjectItemWeaponShotgun;
use crate::data::alife::inherited::alife_object_ph_skeleton_object::AlifeObjectPhSkeletonObject;
use crate::data::alife::inherited::alife_object_physic::AlifeObjectPhysic;
use crate::data::alife::inherited::alife_object_space_restrictor::AlifeObjectSpaceRestrictor;
use crate::data::alife::inherited::alife_object_stationary_mgun::AlifeObjectStationaryMgun;
use crate::data::alife::inherited::alife_object_torrid_zone::AlifeObjectTorridZone;
use crate::data::alife::inherited::alife_object_trader::AlifeObjectTrader;
use crate::data::alife::inherited::alife_smart_cover::AlifeSmartCover;
use crate::data::alife::inherited::alife_smart_terrain::AlifeSmartTerrain;
use crate::data::alife::inherited::alife_squad::AlifeSquad;
use crate::data::alife::inherited::alife_stalker::AlifeStalker;
use crate::data::alife::inherited::alife_zone_visual::AlifeZoneVisual;
use crate::data::alife::spawn_packet_header::SpawnPacketHeader;
use crate::data::generic::shape::Shape;
use crate::data::meta::alife_class::AlifeClass;
use crate::data::physics::physics_net_state::PhysicsNetState;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AlifeObjectInherited {
  SeActor(Box<AlifeActor>),
  SeStalker(Box<AlifeStalker>),
  SeMonster(Box<AlifeMonster>),
  CseAlifeObjectBreakable(Box<AlifeObjectBreakable>),
  CseAlifeObjectClimable(Box<AlifeObjectClimable>),
  CseAlifeGraphPoint(Box<AlifeGraphPoint>),
  CseAlifeSpaceRestrictor(Box<AlifeObjectSpaceRestrictor>),
  SeSmartCover(Box<AlifeSmartCover>),
  CseAlifeAnomalousZone(Box<AlifeAnomalousZone>),
  CseAlifeTorridZone(Box<AlifeObjectTorridZone>),
  SeSmartTerrain(Box<AlifeSmartTerrain>),
  SeLevelChanger(Box<AlifeLevelChanger>),
  CseAlifeZoneVisual(Box<AlifeZoneVisual>),
  CseAlifeCar(Box<AlifeObjectCar>),
  CseAlifeTrader(Box<AlifeObjectTrader>),
  CseAlifeObjectPhysic(Box<AlifeObjectPhysic>),
  CseAlifeHelicopter(Box<AlifeObjectHelicopter>),
  CseAlifeInventoryBox(Box<AlifeObjectInventoryBox>),
  CseAlifeObjectHangingLamp(Box<AlifeObjectHangingLamp>),
  CseAlifeItem(Box<AlifeObjectItem>),
  CseAlifeItemExplosive(Box<AlifeObjectItemExplosive>),
  CseAlifeItemPda(Box<AlifeObjectItemPda>),
  CseAlifeItemAmmo(Box<AlifeObjectItemAmmo>),
  CseAlifeItemGrenade(Box<AlifeObjectItemGrenade>),
  CseAlifeItemArtefact(Box<AlifeObjectItemArtefact>),
  CseAlifeItemWeapon(Box<AlifeObjectItemWeapon>),
  CseAlifeItemDetector(Box<AlifeObjectItemDetector>),
  CseAlifeItemHelmet(Box<AlifeObjectItemHelmet>),
  CseAlifeItemCustomOutfit(Box<AlifeObjectItemCustomOutfit>),
  CseAlifeItemWeaponShotgun(Box<AlifeObjectItemWeaponShotgun>),
  CseAlifeItemWeaponMagazined(Box<AlifeObjectItemWeaponMagazined>),
  CseAlifeItemWeaponMagazinedWGl(Box<AlifeObjectItemWeaponMagazinedWgl>),
  CseAlifeItemBolt(Box<AlifeObjectItem>),
  CseAlifeItemTorch(Box<AlifeObjectItemTorch>),
  CseAlifeItemDocument(Box<AlifeObjectItemDocument>),
  CseAlifeCreatureCrow(Box<AlifeObjectCreature>),
  CseAlifeCreaturePhantom(Box<AlifeObjectCreature>),
  CseAlifeDynamicObjectVisual(Box<AlifeObjectDynamicVisual>),
  CseAlifeObjectProjector(Box<AlifeObjectDynamicVisual>),
  CseAlifePhSkeletonObject(Box<AlifeObjectPhSkeletonObject>),
  CseAlifeStationaryMGun(Box<AlifeObjectStationaryMgun>),
  SimSquadScripted(Box<AlifeSquad>),
}

impl AlifeObjectInherited {
  /// Read the payload of `alife_class` from both packets: each class's `STATE_Read` from the spawn packet, script
  /// fields included, and its `UPDATE_Read` from the update packet, which a class without update fields leaves empty.
  pub fn read<T: ByteOrder, S: ChunkFieldReader, U: ChunkFieldReader>(
    spawn: &mut S,
    update: &mut U,
    alife_class: &AlifeClass,
  ) -> XrfResult<Self> {
    Ok(match alife_class {
      AlifeClass::SeActor => Self::SeActor(Box::new(AlifeActor::read::<T, _, _>(spawn, update)?)),
      AlifeClass::SeStalker => Self::SeStalker(Box::new(AlifeStalker::read::<T, _, _>(spawn, update)?)),
      AlifeClass::SeMonster => Self::SeMonster(Box::new(AlifeMonster::read::<T, _, _>(spawn, update)?)),
      AlifeClass::CseAlifeObjectBreakable => Self::CseAlifeObjectBreakable(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::CseAlifeObjectClimable => Self::CseAlifeObjectClimable(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::CseAlifeGraphPoint => Self::CseAlifeGraphPoint(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::CseAlifeSpaceRestrictor => Self::CseAlifeSpaceRestrictor(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::SeSmartCover => Self::SeSmartCover(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::CseAlifeAnomalousZone => Self::CseAlifeAnomalousZone(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::CseAlifeTorridZone => Self::CseAlifeTorridZone(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::SeSmartTerrain => Self::SeSmartTerrain(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::SeLevelChanger => Self::SeLevelChanger(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::CseAlifeZoneVisual => Self::CseAlifeZoneVisual(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::CseAlifeCar => Self::CseAlifeCar(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::CseAlifeTrader => Self::CseAlifeTrader(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::CseAlifeObjectPhysic => {
        Self::CseAlifeObjectPhysic(Box::new(AlifeObjectPhysic::read::<T, _, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeHelicopter => Self::CseAlifeHelicopter(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::CseAlifeInventoryBox => Self::CseAlifeInventoryBox(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::CseAlifeObjectHangingLamp => Self::CseAlifeObjectHangingLamp(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::CseAlifeItem => Self::CseAlifeItem(Box::new(AlifeObjectItem::read::<T, _, _>(spawn, update)?)),
      AlifeClass::CseAlifeItemExplosive => {
        Self::CseAlifeItemExplosive(Box::new(AlifeObjectItemExplosive::read::<T, _, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemPda => {
        Self::CseAlifeItemPda(Box::new(AlifeObjectItemPda::read::<T, _, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemAmmo => {
        Self::CseAlifeItemAmmo(Box::new(AlifeObjectItemAmmo::read::<T, _, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemGrenade => {
        Self::CseAlifeItemGrenade(Box::new(AlifeObjectItemGrenade::read::<T, _, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemArtefact => {
        Self::CseAlifeItemArtefact(Box::new(AlifeObjectItemArtefact::read::<T, _, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemWeapon => {
        Self::CseAlifeItemWeapon(Box::new(AlifeObjectItemWeapon::read::<T, _, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemDetector => {
        Self::CseAlifeItemDetector(Box::new(AlifeObjectItemDetector::read::<T, _, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemHelmet => {
        Self::CseAlifeItemHelmet(Box::new(AlifeObjectItemHelmet::read::<T, _, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemCustomOutfit => {
        Self::CseAlifeItemCustomOutfit(Box::new(AlifeObjectItemCustomOutfit::read::<T, _, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemWeaponShotgun => {
        Self::CseAlifeItemWeaponShotgun(Box::new(AlifeObjectItemWeaponShotgun::read::<T, _, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemWeaponMagazined => Self::CseAlifeItemWeaponMagazined(Box::new(
        AlifeObjectItemWeaponMagazined::read::<T, _, _>(spawn, update)?,
      )),
      AlifeClass::CseAlifeItemWeaponMagazinedWGl => Self::CseAlifeItemWeaponMagazinedWGl(Box::new(
        AlifeObjectItemWeaponMagazinedWgl::read::<T, _, _>(spawn, update)?,
      )),
      AlifeClass::CseAlifeItemBolt => {
        Self::CseAlifeItemBolt(Box::new(AlifeObjectItem::read::<T, _, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemTorch => {
        Self::CseAlifeItemTorch(Box::new(AlifeObjectItemTorch::read::<T, _, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemDocument => {
        Self::CseAlifeItemDocument(Box::new(AlifeObjectItemDocument::read::<T, _, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeCreatureCrow => {
        Self::CseAlifeCreatureCrow(Box::new(AlifeObjectCreature::read::<T, _, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeCreaturePhantom => {
        Self::CseAlifeCreaturePhantom(Box::new(AlifeObjectCreature::read::<T, _, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeDynamicObjectVisual => Self::CseAlifeDynamicObjectVisual(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::CseAlifeObjectProjector => Self::CseAlifeObjectProjector(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::CseAlifePhSkeletonObject => Self::CseAlifePhSkeletonObject(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::CseAlifeStationaryMGun => {
        Self::CseAlifeStationaryMGun(Box::new(AlifeObjectStationaryMgun::read::<T, _, _>(spawn, update)?))
      }
      AlifeClass::SimSquadScripted => Self::SimSquadScripted(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::Unknown => return Err(Self::new_unknown_class_error()),
    })
  }

  /// The error for a class no spawn can hold, which has no payload to read or import.
  fn new_unknown_class_error() -> XrfError {
    XrfError::new_parsing_error("ALife object of a class no spawn can hold")
  }

  /// The server class whose `STATE_Read` this payload is shaped for.
  pub fn get_class(&self) -> AlifeClass {
    match self {
      Self::SeActor(_) => AlifeClass::SeActor,
      Self::SeStalker(_) => AlifeClass::SeStalker,
      Self::SeMonster(_) => AlifeClass::SeMonster,
      Self::CseAlifeObjectBreakable(_) => AlifeClass::CseAlifeObjectBreakable,
      Self::CseAlifeObjectClimable(_) => AlifeClass::CseAlifeObjectClimable,
      Self::CseAlifeGraphPoint(_) => AlifeClass::CseAlifeGraphPoint,
      Self::CseAlifeSpaceRestrictor(_) => AlifeClass::CseAlifeSpaceRestrictor,
      Self::SeSmartCover(_) => AlifeClass::SeSmartCover,
      Self::CseAlifeAnomalousZone(_) => AlifeClass::CseAlifeAnomalousZone,
      Self::CseAlifeTorridZone(_) => AlifeClass::CseAlifeTorridZone,
      Self::SeSmartTerrain(_) => AlifeClass::SeSmartTerrain,
      Self::SeLevelChanger(_) => AlifeClass::SeLevelChanger,
      Self::CseAlifeZoneVisual(_) => AlifeClass::CseAlifeZoneVisual,
      Self::CseAlifeCar(_) => AlifeClass::CseAlifeCar,
      Self::CseAlifeTrader(_) => AlifeClass::CseAlifeTrader,
      Self::CseAlifeObjectPhysic(_) => AlifeClass::CseAlifeObjectPhysic,
      Self::CseAlifeHelicopter(_) => AlifeClass::CseAlifeHelicopter,
      Self::CseAlifeInventoryBox(_) => AlifeClass::CseAlifeInventoryBox,
      Self::CseAlifeObjectHangingLamp(_) => AlifeClass::CseAlifeObjectHangingLamp,
      Self::CseAlifeItem(_) => AlifeClass::CseAlifeItem,
      Self::CseAlifeItemExplosive(_) => AlifeClass::CseAlifeItemExplosive,
      Self::CseAlifeItemPda(_) => AlifeClass::CseAlifeItemPda,
      Self::CseAlifeItemAmmo(_) => AlifeClass::CseAlifeItemAmmo,
      Self::CseAlifeItemGrenade(_) => AlifeClass::CseAlifeItemGrenade,
      Self::CseAlifeItemArtefact(_) => AlifeClass::CseAlifeItemArtefact,
      Self::CseAlifeItemWeapon(_) => AlifeClass::CseAlifeItemWeapon,
      Self::CseAlifeItemDetector(_) => AlifeClass::CseAlifeItemDetector,
      Self::CseAlifeItemHelmet(_) => AlifeClass::CseAlifeItemHelmet,
      Self::CseAlifeItemCustomOutfit(_) => AlifeClass::CseAlifeItemCustomOutfit,
      Self::CseAlifeItemWeaponShotgun(_) => AlifeClass::CseAlifeItemWeaponShotgun,
      Self::CseAlifeItemWeaponMagazined(_) => AlifeClass::CseAlifeItemWeaponMagazined,
      Self::CseAlifeItemWeaponMagazinedWGl(_) => AlifeClass::CseAlifeItemWeaponMagazinedWGl,
      Self::CseAlifeItemBolt(_) => AlifeClass::CseAlifeItemBolt,
      Self::CseAlifeItemTorch(_) => AlifeClass::CseAlifeItemTorch,
      Self::CseAlifeItemDocument(_) => AlifeClass::CseAlifeItemDocument,
      Self::CseAlifeCreatureCrow(_) => AlifeClass::CseAlifeCreatureCrow,
      Self::CseAlifeCreaturePhantom(_) => AlifeClass::CseAlifeCreaturePhantom,
      Self::CseAlifeDynamicObjectVisual(_) => AlifeClass::CseAlifeDynamicObjectVisual,
      Self::CseAlifeObjectProjector(_) => AlifeClass::CseAlifeObjectProjector,
      Self::CseAlifePhSkeletonObject(_) => AlifeClass::CseAlifePhSkeletonObject,
      Self::CseAlifeStationaryMGun(_) => AlifeClass::CseAlifeStationaryMGun,
      Self::SimSquadScripted(_) => AlifeClass::SimSquadScripted,
    }
  }

  /// The abstract object every alife class inherits, which carries its game vertex, flags and custom data; `None`
  /// for a graph point, which inherits none.
  pub fn get_abstract(&self) -> Option<&AlifeObjectAbstract> {
    match self {
      Self::SeActor(object) => Some(&object.base.base.base.base),
      Self::SeStalker(object) => Some(&object.base.base.base.base.base.base),
      Self::SeMonster(object) => Some(&object.base.base.base.base.base),
      Self::CseAlifeObjectBreakable(object) => Some(&object.base.base),
      Self::CseAlifeObjectClimable(object) => Some(&object.base.base),
      Self::CseAlifeGraphPoint(_) => None,
      Self::CseAlifeSpaceRestrictor(object) => Some(&object.base),
      Self::SeSmartCover(object) => Some(&object.base.base.base),
      Self::CseAlifeAnomalousZone(object) => Some(&object.base.base.base.base),
      Self::CseAlifeTorridZone(object) => Some(&object.base.base.base),
      Self::SeSmartTerrain(object) => Some(&object.base.base.base),
      Self::SeLevelChanger(object) => Some(&object.base.base),
      Self::CseAlifeZoneVisual(object) => Some(&object.base.base.base.base),
      Self::CseAlifeCar(object) => Some(&object.base.base),
      Self::CseAlifeTrader(object) => Some(&object.base.base),
      Self::CseAlifeObjectPhysic(object) => Some(&object.base.base),
      Self::CseAlifeHelicopter(object) => Some(&object.base.base),
      Self::CseAlifeInventoryBox(object) => Some(&object.base.base),
      Self::CseAlifeObjectHangingLamp(object) => Some(&object.base.base),
      Self::CseAlifeItem(object) => Some(&object.base.base),
      Self::CseAlifeItemExplosive(object) => Some(&object.base.base.base),
      Self::CseAlifeItemPda(object) => Some(&object.base.base.base),
      Self::CseAlifeItemAmmo(object) => Some(&object.base.base.base),
      Self::CseAlifeItemGrenade(object) => Some(&object.base.base.base),
      Self::CseAlifeItemArtefact(object) => Some(&object.base.base.base),
      Self::CseAlifeItemWeapon(object) => Some(&object.base.base.base),
      Self::CseAlifeItemDetector(object) => Some(&object.base.base.base),
      Self::CseAlifeItemHelmet(object) => Some(&object.base.base.base),
      Self::CseAlifeItemCustomOutfit(object) => Some(&object.base.base.base),
      Self::CseAlifeItemWeaponShotgun(object) => Some(&object.base.base.base.base.base),
      Self::CseAlifeItemWeaponMagazined(object) => Some(&object.base.base.base.base),
      Self::CseAlifeItemWeaponMagazinedWGl(object) => Some(&object.base.base.base.base.base),
      Self::CseAlifeItemBolt(object) => Some(&object.base.base),
      Self::CseAlifeItemTorch(object) => Some(&object.base.base.base),
      Self::CseAlifeItemDocument(object) => Some(&object.base.base.base),
      Self::CseAlifeCreatureCrow(object) => Some(&object.base.base),
      Self::CseAlifeCreaturePhantom(object) => Some(&object.base.base),
      Self::CseAlifeDynamicObjectVisual(object) => Some(&object.base),
      Self::CseAlifeObjectProjector(object) => Some(&object.base),
      Self::CseAlifePhSkeletonObject(object) => Some(&object.base.base),
      Self::CseAlifeStationaryMGun(object) => Some(&object.base.base),
      Self::SimSquadScripted(object) => Some(&object.base.base.base),
    }
  }

  /// The abstract object [`Self::get_abstract`] reads, to change.
  pub fn get_abstract_mut(&mut self) -> Option<&mut AlifeObjectAbstract> {
    match self {
      Self::SeActor(object) => Some(&mut object.base.base.base.base),
      Self::SeStalker(object) => Some(&mut object.base.base.base.base.base.base),
      Self::SeMonster(object) => Some(&mut object.base.base.base.base.base),
      Self::CseAlifeObjectBreakable(object) => Some(&mut object.base.base),
      Self::CseAlifeObjectClimable(object) => Some(&mut object.base.base),
      Self::CseAlifeGraphPoint(_) => None,
      Self::CseAlifeSpaceRestrictor(object) => Some(&mut object.base),
      Self::SeSmartCover(object) => Some(&mut object.base.base.base),
      Self::CseAlifeAnomalousZone(object) => Some(&mut object.base.base.base.base),
      Self::CseAlifeTorridZone(object) => Some(&mut object.base.base.base),
      Self::SeSmartTerrain(object) => Some(&mut object.base.base.base),
      Self::SeLevelChanger(object) => Some(&mut object.base.base),
      Self::CseAlifeZoneVisual(object) => Some(&mut object.base.base.base.base),
      Self::CseAlifeCar(object) => Some(&mut object.base.base),
      Self::CseAlifeTrader(object) => Some(&mut object.base.base),
      Self::CseAlifeObjectPhysic(object) => Some(&mut object.base.base),
      Self::CseAlifeHelicopter(object) => Some(&mut object.base.base),
      Self::CseAlifeInventoryBox(object) => Some(&mut object.base.base),
      Self::CseAlifeObjectHangingLamp(object) => Some(&mut object.base.base),
      Self::CseAlifeItem(object) => Some(&mut object.base.base),
      Self::CseAlifeItemExplosive(object) => Some(&mut object.base.base.base),
      Self::CseAlifeItemPda(object) => Some(&mut object.base.base.base),
      Self::CseAlifeItemAmmo(object) => Some(&mut object.base.base.base),
      Self::CseAlifeItemGrenade(object) => Some(&mut object.base.base.base),
      Self::CseAlifeItemArtefact(object) => Some(&mut object.base.base.base),
      Self::CseAlifeItemWeapon(object) => Some(&mut object.base.base.base),
      Self::CseAlifeItemDetector(object) => Some(&mut object.base.base.base),
      Self::CseAlifeItemHelmet(object) => Some(&mut object.base.base.base),
      Self::CseAlifeItemCustomOutfit(object) => Some(&mut object.base.base.base),
      Self::CseAlifeItemWeaponShotgun(object) => Some(&mut object.base.base.base.base.base),
      Self::CseAlifeItemWeaponMagazined(object) => Some(&mut object.base.base.base.base),
      Self::CseAlifeItemWeaponMagazinedWGl(object) => Some(&mut object.base.base.base.base.base),
      Self::CseAlifeItemBolt(object) => Some(&mut object.base.base),
      Self::CseAlifeItemTorch(object) => Some(&mut object.base.base.base),
      Self::CseAlifeItemDocument(object) => Some(&mut object.base.base.base),
      Self::CseAlifeCreatureCrow(object) => Some(&mut object.base.base),
      Self::CseAlifeCreaturePhantom(object) => Some(&mut object.base.base),
      Self::CseAlifeDynamicObjectVisual(object) => Some(&mut object.base),
      Self::CseAlifeObjectProjector(object) => Some(&mut object.base),
      Self::CseAlifePhSkeletonObject(object) => Some(&mut object.base.base),
      Self::CseAlifeStationaryMGun(object) => Some(&mut object.base.base),
      Self::SimSquadScripted(object) => Some(&mut object.base.base.base),
    }
  }

  /// The payload's fields of those xrAI fills in when it builds a root spawn, `spawn_id` aside: the game and level
  /// vertices and the distance to the game vertex, a level changer's destination, an anomalous zone's artefact spawn
  /// offset.
  pub(crate) fn get_built_fields(&self, spawn_id: u16) -> AlifeBuiltFields {
    let base: Option<&AlifeObjectAbstract> = self.get_abstract();

    AlifeBuiltFields {
      spawn_id,
      game_vertex_id: base.map(|base| base.game_vertex_id),
      distance: base.map(|base| base.distance),
      level_vertex_id: base.map(|base| base.level_vertex_id),
      destination: match self {
        Self::SeLevelChanger(changer) => Some(AlifeBuiltDestination {
          game_vertex_id: changer.dest_game_vertex_id,
          level_vertex_id: changer.dest_level_vertex_id,
          position: changer.dest_position,
          direction: changer.dest_direction,
        }),
        _ => None,
      },
      artefact_position_offset: match self {
        Self::CseAlifeAnomalousZone(zone) => Some(zone.base.artefact_position_offset),
        Self::CseAlifeZoneVisual(zone) => Some(zone.base.artefact_position_offset),
        _ => None,
      },
    }
  }

  /// Sets the fields [`Self::get_built_fields`] reads, each that both `fields` and the class have.
  pub(crate) fn set_built_fields(&mut self, fields: &AlifeBuiltFields) {
    if let Some(base) = self.get_abstract_mut() {
      base.game_vertex_id = fields.game_vertex_id.unwrap_or(base.game_vertex_id);
      base.distance = fields.distance.unwrap_or(base.distance);
      base.level_vertex_id = fields.level_vertex_id.unwrap_or(base.level_vertex_id);
    }

    match (self, fields.destination, fields.artefact_position_offset) {
      (Self::SeLevelChanger(changer), Some(destination), _) => {
        changer.dest_game_vertex_id = destination.game_vertex_id;
        changer.dest_level_vertex_id = destination.level_vertex_id;
        changer.dest_position = destination.position;
        changer.dest_direction = destination.direction;
      }
      (Self::CseAlifeAnomalousZone(zone), _, Some(offset)) => zone.base.artefact_position_offset = offset,
      (Self::CseAlifeZoneVisual(zone), _, Some(offset)) => zone.base.artefact_position_offset = offset,
      _ => {}
    }
  }

  /// The volume the object encloses, for a class whose `shape()` returns it to the Level Editor: space restrictors
  /// with every class built on them (zones, smart terrains, level changers), climbable objects and smart covers. A
  /// breakable object keeps its shape in its visual and has none here.
  pub fn get_shapes(&self) -> Option<&[Shape]> {
    match self {
      Self::CseAlifeSpaceRestrictor(object) => Some(&object.shape),
      Self::SeLevelChanger(object) => Some(&object.base.shape),
      Self::SeSmartTerrain(object) => Some(&object.base.base.shape),
      Self::CseAlifeTorridZone(object) => Some(&object.base.base.shape),
      Self::CseAlifeAnomalousZone(object) => Some(&object.base.base.base.shape),
      Self::CseAlifeZoneVisual(object) => Some(&object.base.base.base.shape),
      Self::CseAlifeObjectClimable(object) => Some(&object.base.shape),
      Self::SeSmartCover(object) => Some(&object.base.shape),
      _ => None,
    }
  }

  /// The volume [`Self::get_shapes`] reads, to replace.
  pub fn get_shapes_mut(&mut self) -> Option<&mut Vec<Shape>> {
    match self {
      Self::CseAlifeSpaceRestrictor(object) => Some(&mut object.shape),
      Self::SeLevelChanger(object) => Some(&mut object.base.shape),
      Self::SeSmartTerrain(object) => Some(&mut object.base.base.shape),
      Self::CseAlifeTorridZone(object) => Some(&mut object.base.base.shape),
      Self::CseAlifeAnomalousZone(object) => Some(&mut object.base.base.base.shape),
      Self::CseAlifeZoneVisual(object) => Some(&mut object.base.base.base.shape),
      Self::CseAlifeObjectClimable(object) => Some(&mut object.base.shape),
      Self::SeSmartCover(object) => Some(&mut object.base.shape),
      _ => None,
    }
  }

  /// The visual the object is drawn as, by the name its class keeps it under (`cse_visual::visual_name`); `None` for a
  /// class drawn as nothing, and for an object naming no visual.
  pub fn get_visual(&self) -> Option<&str> {
    let name: &str = match self {
      Self::SeActor(object) => object.base.base.base.visual_name.as_str(),
      Self::SeStalker(object) => object.base.base.base.base.base.visual_name.as_str(),
      Self::SeMonster(object) => object.base.base.base.base.visual_name.as_str(),
      Self::CseAlifeObjectBreakable(object) => object.base.visual_name.as_str(),
      Self::CseAlifeCar(object) => object.base.visual_name.as_str(),
      Self::CseAlifeTrader(object) => object.base.visual_name.as_str(),
      Self::CseAlifeObjectPhysic(object) => object.base.visual_name.as_str(),
      Self::CseAlifeHelicopter(object) => object.base.visual_name.as_str(),
      Self::CseAlifeInventoryBox(object) => object.base.visual_name.as_str(),
      Self::CseAlifeObjectHangingLamp(object) => object.base.visual_name.as_str(),
      Self::CseAlifeItem(object) => object.base.visual_name.as_str(),
      Self::CseAlifeItemExplosive(object) => object.base.base.visual_name.as_str(),
      Self::CseAlifeItemPda(object) => object.base.base.visual_name.as_str(),
      Self::CseAlifeItemAmmo(object) => object.base.base.visual_name.as_str(),
      Self::CseAlifeItemGrenade(object) => object.base.base.visual_name.as_str(),
      Self::CseAlifeItemArtefact(object) => object.base.base.visual_name.as_str(),
      Self::CseAlifeItemWeapon(object) => object.base.base.visual_name.as_str(),
      Self::CseAlifeItemDetector(object) => object.base.base.visual_name.as_str(),
      Self::CseAlifeItemHelmet(object) => object.base.base.visual_name.as_str(),
      Self::CseAlifeItemCustomOutfit(object) => object.base.base.visual_name.as_str(),
      Self::CseAlifeItemWeaponShotgun(object) => object.base.base.base.base.visual_name.as_str(),
      Self::CseAlifeItemWeaponMagazined(object) => object.base.base.base.visual_name.as_str(),
      Self::CseAlifeItemWeaponMagazinedWGl(object) => object.base.base.base.base.visual_name.as_str(),
      Self::CseAlifeZoneVisual(object) => object.visual.visual_name.as_str(),
      Self::CseAlifeItemBolt(object) => object.base.visual_name.as_str(),
      Self::CseAlifeItemTorch(object) => object.base.base.visual_name.as_str(),
      Self::CseAlifeItemDocument(object) => object.base.base.visual_name.as_str(),
      Self::CseAlifeCreatureCrow(object) => object.base.visual_name.as_str(),
      Self::CseAlifeCreaturePhantom(object) => object.base.visual_name.as_str(),
      Self::CseAlifeDynamicObjectVisual(object) => object.visual_name.as_str(),
      Self::CseAlifeObjectProjector(object) => object.visual_name.as_str(),
      Self::CseAlifePhSkeletonObject(object) => object.base.visual_name.as_str(),
      Self::CseAlifeStationaryMGun(object) => object.base.visual_name.as_str(),
      Self::CseAlifeObjectClimable(_)
      | Self::CseAlifeGraphPoint(_)
      | Self::CseAlifeSpaceRestrictor(_)
      | Self::SeSmartCover(_)
      | Self::CseAlifeAnomalousZone(_)
      | Self::CseAlifeTorridZone(_)
      | Self::SeSmartTerrain(_)
      | Self::SeLevelChanger(_)
      | Self::SimSquadScripted(_) => return None,
    };

    (!name.is_empty()).then_some(name)
  }

  /// Get custom data of the object if it is supported by underlying alife class.
  /// Custom data is stored on the shared abstract object and reached through inheritance chains.
  pub fn get_custom_data(&self) -> Option<&String> {
    self.get_abstract().map(|it| &it.custom_data)
  }

  /// Move the object to `position`, facing `direction`, in the update packet: a creature's position and torso, an
  /// item's or a physic object's physics state, and the body an actor saved. A class whose update packet stores no
  /// position keeps none to move.
  pub fn move_to(&mut self, position: &Vector3d, direction: &Vector3d) {
    let state: Option<&mut PhysicsNetState> = match self {
      Self::SeActor(object) => {
        object.base.move_to(position, direction);

        return;
      }
      Self::SeStalker(object) => {
        object.move_to(position, direction);

        return;
      }
      Self::SeMonster(object) => {
        object.move_to(position, direction);

        return;
      }
      Self::CseAlifeCreatureCrow(object) | Self::CseAlifeCreaturePhantom(object) => {
        object.move_to(position, direction);

        return;
      }
      Self::CseAlifeObjectPhysic(object) => object.update_state.as_mut(),
      Self::CseAlifeItemBolt(object) => object.update_state.as_mut(),
      Self::CseAlifeItemTorch(object) => object.base.update_state.as_mut(),
      Self::CseAlifeItemDocument(object) => object.base.update_state.as_mut(),
      Self::CseAlifeItem(object) => object.update_state.as_mut(),
      Self::CseAlifeItemExplosive(object) => object.base.update_state.as_mut(),
      Self::CseAlifeItemPda(object) => object.base.update_state.as_mut(),
      Self::CseAlifeItemAmmo(object) => object.base.update_state.as_mut(),
      Self::CseAlifeItemGrenade(object) => object.base.update_state.as_mut(),
      Self::CseAlifeItemArtefact(object) => object.base.update_state.as_mut(),
      Self::CseAlifeItemWeapon(object) => object.base.update_state.as_mut(),
      Self::CseAlifeItemDetector(object) => object.base.update_state.as_mut(),
      Self::CseAlifeItemHelmet(object) => object.base.update_state.as_mut(),
      Self::CseAlifeItemCustomOutfit(object) => object.base.update_state.as_mut(),
      Self::CseAlifeItemWeaponShotgun(object) => object.base.base.base.update_state.as_mut(),
      Self::CseAlifeItemWeaponMagazined(object) => object.base.base.update_state.as_mut(),
      Self::CseAlifeItemWeaponMagazinedWGl(object) => object.base.base.base.update_state.as_mut(),
      Self::CseAlifeObjectBreakable(_)
      | Self::CseAlifeObjectClimable(_)
      | Self::CseAlifeGraphPoint(_)
      | Self::CseAlifeSpaceRestrictor(_)
      | Self::SeSmartCover(_)
      | Self::CseAlifeAnomalousZone(_)
      | Self::CseAlifeTorridZone(_)
      | Self::SeSmartTerrain(_)
      | Self::SeLevelChanger(_)
      | Self::CseAlifeZoneVisual(_)
      | Self::CseAlifeCar(_)
      | Self::CseAlifeTrader(_)
      | Self::CseAlifeHelicopter(_)
      | Self::CseAlifeInventoryBox(_)
      | Self::CseAlifeObjectHangingLamp(_)
      | Self::CseAlifeDynamicObjectVisual(_)
      | Self::CseAlifeObjectProjector(_)
      | Self::CseAlifePhSkeletonObject(_)
      | Self::CseAlifeStationaryMGun(_)
      | Self::SimSquadScripted(_) => None,
    };

    if let Some(state) = state {
      state.set_position(position);
    }
  }

  /// Set the update fields as the server entity holds them right after `Spawn_Read`, as each class's
  /// `AlifePacketsReadWrite::set_spawned_update` does; a class with an empty update packet has none.
  pub fn set_spawned_update(&mut self, header: &SpawnPacketHeader) {
    match self {
      Self::SeActor(object) => object.set_spawned_update(header),
      Self::SeStalker(object) => object.set_spawned_update(header),
      Self::SeMonster(object) => object.set_spawned_update(header),
      Self::CseAlifeObjectPhysic(object) => object.set_spawned_update(header),
      Self::CseAlifeItem(object) | Self::CseAlifeItemBolt(object) => object.set_spawned_update(header),
      Self::CseAlifeItemExplosive(object) => object.set_spawned_update(header),
      Self::CseAlifeItemPda(object) => object.set_spawned_update(header),
      Self::CseAlifeItemAmmo(object) => object.set_spawned_update(header),
      Self::CseAlifeItemGrenade(object) => object.set_spawned_update(header),
      Self::CseAlifeItemArtefact(object) => object.set_spawned_update(header),
      Self::CseAlifeItemWeapon(object) => object.set_spawned_update(header),
      Self::CseAlifeItemDetector(object) => object.set_spawned_update(header),
      Self::CseAlifeItemHelmet(object) => object.set_spawned_update(header),
      Self::CseAlifeItemCustomOutfit(object) => object.set_spawned_update(header),
      Self::CseAlifeItemWeaponShotgun(object) => object.set_spawned_update(header),
      Self::CseAlifeItemWeaponMagazined(object) => object.set_spawned_update(header),
      Self::CseAlifeItemWeaponMagazinedWGl(object) => object.set_spawned_update(header),
      Self::CseAlifeItemTorch(object) => object.set_spawned_update(header),
      Self::CseAlifeItemDocument(object) => object.set_spawned_update(header),
      Self::CseAlifeCreatureCrow(object) | Self::CseAlifeCreaturePhantom(object) => object.set_spawned_update(header),
      Self::CseAlifeStationaryMGun(object) => object.set_spawned_update(header),
      Self::CseAlifeObjectBreakable(_)
      | Self::CseAlifeObjectClimable(_)
      | Self::CseAlifeGraphPoint(_)
      | Self::CseAlifeSpaceRestrictor(_)
      | Self::SeSmartCover(_)
      | Self::CseAlifeAnomalousZone(_)
      | Self::CseAlifeTorridZone(_)
      | Self::SeSmartTerrain(_)
      | Self::SeLevelChanger(_)
      | Self::CseAlifeZoneVisual(_)
      | Self::CseAlifeCar(_)
      | Self::CseAlifeTrader(_)
      | Self::CseAlifeHelicopter(_)
      | Self::CseAlifeInventoryBox(_)
      | Self::CseAlifeObjectHangingLamp(_)
      | Self::CseAlifeDynamicObjectVisual(_)
      | Self::CseAlifeObjectProjector(_)
      | Self::CseAlifePhSkeletonObject(_)
      | Self::SimSquadScripted(_) => {}
    }
  }

  /// Write the payload's spawn fields into `spawn` and its update fields into `update`.
  pub fn write<T: ByteOrder, S: ChunkFieldWriter, U: ChunkFieldWriter>(
    &self,
    spawn: &mut S,
    update: &mut U,
  ) -> XrfResult {
    match self {
      Self::SeActor(object) => object.write::<T, _, _>(spawn, update)?,
      Self::SeStalker(object) => object.write::<T, _, _>(spawn, update)?,
      Self::SeMonster(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeObjectBreakable(object) => spawn.write_xr::<T, _>(object.deref())?,
      Self::CseAlifeObjectClimable(object) => spawn.write_xr::<T, _>(object.deref())?,
      Self::CseAlifeGraphPoint(object) => spawn.write_xr::<T, _>(object.deref())?,
      Self::CseAlifeSpaceRestrictor(object) => spawn.write_xr::<T, _>(object.deref())?,
      Self::SeSmartCover(object) => spawn.write_xr::<T, _>(object.deref())?,
      Self::CseAlifeAnomalousZone(object) => spawn.write_xr::<T, _>(object.deref())?,
      Self::CseAlifeTorridZone(object) => spawn.write_xr::<T, _>(object.deref())?,
      Self::SeSmartTerrain(object) => spawn.write_xr::<T, _>(object.deref())?,
      Self::SeLevelChanger(object) => spawn.write_xr::<T, _>(object.deref())?,
      Self::CseAlifeZoneVisual(object) => spawn.write_xr::<T, _>(object.deref())?,
      Self::CseAlifeCar(object) => spawn.write_xr::<T, _>(object.deref())?,
      Self::CseAlifeTrader(object) => spawn.write_xr::<T, _>(object.deref())?,
      Self::CseAlifeObjectPhysic(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeHelicopter(object) => spawn.write_xr::<T, _>(object.deref())?,
      Self::CseAlifeInventoryBox(object) => spawn.write_xr::<T, _>(object.deref())?,
      Self::CseAlifeObjectHangingLamp(object) => spawn.write_xr::<T, _>(object.deref())?,
      Self::CseAlifeItem(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeItemExplosive(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeItemPda(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeItemAmmo(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeItemGrenade(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeItemArtefact(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeItemWeapon(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeItemDetector(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeItemHelmet(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeItemCustomOutfit(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeItemWeaponShotgun(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeItemWeaponMagazined(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeItemWeaponMagazinedWGl(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeItemBolt(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeItemTorch(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeItemDocument(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeCreatureCrow(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeCreaturePhantom(object) => object.write::<T, _, _>(spawn, update)?,
      Self::CseAlifeDynamicObjectVisual(object) => spawn.write_xr::<T, _>(object.deref())?,
      Self::CseAlifeObjectProjector(object) => spawn.write_xr::<T, _>(object.deref())?,
      Self::CseAlifePhSkeletonObject(object) => spawn.write_xr::<T, _>(object.deref())?,
      Self::CseAlifeStationaryMGun(object) => object.write::<T, _, _>(spawn, update)?,
      Self::SimSquadScripted(object) => spawn.write_xr::<T, _>(object.deref())?,
    }

    Ok(())
  }

  pub fn import<L: LtxRead>(section_name: &str, ltx: &L, alife_class: &AlifeClass) -> XrfResult<Self> {
    Ok(match alife_class {
      AlifeClass::SeActor => Self::SeActor(Box::new(AlifeActor::import(section_name, ltx)?)),
      AlifeClass::SeStalker => Self::SeStalker(Box::new(AlifeStalker::import(section_name, ltx)?)),
      AlifeClass::SeMonster => Self::SeMonster(Box::new(AlifeMonster::import(section_name, ltx)?)),
      AlifeClass::CseAlifeObjectBreakable => {
        Self::CseAlifeObjectBreakable(Box::new(AlifeObjectBreakable::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeObjectClimable => {
        Self::CseAlifeObjectClimable(Box::new(AlifeObjectClimable::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeGraphPoint => Self::CseAlifeGraphPoint(Box::new(AlifeGraphPoint::import(section_name, ltx)?)),
      AlifeClass::CseAlifeSpaceRestrictor => {
        Self::CseAlifeSpaceRestrictor(Box::new(AlifeObjectSpaceRestrictor::import(section_name, ltx)?))
      }
      AlifeClass::SeSmartCover => Self::SeSmartCover(Box::new(AlifeSmartCover::import(section_name, ltx)?)),
      AlifeClass::CseAlifeAnomalousZone => {
        Self::CseAlifeAnomalousZone(Box::new(AlifeAnomalousZone::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeTorridZone => {
        Self::CseAlifeTorridZone(Box::new(AlifeObjectTorridZone::import(section_name, ltx)?))
      }
      AlifeClass::SeSmartTerrain => Self::SeSmartTerrain(Box::new(AlifeSmartTerrain::import(section_name, ltx)?)),
      AlifeClass::SeLevelChanger => Self::SeLevelChanger(Box::new(AlifeLevelChanger::import(section_name, ltx)?)),
      AlifeClass::CseAlifeZoneVisual => Self::CseAlifeZoneVisual(Box::new(AlifeZoneVisual::import(section_name, ltx)?)),
      AlifeClass::CseAlifeCar => Self::CseAlifeCar(Box::new(AlifeObjectCar::import(section_name, ltx)?)),
      AlifeClass::CseAlifeTrader => Self::CseAlifeTrader(Box::new(AlifeObjectTrader::import(section_name, ltx)?)),
      AlifeClass::CseAlifeObjectPhysic => {
        Self::CseAlifeObjectPhysic(Box::new(AlifeObjectPhysic::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeHelicopter => {
        Self::CseAlifeHelicopter(Box::new(AlifeObjectHelicopter::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeInventoryBox => {
        Self::CseAlifeInventoryBox(Box::new(AlifeObjectInventoryBox::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeObjectHangingLamp => {
        Self::CseAlifeObjectHangingLamp(Box::new(AlifeObjectHangingLamp::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeItem => Self::CseAlifeItem(Box::new(AlifeObjectItem::import(section_name, ltx)?)),
      AlifeClass::CseAlifeItemExplosive => {
        Self::CseAlifeItemExplosive(Box::new(AlifeObjectItemExplosive::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeItemPda => Self::CseAlifeItemPda(Box::new(AlifeObjectItemPda::import(section_name, ltx)?)),
      AlifeClass::CseAlifeItemAmmo => Self::CseAlifeItemAmmo(Box::new(AlifeObjectItemAmmo::import(section_name, ltx)?)),
      AlifeClass::CseAlifeItemGrenade => {
        Self::CseAlifeItemGrenade(Box::new(AlifeObjectItemGrenade::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeItemArtefact => {
        Self::CseAlifeItemArtefact(Box::new(AlifeObjectItemArtefact::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeItemWeapon => {
        Self::CseAlifeItemWeapon(Box::new(AlifeObjectItemWeapon::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeItemDetector => {
        Self::CseAlifeItemDetector(Box::new(AlifeObjectItemDetector::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeItemHelmet => {
        Self::CseAlifeItemHelmet(Box::new(AlifeObjectItemHelmet::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeItemCustomOutfit => {
        Self::CseAlifeItemCustomOutfit(Box::new(AlifeObjectItemCustomOutfit::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeItemWeaponShotgun => {
        Self::CseAlifeItemWeaponShotgun(Box::new(AlifeObjectItemWeaponShotgun::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeItemWeaponMagazined => {
        Self::CseAlifeItemWeaponMagazined(Box::new(AlifeObjectItemWeaponMagazined::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeItemWeaponMagazinedWGl => {
        Self::CseAlifeItemWeaponMagazinedWGl(Box::new(AlifeObjectItemWeaponMagazinedWgl::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeItemBolt => Self::CseAlifeItemBolt(Box::new(AlifeObjectItem::import(section_name, ltx)?)),
      AlifeClass::CseAlifeItemTorch => {
        Self::CseAlifeItemTorch(Box::new(AlifeObjectItemTorch::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeItemDocument => {
        Self::CseAlifeItemDocument(Box::new(AlifeObjectItemDocument::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeCreatureCrow => {
        Self::CseAlifeCreatureCrow(Box::new(AlifeObjectCreature::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeCreaturePhantom => {
        Self::CseAlifeCreaturePhantom(Box::new(AlifeObjectCreature::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeDynamicObjectVisual => {
        Self::CseAlifeDynamicObjectVisual(Box::new(AlifeObjectDynamicVisual::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeObjectProjector => {
        Self::CseAlifeObjectProjector(Box::new(AlifeObjectDynamicVisual::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifePhSkeletonObject => {
        Self::CseAlifePhSkeletonObject(Box::new(AlifeObjectPhSkeletonObject::import(section_name, ltx)?))
      }
      AlifeClass::CseAlifeStationaryMGun => {
        Self::CseAlifeStationaryMGun(Box::new(AlifeObjectStationaryMgun::import(section_name, ltx)?))
      }
      AlifeClass::SimSquadScripted => Self::SimSquadScripted(Box::new(AlifeSquad::import(section_name, ltx)?)),
      AlifeClass::Unknown => return Err(Self::new_unknown_class_error()),
    })
  }

  pub fn export(&self, section_name: &str, ltx: &mut impl LtxWrite) -> XrfResult {
    match self {
      Self::SeActor(object) => object.export(section_name, ltx),
      Self::SeStalker(object) => object.export(section_name, ltx),
      Self::SeMonster(object) => object.export(section_name, ltx),
      Self::CseAlifeObjectBreakable(object) => object.export(section_name, ltx),
      Self::CseAlifeObjectClimable(object) => object.export(section_name, ltx),
      Self::CseAlifeGraphPoint(object) => object.export(section_name, ltx),
      Self::CseAlifeSpaceRestrictor(object) => object.export(section_name, ltx),
      Self::SeSmartCover(object) => object.export(section_name, ltx),
      Self::CseAlifeAnomalousZone(object) => object.export(section_name, ltx),
      Self::CseAlifeTorridZone(object) => object.export(section_name, ltx),
      Self::SeSmartTerrain(object) => object.export(section_name, ltx),
      Self::SeLevelChanger(object) => object.export(section_name, ltx),
      Self::CseAlifeZoneVisual(object) => object.export(section_name, ltx),
      Self::CseAlifeCar(object) => object.export(section_name, ltx),
      Self::CseAlifeTrader(object) => object.export(section_name, ltx),
      Self::CseAlifeObjectPhysic(object) => object.export(section_name, ltx),
      Self::CseAlifeHelicopter(object) => object.export(section_name, ltx),
      Self::CseAlifeInventoryBox(object) => object.export(section_name, ltx),
      Self::CseAlifeObjectHangingLamp(object) => object.export(section_name, ltx),
      Self::CseAlifeItem(object) => object.export(section_name, ltx),
      Self::CseAlifeItemExplosive(object) => object.export(section_name, ltx),
      Self::CseAlifeItemPda(object) => object.export(section_name, ltx),
      Self::CseAlifeItemAmmo(object) => object.export(section_name, ltx),
      Self::CseAlifeItemGrenade(object) => object.export(section_name, ltx),
      Self::CseAlifeItemArtefact(object) => object.export(section_name, ltx),
      Self::CseAlifeItemWeapon(object) => object.export(section_name, ltx),
      Self::CseAlifeItemDetector(object) => object.export(section_name, ltx),
      Self::CseAlifeItemHelmet(object) => object.export(section_name, ltx),
      Self::CseAlifeItemCustomOutfit(object) => object.export(section_name, ltx),
      Self::CseAlifeItemWeaponShotgun(object) => object.export(section_name, ltx),
      Self::CseAlifeItemWeaponMagazined(object) => object.export(section_name, ltx),
      Self::CseAlifeItemWeaponMagazinedWGl(object) => object.export(section_name, ltx),
      Self::CseAlifeItemBolt(object) => object.export(section_name, ltx),
      Self::CseAlifeItemTorch(object) => object.export(section_name, ltx),
      Self::CseAlifeItemDocument(object) => object.export(section_name, ltx),
      Self::CseAlifeCreatureCrow(object) => object.export(section_name, ltx),
      Self::CseAlifeCreaturePhantom(object) => object.export(section_name, ltx),
      Self::CseAlifeDynamicObjectVisual(object) => object.export(section_name, ltx),
      Self::CseAlifeObjectProjector(object) => object.export(section_name, ltx),
      Self::CseAlifePhSkeletonObject(object) => object.export(section_name, ltx),
      Self::CseAlifeStationaryMGun(object) => object.export(section_name, ltx),
      Self::SimSquadScripted(object) => object.export(section_name, ltx),
    }
  }
}

#[cfg(test)]
mod tests {
  use byteorder::ReadBytesExt;
  use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;

  use crate::constants::NET_ACTION_UPDATE;
  use crate::data::alife::alife_object_inherited::AlifeObjectInherited;
  use crate::fixtures::GameWrittenPackets;

  /// What the game writes for a new object of each class reads whole as that class and writes back byte for byte.
  #[test]
  fn reads_and_writes_what_the_game_writes_for_each_class() -> XrfResult {
    for (section, class, spawn, update) in GameWrittenPackets::CLASSES {
      let spawn: Vec<u8> = GameWrittenPackets::bytes(spawn);
      let update: Vec<u8> = GameWrittenPackets::bytes(update);
      // An object's update packet opens with its action, so even an empty class's packet holds bytes to read past.
      let framed_update: Vec<u8> = [NET_ACTION_UPDATE.to_le_bytes().as_slice(), &update].concat();
      let mut spawn_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&spawn)?;
      let mut update_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&framed_update)?;

      update_reader.read_u16::<XRayByteOrder>()?;

      let object: AlifeObjectInherited =
        AlifeObjectInherited::read::<XRayByteOrder, _, _>(&mut spawn_reader, &mut update_reader, class)?;

      spawn_reader.assert_read(&format!("Expect the game's {section} spawn packet to be read whole"))?;
      update_reader.assert_read(&format!("Expect the game's {section} update packet to be read whole"))?;

      let mut spawn_writer: ChunkWriter = ChunkWriter::new();
      let mut update_writer: ChunkWriter = ChunkWriter::new();

      object.write::<XRayByteOrder, _, _>(&mut spawn_writer, &mut update_writer)?;

      assert_eq!(&object.get_class(), class, "{section}");
      assert_eq!(
        (spawn_writer.buffer, update_writer.buffer),
        (spawn, update),
        "{section}"
      );
    }

    Ok(())
  }
}
