//! What a post-process effect does to the screen over time, as `CPostprocessAnimator::Process` plays it.

use std::time::Duration;

use xrf_animation_envelope::{AnimationEnvelope, AnimationPlayback};

use crate::{PpeColor, PpeFile, PpeScreen};

/// What a keyed noise fps is multiplied by: the file stores the rate in hundredths.
const NOISE_FPS_SCALE: f32 = 100.0;

impl PpeFile {
  /// What the effect does to the screen `time` seconds into it, at full strength: each colour is the identity's plus
  /// its own channels, a noise parameter carrying no keys keeps the identity's value, a keyed noise fps plays at a
  /// hundred times its stored value, and the rest are their envelopes' values.
  pub fn evaluate(&self, time: f32) -> PpeScreen {
    let identity: PpeScreen = PpeScreen::IDENTITY;
    let color = |color: &PpeColor, identity: [f32; 3]| {
      let [red, green, blue] = color.evaluate(time);

      [identity[0] + red, identity[1] + green, identity[2] + blue]
    };
    let noise = |envelope: &AnimationEnvelope, identity: f32, scale: f32| {
      if envelope.keys.is_empty() {
        identity
      } else {
        envelope.evaluate(time) * scale
      }
    };

    PpeScreen {
      base_color: color(&self.base_color, identity.base_color),
      add_color: color(&self.add_color, identity.add_color),
      gray_color: color(&self.gray_color, identity.gray_color),
      gray_value: self.gray_value.evaluate(time),
      blur: self.blur.evaluate(time),
      duality_horizontal: self.duality_horizontal.evaluate(time),
      duality_vertical: self.duality_vertical.evaluate(time),
      noise_intensity: noise(&self.noise_intensity, identity.noise_intensity, 1.0),
      noise_granularity: noise(&self.noise_granularity, identity.noise_granularity, 1.0),
      noise_fps: noise(&self.noise_fps, identity.noise_fps, NOISE_FPS_SCALE),
      color_map_influence: self
        .color_map
        .as_ref()
        .map_or(0.0, |color_map| color_map.influence.evaluate(time)),
    }
  }

  /// The time, in seconds, the effect plays at `elapsed` after it starts: from zero rather than from its first key, and
  /// past [`Self::get_length_seconds`], a looped (cyclic) effect starts again by whole lengths while one played once
  /// runs on, its envelopes' behaviours deciding what it shows, until its effector ends that long after it began. An
  /// effect of no length never starts again.
  pub fn get_played_time(&self, elapsed: Duration, playback: AnimationPlayback) -> f32 {
    let Some(period) = Duration::try_from_secs_f32(self.get_length_seconds())
      .ok()
      .filter(|period| !period.is_zero())
    else {
      return elapsed.as_secs_f32();
    };

    match playback {
      AnimationPlayback::Looped if elapsed > period => {
        Duration::from_nanos((elapsed.as_nanos() % period.as_nanos()) as u64).as_secs_f32()
      }
      AnimationPlayback::Looped | AnimationPlayback::Once => elapsed.as_secs_f32(),
    }
  }
}
