use std::time::Duration;

use xrf_audio_scene::AudioSceneEmitter;

use crate::contract::sound::world_sound_kind::WorldSoundKind;

/// One sound a scene plays, as the world schedules it: the emitter its frames hand the audio, what kind of sound it is
/// and the level object it plays at, and when it plays out. Its place in the sound follows from the clock, never from
/// what was heard.
#[derive(Clone, Debug)]
pub(crate) struct LiveEmitter {
  pub(crate) emitter: AudioSceneEmitter,
  pub(crate) kind: WorldSoundKind,
  /// The level object it plays at, by the id its helper carries; none for a sound of no object.
  pub(crate) object: Option<u32>,
  /// When it plays out on the level's clock, none while looped.
  pub(crate) end: Option<Duration>,
  /// Whether no frame has carried it yet, which keeps a short sound for the next frame even once it is over.
  pub(crate) is_new: bool,
}
