use std::sync::Arc;

use tauri::State;
use xrf_renderer::{RenderLevelSource, RenderViewportId};

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::render_source::LevelRenderSource;
use crate::plugins::levels::state::{LevelState, SelectedLevel};
use crate::plugins::render::state::RenderState;

/// Draw the open level in a viewport, its sectors and textures read by the renderer; no session draws none.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "show_level"))]
#[tauri::command(rename = "show_level")]
pub fn render_show_level(
  state: State<'_, RenderState>,
  levels: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  viewport: RenderViewportId,
  session_id: Option<SessionId>,
) -> TauriResult {
  let source: Option<Arc<dyn RenderLevelSource>> = match session_id {
    Some(session_id) => {
      let level: Arc<SessionSnapshot<SelectedLevel>> = levels.selected.require(session_id)?;

      log::info!(
        "Showing level '{}' in native viewport {}",
        level.source.get_label(),
        viewport.0
      );

      Some(Arc::new(LevelRenderSource::new(level, AssetMountState::clone(&assets))))
    }
    None => None,
  };

  let ticket: u64 = state.ask_show(viewport);

  state.show(viewport, ticket, source);

  Ok(())
}
