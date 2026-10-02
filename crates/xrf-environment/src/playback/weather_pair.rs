use crate::mixer::WeatherMixer;
use crate::playback::weather_played_keyframe::WeatherPlayedKeyframe;

/// `Current[0]` and `Current[1]`: the two keyframes the engine blends between, kept from frame to frame.
///
/// The pair moves on only once the clock passes its second keyframe, taking the next from whatever plays by then, so a
/// weather changed under it blends in over the rest of the span rather than at once (`SetWeather(name, false)`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeatherPair {
  pair: Option<[WeatherPlayedKeyframe; 2]>,
}

impl WeatherPair {
  /// The pair, or none before anything was selected.
  pub fn get(&self) -> Option<&[WeatherPlayedKeyframe; 2]> {
    self.pair.as_ref()
  }

  /// `SelectEnvs` on a forced start: the keyframes around the time, sorted by time, whatever was blended before.
  pub fn force(&mut self, keyframes: &[WeatherPlayedKeyframe], time: f32) {
    self.pair = WeatherMixer::select_by(keyframes, |keyframe| keyframe.time, time)
      .map(|[from, to]| [keyframes[from].clone(), keyframes[to].clone()]);
  }

  /// What to blend between from now on, as `StopWFX` hands the cycle back its keyframes.
  pub fn set(&mut self, pair: [WeatherPlayedKeyframe; 2]) {
    self.pair = Some(pair);
  }

  /// Forgets the pair, so the next advance selects afresh.
  pub fn reset(&mut self) {
    self.pair = None;
  }

  /// `SelectEnvs` once started: past the second keyframe, it becomes the first and the next is the first at or after
  /// the time; across midnight, only while the time is between the two.
  pub fn advance(&mut self, keyframes: &[WeatherPlayedKeyframe], time: f32) {
    let Some([from, to]) = &self.pair else {
      return self.force(keyframes, time);
    };
    let is_past: bool = if from.time > to.time {
      time > to.time && time < from.time
    } else {
      time > to.time
    };

    if is_past {
      let next: WeatherPlayedKeyframe = Self::select_next(keyframes, time).unwrap_or(to).clone();

      self.pair = Some([to.clone(), next]);
    }
  }

  /// `SelectEnv`: the first keyframe at or after a time, the first of the day past the last.
  pub fn select_next(keyframes: &[WeatherPlayedKeyframe], time: f32) -> Option<&WeatherPlayedKeyframe> {
    keyframes
      .iter()
      .find(|keyframe| keyframe.time >= time)
      .or(keyframes.first())
  }
}
