use std::sync::Arc;
use std::time::Instant;

use tauri::State;
use xrf_dltx::select_ltx_dialect;
use xrf_ltx::LtxDialect;
use xrf_vfs::{XrayRoots, XraySearch};

use crate::core::assets::{AssetMountState, detect_search_engine};
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionDerived, SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::level_read::{describe_level, read_level};
use crate::plugins::levels::report::{report_open, report_opening};
use crate::plugins::levels::request::LevelOpenRequest;
use crate::plugins::levels::state::{LevelState, SelectedLevel, SelectedLevelDescription};

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

  state.selected.begin_open(session_id)?;

  let started: Instant = Instant::now();

  report_opening(&source);

  let roots: XrayRoots = roots.centred_on(source.get_physical_path());
  let dialect: Arc<dyn LtxDialect> = select_ltx_dialect(is_dltx);
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let (opened, search) = execution
    .run_blocking("Opening the level", move || {
      let search: XraySearch = assets.search_fresh(&roots)?;
      let opened: SessionDerived<SelectedLevel> =
        read_level(search.clone(), source, roots, dialect, |search, roots| {
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

  let selected: Arc<SessionSnapshot<SelectedLevel>> =
    state
      .selected
      .commit_open_with(session_id, opened.value, opened.inputs, opened.parts)?;

  report_open(&selected, started);

  // The start was decided as the level opened, so describing it reads nothing.
  Ok(describe_level(&selected, &search))
}
