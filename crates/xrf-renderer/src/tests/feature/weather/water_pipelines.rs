use crate::feature::weather::water::draw::water_pipelines::WaterPipelines;
use crate::frame::targets::view_targets::ViewTargets;

// Bytes per sample as WebGPU counts them: each target aligned to its component, in attachment order.
#[test]
fn checks_the_device_for_the_bytes_the_guided_water_takes() {
  let formats = [ViewTargets::SCENE, ViewTargets::DISTORTION, ViewTargets::MOTION]
    .into_iter()
    .chain(WaterPipelines::GUIDES);
  let bytes: u32 = formats.fold(0, |bytes, format| {
    let alignment: u32 = format.target_component_alignment().unwrap();

    bytes.next_multiple_of(alignment) + format.target_pixel_byte_cost().unwrap()
  });

  assert_eq!(bytes, WaterPipelines::GUIDED_BYTES);
}
