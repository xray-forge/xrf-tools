use std::sync::Arc;
use std::time::Instant;

use tauri::State;
use xrf_app_core::assets::{AssetMountState, detect_search_engine};
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionDerived, SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_dltx::select_ltx_dialect;
use xrf_ltx::LtxDialect;
use xrf_vfs::{XrayRoots, XraySearch};

use crate::level_report::LevelReport;
use crate::opened_level::OpenedLevel;
use crate::parts::{LevelSurfacesPart, LevelTexturesPart};
use crate::request::LevelOpenRequest;
use crate::state::{LevelState, SelectedLevel, SelectedLevelDescription};

/// Select a compiled level and report what it is built out of, without reading any of its geometry.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_level"))]
#[tauri::command(rename = "open_level")]
pub async fn levels_open_level(
  session_id: SessionId,
  request: LevelOpenRequest,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<SelectedLevelDescription>> {
  let LevelOpenRequest {
    source,
    roots,
    is_dltx,
    engine,
  } = request;

  state.levels.begin_open(session_id)?;

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
  let level: &SelectedLevel = &opened.value;

  log::info!(
    "Reading level '{}' as {}",
    level.source.get_label(),
    level.engine.describe()
  );

  let committed: Arc<SessionSnapshot<SelectedLevel>> = state.levels.commit_open_with(
    session_id,
    opened.value,
    search.get_vfs().get_revision(),
    opened.inputs,
    opened.parts,
  )?;

  // What the open came to, its surfaces and textures as it resolved and kept them.
  LevelReport::opened(
    &committed,
    (
      &LevelSurfacesPart::find(&committed).unwrap_or_default(),
      &LevelTexturesPart::find(&committed).unwrap_or_default(),
    ),
    started,
  );

  let selected: Arc<SessionSnapshot<SelectedLevel>> = state.levels.catch_up(committed, &assets);

  // The level itself reads nothing to describe; the page asks for each part it shows apart from it.
  Ok(selected.answer(selected.describe()))
}
