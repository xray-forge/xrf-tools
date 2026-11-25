use xrf_math::glam::{Vec2, Vec3, Vec4};

use crate::contract::feature::effects::render_rain_drops_mode::RenderRainDropsMode;
use crate::feature::effects::rain::rain_fall::RainFall;
use crate::feature::effects::rain::rain_frame::RainFrame;
use crate::feature::effects::rain::rain_uniform::RainUniform;
use crate::feature::effects::rain::rings::rain_rings::RainRings;
use crate::feature::scene::wind::wind_uniform::WindUniform;
use crate::lighting::weather::render_fog::RenderFog;
use crate::lighting::weather::render_rainfall::RenderRainfall;
use crate::lighting::wind::ambient_gust::AmbientGust;
use crate::lighting::wind::render_tree_wind::RenderTreeWind;
use crate::lighting::wind::render_wind::{RENDER_STRONGEST_WIND, RenderWind};
use crate::scene::level::level_time::LevelTime;
use crate::scene::level::shadow::shadow_sway::ShadowSway;
use crate::tests::support::test_time::at;

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
fn draws_the_engine_s_streaks_and_the_enhanced_drops_each_by_its_look() {
  let rain = |drops: RenderRainDropsMode, density: f32| {
    falling_rain(Falling {
      drops,
      density,
      ..Falling::default()
    })
  };
  let streaks: RainUniform = rain(RenderRainDropsMode::Engine, 1.0);
  let drops: RainUniform = rain(RenderRainDropsMode::Enhanced, 1.0);

  // A hundredth of them at the lightest rain; all 2,500 streaks at the heaviest, all 6,000 drops.
  assert_eq!(rain(RenderRainDropsMode::Engine, 0.0).count, 25);
  assert_eq!(streaks.count, RainUniform::STREAKS);
  assert_eq!(rain(RenderRainDropsMode::Enhanced, 0.0).count, 60);
  assert_eq!(drops.count, 6000);
  // A streak is five metres long, falling 40 to 80 metres a second; a drop 0.55 metres long and a fifth of a metre
  // wide, falling 24 to 48.
  assert_eq!(streaks.is_enhanced, 0);
  assert_eq!(streaks.streak, Vec4::new(5.0, 0.3, 40.0, 80.0));
  assert_eq!(drops.is_enhanced, 1);
  assert!(drops.streak.distance(Vec4::new(0.55, 0.1, 24.0, 48.0)) < 1e-4);
  // Both fall the one way.
  assert_eq!(streaks.axis, drops.axis);
  assert_eq!(streaks.fall, drops.fall);
  assert_eq!(streaks.aim, drops.aim);
  assert_eq!(streaks.pace, drops.pace);
  assert_eq!(streaks.near, drops.near);
}

#[test]
fn falls_the_rain_as_monolith_leans_scatters_and_slows_it() {
  let rain = |strength: f32, gust: f32| {
    falling_rain(Falling {
      strength,
      gust,
      ..Falling::default()
    })
  };
  let still: RainUniform = rain(0.0, 0.0);

  // From twenty metres up over a square fifteen metres a side, five metres ahead of the camera.
  assert_eq!(still.fall.truncate(), Vec3::new(20.0, 30.0, 15.0));
  assert_eq!(still.aim.truncate().truncate(), AIM);
  // Still air falls straight down at half the pace, each drop scattered by up to fifteen degrees and a half more; a
  // wind past two thirds of Monolith's strongest falls at all of the pace, and its strongest leans them thirty five
  // degrees, scattered by half of fifteen.
  assert!(still.axis.truncate().distance(Vec3::NEG_Y) < 1e-6);
  assert!((still.axis.w - 22.5f32.to_radians()).abs() < 1e-6);
  assert_eq!(still.pace, 0.5);
  assert_eq!(rain(0.7, 0.0).pace, 1.0);
  assert!((rain(1.0, 0.0).axis.y + 35f32.to_radians().cos()).abs() < 1e-5);
  assert!((rain(1.0, 0.0).axis.w - 7.5f32.to_radians()).abs() < 1e-6);
  // The rain's own gust leans still air, and takes a wind's lean off.
  assert!((rain(0.0, 0.5).axis.y + 17.5f32.to_radians().cos()).abs() < 1e-5);
  assert!(rain(0.5, -0.5).axis.truncate().distance(Vec3::NEG_Y) < 1e-6);
}

