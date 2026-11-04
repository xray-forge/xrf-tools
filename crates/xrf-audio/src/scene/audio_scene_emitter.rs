use std::sync::Arc;
use std::time::Duration;

use crate::mixer::audio_bus::AudioBus;

/// One sound a scene plays, as its host schedules it: the engine's emitter, whose place in the sound follows from the
/// scene's clock.
#[derive(Clone, Debug, PartialEq)]
pub struct AudioSceneEmitter {
  /// Stable while the same play goes on; a new play takes a new one.
  pub id: u64,
  /// The sound's reference, read through the scene's asset source.
  pub sound: Arc<str>,
  pub bus: AudioBus,
  /// In the scene's space; for a 2D sound, from the listener.
  pub position: [f32; 3],
  /// When it starts playing, on the scene's clock.
  pub start: Duration,
  /// How far into its sound it starts, at its own speed: nought from its first frame.
  pub offset: Duration,
  /// When a deferred stop began, on the scene's clock.
  pub stop: Option<Duration>,
  pub is_looped: bool,
  pub is_2d: bool,
  /// `p_source.volume`.
  pub volume: f32,
  /// Playback speed, `p_source.freq`.
  pub frequency: f32,
  pub min_distance: f32,
  pub max_distance: f32,
  pub base_volume: f32,
  /// Whether the listener's reverb hears it.
  pub has_reverb: bool,
}
