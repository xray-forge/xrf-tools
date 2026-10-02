use xrf_math::EPS;

use crate::mixer::WeatherMixer;
use crate::playback::weather_effect_start::WeatherEffectStart;
use crate::playback::weather_pair::WeatherPair;
use crate::playback::weather_played_keyframe::WeatherPlayedKeyframe;
use crate::weather::WeatherTime;

/// A weather effect laid over the cycle from the time it started, as `SetWeatherFX` lays it.
#[derive(Clone, Debug, PartialEq)]
pub struct WeatherEffectTimeline {
  pub name: String,
  /// Sorted by time of day: the lead-in, the effect's own keyframes, and the way back into the cycle.
  pub keyframes: Vec<WeatherPlayedKeyframe>,
  /// The pair it starts blending: the lead-in and the cycle's next keyframe, `C0` and `C1`.
  pub start: [WeatherPlayedKeyframe; 2],
  /// The pair it hands the cycle back as it ends, `WFX_end_desc`.
  pub end: [WeatherPlayedKeyframe; 2],
  /// Game seconds from its start until the cycle takes over again, `wfx_time`.
  pub duration: f32,
}

impl WeatherEffectTimeline {
  /// `WFX_TRANS_TIME`: real seconds an effect takes to lead in, and to lead back into the cycle.
  pub const TRANSITION: f32 = 5.0;

  /// `SetWeatherFX`: the cycle's two keyframes around the start, carried on at the weight they stand at into the second
  /// one a lead-in later; the effect's keyframes after that, its first replaced by that second one as the engine
  /// replaces it; then the cycle's keyframe at or after the effect's last, a lead-in on, from which the cycle takes
  /// over with it and the one after. None for an effect or a cycle without keyframes.
  pub fn new(start: WeatherEffectStart<'_>) -> Option<Self> {
    let WeatherEffectStart {
      name,
      effect,
      cycle,
      current: [a, b],
      time,
      factor,
    } = start;

    if effect.is_empty() || cycle.is_empty() {
      return None;
    }

    let rewind: f32 = Self::TRANSITION * factor;
    let begin: f32 = time + rewind;
    let to_next: f32 = WeatherMixer::get_elapsed(WeatherTime::of_day(time), WeatherTime::of_day(b.time));
    let length: f32 = WeatherMixer::get_elapsed(WeatherTime::of_day(a.time), WeatherTime::of_day(b.time));
    // The first keyframe moved back so the weight at the start stays what it was.
    let lead_in: WeatherPlayedKeyframe = a.at(WeatherTime::of_day(if to_next < EPS {
      time
    } else {
      time - ((rewind / to_next) * length - rewind)
    }));
    let played: Vec<WeatherPlayedKeyframe> = std::iter::once(b.at(WeatherTime::of_day(begin)))
      .chain(
        effect[1..]
          .iter()
          .map(|keyframe| keyframe.at(WeatherTime::of_day(begin + keyframe.time))),
      )
      .collect();
    let last: f32 = begin + effect.last().filter(|_| effect.len() > 1).map_or(0.0, |it| it.time);
    // `SelectEnv` twice: the cycle's first keyframe at or after the effect's last, and the one after that.
    let back: &WeatherPlayedKeyframe = WeatherPair::select_next(cycle, WeatherTime::of_day(last)).unwrap_or(b);
    let after: &WeatherPlayedKeyframe =
      WeatherPair::select_next(cycle, WeatherTime::of_day(back.time + 0.5)).unwrap_or(back);
    let start: [WeatherPlayedKeyframe; 2] = [lead_in.clone(), played[0].clone()];
    let mut keyframes: Vec<WeatherPlayedKeyframe> = std::iter::once(lead_in)
      .chain(played)
      .chain(std::iter::once(back.at(WeatherTime::of_day(last + rewind))))
      .collect();

    keyframes.sort_by(|first, second| first.time.total_cmp(&second.time));

    Some(Self {
      name: name.to_owned(),
      keyframes,
      start,
      end: [back.clone(), after.clone()],
      duration: last + rewind - time,
    })
  }
}
