use std::sync::Arc;

use tauri::State;
use xrf_renderer::{RenderLevelSource, RenderViewportId};

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::render::texture_reloads::TextureReloads;
use crate::plugins::render::viewport_shows::ViewportShows;
use crate::plugins::visuals::render_source::VisualRenderSource;
use crate::plugins::visuals::state::{SelectedVisual, VisualState};

/// Draw the open model in a viewport, `detail` down its collapse chain, its textures read by the renderer; no session
/// draws none. Answers the scene drawn, which the viewport's load reports name, or none.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "show_model"))]
#[tauri::command(rename = "show_model")]
pub fn visuals_show_model(
  shows: State<'_, ViewportShows>,
  reloads: State<'_, TextureReloads>,
  visuals: State<'_, VisualState>,
  assets: State<'_, AssetMountState>,
  viewport: RenderViewportId,
  session_id: Option<SessionId>,
  detail: f32,
) -> TauriResult<Option<u32>> {
  let source: Option<Arc<dyn RenderLevelSource>> = match session_id {
    Some(session_id) => {
      let visual: Arc<SessionSnapshot<SelectedVisual>> = visuals.models.require(session_id)?;

      reloads.watch(&visual.roots);

      log::info!(
        "Showing model '{}' in native viewport {} at detail {detail}",
        visual.source.label(),
        viewport.0
      );

      Some(Arc::new(VisualRenderSource::new(
        visual,
        AssetMountState::clone(&assets),
        detail,
      )))
    }
    None => None,
  };

  let ticket: u64 = shows.ask(viewport);

  Ok(shows.show(viewport, ticket, source).map(|scene| scene.0))
}
