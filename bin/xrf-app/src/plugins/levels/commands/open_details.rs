use std::sync::Arc;
use std::time::Instant;

use tauri::State;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::details::{PackedLevelDetails, pack_details};
use crate::plugins::levels::report::report_details;
use crate::plugins::levels::state::{LevelDetailsDescription, LevelState, SelectedLevel};

/// Pack the open level's grass and describe it, or answer nothing for a level with no detail library.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_details"))]
#[tauri::command(rename = "open_details")]
pub async fn levels_open_details(
  session_id: SessionId,
  details_id: SessionId,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Option<LevelDetailsDescription>>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let started: Instant = Instant::now();
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let read: Arc<SessionSnapshot<SelectedLevel>> = Arc::clone(&current);
  let packed: Option<PackedLevelDetails> = execution
    .run_blocking("Packing the level grass", move || {
      let directory: Option<String> = read.source.get_logical_directory();

      assets.with_probe(&read.roots, |probe| {
        pack_details(&read.source, probe, directory.as_deref())
      })
    })
    .await???;

  report_details(
    &current.source,
    packed.as_ref().map(|it| &it.package.description),
    started,
  );

  let Some(packed) = packed else {
    return Ok(SessionSnapshot {
      session_id: details_id,
      value: None,
    });
  };

  let value: LevelDetailsDescription = LevelDetailsDescription {
    details: packed.package.description,
    surfaces: packed.surfaces,
    textures: packed.textures,
  };

  current.details.park(details_id, packed.package.buffer)?;

  Ok(SessionSnapshot {
    session_id: details_id,
    value: Some(value),
  })
}
