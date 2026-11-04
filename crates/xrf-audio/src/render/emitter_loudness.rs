use crate::acoustics::distance_curve::DistanceCurve;
use crate::render::emitter_culling::EmitterCulling;
use crate::scene::audio_scene_emitter::AudioSceneEmitter;

/// Below this a sound is not worth a voice.
const CULL: f32 = 0.01;

/// How much more a sound heard as it was recorded deserves a voice than a placed one of the same loudness: music and
/// the weather's 2D sounds keep theirs while placed sounds crowd in.
const PRIORITY_2D: f32 = 100.0;

/// How loud an emitter plays, one model for every engine: its volume, the distance curve from its minimum and maximum
/// distance for a placed sound, and a fade in and out at ten a second, from which whether it deserves a voice, and how
/// much, follow. How walls and rooms change it is the acoustics', not this.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EmitterLoudness {
  fade: f32,
}

impl EmitterLoudness {
  /// Faded in.
  pub(crate) fn start() -> Self {
    Self { fade: 1.0 }
  }

  /// `seconds` on, the listener `distance` metres away (none for a 2D sound), fading out while `is_stopping` or too
  /// quiet to hear.
  pub(crate) fn update(
    &mut self,
    emitter: &AudioSceneEmitter,
    distance: Option<f32>,
    seconds: f32,
    is_stopping: bool,
  ) -> EmitterCulling {
    if distance.is_some_and(|distance| distance > emitter.max_distance) {
      self.fade = 0.0;

      return EmitterCulling {
        is_audible: false,
        priority: 0.0,
      };
    }

    let heard: f32 = Self::get_volume(emitter) * Self::get_attenuation(emitter, distance);
    let step: f32 = seconds * 10.0;

    self.fade = (self.fade + if is_stopping || heard < CULL { -step } else { step }).clamp(0.0, 1.0);

    let loudness: f32 = heard * self.fade;

    EmitterCulling {
      is_audible: loudness >= CULL,
      priority: if distance.is_none() {
        loudness * PRIORITY_2D
      } else {
        loudness
      },
    }
  }

  /// Whether a stopping emitter has faded out, when its sound is let go of.
  pub(crate) fn is_faded(&self) -> bool {
    self.fade <= 0.0
  }

  /// The gain before distance: its volume and fade.
  pub(crate) fn get_gain(&self, emitter: &AudioSceneEmitter) -> f32 {
    Self::get_volume(emitter) * self.fade
  }

  /// The distance curve's gain for a placed sound `distance` metres away, 1 for a 2D one.
  pub(crate) fn get_attenuation(emitter: &AudioSceneEmitter, distance: Option<f32>) -> f32 {
    distance.map_or(1.0, |distance| {
      DistanceCurve::get_attenuation(emitter.min_distance, emitter.max_distance, distance)
    })
  }

  fn get_volume(emitter: &AudioSceneEmitter) -> f32 {
    (emitter.base_volume * emitter.volume).max(0.0)
  }
}
