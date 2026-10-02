use xrf_engine_target::XrayEngine;
use xrf_math::EPS;

use crate::mixer::weather_mix::WeatherMix;
use crate::mixer::weather_mix_keyframe::WeatherMixKeyframe;
use crate::mixer::weather_mix_point::WeatherMixPoint;
use crate::mixer::weather_modifier::WeatherModifier;
use crate::mixer::weather_modifiers_sum::WeatherModifiersSum;
use crate::mixer::weather_sun_source::WeatherSunSource;
use crate::weather::{WeatherDescriptor, WeatherTime};

/// Mixes keyframes at a time of day as `CEnvironment::lerp` does, with the modifiers reaching the view.
#[derive(Clone, Copy, Debug)]
pub struct WeatherMixer<'a> {
  pub engine: XrayEngine,
  pub sun: WeatherSunSource<'a>,
  /// The level's `level.env_mod` volumes.
  pub modifiers: &'a [WeatherModifier],
}

impl WeatherMixer<'_> {
  /// A cycle's keyframes, sorted by time, mixed at a time of day around it, seen from a point; none for a cycle without
  /// any.
  pub fn mix(&self, keyframes: &[WeatherDescriptor], point: WeatherMixPoint) -> Option<WeatherMix> {
    let time: f32 = WeatherTime::of_day(point.time);
    let [from, to] = Self::select(keyframes, time)?;

    Some(self.mix_pair(
      [
        WeatherMixKeyframe::of(&keyframes[from]),
        WeatherMixKeyframe::of(&keyframes[to]),
      ],
      point,
    ))
  }

  /// Two keyframes mixed at a time of day between them, seen from a point: the pair the engine blends, which keeps it
  /// from frame to frame rather than selecting around the time each.
  pub fn mix_pair(&self, [a, b]: [WeatherMixKeyframe<'_>; 2], point: WeatherMixPoint) -> WeatherMix {
    let WeatherMixPoint { time, view } = point;
    let time: f32 = WeatherTime::of_day(time);
    let modified: WeatherModifiersSum = WeatherModifiersSum::at(self.modifiers, view);
    let scale: f32 = modified.get_scale();
    let between: [f32; 2] = [a.time, b.time];
    let (a, b) = (a.descriptor, b.descriptor);
    let f: f32 = Self::weigh(time, between);
    let scalar = |a: f32, b: f32| (1.0 - f) * a + f * b;
    let vector = |a: &[f32], b: &[f32]| -> Vec<f32> { a.iter().zip(b).map(|(a, b)| scalar(*a, *b)).collect() };
    // A value some modifier reaches is the mix plus what they add, of which the environment keeps its share.
    let modify = |flag: u16, value: f32, added: f32| {
      if modified.has(flag) {
        (value + added) * scale
      } else {
        value
      }
    };
    let modify_vector = |flag: u16, value: Vec<f32>, added: [f32; 3]| -> [f32; 3] {
      std::array::from_fn(|axis| modify(flag, value[axis], added[axis]))
    };
    let far_plane: f32 = modify(
      WeatherModifier::FAR_PLANE,
      scalar(a.far_plane, b.far_plane),
      modified.far_plane,
    );
    let fog_density: f32 = modify(
      WeatherModifier::FOG_DENSITY,
      scalar(a.fog_density, b.fog_density),
      modified.fog_density,
    );
    let hemi_color: [f32; 4] = {
      let [red, green, blue] = modify_vector(
        WeatherModifier::HEMI_COLOR,
        vector(&a.hemi_color, &b.hemi_color),
        modified.hemi_color,
      );

      [red, green, blue, scalar(a.hemi_color[3], b.hemi_color[3])]
    };
    let fog_distance: f32 = match self.engine {
      XrayEngine::Vanilla => scalar(a.fog_distance, b.fog_distance),
      // `clamp(fog_distance, 1.f, far_plane - 10)`, the low bound first.
      XrayEngine::Extended => {
        let distance: f32 = scalar(a.fog_distance, b.fog_distance);

        if distance < 1.0 {
          1.0
        } else {
          distance.min(far_plane - 10.0)
        }
      }
    };
    let (sun_color, sun_direction) = self.mix_sun([a, b], time, f);

    WeatherMix {
      ambient_color: modify_vector(
        WeatherModifier::AMBIENT_COLOR,
        vector(&a.ambient_color, &b.ambient_color),
        modified.ambient,
      ),
      clouds_color: to_array(&vector(&a.clouds_color, &b.clouds_color)),
      clouds_rotation: scalar(a.clouds_rotation, b.clouds_rotation),
      far_plane,
      fog_color: modify_vector(
        WeatherModifier::FOG_COLOR,
        vector(&a.fog_color, &b.fog_color),
        modified.fog_color,
      ),
      fog_density,
      fog_distance,
      fog_far: 0.99 * fog_distance,
      fog_near: (1.0 - fog_density) * 0.85 * fog_distance,
      hemi_color,
      between,
      sky_color: modify_vector(
        WeatherModifier::SKY_COLOR,
        vector(&a.sky_color, &b.sky_color),
        modified.sky_color,
      ),
      sky_rotation: scalar(a.sky_rotation, b.sky_rotation),
      sun_color,
      sun_direction,
      time,
      view,
      tree_amplitude: scalar(a.tree_amplitude, b.tree_amplitude),
      tree_rotation: scalar(a.tree_rotation, b.tree_rotation),
      tree_speed: scalar(a.tree_speed, b.tree_speed),
      tree_wave: to_array(&vector(&a.tree_wave, &b.tree_wave)),
      water_intensity: scalar(a.water_intensity, b.water_intensity),
      thunderbolt_collection: if f < 0.5 {
        &a.thunderbolt_collection
      } else {
        &b.thunderbolt_collection
      }
      .clone(),
      thunderbolt_duration: scalar(a.thunderbolt_duration, b.thunderbolt_duration),
      thunderbolt_period: scalar(a.thunderbolt_period, b.thunderbolt_period),
      rain_color: to_array(&vector(&a.rain_color, &b.rain_color)),
      rain_density: scalar(a.rain_density, b.rain_density),
      wind_direction: scalar(a.wind_direction, b.wind_direction),
      wind_velocity: scalar(a.wind_velocity, b.wind_velocity),
      weight: f,
      modifiers: modified.count,
    }
  }

  /// The sun a mix stands, its colour and the direction its light travels: the keyframes' own directions blended,
  /// the astronomical sun, or Monolith's sun table.
  fn mix_sun(&self, [a, b]: [&WeatherDescriptor; 2], time: f32, f: f32) -> ([f32; 3], [f32; 3]) {
    let scalar = |a: f32, b: f32| (1.0 - f) * a + f * b;
    let color: [f32; 3] = std::array::from_fn(|axis| scalar(a.sun_color[axis], b.sun_color[axis]));

    match self.sun {
      WeatherSunSource::Authored => {
        let down: [f32; 3] = [0.0, -1.0, 0.0];
        let [from, to] = [a.sun_direction.unwrap_or(down), b.sun_direction.unwrap_or(down)];

        (
          color,
          normalise(std::array::from_fn(|axis| scalar(from[axis], to[axis]))),
        )
      }
      // The engine passes the mixed `exec_time`, which runs backwards across midnight; the time of day it stands for
      // is the same everywhere else.
      WeatherSunSource::Dynamic => {
        let (direction, blend) = WeatherSunSource::dynamic(time, scalar(a.sun_azimuth, b.sun_azimuth));

        (color.map(|channel| channel * blend), direction)
      }
      WeatherSunSource::Table(positions) => (color, WeatherSunSource::table(positions, time)),
    }
  }

  /// `SelectEnvs` on a forced start: the first keyframe at or after the time and the one before it, the last and the
  /// first around midnight.
  pub fn select(keyframes: &[WeatherDescriptor], time: f32) -> Option<[usize; 2]> {
    Self::select_by(keyframes, |keyframe| keyframe.time as f32, time)
  }

  /// [`Self::select`] over anything standing at a time of day, sorted by it.
  pub fn select_by<T>(keyframes: &[T], time_of: impl Fn(&T) -> f32, time: f32) -> Option<[usize; 2]> {
    let last: usize = keyframes.len().checked_sub(1)?;
    let next: usize = keyframes.partition_point(|keyframe| time_of(keyframe) < time);

    Some(match next {
      _ if next > last => [last, 0],
      0 => [last, 0],
      _ => [next - 1, next],
    })
  }

  /// `TimeWeight`: how far a time is from one keyframe's to the next's, around midnight where the first is later.
  pub fn weigh(time: f32, [from, to]: [f32; 2]) -> f32 {
    let span: f32 = Self::get_elapsed(from, to);

    if span.abs() < EPS {
      return 0.0;
    }

    let is_within: bool = if from > to {
      time >= from || time <= to
    } else {
      time >= from && time <= to
    };

    if is_within {
      (Self::get_elapsed(from, time) / span).clamp(0.0, 1.0)
    } else {
      0.0
    }
  }

  /// `TimeDiff`: seconds from one time of day to the next, around midnight where it comes first.
  pub fn get_elapsed(from: f32, to: f32) -> f32 {
    if from > to {
      WeatherTime::DAY as f32 - from + to
    } else {
      to - from
    }
  }
}

fn to_array<const N: usize>(values: &[f32]) -> [f32; N] {
  std::array::from_fn(|index| values.get(index).copied().unwrap_or(0.0))
}

fn normalise([x, y, z]: [f32; 3]) -> [f32; 3] {
  let length: f32 = (x * x + y * y + z * z).sqrt();

  if length > 0.0 {
    [x / length, y / length, z / length]
  } else {
    [0.0, -1.0, 0.0]
  }
}
