use std::time::{Duration, Instant};

use crate::contract::feature::lighting::model::render_light_model::RenderLightModel;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::feature::image::exposure::exposure_feature::ExposureFeature;
use crate::tests::support::test_context::create_test_context;

/// A restarted exposure takes its next frame's measure whole, under the engine's light and the physical alike, rather
/// than easing from the scale the frames before it left, and eases again from the frame after: two bursts from one time
/// start from the same exposure whatever came before them. Skipped, and says so, on a machine with no adapter at all.
#[test]
fn a_restarted_exposure_takes_its_next_measure_whole() {
  let Some(context) = create_test_context("to adapt with") else {
    return;
  };
  let (device, queue): (&wgpu::Device, &wgpu::Queue) = (&context.device, &context.queue);
  let engine: RenderViewOptions = RenderViewOptions::default();
  let mut physical: RenderViewOptions = RenderViewOptions::default();
  let start: Instant = Instant::now();
  let frame = |index: u32| start + Duration::from_millis(u64::from(index) * 16);

  physical.features.light_model = RenderLightModel::Physical;

  let mut exposure: ExposureFeature = ExposureFeature::new(device, queue);

  for index in 0..4 {
    exposure.prepare(queue, &engine, frame(index));
  }

  assert!(exposure.params.blend < 1.0, "an adapting exposure eases");

  exposure.restart();
  exposure.prepare(queue, &engine, frame(4));

  assert_eq!(
    exposure.params.blend, 1.0,
    "the engine's exposure takes the measure whole"
  );

  exposure.prepare(queue, &engine, frame(5));

  assert!(exposure.params.blend < 1.0, "and eases again after");

  for index in 6..10 {
    exposure.prepare(queue, &physical, frame(index));
  }

  assert_eq!(exposure.get_histogram().snap, 0.0, "the physical exposure eases");

  exposure.restart();
  exposure.prepare(queue, &physical, frame(10));

  assert_eq!(
    exposure.get_histogram().snap,
    1.0,
    "the physical exposure takes the measure whole"
  );

  exposure.prepare(queue, &physical, frame(11));

  assert_eq!(exposure.get_histogram().snap, 0.0, "and eases again after");
}
