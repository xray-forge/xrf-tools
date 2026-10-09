use std::collections::HashMap;

use crate::contract::render_viewport_id::RenderViewportId;
use crate::host::render_scene_feedback::RenderSceneFeedback;
use crate::host::render_scene_frame::RenderSceneFrame;
use crate::host::render_scene_id::RenderSceneId;
use crate::host::render_view_frame::RenderViewFrame;
use crate::host::render_view_input::RenderViewInput;

/// The world a renderer draws, run inline on the render thread before each frame: it advances what it simulates,
/// publishes what changed, and answers what the frame draws: each viewport's camera and environment, then each scene's
/// changes.
pub trait RenderWorld: Send {
  /// The scene a viewport asks to show now, which a frame's answer catches up with; none for nothing.
  fn get_asked_scene(&self, viewport: RenderViewportId) -> Option<RenderSceneId>;

  /// Moves a viewport's camera and weather on a frame, answering what it draws.
  fn advance_view(&mut self, viewport: RenderViewportId, input: RenderViewInput<'_>) -> RenderViewFrame;

  /// Moves every scene on a frame, after every viewport: streams it, poses it and plays its effects by its driving
  /// viewport. `feedback` is what each scene told of the last frame; a scene the world no longer answers is gone.
  fn advance_scenes(
    &mut self,
    feedback: HashMap<RenderSceneId, RenderSceneFeedback>,
  ) -> HashMap<RenderSceneId, RenderSceneFrame>;

  /// Streams every scene in again from the start: the renderer let what it held of them go with its GPU.
  fn restart_scenes(&mut self);
}
