use tauri::State;

use xrf_audio::AudioTransportId;

use crate::preview::PreviewTransports;

/// Stop and forget the preview, as its player goes; one already closed is left alone.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "close_preview")]
pub fn audio_close_preview(transports: State<'_, PreviewTransports>, transport: AudioTransportId) {
  transports.close(transport);
}
