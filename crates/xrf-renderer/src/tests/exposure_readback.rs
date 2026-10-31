use std::time::Instant;

use crate::contract::render_light_model::RenderLightModel;
use crate::contract::render_view_options::RenderViewOptions;
use crate::pass::exposure_head::ExposureHead;
use crate::scene::level::feature::exposure_feature::ExposureFeature;
use crate::tests::test_context::create_test_context;

/// Puts a head metering `adapted` into the exposure's state and copies it out as a frame does, asking for it back.
fn copy_out(device: &wgpu::Device, queue: &wgpu::Queue, exposure: &ExposureFeature, adapted: f32) {
  let head: ExposureHead = ExposureHead {
    adapted,
    used: adapted,
    key: -2.0,
    bright: 1.0,
  };
  let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

  queue.write_buffer(&exposure.state, 0, bytemuck::bytes_of(&head));
  exposure.record_readback(&mut encoder, &exposure.state);
  queue.submit([encoder.finish()]);
  exposure.request_readback();
}

/// A change of light model starts the physical exposure afresh: a head copied out under the other model, back by the
/// switch or still on its way then, never sets the scale the light is written at or what the pictures are shown by,
/// while a head copied out since is taken as before. Skipped, and says so, on a machine with no adapter at all.
#[test]
fn a_light_model_switch_drops_what_was_metered_before_it() {
  let Some(context) = create_test_context("to read back with") else {
    return;
  };
  let (device, queue): (&wgpu::Device, &wgpu::Queue) = (&context.device, &context.queue);
  let wait = || device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
  let engine: RenderViewOptions = RenderViewOptions::default();
  let mut physical: RenderViewOptions = RenderViewOptions::default();

  physical.features.light_model = RenderLightModel::Physical;

  let now: Instant = Instant::now();
  let mut exposure: ExposureFeature = ExposureFeature::new(device, queue);

  exposure.prepare(queue, &physical, now);

  let start: f32 = exposure.get_pre_exposure().current;
  let metered: f32 = start * 4096.0;

  // Back by the switch to the engine's light.
  copy_out(device, queue, &exposure, metered);
  wait();
  exposure.prepare(queue, &engine, now);
  exposure.prepare(queue, &physical, now);

  assert_eq!(exposure.get_pre_exposure().current, start);

  // Still on its way across both switches, back only after them.
  copy_out(device, queue, &exposure, metered);
  exposure.prepare(queue, &engine, now);
  exposure.prepare(queue, &physical, now);
  wait();
  exposure.prepare(queue, &physical, now);

  assert_eq!(exposure.get_pre_exposure().current, start);
  assert_eq!(exposure.get_metering().land, None);

  // Copied out under the physical light since.
  copy_out(device, queue, &exposure, metered);
  wait();
  exposure.prepare(queue, &physical, now);

  assert_eq!(exposure.get_pre_exposure().current, metered);
  assert_eq!(exposure.get_metering().bright, Some(2.0));
}
