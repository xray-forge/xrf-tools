use crate::contract::feature::tracing::render_reflection_mode::RenderReflectionMode;
use crate::contract::feature::tracing::render_reflection_settings::RenderReflectionSettings;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::feature::tracing::reflections::history::reflection_history::ReflectionHistory;
use crate::feature::tracing::reflections::trace::reflection_trace::ReflectionTrace;
use crate::feature::view::view_restart::ViewRestart;
use crate::frame::targets::view_targets::ViewTargets;
use crate::tests::support::test_context::create_test_context;

const SIZE: (u32, u32) = (64, 48);

fn enhanced_trace() -> ReflectionTrace {
  let mut options: RenderViewOptions = RenderViewOptions::default();

  options.features.reflections = RenderReflectionSettings {
    mode: RenderReflectionMode::Enhanced,
    ..RenderReflectionSettings::default()
  };

  ReflectionTrace::new(&options).expect("the enhanced reflections trace")
}

// A cut keeps the noise turning; only a motion restart resets it.
#[test]
fn a_cut_and_a_restart_leave_the_reflections_nothing_of_the_place_before() {
  let Some(context) = create_test_context("to hold the reflections' history") else {
    return;
  };
  let device: &wgpu::Device = &context.device;
  let mut history: ReflectionHistory = ReflectionHistory::new(device, enhanced_trace(), SIZE);
  let mut depth: wgpu::TextureView = ViewTargets::create_depth(device, SIZE);

  for _ in 0..3 {
    history.swap(&mut depth);
  }

  assert!(history.has_history() && history.is_source_valid);
  assert_eq!(history.frame, 3);

  history.restart(ViewRestart::Cut);

  assert!(
    !history.has_history(),
    "a cut reads no denoised frame of the place before"
  );
  assert!(
    !history.is_source_valid,
    "a cut reads no scene kept of the place before"
  );
  assert_eq!(history.frame, 3, "a cut turns the noise on");

  history.swap(&mut depth);
  history.restart(ViewRestart::Motion);

  assert!(!history.has_history() && !history.is_source_valid);
  assert_eq!(history.frame, 0, "a restart turns the noise from none");
}
