use crate::contract::viewport::render_viewport_id::RenderViewportId;
use crate::host::world::render_scene_id::RenderSceneId;

/// A scene this frame draws and the viewport recording its own work, the first of its views to ready a frame: the
/// lights' faces, offered under it and readied from its frame.
#[derive(Clone, Copy)]
pub struct SceneRecorder {
  pub scene: RenderSceneId,
  pub viewport: RenderViewportId,
}
