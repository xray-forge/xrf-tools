use std::sync::mpsc::Receiver;
use std::time::Duration;

use tauri::State;
use xrf_renderer::RenderViewportId;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::render::state::RenderState;

/// How long a location waits for the render thread to take it up.
const LOCATE_TIMEOUT: Duration = Duration::from_secs(2);

/// Find a spawned object's bounding sphere in a viewport's level, centre then radius in renderer space; none until
/// its model is drawn.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "locate_spawn_object"))]
#[tauri::command(rename = "locate_spawn_object")]
pub async fn render_locate_spawn_object(
  state: State<'_, RenderState>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
  object: u32,
) -> TauriResult<Option<[f32; 4]>> {
  let answer: Receiver<Option<[f32; 4]>> = state.renderer.locate_spawn_object(viewport, object);

  execution
    .run_blocking("Locating a spawned object", move || -> TauriResult<Option<[f32; 4]>> {
      answer
        .recv_timeout(LOCATE_TIMEOUT)
        .map_err(|_| format!("Viewport {} answered no location", viewport.0))
    })
    .await?
}
