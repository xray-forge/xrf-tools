use std::ops::Deref;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport};
use xrf_math::Vector3d;

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_actor::AlifeActor;
use crate::data::alife::inherited::alife_anomalous_zone::AlifeAnomalousZone;
use crate::data::alife::inherited::alife_graph_point::AlifeGraphPoint;
use crate::data::alife::inherited::alife_level_changer::AlifeLevelChanger;
use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
use crate::data::alife::inherited::alife_object_breakable::AlifeObjectBreakable;
use crate::data::alife::inherited::alife_object_car::AlifeObjectCar;
use crate::data::alife::inherited::alife_object_climable::AlifeObjectClimable;
use crate::data::alife::inherited::alife_object_hanging_lamp::AlifeObjectHangingLamp;
use crate::data::alife::inherited::alife_object_helicopter::AlifeObjectHelicopter;
use crate::data::alife::inherited::alife_object_inventory_box::AlifeObjectInventoryBox;
use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;
use crate::data::alife::inherited::alife_object_item_ammo::AlifeObjectItemAmmo;
use crate::data::alife::inherited::alife_object_item_artefact::AlifeObjectItemArtefact;
use crate::data::alife::inherited::alife_object_item_custom_outfit::AlifeObjectItemCustomOutfit;
use crate::data::alife::inherited::alife_object_item_detector::AlifeObjectItemDetector;
use crate::data::alife::inherited::alife_object_item_explosive::AlifeObjectItemExplosive;
use crate::data::alife::inherited::alife_object_item_grenade::AlifeObjectItemGrenade;
use crate::data::alife::inherited::alife_object_item_helmet::AlifeObjectItemHelmet;
use crate::data::alife::inherited::alife_object_item_pda::AlifeObjectItemPda;
use crate::data::alife::inherited::alife_object_item_weapon::AlifeObjectItemWeapon;
use crate::data::alife::inherited::alife_object_item_weapon_magazined::AlifeObjectItemWeaponMagazined;
use crate::data::alife::inherited::alife_object_item_weapon_magazined_wgl::AlifeObjectItemWeaponMagazinedWgl;
use crate::data::alife::inherited::alife_object_item_weapon_shotgun::AlifeObjectItemWeaponShotgun;
use crate::data::alife::inherited::alife_object_physic::AlifeObjectPhysic;
use crate::data::alife::inherited::alife_object_space_restrictor::AlifeObjectSpaceRestrictor;
use crate::data::alife::inherited::alife_object_torrid_zone::AlifeObjectTorridZone;
use crate::data::alife::inherited::alife_object_trader::AlifeObjectTrader;
use crate::data::alife::inherited::alife_smart_cover::AlifeSmartCover;
use crate::data::alife::inherited::alife_smart_terrain::AlifeSmartTerrain;
use crate::data::alife::inherited::alife_zone_visual::AlifeZoneVisual;
use crate::data::meta::alife_class::AlifeClass;
use crate::data::physics::physics_net_state::PhysicsNetState;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AlifeObjectInherited {
  SeActor(Box<AlifeActor>),
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
}

