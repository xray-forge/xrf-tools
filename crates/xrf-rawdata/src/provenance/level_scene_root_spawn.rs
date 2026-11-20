use serde::{Deserialize, Serialize};

use crate::provenance::level_scene_disagreement::LevelSceneDisagreement;
use crate::provenance::level_scene_update_packet::LevelSceneUpdatePacket;

/// The root spawn a scene's entities were decompiled from: taken for the level it holds in place of
/// `level.spawn`'s entities, with what the two disagree on and what the scene cannot carry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneRootSpawn {
  /// Its path as given, such as `../../spawns/all.spawn`.
  pub file: String,
  /// Its SHA-256, lowercase hex.
  pub sha256: String,
  /// Its objects standing on the level, which the scene takes; none where it does not hold the level, whose entities
  /// then come from `level.spawn`.
  pub objects: usize,
  /// Its objects of the whole spawn that did not read, on whichever level they stand, which cannot be told.
  pub unread: usize,
  pub disagreements: Vec<LevelSceneDisagreement>,
  pub update_packets: Vec<LevelSceneUpdatePacket>,
}
