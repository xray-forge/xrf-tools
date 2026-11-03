use tauri::State;
use xrf_renderer::{RenderOverlay, RenderViewportId};

use crate::Renderer;

/// Set the helpers drawn over one viewport's frame: at once, or for a scene from the frame the viewport shows it, as a
/// load report names it, so helpers sized to a model never stand over the one it replaces.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "set_overlays")]
pub fn render_set_overlays(
  renderer: State<'_, Renderer>,
  viewport: RenderViewportId,
  overlays: Vec<RenderOverlay>,
  scene: Option<u32>,
) {
  renderer.set_overlays(viewport, overlays, scene);
}
