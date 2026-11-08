use tauri::State;
use xrf_app_core::TauriResult;

use xrf_audio::AudioTransportId;

use crate::preview::PreviewTransports;

/// Hold the preview where it plays now.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "pause_preview")]
pub(crate) fn audio_pause_preview(
  transports: State<'_, PreviewTransports>,
  transport: AudioTransportId,
) -> TauriResult<()> {
  transports.pause(transport)
}
