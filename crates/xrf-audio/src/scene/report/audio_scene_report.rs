use crate::scene::report::audio_acoustics_report::AudioAcousticsReport;
use crate::scene::report::audio_emitter_report::AudioEmitterReport;
use crate::scene::report::audio_scene_refusal::AudioSceneRefusal;

/// How the control thread plays a scene as of its last tick: each emitter's gain and whether it holds a voice, the
/// sounds it could not play, its targets, and how its acoustics run.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AudioSceneReport {
  pub emitters: Vec<AudioEmitterReport>,
  pub refused: Vec<AudioSceneRefusal>,
  /// Emitters holding a voice, and the engine's targets for them.
  pub real_voices: u32,
  pub targets: u32,
  /// The mixer's share of the device's period, as last measured.
  pub mixer_load: f32,
  pub acoustics: AudioAcousticsReport,
}
