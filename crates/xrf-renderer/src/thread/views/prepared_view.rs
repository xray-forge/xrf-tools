use crate::contract::view::render_view_options::RenderViewOptions;
use crate::contract::viewport::render_rect::RenderRect;
use crate::contract::viewport::render_viewport_id::RenderViewportId;
use crate::host::world::render_scene_id::RenderSceneId;
use crate::scene::level::view::scene_asks::SceneAsks;
use crate::thread::render_thread::ViewFrame;

/// A viewport's view of a scene readied this frame, waiting for its scene to settle every view's asks: the scene, what
/// it asked of it, the options and the rectangle its frame was prepared with, which it finishes with, and its graph's
/// frame, none where it readied none.
pub struct PreparedView {
  pub id: RenderViewportId,
  pub scene: RenderSceneId,
  pub asks: SceneAsks,
  pub options: RenderViewOptions,
  pub output: RenderRect,
  pub frame: Option<ViewFrame>,
}
