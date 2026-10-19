use glam::{Vec2, Vec3, Vec4};

use crate::contract::render_rain_drops_mode::RenderRainDropsMode;
use crate::lighting::render_fog::RenderFog;
use crate::lighting::render_rainfall::RenderRainfall;
use crate::lighting::render_tree_wind::RenderTreeWind;
use crate::lighting::render_wind::RenderWind;
use crate::pass::rain_uniform::RainUniform;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::level::shadow_sway::ShadowSway;

#[test]
fn fogs_by_the_engine_ramp_and_ends_the_view_where_it_is_total() {
  let fog: RenderFog = RenderFog {
    color: Vec3::ONE,
    density: 0.5,
    distance: 100.0,
    far_plane: 400.0,
  };
  let (offset, scale) = fog.get_params();
  let amount = |distance: f32| (distance * scale + offset).clamp(0.0, 1.0);

  // From `(1 - density) * 0.85 * distance` to `0.99 * distance`.
  assert!(amount(42.5) < 1e-5);
  assert!((amount(99.0) - 1.0).abs() < 1e-5);
  assert_eq!(fog.get_total_distance(), 99.0);
  assert_eq!(RenderFog { far_plane: 50.0, ..fog }.get_total_distance(), 50.0);
}

#[test]
fn rains_as_many_streaks_as_the_engine_and_leans_them_with_the_wind() {
  let rain = |density: f32, velocity: f32| falling_rain(RenderRainDropsMode::Engine, density, (velocity, 0.5));

  assert_eq!(RainUniform::get_count(0.0), RainUniform::STREAKS / 2);
  assert_eq!(RainUniform::get_count(1.0), RainUniform::STREAKS);
  assert!(rain(1.0, 0.0).axis.truncate().distance(Vec3::NEG_Y) < 1e-6);
  // The strongest wind leans them ten degrees.
  assert!((rain(1.0, 400.0).axis.y + 10f32.to_radians().cos()).abs() < 1e-5);
  assert_eq!(rain(1.0, 0.0).is_enhanced, 0);
}

#[test]
fn rains_the_enhanced_drops_as_monolith_counts_slows_and_leans_them() {
  let rain = |density: f32, wind: (f32, f32)| falling_rain(RenderRainDropsMode::Enhanced, density, wind);

  // A hundredth of the drops at the lightest rain, all 6,000 at the heaviest.
  assert_eq!(RainUniform::get_enhanced_count(0.0), 60);
  assert_eq!(RainUniform::get_enhanced_count(1.0), 6000);
  assert_eq!(rain(1.0, (0.0, 0.5)).is_enhanced, 1);
  // A drop is 0.55 metres long and a fifth of a metre wide; still air halves its speed, a wind past two thirds of
  // Monolith's thousand gives it whole.
  assert!(rain(1.0, (0.0, 0.5)).streak.distance(Vec4::new(0.55, 0.1, 12.0, 24.0)) < 1e-4);
  assert!(
    rain(1.0, (700.0, 0.5))
      .streak
      .distance(Vec4::new(0.55, 0.1, 24.0, 48.0))
      < 1e-4
  );
  // Still air falls straight down, a gust leans it, and the strongest wind thirty five degrees.
  assert!(rain(1.0, (0.0, 0.5)).axis.truncate().distance(Vec3::NEG_Y) < 1e-6);
  assert!(rain(1.0, (0.0, 0.75)).axis.y > -1.0 + 1e-3);
  assert!((rain(1.0, (1000.0, 0.5)).axis.y + 35f32.to_radians().cos()).abs() < 1e-5);
}

#[test]
fn aims_the_enhanced_drops_ahead_and_follows_a_turn_over_about_a_second() {
  let ahead: Vec2 = RainUniform::ease_aim(None, Vec3::NEG_Z, 0.0);

  // Five metres along the camera's way at once, its height dropped.
  assert!(ahead.distance(Vec2::new(0.0, -5.0)) < 1e-6);
  assert!(RainUniform::ease_aim(None, Vec3::NEG_Y, 0.0).length() < 1e-6);
  // Turned round, a frame moves it a little, a second most of the way.
  assert!(RainUniform::ease_aim(Some(ahead), Vec3::Z, 1.0 / 60.0).distance(ahead) < 0.2);
  assert!(RainUniform::ease_aim(Some(ahead), Vec3::Z, 1.0).y > 1.0);
}

fn falling_rain(drops: RenderRainDropsMode, density: f32, (velocity, strength): (f32, f32)) -> RainUniform {
  RainUniform::new(
    (
      &RenderRainfall {
        color: Vec3::ONE,
        density,
      },
      drops,
    ),
    (
      RenderWind {
        direction: 0.0,
        velocity,
      },
      strength,
    ),
    (Vec4::ZERO, Vec4::ZERO),
    (0.0, Vec2::ZERO),
    0,
  )
}

#[test]
fn sways_the_trees_by_the_wind_turning_and_stills_them_without_one() {
  let trees: RenderTreeWind = RenderTreeWind {
    amplitude: 0.01,
    ..Default::default()
  };
  let wind: WindUniform = WindUniform::new(Some(&trees), 0.0);

  assert!(wind.wind.truncate().distance(Vec3::new(0.0, 0.0, -0.01)) < 1e-7);
  assert!(wind.is_swaying());
  // A quarter of its turn later it leans across.
  assert!(
    WindUniform::new(Some(&trees), 2.5)
      .wind
      .truncate()
      .distance(Vec3::new(0.01, 0.0, 0.0))
      < 1e-6
  );
  assert!(!WindUniform::new(None, 1.0).is_swaying());
  assert!((wind.get_amplitude() - 0.01).abs() < 1e-7);
}

#[test]
fn redraws_a_shadow_for_the_sway_where_the_lean_shows_once_an_interval() {
  let sway = |amplitude: f32, time: f32| ShadowSway {
    amplitude,
    reach: 20.0,
    time,
    places: &[],
  };

  // Twenty metres of reach at a hundredth's amplitude lean a fifth of a metre: shown at a decimetre's texels, not a
  // metre's.
  assert!(sway(0.01, 1.0).is_redrawn(20.0, 0.1, 0.0));
  assert!(!sway(0.01, 1.0).is_redrawn(20.0, 1.0, 0.0));
  // Still trees never draw a map again.
  assert!(!sway(0.0, 1.0).is_redrawn(20.0, 0.01, 0.0));
  // Within an interval it waits; a hair short of one, it draws.
  assert!(!sway(0.01, 1.0 + ShadowSway::INTERVAL * 0.5).is_redrawn(20.0, 0.1, 1.0));
  assert!(sway(0.01, 1.0 + ShadowSway::INTERVAL * 0.95).is_redrawn(20.0, 0.1, 1.0));
}

#[test]
fn carries_the_last_frames_sway_for_the_trees_motion() {
  let trees: RenderTreeWind = RenderTreeWind {
    amplitude: 0.01,
    ..Default::default()
  };
  let first: WindUniform = WindUniform::new(Some(&trees), 0.0).following(None);
  let second: WindUniform = WindUniform::new(Some(&trees), 2.5).following(Some(&first));

  assert_eq!(first.previous_wind, first.wind);
  assert_eq!(second.previous_wind, first.wind);
  assert_eq!(second.previous_wave, first.wave);
  assert_ne!(second.wind, first.wind);
}
