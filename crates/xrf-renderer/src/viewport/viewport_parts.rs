use crate::capture::capture_job::CaptureJob;
use crate::contract::overlay::render_selection::RenderSelection;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::host::world::render_scene_id::RenderSceneId;
use crate::host::world::view::render_view_frame::RenderViewFrame;
use crate::pass::view::view_binding::ViewBinding;
use crate::scene::level::view::scene_view::SceneView;
use crate::viewport::viewport_overlays::ViewportOverlays;

/// A viewport taken apart for readying its frame (`RenderViewport::split`): what it is drawn with, read-only, and its
/// own view, its camera's binding and its capture, which readying writes.
pub(crate) struct ViewportParts<'a> {
  pub options: &'a RenderViewOptions,
  /// What its world gave it this frame.
  pub world: &'a RenderViewFrame,
  pub overlays: &'a ViewportOverlays,
  pub selection: Option<&'a RenderSelection>,
  /// The scene it shows and its own view of it, none where it shows none.
  pub scene: Option<RenderSceneId>,
  pub view: Option<&'a mut SceneView>,
  /// Its camera on the GPU, made by the first frame that needs it.
  pub binding: &'a mut Option<ViewBinding>,
  /// The capture it draws in place of its own frames, while one runs.
  pub capture: Option<&'a mut CaptureJob>,
}
