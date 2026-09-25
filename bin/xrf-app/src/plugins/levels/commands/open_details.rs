use std::sync::Arc;
use std::time::Instant;

use tauri::State;

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::details::{PackedLevelDetails, pack_details};
use crate::plugins::levels::state::{LevelDetailsDescription, LevelState, SelectedLevel};

/// Pack the open level's grass and describe it, or answer nothing for a level with no detail library.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_details"))]
#[tauri::command(rename = "open_details")]
pub async fn levels_open_details(
  session_id: SessionId,
  details_id: SessionId,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
) -> TauriResult<SessionSnapshot<Option<LevelDetailsDescription>>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let started: Instant = Instant::now();
  let directory: Option<String> = current.source.get_logical_directory();
  let packed: Option<PackedLevelDetails> = assets.with_probe(&current.roots, |probe| {
    pack_details(&current.source, probe, directory.as_deref())
  })??;

  let Some(packed) = packed else {
    log::info!("Level {} has no detail library", current.source.get_label());

    return Ok(SessionSnapshot {
      session_id: details_id,
      value: None,
    });
  };

  let description = &packed.package.description;

  log::info!(
    "Packed grass of {}: {} planted slots, {} triangles, {} bin entries, {} models, {} in {:?}",
    current.source.get_label(),
    description.slot_count,
    description.triangle_count,
    description.bin_length,
    description.models.len(),
    xrf_utils::format_bytes(u64::from(description.buffer_length)),
    started.elapsed()
  );

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
