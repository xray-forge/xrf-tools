/// Why a view starts again what its units carry from frame to frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ViewRestart {
  /// Its motion starts over (`SceneView::restart_motion`): every clock, easing, noise turn and history from nothing,
  /// as at its first frame, so two bursts from one time draw alike.
  Motion,
  /// Its camera cut away (`ResolveFeature::is_cut`): no history is read; clocks and easings run on.
  Cut,
}
