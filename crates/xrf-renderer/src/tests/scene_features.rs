use crate::context::gpu_context::GpuContext;
use crate::contract::render_backend::RenderBackend;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::scene_features::SceneFeatures;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::tests::test_bundle::checkout_dxc;

/// A view's feature units hold nothing on the GPU before they draw and nothing once let go of; the exposure's state,
/// which every lit pass reads its scale from, is the one kept whatever the settings. Skipped, and says so, on a machine
/// with no adapter at all.
#[test]
fn holds_nothing_while_off() {
  let context: GpuContext = match GpuContext::create_headless(RenderBackend::D3d12, checkout_dxc().as_deref())
    .or_else(|_| GpuContext::create_headless(RenderBackend::Vulkan, None))
  {
    Ok(context) => context,
    Err(error) => {
      eprintln!("Skipped: no GPU to draw with ({error})");

      return;
    }
  };
  let layout: wgpu::BindGroupLayout = ViewBinding::create_layout(&context.device);
  let mut features: SceneFeatures = SceneFeatures::new((&context.device, &context.queue), &layout, 64);
  let kept: u64 = features.exposure.get_memory();

  assert!(kept > 0, "the exposure's state is made with the view");
  assert_eq!(features.get_memory(), kept, "a unit holds memory before it draws");

  for feature in features.list_mut() {
    feature.release();
  }

  assert_eq!(features.get_memory(), kept, "a unit holds memory once let go of");
}
