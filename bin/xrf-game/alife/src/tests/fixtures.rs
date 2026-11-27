use xrf_game_objects::{ObjectId, ObjectRecord, ObjectRegistry, ObjectSchedule, ObjectScheduleKind, ObjectSwitching};
use xrf_math::glam::Vec3;
use xrf_spawn::ClsId;

/// The level the tests stand on, and another.
pub(crate) const LEVEL: u8 = 108;
pub(crate) const OTHER_LEVEL: u8 = 118;

/// An object at `x` metres along `+x` from the origin, where the actor stands.
pub(crate) fn object(x: f32, switching: ObjectSwitching) -> ObjectRecord {
  ObjectRecord {
    section: String::from("stalker"),
    name: String::from("stalker"),
    class: ClsId::AiStlS,
    position: Vec3::new(x, 0.0, 0.0),
    direction: Vec3::Z,
    level: Some(LEVEL),
    game_vertex: Some(0),
    level_vertex: Some(0),
    parent: None,
    story_id: None,
    spawn_id: 0,
    switching,
  }
}

/// Gives `id` a schedule row A-Life steers, of a group, which needs updating whenever it is offline.
pub(crate) fn schedule(registry: &mut ObjectRegistry, id: u16) {
  registry.get_schedules_mut().insert(
    ObjectId(id),
    ObjectSchedule {
      kind: ObjectScheduleKind::Group,
      direct_control: true,
      updated: None,
    },
  );
}

/// A registry holding `records`, by their place.
pub(crate) fn registry(records: Vec<ObjectRecord>) -> ObjectRegistry {
  let mut registry: ObjectRegistry = ObjectRegistry::default();

  for (id, record) in (0..).map(ObjectId).zip(records) {
    registry.insert(id, record);
  }

  registry
}
