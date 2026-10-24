use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_render::TextureReloads;
use xrf_app_render::ViewportShows;
use xrf_renderer::{RenderLevelSource, RenderViewportId};

use crate::{SelectedVisual, VisualModels, VisualPoses, VisualRenderSource};

/// Draw the open model in a viewport, `detail` down its collapse chain, its textures read by the renderer; no session
/// draws none. Answers the scene drawn, which the viewport's load reports name, or none. Off the main thread, as every
/// show is, its ticket taken first so a model asked for later is never replaced by this one.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "show_model"))]
#[tauri::command(rename = "show_model")]
// Tauri injects each managed state as an argument of its own.
#[allow(clippy::too_many_arguments)]
pub async fn visuals_show_model(
  shows: State<'_, ViewportShows>,
  reloads: State<'_, TextureReloads>,
  models: State<'_, VisualModels>,
  poses: State<'_, VisualPoses>,
  assets: State<'_, AssetMountState>,
  viewport: RenderViewportId,
  session_id: Option<SessionId>,
  detail: f32,
) -> TauriResult<Option<u32>> {
  let ticket: u64 = shows.ask(viewport);
  let source: Option<Arc<dyn RenderLevelSource>> = match session_id {
    Some(session_id) => {
      let visual: Arc<SessionSnapshot<SelectedVisual>> = models.require(session_id)?;

      reloads.watch(&visual.description.roots);

      log::info!(
        "Showing model '{}' in native viewport {} at detail {detail}",
        visual.description.source.label(),
        viewport.0
      );

      Some(Arc::new(VisualRenderSource::new(
        visual,
        VisualPoses::clone(&poses),
        AssetMountState::clone(&assets),
        detail,
      )))
    }
    None => None,
  };

  Ok(shows.show(viewport, ticket, source).map(|scene| scene.0))
}
