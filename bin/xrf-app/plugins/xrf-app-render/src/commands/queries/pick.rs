use tauri::State;
use xrf_app_core::TauriResult;
use xrf_renderer::{RenderLevelHit, RenderViewportId};

use crate::{RenderPickRequest, Renderer};

/// Name what a viewport's level draws under a point, or nothing; glass is the nearest thing a click meets unless the
/// pick passes it by.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "pick")]
pub async fn render_pick(
  renderer: State<'_, Renderer>,
  viewport: RenderViewportId,
  request: RenderPickRequest,
) -> TauriResult<Option<RenderLevelHit>> {
  renderer.pick(viewport, request).await
}
