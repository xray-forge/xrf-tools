use tauri::State;
use xrf_app_core::TauriResult;
use xrf_renderer::RenderViewportId;

use crate::Renderer;

/// Find a spawned object's bounding sphere in a viewport's level, centre then radius in renderer space; none until
/// its model is drawn.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "locate_spawn_object")]
pub(crate) async fn render_locate_spawn_object(
  renderer: State<'_, Renderer>,
  viewport: RenderViewportId,
  object: u32,
) -> TauriResult<Option<[f32; 4]>> {
  renderer.locate_spawn_object(viewport, object).await
}
