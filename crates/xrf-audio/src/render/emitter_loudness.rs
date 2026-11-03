use crate::scene::audio_scene_emitter::AudioSceneEmitter;
use crate::scene::audio_scene_rules::AudioSceneRules;

/// `psSoundCull`: below this a sound is not worth a target.
pub(crate) const CULL: f32 = 0.01;

/// `psSoundRolloff`, which both engines hand OpenAL for every source.
pub(crate) const ROLLOFF: f32 = 0.75;

/// `EPS_S`: the least gain a target is handed.
const LEAST_GAIN: f32 = 0.000_000_1;

/// The engine's updates a second the smoothing below is written for: OpenXRay moves a tenth of the way each frame, so a
/// tick of another length moves as many frames' worth.
const UPDATE_RATE: f32 = 60.0;

/// `priority_scale` of a 2D sound (`switch_to_2D`).
const PRIORITY_2D: f32 = 100.0;

/// How loud an emitter plays, as `CSoundRender_Emitter::update_culling` keeps it from one update to the next: its fade
/// in and out, how occluded it is, and its smoothed volume, from which its target's gain and its priority follow.
///
/// OpenXRay smooths the volume a tenth of the way to its target each update and leaves the distance to OpenAL's
/// inverse-clamped model; Monolith assigns the volume, falling linearly from the minimum distance to the maximum, and
/// OpenAL's model still applies on top.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EmitterLoudness {
  smooth: f32,
  fade: f32,
  occluder: f32,
}

/// What an update answers: whether the emitter is loud enough for a target, and how much it deserves one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EmitterCulling {
  pub(crate) is_audible: bool,
  pub(crate) priority: f32,
}

impl EmitterLoudness {
  /// `stStarting`: faded in, unoccluded until the first ray, and smoothed from its volume at once.
  pub(crate) fn start(emitter: &AudioSceneEmitter) -> Self {
    Self {
      smooth: emitter.base_volume * emitter.volume,
      fade: 1.0,
      occluder: 1.0,
    }
  }

  /// The smoothed volume, `smooth_volume`.
  pub(crate) fn get_smooth(&self) -> f32 {
    self.smooth
  }

  /// `update_culling`: `seconds` on, the listener `distance` metres away (none for a 2D sound), `occlusion` the
  /// scene's for it, fading out while `is_stopping`.
  pub(crate) fn update(
    &mut self,
    rules: AudioSceneRules,
    emitter: &AudioSceneEmitter,
    distance: Option<f32>,
    occlusion: f32,
    seconds: f32,
    is_stopping: bool,
  ) -> EmitterCulling {
    let fade_step: f32 = seconds * 10.0;
    let volume: f32 = emitter.base_volume * emitter.volume;
    let attenuation: f32 = match distance {
      None => {
        self.occluder = 1.0;
        self.fade += if is_stopping { -fade_step } else { fade_step };

        match rules {
          AudioSceneRules::Vanilla => 1.0,
          AudioSceneRules::Extended => emitter.volume,
        }
      }
      Some(distance) => {
        if distance > emitter.max_distance {
          self.smooth = 0.0;

          return EmitterCulling {
            is_audible: false,
            priority: 0.0,
          };
        }

        let attenuation: f32 = match rules {
          AudioSceneRules::Vanilla => Self::get_inverse(emitter, distance),
          // The fall over the range is capped by the volume rather than scaled by it.
          AudioSceneRules::Extended => Self::get_linear(emitter, distance).clamp(0.0, emitter.volume.max(0.0)),
        };
        let heard: f32 = match rules {
          AudioSceneRules::Vanilla => attenuation * volume,
          AudioSceneRules::Extended => volume,
        };

        self.fade += if is_stopping || heard < CULL {
          -fade_step
        } else {
          fade_step
        };
        self.occluder = Self::approach(self.occluder, occlusion, seconds).clamp(0.0, 1.0);

        attenuation
      }
    };

    self.fade = self.fade.clamp(0.0, 1.0);

    match rules {
      AudioSceneRules::Vanilla => {
        let target: f32 = volume * self.occluder * self.fade;
        let keep: f32 = 0.9f32.powf(seconds * UPDATE_RATE);

        self.smooth = keep * self.smooth + (1.0 - keep) * target;
      }
      AudioSceneRules::Extended => {
        self.smooth = emitter.base_volume * attenuation * self.occluder * self.fade;
      }
    }

    EmitterCulling {
      is_audible: self.smooth >= CULL,
      priority: self.get_priority(rules, emitter, distance),
    }
  }

  /// Whether a stopping emitter has faded out, when the engine lets go of its sound.
  pub(crate) fn is_faded(&self) -> bool {
    self.fade <= 0.0
  }

  /// The gain its target plays at, `AL_GAIN` after OpenAL's inverse-clamped distance model for a 3D source.
  pub(crate) fn get_gain(&self, emitter: &AudioSceneEmitter, distance: Option<f32>) -> f32 {
    let gain: f32 = self.smooth.clamp(LEAST_GAIN, 1.0);

    match distance {
      None => gain,
      Some(distance) => gain * Self::get_openal(emitter, distance),
    }
  }

  /// `priority`: how much it deserves a target.
  fn get_priority(&self, rules: AudioSceneRules, emitter: &AudioSceneEmitter, distance: Option<f32>) -> f32 {
    match (rules, distance) {
      (_, None) => self.smooth * PRIORITY_2D,
      (AudioSceneRules::Vanilla, Some(distance)) => self.smooth * Self::get_inverse(emitter, distance),
      (AudioSceneRules::Extended, Some(distance)) => {
        self.smooth * Self::get_linear(emitter, distance).clamp(0.0, emitter.volume.max(0.0))
      }
    }
  }

  /// `min_distance / (psSoundRolloff * dist)`, held within nought and one.
  fn get_inverse(emitter: &AudioSceneEmitter, distance: f32) -> f32 {
    (emitter.min_distance / (ROLLOFF * distance)).clamp(0.0, 1.0)
  }

  /// Monolith's `(max_distance - dist) / (max_distance - min_distance)`.
  fn get_linear(emitter: &AudioSceneEmitter, distance: f32) -> f32 {
    (emitter.max_distance - distance) / (emitter.max_distance - emitter.min_distance)
  }

  /// `AL_INVERSE_DISTANCE_CLAMPED` with the source's range as reference and maximum distance.
  fn get_openal(emitter: &AudioSceneEmitter, distance: f32) -> f32 {
    let reference: f32 = emitter.min_distance.max(0.0);
    let clamped: f32 = distance.clamp(reference, emitter.max_distance.max(reference));
    let denominator: f32 = reference + ROLLOFF * (clamped - reference);

    if denominator > 0.0 {
      reference / denominator
    } else {
      1.0
    }
  }

  /// `volume_lerp` at one a second.
  fn approach(current: f32, target: f32, seconds: f32) -> f32 {
    let difference: f32 = target - current;

    if difference.abs() < LEAST_GAIN {
      current
    } else {
      current + difference.signum() * seconds.min(difference.abs())
    }
  }
}
