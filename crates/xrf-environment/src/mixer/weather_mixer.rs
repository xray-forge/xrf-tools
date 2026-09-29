use xrf_engine_target::XrayEngine;

use crate::mixer::weather_mix::WeatherMix;
use crate::mixer::weather_sun_source::WeatherSunSource;
use crate::weather::{WeatherDescriptor, WeatherTime};

/// `EPS`, what `TimeWeight` takes a zero span by.
const EPS: f32 = 0.000_01;

/// Mixes a cycle's keyframes at a time of day as `CEnvironment::lerp` does, without modifiers.
#[derive(Clone, Copy, Debug)]
pub struct WeatherMixer<'a> {
  /// Sorted by time, as a cycle holds them.
  pub keyframes: &'a [WeatherDescriptor],
  pub engine: XrayEngine,
  pub sun: WeatherSunSource<'a>,
}

impl WeatherMixer<'_> {
  /// The keyframes mixed at a time of day, in seconds since midnight; none for a cycle without any.
  pub fn mix(&self, time: f32) -> Option<WeatherMix> {
    let time: f32 = time.rem_euclid(WeatherTime::DAY as f32);
    let [from, to] = self.select(time)?;
    let (a, b) = (&self.keyframes[from], &self.keyframes[to]);
    let f: f32 = Self::weigh(time, [a.time as f32, b.time as f32]);
    let scalar = |a: f32, b: f32| (1.0 - f) * a + f * b;
    let vector = |a: &[f32], b: &[f32]| -> Vec<f32> { a.iter().zip(b).map(|(a, b)| scalar(*a, *b)).collect() };
    let far_plane: f32 = scalar(a.far_plane, b.far_plane);
    let fog_density: f32 = scalar(a.fog_density, b.fog_density);
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
    let mut sun_color: [f32; 3] = to_array(&vector(&a.sun_color, &b.sun_color));
    let sun_direction: [f32; 3] = match self.sun {
      WeatherSunSource::Authored => {
        let down: [f32; 3] = [0.0, -1.0, 0.0];

        normalise(to_array(&vector(
          &a.sun_direction.unwrap_or(down),
          &b.sun_direction.unwrap_or(down),
        )))
      }
      // The engine passes the mixed `exec_time`, which runs backwards across midnight; the time of day it stands for
      // is the same everywhere else.
      WeatherSunSource::Dynamic => {
        let (direction, blend) = WeatherSunSource::dynamic(time, scalar(a.sun_azimuth, b.sun_azimuth));

        sun_color = sun_color.map(|channel| channel * blend);

        direction
      }
      WeatherSunSource::Table(positions) => WeatherSunSource::table(positions, time),
    };

    Some(WeatherMix {
      ambient_color: to_array(&vector(&a.ambient_color, &b.ambient_color)),
      far_plane,
      fog_color: to_array(&vector(&a.fog_color, &b.fog_color)),
      fog_density,
      fog_distance,
      fog_far: 0.99 * fog_distance,
      fog_near: (1.0 - fog_density) * 0.85 * fog_distance,
      hemi_color: to_array(&vector(&a.hemi_color, &b.hemi_color)),
      keyframes: [from, to],
      sky_color: to_array(&vector(&a.sky_color, &b.sky_color)),
      sky_rotation: scalar(a.sky_rotation, b.sky_rotation),
      sun_color,
      sun_direction,
      time,
      tree_amplitude: scalar(a.tree_amplitude, b.tree_amplitude),
      tree_rotation: scalar(a.tree_rotation, b.tree_rotation),
      tree_speed: scalar(a.tree_speed, b.tree_speed),
      tree_wave: to_array(&vector(&a.tree_wave, &b.tree_wave)),
      water_intensity: scalar(a.water_intensity, b.water_intensity),
      weight: f,
    })
  }

  /// `SelectEnvs` on a forced start: the first keyframe at or after the time and the one before it, the last and the
  /// first around midnight.
  pub fn select(&self, time: f32) -> Option<[usize; 2]> {
    let last: usize = self.keyframes.len().checked_sub(1)?;
    let next: usize = self.keyframes.partition_point(|keyframe| (keyframe.time as f32) < time);

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
  fn get_elapsed(from: f32, to: f32) -> f32 {
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