#[test]
fn leans_the_rain_against_monolith_s_negated_wind_direction() {
  let lean: Vec4 = falling_rain(Falling {
    strength: 1.0,
    direction: 90f32.to_radians(),
    ..Falling::default()
  })
  .axis;

  // `setHP` of a quarter turn leans along `-x`; negated, along `+x`.
  assert!((lean.x - 35f32.to_radians().sin()).abs() < 1e-5);
  assert!(lean.z.abs() < 1e-6);
}

#[test]
fn aims_monolith_s_drops_ahead_and_follows_a_turn_over_about_a_second() {
  let ahead: Vec2 = RainFall::ease_aim(None, Vec3::NEG_Z, 0.0);

  // Five metres along the camera's way at once, its height dropped.
  assert!(ahead.distance(AIM) < 1e-6);
  assert!(RainFall::ease_aim(None, Vec3::NEG_Y, 0.0).length() < 1e-6);
  // Turned round, a frame moves it a little, a second most of the way.
  assert!(RainFall::ease_aim(Some(ahead), Vec3::Z, 1.0 / 60.0).distance(ahead) < 0.2);
  assert!(RainFall::ease_aim(Some(ahead), Vec3::Z, 1.0).y > 1.0);
}

#[test]
fn rings_the_enhanced_drops_tile_on_the_water_and_nothing_for_the_engine_streaks() {
  let water: Vec4 = Vec4::new(0.0, 0.0, 128.0, 50.0);
  let falling = |drops: RenderRainDropsMode| {
    falling_rain(Falling {
      drops,
      ..Falling::default()
    })
  };
  let on_water = |drops: RenderRainDropsMode| RainUniform {
    water,
    ..falling(drops)
  };

  // The square the drops' columns repeat over, which the rings wrap over.
  assert_eq!(on_water(RenderRainDropsMode::Enhanced).get_rings(), 15.0);
  // None without the level water's map to land on, nor for the engine's streaks.
  assert_eq!(falling(RenderRainDropsMode::Enhanced).get_rings(), 0.0);
  assert_eq!(on_water(RenderRainDropsMode::Engine).get_rings(), 0.0);
}

#[test]
fn falls_the_enhanced_drops_over_a_wider_square_thinning_out_past_monolith_s() {
  let rain: RainUniform = falling_rain(Falling {
    drops: RenderRainDropsMode::Enhanced,
    ..Falling::default()
  });

  // Monolith's square falls them all; a narrower one fewer by its area, every drop near; a wider one thins out past
  // Monolith's in rings of far cells, ten times as wide falling under seven times the drops; held within five and
  // 150 metres.
  assert_eq!(rain.squared(15.0).count, 6000);
  assert_eq!(rain.squared(7.5).count, 1500);
  assert_eq!(rain.squared(7.5).near, 7.5);
  // The centre's cell as full as the near square, then five rings of far cells, 1,500 drops a cell in the first, 60
  // in the fifth.
  assert_eq!(
    rain.squared(150.0).count,
    6000 + 6000 + 12_000 + 6000 + 4008 + 3008 + 2400
  );
  assert_eq!(rain.squared(150.0).near_count, 6000);
  assert_eq!(rain.squared(150.0).near, 15.0);
  assert_eq!(rain.squared(500.0).fall.z, 150.0);
  assert_eq!(falling_rain(Falling::default()).squared(30.0).count, 2500);
  // The rings' tile two centimetres a texel, in whole reduction blocks, never under the coarsest level's block.
  assert_eq!(RainRings::get_size(15.0), 752);
  assert_eq!(RainRings::get_size(5.0), 512);
}

/// Where the camera's way aims Monolith's drops in these tests, in renderer `x` and `z`.
const AIM: Vec2 = Vec2::new(0.0, -5.0);

/// A rain to fall: the heaviest in still air, drawn as the engine's streaks, unless said otherwise.
struct Falling {
  drops: RenderRainDropsMode,
  density: f32,
  /// The share of its strongest the wind blows at, and its direction.
  strength: f32,
  direction: f32,
  /// The gust of Monolith's rain.
  gust: f32,
}

