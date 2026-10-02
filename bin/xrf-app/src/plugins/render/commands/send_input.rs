use tauri::State;
use xrf_renderer::{RenderInputEvent, RenderViewportId};

use crate::plugins::render::state::RenderState;

/// Hand a viewport one gesture the page heard over it.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "send_input"))]
#[tauri::command(rename = "send_input")]
pub fn render_send_input(state: State<'_, RenderState>, viewport: RenderViewportId, event: RenderInputEvent) {
  state.renderer.send_input(viewport, event);
}
