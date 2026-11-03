use tauri::State;
use xrf_app_core::TauriResult;

use crate::preview::{AudioTransportId, PreviewTransports};

/// Hold the preview where it plays now.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "pause_preview")]
pub fn audio_pause_preview(transports: State<'_, PreviewTransports>, transport: AudioTransportId) -> TauriResult<()> {
  transports.pause(transport)
}
