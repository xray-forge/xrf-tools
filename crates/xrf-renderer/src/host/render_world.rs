use crate::contract::render_viewport_id::RenderViewportId;
use crate::host::render_world_frame::RenderWorldFrame;
use crate::host::render_world_input::RenderWorldInput;

/// The world a renderer draws, run inline on the render thread before each viewport's frame: it advances what it
/// simulates for the viewport, publishes what changed, and answers what the frame draws.
pub trait RenderWorld: Send {
  fn advance(&mut self, viewport: RenderViewportId, input: RenderWorldInput<'_>) -> RenderWorldFrame;
}
