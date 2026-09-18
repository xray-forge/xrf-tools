//! Spawn sets and the alife records inside them, with the graph, patrols and artefact points they reference.

pub(crate) mod chunks;
pub(crate) mod constants;
pub(crate) mod data;
pub(crate) mod level_spawn_file;
pub(crate) mod level_spawn_object;
pub(crate) mod spawn_file;
pub(crate) mod types;

#[cfg(test)]
mod tests;

pub use xrf_chunk::XRayByteOrder;

pub use crate::chunks::{
  spawn_alife_spawns_chunk::SpawnALifeSpawnsChunk, spawn_artefact_spawns_chunk::SpawnArtefactSpawnsChunk,
  spawn_graphs_chunk::SpawnGraphsChunk, spawn_header_chunk::SpawnHeaderChunk, spawn_patrols_chunk::SpawnPatrolsChunk,
};
pub use crate::data::{
  alife::{
    alife_object::AlifeObject,
    alife_object_inherited::AlifeObjectInherited,
    inherited::{
      alife_actor::AlifeActor, alife_anomalous_zone::AlifeAnomalousZone, alife_graph_point::AlifeGraphPoint,
      alife_level_changer::AlifeLevelChanger, alife_object_abstract::AlifeObjectAbstract,
      alife_object_actor::AlifeObjectActor, alife_object_anomaly_zone::AlifeObjectAnomalyZone,
      alife_object_breakable::AlifeObjectBreakable, alife_object_climable::AlifeObjectClimable,
      alife_object_creature::AlifeObjectCreature, alife_object_custom_zone::AlifeObjectCustomZone,
      alife_object_dynamic::AlifeObjectDynamic, alife_object_dynamic_visual::AlifeObjectDynamicVisual,
      alife_object_hanging_lamp::AlifeObjectHangingLamp, alife_object_helicopter::AlifeObjectHelicopter,
      alife_object_inventory_box::AlifeObjectInventoryBox, alife_object_item::AlifeObjectItem,
      alife_object_item_ammo::AlifeObjectItemAmmo, alife_object_item_artefact::AlifeObjectItemArtefact,
      alife_object_item_custom_outfit::AlifeObjectItemCustomOutfit,
      alife_object_item_detector::AlifeObjectItemDetector, alife_object_item_explosive::AlifeObjectItemExplosive,
      alife_object_item_grenade::AlifeObjectItemGrenade, alife_object_item_helmet::AlifeObjectItemHelmet,
      alife_object_item_pda::AlifeObjectItemPda, alife_object_item_weapon::AlifeObjectItemWeapon,
      alife_object_item_weapon_magazined::AlifeObjectItemWeaponMagazined,
      alife_object_item_weapon_magazined_wgl::AlifeObjectItemWeaponMagazinedWgl,
      alife_object_item_weapon_shotgun::AlifeObjectItemWeaponShotgun, alife_object_motion::AlifeObjectMotion,
      alife_object_physic::AlifeObjectPhysic, alife_object_shape::AlifeObjectShape,
      alife_object_skeleton::AlifeObjectSkeleton, alife_object_smart_cover::AlifeObjectSmartCover,
      alife_object_space_restrictor::AlifeObjectSpaceRestrictor, alife_object_torrid_zone::AlifeObjectTorridZone,
      alife_object_trader_abstract::AlifeObjectTraderAbstract, alife_object_visual::AlifeObjectVisual,
      alife_smart_cover::AlifeSmartCover, alife_smart_cover_loophole::AlifeSmartCoverLoophole,
      alife_smart_terrain::AlifeSmartTerrain, alife_smart_zone::AlifeSmartZone, alife_zone_visual::AlifeZoneVisual,
    },
  },
  artefact_spawn::artefact_spawn_point::ArtefactSpawnPoint,
  generic::{shape::Shape, time::Time, u32_bytes::U32Bytes},
  graph::{
    graph_cross_table::GraphCrossTable, graph_edge::GraphEdge, graph_header::GraphHeader, graph_level::GraphLevel,
    graph_level_point::GraphLevelPoint, graph_vertex::GraphVertex,
  },
  meta::cls_id::ClsId,
  patrols::{patrol::Patrol, patrol_link::PatrolLink, patrol_point::PatrolPoint},
};
pub use crate::level_spawn_file::*;
pub use crate::level_spawn_object::*;
pub use crate::spawn_file::*;
pub use crate::types::*;
