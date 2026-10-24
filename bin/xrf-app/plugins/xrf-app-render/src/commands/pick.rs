use std::sync::mpsc::Receiver;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_error::XrfResult;
use xrf_renderer::{RenderLevelHit, RenderPick, RenderViewportId};

use crate::request::RenderPickRequest;
use crate::{RenderAnswer, Renderer};

/// Name what a viewport's level draws under a point, or nothing; glass is the nearest thing a click meets unless the
/// pick passes it by.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "pick"))]
#[tauri::command(rename = "pick")]
pub async fn render_pick(
  renderer: State<'_, Renderer>,
  viewport: RenderViewportId,
  request: RenderPickRequest,
) -> TauriResult<Option<RenderLevelHit>> {
  let RenderPickRequest { x, y, is_through } = request;
  let answer: Receiver<XrfResult<RenderPick>> = renderer.pick(viewport, x, y, is_through);

  RenderAnswer::await_viewport(answer, viewport, "pick")
    .await?
    .map(|pick| pick.hit)
    .map_err(|error| error.to_string())
}
