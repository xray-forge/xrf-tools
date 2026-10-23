use std::sync::Arc;

use tauri::State;
use xrf_renderer::{RenderLevelSource, RenderViewportId};

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::state::{LevelState, SelectedLevel};
use crate::plugins::render::state::RenderState;

/// Draw the open level in a viewport, its sectors and textures read by the renderer; no session draws none. Answers
/// the scene drawn, which the viewport's load reports name, or none.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "show_level"))]
#[tauri::command(rename = "show_level")]
pub fn render_show_level(
  state: State<'_, RenderState>,
  levels: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  viewport: RenderViewportId,
  session_id: Option<SessionId>,
) -> TauriResult<Option<u32>> {
  let source: Option<Arc<dyn RenderLevelSource>> = match session_id {
    Some(session_id) => {
      let level: Arc<SessionSnapshot<SelectedLevel>> = levels.selected.require(session_id)?;

      state.watch_textures(&level.roots);

      log::info!(
        "Showing level '{}' in native viewport {}",
        level.source.get_label(),
        viewport.0
      );

      Some(state.get_level_source(level, &assets) as Arc<dyn RenderLevelSource>)
    }
    None => None,
  };

  let ticket: u64 = state.ask_show(viewport);

  Ok(state.show(viewport, ticket, source).map(|scene| scene.0))
}
