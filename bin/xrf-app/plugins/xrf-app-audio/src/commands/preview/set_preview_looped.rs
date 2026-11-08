use tauri::State;
use xrf_app_core::TauriResult;

use xrf_audio::AudioTransportId;

use crate::preview::PreviewTransports;

/// Whether the preview starts over at its end.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "set_preview_looped")]
pub(crate) fn audio_set_preview_looped(
  transports: State<'_, PreviewTransports>,
  transport: AudioTransportId,
  is_looped: bool,
) -> TauriResult<()> {
  transports.set_looped(transport, is_looped)
}
