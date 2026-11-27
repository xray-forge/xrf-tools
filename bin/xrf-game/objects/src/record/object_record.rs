use xrf_math::glam::Vec3;
use xrf_spawn::{AlifeObject, ClsId};

use crate::id::object_id::ObjectId;
use crate::record::object_switching::ObjectSwitching;

/// What every object of the game is, whatever system moves it: its section and class, where it stands in the engine's
/// space, on which level and graph vertices, what holds it, its story and spawn ids, and how it switches. The game's own
/// record, read once from the spawn; the spawn's packet types stay at that edge. What only some objects have (a
/// creature's health, an ammo box's rounds) is a concern table of the registry's, never a field here.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectRecord {
  /// The config section it is built from.
  pub section: String,
  /// The name the level gives it.
  pub name: String,
  pub class: ClsId,
  pub position: Vec3,
  pub direction: Vec3,
  /// The graph level it stands on, by its id, none for an object with no game vertex.
  pub level: Option<u8>,
  pub game_vertex: Option<u16>,
  pub level_vertex: Option<u32>,
  /// The object holding it, as a stalker holds an item.
  pub parent: Option<ObjectId>,
  pub story_id: Option<u32>,
  /// Its entry in the spawn (`CSE_Abstract::m_tSpawnID`), which X-Ray's spawn registry keys spawns by.
  pub spawn_id: u16,
  /// How it goes online and offline.
  pub switching: ObjectSwitching,
}

impl ObjectRecord {
  /// The name X-Ray gives an object created during a game (`CALifeSimulatorBase::spawn_item`'s `name_replace`): its
  /// section and its id, padded to four digits.
  pub fn name_created(section: &str, id: ObjectId) -> String {
    format!("{section}{:04}", id.0)
  }

  /// The record of a spawned object, standing on graph level `level` and held by `parent`, which the registry resolves
  /// from the spawn's ids.
  pub fn of_spawn(object: &AlifeObject, level: Option<u8>, parent: Option<ObjectId>) -> Self {
    let header = &object.header;
    let placed = object.inherited.get_abstract();

    Self {
      section: header.section.clone(),
      name: header.name.clone(),
      class: object.clsid.clone(),
      position: Vec3::new(header.position.x, header.position.y, header.position.z),
      direction: Vec3::new(header.direction.x, header.direction.y, header.direction.z),
      level,
      game_vertex: placed.map(|placed| placed.game_vertex_id),
      level_vertex: placed.map(|placed| placed.level_vertex_id),
      parent,
      story_id: placed.and_then(|placed| placed.get_story_id()),
      spawn_id: header.spawn_id,
      switching: ObjectSwitching::of_spawn(object),
    }
  }
}
