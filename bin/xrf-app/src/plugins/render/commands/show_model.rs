use std::sync::Arc;

use tauri::State;
use xrf_renderer::{RenderLevelSource, RenderViewportId};

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::render::state::RenderState;
use crate::plugins::visuals::render_source::VisualRenderSource;
use crate::plugins::visuals::state::{SelectedVisual, VisualState};

/// Draw the open model in a viewport, `detail` down its collapse chain, its textures read by the renderer; no session
/// draws none.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "show_model"))]
#[tauri::command(rename = "show_model")]
pub fn render_show_model(
  state: State<'_, RenderState>,
  visuals: State<'_, VisualState>,
  assets: State<'_, AssetMountState>,
  viewport: RenderViewportId,
  session_id: Option<SessionId>,
  detail: f32,
) -> TauriResult {
  let source: Option<Arc<dyn RenderLevelSource>> = match session_id {
    Some(session_id) => {
      let visual: Arc<SessionSnapshot<SelectedVisual>> = visuals.selected.require(session_id)?;

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

  state.renderer.show_level(viewport, source);

  Ok(())
}
