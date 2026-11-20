use xrf_editable_level::EditableSpawnPointObject;

use crate::provenance::level_scene_disagreement::LevelSceneDisagreement;
use crate::provenance::level_scene_refusal::LevelSceneRefusal;
use crate::provenance::level_scene_update_packet::LevelSceneUpdatePacket;

/// A level's entities decompiled: the spawn points, the entities refused, and from a root spawn, what it and
/// `level.spawn` disagree on and the update packets the scene cannot keep.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelEntityDecompile {
  pub entities: Vec<EditableSpawnPointObject>,
  pub refused: Vec<LevelSceneRefusal>,
  pub disagreements: Vec<LevelSceneDisagreement>,
  pub update_packets: Vec<LevelSceneUpdatePacket>,
}
