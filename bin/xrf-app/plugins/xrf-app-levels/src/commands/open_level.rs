use std::sync::Arc;
use std::time::Instant;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::{AssetMountState, detect_search_engine};
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_dltx::DltxDialect;
use xrf_ltx::LtxDialect;
use xrf_vfs::XrayRoots;

use crate::level::{LevelOpenRequest, OpenedLevel, SelectedLevel, SelectedLevelDescription, SelectedLevels};
use crate::parts::{LevelSurfacesPart, LevelTexturesPart};
use crate::report::LevelReport;

/// Select a compiled level and report what it is built out of, without reading any of its geometry.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "open_level")]
pub async fn levels_open_level(
  session_id: SessionId,
  request: LevelOpenRequest,
  levels: State<'_, SelectedLevels>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<SelectedLevelDescription>> {
  let LevelOpenRequest {
    source,
    roots,
    is_dltx,
    engine,
  } = request;

  let started: Instant = Instant::now();

  LevelReport::opening(&source);

  let search_roots: XrayRoots = roots.centred_on(source.get_physical_path());
  let roots: XrayRoots = search_roots.clone();
  let dialect: Arc<dyn LtxDialect> = DltxDialect::select(is_dltx);
  let selected: Arc<SessionSnapshot<SelectedLevel>> = levels
    .open_followed(
      &execution,
      &assets,
      session_id,
      search_roots,
      "Opening the level",
      move |search| {
        OpenedLevel::read(search, source, roots, dialect, |search, roots| {
          engine.resolve(|| detect_search_engine(search, roots))
        })
      },
    )
    .await?;

  // What the open came to: the level as read, then its surfaces and textures as it resolved and kept them.
  LevelReport::reading(&selected.value.source, &selected.value.engine);
  LevelReport::opened(
    &selected,
    &LevelSurfacesPart::find(&selected).unwrap_or_default(),
    &LevelTexturesPart::find(&selected).unwrap_or_default(),
    started,
  );

  // The level itself reads nothing to describe; the page asks for each part it shows apart from it.
  Ok(selected.map(SelectedLevel::describe))
}
