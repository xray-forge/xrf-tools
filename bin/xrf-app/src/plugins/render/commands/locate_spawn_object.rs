use std::sync::mpsc::Receiver;

use tauri::State;
use xrf_renderer::RenderViewportId;

use crate::core::types::TauriResult;
use crate::plugins::render::render_answer::{ANSWER_TIMEOUT, await_answer};
use crate::plugins::render::state::RenderState;

/// Find a spawned object's bounding sphere in a viewport's level, centre then radius in renderer space; none until
/// its model is drawn.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "locate_spawn_object"))]
#[tauri::command(rename = "locate_spawn_object")]
pub async fn render_locate_spawn_object(
  state: State<'_, RenderState>,
  viewport: RenderViewportId,
  object: u32,
) -> TauriResult<Option<[f32; 4]>> {
  let answer: Receiver<Option<[f32; 4]>> = state.renderer.locate_spawn_object(viewport, object);

  await_answer(
    answer,
    ANSWER_TIMEOUT,
    format!("Viewport {} answered no location", viewport.0),
  )
  .await
}
