use std::time::{Duration, Instant};

use crate::contract::feature::lighting::model::render_light_level::RenderLightLevel;
use crate::contract::feature::lighting::model::render_light_model::RenderLightModel;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::feature::image::exposure::exposure_feature::ExposureFeature;
use crate::tests::support::test_context::create_test_context;

// A lightning strike reads as a flash: the physical levels' law takes no step while a bolt lights the frame, nor a
// moment after, then steps on; the authored levels keep the engine's own adaptation through it, as the engine does.
#[test]
fn a_strike_holds_the_physical_law_where_it_stands() {
  let Some(context) = create_test_context("to adapt with") else {
    return;
  };
  let (device, queue): (&wgpu::Device, &wgpu::Queue) = (&context.device, &context.queue);
  let mut physical: RenderViewOptions = RenderViewOptions::default();
  let start: Instant = Instant::now();
  let frame = |index: u64| start + Duration::from_millis(index * 16);

  physical.features.light_model = RenderLightModel::Physical;
  physical.features.light_level = RenderLightLevel::Physical;

  let mut exposure: ExposureFeature = ExposureFeature::new(device, queue);

  exposure.prepare(queue, &physical, false, frame(0));
  exposure.prepare(queue, &physical, false, frame(1));

  assert!(exposure.get_histogram().delta > 0.0, "the law steps between strikes");

  exposure.prepare(queue, &physical, true, frame(2));

  assert_eq!(exposure.get_histogram().delta, 0.0, "a strike holds it");

  // The strike's light is out; within the tail it still holds.
  exposure.prepare(queue, &physical, false, frame(10));

  assert_eq!(exposure.get_histogram().delta, 0.0, "and a moment after");

  exposure.prepare(queue, &physical, false, frame(30));

  assert!(exposure.get_histogram().delta > 0.0, "then it steps on");

  let mut authored: RenderViewOptions = physical.clone();

  authored.features.light_level = RenderLightLevel::Authored;
  exposure.prepare(queue, &authored, true, frame(31));

  assert!(
    exposure.get_histogram().engine_blend > 0.0,
    "the engine's adaptation moves through a strike"
  );
}
