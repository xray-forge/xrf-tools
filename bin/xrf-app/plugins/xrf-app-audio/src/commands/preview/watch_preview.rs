use tauri::State;
use tauri::ipc::Channel;
use xrf_app_core::TauriResult;
use xrf_audio::AudioTransportState;

use xrf_audio::AudioTransportId;

use crate::preview::PreviewTransports;

/// Send the preview's state to `states` now and whenever it changes, its position included, until it closes.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "watch_preview")]
pub fn audio_watch_preview(
  transports: State<'_, PreviewTransports>,
  transport: AudioTransportId,
  states: Channel<AudioTransportState>,
) -> TauriResult<()> {
  transports.watch(transport, states)
}
