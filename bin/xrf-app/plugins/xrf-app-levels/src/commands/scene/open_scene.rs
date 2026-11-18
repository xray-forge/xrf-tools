use std::sync::Arc;
use std::time::Instant;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_vfs::XrayRoots;

use crate::report::LevelReport;
use crate::scene::{
  OpenedScene, SceneGameData, SceneOpenRequest, SelectedScene, SelectedSceneDescription, SelectedScenes,
};

/// Open a Level Editor scene from its `.level`, listing its tools' part files and what the editor would skip or refuse,
/// without reading their objects.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "open_scene")]
pub(crate) async fn levels_open_scene(
  session_id: SessionId,
  request: SceneOpenRequest,
  scenes: State<'_, SelectedScenes>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<SelectedSceneDescription>> {
  let SceneOpenRequest {
    source,
    objects_roots,
    game_roots,
    is_dltx,
    engine,
  } = request;

  let started: Instant = Instant::now();

  LevelReport::opening_scene(&source);

  let roots: XrayRoots = source.get_roots()?;
  let scene: OpenedScene = OpenedScene {
    objects_roots: source.get_objects_roots(objects_roots),
    game: SceneGameData {
      roots: source.get_game_roots(game_roots),
      is_dltx,
      engine,
    },
    source,
  };
  let opened: Arc<SessionSnapshot<SelectedScene>> = scenes
    .open_followed(
      &execution,
      &assets,
      session_id,
      roots,
      "Opening the scene",
      move |search| scene.read(search),
    )
    .await?;

  LevelReport::opened_scene(&opened, started);

  // The page asks for each part's description separately.
  Ok(opened.map(SelectedScene::describe))
}
