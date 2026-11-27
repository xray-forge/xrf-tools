use xrf_game_objects::{ObjectId, ObjectRecord, ObjectRegistry, ObjectSchedule, ObjectScheduleKind, ObjectSwitching};
use xrf_math::glam::Vec3;
use xrf_spawn::ClsId;

/// The level the tests stand on, and another.
pub(crate) const LEVEL: u8 = 108;
pub(crate) const OTHER_LEVEL: u8 = 118;

/// The id of the first object of slot `index`.
pub(crate) fn id(index: u32) -> ObjectId {
  ObjectId::of_index(index)
}

/// An object at `x` metres along `+x` from the origin, where the actor stands.
pub(crate) fn object(x: f32, switching: ObjectSwitching) -> ObjectRecord {
  ObjectRecord {
    section: String::from("stalker"),
    name: String::from("stalker"),
    class: ClsId::AiStlS,
    position: Vec3::new(x, 0.0, 0.0),
    direction: Vec3::Z,
    game_vertex: Some(0),
    level_vertex: Some(0),
    story_id: None,
    spawn_id: 0,
    switching,
  }
}

/// Gives object `index` a schedule row A-Life steers, of a group, which needs updating whenever it is offline.
pub(crate) fn schedule(registry: &mut ObjectRegistry, index: u32) {
  registry.set_schedule(
    id(index),
    ObjectSchedule {
      kind: ObjectScheduleKind::Group,
      direct_control: true,
      updated: None,
    },
  );
}

/// A registry holding `records` on the tests' level, by their place.
pub(crate) fn registry(records: Vec<ObjectRecord>) -> ObjectRegistry {
  let mut registry: ObjectRegistry = ObjectRegistry::default();

  for record in records {
    registry.create(record, Some(LEVEL));
  }

  registry
}
