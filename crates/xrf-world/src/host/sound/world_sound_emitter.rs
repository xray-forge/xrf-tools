use std::sync::Arc;
use std::time::Duration;

use crate::contract::sound::world_sound_kind::WorldSoundKind;

/// One sound a scene plays, as its frame tells it: what plays where, from when on the level's clock, and how the
/// engine's emitter would play it. Its place in the sound follows from the clock, never from what was heard.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldSoundEmitter {
  /// Stable while the same play goes on; a sound played again takes a new one.
  pub id: u64,
  /// The sound's reference.
  pub sound: Arc<str>,
  pub kind: WorldSoundKind,
  /// The level object it plays at, by the id its helper carries; none for a sound of no object.
  pub object: Option<u32>,
  /// In engine space; for a 2D sound, from the listener.
  pub position: [f32; 3],
  /// When its first frame plays, on the level's clock; later than the frame for a delayed start.
  pub start: Duration,
  /// When a deferred stop began, on the level's clock: it fades out from there.
  pub stop: Option<Duration>,
  pub is_looped: bool,
  pub is_2d: bool,
  /// `p_source.volume`, before the file's base volume.
  pub volume: f32,
  /// Playback speed, `p_source.freq`.
  pub frequency: f32,
  pub min_distance: f32,
  pub max_distance: f32,
  pub base_volume: f32,
  /// Whether it feeds the reverb: never for Monolith's music, played `sm_Intro`.
  pub has_reverb: bool,
}
