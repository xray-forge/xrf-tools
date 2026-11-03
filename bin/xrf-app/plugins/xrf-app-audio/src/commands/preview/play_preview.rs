use tauri::State;
use xrf_app_core::TauriResult;

use xrf_audio::AudioTransportId;

use crate::preview::PreviewTransports;

/// Play the preview from where it holds, or from the start once it has ended.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "play_preview")]
pub fn audio_play_preview(transports: State<'_, PreviewTransports>, transport: AudioTransportId) -> TauriResult<()> {
  transports.play(transport)
}