impl Default for Falling {
  fn default() -> Self {
    Self {
      drops: RenderRainDropsMode::Engine,
      density: 1.0,
      strength: 0.0,
      direction: 0.0,
      gust: 0.0,
    }
  }
}

fn falling_rain(falling: Falling) -> RainUniform {
  RainUniform::new(RainFrame {
    rain: &RenderRainfall {
      color: Vec3::ONE,
      density: falling.density,
    },
    drops: falling.drops,
    wind: RenderWind {
      direction: falling.direction,
      speed: falling.strength * RENDER_STRONGEST_WIND,
    },
    gust: AmbientGust {
      rain: falling.gust,
      ..AmbientGust::default()
    },
    window: Vec4::ZERO,
    cover_shape: Vec4::ZERO,
    aim: AIM,
    splash_indices: 0,
  })
}

#[test]
fn sways_the_trees_by_the_wind_turning_and_stills_them_without_one() {
  let trees: RenderTreeWind = RenderTreeWind {
    amplitude: 0.01,
    ..Default::default()
  };
  let wind: WindUniform = WindUniform::new(Some(&trees), at(0.0));

  assert!(wind.wind.truncate().distance(Vec3::new(0.0, 0.0, -0.01)) < 1e-7);
  assert!(wind.is_swaying());
  // A quarter of its turn later it leans across.
  assert!(
    WindUniform::new(Some(&trees), at(2.5))
      .wind
      .truncate()
      .distance(Vec3::new(0.01, 0.0, 0.0))
      < 1e-6
  );
  assert!(!WindUniform::new(None, at(1.0)).is_swaying());
  assert!((wind.get_amplitude() - 0.01).abs() < 1e-7);
}

#[test]
fn redraws_a_shadow_for_the_sway_where_the_lean_shows_once_an_interval() {
  let sway = |amplitude: f32, time: f32| ShadowSway {
    amplitude,
    reach: 20.0,
    time: at(f64::from(time)),
    places: &[],
  };

  // Twenty metres of reach at a hundredth's amplitude lean a fifth of a metre: shown at a decimetre's texels, not a
  // metre's.
  assert!(sway(0.01, 1.0).is_redrawn(20.0, 0.1, at(0.0)));
  assert!(!sway(0.01, 1.0).is_redrawn(20.0, 1.0, at(0.0)));
  // Still trees never draw a map again.
  assert!(!sway(0.0, 1.0).is_redrawn(20.0, 0.01, at(0.0)));
  // Within an interval it waits; a hair short of one, it draws.
  assert!(!sway(0.01, 1.0 + ShadowSway::INTERVAL * 0.5).is_redrawn(20.0, 0.1, at(1.0)));
  assert!(sway(0.01, 1.0 + ShadowSway::INTERVAL * 0.95).is_redrawn(20.0, 0.1, at(1.0)));
}

// A burst from an earlier time leaves maps drawn ahead of the clock; a saturated wait of zero would hold them.
#[test]
fn redraws_a_shadow_drawn_after_the_clock_moved_back() {
  let sway = ShadowSway {
    amplitude: 0.01,
    reach: 20.0,
    time: at(1.0),
    places: &[],
  };
  let ahead: LevelTime = at(600.0);

  assert!(sway.is_redrawn(20.0, 0.1, ahead));
  assert!(!sway.can_wait(ahead, ShadowSway::INTERVAL * 4.0));
  assert!(!ShadowSway { amplitude: 0.0, ..sway }.is_redrawn(20.0, 0.1, ahead));
  assert!(sway.can_wait(at(1.0 - f64::from(ShadowSway::INTERVAL)), ShadowSway::INTERVAL * 4.0));
}

#[test]
fn carries_the_last_frames_sway_for_the_trees_motion() {
  let trees: RenderTreeWind = RenderTreeWind {
    amplitude: 0.01,
    ..Default::default()
  };
  let first: WindUniform = WindUniform::new(Some(&trees), at(0.0)).following(None);
  let second: WindUniform = WindUniform::new(Some(&trees), at(2.5)).following(Some(&first));

  assert_eq!(first.previous_wind, first.wind);
  assert_eq!(second.previous_wind, first.wind);
  assert_eq!(second.previous_wave, first.wave);
  assert_ne!(second.wind, first.wind);
}
