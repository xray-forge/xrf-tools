use xrf_game_objects::{ObjectRecord, ObjectSwitching};
use xrf_math::glam::Vec3;
use xrf_spawn::ClsId;

/// The player's record as a new game holds it, at the origin.
pub(crate) fn player() -> ObjectRecord {
  ObjectRecord {
    section: String::from("actor"),
    name: String::from("actor"),
    class: ClsId::SActor,
    position: Vec3::ZERO,
    direction: Vec3::ZERO,
    game_vertex: None,
    level_vertex: None,
    story_id: None,
    spawn_id: 0,
    switching: ObjectSwitching::ByDistance,
  }
}
