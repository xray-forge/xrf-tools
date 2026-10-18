use xrf_engine_target::XrayEngine;

use crate::contract::render_sun_shafts::RenderSunShafts;
use crate::contract::render_sun_shafts_quality::RenderSunShaftsQuality;
use crate::lighting::render_lighting::RenderLighting;

#[test]
fn steps_each_quality_as_its_engine_does() {
  let steps = |engine: XrayEngine| {
    [
      RenderSunShaftsQuality::Low,
      RenderSunShaftsQuality::Medium,
      RenderSunShaftsQuality::High,
    ]
    .map(|quality| quality.get_steps(engine))
  };

  assert_eq!(steps(XrayEngine::Vanilla), [20, 20, 40]);
  assert_eq!(steps(XrayEngine::Extended), [15, 25, 30]);
}

// Steps asked for stand in for the quality's, held to the most a ray takes; none asked steps as the quality does.
#[test]
fn steps_as_asked_over_the_quality() {
  let shafts = |steps: Option<u32>| RenderSunShafts {
    quality: RenderSunShaftsQuality::High,
    minimum: 0.0,
    steps,
  };

  assert_eq!(shafts(None).get_steps(XrayEngine::Vanilla), 40);
  assert_eq!(shafts(Some(12)).get_steps(XrayEngine::Vanilla), 12);
  assert_eq!(shafts(Some(0)).get_steps(XrayEngine::Extended), 1);
  assert_eq!(
    shafts(Some(500)).get_steps(XrayEngine::Extended),
    RenderSunShafts::MAX_STEPS
  );
}

// Monolith's `r2_sunshafts_min`, held to its console's bounds, lifts the mixed density on any engine; zero leaves it.
#[test]
fn lifts_the_density_by_its_floor() {
  let shafts: RenderSunShafts = RenderSunShafts {
    quality: RenderSunShaftsQuality::High,
    minimum: 0.2,
    steps: None,
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
