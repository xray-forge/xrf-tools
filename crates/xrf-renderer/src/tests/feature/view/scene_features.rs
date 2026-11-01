use crate::feature::view::scene_features::SceneFeatures;
use crate::feature::view::view_feature::ViewFeature;
use crate::feature::view::view_stages::ViewStages;
use crate::pass::view::view_binding::ViewBinding;
use crate::tests::support::test_context::create_test_context;

/// A view's feature units hold nothing on the GPU before they draw and nothing once let go of, but for the state made
/// with the view whatever the settings: the exposure's, which every lit pass reads its scale from, the lights' records
/// and clusters, and the particles' first quads. Skipped, and says so, on a machine with no adapter at all.
#[test]
fn holds_nothing_while_off() {
  let Some(context) = create_test_context("to draw with") else {
    return;
  };
  let layout: wgpu::BindGroupLayout = ViewBinding::create_layout(&context.device);
  let mut features: SceneFeatures = SceneFeatures::new(&context.device, &context.queue, &layout);
  let mut stages: ViewStages = ViewStages::new();
  let kept: u64 = [
    &features.exposure as &dyn ViewFeature,
    &features.lights,
    &features.particles,
  ]
  .iter()
  .map(|feature| feature.get_memory())
  .sum();
  let get_staged = |stages: &ViewStages| -> u64 { stages.list().map(ViewFeature::get_memory).sum() };

  assert!(
    features.exposure.get_memory() > 0,
    "the exposure's state is made with the view"
  );
  assert_eq!(features.get_memory(), kept, "a unit holds memory before it draws");
  assert_eq!(get_staged(&stages), 0, "a stage's unit holds memory before it draws");

  for feature in features.list_mut().into_iter().chain(stages.list_mut()) {
    feature.release();
  }

  assert_eq!(features.get_memory(), kept, "a unit holds memory once let go of");
  assert_eq!(get_staged(&stages), 0, "a stage's unit holds memory once let go of");
}
