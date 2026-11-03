use tauri::State;
use xrf_app_core::TauriResult;

use crate::preview::{AudioTransportId, PreviewTransports};

/// Put the preview back at its start, silent.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "stop_preview")]
pub fn audio_stop_preview(transports: State<'_, PreviewTransports>, transport: AudioTransportId) -> TauriResult<()> {
  transports.stop(transport)
}
