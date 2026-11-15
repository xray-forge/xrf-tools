use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_render::{TextureReloads, ViewportShows};
use xrf_renderer::RenderViewportId;
use xrf_world::WorldLevelSource;

use crate::report::LevelReport;
use crate::scene::{SceneRenderSource, SelectedScene, SelectedScenes, ShownScenes};

/// Draw the open scene in a viewport, each scene object a model compiled from its `.object` as the renderer asks, its
/// textures read from the scene's game data; no session draws none. Answers the scene drawn, which the viewport's load
/// reports name, or none. The source is made on the pool, after the ticket that keeps a scene asked for later from being
/// replaced by this one.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "show_scene")]
#[expect(
  clippy::too_many_arguments,
  reason = "Tauri injects each managed state as an argument of its own"
)]
pub(crate) async fn levels_show_scene(
  shows: State<'_, ViewportShows>,
  shown: State<'_, ShownScenes>,
  reloads: State<'_, TextureReloads>,
  scenes: State<'_, SelectedScenes>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
  session_id: Option<SessionId>,
) -> TauriResult<Option<u32>> {
  let ticket: u64 = shows.ask(viewport);
  let source: Option<Arc<dyn WorldLevelSource>> = match session_id {
    Some(session_id) => {
      let scene: Arc<SessionSnapshot<SelectedScene>> = scenes.require(session_id)?;

      reloads.watch(&scene.game.roots);
      LevelReport::showing_scene(&scene, viewport);

      let shown: ShownScenes = ShownScenes::clone(&shown);
      let assets: AssetMountState = AssetMountState::clone(&assets);
      let source: Arc<SceneRenderSource> = execution
        .run_blocking("Showing the scene", move || shown.get_source(scene, &assets))
        .await?;

      Some(source)
    }
    None => None,
  };

  Ok(shows.show(viewport, ticket, source).map(|scene| scene.0))
}
