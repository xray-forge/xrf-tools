/// Why a view starts again what its units carry from frame to frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewRestart {
  /// Its motion starts over (`SceneView::restart_motion`): every clock, easing, noise turn and history from nothing, as
  /// at its first frame, so what it draws from then on follows from its frames since alone, as two bursts from one
  /// time must.
  Motion,
  /// Its camera cut away (`ResolveFeature::is_cut`): what a history holds of the place it left leads nowhere true, so
  /// none is read; its clocks and easings run on.
  Cut,
}
