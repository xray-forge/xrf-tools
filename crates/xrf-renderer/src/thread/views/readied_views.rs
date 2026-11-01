use crate::thread::views::readied_view::ReadiedView;
use crate::thread::views::scene_recorder::SceneRecorder;

/// The frames the viewports readied this frame, and the view recording each scene's own work.
pub struct ReadiedViews {
  pub frames: Vec<ReadiedView>,
  pub recorders: Vec<SceneRecorder>,
}
