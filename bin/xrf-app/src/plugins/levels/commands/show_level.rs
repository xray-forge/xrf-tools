use std::sync::Arc;

use tauri::State;
use xrf_renderer::{RenderLevelSource, RenderViewportId};

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::shown_levels::ShownLevels;
use crate::plugins::levels::state::{LevelState, SelectedLevel};
use crate::plugins::render::texture_reloads::TextureReloads;
use crate::plugins::render::viewport_shows::ViewportShows;

/// Draw the open level in a viewport, its sectors and textures read by the renderer; no session draws none. Answers
/// the scene drawn, which the viewport's load reports name, or none.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "show_level"))]
#[tauri::command(rename = "show_level")]
pub fn levels_show_level(
  shows: State<'_, ViewportShows>,
  shown: State<'_, ShownLevels>,
  reloads: State<'_, TextureReloads>,
  levels: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  viewport: RenderViewportId,
  session_id: Option<SessionId>,
) -> TauriResult<Option<u32>> {
  let source: Option<Arc<dyn RenderLevelSource>> = match session_id {
    Some(session_id) => {
      let level: Arc<SessionSnapshot<SelectedLevel>> = levels.selected.require(session_id)?;

      reloads.watch(&level.roots);

      log::info!(
        "Showing level '{}' in native viewport {}",
        level.source.get_label(),
        viewport.0
      );

      Some(shown.get_source(level, &assets) as Arc<dyn RenderLevelSource>)
    }
    None => None,
  };

  let ticket: u64 = shows.ask(viewport);

  Ok(shows.show(viewport, ticket, source).map(|scene| scene.0))
}
