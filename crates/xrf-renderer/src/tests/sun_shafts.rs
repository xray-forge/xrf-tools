use xrf_engine_target::XrayEngine;

use crate::contract::render_sun_shafts::RenderSunShafts;
use crate::lighting::render_lighting::RenderLighting;

// Monolith's high quality by default, the steps held to their range.
#[test]
fn steps_as_asked_within_their_range() {
  let steps = |steps: u32| {
    RenderSunShafts {
      steps,
      ..RenderSunShafts::default()
    }
    .get_steps()
  };

  assert_eq!(RenderSunShafts::default().get_steps(), 30);
  assert_eq!((steps(0), steps(40), steps(1000)), (8, 40, 64));
}

// Monolith's `r2_sunshafts_min`, held to its console's bounds, lifts the mixed density on any engine; zero leaves it.
#[test]
fn lifts_the_density_by_its_floor() {
  let shafts: RenderSunShafts = RenderSunShafts {
    steps: RenderSunShafts::STEPS,
    minimum: 0.2,
  };
  let lit = |engine: XrayEngine, density: f32| RenderLighting {
    engine,
    sun_shafts: density,
    ..RenderLighting::default()
  };

  let none: RenderSunShafts = RenderSunShafts { minimum: 0.0, ..shafts };
  let past: RenderSunShafts = RenderSunShafts { minimum: 0.9, ..shafts };

  assert_eq!(lit(XrayEngine::Vanilla, 0.4).get_sun_shafts(&none), 0.4);
  assert!((lit(XrayEngine::Vanilla, 0.0).get_sun_shafts(&shafts) - 0.2).abs() < 1e-6);
  assert!((lit(XrayEngine::Extended, 0.5).get_sun_shafts(&shafts) - 0.6).abs() < 1e-6);
  assert!((lit(XrayEngine::Extended, 0.0).get_sun_shafts(&past) - 0.5).abs() < 1e-6);
}
