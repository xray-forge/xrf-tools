//! The wind the weather's ambient effects bring: the noise's strength and the blast's rise, turn and fall.

use std::time::Duration;

use glam::Vec3;
use xrf_renderer::{AmbientGust, WindNoise};

use crate::host::world_ambient_effect::WorldAmbientEffect;
use crate::host::world_wind_blast::WorldWindBlast;
use crate::level::ambient::ambient_wind::AmbientWind;

/// An effect living five seconds whose blast rises to full strength over two, towards `+z`, and falls over two.
fn blasting(gust_factor: f32) -> WorldAmbientEffect {
  WorldAmbientEffect {
    particles: String::from("nature\\vortex_01"),
    life_time: Duration::from_secs(5),
    offset: [0.0; 3],
    wind_gust_factor: gust_factor,
    wind_blast: WorldWindBlast {
      strength: 1.0,
      longitude: 0.0,
      in_time: Duration::from_secs(2),
      out_time: Duration::from_secs(2),
    },
  }
}

/// One frame of the wind as the ambient effects take it at `seconds`, the effect started at `started` seconds and
/// stopped with its life.
fn frame(wind: &mut AmbientWind, seconds: f32, (effect, started): (&WorldAmbientEffect, f32)) -> AmbientGust {
  let time: Duration = Duration::from_secs_f32(seconds);

  wind.blow(time);

  if seconds == started {
    wind.start(effect, time);
  }

  wind.rise(time);

  if seconds >= started + effect.life_time.as_secs_f32() {
    wind.calm();
  }

  wind.fall(time);
  wind.get_gust()
}

#[test]
fn still_air_stands_at_half_strength() {
  let mut wind: AmbientWind = AmbientWind::default();

  for time in [0.1, 5.0, 60.0, 600.0] {
    wind.blow(Duration::from_secs_f32(time));
    assert_eq!(wind.get_gust(), AmbientGust::default());
  }
}

#[test]
fn gusts_wander_within_two_thirds_of_nothing() {
  let mut noise: WindNoise = WindNoise::new(7);
  let values: Vec<f32> = (1..2000)
    .map(|frame| noise.read(Duration::from_millis(frame * 50), 0.03))
    .collect();

  assert!(values.iter().all(|value| value.abs() <= 0.66666));
  assert!(values.iter().any(|value| (value - values[0]).abs() > 1e-3));
}

// A year in, a frame's step still moves every gust: the noise's own times stay within its period rather than grow past
// where a float holds a frame.
#[test]
fn gusts_keep_moving_a_year_on() {
  let mut noise: WindNoise = WindNoise::new(7);
  let year: Duration = Duration::from_secs(365 * 24 * 60 * 60);
  let values: Vec<f32> = (0..60)
    .map(|frame| noise.read(year + Duration::from_millis(frame * 16), 1.0))
    .collect();

  assert!(values.windows(2).all(|pair| pair[0] != pair[1]), "{values:?}");
}

#[test]
fn a_blast_rises_turns_falls_and_leaves_no_wind_behind() {
  let effect: WorldAmbientEffect = blasting(0.0);
  let mut wind: AmbientWind = AmbientWind::default();
  let mut at = |time: f32| frame(&mut wind, time, (&effect, 10.0));

  at(9.0);

  let started: AmbientGust = at(10.0);
  let rising: AmbientGust = at(11.0);
  let risen: AmbientGust = at(12.0);
  let blowing: AmbientGust = at(13.0);
  let falling: AmbientGust = at(16.0);
  let gone: AmbientGust = at(18.0);

  // From the half the still air stood at, turning from `+x` to the blast's `+z` the shorter way.
  assert!((started.strength - 0.5).abs() < 1e-6);
  assert!((rising.strength - 0.75).abs() < 1e-5);
  assert!(rising.direction.abs_diff_eq(Vec3::new(1.0, 0.0, 1.0).normalize(), 1e-5));
  assert!((risen.strength - 1.0).abs() < 1e-5);
  assert!(risen.direction.abs_diff_eq(Vec3::Z, 1e-5));
  // Risen, the noise takes the strength back; once the life is up it falls from there to nothing, and stays so.
  assert!((blowing.strength - 0.5).abs() < 1e-6);
  assert!((falling.strength - 0.25).abs() < 1e-5);
  assert_eq!(gone.strength, 0.0);
  assert_eq!(at(60.0).strength, 0.0);
}

#[test]
fn a_campfire_is_blown_along_the_blast_by_the_strength() {
  let gust: AmbientGust = AmbientGust {
    strength: 0.25,
    direction: Vec3::Z,
  };

  assert_eq!(gust.get_velocity(), Vec3::new(0.0, 0.0, 0.25));
}
