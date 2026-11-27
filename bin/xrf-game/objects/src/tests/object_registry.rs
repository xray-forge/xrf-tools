use xrf_math::glam::Vec3;
use xrf_spawn::{ClsId, SpawnPlacedObjects};

use crate::id::object_id::ObjectId;
use crate::record::object_record::ObjectRecord;
use crate::registry::object_registry::ObjectRegistry;

fn record(level: u8) -> ObjectRecord {
  ObjectRecord {
    section: String::from("stalker"),
    name: String::from("stalker_0001"),
    class: ClsId::AiStlS,
    position: Vec3::ZERO,
    direction: Vec3::Z,
    level: Some(level),
    game_vertex: Some(1),
    level_vertex: Some(2),
    parent: None,
    story_id: None,
    spawn_id: 0,
  }
}

#[test]
fn builds_an_empty_registry_from_a_spawn_with_no_objects() {
  let registry: ObjectRegistry = ObjectRegistry::of_spawn(&SpawnPlacedObjects {
    levels: Vec::new(),
    objects: Vec::new(),
    total: 0,
    skipped: Vec::new(),
  });

  assert!(registry.get_records().is_empty());
  assert!(registry.get_levels().is_empty());
}

#[test]
fn counts_the_objects_standing_on_a_level() {
  let mut registry: ObjectRegistry = ObjectRegistry::default();

  registry.get_records_mut().insert(ObjectId(0), record(108));
  registry.get_records_mut().insert(ObjectId(5), record(108));
  registry.get_records_mut().insert(ObjectId(9), record(118));

  assert_eq!(registry.count_on_level(108), 2);
  assert_eq!(registry.count_on_level(118), 1);
  assert_eq!(registry.count_on_level(7), 0);
}
