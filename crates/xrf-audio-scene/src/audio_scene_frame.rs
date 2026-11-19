use std::time::Duration;

use crate::audio_bus::AudioBus;
use crate::audio_listener::AudioListener;
use crate::audio_scene_emitter::AudioSceneEmitter;
use crate::audio_scene_rules::AudioSceneRules;

/// A scene's sound at a moment on its clock, which the control thread plays on from until the next frame. Frames travel
/// through an [`crate::AudioSceneInbox`] and come back to be refilled.
#[derive(Clone, Debug, Default)]
pub struct AudioSceneFrame {
  /// The host's id for the scene, which its report is asked by.
  pub scene: u32,
  /// The scene's clock as of the frame.
  pub time: Duration,
  pub rules: AudioSceneRules,
  /// Whether it is heard; a scene not heard fades out and plays on, silent.
  pub is_audible: bool,
  pub listener: AudioListener,
  /// Which buses of its sound are heard, by [`AudioBus::get_index`].
  pub heard: [bool; AudioBus::COUNT],
  pub emitters: Vec<AudioSceneEmitter>,
}
