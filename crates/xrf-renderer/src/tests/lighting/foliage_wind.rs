use xrf_math::glam::{Vec2, Vec4};

use crate::contract::feature::scene::render_foliage_settings::RenderFoliageSettings;
use crate::feature::scene::wind::foliage_wind_values::FoliageWindValues;
use crate::lighting::wind::foliage_motion::FoliageMotion;
use crate::lighting::wind::foliage_wind::FoliageWind;
use crate::lighting::wind::render_wind::{RENDER_STRONGEST_WIND, RenderWind};
use crate::tests::shader::wgsl_probe::run_wgsl_probe;
use crate::tests::support::test_time::frame_at;

/// A field in the strongest wind, stepped a tenth of a second at a time.
struct Field {
  wind: FoliageWind,
  settings: RenderFoliageSettings,
  at: f64,
}

impl Field {
  const STEP: f64 = 0.1;

  fn new(settings: RenderFoliageSettings) -> Self {
    Self {
      wind: FoliageWind::default(),
      settings,
      at: 0.0,
    }
  }

  fn step(&mut self, direction: f32, motion: FoliageMotion) -> FoliageWindValues {
    let from: f64 = self.at;

    self.at += Self::STEP;
    self.wind.advance(
      frame_at(self.at, Some(from)),
      &self.settings,
      RenderWind {
        direction,
        speed: RENDER_STRONGEST_WIND,
      },
      motion,
      0.0,
    )
  }
}

fn gusty() -> RenderFoliageSettings {
  RenderFoliageSettings {
    gusts: 1.0,
    gust_spacing: 4.0,
    ..Default::default()
  }
}

// At nothing the foliage's shaders skip the gusts whole, so a view that never asks for them draws as it did.
#[test]
fn foliage_wind_asks_for_no_gusts_by_default() {
  let values: FoliageWindValues = Field::new(RenderFoliageSettings::default()).step(0.0, FoliageMotion::Blowing);

  assert_eq!(values.gusts.x, 0.0);
  assert_eq!(values.previous_gusts.x, 0.0);
}

// The foliage's motion is the difference of its place now and a frame before, so each frame's last place of the gusts
// is the one the frame before drew. They travel at the wind's own speed, a half of their 4 m spacing in a tenth of a
// second, the smaller layer's cells of 0.45 of it 1.3 times as fast.
#[test]
fn foliage_wind_carries_the_gusts_last_travel_into_the_next_frame() {
  let mut field: Field = Field::new(gusty());
  let first: FoliageWindValues = field.step(0.0, FoliageMotion::Blowing);
  let second: FoliageWindValues = field.step(0.0, FoliageMotion::Blowing);

  assert_eq!(first.previous_gusts.z, 0.0);
  assert!((first.gusts.z - 0.5).abs() < 1e-5);
  assert!((first.gusts.w - 0.5 * 1.3 / 0.45).abs() < 1e-5);
  assert_eq!(second.previous_gusts, first.gusts);
  assert!((second.gusts.z - 1.0).abs() < 1e-5);
}

// A turning wind turns the gusts and leaves how far they came alone, and a still frame holds them.
#[test]
fn foliage_wind_travels_the_gusts_by_distance_whatever_the_wind_does() {
  let mut field: Field = Field::new(gusty());

  field.step(0.0, FoliageMotion::Blowing);

  let turned: FoliageWindValues = field.step(2.0, FoliageMotion::Blowing);
  let held: FoliageWindValues = field.step(2.0, FoliageMotion::Still);

  assert!((turned.gusts.z - 1.0).abs() < 1e-5);
  assert_eq!(held.gusts, turned.gusts);
  assert_eq!(held.previous_gusts, turned.gusts);
}

// However long the wind blows, both layers' moves stay within the repeat and step as finely as in the first minute.
#[test]
fn foliage_wind_keeps_the_gusts_within_their_repeat() {
  let repeat: f32 = FoliageWind::GUST_REPEAT as f32;
  let mut field: Field = Field::new(gusty());

  for _ in 0..6000 {
    let values: FoliageWindValues = field.step(0.0, FoliageMotion::Blowing);

    for (now, before, step) in [
      (values.gusts.z, values.previous_gusts.z, 0.5),
      (values.gusts.w, values.previous_gusts.w, 0.5 * 1.3 / 0.45),
    ] {
      assert!((0.0..repeat).contains(&now), "{now}");
      assert!(
        ((now - before).rem_euclid(repeat) - step).abs() < 1e-3,
        "{before} to {now}"
      );
    }
  }
}

// A gust moved by a whole repeat, either layer's, stands where it stood, so its move may be taken within one.
#[test]
fn a_gust_stands_the_same_a_repeat_on() {
  let places: [Vec2; 5] = [
    Vec2::new(0.4, 0.7),
    Vec2::new(-3.1, 8.9),
    Vec2::new(57.3, -41.2),
    Vec2::new(-412.6, 306.5),
    Vec2::new(733.1, 918.4),
  ];
  let repeat: f32 = FoliageWind::GUST_REPEAT as f32;
  let cases: Vec<(Vec4, Vec4)> = places
    .into_iter()
    .map(|place| (place.extend(3.25).extend(1019.5), Vec4::new(repeat, 0.0, 0.0, repeat)))
    .collect();
  let risen = |shift: &str| {
    format!(
      "foliage_gust(probe.first.xy, FoliageSetup(vec4<f32>(0.6, 0.8, 1.0, 1.0), vec4<f32>(), vec4<f32>(), \
       vec4<f32>(), vec4<f32>(1.0, 6.0, probe.first.zw + {shift})))"
    )
  };
  let Some(answered) = run_wgsl_probe(
    "common/foliage_wind",
    &format!(
      "vec4<f32>({}, {}, {}, 0.0)",
      risen("vec2<f32>(0.0)"),
      risen("probe.second.xy"),
      risen("probe.second.zw")
    ),
    &cases,
  ) else {
    return;
  };

  for found in &answered {
    assert!((found.x - found.y).abs() < 5e-3, "{found}");
    assert!((found.x - found.z).abs() < 5e-3, "{found}");
  }

  assert!(answered.iter().any(|found| (found.x - answered[0].x).abs() > 0.05));
}
