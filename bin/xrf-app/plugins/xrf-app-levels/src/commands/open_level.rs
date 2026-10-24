use std::sync::Arc;
use std::time::Instant;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::{AssetMountState, detect_search_engine};
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionDerived, SessionId, SessionOpening, SessionSnapshot};
use xrf_dltx::select_ltx_dialect;
use xrf_ltx::LtxDialect;
use xrf_vfs::{XrayRoots, XraySearch};

use crate::level::{LevelOpenRequest, OpenedLevel, SelectedLevel, SelectedLevelDescription, SelectedLevels};
use crate::level_report::LevelReport;
use crate::parts::{LevelSurfacesPart, LevelTexturesPart};

/// Select a compiled level and report what it is built out of, without reading any of its geometry.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_level"))]
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

  let opening: SessionOpening<SelectedLevel> = levels.begin_open(session_id)?;

  let started: Instant = Instant::now();

  LevelReport::opening(&source);

  let roots: XrayRoots = roots.centred_on(source.get_physical_path());
  let dialect: Arc<dyn LtxDialect> = select_ltx_dialect(is_dltx);
  let reader: AssetMountState = AssetMountState::clone(&assets);
  let (opened, search) = execution
    .run_blocking("Opening the level", move || {
      let search: XraySearch = reader.search_fresh(&roots)?;
      let opened: SessionDerived<SelectedLevel> =
        OpenedLevel::read(search.clone(), source, roots, dialect, |search, roots| {
          engine.resolve(|| detect_search_engine(search, roots))
        })?;

      TauriResult::Ok((opened, search))
    })
    .await??;

  LevelReport::reading(&opened.value.source, &opened.value.engine);

  // Committed at the search's revision, and caught up with any change published since.
  let selected: Arc<SessionSnapshot<SelectedLevel>> = opening.commit_read(opened, &search, &assets)?;

  // What the open came to, its surfaces and textures as it resolved and kept them.
  LevelReport::opened(
    &selected,
    &LevelSurfacesPart::find(&selected).unwrap_or_default(),
    &LevelTexturesPart::find(&selected).unwrap_or_default(),
    started,
  );

  // The level itself reads nothing to describe; the page asks for each part it shows apart from it.
  Ok(selected.map(SelectedLevel::describe))
}