impl AlifeObjectInherited {
  /// Read the payload of `alife_class` from both packets: each class's `STATE_Read` from the spawn packet, script
  /// fields included, and its `UPDATE_Read` from the update packet, which a class without update fields leaves empty.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
    alife_class: &AlifeClass,
  ) -> XrfResult<Self> {
    Ok(match alife_class {
      AlifeClass::SeActor => Self::SeActor(Box::new(AlifeActor::read::<T, _>(spawn, update)?)),
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
        Self::CseAlifeObjectPhysic(Box::new(AlifeObjectPhysic::read::<T, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeHelicopter => Self::CseAlifeHelicopter(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::CseAlifeInventoryBox => Self::CseAlifeInventoryBox(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::CseAlifeObjectHangingLamp => Self::CseAlifeObjectHangingLamp(Box::new(spawn.read_xr::<T, _>()?)),
      AlifeClass::CseAlifeItem => Self::CseAlifeItem(Box::new(AlifeObjectItem::read::<T, _>(spawn, update)?)),
      AlifeClass::CseAlifeItemExplosive => {
        Self::CseAlifeItemExplosive(Box::new(AlifeObjectItemExplosive::read::<T, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemPda => Self::CseAlifeItemPda(Box::new(AlifeObjectItemPda::read::<T, _>(spawn, update)?)),
      AlifeClass::CseAlifeItemAmmo => {
        Self::CseAlifeItemAmmo(Box::new(AlifeObjectItemAmmo::read::<T, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemGrenade => {
        Self::CseAlifeItemGrenade(Box::new(AlifeObjectItemGrenade::read::<T, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemArtefact => {
        Self::CseAlifeItemArtefact(Box::new(AlifeObjectItemArtefact::read::<T, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemWeapon => {
        Self::CseAlifeItemWeapon(Box::new(AlifeObjectItemWeapon::read::<T, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemDetector => {
        Self::CseAlifeItemDetector(Box::new(AlifeObjectItemDetector::read::<T, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemHelmet => {
        Self::CseAlifeItemHelmet(Box::new(AlifeObjectItemHelmet::read::<T, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemCustomOutfit => {
        Self::CseAlifeItemCustomOutfit(Box::new(AlifeObjectItemCustomOutfit::read::<T, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemWeaponShotgun => {
        Self::CseAlifeItemWeaponShotgun(Box::new(AlifeObjectItemWeaponShotgun::read::<T, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemWeaponMagazined => {
        Self::CseAlifeItemWeaponMagazined(Box::new(AlifeObjectItemWeaponMagazined::read::<T, _>(spawn, update)?))
      }
      AlifeClass::CseAlifeItemWeaponMagazinedWGl => Self::CseAlifeItemWeaponMagazinedWGl(Box::new(
        AlifeObjectItemWeaponMagazinedWgl::read::<T, _>(spawn, update)?,
      )),
      _ => {
        return Err(XrfError::new_parsing_error(format!(
          "Not implemented parser for {}",
          alife_class
        )));
      }
    })
  }

  /// The server class whose `STATE_Read` this payload is shaped for.
  pub fn get_class(&self) -> AlifeClass {
    match self {
      AlifeObjectInherited::SeActor(_) => AlifeClass::SeActor,
      AlifeObjectInherited::CseAlifeObjectBreakable(_) => AlifeClass::CseAlifeObjectBreakable,
      AlifeObjectInherited::CseAlifeObjectClimable(_) => AlifeClass::CseAlifeObjectClimable,
      AlifeObjectInherited::CseAlifeGraphPoint(_) => AlifeClass::CseAlifeGraphPoint,
      AlifeObjectInherited::CseAlifeSpaceRestrictor(_) => AlifeClass::CseAlifeSpaceRestrictor,
      AlifeObjectInherited::SeSmartCover(_) => AlifeClass::SeSmartCover,
      AlifeObjectInherited::CseAlifeAnomalousZone(_) => AlifeClass::CseAlifeAnomalousZone,
      AlifeObjectInherited::CseAlifeTorridZone(_) => AlifeClass::CseAlifeTorridZone,
      AlifeObjectInherited::SeSmartTerrain(_) => AlifeClass::SeSmartTerrain,
      AlifeObjectInherited::SeLevelChanger(_) => AlifeClass::SeLevelChanger,
      AlifeObjectInherited::CseAlifeZoneVisual(_) => AlifeClass::CseAlifeZoneVisual,
      AlifeObjectInherited::CseAlifeCar(_) => AlifeClass::CseAlifeCar,
      AlifeObjectInherited::CseAlifeTrader(_) => AlifeClass::CseAlifeTrader,
      AlifeObjectInherited::CseAlifeObjectPhysic(_) => AlifeClass::CseAlifeObjectPhysic,
      AlifeObjectInherited::CseAlifeHelicopter(_) => AlifeClass::CseAlifeHelicopter,
      AlifeObjectInherited::CseAlifeInventoryBox(_) => AlifeClass::CseAlifeInventoryBox,
      AlifeObjectInherited::CseAlifeObjectHangingLamp(_) => AlifeClass::CseAlifeObjectHangingLamp,
      AlifeObjectInherited::CseAlifeItem(_) => AlifeClass::CseAlifeItem,
      AlifeObjectInherited::CseAlifeItemExplosive(_) => AlifeClass::CseAlifeItemExplosive,
      AlifeObjectInherited::CseAlifeItemPda(_) => AlifeClass::CseAlifeItemPda,
      AlifeObjectInherited::CseAlifeItemAmmo(_) => AlifeClass::CseAlifeItemAmmo,
      AlifeObjectInherited::CseAlifeItemGrenade(_) => AlifeClass::CseAlifeItemGrenade,
      AlifeObjectInherited::CseAlifeItemArtefact(_) => AlifeClass::CseAlifeItemArtefact,
      AlifeObjectInherited::CseAlifeItemWeapon(_) => AlifeClass::CseAlifeItemWeapon,
      AlifeObjectInherited::CseAlifeItemDetector(_) => AlifeClass::CseAlifeItemDetector,
      AlifeObjectInherited::CseAlifeItemHelmet(_) => AlifeClass::CseAlifeItemHelmet,
      AlifeObjectInherited::CseAlifeItemCustomOutfit(_) => AlifeClass::CseAlifeItemCustomOutfit,
      AlifeObjectInherited::CseAlifeItemWeaponShotgun(_) => AlifeClass::CseAlifeItemWeaponShotgun,
      AlifeObjectInherited::CseAlifeItemWeaponMagazined(_) => AlifeClass::CseAlifeItemWeaponMagazined,
      AlifeObjectInherited::CseAlifeItemWeaponMagazinedWGl(_) => AlifeClass::CseAlifeItemWeaponMagazinedWGl,
    }
  }

  /// The abstract object every alife class inherits, which carries its game vertex, flags and custom data; `None`
  /// for a graph point, which inherits none.
  pub fn get_abstract(&self) -> Option<&AlifeObjectAbstract> {
    match self {
      AlifeObjectInherited::SeActor(object) => Some(&object.base.base.base.base),
      AlifeObjectInherited::CseAlifeObjectBreakable(object) => Some(&object.base.base),
      AlifeObjectInherited::CseAlifeObjectClimable(object) => Some(&object.base.base),
      AlifeObjectInherited::CseAlifeGraphPoint(_) => None,
      AlifeObjectInherited::CseAlifeSpaceRestrictor(object) => Some(&object.base),
      AlifeObjectInherited::SeSmartCover(object) => Some(&object.base.base.base),
      AlifeObjectInherited::CseAlifeAnomalousZone(object) => Some(&object.base.base.base.base),
      AlifeObjectInherited::CseAlifeTorridZone(object) => Some(&object.base.base.base),
      AlifeObjectInherited::SeSmartTerrain(object) => Some(&object.base.base.base),
      AlifeObjectInherited::SeLevelChanger(object) => Some(&object.base.base),
      AlifeObjectInherited::CseAlifeZoneVisual(object) => Some(&object.base.base.base.base),
      AlifeObjectInherited::CseAlifeCar(object) => Some(&object.base.base),
      AlifeObjectInherited::CseAlifeTrader(object) => Some(&object.base.base),
      AlifeObjectInherited::CseAlifeObjectPhysic(object) => Some(&object.base.base),
      AlifeObjectInherited::CseAlifeHelicopter(object) => Some(&object.base.base),
      AlifeObjectInherited::CseAlifeInventoryBox(object) => Some(&object.base.base),
      AlifeObjectInherited::CseAlifeObjectHangingLamp(object) => Some(&object.base.base),
      AlifeObjectInherited::CseAlifeItem(object) => Some(&object.base.base),
      AlifeObjectInherited::CseAlifeItemExplosive(object) => Some(&object.base.base.base),
      AlifeObjectInherited::CseAlifeItemPda(object) => Some(&object.base.base.base),
      AlifeObjectInherited::CseAlifeItemAmmo(object) => Some(&object.base.base.base),
      AlifeObjectInherited::CseAlifeItemGrenade(object) => Some(&object.base.base.base),
      AlifeObjectInherited::CseAlifeItemArtefact(object) => Some(&object.base.base.base),
      AlifeObjectInherited::CseAlifeItemWeapon(object) => Some(&object.base.base.base),
      AlifeObjectInherited::CseAlifeItemDetector(object) => Some(&object.base.base.base),
      AlifeObjectInherited::CseAlifeItemHelmet(object) => Some(&object.base.base.base),
      AlifeObjectInherited::CseAlifeItemCustomOutfit(object) => Some(&object.base.base.base),
      AlifeObjectInherited::CseAlifeItemWeaponShotgun(object) => Some(&object.base.base.base.base.base),
      AlifeObjectInherited::CseAlifeItemWeaponMagazined(object) => Some(&object.base.base.base.base),
      AlifeObjectInherited::CseAlifeItemWeaponMagazinedWGl(object) => Some(&object.base.base.base.base.base),
    }
  }

  /// The visual the object is drawn as, by the name its class keeps it under (`cse_visual::visual_name`); `None` for a
  /// class drawn as nothing, and for an object naming no visual.
  pub fn get_visual(&self) -> Option<&str> {
    let name: &str = match self {
      AlifeObjectInherited::SeActor(object) => object.base.base.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeObjectBreakable(object) => object.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeCar(object) => object.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeTrader(object) => object.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeObjectPhysic(object) => object.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeHelicopter(object) => object.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeInventoryBox(object) => object.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeObjectHangingLamp(object) => object.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeItem(object) => object.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeItemExplosive(object) => object.base.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeItemPda(object) => object.base.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeItemAmmo(object) => object.base.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeItemGrenade(object) => object.base.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeItemArtefact(object) => object.base.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeItemWeapon(object) => object.base.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeItemDetector(object) => object.base.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeItemHelmet(object) => object.base.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeItemCustomOutfit(object) => object.base.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeItemWeaponShotgun(object) => object.base.base.base.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeItemWeaponMagazined(object) => object.base.base.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeItemWeaponMagazinedWGl(object) => object.base.base.base.base.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeZoneVisual(object) => object.visual.visual_name.as_str(),
      AlifeObjectInherited::CseAlifeObjectClimable(_)
      | AlifeObjectInherited::CseAlifeGraphPoint(_)
      | AlifeObjectInherited::CseAlifeSpaceRestrictor(_)
      | AlifeObjectInherited::SeSmartCover(_)
      | AlifeObjectInherited::CseAlifeAnomalousZone(_)
      | AlifeObjectInherited::CseAlifeTorridZone(_)
      | AlifeObjectInherited::SeSmartTerrain(_)
      | AlifeObjectInherited::SeLevelChanger(_) => return None,
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
      AlifeObjectInherited::SeActor(object) => {
        object.base.move_to(position, direction);

        return;
      }
      AlifeObjectInherited::CseAlifeObjectPhysic(object) => object.update_state.as_mut(),
      AlifeObjectInherited::CseAlifeItem(object) => object.update_state.as_mut(),
      AlifeObjectInherited::CseAlifeItemExplosive(object) => object.base.update_state.as_mut(),
      AlifeObjectInherited::CseAlifeItemPda(object) => object.base.update_state.as_mut(),
      AlifeObjectInherited::CseAlifeItemAmmo(object) => object.base.update_state.as_mut(),
      AlifeObjectInherited::CseAlifeItemGrenade(object) => object.base.update_state.as_mut(),
      AlifeObjectInherited::CseAlifeItemArtefact(object) => object.base.update_state.as_mut(),
      AlifeObjectInherited::CseAlifeItemWeapon(object) => object.base.update_state.as_mut(),
      AlifeObjectInherited::CseAlifeItemDetector(object) => object.base.update_state.as_mut(),
      AlifeObjectInherited::CseAlifeItemHelmet(object) => object.base.update_state.as_mut(),
      AlifeObjectInherited::CseAlifeItemCustomOutfit(object) => object.base.update_state.as_mut(),
      AlifeObjectInherited::CseAlifeItemWeaponShotgun(object) => object.base.base.base.update_state.as_mut(),
      AlifeObjectInherited::CseAlifeItemWeaponMagazined(object) => object.base.base.update_state.as_mut(),
      AlifeObjectInherited::CseAlifeItemWeaponMagazinedWGl(object) => object.base.base.base.update_state.as_mut(),
      AlifeObjectInherited::CseAlifeObjectBreakable(_)
      | AlifeObjectInherited::CseAlifeObjectClimable(_)
      | AlifeObjectInherited::CseAlifeGraphPoint(_)
      | AlifeObjectInherited::CseAlifeSpaceRestrictor(_)
      | AlifeObjectInherited::SeSmartCover(_)
      | AlifeObjectInherited::CseAlifeAnomalousZone(_)
      | AlifeObjectInherited::CseAlifeTorridZone(_)
      | AlifeObjectInherited::SeSmartTerrain(_)
      | AlifeObjectInherited::SeLevelChanger(_)
      | AlifeObjectInherited::CseAlifeZoneVisual(_)
      | AlifeObjectInherited::CseAlifeCar(_)
      | AlifeObjectInherited::CseAlifeTrader(_)
      | AlifeObjectInherited::CseAlifeHelicopter(_)
      | AlifeObjectInherited::CseAlifeInventoryBox(_)
      | AlifeObjectInherited::CseAlifeObjectHangingLamp(_) => None,
    };

    if let Some(state) = state {
      state.set_position(position);
    }
  }

  /// Write the payload's spawn fields into `spawn` and its update fields into `update`.
  pub fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    match self {
      AlifeObjectInherited::SeActor(object) => object.write::<T>(spawn, update)?,
      AlifeObjectInherited::CseAlifeObjectBreakable(object) => spawn.write_xr::<T, _>(object.deref())?,
      AlifeObjectInherited::CseAlifeObjectClimable(object) => spawn.write_xr::<T, _>(object.deref())?,
      AlifeObjectInherited::CseAlifeGraphPoint(object) => spawn.write_xr::<T, _>(object.deref())?,
      AlifeObjectInherited::CseAlifeSpaceRestrictor(object) => spawn.write_xr::<T, _>(object.deref())?,
      AlifeObjectInherited::SeSmartCover(object) => spawn.write_xr::<T, _>(object.deref())?,
      AlifeObjectInherited::CseAlifeAnomalousZone(object) => spawn.write_xr::<T, _>(object.deref())?,
      AlifeObjectInherited::CseAlifeTorridZone(object) => spawn.write_xr::<T, _>(object.deref())?,
      AlifeObjectInherited::SeSmartTerrain(object) => spawn.write_xr::<T, _>(object.deref())?,
      AlifeObjectInherited::SeLevelChanger(object) => spawn.write_xr::<T, _>(object.deref())?,
      AlifeObjectInherited::CseAlifeZoneVisual(object) => spawn.write_xr::<T, _>(object.deref())?,
      AlifeObjectInherited::CseAlifeCar(object) => spawn.write_xr::<T, _>(object.deref())?,
      AlifeObjectInherited::CseAlifeTrader(object) => spawn.write_xr::<T, _>(object.deref())?,
      AlifeObjectInherited::CseAlifeObjectPhysic(object) => object.write::<T>(spawn, update)?,
      AlifeObjectInherited::CseAlifeHelicopter(object) => spawn.write_xr::<T, _>(object.deref())?,
      AlifeObjectInherited::CseAlifeInventoryBox(object) => spawn.write_xr::<T, _>(object.deref())?,
      AlifeObjectInherited::CseAlifeObjectHangingLamp(object) => spawn.write_xr::<T, _>(object.deref())?,
      AlifeObjectInherited::CseAlifeItem(object) => object.write::<T>(spawn, update)?,
      AlifeObjectInherited::CseAlifeItemExplosive(object) => object.write::<T>(spawn, update)?,
      AlifeObjectInherited::CseAlifeItemPda(object) => object.write::<T>(spawn, update)?,
      AlifeObjectInherited::CseAlifeItemAmmo(object) => object.write::<T>(spawn, update)?,
      AlifeObjectInherited::CseAlifeItemGrenade(object) => object.write::<T>(spawn, update)?,
      AlifeObjectInherited::CseAlifeItemArtefact(object) => object.write::<T>(spawn, update)?,
      AlifeObjectInherited::CseAlifeItemWeapon(object) => object.write::<T>(spawn, update)?,
      AlifeObjectInherited::CseAlifeItemDetector(object) => object.write::<T>(spawn, update)?,
      AlifeObjectInherited::CseAlifeItemHelmet(object) => object.write::<T>(spawn, update)?,
      AlifeObjectInherited::CseAlifeItemCustomOutfit(object) => object.write::<T>(spawn, update)?,
      AlifeObjectInherited::CseAlifeItemWeaponShotgun(object) => object.write::<T>(spawn, update)?,
      AlifeObjectInherited::CseAlifeItemWeaponMagazined(object) => object.write::<T>(spawn, update)?,
      AlifeObjectInherited::CseAlifeItemWeaponMagazinedWGl(object) => object.write::<T>(spawn, update)?,
    }

    Ok(())
  }

  pub fn import(section_name: &str, ltx: &Ltx, alife_class: &AlifeClass) -> XrfResult<Self> {
    Ok(match alife_class {
      AlifeClass::SeActor => Self::SeActor(Box::new(AlifeActor::import(section_name, ltx)?)),
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
      _ => {
        return Err(XrfError::new_parsing_error(format!(
          "Not implemented parser for {}",
          alife_class
        )));
      }
    })
  }

  pub fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    match self {
      AlifeObjectInherited::SeActor(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeObjectBreakable(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeObjectClimable(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeGraphPoint(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeSpaceRestrictor(object) => object.export(section_name, ltx),
      AlifeObjectInherited::SeSmartCover(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeAnomalousZone(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeTorridZone(object) => object.export(section_name, ltx),
      AlifeObjectInherited::SeSmartTerrain(object) => object.export(section_name, ltx),
      AlifeObjectInherited::SeLevelChanger(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeZoneVisual(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeCar(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeTrader(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeObjectPhysic(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeHelicopter(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeInventoryBox(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeObjectHangingLamp(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeItem(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeItemExplosive(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeItemPda(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeItemAmmo(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeItemGrenade(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeItemArtefact(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeItemWeapon(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeItemDetector(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeItemHelmet(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeItemCustomOutfit(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeItemWeaponShotgun(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeItemWeaponMagazined(object) => object.export(section_name, ltx),
      AlifeObjectInherited::CseAlifeItemWeaponMagazinedWGl(object) => object.export(section_name, ltx),
    }
  }
}
