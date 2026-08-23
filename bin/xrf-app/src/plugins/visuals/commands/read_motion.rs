use std::sync::MutexGuard;

use tauri::State;
use tauri::ipc::Response;

use crate::core::types::TauriResult;
use crate::plugins::visuals::state::{SelectedVisual, VisualState};

/// The baked joint positions of the posed motion, as bytes.
///
/// Frame major `f32` triples: frame zero's bones, then frame one's. The frame and bone counts to read them by came
/// back from `open_motion`, the same pairing geometry uses.
///
/// Serves only what `open_motion` parked, and is named by that motion rather than taking one: composing here instead
/// would let a caller ask for bytes whose shape no description had reported.
#[tauri::command(rename = "read_motion")]
pub async fn visuals_read_motion(name: String, state: State<'_, VisualState>) -> TauriResult<Response> {
  let selected: MutexGuard<Option<SelectedVisual>> = state
    .selected
    .lock()
    .map_err(|error| format!("Failed to read motion - selection state is unavailable: {error}"))?;

  let Some(posed) = selected.as_ref().and_then(|it| it.posed.as_ref()) else {
    return Err(String::from("No motion is posed to read"));
  };

  if posed.description.name != name {
    return Err(format!(
      "Motion '{name}' is not the posed one: '{}' is",
      posed.description.name
    ));
  }

  let bytes: Vec<u8> = posed.positions.iter().flat_map(|it| it.to_le_bytes()).collect();

  log::info!(
    "Serving {} bytes of posed motion '{name}', {} frames of {} bones",
    bytes.len(),
    posed.description.frame_count,
    posed.description.bone_count
  );

  Ok(Response::new(bytes))
}
