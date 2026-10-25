use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_render::{TextureReloads, ViewportShows};
use xrf_renderer::{RenderLevelSource, RenderViewportId};

use crate::level::{SelectedLevel, SelectedLevels};
use crate::render::{LevelRenderSource, ShownLevels};
use crate::report::LevelReport;

/// Draw the open level in a viewport, its sectors and textures read by the renderer; no session draws none. Answers
/// the scene drawn, which the viewport's load reports name, or none. The source may resolve the level's surfaces, so
/// it is made on the pool, after the ticket that keeps a level asked for later from being replaced by this one.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "show_level")]
// Tauri injects each managed state as an argument of its own.
#[allow(clippy::too_many_arguments)]
pub async fn levels_show_level(
  shows: State<'_, ViewportShows>,
  shown: State<'_, ShownLevels>,
  reloads: State<'_, TextureReloads>,
  levels: State<'_, SelectedLevels>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
  session_id: Option<SessionId>,
) -> TauriResult<Option<u32>> {
  let ticket: u64 = shows.ask(viewport);
  let source: Option<Arc<dyn RenderLevelSource>> = match session_id {
    Some(session_id) => {
      let level: Arc<SessionSnapshot<SelectedLevel>> = levels.require(session_id)?;

      reloads.watch(&level.roots);
      LevelReport::showing(&level.source, viewport);

      let shown: ShownLevels = ShownLevels::clone(&shown);
      let assets: AssetMountState = AssetMountState::clone(&assets);
      let source: Arc<LevelRenderSource> = execution
        .run_blocking("Showing the level", move || shown.get_source(level, &assets))
        .await?;

      Some(source)
    }
    None => None,
  };

  Ok(shows.show(viewport, ticket, source).map(|scene| scene.0))
}
